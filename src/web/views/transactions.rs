use std::fmt::Write as _;

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use chrono::Utc;
use serde::Deserialize;

use super::{render, LayoutCtx};
use crate::auth::AuthUser;
use crate::domain::transaction::{
    fmt_kw, fmt_kwh, live_meter, parse_kwh_to_wh, parse_minutes, Transaction,
};
use crate::domain::user::User;
use crate::domain::wallbox::ConnectorEvent;
use crate::i18n::Lang;
use crate::{AppError, AppResult, AppState};

pub struct TxRow {
    pub id: i64,
    pub wallbox_name: String,
    pub connector_id: i64,
    pub id_tag: String,
    pub user_name: Option<String>,
    pub start_time: String,
    pub stop_time: Option<String>,
    /// Geladene Energie, formatiert („12,3“). Bei laufenden Transaktionen der
    /// aktuelle Stand aus den zuletzt gemeldeten MeterValues. None, wenn
    /// (noch) keine Messung vorliegt.
    pub energy_kwh: Option<String>,
    /// Aktuelle Ladeleistung, formatiert. Nur bei laufenden Transaktionen gesetzt.
    pub power_kw: Option<String>,
}

#[derive(Template)]
#[template(path = "transactions.html")]
struct ListTpl {
    layout: LayoutCtx,
    rows: Vec<TxRow>,
    filter_user: Option<String>,
    /// Nicht-Admins sehen ausschliesslich eigene Ladungen, dann entfaellt der
    /// Filter nach Person.
    can_filter: bool,
}

#[derive(Deserialize)]
pub struct Filter {
    /// Namensfilter (nur fuer Admins).
    pub employee: Option<String>,
}

type ListRow = (
    i64,
    String,
    i64,
    String,
    Option<String>,
    String,
    Option<String>,
    Option<i64>,
    i64,
);

pub async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    lang: Lang,
    Query(filter): Query<Filter>,
) -> AppResult<Response> {
    // Nicht-Admins sehen nur die Ladungen, die ueber ihre eigenen Chips liefen.
    let mine_only = !user.is_admin();
    let select = "SELECT t.id, w.name, t.connector_id, t.id_tag, u.display_name,
                         t.start_time, t.stop_time, t.stop_meter_wh, t.start_meter_wh
                  FROM transactions t
                  JOIN wallboxes w ON w.id = t.wallbox_id
                  LEFT JOIN users u ON u.id = t.user_id";
    let rows: Vec<ListRow> = if mine_only {
        sqlx::query_as(&format!(
            "{select} WHERE t.user_id = ?1 ORDER BY t.start_time DESC LIMIT 500"
        ))
        .bind(user.id)
        .fetch_all(&state.db)
        .await?
    } else if let Some(name) = filter.employee.as_deref().filter(|s| !s.is_empty()) {
        sqlx::query_as(&format!(
            "{select} WHERE u.display_name LIKE ?1 OR t.guest_label LIKE ?1
             ORDER BY t.start_time DESC LIMIT 500"
        ))
        .bind(format!("%{name}%"))
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(&format!(
            "{select} ORDER BY t.start_time DESC LIMIT 500"
        ))
        .fetch_all(&state.db)
        .await?
    };

    let mut out = Vec::with_capacity(rows.len());
    for (id, wn, cid, tag, un, st, et, stop_m, start_m) in rows {
        // Laufende Transaktionen: aktuellen Stand aus den MeterValues holen.
        let (energy_wh, power_kw) = if et.is_none() {
            let live = live_meter(&state.db, id, start_m).await?;
            (live.energy_wh, live.power_w.map(fmt_kw))
        } else {
            (stop_m.map(|s| (s - start_m).max(0)), None)
        };
        out.push(TxRow {
            id,
            wallbox_name: wn,
            connector_id: cid,
            id_tag: tag,
            user_name: un,
            start_time: st,
            stop_time: et,
            energy_kwh: energy_wh.map(fmt_kwh),
            power_kw,
        });
    }
    let rows = out;

    Ok(render(&ListTpl {
        layout: LayoutCtx::new("transactions", Some(user), lang),
        rows,
        filter_user: filter.employee,
        can_filter: !mine_only,
    })?
    .into_response())
}

type CsvRow = (
    i64,
    String,
    i64,
    String,
    Option<String>,
    String,
    Option<String>,
    Option<i64>,
    i64,
    Option<String>,
);

pub async fn export_csv(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    lang: Lang,
    Query(filter): Query<Filter>,
) -> AppResult<Response> {
    let mine_only = !user.is_admin();
    let select = "SELECT t.id, w.name, t.connector_id, t.id_tag, u.display_name,
                         t.start_time, t.stop_time, t.stop_meter_wh, t.start_meter_wh,
                         t.stop_reason
                  FROM transactions t
                  JOIN wallboxes w ON w.id = t.wallbox_id
                  LEFT JOIN users u ON u.id = t.user_id";
    let rows: Vec<CsvRow> = if mine_only {
        sqlx::query_as(&format!(
            "{select} WHERE t.user_id = ?1 ORDER BY t.start_time DESC"
        ))
        .bind(user.id)
        .fetch_all(&state.db)
        .await?
    } else if let Some(name) = filter.employee.as_deref().filter(|s| !s.is_empty()) {
        sqlx::query_as(&format!(
            "{select} WHERE u.display_name LIKE ?1 OR t.guest_label LIKE ?1
             ORDER BY t.start_time DESC"
        ))
        .bind(format!("%{name}%"))
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(&format!("{select} ORDER BY t.start_time DESC"))
            .fetch_all(&state.db)
            .await?
    };

    let mut out = String::with_capacity(256 + rows.len() * 120);
    out.push('\u{FEFF}');
    out.push_str(lang.t("csv.header"));
    out.push('\n');
    for (id, wname, cid, tag, uname, st, et, stop_m, start_m, reason) in rows {
        // Laufende Transaktionen: aktuellen Stand aus den MeterValues exportieren,
        // konsistent zur HTML-Liste.
        let energy = if et.is_none() {
            live_meter(&state.db, id, start_m)
                .await?
                .energy_wh
                .unwrap_or(0)
        } else {
            stop_m.map(|s| (s - start_m).max(0)).unwrap_or(0)
        };
        let _ = writeln!(
            out,
            "{id};{w};{cid};{tag};{u};{st};{et};{energy};{reason}",
            w = csv_escape(&wname),
            tag = csv_escape(&tag),
            u = csv_escape(uname.as_deref().unwrap_or("")),
            et = et.as_deref().unwrap_or(""),
            reason = csv_escape(reason.as_deref().unwrap_or("")),
        );
    }

    let filename = format!(
        "{}_{}.csv",
        lang.t("csv.filename"),
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let mut resp = (StatusCode::OK, out).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")).unwrap(),
    );
    Ok(resp)
}

// -----------------------------------------------------------------------------
// Laufende Ladung verwalten: Limits setzen, vorzeitig stoppen
// -----------------------------------------------------------------------------

/// Laedt eine Transaktion und prueft, ob der angemeldete Benutzer sie verwalten
/// darf: der Admin jede, ein Mitarbeiter nur die eigenen.
async fn own_transaction(state: &AppState, user: &User, tx_id: i64) -> AppResult<Transaction> {
    let tx: Transaction = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?1")
        .bind(tx_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    if !user.is_admin() && tx.user_id != Some(user.id) {
        return Err(AppError::Forbidden);
    }
    Ok(tx)
}

#[derive(Deserialize)]
pub struct LimitForm {
    /// Ziel-Energie in kWh; leer oder 0 hebt das Energielimit auf.
    pub limit_kwh: Option<String>,
    /// Restlaufzeit ab jetzt in Minuten; leer oder 0 hebt den Timer auf.
    pub limit_minutes: Option<String>,
    /// Wohin nach dem Speichern zurueckgesprungen wird ("/me" oder "/transactions").
    pub back: Option<String>,
}

/// Timer und Ziel-kWh einer laufenden Ladung setzen oder aufheben.
pub async fn set_limit(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    lang: Lang,
    axum::Form(form): axum::Form<LimitForm>,
) -> AppResult<Response> {
    let tx = own_transaction(&state, &user, id).await?;
    if !tx.is_running() {
        return Err(AppError::BadRequest(lang.t("err.tx_not_running").into()));
    }

    let wh = parse_kwh_to_wh(form.limit_kwh.as_deref())
        .map_err(|_| AppError::BadRequest(lang.t("err.limit_kwh").into()))?;
    let minutes = parse_minutes(form.limit_minutes.as_deref())
        .map_err(|_| AppError::BadRequest(lang.t("err.limit_minutes").into()))?;
    // Die Restlaufzeit zaehlt ab jetzt, nicht ab Ladebeginn. Das entspricht
    // dem, was an einer laufenden Ladung gemeint ist ("noch 90 Minuten").
    let until = minutes.map(|m| (Utc::now() + chrono::Duration::minutes(m)).to_rfc3339());

    sqlx::query(
        "UPDATE transactions
            SET limit_wh = ?1, limit_until = ?2, limit_stopped = 0
          WHERE id = ?3",
    )
    .bind(wh)
    .bind(&until)
    .bind(id)
    .execute(&state.db)
    .await?;

    // Ein bereits ueberschrittenes Limit soll nicht erst beim naechsten Takt greifen.
    crate::ocpp::limits::enforce_transaction(&state, id).await;

    Ok(Redirect::to(back_target(form.back.as_deref())).into_response())
}

#[derive(Deserialize)]
pub struct StopForm {
    pub back: Option<String>,
}

/// Eigene laufende Ladung vorzeitig beenden.
pub async fn stop(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    lang: Lang,
    axum::Form(form): axum::Form<StopForm>,
) -> AppResult<Response> {
    let tx = own_transaction(&state, &user, id).await?;
    if !tx.is_running() {
        return Err(AppError::BadRequest(lang.t("err.tx_not_running").into()));
    }

    let (cp_id,): (String,) =
        sqlx::query_as("SELECT charge_point_id FROM wallboxes WHERE id = ?1")
            .bind(tx.wallbox_id)
            .fetch_one(&state.db)
            .await?;
    let ocpp_tx_id = tx.ocpp_transaction_id.unwrap_or(tx.id);

    let status = crate::ocpp::ocpp16::remote_stop(&state.ocpp_hub, &cp_id, ocpp_tx_id)
        .await
        .map_err(|e| AppError::Conflict(format!("RemoteStop: {e}")))?;
    if status != "Accepted" {
        return Err(AppError::Conflict(format!(
            "{} {status}",
            lang.t("err.remote_stop_rejected")
        )));
    }
    Ok(Redirect::to(back_target(form.back.as_deref())).into_response())
}

/// Nur die zwei bekannten Ziele zulassen. Ein freies Redirect-Feld waere ein
/// offener Redirect.
fn back_target(back: Option<&str>) -> &'static str {
    match back {
        Some("transactions") => "/transactions",
        _ => "/me",
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains([';', '"', '\n', '\r']) {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

// -----------------------------------------------------------------------------
// Detailseite einer Ladung mit den Meldungen der Wallbox aus dem Zeitraum
// -----------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "transaction_detail.html")]
struct DetailTpl {
    layout: LayoutCtx,
    tx: Transaction,
    wallbox_name: String,
    user_name: Option<String>,
    energy_kwh: Option<String>,
    power_kw: Option<String>,
    events: Vec<ConnectorEvent>,
    lang: Lang,
}

/// Eine einzelne Ladung im Detail. Ein Mitarbeiter sieht nur die eigenen,
/// der Admin alle.
pub async fn detail(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    lang: Lang,
) -> AppResult<Response> {
    let tx = own_transaction(&state, &user, id).await?;

    let (wallbox_name,): (String,) =
        sqlx::query_as("SELECT name FROM wallboxes WHERE id = ?1")
            .bind(tx.wallbox_id)
            .fetch_one(&state.db)
            .await?;

    let user_name: Option<String> = match tx.user_id {
        Some(uid) => sqlx::query_as::<_, (String,)>("SELECT display_name FROM users WHERE id = ?1")
            .bind(uid)
            .fetch_optional(&state.db)
            .await?
            .map(|(n,)| n),
        None => tx.guest_label.clone(),
    };

    // Laufende Ladung: aktueller Stand aus den MeterValues, sonst der Endstand.
    let (energy_wh, power_kw) = if tx.is_running() {
        let live = live_meter(&state.db, tx.id, tx.start_meter_wh).await?;
        (live.energy_wh, live.power_w.map(fmt_kw))
    } else {
        (tx.energy_wh(), None)
    };

    let events = crate::domain::wallbox::events_for_session(
        &state.db,
        tx.wallbox_id,
        tx.connector_id,
        &tx.start_time,
        tx.stop_time.as_deref(),
    )
    .await?;

    Ok(render(&DetailTpl {
        layout: LayoutCtx::new("transactions", Some(user), lang),
        tx,
        wallbox_name,
        user_name,
        energy_kwh: energy_wh.map(fmt_kwh),
        power_kw,
        events,
        lang,
    })?
    .into_response())
}
