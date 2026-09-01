use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Ein Benutzer ist zugleich der Mitarbeiter. Seit Migration 0003 gibt es
/// keine getrennte `employees`-Tabelle mehr. `role = 'user'` ist ein normaler
/// Mitarbeiter, der ausschliesslich seine eigenen Ladungen verwalten darf.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub department: Option<String>,
    pub role: String,
    pub auth_source: String,
    pub password_hash: Option<String>,
    pub external_id: Option<String>,
    pub disabled: i64,
    /// 1 = beim naechsten Aufruf muss ein neues Passwort gesetzt werden.
    pub must_change_password: i64,
    /// Standard-Energielimit in Wh, das beim Start einer Ladung uebernommen wird.
    pub default_limit_wh: Option<i64>,
    /// Standard-Zeitlimit in Minuten, das beim Start einer Ladung uebernommen wird.
    pub default_limit_minutes: Option<i64>,
    pub created_at: String,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
    pub fn is_disabled(&self) -> bool {
        self.disabled != 0
    }
    /// Ohne Passwort-Hash ist kein lokaler Login moeglich. So entstehen die
    /// aus `employees` uebernommenen Konten, bis der Admin ein Passwort vergibt.
    pub fn has_login(&self) -> bool {
        self.password_hash.is_some() || self.auth_source != "local"
    }
    /// Standard-Energielimit als kWh-Zahl fuer Formularfelder ("" = kein Limit).
    pub fn default_limit_kwh_str(&self) -> String {
        match self.default_limit_wh {
            Some(wh) => format!("{:.1}", wh as f64 / 1000.0),
            None => String::new(),
        }
    }
    pub fn default_limit_minutes_str(&self) -> String {
        match self.default_limit_minutes {
            Some(m) => m.to_string(),
            None => String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub token: String,
    pub user_id: i64,
    pub created_at: String,
    pub expires_at: String,
}

pub fn session_expires_at() -> DateTime<Utc> {
    Utc::now() + chrono::Duration::hours(12)
}
