# easy-ocpp

🌐 [English](README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md)

[![CI](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml/badge.svg)](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hilman2/easy-ocpp)](https://github.com/hilman2/easy-ocpp/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Selbst gehosteter **OCPP-Server (CSMS)** für Ladestationen, gebaut für **KMUs mit 1–10 Wallboxen**. Ladepunkt-Verwaltung, RFID-Chips, Ladelimits und Auswertungen — ohne Cloud-Abo.

- **Ein Binary, eine SQLite-Datei** – keine externe Datenbank, kein Message-Broker.
- **OCPP 1.6J** (vollständig) + **OCPP 2.0.1** (WebSocket-Gerüst, BootNotification / TransactionEvent) + **OCPP 1.5 SOAP** (Gerüst für Boot/Heartbeat).
- **Moderne Web-UI** (Askama + htmx), lokale Benutzer mit Argon2-Passwörtern.
- Active Directory (LDAP) und Microsoft Entra (OIDC) sind als Konfigurations-Felder vorbereitet – die konkreten Bind-/Flow-Implementierungen folgen.
- Läuft auf **Windows** (Fokus) **und Linux**.

## Features

| Bereich         | Status |
|-----------------|--------|
| Wallbox-Inventar + Status (online/offline, Connectors, Firmware) | ✅ |
| Chip-Verwaltung, Anlernen via „Lernfenster" (2 min, Authorize-Intercept) | ✅ |
| Chip → Benutzer-Zuordnung; ein Chip ohne Benutzer ist ein Gast-Chip | ✅ |
| Remote-Freischalten für Gäste, Buchung auf Gast-Label | ✅ |
| Live-Werte während der Ladung (geladene kWh, aktuelle Leistung, SoC) | ✅ |
| Transaktionsliste, Filter nach Benutzer | ✅ |
| Statistik nach Monat / Quartal / Jahr | ✅ |
| Benutzer **sind** die Mitarbeiter — Chips, Ladungen und Limits hängen am Konto | ✅ |
| Ladelimits je Ladung: Ziel-kWh und/oder Timer, automatische Abschaltung | ✅ |
| Standardvorgaben dafür je Person, gesetzt vom Mitarbeiter oder vom Admin | ✅ |
| Mitarbeiter-Selbstbedienung: eigene Ladung live, eigene Limits, selbst beenden | ✅ |
| Mehrsprachige UI (Deutsch, English, Français, Español) | ✅ |
| Active Directory (LDAP) | 🟡 Konfig vorbereitet |
| Entra ID (OIDC)        | 🟡 Konfig vorbereitet |

## Starten

**Fertige Binaries** (Windows x64, Linux x64) gibt es unter
[Releases](https://github.com/hilman2/easy-ocpp/releases/latest) — entpacken,
`easy-ocpp.exe` bzw. `easy-ocpp` starten, fertig (siehe `ANLEITUNG.md` im Paket).

Oder selbst bauen:

```bash
# einmalig: Konfig anlegen (optional)
copy config.example.toml config.toml

# Build + Start
cargo run --release
```

Danach die UI unter <http://localhost:8080> öffnen. **Default-Login beim ersten Start:** `admin` / `admin` – Passwort bitte direkt unter „Benutzer" ändern.

Admin-Passwort vergessen?

```bash
cargo run --release -- --reset-admin "neuesPasswort123"
```

## Wallbox konfigurieren

Wallboxen stellen eine WebSocket-Verbindung her:

```
ws://<host>:8080/ocpp/<ChargePointId>
```

Subprotokolle werden automatisch ausgehandelt (`ocpp1.6` oder `ocpp2.0.1`).

Für ältere OCPP-1.5-Geräte (SOAP):

```
POST http://<host>:8080/ocpp15
```

### Live-Messwerte während der Ladung

Beim Verbinden einer OCPP-1.6-Wallbox konfiguriert der Server sie automatisch so,
dass sie während einer Ladung alle 30 Sekunden Zählerstand, Leistung und (falls
verfügbar) SoC meldet (`MeterValueSampleInterval`, `MeterValuesSampledData`).
Das Intervall ist über `config.toml` einstellbar, `0` deaktiviert die
Auto-Konfiguration:

```toml
[ocpp]
meter_interval_s = 30
```

Cockpit und Wallbox-Detailseite aktualisieren die laufenden Ladungen alle
10 Sekunden automatisch (htmx-Polling). Meldet eine Wallbox keine Leistung,
wird sie aus den letzten beiden Zählerständen abgeleitet.

## Rollen und Rechte

Es gibt genau eine Sorte Person: den **Benutzer**. Ein Mitarbeiter ist ein
Benutzer mit `role = user`; seine Chips, Ladungen und Limits hängen direkt an
diesem Konto. Einen separaten Mitarbeiter-Datensatz gibt es nicht mehr.

| | Admin | Mitarbeiter |
|---|---|---|
| Cockpit, Wallboxen, Chips, Benutzerverwaltung | ✅ | — |
| Ladungen aller Personen | ✅ | — |
| Eigene Ladungen (Liste, CSV, Statistik, Monats-PDF) | ✅ | ✅ |
| Eigene laufende Ladung live sehen und beenden | ✅ | ✅ |
| Ziel-kWh / Timer an der eigenen laufenden Ladung | ✅ | ✅ |
| Standardvorgaben — eigene | ✅ | ✅ |
| Standardvorgaben — die jedes Mitarbeiters | ✅ | — |

Ein Mitarbeiter landet nach dem Login auf **`/me`**, nicht im Cockpit; die
Navigation zeigt ihm nur, was er auch öffnen darf. Die Einschränkungen greifen
serverseitig, nicht bloß durch Ausblenden in der Oberfläche.

Mitarbeiter aus der Zeit vor diesem Kontomodell werden **ohne Passwort**
übernommen — ihre Ladungen werden erfasst, anmelden können sie sich erst, wenn
ein Admin unter „Benutzer" eines vergibt.

## Ladelimits

Eine Ladung wird automatisch beendet, sobald sie eine **Ziel-Energie** oder einen
**Timer** erreicht — was zuerst eintritt. Beides ist optional und einzeln
abschaltbar.

- **Standardvorgaben je Person**: der Mitarbeiter setzt sie auf seiner eigenen
  Seite, der Admin auf der Benutzer-Detailseite. Sie werden beim Start jeder
  Ladung übernommen, der Timer zählt dann ab Ladebeginn.
- **An der laufenden Ladung**: die Werte lassen sich während des Ladens noch
  ändern. Dort bedeutet der Timer „in N Minuten abschalten" — das, was man
  meint, wenn man vor der Wallbox steht.

Ein Watchdog prüft alle 15 Sekunden und schickt `RemoteStopTransaction`. Das
Energielimit wird zusätzlich sofort beim Eintreffen neuer Messwerte geprüft,
damit nicht bis zum nächsten Takt weitergeladen wird. Lehnt die Wallbox den Stop
ab, wird beim nächsten Takt erneut versucht — abgehakt wird die Ladung erst,
wenn die Wallbox angenommen hat.

## Datenhaltung

Alles liegt in **einer SQLite-Datei** unter `data/easy-ocpp.db` (über `config.toml` änderbar). Migrationen liegen in `migrations/` und werden beim Start automatisch angewendet.

### Sanity-Checks beim Datenempfang

- **Timestamps**: >24 h in der Zukunft oder >10 Jahre in der Vergangenheit werden verworfen – Fallback auf die Server-Uhrzeit.
- **StartTransaction / StopTransaction**: Idempotent gegen Wiederholungen; rückläufige Meter-Werte werden korrigiert.
- **StatusNotification**: UPSERT pro (Wallbox, Connector) – keine Duplikate.
- **MeterValues**: negative Werte werden verworfen, SoC auf 0–100 % validiert, kWh → Wh normalisiert.
- **Enrollment**: Ein neu erfasster Tag wird genau einer offenen Lernfenster-Session zugeordnet.

## Projekt-Layout

```
src/
  main.rs           – Einstiegspunkt, Tokio-Runtime, SQLite-Pool
  config.rs         – TOML-Konfiguration
  db.rs             – Bootstrap + Helpers (Argon2, Settings)
  error.rs          – AppError / IntoResponse
  auth/             – Session-Cookies + lokaler Login
  domain/           – Datenmodelle (FromRow)
  ocpp/
    wire.rs         – OCPP-JSON-Frame-Parser
    hub.rs          – Registry aller aktiven Verbindungen
    ocpp16.rs       – OCPP 1.6J (vollständig)
    ocpp20.rs       – OCPP 2.0.1 (Bootstrap)
    soap15.rs       – OCPP 1.5 SOAP-Endpunkt
    limits.rs       – Watchdog für Ziel-kWh / Timer
  web/              – axum-Router, Askama-Views
templates/          – HTML-Templates (Askama)
static/             – CSS + htmx-Shim (embedded via rust-embed)
migrations/         – SQLite-Migrationen
```

## Lizenz

MIT
