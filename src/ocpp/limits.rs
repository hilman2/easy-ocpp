//! Ladelimits: Timer und Ziel-Energie pro laufender Ladung.
//!
//! Ein Mitarbeiter hinterlegt entweder direkt an der laufenden Ladung oder als
//! Vorgabe in seinem Profil, wie viele kWh geladen und/oder wie lange geladen
//! werden soll. Erreicht eine Ladung eines der beiden Limits, schickt der
//! Watchdog ein `RemoteStopTransaction` an die Wallbox.
//!
//! Geprueft wird an zwei Stellen:
//!  - periodisch (siehe [`spawn_watchdog`]), deckt vor allem den Timer ab,
//!  - direkt nach eingehenden MeterValues, damit das Energielimit ohne
//!    Verzoegerung greift.

use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::AppState;

/// Prueftakt des Watchdogs. Feiner als das MeterValue-Intervall (30 s), damit
/// ein Timer nicht nennenswert ueberzogen wird.
const TICK: Duration = Duration::from_secs(15);

/// Grund, warum eine Ladung automatisch beendet wurde. Wird in
/// `transactions.limit_stopped` abgelegt.
pub const STOPPED_BY_ENERGY: i64 = 1;
pub const STOPPED_BY_TIME: i64 = 2;

/// Startet den Hintergrund-Task, der laufende Ladungen gegen ihre Limits prueft.
pub fn spawn_watchdog(state: AppState) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(TICK);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            if let Err(e) = enforce_all(&state).await {
                tracing::warn!("Limit-Watchdog: {e}");
            }
        }
    });
}

/// Prueft alle laufenden Ladungen mit Limit und stoppt die faelligen.
pub async fn enforce_all(state: &AppState) -> anyhow::Result<()> {
    let due = load_candidates(state, None).await?;
    for c in due {
        enforce_one(state, &c).await;
    }
    Ok(())
}

/// Prueft genau eine Ladung. Wird nach dem Eintreffen neuer MeterValues
/// aufgerufen, damit das Energielimit sofort greift.
pub async fn enforce_transaction(state: &AppState, tx_id: i64) {
    match load_candidates(state, Some(tx_id)).await {
        Ok(list) => {
            for c in list {
                enforce_one(state, &c).await;
            }
        }
        Err(e) => tracing::warn!("Limit-Pruefung fuer tx {tx_id}: {e}"),
    }
}

struct Candidate {
    tx_id: i64,
    ocpp_tx_id: i64,
    charge_point_id: String,
    limit_wh: Option<i64>,
    limit_until: Option<String>,
    start_meter_wh: i64,
    /// Zuletzt gemeldeter Zaehlerstand, falls vorhanden.
    last_meter_wh: Option<i64>,
}

async fn load_candidates(state: &AppState, only: Option<i64>) -> anyhow::Result<Vec<Candidate>> {
    // Nur laufende Ladungen, die ueberhaupt ein Limit tragen und fuer die noch
    // kein Stop ausgeloest wurde.
    let base = "SELECT t.id, COALESCE(t.ocpp_transaction_id, t.id), w.charge_point_id,
                       t.limit_wh, t.limit_until, t.start_meter_wh,
                       (SELECT mv.energy_wh FROM meter_values mv
                         WHERE mv.transaction_id = t.id AND mv.energy_wh IS NOT NULL
                         ORDER BY mv.timestamp DESC, mv.id DESC LIMIT 1)
                  FROM transactions t
                  JOIN wallboxes w ON w.id = t.wallbox_id
                 WHERE t.stop_time IS NULL
                   AND t.limit_stopped = 0
                   AND (t.limit_wh IS NOT NULL OR t.limit_until IS NOT NULL)";

    type Row = (i64, i64, String, Option<i64>, Option<String>, i64, Option<i64>);
    let rows: Vec<Row> = match only {
        Some(id) => sqlx::query_as(&format!("{base} AND t.id = ?1"))
            .bind(id)
            .fetch_all(&state.db)
            .await?,
        None => sqlx::query_as(base).fetch_all(&state.db).await?,
    };

    Ok(rows
        .into_iter()
        .map(|r| Candidate {
            tx_id: r.0,
            ocpp_tx_id: r.1,
            charge_point_id: r.2,
            limit_wh: r.3,
            limit_until: r.4,
            start_meter_wh: r.5,
            last_meter_wh: r.6,
        })
        .collect())
}

/// Entscheidet, ob ein Limit erreicht ist und welches zuerst.
fn reached(c: &Candidate, now: DateTime<Utc>) -> Option<i64> {
    if let (Some(limit), Some(last)) = (c.limit_wh, c.last_meter_wh) {
        if (last - c.start_meter_wh).max(0) >= limit {
            return Some(STOPPED_BY_ENERGY);
        }
    }
    if let Some(until) = c.limit_until.as_deref() {
        if let Ok(dt) = DateTime::parse_from_rfc3339(until) {
            if dt.with_timezone(&Utc) <= now {
                return Some(STOPPED_BY_TIME);
            }
        }
    }
    None
}

async fn enforce_one(state: &AppState, c: &Candidate) {
    let Some(reason) = reached(c, Utc::now()) else {
        return;
    };
    let what = if reason == STOPPED_BY_ENERGY { "Energielimit" } else { "Zeitlimit" };

    match crate::ocpp::ocpp16::remote_stop(&state.ocpp_hub, &c.charge_point_id, c.ocpp_tx_id).await {
        Ok(status) if status == "Accepted" => {
            // Erst nach einer angenommenen Anfrage markieren. Sonst wuerde ein
            // fehlgeschlagener Stop nie wiederholt.
            if let Err(e) = sqlx::query("UPDATE transactions SET limit_stopped = ?1 WHERE id = ?2")
                .bind(reason)
                .bind(c.tx_id)
                .execute(&state.db)
                .await
            {
                tracing::warn!("tx {}: limit_stopped konnte nicht gesetzt werden: {e}", c.tx_id);
            }
            tracing::info!(
                "tx {}: {what} erreicht, RemoteStop an {} gesendet",
                c.tx_id,
                c.charge_point_id
            );
        }
        Ok(status) => tracing::warn!(
            "tx {}: {what} erreicht, aber {} lehnte RemoteStop ab ({status}), naechster Versuch folgt",
            c.tx_id,
            c.charge_point_id
        ),
        Err(e) => tracing::warn!(
            "tx {}: {what} erreicht, RemoteStop an {} fehlgeschlagen: {e}, naechster Versuch folgt",
            c.tx_id,
            c.charge_point_id
        ),
    }
}
