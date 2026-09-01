//! Passwortwechsel durch den Benutzer selbst.
//!
//! Zwei Wege fuehren hierher: der freiwillige Wechsel ueber die eigene Seite
//! und der erzwungene, nachdem ein Administrator ein Passwort vergeben und
//! dabei den Haken gesetzt hat. Im erzwungenen Fall laesst die Middleware in
//! [`crate::web::force_password_change`] keine andere Seite mehr zu.

use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use serde::Deserialize;

use super::{render, LayoutCtx};
use crate::auth::AuthUser;
use crate::db::{hash_password, verify_password};
use crate::domain::user::User;
use crate::i18n::Lang;
use crate::{AppError, AppResult, AppState};

#[derive(Template)]
#[template(path = "password_change.html")]
struct ChangeTpl {
    layout: LayoutCtx,
    /// Steuert den Hinweistext: erzwungen oder freiwillig.
    forced: bool,
    error: Option<String>,
}

pub async fn form(AuthUser(user): AuthUser, lang: Lang) -> AppResult<Response> {
    let forced = user.must_change_password != 0;
    Ok(render(&ChangeTpl {
        layout: LayoutCtx::new("password", Some(user), lang),
        forced,
        error: None,
    })?
    .into_response())
}

#[derive(Deserialize)]
pub struct ChangeForm {
    pub current: String,
    pub new_password: String,
    pub repeat: String,
}

pub async fn submit(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    lang: Lang,
    Form(form): Form<ChangeForm>,
) -> AppResult<Response> {
    // Konten ohne Passwort-Hash gibt es (aus employees uebernommen), dort kann
    // nur ein Administrator eines vergeben.
    let Some(hash) = user.password_hash.as_deref() else {
        return Err(AppError::BadRequest(lang.t("err.pw_no_local").into()));
    };

    let fehler = if !verify_password(&form.current, hash).map_err(AppError::Other)? {
        Some(lang.t("err.pw_current_wrong"))
    } else if form.new_password.len() < 6 {
        Some(lang.t("err.pw_min6"))
    } else if form.new_password != form.repeat {
        Some(lang.t("err.pw_repeat"))
    } else if form.new_password == form.current {
        Some(lang.t("err.pw_same"))
    } else {
        None
    };

    if let Some(msg) = fehler {
        let forced = user.must_change_password != 0;
        return Ok(render(&ChangeTpl {
            layout: LayoutCtx::new("password", Some(user), lang),
            forced,
            error: Some(msg.to_string()),
        })?
        .into_response());
    }

    let neu = hash_password(&form.new_password).map_err(AppError::Other)?;
    sqlx::query("UPDATE users SET password_hash = ?1, must_change_password = 0 WHERE id = ?2")
        .bind(neu)
        .bind(user.id)
        .execute(&state.db)
        .await?;
    tracing::info!("Benutzer {} hat sein Passwort geaendert", user.username);

    Ok(Redirect::to(ziel(&user)).into_response())
}

/// Nach dem Wechsel dorthin, wo der Benutzer ohnehin hingehoert.
fn ziel(user: &User) -> &'static str {
    if user.is_admin() {
        "/"
    } else {
        "/me"
    }
}
