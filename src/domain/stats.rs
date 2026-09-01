use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize)]
pub struct PeriodStat {
    pub bucket: String,
    pub sessions: i64,
    pub energy_wh: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NamedStat {
    pub label: String,
    pub sessions: i64,
    pub energy_wh: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum Granularity {
    Month,
    Quarter,
    Year,
}

impl Granularity {
    /// SQLite strftime-Formatstring für die jeweilige Periode.
    pub fn bucket_expr(&self) -> &'static str {
        match self {
            // strftime liefert '2026-04'
            Granularity::Month => "strftime('%Y-%m', start_time)",
            Granularity::Year => "strftime('%Y', start_time)",
            // Quartal erfordert Arithmetik – wir berechnen manuell in Rust
            Granularity::Quarter => "strftime('%Y-%m', start_time)",
        }
    }
}

/// Sichtbereich einer Auswertung. Ein Mitarbeiter darf nur seine eigenen
/// Ladungen sehen; der Admin sieht alle.
#[derive(Debug, Clone, Copy, Default)]
pub struct Scope {
    /// Some(id) = nur Ladungen dieses Benutzers.
    pub user_id: Option<i64>,
}

impl Scope {
    pub fn all() -> Self {
        Self { user_id: None }
    }
    pub fn only(user_id: i64) -> Self {
        Self {
            user_id: Some(user_id),
        }
    }
    /// Zusätzliche WHERE-Bedingung; der Platzhalter wird als letzter Wert gebunden.
    fn clause(&self, col: &str) -> String {
        match self.user_id {
            Some(_) => format!(" AND {col} = ?"),
            None => String::new(),
        }
    }
}

/// Bindet erst den Zeitraum, dann (falls gesetzt) den Benutzer. Die
/// Reihenfolge muss der Reihenfolge der Platzhalter im SQL entsprechen.
fn bind_filters<'q>(
    mut q: sqlx::query::QueryAs<'q, sqlx::Sqlite, (String, i64, i64), sqlx::sqlite::SqliteArguments<'q>>,
    since: Option<&'q str>,
    scope: Scope,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, (String, i64, i64), sqlx::sqlite::SqliteArguments<'q>> {
    if let Some(s) = since {
        q = q.bind(s);
    }
    if let Some(uid) = scope.user_id {
        q = q.bind(uid);
    }
    q
}

pub async fn overview(
    db: &SqlitePool,
    gran: Granularity,
    since: Option<&str>,
    scope: Scope,
) -> Result<Vec<PeriodStat>> {
    let expr = gran.bucket_expr();
    let since_clause = if since.is_some() { " AND start_time >= ?" } else { "" };
    let user_clause = scope.clause("user_id");
    let sql = format!(
        "SELECT {expr} AS bucket,
                COUNT(*) AS sessions,
                COALESCE(SUM(COALESCE(stop_meter_wh,0) - start_meter_wh), 0) AS energy_wh
         FROM transactions
         WHERE stop_meter_wh IS NOT NULL{since_clause}{user_clause}
         GROUP BY bucket
         ORDER BY bucket DESC
         LIMIT 60"
    );
    let rows: Vec<(String, i64, i64)> = bind_filters(sqlx::query_as(&sql), since, scope)
        .fetch_all(db)
        .await?;

    let mut out: Vec<PeriodStat> = rows
        .into_iter()
        .map(|(bucket, sessions, energy_wh)| PeriodStat {
            bucket,
            sessions,
            energy_wh,
        })
        .collect();

    if let Granularity::Quarter = gran {
        use std::collections::BTreeMap;
        let mut acc: BTreeMap<String, (i64, i64)> = BTreeMap::new();
        for p in out.drain(..) {
            // bucket = YYYY-MM
            let year = &p.bucket[..4];
            let month: u32 = p.bucket[5..7].parse().unwrap_or(1);
            let q = (month - 1) / 3 + 1;
            let key = format!("{year}-Q{q}");
            let e = acc.entry(key).or_default();
            e.0 += p.sessions;
            e.1 += p.energy_wh;
        }
        out = acc
            .into_iter()
            .rev()
            .map(|(bucket, (sessions, energy_wh))| PeriodStat {
                bucket,
                sessions,
                energy_wh,
            })
            .collect();
    }

    Ok(out)
}

/// Verbrauch je Person. Ladungen ohne Benutzer laufen unter ihrem Gast-Label.
pub async fn by_user(db: &SqlitePool, since: Option<&str>, scope: Scope) -> Result<Vec<NamedStat>> {
    let since_clause = if since.is_some() { " AND t.start_time >= ?" } else { "" };
    let user_clause = scope.clause("t.user_id");
    let sql = format!(
        "SELECT COALESCE(u.display_name, t.guest_label, '(Gast)') AS label,
                COUNT(*) AS sessions,
                COALESCE(SUM(stop_meter_wh - start_meter_wh), 0) AS energy_wh
         FROM transactions t
         LEFT JOIN users u ON u.id = t.user_id
         WHERE stop_meter_wh IS NOT NULL{since_clause}{user_clause}
         GROUP BY label
         ORDER BY energy_wh DESC
         LIMIT 100"
    );
    let rows = bind_filters(sqlx::query_as(&sql), since, scope)
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(label, sessions, energy_wh)| NamedStat {
            label,
            sessions,
            energy_wh,
        })
        .collect())
}

pub async fn by_wallbox(
    db: &SqlitePool,
    since: Option<&str>,
    scope: Scope,
) -> Result<Vec<NamedStat>> {
    let since_clause = if since.is_some() { " AND t.start_time >= ?" } else { "" };
    let user_clause = scope.clause("t.user_id");
    let sql = format!(
        "SELECT w.name AS label,
                COUNT(*) AS sessions,
                COALESCE(SUM(stop_meter_wh - start_meter_wh), 0) AS energy_wh
         FROM transactions t
         JOIN wallboxes w ON w.id = t.wallbox_id
         WHERE stop_meter_wh IS NOT NULL{since_clause}{user_clause}
         GROUP BY w.id
         ORDER BY energy_wh DESC"
    );
    let rows = bind_filters(sqlx::query_as(&sql), since, scope)
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(label, sessions, energy_wh)| NamedStat {
            label,
            sessions,
            energy_wh,
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GuestSplit {
    pub employee_sessions: i64,
    pub employee_wh: i64,
    pub guest_sessions: i64,
    pub guest_wh: i64,
}

/// Mitarbeiter- gegen Gast-Ladungen. Gast = keinem Benutzer zugeordnet.
/// Für die Mitarbeiter-Sicht (`scope` eingeschränkt) ist die Aufteilung ohne
/// Aussage. Dort liefert die Funktion nur den eigenen Anteil.
pub async fn employee_vs_guest(
    db: &SqlitePool,
    since: Option<&str>,
    scope: Scope,
) -> Result<GuestSplit> {
    let since_clause = if since.is_some() { " AND t.start_time >= ?" } else { "" };
    let user_clause = scope.clause("t.user_id");
    let sql = format!(
        "SELECT
           SUM(CASE WHEN t.user_id IS NOT NULL THEN 1 ELSE 0 END) AS emp_s,
           SUM(CASE WHEN t.user_id IS NOT NULL THEN (stop_meter_wh - start_meter_wh) ELSE 0 END) AS emp_e,
           SUM(CASE WHEN t.user_id IS NULL THEN 1 ELSE 0 END) AS g_s,
           SUM(CASE WHEN t.user_id IS NULL THEN (stop_meter_wh - start_meter_wh) ELSE 0 END) AS g_e
         FROM transactions t
         WHERE stop_meter_wh IS NOT NULL{since_clause}{user_clause}"
    );
    let mut q = sqlx::query_as::<_, (Option<i64>, Option<i64>, Option<i64>, Option<i64>)>(&sql);
    if let Some(s) = since {
        q = q.bind(s);
    }
    if let Some(uid) = scope.user_id {
        q = q.bind(uid);
    }
    let row = q.fetch_one(db).await?;
    Ok(GuestSplit {
        employee_sessions: row.0.unwrap_or(0),
        employee_wh: row.1.unwrap_or(0).max(0),
        guest_sessions: row.2.unwrap_or(0),
        guest_wh: row.3.unwrap_or(0).max(0),
    })
}
