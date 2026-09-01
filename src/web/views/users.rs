//! Benutzerverwaltung. Seit Migration 0003 ist ein Benutzer zugleich der
//! Mitarbeiter — Chips, Ladungen und Ladelimits hängen direkt am `users`-Eintrag.

use askama::Template;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use serde::Deserialize;

use super::{render, LayoutCtx};
use crate::auth::{AdminUser, AuthUser};
use crate::db::hash_password;
use crate::domain::chip::Chip;
use crate::domain::transaction::{parse_kwh_to_wh, parse_minutes};
use crate::domain::user::User;
use crate::i18n::Lang;
use crate::{AppError, AppResult, AppState};

pub struct UserRow {
    pub user: User,
    pub chip_count: i64,
    pub tx_count: i64,
    pub total_wh: i64,
}

#[derive(Template)]
#[template(path = "users.html")]
struct ListTpl {
    layout: LayoutCtx,
    users: Vec<UserRow>,
}

pub async fn list(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    lang: Lang,
) -> AppResult<Response> {
    let users: Vec<User> = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY display_name")
        .fetch_all(&state.db)
        .await?;

    // Rollup je Benutzer: Chips, Anzahl Ladungen, geladene Energie.
    let mut rows = Vec::with_capacity(users.len());
    for user in users {
        let (chip_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chips WHERE user_id = ?1")
                .bind(user.id)
                .fetch_one(&state.db)
                .await?;
        let (tx_count, total_wh): (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN stop_meter_wh IS NOT NULL
                                      THEN stop_meter_wh - start_meter_wh ELSE 0 END), 0)
             FROM transactions WHERE user_id = ?1",
        )
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;
        rows.push(UserRow {
            user,
            chip_count,
            tx_count,
            total_wh: total_wh.max(0),
        });
    }

    Ok(render(&ListTpl {
        layout: LayoutCtx::new("users", Some(user), lang),
        users: rows,
    })?
    .into_response())
}

#[derive(Deserialize)]
pub struct CreateForm {
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub department: Option<String>,
    pub role: String,
    /// Leer = Konto ohne Login (der Mitarbeiter kann sich noch nicht anmelden).
    pub password: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    lang: Lang,
    Form(form): Form<CreateForm>,
) -> AppResult<Response> {
    let username = form.username.trim();
    let display = form.display_name.trim();
    if username.is_empty() || display.is_empty() {
        return Err(AppError::BadRequest(
            lang.t("err.user_fields_required").into(),
        ));
    }
    if form.role != "admin" && form.role != "user" {
        return Err(AppError::BadRequest(lang.t("err.invalid_role").into()));
    }
    let password = form.password.as_deref().unwrap_or("");
    let hash = if password.is_empty() {
        None
    } else if password.len() < 6 {
        return Err(AppError::BadRequest(lang.t("err.pw_min6").into()));
    } else {
        Some(hash_password(password).map_err(AppError::Other)?)
    };

    let res = sqlx::query(
        "INSERT INTO users (username, display_name, email, department, role, auth_source,
                            password_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, 'local', ?6)",
    )
    .bind(username)
    .bind(display)
    .bind(opt(form.email.as_deref()))
    .bind(opt(form.department.as_deref()))
    .bind(&form.role)
    .bind(hash)
    .execute(&state.db)
    .await;
    if let Err(sqlx::Error::Database(db)) = &res {
        if db.is_unique_violation() {
            return Err(AppError::Conflict(format!(
                "{} '{username}'",
                lang.t("err.user_exists")
            )));
        }
    }
    res?;
    Ok(Redirect::to("/users").into_response())
}

pub struct RecentTx {
    pub start_time: String,
    pub stop_time: Option<String>,
    pub wallbox: String,
    pub id_tag: String,
    pub energy_wh: i64,
}

#[derive(Template)]
#[template(path = "user_detail.html")]
struct DetailTpl {
    layout: LayoutCtx,
    /// Der angezeigte Benutzer — nicht zwingend der angemeldete.
    subject: User,
    chips: Vec<Chip>,
    recent_tx: Vec<RecentTx>,
}

pub async fn detail(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Path(id): Path<i64>,
    lang: Lang,
) -> AppResult<Response> {
    let subject: User = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let chips: Vec<Chip> =
        sqlx::query_as::<_, Chip>("SELECT * FROM chips WHERE user_id = ?1 ORDER BY created_at DESC")
            .bind(id)
            .fetch_all(&state.db)
            .await?;

    let recent_tx = recent_transactions(&state, id).await?;

    Ok(render(&DetailTpl {
        layout: LayoutCtx::new("users", Some(user), lang),
        subject,
        chips,
        recent_tx,
    })?
    .into_response())
}

pub async fn recent_transactions(state: &AppState, user_id: i64) -> AppResult<Vec<RecentTx>> {
    let rows: Vec<(String, Option<String>, String, String, Option<i64>, i64)> = sqlx::query_as(
        "SELECT t.start_time, t.stop_time, w.name, t.id_tag, t.stop_meter_wh, t.start_meter_wh
         FROM transactions t
         JOIN wallboxes w ON w.id = t.wallbox_id
         WHERE t.user_id = ?1
         ORDER BY t.start_time DESC
         LIMIT 100",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(st, et, wb, tag, stop_m, start_m)| RecentTx {
            start_time: st,
            stop_time: et,
            wallbox: wb,
            id_tag: tag,
            energy_wh: stop_m.map(|s| (s - start_m).max(0)).unwrap_or(0),
        })
        .collect())
}

#[derive(Deserialize)]
pub struct UpdateForm {
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub department: Option<String>,
    pub role: String,
    /// Checkbox "aktiv"; fehlt sie im Formular, ist das Konto deaktiviert.
    pub active: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    Path(id): Path<i64>,
    lang: Lang,
    Form(form): Form<UpdateForm>,
) -> AppResult<Response> {
    let username = form.username.trim();
    let display = form.display_name.trim();
    if username.is_empty() || display.is_empty() {
        return Err(AppError::BadRequest(
            lang.t("err.user_fields_required").into(),
        ));
    }
    if form.role != "admin" && form.role != "user" {
        return Err(AppError::BadRequest(lang.t("err.invalid_role").into()));
    }
    let disabled: i64 = if form.active.as_deref() == Some("1") { 0 } else { 1 };

    // Das System muss mindestens einen aktiven Admin behalten, sonst sperrt
    // sich die Installation aus.
    if (form.role != "admin" || disabled == 1) && is_last_active_admin(&state, id).await? {
        return Err(AppError::BadRequest(lang.t("err.last_admin").into()));
    }

    let res = sqlx::query(
        "UPDATE users
            SET username = ?1, display_name = ?2, email = ?3, department = ?4,
                role = ?5, disabled = ?6
          WHERE id = ?7",
    )
    .bind(username)
    .bind(display)
    .bind(opt(form.email.as_deref()))
    .bind(opt(form.department.as_deref()))
    .bind(&form.role)
    .bind(disabled)
    .bind(id)
    .execute(&state.db)
    .await;
    if let Err(sqlx::Error::Database(db)) = &res {
        if db.is_unique_violation() {
            return Err(AppError::Conflict(format!(
                "{} '{username}'",
                lang.t("err.user_exists")
            )));
        }
    }
    res?;
    Ok(Redirect::to(&format!("/users/{id}")).into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    Path(id): Path<i64>,
    lang: Lang,
) -> AppResult<Response> {
    if id == admin.id {
        return Err(AppError::BadRequest(lang.t("err.self_delete").into()));
    }
    if is_last_active_admin(&state, id).await? {
        return Err(AppError::BadRequest(lang.t("err.last_admin").into()));
    }
    // chips.user_id und transactions.user_id sind ON DELETE SET NULL —
    // die Ladehistorie bleibt erhalten, verliert aber ihre Zuordnung.
    sqlx::query("DELETE FROM users WHERE id = ?1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Redirect::to("/users").into_response())
}

#[derive(Deserialize)]
pub struct PwForm {
    pub password: String,
}

pub async fn set_password(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    Path(id): Path<i64>,
    lang: Lang,
    Form(form): Form<PwForm>,
) -> AppResult<Response> {
    if form.password.len() < 6 {
        return Err(AppError::BadRequest(lang.t("err.pw_min6").into()));
    }
    let hash = hash_password(&form.password).map_err(AppError::Other)?;
    sqlx::query("UPDATE users SET password_hash = ?1 WHERE id = ?2 AND auth_source = 'local'")
        .bind(hash)
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Redirect::to(&format!("/users/{id}")).into_response())
}

#[derive(Deserialize)]
pub struct DefaultsForm {
    /// Ziel-Energie in kWh; leer oder 0 = kein Standardlimit.
    pub limit_kwh: Option<String>,
    /// Ladedauer in Minuten; leer oder 0 = kein Standardlimit.
    pub limit_minutes: Option<String>,
}

/// Standard-Ladelimits eines Mitarbeiters — gesetzt vom Admin auf der Benutzerseite.
pub async fn set_defaults(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    Path(id): Path<i64>,
    lang: Lang,
    Form(form): Form<DefaultsForm>,
) -> AppResult<Response> {
    store_defaults(&state, id, &form, lang).await?;
    Ok(Redirect::to(&format!("/users/{id}")).into_response())
}

/// Dieselben Vorgaben, aber vom Mitarbeiter selbst auf seiner eigenen Seite.
pub async fn set_own_defaults(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    lang: Lang,
    Form(form): Form<DefaultsForm>,
) -> AppResult<Response> {
    store_defaults(&state, user.id, &form, lang).await?;
    Ok(Redirect::to("/me").into_response())
}

async fn store_defaults(
    state: &AppState,
    user_id: i64,
    form: &DefaultsForm,
    lang: Lang,
) -> AppResult<()> {
    let wh = parse_kwh_to_wh(form.limit_kwh.as_deref())
        .map_err(|_| AppError::BadRequest(lang.t("err.limit_kwh").into()))?;
    let minutes = parse_minutes(form.limit_minutes.as_deref())
        .map_err(|_| AppError::BadRequest(lang.t("err.limit_minutes").into()))?;
    sqlx::query("UPDATE users SET default_limit_wh = ?1, default_limit_minutes = ?2 WHERE id = ?3")
        .bind(wh)
        .bind(minutes)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// Trimmt ein Formularfeld und macht aus einem leeren Wert NULL.
fn opt(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Wäre dieser Benutzer nach einer Änderung der letzte aktive Admin?
async fn is_last_active_admin(state: &AppState, id: i64) -> AppResult<bool> {
    let (others,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE role = 'admin' AND disabled = 0 AND id <> ?1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    if others > 0 {
        return Ok(false);
    }
    let (is_admin,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE id = ?1 AND role = 'admin' AND disabled = 0",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    Ok(is_admin > 0)
}
