//! Eigene Seite eines Mitarbeiters: laufende Ladungen mit Live-Werten, Timer
//! und Ziel-kWh, dazu die persoenlichen Standardvorgaben und die letzten
//! eigenen Ladungen. Ein Mitarbeiter sieht hier ausschliesslich, was ueber
//! einen ihm zugeordneten Chip gelaufen ist.

use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};

use super::users::{recent_transactions, RecentTx};
use super::{render, LayoutCtx};
use crate::auth::AuthUser;
use crate::domain::transaction::{fmt_kw, fmt_kwh, live_meter};
use crate::domain::user::User;
use crate::i18n::Lang;
use crate::{AppResult, AppState};

pub struct MySession {
    pub tx_id: i64,
    pub wallbox_name: String,
    pub connector_id: i64,
    pub start_time: String,
    pub energy_kwh: Option<String>,
    pub power_kw: Option<String>,
    pub soc_percent: Option<i64>,
    /// Gesetztes Energielimit als kWh-Text, leer wenn keines gesetzt ist.
    pub limit_kwh: String,
    /// Verbleibende Minuten bis zur Zeitabschaltung, falls ein Timer laeuft.
    pub limit_minutes_left: Option<i64>,
    /// 1 = wegen Energielimit gestoppt, 2 = wegen Zeitlimit — die Wallbox
    /// beendet die Ladung gleich, die Zeile bleibt bis dahin sichtbar.
    pub limit_stopped: i64,
}

impl MySession {
    pub fn stopped_by_energy(&self) -> bool {
        self.limit_stopped == crate::ocpp::limits::STOPPED_BY_ENERGY
    }
    pub fn stopped_by_time(&self) -> bool {
        self.limit_stopped == crate::ocpp::limits::STOPPED_BY_TIME
    }
}

#[derive(Template)]
#[template(path = "me.html")]
struct MeTpl {
    layout: LayoutCtx,
    me: User,
    sessions: Vec<MySession>,
    recent_tx: Vec<RecentTx>,
    total_wh: i64,
    session_count: i64,
    lang: Lang,
}

#[derive(Template)]
#[template(path = "_my_sessions.html")]
struct MySessionsTpl {
    sessions: Vec<MySession>,
    lang: Lang,
}

pub async fn get(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    lang: Lang,
) -> AppResult<Response> {
    let sessions = load_my_sessions(&state, user.id).await?;
    let recent_tx = recent_transactions(&state, user.id).await?;
    let (session_count, total_wh): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),
                COALESCE(SUM(CASE WHEN stop_meter_wh IS NOT NULL
                                  THEN stop_meter_wh - start_meter_wh ELSE 0 END), 0)
         FROM transactions WHERE user_id = ?1",
    )
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;

    Ok(render(&MeTpl {
        layout: LayoutCtx::new("me", Some(user.clone()), lang),
        me: user,
        sessions,
        recent_tx,
        total_wh: total_wh.max(0),
        session_count,
        lang,
    })?
    .into_response())
}

/// htmx-Fragment: die eigenen laufenden Ladungen, alle paar Sekunden nachgeladen.
pub async fn live_fragment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    lang: Lang,
) -> AppResult<Response> {
    let sessions = load_my_sessions(&state, user.id).await?;
    Ok(render(&MySessionsTpl { sessions, lang })?.into_response())
}

type SessionRow = (
    i64,
    String,
    i64,
    String,
    i64,
    Option<i64>,
    Option<String>,
    i64,
);

async fn load_my_sessions(state: &AppState, user_id: i64) -> AppResult<Vec<MySession>> {
    let rows: Vec<SessionRow> = sqlx::query_as(
        "SELECT t.id, w.name, t.connector_id, t.start_time, t.start_meter_wh,
                t.limit_wh, t.limit_until, t.limit_stopped
         FROM transactions t
         JOIN wallboxes w ON w.id = t.wallbox_id
         WHERE t.stop_time IS NULL AND t.user_id = ?1
         ORDER BY t.start_time DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let now = Utc::now();
    let mut out = Vec::with_capacity(rows.len());
    for (tx_id, wb, conn, start, start_wh, limit_wh, limit_until, limit_stopped) in rows {
        let live = live_meter(&state.db, tx_id, start_wh).await?;
        out.push(MySession {
            tx_id,
            wallbox_name: wb,
            connector_id: conn,
            start_time: start,
            energy_kwh: live.energy_wh.map(fmt_kwh),
            power_kw: live.power_w.map(fmt_kw),
            soc_percent: live.soc_percent,
            limit_kwh: limit_wh
                .map(|wh| format!("{:.1}", wh as f64 / 1000.0))
                .unwrap_or_default(),
            limit_minutes_left: limit_until.as_deref().and_then(|s| minutes_left(s, now)),
            limit_stopped,
        });
    }
    Ok(out)
}

/// Verbleibende Minuten bis zum Abschaltzeitpunkt, aufgerundet. Ein bereits
/// verstrichener Zeitpunkt ergibt 0 — nicht None, damit die UI weiter „0 min“
/// statt „kein Timer“ zeigt, solange die Wallbox noch nicht gestoppt hat.
fn minutes_left(until: &str, now: DateTime<Utc>) -> Option<i64> {
    let dt = DateTime::parse_from_rfc3339(until).ok()?;
    let secs = (dt.with_timezone(&Utc) - now).num_seconds();
    Some(if secs <= 0 { 0 } else { (secs + 59) / 60 })
}
