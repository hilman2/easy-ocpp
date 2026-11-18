//! Standortweite Leistungsverteilung. Positive Netzleistung bedeutet Bezug,
//! negative Einspeisung. Netz- und Ladeleistung muessen zeitgleich gemessen sein.
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{Mutex, Notify, RwLock};

use crate::{db, AppState};

const DEFAULT_PROFILE: i64 = 2_000_000_000;
const LEASE_S: i64 = 30;
const CALL_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_W: i64 = 10_000_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub enabled: bool,
    pub grid_limit_w: i64,
    pub reserve_w: i64,
    pub max_age_s: i64,
    pub resume_margin_w: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            grid_limit_w: 150_000,
            reserve_w: 10_000,
            max_age_s: 15,
            resume_margin_w: 500,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !(1..=MAX_W).contains(&self.grid_limit_w)
            || !(0..self.grid_limit_w).contains(&self.reserve_w)
            || !(1..=15).contains(&self.max_age_s)
            || !(0..=100_000).contains(&self.resume_margin_w)
        {
            bail!("energy.invalid_settings");
        }
        Ok(())
    }

    pub async fn load(state: &AppState) -> Result<Self> {
        let settings: Self = match db::setting_get(&state.db, "energy").await? {
            Some(raw) => serde_json::from_str(&raw)?,
            None => Self::default(),
        };
        settings.validate()?;
        Ok(settings)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Meter {
    pub timestamp: DateTime<Utc>,
    pub grid_power_w: i64,
    /// Summe aller von easy-ocpp verwalteten Ladeanschluesse, im Netz-Messwert
    /// bereits enthalten. Keine Zaehlerstaende und keine Sollwerte verwenden.
    pub charging_power_w: i64,
}

impl Meter {
    fn validate(&self, now: DateTime<Utc>, max_age_s: i64) -> Result<()> {
        let age = now.signed_duration_since(self.timestamp);
        if age < chrono::Duration::zero()
            || age > chrono::Duration::seconds(max_age_s)
            || !(-MAX_W..=MAX_W).contains(&self.grid_power_w)
            || !(0..=MAX_W).contains(&self.charging_power_w)
        {
            bail!("energy.invalid_meter");
        }
        Ok(())
    }
}

#[derive(Clone, Default, Serialize)]
pub struct Status {
    pub enabled: bool,
    pub meter: Option<Meter>,
    pub fresh: bool,
    pub surplus_w: i64,
    pub capacity_w: i64,
    pub sessions: Vec<SessionStatus>,
    pub errors: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct SessionStatus {
    pub id: i64,
    pub charge_point_id: String,
    pub connector_id: i64,
    pub target_w: i64,
    pub applied_w: Option<i64>,
    pub priority: i64,
    pub allow_grid: bool,
}

#[derive(Clone)]
struct Applied {
    connection: u64,
    watts: i64,
    at: DateTime<Utc>,
}

#[derive(Default)]
struct Runtime {
    cleaned: HashMap<String, u64>,
    applied: HashMap<i64, Applied>,
    /// Auch eine verlorene Antwort kann bedeuten, dass ein Profil angewandt
    /// wurde. Vor weiteren Erhoehungen deshalb erst bestaetigt auf 0 setzen.
    uncertain: HashSet<i64>,
    /// Senkungen erst nach einer neuen Messung fuer andere Autos freigeben.
    wait_for_meter_after: Option<DateTime<Utc>>,
}

#[derive(Default)]
pub struct Controller {
    meter: RwLock<Option<Meter>>,
    pub status: RwLock<Status>,
    runtime: Mutex<Runtime>,
    ready: RwLock<HashMap<String, (u64, u64)>>,
    boot_generation: RwLock<HashMap<String, u64>>,
    pub wake: Notify,
}

impl Controller {
    pub async fn submit(&self, meter: Meter, settings: &Settings) -> Result<()> {
        meter.validate(Utc::now(), settings.max_age_s)?;
        let mut previous = self.meter.write().await;
        if previous
            .as_ref()
            .is_some_and(|m| meter.timestamp <= m.timestamp)
        {
            bail!("energy.old_meter");
        }
        *previous = Some(meter);
        drop(previous);
        self.wake.notify_one();
        Ok(())
    }

    pub async fn reset_wallbox(&self, cp_id: &str) {
        *self
            .boot_generation
            .write()
            .await
            .entry(cp_id.to_string())
            .or_default() += 1;
        self.ready.write().await.remove(cp_id);
        self.wake.notify_one();
    }

    /// Keine OCPP-CALLs aus dem Empfangs-Task abwarten: dieser muss waehrend
    /// der Profilinstallation weiter CALLRESULT-Nachrichten empfangen koennen.
    pub async fn may_start(&self, state: &AppState, cp_id: &str) -> bool {
        let Ok(settings) = Settings::load(state).await else {
            return false;
        };
        if !settings.enabled {
            return true;
        }
        let Some(conn) = state.ocpp_hub.get(cp_id) else {
            return false;
        };
        if conn.ocpp_version != "ocpp1.6" {
            return false;
        }
        let fresh = self
            .meter
            .read()
            .await
            .as_ref()
            .is_some_and(|m| m.validate(Utc::now(), settings.max_age_s).is_ok());
        let generation = self
            .boot_generation
            .read()
            .await
            .get(cp_id)
            .copied()
            .unwrap_or(0);
        fresh && self.ready.read().await.get(cp_id) == Some(&(conn.id, generation))
    }
}

#[derive(Clone, Debug, sqlx::FromRow)]
struct Demand {
    id: i64,
    charge_point_id: String,
    connector_id: i64,
    ocpp_tx_id: i64,
    priority: i64,
    allow_grid: bool,
    min_w: i64,
    max_w: i64,
    phases: i64,
    rate_unit: String,
}

impl Demand {
    fn valid(&self) -> bool {
        self.connector_id > 0
            && self.connector_id < 1_000_000
            && self.min_w > 0
            && self.max_w >= self.min_w
            && self.max_w <= MAX_W
            && matches!(self.phases, 1 | 3)
            && matches!(self.rate_unit.as_str(), "W" | "A")
    }

    fn quantize(&self, watts: i64) -> i64 {
        if self.rate_unit == "A" {
            // 230 V je Phase; Abrunden verhindert, dass Umrechnung das Budget erhoeht.
            (watts * 10 / (230 * self.phases)) * (230 * self.phases) / 10
        } else {
            watts
        }
    }

    fn minimum(&self) -> i64 {
        if self.rate_unit == "A" {
            let step = 230 * self.phases;
            ((self.min_w * 10 + step - 1) / step) * step / 10
        } else {
            self.min_w
        }
    }

    fn profile_id(&self) -> i64 {
        1_000_000 + self.connector_id
    }
}

/// Netzleistung enthaelt die Autos bereits. Ihr Abzug ergibt die restliche
/// Standortlast nach PV. So wird laufende Ladung nicht doppelt gezaehlt.
fn budgets(settings: &Settings, meter: &Meter) -> (i64, i64) {
    let base = meter.grid_power_w - meter.charging_power_w;
    let surplus = (-base).max(0);
    let capacity = (settings.grid_limit_w - settings.reserve_w - base).max(0);
    (surplus.min(capacity), capacity)
}

/// Strikte Prioritaet zwischen Gruppen, gleiche Leistung innerhalb einer
/// Gruppe. Unterhalb der Mindestleistung wartet ein Auto mit 0 W.
fn allocate(
    demands: &[Demand],
    surplus: i64,
    capacity: i64,
    margin: i64,
    applied: &HashMap<i64, Applied>,
) -> Vec<i64> {
    let mut out = vec![0; demands.len()];
    let mut solar = surplus;
    let mut total = capacity;
    for priority in [1, 0] {
        let group: Vec<usize> = demands
            .iter()
            .enumerate()
            .filter(|(_, d)| d.priority == priority && d.valid())
            .map(|(i, _)| i)
            .collect();
        for &i in &group {
            let d = &demands[i];
            let min = d.minimum();
            let available = if d.allow_grid {
                total
            } else {
                total.min(solar)
            };
            let restarting = !applied.get(&d.id).is_some_and(|p| p.watts > 0);
            let needed = min + if restarting { margin } else { 0 };
            if min <= d.max_w && available >= needed {
                out[i] = min;
                total -= min;
                solar = (solar - min).max(0);
            }
        }
        // In kleinen Schritten verteilen, damit ein einzelner Mitarbeiter
        // nicht das ganze Restbudget seiner Prioritaetsgruppe erhaelt.
        loop {
            let mut changed = false;
            for &i in &group {
                let d = &demands[i];
                if out[i] == 0 {
                    continue;
                }
                let available = if d.allow_grid {
                    total
                } else {
                    total.min(solar)
                };
                let next = d.quantize((out[i] + 100).min(d.max_w).min(out[i] + available));
                let extra = next - out[i];
                if extra > 0 {
                    out[i] = next;
                    total -= extra;
                    solar = (solar - extra).max(0);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }
    out
}

fn zero_default() -> Value {
    json!({"connectorId": 0, "csChargingProfiles": {
        "chargingProfileId": DEFAULT_PROFILE, "stackLevel": 0,
        "chargingProfilePurpose": "TxDefaultProfile", "chargingProfileKind": "Absolute",
        "chargingSchedule": {"startSchedule": Utc::now(), "chargingRateUnit": "W",
            "chargingSchedulePeriod": [{"startPeriod": 0, "limit": 0}]}
    }})
}

fn profile(d: &Demand, watts: i64) -> Value {
    let rate = if d.rate_unit == "A" {
        watts as f64 / (230 * d.phases) as f64
    } else {
        watts as f64
    };
    // Kein validTo/duration: die Null-Periode bleibt nach Ablauf bestehen,
    // auch wenn der Server stirbt oder eine Verbindung verloren geht.
    json!({"connectorId": d.connector_id, "csChargingProfiles": {
        "chargingProfileId": d.profile_id(), "transactionId": d.ocpp_tx_id,
        "stackLevel": 0, "chargingProfilePurpose": "TxProfile",
        "chargingProfileKind": "Absolute",
        "chargingSchedule": {"startSchedule": Utc::now(), "chargingRateUnit": d.rate_unit,
            "chargingSchedulePeriod": [
                {"startPeriod": 0, "limit": (rate * 10.0).floor() / 10.0, "numberPhases": d.phases},
                {"startPeriod": LEASE_S, "limit": 0, "numberPhases": d.phases}
            ]}
    }})
}

async fn accepted(state: &AppState, cp: &str, action: &str, payload: Value) -> Result<()> {
    let reply = state
        .ocpp_hub
        .call(cp, action, payload, CALL_TIMEOUT)
        .await?;
    if reply["status"] != "Accepted" {
        bail!("{cp}: {action}: {}", reply["status"]);
    }
    Ok(())
}

async fn apply(state: &AppState, d: &Demand, watts: i64) -> Result<()> {
    if let Err(e) = accepted(
        state,
        &d.charge_point_id,
        "SetChargingProfile",
        profile(d, watts),
    )
    .await
    {
        // Nicht als gedrosselt buchen, wenn die Wallbox das Profil abgelehnt hat.
        // RemoteStop ist nur der Rueckfall; Ablehnungen werden weiter versucht.
        let _ = crate::ocpp::ocpp16::remote_stop(&state.ocpp_hub, &d.charge_point_id, d.ocpp_tx_id)
            .await;
        return Err(e);
    }
    Ok(())
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! { _ = ticker.tick() => {}, _ = state.energy.wake.notified() => {} }
            if let Err(e) = reconcile(&state).await {
                tracing::error!("Laderegelung: {e:#}");
                state.energy.status.write().await.errors = vec![e.to_string()];
            }
        }
    });
}

async fn reconcile(state: &AppState) -> Result<()> {
    let settings = Settings::load(state).await?;
    let mut rt = state.energy.runtime.lock().await;
    let demands: Vec<Demand> = sqlx::query_as(
        "SELECT t.id, w.charge_point_id, t.connector_id,
                COALESCE(t.ocpp_transaction_id, t.id) AS ocpp_tx_id,
                COALESCE(c.charging_priority, 0) AS priority,
                COALESCE(c.allow_grid, 0) != 0 AND COALESCE(c.enabled, 0) != 0 AS allow_grid,
                w.charging_min_w AS min_w, w.charging_max_w AS max_w,
                w.charging_phases AS phases, w.charging_rate_unit AS rate_unit
           FROM transactions t JOIN wallboxes w ON w.id = t.wallbox_id
           LEFT JOIN chips c ON c.id = t.chip_id
          WHERE t.stop_time IS NULL ORDER BY t.id",
    )
    .fetch_all(&state.db)
    .await?;
    rt.applied
        .retain(|id, _| demands.iter().any(|d| d.id == *id));
    rt.uncertain.retain(|id| demands.iter().any(|d| d.id == *id));
    let meter = state.energy.meter.read().await.clone();
    let fresh = meter
        .as_ref()
        .is_some_and(|m| m.validate(Utc::now(), settings.max_age_s).is_ok());
    let (solar, capacity) = meter
        .as_ref()
        .filter(|_| fresh)
        .map(|m| budgets(&settings, m))
        .unwrap_or((0, 0));
    let mut status = Status {
        enabled: settings.enabled,
        meter: meter.clone(),
        fresh,
        surplus_w: solar,
        capacity_w: capacity,
        ..Status::default()
    };

    if !settings.enabled {
        // Auch nach Neustart gezielt eigene persistente Profile entfernen.
        // Andere Profile werden nicht geloescht.
        let was_enabled = db::setting_get(&state.db, "energy_ever_enabled")
            .await?
            .is_some();
        for cp in state
            .ocpp_hub
            .list_online()
            .into_iter()
            .filter(|_| was_enabled)
        {
            let Some(conn) = state.ocpp_hub.get(&cp) else {
                continue;
            };
            if conn.ocpp_version != "ocpp1.6" || rt.cleaned.get(&cp) == Some(&conn.id) {
                continue;
            }
            let mut ids = vec![DEFAULT_PROFILE];
            ids.extend(
                demands
                    .iter()
                    .filter(|d| d.charge_point_id == cp)
                    .map(Demand::profile_id),
            );
            let mut ok = true;
            for id in ids {
                match state
                    .ocpp_hub
                    .call(&cp, "ClearChargingProfile", json!({"id": id}), CALL_TIMEOUT)
                    .await
                {
                    Ok(v) if v["status"] == "Accepted" || v["status"] == "Unknown" => {}
                    result => {
                        ok = false;
                        status
                            .errors
                            .push(format!("{cp}: ClearChargingProfile: {result:?}"));
                    }
                }
            }
            if ok {
                rt.cleaned.insert(cp, conn.id);
            }
        }
        state.energy.ready.write().await.clear();
        rt.applied.clear();
        rt.uncertain.clear();
        rt.wait_for_meter_after = None;
        *state.energy.status.write().await = status;
        return Ok(());
    }
    rt.cleaned.clear();
    // Persistenter Null-Default verhindert unkontrolliertes Anladen und
    // schuetzt neue Transaktionen ohne eigenes Profil.
    let online = state.ocpp_hub.list_online();
    let ready = state.energy.ready.read().await.clone();
    let generations = state.energy.boot_generation.read().await.clone();
    let installs = online.iter().filter_map(|cp| {
        let conn = state.ocpp_hub.get(cp)?;
        let generation = generations.get(cp).copied().unwrap_or(0);
        if conn.ocpp_version != "ocpp1.6" || ready.get(cp) == Some(&(conn.id, generation)) {
            return None;
        }
        Some(async move {
            let result = install_guard(state, cp).await;
            (cp.clone(), conn.id, generation, result)
        })
    });
    for (cp, connection, generation, result) in join_all(installs).await {
        match result {
            Ok(()) => {
                let current = state
                    .energy
                    .boot_generation
                    .read()
                    .await
                    .get(&cp)
                    .copied()
                    .unwrap_or(0);
                if current == generation {
                    state
                        .energy
                        .ready
                        .write()
                        .await
                        .insert(cp, (connection, generation));
                }
            }
            Err(e) => status.errors.push(e.to_string()),
        }
    }
    let targets = allocate(
        &demands,
        solar,
        capacity,
        settings.resume_margin_w,
        &rt.applied,
    );
    let mut safe = true;
    let mut reduced = false;
    let now = Utc::now();
    let ready = state.energy.ready.read().await.clone();
    let generations = state.energy.boot_generation.read().await.clone();
    let mut must_stop = Vec::new();
    // Alle Senkungen (einschliesslich unbekannter Ist-Limits nach einem
    // Neustart) abschliessen, bevor irgendwo Leistung erhoeht wird.
    let reductions = demands
        .iter()
        .zip(&targets)
        .filter_map(|(d, target)| {
            let conn = state.ocpp_hub.get(&d.charge_point_id);
            let previous = rt
                .applied
                .get(&d.id)
                .filter(|p| !rt.uncertain.contains(&d.id)
                    && conn.as_ref().is_some_and(|c| c.id == p.connection));
            let unsupported =
                conn.as_ref().map_or(true, |c| c.ocpp_version != "ocpp1.6") || !d.valid();
            let generation = generations.get(&d.charge_point_id).copied().unwrap_or(0);
            let guarded = conn
                .as_ref()
                .is_some_and(|c| ready.get(&d.charge_point_id) == Some(&(c.id, generation)));
            if unsupported || !guarded {
                safe = false;
                status
                    .errors
                    .push(format!("{}: energy.not_ready", d.charge_point_id));
                if conn.as_ref().is_some_and(|c| c.ocpp_version == "ocpp1.6") {
                    must_stop.push(d);
                }
                return None;
            }
            let watts = if previous.is_none() { 0 } else { *target };
            if previous.map_or(true, |p| {
                watts < p.watts || (watts == 0 && (now - p.at).num_seconds() >= 10)
            }) {
                let connection = conn.unwrap().id;
                Some(async move { (d.id, connection, watts, apply(state, d, watts).await) })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let (reduction_results, stop_results) = tokio::join!(
        join_all(reductions),
        join_all(
            must_stop
                .into_iter()
                .map(|d| crate::ocpp::ocpp16::remote_stop(
                    &state.ocpp_hub,
                    &d.charge_point_id,
                    d.ocpp_tx_id
                ))
        )
    );
    for result in stop_results {
        if let Err(e) = result {
            status.errors.push(e.to_string());
        }
    }
    for (id, connection, watts, result) in reduction_results {
        match result {
            Ok(()) => {
                let was_uncertain = rt.uncertain.remove(&id);
                reduced |= was_uncertain || rt.applied.get(&id).map_or(true, |p| watts < p.watts);
                rt.applied.insert(
                    id,
                    Applied {
                        connection,
                        watts,
                        at: Utc::now(),
                    },
                );
            }
            Err(e) => {
                rt.uncertain.insert(id);
                safe = false;
                status.errors.push(e.to_string());
            }
        }
    }
    if reduced {
        rt.wait_for_meter_after = Some(Utc::now());
    }
    let reserved_w: i64 = rt.applied.values().map(|p| p.watts).sum();
    let confirmed = rt.wait_for_meter_after.map_or(true, |after| {
        meter.as_ref().is_some_and(|m| m.timestamp > after && m.charging_power_w <= reserved_w)
    });
    let latest = state.energy.meter.read().await.clone();
    let still_fresh = latest.as_ref().is_some_and(|m| {
        m.validate(Utc::now(), settings.max_age_s).is_ok()
            && meter
                .as_ref()
                .is_some_and(|old| old.timestamp == m.timestamp)
    });
    if !still_fresh && fresh {
        state.energy.wake.notify_one();
    }
    if safe && confirmed && still_fresh {
        let increases = demands.iter().zip(&targets).filter_map(|(d, target)| {
            let previous = rt.applied.get(&d.id)?;
            if *target <= 0 || (*target == previous.watts && (now - previous.at).num_seconds() < 10)
            {
                return None;
            }
            let connection = previous.connection;
            Some(async move { (d.id, connection, *target, apply(state, d, *target).await) })
        });
        for (id, connection, watts, result) in join_all(increases).await {
            match result {
                Ok(()) => {
                    rt.uncertain.remove(&id);
                    rt.applied.insert(
                        id,
                        Applied {
                            connection,
                            watts,
                            at: Utc::now(),
                        },
                    );
                }
                Err(e) => {
                    rt.uncertain.insert(id);
                    status.errors.push(e.to_string());
                },
            }
        }
    }
    status.sessions = demands
        .iter()
        .zip(&targets)
        .map(|(d, target)| SessionStatus {
            id: d.id,
            charge_point_id: d.charge_point_id.clone(),
            connector_id: d.connector_id,
            target_w: *target,
            applied_w: rt.applied.get(&d.id).map(|p| p.watts),
            priority: d.priority,
            allow_grid: d.allow_grid,
        })
        .collect();
    *state.energy.status.write().await = status;
    Ok(())
}

async fn install_guard(state: &AppState, cp: &str) -> Result<()> {
    let (unit,): (String,) =
        sqlx::query_as("SELECT charging_rate_unit FROM wallboxes WHERE charge_point_id = ?1")
            .bind(cp)
            .fetch_one(&state.db)
            .await?;
    let mut default = zero_default();
    default["csChargingProfiles"]["chargingSchedule"]["chargingRateUnit"] = json!(unit);
    accepted(state, cp, "SetChargingProfile", default).await?;
    // Cache und Offline-Autorisierung duerfen den Null-Default nicht umgehen.
    for (key, value) in [
        ("LocalAuthorizeOffline", "false"),
        ("LocalPreAuthorize", "false"),
        ("AuthorizationCacheEnabled", "false"),
        ("AllowOfflineTxForUnknownId", "false"),
    ] {
        accepted(
            state,
            cp,
            "ChangeConfiguration",
            json!({"key": key, "value": value}),
        )
        .await
        .with_context(|| format!("{cp}: {key}"))?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ocpp::{
        hub::{Connection, Hub},
        OcppMessage,
    };
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    #[tokio::test]
    async fn existing_guest_chips_gain_priority_without_grid_permission() {
        let db = SqlitePoolOptions::new().max_connections(1)
            .connect("sqlite::memory:").await.unwrap();
        for sql in [include_str!("../../migrations/0001_initial.sql"),
            include_str!("../../migrations/0002_employees.sql"),
            include_str!("../../migrations/0003_users_are_employees.sql"),
            include_str!("../../migrations/0004_password_change_and_report_mail.sql"),
            include_str!("../../migrations/0005_connector_events.sql")] {
            sqlx::raw_sql(sql).execute(&db).await.unwrap();
        }
        sqlx::query("INSERT INTO chips(id_tag, kind) VALUES('private', 'employee'), ('guest', 'guest')")
            .execute(&db).await.unwrap();
        sqlx::raw_sql(include_str!("../../migrations/0006_surplus_charging.sql"))
            .execute(&db).await.unwrap();
        let rows: Vec<(String, i64, i64)> = sqlx::query_as(
            "SELECT id_tag, charging_priority, allow_grid FROM chips ORDER BY id_tag")
            .fetch_all(&db).await.unwrap();
        assert_eq!(rows, vec![("guest".into(), 1, 0), ("private".into(), 0, 0)]);
    }
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    fn demand(id: i64, priority: i64, allow_grid: bool) -> Demand {
        Demand {
            id,
            charge_point_id: "test".into(),
            connector_id: id,
            ocpp_tx_id: id,
            priority,
            allow_grid,
            min_w: 4140,
            max_w: 11040,
            phases: 3,
            rate_unit: "W".into(),
        }
    }

    #[test]
    fn site_budgets_do_not_count_running_cars_twice() {
        let s = Settings::default();
        let m = Meter {
            timestamp: Utc::now(),
            grid_power_w: -5_000,
            charging_power_w: 10_000,
        };
        assert_eq!(budgets(&s, &m), (15_000, 155_000));
        let m = Meter {
            grid_power_w: 148_000,
            charging_power_w: 20_000,
            ..m
        };
        assert_eq!(budgets(&s, &m), (0, 12_000));
    }

    #[test]
    fn private_cars_wait_without_surplus_company_cars_respect_reserve() {
        let d = vec![demand(1, 0, false), demand(2, 0, true)];
        assert_eq!(allocate(&d, 0, 6_000, 0, &HashMap::new()), vec![0, 6_000]);
        assert_eq!(allocate(&d, 0, 4_000, 0, &HashMap::new()), vec![0, 0]);
        assert_eq!(allocate(&d, 0, 0, 0, &HashMap::new()), vec![0, 0]);
    }

    #[test]
    fn management_and_guests_take_priority_without_implicit_grid_permission() {
        let d = vec![demand(1, 0, false), demand(2, 1, false), demand(3, 0, true)];
        let a = allocate(&d, 6_000, 30_000, 0, &HashMap::new());
        assert_eq!(a, vec![0, 6_000, 11_040]);
        assert_eq!(
            allocate(&d[..2], 0, 140_000, 0, &HashMap::new()),
            vec![0, 0]
        );
    }

    #[test]
    fn same_priority_shares_surplus_and_small_budgets_pause_whole_cars() {
        let d = vec![demand(1, 0, false), demand(2, 0, false)];
        let a = allocate(&d, 12_000, 12_000, 0, &HashMap::new());
        assert!((a[0] - a[1]).abs() <= 100);
        assert_eq!(a.iter().sum::<i64>(), 12_000);
        assert_eq!(
            allocate(&d, 5_000, 5_000, 0, &HashMap::new()),
            vec![5_000, 0]
        );
    }

    #[test]
    fn restart_margin_prevents_chattering_but_reductions_are_immediate() {
        let d = vec![demand(1, 0, false)];
        assert_eq!(allocate(&d, 4500, 4500, 500, &HashMap::new()), vec![0]);
        let running = HashMap::from([(
            1,
            Applied {
                connection: 1,
                watts: 6000,
                at: Utc::now(),
            },
        )]);
        assert_eq!(allocate(&d, 4500, 4500, 500, &running), vec![4500]);
        assert_eq!(allocate(&d, 4000, 4000, 500, &running), vec![0]);
    }

    #[test]
    fn allocation_invariants_with_mixed_permissions_phases_and_current_units() {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(150_000);
        for _ in 0..1200 {
            let d: Vec<_> = (1..=8)
                .map(|id| {
                    let mut d = demand(id, rng.gen_range(0..=1), rng.gen_bool(0.5));
                    if rng.gen_bool(0.5) {
                        d.rate_unit = "A".into();
                    }
                    if rng.gen_bool(0.5) {
                        d.phases = 1;
                        d.min_w = 1380;
                        d.max_w = 3680;
                    }
                    d
                })
                .collect();
            let capacity = rng.gen_range(0..100_000);
            let solar = rng.gen_range(0..=capacity);
            let a = allocate(&d, solar, capacity, 500, &HashMap::new());
            assert!(a.iter().sum::<i64>() <= capacity);
            assert!(
                d.iter()
                    .zip(&a)
                    .filter(|(d, _)| !d.allow_grid)
                    .map(|(_, w)| w)
                    .sum::<i64>()
                    <= solar
            );
            for (d, w) in d.iter().zip(&a) {
                assert!(*w == 0 || (*w >= d.min_w && *w <= d.max_w));
                assert_eq!(*w, d.quantize(*w));
            }
        }
    }

    #[test]
    fn charging_profiles_fall_back_to_zero_without_expiring_to_unlimited() {
        let mut d = demand(7, 1, true);
        d.rate_unit = "A".into();
        let p = profile(&d, d.quantize(7_000));
        let p = &p["csChargingProfiles"];
        assert_eq!(p["transactionId"], 7);
        assert_eq!(p["chargingProfilePurpose"], "TxProfile");
        assert!(p.get("validTo").is_none());
        assert!(p["chargingSchedule"].get("duration").is_none());
        let periods = &p["chargingSchedule"]["chargingSchedulePeriod"];
        assert_eq!(periods[1]["startPeriod"], LEASE_S);
        assert_eq!(periods[1]["limit"], 0);
        assert!(periods[0]["limit"].as_f64().unwrap() * 690.0 <= 7_000.0);
        assert_eq!(
            zero_default()["csChargingProfiles"]["chargingSchedule"]["chargingSchedulePeriod"][0]
                ["limit"],
            0
        );
    }

    #[tokio::test]
    async fn meter_rejects_stale_future_negative_charging_and_replayed_readings() {
        let c = Controller::default();
        let s = Settings::default();
        let m = Meter {
            timestamp: Utc::now(),
            grid_power_w: -5_000,
            charging_power_w: 0,
        };
        c.submit(m.clone(), &s).await.unwrap();
        assert!(c.submit(m.clone(), &s).await.is_err());
        for bad in [
            Meter {
                timestamp: Utc::now() - chrono::Duration::seconds(16),
                ..m.clone()
            },
            Meter {
                timestamp: Utc::now() + chrono::Duration::seconds(1),
                ..m.clone()
            },
            Meter {
                timestamp: Utc::now(),
                charging_power_w: -1,
                ..m.clone()
            },
        ] {
            assert!(c.submit(bad, &s).await.is_err());
        }
    }

    pub(crate) async fn state() -> AppState {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(":memory:")
                    .foreign_keys(true),
            )
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        AppState {
            db,
            ocpp_hub: Arc::new(Hub::default()),
            config: Arc::new(crate::config::Config::default()),
            energy: Arc::new(Controller::default()),
        }
    }

    async fn prepare(state: &AppState) {
        let s = Settings {
            enabled: true,
            ..Settings::default()
        };
        db::setting_set(&state.db, "energy", &serde_json::to_string(&s).unwrap())
            .await
            .unwrap();
        sqlx::query("INSERT INTO wallboxes(id, charge_point_id, name) VALUES(1, 'test', 'Test')")
            .execute(&state.db)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO chips(id, id_tag, kind, allow_grid) VALUES(1, 'company', 'employee', 1)",
        )
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query("INSERT INTO transactions(wallbox_id, connector_id, id_tag, chip_id, start_time, start_meter_wh)
            VALUES(1, 1, 'company', 1, datetime('now'), 0)").execute(&state.db).await.unwrap();
    }

    fn mock(state: &AppState, reject: Arc<AtomicBool>) -> Arc<Mutex<Vec<(String, Value)>>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let conn = Arc::new(Connection::new("test".into(), "ocpp1.6".into(), tx));
        state.ocpp_hub.register(conn.clone());
        let messages = Arc::new(Mutex::new(Vec::new()));
        let captured = messages.clone();
        tokio::spawn(async move {
            while let Some(raw) = rx.recv().await {
                if let OcppMessage::Call {
                    unique_id,
                    action,
                    payload,
                } = OcppMessage::parse(&raw).unwrap()
                {
                    let status = if action == "SetChargingProfile" && reject.load(Ordering::SeqCst)
                    {
                        "Rejected"
                    } else {
                        "Accepted"
                    };
                    captured.lock().await.push((action, payload));
                    if let Some(pending) = conn.pending.lock().unwrap().remove(&unique_id) {
                        let _ = pending.responder.send(Ok(json!({"status": status})));
                    }
                }
            }
        });
        messages
    }

    async fn reading(state: &AppState, grid: i64, charging: i64) {
        state
            .energy
            .submit(
                Meter {
                    timestamp: Utc::now(),
                    grid_power_w: grid,
                    charging_power_w: charging,
                },
                &Settings::default(),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn guard_then_zero_then_fresh_meter_then_charge_and_pause_on_stale_data() {
        let state = state().await;
        prepare(&state).await;
        let messages = mock(&state, Arc::new(AtomicBool::new(false)));
        assert!(!state.energy.may_start(&state, "test").await);
        reading(&state, 130_000, 0).await;
        reconcile(&state).await.unwrap();
        assert!(state.energy.may_start(&state, "test").await);
        assert_eq!(state.energy.runtime.lock().await.applied[&1].watts, 0);
        reading(&state, 130_000, 0).await;
        reconcile(&state).await.unwrap();
        assert_eq!(state.energy.runtime.lock().await.applied[&1].watts, 10_000);
        state.energy.meter.write().await.as_mut().unwrap().timestamp -=
            chrono::Duration::seconds(16);
        reconcile(&state).await.unwrap();
        assert_eq!(state.energy.runtime.lock().await.applied[&1].watts, 0);
        assert!(!state.energy.may_start(&state, "test").await);
        let log = messages.lock().await;
        let profiles: Vec<_> = log
            .iter()
            .filter(|(action, _)| action == "SetChargingProfile")
            .collect();
        assert_eq!(
            profiles[0].1["csChargingProfiles"]["chargingProfilePurpose"],
            "TxDefaultProfile"
        );
        assert_eq!(
            profiles[1].1["csChargingProfiles"]["chargingSchedule"]["chargingSchedulePeriod"][0]
                ["limit"]
                .as_f64(),
            Some(0.0)
        );
    }

    #[tokio::test]
    async fn rejected_reduction_keeps_old_reservation_and_attempts_stop() {
        let state = state().await;
        prepare(&state).await;
        let reject = Arc::new(AtomicBool::new(false));
        let messages = mock(&state, reject.clone());
        reading(&state, 100_000, 0).await;
        reconcile(&state).await.unwrap();
        reading(&state, 100_000, 0).await;
        reconcile(&state).await.unwrap();
        let before = state.energy.runtime.lock().await.applied[&1].watts;
        reject.store(true, Ordering::SeqCst);
        reading(&state, 160_000, before).await;
        reconcile(&state).await.unwrap();
        assert_eq!(state.energy.runtime.lock().await.applied[&1].watts, before);
        assert!(!state.energy.status.read().await.errors.is_empty());
        assert!(messages
            .lock()
            .await
            .iter()
            .any(|(a, _)| a == "RemoteStopTransaction"));
    }

    #[tokio::test]
    async fn never_enabled_does_not_change_wallbox_and_disabled_cleans_owned_profiles() {
        let state = state().await;
        prepare(&state).await;
        let messages = mock(&state, Arc::new(AtomicBool::new(false)));
        db::setting_set(
            &state.db,
            "energy",
            &serde_json::to_string(&Settings::default()).unwrap(),
        )
        .await
        .unwrap();
        reconcile(&state).await.unwrap();
        assert!(messages.lock().await.is_empty());
        db::setting_set(&state.db, "energy_ever_enabled", "1")
            .await
            .unwrap();
        reconcile(&state).await.unwrap();
        let log = messages.lock().await;
        assert_eq!(log.len(), 2);
        assert!(log.iter().all(
            |(action, payload)| action == "ClearChargingProfile" && payload.get("id").is_some()
        ));
    }

    #[tokio::test]
    async fn priority_change_reclaims_power_only_after_ack_and_a_new_measurement() {
        let state = state().await;
        prepare(&state).await;
        let messages = mock(&state, Arc::new(AtomicBool::new(false)));
        reading(&state, -6000, 0).await;
        reconcile(&state).await.unwrap();
        reading(&state, -6000, 0).await;
        reconcile(&state).await.unwrap();
        assert_eq!(state.energy.runtime.lock().await.applied[&1].watts, 11040);
        sqlx::query(
            "INSERT INTO chips(id, id_tag, kind, charging_priority) VALUES(2, 'guest', 'guest', 1)",
        )
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query("INSERT INTO transactions(wallbox_id, connector_id, id_tag, chip_id, start_time, start_meter_wh)
            VALUES(1, 2, 'guest', 2, datetime('now'), 0)").execute(&state.db).await.unwrap();
        // 6 kW Gesamtbudget, vollstaendig Ueberschuss: nur der Gast darf laden.
        let settings = Settings {
            enabled: true,
            grid_limit_w: 1,
            reserve_w: 0,
            ..Settings::default()
        };
        db::setting_set(
            &state.db,
            "energy",
            &serde_json::to_string(&settings).unwrap(),
        )
        .await
        .unwrap();
        reading(&state, 5040, 11040).await;
        messages.lock().await.clear();
        reconcile(&state).await.unwrap();
        {
            let rt = state.energy.runtime.lock().await;
            assert_eq!(rt.applied[&1].watts, 0);
            assert_eq!(rt.applied[&2].watts, 0);
        }
        reconcile(&state).await.unwrap();
        assert_eq!(state.energy.runtime.lock().await.applied[&2].watts, 0);
        // Ein neuer Zeitstempel reicht nicht, solange die Autos die Senkung
        // laut Gesamtmessung noch nicht umgesetzt haben.
        reading(&state, 5040, 11040).await;
        reconcile(&state).await.unwrap();
        assert_eq!(state.energy.runtime.lock().await.applied[&2].watts, 0);
        reading(&state, -6000, 0).await;
        reconcile(&state).await.unwrap();
        let rt = state.energy.runtime.lock().await;
        assert_eq!(rt.applied[&1].watts, 0);
        assert_eq!(rt.applied[&2].watts, 6000);
    }

    #[tokio::test]
    async fn uncertain_increase_requires_zero_confirmation_before_retrying() {
        let state = state().await;
        prepare(&state).await;
        let reject = Arc::new(AtomicBool::new(false));
        let messages = mock(&state, reject.clone());
        reading(&state, 130_000, 0).await;
        reconcile(&state).await.unwrap();
        reject.store(true, Ordering::SeqCst);
        reading(&state, 130_000, 0).await;
        reconcile(&state).await.unwrap();
        assert!(state.energy.runtime.lock().await.uncertain.contains(&1));
        reject.store(false, Ordering::SeqCst);
        reading(&state, 130_000, 0).await;
        messages.lock().await.clear();
        reconcile(&state).await.unwrap();
        {
            let rt = state.energy.runtime.lock().await;
            assert!(!rt.uncertain.contains(&1));
            assert_eq!(rt.applied[&1].watts, 0);
        }
        reading(&state, 130_000, 0).await;
        reconcile(&state).await.unwrap();
        assert_eq!(state.energy.runtime.lock().await.applied[&1].watts, 10_000);
    }
}
