use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub ocpp: OcppConfig,
    /// Fehlt der Abschnitt, findet kein Monatsversand statt.
    #[serde(default)]
    pub mail: Option<MailConfig>,
}

/// Verschluesselung der SMTP-Verbindung.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MailSecurity {
    /// Ohne Verschluesselung. Nur fuer einen Relay im eigenen Netz sinnvoll.
    None,
    /// Klartext-Verbindung, die per STARTTLS hochgestuft wird. Ueblich auf 587.
    Starttls,
    /// Von Anfang an verschluesselt. Ueblich auf 465.
    Tls,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MailConfig {
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default = "default_mail_security")]
    pub security: MailSecurity,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    /// Absender, entweder "adresse@example.com" oder "Name <adresse@example.com>".
    pub from: String,
    /// Sprache der Berichtsmails. Die Oberflaechensprache haengt am Browser
    /// und steht fuer einen Versand ohne Besucher nicht zur Verfuegung.
    #[serde(default = "default_mail_lang")]
    pub lang: String,
    /// Stunde am Monatsersten (lokale Zeit), ab der verschickt wird.
    #[serde(default = "default_send_hour")]
    pub send_hour: u32,
}

fn default_smtp_port() -> u16 {
    587
}
fn default_mail_security() -> MailSecurity {
    MailSecurity::Starttls
}
fn default_mail_lang() -> String {
    "de".to_string()
}
fn default_send_hour() -> u32 {
    6
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OcppConfig {
    /// Intervall in Sekunden, in dem Wallboxen während einer Ladung Zählerstand
    /// und Leistung melden sollen (MeterValues). Wird beim Verbinden per
    /// ChangeConfiguration in der Wallbox gesetzt.
    /// 0 = Auto-Konfiguration deaktivieren, Wallbox-Einstellung bleibt unberührt.
    #[serde(default = "default_meter_interval_s")]
    pub meter_interval_s: u32,
}

impl Default for OcppConfig {
    fn default() -> Self {
        Self {
            meter_interval_s: default_meter_interval_s(),
        }
    }
}

fn default_meter_interval_s() -> u32 {
    30
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HttpConfig {
    #[serde(default = "default_bind")]
    pub bind: String,
    #[serde(default = "default_public_base_url")]
    pub public_base_url: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            bind: default_bind(),
            public_base_url: default_public_base_url(),
        }
    }
}

fn default_bind() -> String {
    "0.0.0.0:8080".to_string()
}
fn default_public_base_url() -> String {
    "http://localhost:8080".to_string()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "default_db_file")]
    pub db_file: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            db_file: default_db_file(),
        }
    }
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("data")
}
fn default_db_file() -> String {
    "easy-ocpp.db".to_string()
}

/// Dateiname der Datenbank bis einschliesslich v0.3.1. Damals hiess das
/// Produkt noch "easy-occp" (Dreher im Protokollnamen). Bestehende
/// Installationen behalten ihre Datei, siehe [`Config::db_path`].
pub const LEGACY_DB_FILE: &str = "easy-occp.db";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub ldap: Option<LdapConfig>,
    #[serde(default)]
    pub oidc: Option<OidcConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LdapConfig {
    pub url: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub user_base_dn: String,
    pub user_filter: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::warn!(
                "Keine Konfigurationsdatei unter {:?} gefunden – verwende Defaults.",
                path
            );
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Kann Konfigurationsdatei {:?} nicht lesen", path))?;
        let cfg: Config =
            toml::from_str(&raw).with_context(|| format!("TOML-Fehler in {:?}", path))?;
        Ok(cfg)
    }

    pub fn data_dir(&self) -> &Path {
        &self.storage.data_dir
    }

    /// Pfad zur SQLite-Datei.
    ///
    /// Beim Umbenennen von `easy-occp` auf `easy-ocpp` hat sich der Default-Name
    /// der Datenbank geaendert. Liegt im Datenverzeichnis nur noch die alte
    /// Datei, wird weiter mit ihr gearbeitet. Sonst wuerde ein Update still
    /// eine leere Datenbank anlegen und wie ein Datenverlust aussehen.
    pub fn db_path(&self) -> PathBuf {
        let configured = self.storage.data_dir.join(&self.storage.db_file);
        if configured.exists() || !self.using_legacy_db() {
            return configured;
        }
        self.storage.data_dir.join(LEGACY_DB_FILE)
    }

    /// True, wenn die Datenbank aus der Zeit vor der Umbenennung stammt und
    /// mangels neuer Datei weiterverwendet wird. Nur fuer den Hinweis beim Start.
    pub fn using_legacy_db(&self) -> bool {
        self.storage.db_file != LEGACY_DB_FILE
            && !self.storage.data_dir.join(&self.storage.db_file).exists()
            && self.storage.data_dir.join(LEGACY_DB_FILE).exists()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http: HttpConfig::default(),
            storage: StorageConfig::default(),
            auth: AuthConfig::default(),
            ocpp: OcppConfig::default(),
            mail: None,
        }
    }
}
