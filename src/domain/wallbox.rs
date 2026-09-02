use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Wallbox {
    pub id: i64,
    pub charge_point_id: String,
    pub name: String,
    pub location: Option<String>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub firmware: Option<String>,
    pub serial_number: Option<String>,
    pub ocpp_version: Option<String>,
    pub auth_basic_user: Option<String>,
    pub auth_basic_pass: Option<String>,
    pub last_heartbeat: Option<String>,
    pub last_boot: Option<String>,
    pub connector_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Connector {
    pub id: i64,
    pub wallbox_id: i64,
    pub connector_id: i64,
    pub status: Option<String>,
    pub error_code: Option<String>,
    pub info: Option<String>,
    pub updated_at: String,
}

/// Eine StatusNotification, wie sie die Wallbox gemeldet hat.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConnectorEvent {
    pub id: i64,
    pub wallbox_id: i64,
    pub connector_id: i64,
    pub status: String,
    pub error_code: Option<String>,
    pub info: Option<String>,
    pub vendor_error: Option<String>,
    pub timestamp: String,
    pub created_at: String,
}

impl ConnectorEvent {
    /// Meldungen, die Aufmerksamkeit verdienen: der Anschluss ist gestoert oder
    /// die Wallbox nennt einen Fehlercode.
    pub fn is_fault(&self) -> bool {
        self.status.eq_ignore_ascii_case("Faulted")
            || self
                .error_code
                .as_deref()
                .map(|e| !e.is_empty() && !e.eq_ignore_ascii_case("NoError"))
                .unwrap_or(false)
    }
}

/// Meldungen einer Wallbox, neueste zuerst.
pub async fn events_for_wallbox(
    db: &sqlx::SqlitePool,
    wallbox_id: i64,
    limit: i64,
) -> sqlx::Result<Vec<ConnectorEvent>> {
    sqlx::query_as::<_, ConnectorEvent>(
        "SELECT * FROM connector_events
          WHERE wallbox_id = ?1
          ORDER BY timestamp DESC, id DESC
          LIMIT ?2",
    )
    .bind(wallbox_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

/// Meldungen aus dem Zeitraum einer Ladung, aelteste zuerst, damit sich der
/// Verlauf von oben nach unten liest.
///
/// Das Fenster reicht fuenf Minuten ueber das Ende hinaus: Finishing und
/// Available treffen regelmaessig erst nach dem StopTransaction ein, und genau
/// die sagen aus, ob sauber beendet wurde.
pub async fn events_for_session(
    db: &sqlx::SqlitePool,
    wallbox_id: i64,
    connector_id: i64,
    start: &str,
    stop: Option<&str>,
) -> sqlx::Result<Vec<ConnectorEvent>> {
    let sql = "SELECT * FROM connector_events
                WHERE wallbox_id = ?1 AND connector_id = ?2
                  AND timestamp >= ?3";
    match stop {
        Some(ende) => {
            sqlx::query_as::<_, ConnectorEvent>(&format!(
                "{sql} AND timestamp <= datetime(?4, '+5 minutes') ORDER BY timestamp, id"
            ))
            .bind(wallbox_id)
            .bind(connector_id)
            .bind(start)
            .bind(ende)
            .fetch_all(db)
            .await
        }
        None => {
            sqlx::query_as::<_, ConnectorEvent>(&format!("{sql} ORDER BY timestamp, id"))
                .bind(wallbox_id)
                .bind(connector_id)
                .bind(start)
                .fetch_all(db)
                .await
        }
    }
}
