use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod auth;
mod config;
mod db;
mod domain;
mod error;
mod i18n;
mod mail;
mod ocpp;
mod web;

pub use error::{AppError, AppResult};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub ocpp_hub: Arc<ocpp::hub::Hub>,
    pub config: Arc<config::Config>,
}

#[derive(Parser, Debug)]
#[command(name = "easy-ocpp", about = "Wallbox-Management-Tool (OCPP)")]
struct Cli {
    /// Pfad zur Konfigurationsdatei (TOML).
    #[arg(long, default_value = "config.toml")]
    config: PathBuf,
    /// Admin-Passwort zurücksetzen (setzt auf das angegebene Passwort).
    #[arg(long, value_name = "PASSWORT")]
    reset_admin: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn")))
        .with(fmt::layer().with_target(false))
        .init();

    let cli = Cli::parse();
    let cfg = config::Config::load(&cli.config)?;
    std::fs::create_dir_all(cfg.data_dir())
        .with_context(|| format!("Konnte Datenverzeichnis {:?} nicht anlegen", cfg.data_dir()))?;

    if cfg.using_legacy_db() {
        tracing::warn!(
            "Datenbank {} aus der Zeit vor der Umbenennung wird weiterverwendet. \
             Zum Umstellen das Programm stoppen und die Datei in {} umbenennen.",
            config::LEGACY_DB_FILE,
            cfg.storage.db_file
        );
    }
    let db_url = format!("sqlite://{}", cfg.db_path().to_string_lossy().replace('\\', "/"));
    tracing::info!("Datenbank: {}", db_url);

    let opts = SqliteConnectOptions::new()
        .filename(cfg.db_path())
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .foreign_keys(true);

    let db = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await
        .context("SQLite-Pool konnte nicht geöffnet werden")?;

    let migrator = sqlx::migrate!("./migrations");
    db::repair_line_ending_checksums(&db, &migrator).await?;
    migrator
        .run(&db)
        .await
        .context("Migrationen fehlgeschlagen")?;

    db::bootstrap_admin(&db, cli.reset_admin.as_deref()).await?;

    let state = AppState {
        db: db.clone(),
        ocpp_hub: Arc::new(ocpp::hub::Hub::default()),
        config: Arc::new(cfg.clone()),
    };

    // Wacht ueber Timer und Ziel-kWh der laufenden Ladungen.
    ocpp::limits::spawn_watchdog(state.clone());

    // Verschickt die Monatsberichte, sofern [mail] konfiguriert ist.
    mail::spawn(state.clone());

    // Raeumt alte Wallbox-Meldungen weg: einmal beim Start, danach taeglich.
    {
        let st = state.clone();
        let tage = cfg.storage.event_retention_days;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(24 * 3600));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                match db::prune_connector_events(&st.db, tage).await {
                    Ok(n) if n > 0 => tracing::info!("{n} alte Wallbox-Meldungen geloescht."),
                    Ok(_) => {}
                    Err(e) => tracing::warn!("Aufraeumen der Wallbox-Meldungen: {e:#}"),
                }
            }
        });
    }

    let http_addr: SocketAddr = cfg.http.bind.parse().context("http.bind ungültig")?;
    let app = web::router(state.clone());

    tracing::info!("HTTP/UI hört auf http://{}", http_addr);
    tracing::info!("OCPP-WebSocket unter ws://<host>:<port>/ocpp/<chargePointId>");

    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .context("HTTP-Server beendet")?;

    Ok(())
}
