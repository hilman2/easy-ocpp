use askama::Template;
use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::{IntoResponse, Redirect, Response},
    Form, Json,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{render, LayoutCtx};
use crate::{
    auth::AdminUser,
    domain::wallbox::Wallbox,
    i18n::Lang,
    ocpp::energy::{Meter, Settings, Status},
    AppError, AppResult, AppState,
};

#[derive(Template)]
#[template(path = "energy.html")]
struct Page {
    layout: LayoutCtx,
    settings: Settings,
    wallboxes: Vec<Wallbox>,
    meter_configured: bool,
    status: Status,
    lang: Lang,
}

pub async fn page(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    lang: Lang,
) -> AppResult<Response> {
    Ok(render(&Page {
        layout: LayoutCtx::new("energy", Some(user), lang),
        settings: Settings::load(&state).await?,
        wallboxes: sqlx::query_as("SELECT * FROM wallboxes ORDER BY name")
            .fetch_all(&state.db)
            .await?,
        meter_configured: state
            .config
            .energy
            .meter_token
            .as_deref()
            .is_some_and(|s| s.len() >= 24),
        status: state.energy.status.read().await.clone(),
        lang,
    })?
    .into_response())
}

#[derive(Deserialize)]
pub struct SettingsForm {
    pub enabled: Option<String>,
    pub grid_limit_w: i64,
    pub reserve_w: i64,
    pub max_age_s: i64,
    pub resume_margin_w: i64,
}

pub async fn save(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    lang: Lang,
    Form(form): Form<SettingsForm>,
) -> AppResult<Response> {
    let settings = Settings {
        enabled: form.enabled.as_deref() == Some("1"),
        grid_limit_w: form.grid_limit_w,
        reserve_w: form.reserve_w,
        max_age_s: form.max_age_s,
        resume_margin_w: form.resume_margin_w,
    };
    settings
        .validate()
        .map_err(|_| AppError::BadRequest(lang.t("energy.invalid_settings").into()))?;
    if settings.enabled {
        if !state
            .config
            .energy
            .meter_token
            .as_deref()
            .is_some_and(|s| s.len() >= 24)
        {
            return Err(AppError::BadRequest(lang.t("energy.token_missing").into()));
        }
        if state.ocpp_hub.list_online().iter().any(|cp| {
            state
                .ocpp_hub
                .get(cp)
                .is_some_and(|c| c.ocpp_version != "ocpp1.6")
        }) {
            return Err(AppError::Conflict(lang.t("energy.only16").into()));
        }
    }
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO settings(key, value) VALUES ('energy', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind(serde_json::to_string(&settings).map_err(anyhow::Error::from)?).execute(&mut *tx).await?;
    if settings.enabled {
        sqlx::query("INSERT INTO settings(key, value) VALUES ('energy_ever_enabled', '1') ON CONFLICT DO NOTHING")
            .execute(&mut *tx).await?;
    }
    tx.commit().await?;
    state.energy.wake.notify_one();
    Ok(Redirect::to("/energy").into_response())
}

#[derive(Deserialize)]
pub struct HardwareForm {
    pub charging_min_w: i64,
    pub charging_max_w: i64,
    pub charging_phases: i64,
    pub charging_rate_unit: String,
}

pub async fn hardware(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    axum::extract::Path(id): axum::extract::Path<i64>,
    lang: Lang,
    Form(form): Form<HardwareForm>,
) -> AppResult<Response> {
    // Aenderungen der Phasenzahl waehrend des Ladens koennten eine bereits
    // erteilte Stromfreigabe in eine zu hohe Leistung uebersetzen.
    if Settings::load(&state).await?.enabled {
        return Err(AppError::Conflict(lang.t("energy.disable_first").into()));
    }
    if form.charging_min_w < 1
        || form.charging_max_w < form.charging_min_w
        || form.charging_max_w > 10_000_000
        || !matches!(form.charging_phases, 1 | 3)
        || !matches!(form.charging_rate_unit.as_str(), "W" | "A")
    {
        return Err(AppError::BadRequest(
            lang.t("energy.invalid_settings").into(),
        ));
    }
    let changed = sqlx::query(
        "UPDATE wallboxes SET charging_min_w = ?1, charging_max_w = ?2,
        charging_phases = ?3, charging_rate_unit = ?4 WHERE id = ?5",
    )
    .bind(form.charging_min_w)
    .bind(form.charging_max_w)
    .bind(form.charging_phases)
    .bind(form.charging_rate_unit)
    .bind(id)
    .execute(&state.db)
    .await?;
    if changed.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Redirect::to("/energy").into_response())
}

#[derive(Template)]
#[template(path = "_energy_status.html")]
struct StatusTpl {
    status: Status,
    lang: Lang,
}

pub async fn live(
    State(state): State<AppState>,
    AdminUser(_): AdminUser,
    lang: Lang,
) -> AppResult<Response> {
    Ok(render(&StatusTpl {
        status: state.energy.status.read().await.clone(),
        lang,
    })?
    .into_response())
}

pub async fn status(State(state): State<AppState>, AdminUser(_): AdminUser) -> Json<Status> {
    Json(state.energy.status.read().await.clone())
}

/// Ein eigener Token statt Browser-Cookies fuer den spaeteren Zaehleradapter.
pub async fn meter(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(meter): Json<Meter>,
) -> AppResult<Response> {
    let expected = state
        .config
        .energy
        .meter_token
        .as_deref()
        .filter(|s| s.len() >= 24)
        .ok_or(AppError::Unauthorized)?;
    let supplied = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    // Feste Hash-Laenge; nicht beim ersten abweichenden Token-Zeichen abbrechen.
    let a = Sha256::digest(expected.as_bytes());
    let b = Sha256::digest(supplied.as_bytes());
    if a.iter()
        .zip(b.iter())
        .fold(0u8, |diff, (x, y)| diff | (x ^ y))
        != 0
    {
        return Err(AppError::Unauthorized);
    }
    let settings = Settings::load(&state).await?;
    state
        .energy
        .submit(meter, &settings)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(axum::http::StatusCode::ACCEPTED.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn meter_api_requires_its_token_and_admin_pages_require_a_session() {
        let mut state = crate::ocpp::energy::tests::state().await;
        let mut cfg = crate::config::Config::default();
        let token = "only-for-tests-at-least-24-characters";
        cfg.energy.meter_token = Some(token.into());
        state.config = std::sync::Arc::new(cfg);
        let app = crate::web::router(state.clone());
        let body = serde_json::to_string(&Meter {
            timestamp: Utc::now(),
            grid_power_w: -5000,
            charging_power_w: 0,
        })
        .unwrap();
        for auth in [None, Some("Bearer wrong-token")] {
            let mut request =
                Request::post("/api/energy/meter").header("content-type", "application/json");
            if let Some(auth) = auth {
                request = request.header("authorization", auth);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::from(body.clone())).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/energy/meter")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        for path in ["/energy", "/energy/live", "/api/energy/status"] {
            let response = app
                .clone()
                .oneshot(Request::get(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
        sqlx::query("INSERT INTO users(id, username, display_name, role, auth_source) VALUES(1, 'employee', 'Employee', 'user', 'local')")
            .execute(&state.db).await.unwrap();
        sqlx::query("INSERT INTO sessions(token, user_id, expires_at) VALUES('employee-session', 1, datetime('now', '+1 day'))")
            .execute(&state.db).await.unwrap();
        for path in ["/energy", "/energy/live", "/api/energy/status"] {
            let response = app
                .clone()
                .oneshot(
                    Request::get(path)
                        .header("cookie", "easy_ocpp_sid=employee-session")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
    }
}
