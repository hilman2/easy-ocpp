use askama::Template;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use chrono::{Duration, Utc};
use serde::Deserialize;

use super::{render, LayoutCtx};
use crate::auth::AuthUser;
use crate::domain::stats::{
    by_user, by_wallbox, employee_vs_guest, overview, Granularity, GuestSplit, NamedStat,
    PeriodStat, Scope,
};
use crate::{AppResult, AppState};

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTpl {
    layout: LayoutCtx,
    granularity: String,
    range: String,
    rows: Vec<PeriodStat>,
    per_user: Vec<NamedStat>,
    per_wallbox: Vec<NamedStat>,
    split: GuestSplit,
    /// Ein Mitarbeiter sieht nur seine eigenen Zahlen — die Aufstellung nach
    /// Person und der Gast-Anteil ergeben dort keinen Sinn.
    is_admin: bool,
}

#[derive(Deserialize)]
pub struct Filter {
    pub g: Option<String>,
    /// Zeitraum für die Rollups: "30d", "90d", "365d", "all" (default 90d).
    pub r: Option<String>,
}

pub async fn show(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    lang: crate::i18n::Lang,
    Query(filter): Query<Filter>,
) -> AppResult<Response> {
    let g = filter.g.as_deref().unwrap_or("month");
    let gran = match g {
        "quarter" => Granularity::Quarter,
        "year" => Granularity::Year,
        _ => Granularity::Month,
    };
    let range = filter.r.as_deref().unwrap_or("90d");
    let since = match range {
        "30d" => Some((Utc::now() - Duration::days(30)).to_rfc3339()),
        "365d" => Some((Utc::now() - Duration::days(365)).to_rfc3339()),
        "all" => None,
        _ => Some((Utc::now() - Duration::days(90)).to_rfc3339()),
    };
    let since_ref = since.as_deref();

    // Nicht-Admins bekommen ausschliesslich ihre eigenen Ladungen zu sehen.
    let is_admin = user.is_admin();
    let scope = if is_admin {
        Scope::all()
    } else {
        Scope::only(user.id)
    };

    let rows = overview(&state.db, gran, since_ref, scope)
        .await
        .map_err(crate::AppError::Other)?;
    let per_user = by_user(&state.db, since_ref, scope)
        .await
        .map_err(crate::AppError::Other)?;
    let per_wallbox = by_wallbox(&state.db, since_ref, scope)
        .await
        .map_err(crate::AppError::Other)?;
    let split = if is_admin {
        employee_vs_guest(&state.db, since_ref, scope)
            .await
            .map_err(crate::AppError::Other)?
    } else {
        GuestSplit::default()
    };

    let tpl = StatsTpl {
        layout: LayoutCtx::new("stats", Some(user), lang),
        granularity: g.to_string(),
        range: range.to_string(),
        rows,
        per_user,
        per_wallbox,
        split,
        is_admin,
    };
    Ok(render(&tpl)?.into_response())
}
