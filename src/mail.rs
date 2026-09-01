//! Monatlicher Versand der Ladeberichte per E-Mail.
//!
//! Am Monatsersten bekommt jeder Benutzer, der im Vormonat geladen hat und
//! eine E-Mail-Adresse hinterlegt hat, seinen eigenen Bericht als PDF. Wer
//! nicht geladen hat, bekommt nichts.
//!
//! Der Versand ist ausgeschaltet, solange in der `config.toml` kein
//! `[mail]`-Abschnitt steht. Was hinausgegangen ist, steht in `report_mails`.
//! Ohne dieses Protokoll wuerde ein Neustart oder ein zweiter Anlauf nach
//! einem Teilfehler dieselbe Mail erneut verschicken.

use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{Datelike, Local, Timelike};
use lettre::message::{header::ContentType, Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::{MailConfig, MailSecurity};
use crate::i18n::Lang;
use crate::web::views::reports;
use crate::AppState;

/// Pruefintervall. Der Versand haengt an einem Datum, nicht an einer Uhrzeit
/// auf die Minute genau, deshalb reicht ein grober Takt.
const TICK: Duration = Duration::from_secs(15 * 60);

pub fn spawn(state: AppState) {
    if state.config.mail.is_none() {
        tracing::info!("Kein [mail]-Abschnitt konfiguriert, kein Monatsversand.");
        return;
    }
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(TICK);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            if let Err(e) = run_once(&state).await {
                tracing::warn!("Monatsversand: {e:#}");
            }
        }
    });
}

/// Prueft, ob der Bericht des Vormonats faellig ist, und verschickt ihn.
pub async fn run_once(state: &AppState) -> Result<()> {
    let Some(cfg) = state.config.mail.as_ref() else {
        return Ok(());
    };

    let now = Local::now();
    // Am Ersten wird zur eingestellten Stunde verschickt. Lief der Server an
    // diesem Tag nicht, wird spaeter im Monat nachgeholt, statt den Bericht
    // ganz ausfallen zu lassen.
    let faellig = now.day() > 1 || now.hour() >= cfg.send_hour;
    if !faellig {
        return Ok(());
    }

    let (year, month) = vormonat(now.year(), now.month());
    let period = format!("{year:04}-{month:02}");
    let lang = Lang::from_code(&cfg.lang).unwrap_or(Lang::De);

    // Nur Personen mit Adresse, die im Zeitraum ueberhaupt geladen haben.
    let offen = reports::collect(state, year, month, None, lang)
        .await
        .map_err(|e| anyhow::anyhow!("Bericht {period}: {e}"))?;

    let mut versendet = 0usize;
    for person in offen {
        let Some(adresse) = person.email.clone().filter(|a| !a.trim().is_empty()) else {
            continue;
        };
        if bereits_versendet(state, person.id, &period).await? {
            continue;
        }

        let pdf = reports::build_pdf(year, month, std::slice::from_ref(&person), lang)
            .with_context(|| format!("PDF fuer {} ({period})", person.name))?;

        match sende(cfg, lang, &adresse, &person, year, month, pdf).await {
            Ok(()) => {
                merke_versendet(state, person.id, &period).await?;
                versendet += 1;
            }
            // Ein fehlgeschlagener Empfaenger darf die uebrigen nicht aufhalten.
            // Ohne Eintrag in report_mails wird es beim naechsten Takt erneut
            // versucht.
            Err(e) => tracing::warn!("Bericht {period} an {adresse}: {e:#}"),
        }
    }

    if versendet > 0 {
        tracing::info!("Monatsbericht {period}: {versendet} Mail(s) verschickt.");
    }
    Ok(())
}

fn vormonat(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

async fn bereits_versendet(state: &AppState, user_id: i64, period: &str) -> Result<bool> {
    let (n,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM report_mails WHERE user_id = ?1 AND period = ?2")
            .bind(user_id)
            .bind(period)
            .fetch_one(&state.db)
            .await?;
    Ok(n > 0)
}

async fn merke_versendet(state: &AppState, user_id: i64, period: &str) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO report_mails (user_id, period) VALUES (?1, ?2)")
        .bind(user_id)
        .bind(period)
        .execute(&state.db)
        .await?;
    Ok(())
}

async fn sende(
    cfg: &MailConfig,
    lang: Lang,
    adresse: &str,
    person: &reports::Person,
    year: i32,
    month: u32,
    pdf: Vec<u8>,
) -> Result<()> {
    let monatsname = lang.t(monat_key(month));
    let betreff = format!("{} {} {}", lang.t("mail.subject"), monatsname, year);
    let kwh = format!("{:.1}", person.total_wh as f64 / 1000.0).replace('.', ",");

    let text = format!(
        "{anrede} {name},\n\n{satz1}\n\n{ladungen}: {anzahl}\n{energie}: {kwh} kWh\n\n{schluss}\n",
        anrede = lang.t("mail.greeting"),
        name = person.name,
        satz1 = lang.t("mail.body").replace("{month}", &format!("{monatsname} {year}")),
        ladungen = lang.t("common.sessions"),
        anzahl = person.session_count(),
        energie = lang.t("common.energy_kwh"),
        kwh = kwh,
        schluss = lang.t("mail.footer"),
    );

    let dateiname = format!("{}_{year:04}-{month:02}.pdf", lang.t("pdf.filename"));
    let anhang = Attachment::new(dateiname).body(pdf, ContentType::parse("application/pdf")?);

    let mail = Message::builder()
        .from(cfg.from.parse().context("mail.from ist keine gueltige Adresse")?)
        .to(adresse
            .parse()
            .with_context(|| format!("Empfaengeradresse ungueltig: {adresse}"))?)
        .subject(betreff)
        .multipart(
            MultiPart::mixed()
                .singlepart(SinglePart::plain(text))
                .singlepart(anhang),
        )?;

    transport(cfg)?.send(mail).await.context("SMTP")?;
    Ok(())
}

fn transport(cfg: &MailConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let mut builder = match cfg.security {
        MailSecurity::None => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.smtp_host)
        }
        MailSecurity::Starttls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp_host)
                .context("SMTP-Transport (STARTTLS)")?
        }
        MailSecurity::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)
            .context("SMTP-Transport (TLS)")?,
    }
    .port(cfg.smtp_port);

    if let (Some(user), Some(pass)) = (cfg.username.as_ref(), cfg.password.as_ref()) {
        builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
    }
    Ok(builder.build())
}

fn monat_key(month: u32) -> &'static str {
    match month {
        1 => "month.1",
        2 => "month.2",
        3 => "month.3",
        4 => "month.4",
        5 => "month.5",
        6 => "month.6",
        7 => "month.7",
        8 => "month.8",
        9 => "month.9",
        10 => "month.10",
        11 => "month.11",
        _ => "month.12",
    }
}
