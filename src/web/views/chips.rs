use askama::Template;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use chrono::{Duration, Utc};
use serde::Deserialize;

use super::{render, LayoutCtx};
use crate::auth::AdminUser;
use crate::domain::chip::{Chip, EnrollmentSession};
use crate::{AppError, AppResult, AppState};

pub struct ChipRow {
    pub chip: Chip,
}

impl ChipRow {
    pub fn is_assigned_to(&self, uid: &i64) -> bool {
        self.chip.user_id == Some(*uid)
    }
}

/// Auswahleintrag "Chip gehoert zu ..." — seit 0003 ist das ein Benutzer.
pub struct UserOpt {
    pub id: i64,
    pub name: String,
}

#[derive(Template)]
#[template(path = "chips.html")]
struct ListTpl {
    layout: LayoutCtx,
    chips: Vec<ChipRow>,
    active_enrollment: Option<EnrollmentSession>,
    wallboxes: Vec<(i64, String)>,
    users: Vec<UserOpt>,
}

pub async fn list(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    lang: crate::i18n::Lang,
) -> AppResult<Response> {
    let chips: Vec<ChipRow> =
        sqlx::query_as::<_, Chip>("SELECT * FROM chips ORDER BY created_at DESC")
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|chip| ChipRow { chip })
            .collect();

    let active: Option<EnrollmentSession> = sqlx::query_as::<_, EnrollmentSession>(
        "SELECT * FROM enrollment_sessions
         WHERE started_by = ?1
           AND consumed = 0
           AND datetime(expires_at) > datetime('now')
         ORDER BY id DESC LIMIT 1",
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?;

    let wallboxes: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, name FROM wallboxes ORDER BY name")
            .fetch_all(&state.db)
            .await?;
    let user_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, display_name FROM users WHERE disabled = 0 ORDER BY display_name",
    )
    .fetch_all(&state.db)
    .await?;
    let users: Vec<UserOpt> = user_rows
        .into_iter()
        .map(|(id, name)| UserOpt { id, name })
        .collect();

    Ok(render(&ListTpl {
        layout: LayoutCtx::new("chips", Some(user), lang),
        chips,
        active_enrollment: active,
        wallboxes,
        users,
    })?
    .into_response())
}

#[derive(Deserialize)]
pub struct UpdateForm {
    /// Leer = Gast-Chip, sonst der Benutzer, dem der Chip gehört.
    pub user_id: Option<String>,
    pub label: Option<String>,
    pub enabled: Option<String>,
    pub expires_at: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    Path(id): Path<i64>,
    lang: crate::i18n::Lang,
    Form(form): Form<UpdateForm>,
) -> AppResult<Response> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM chips WHERE id = ?1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound);
    }

    // Leerer Select-Wert => Zuordnung entfernen.
    let user_id: Option<i64> = form
        .user_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse().ok());

    let label = form
        .label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    // Die Kategorie ergibt sich aus der Zuordnung — ein Chip ohne Benutzer ist
    // ein Gast-Chip. Beides getrennt pflegen zu lassen, erzeugte nur
    // widersprüchliche Kombinationen.
    let kind = chip_kind(user_id);

    let enabled: i64 = if form.enabled.as_deref() == Some("1") { 1 } else { 0 };

    let expires_at = form
        .expires_at
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    sqlx::query(
        "UPDATE chips
            SET user_id     = ?1,
                label       = ?2,
                kind        = ?3,
                enabled     = ?4,
                expires_at  = ?5
          WHERE id = ?6",
    )
    .bind(user_id)
    .bind(&label)
    .bind(kind)
    .bind(enabled)
    .bind(&expires_at)
    .bind(id)
    .execute(&state.db)
    .await?;

    Ok(Redirect::to("/chips").into_response())
}

#[derive(Deserialize)]
pub struct EnrollForm {
    pub wallbox_id: Option<i64>,
}

pub async fn enroll_start(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Form(form): Form<EnrollForm>,
) -> AppResult<Response> {
    let expires = Utc::now() + Duration::minutes(2);
    let res = sqlx::query(
        "INSERT INTO enrollment_sessions (started_by, wallbox_id, expires_at)
         VALUES (?1, ?2, ?3)",
    )
    .bind(user.id)
    .bind(form.wallbox_id)
    .bind(expires.to_rfc3339())
    .execute(&state.db)
    .await?;
    let id = res.last_insert_rowid();
    Ok(Redirect::to(&format!("/chips/enroll/{id}")).into_response())
}

#[derive(Template)]
#[template(path = "chip_enroll.html")]
struct EnrollTpl {
    layout: LayoutCtx,
    session: EnrollmentSession,
    users: Vec<(i64, String)>,
}

pub async fn enroll_poll(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Path(id): Path<i64>,
    lang: crate::i18n::Lang,
) -> AppResult<Response> {
    let sess: EnrollmentSession = sqlx::query_as::<_, EnrollmentSession>(
        "SELECT * FROM enrollment_sessions WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    let users: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, display_name FROM users WHERE disabled = 0 ORDER BY display_name",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(render(&EnrollTpl {
        layout: LayoutCtx::new("chips", Some(user), lang),
        session: sess,
        users,
    })?
    .into_response())
}

#[derive(Deserialize)]
pub struct EnrollSave {
    pub label: Option<String>,
    /// Leer = Gast-Chip, sonst der Benutzer, dem der Chip gehört.
    pub user_id: Option<i64>,
    pub expires_at: Option<String>,
}

pub async fn enroll_save(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    Path(id): Path<i64>,
    lang: crate::i18n::Lang,
    Form(form): Form<EnrollSave>,
) -> AppResult<Response> {
    let sess: EnrollmentSession = sqlx::query_as::<_, EnrollmentSession>(
        "SELECT * FROM enrollment_sessions WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let Some(tag) = sess.captured_id_tag.as_deref() else {
        return Err(AppError::BadRequest(lang.t("err.no_chip_captured").into()));
    };
    if sess.consumed != 0 {
        return Err(AppError::BadRequest(lang.t("err.enroll_done").into()));
    }
    let expires_at = form
        .expires_at
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM chips WHERE id_tag = ?1")
        .bind(tag)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_some() {
        return Err(AppError::Conflict(format!(
            "{} {tag}",
            lang.t("err.chip_exists")
        )));
    }

    sqlx::query(
        "INSERT INTO chips (id_tag, label, user_id, kind, enabled, expires_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5)",
    )
    .bind(tag)
    .bind(form.label.as_deref())
    .bind(form.user_id)
    .bind(chip_kind(form.user_id))
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    sqlx::query("UPDATE enrollment_sessions SET consumed = 1 WHERE id = ?1")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Redirect::to("/chips").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    sqlx::query("DELETE FROM chips WHERE id = ?1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Redirect::to("/chips").into_response())
}

/// Kategorie eines Chips — abgeleitet aus der Zuordnung, nicht separat gepflegt.
fn chip_kind(user_id: Option<i64>) -> &'static str {
    if user_id.is_some() {
        "employee"
    } else {
        "guest"
    }
}
