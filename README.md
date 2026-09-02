# easy-ocpp

🌐 [English](README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md)

[![CI](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml/badge.svg)](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hilman2/easy-ocpp)](https://github.com/hilman2/easy-ocpp/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**[Project website and documentation](https://hilman2.github.io/easy-ocpp/)**

Self-hosted **OCPP server (CSMS)** for EV charging stations, built for **small businesses with 1–10 wallboxes**. Charge point management, RFID chip handling, charging limits and reports, without a cloud subscription.

- **One binary, one SQLite file** – no external database, no message broker.
- **OCPP 1.6J** (complete) + **OCPP 2.0.1** (WebSocket scaffold, BootNotification / TransactionEvent) + **OCPP 1.5 SOAP** (scaffold for Boot/Heartbeat).
- **Modern web UI** (Askama + htmx), local accounts with Argon2 passwords.
- Accounts and passwords live in the SQLite file. There is no directory or single sign-on integration.
- Runs on **Windows** (primary focus) **and Linux**.

## Features

| Area            | Status |
|-----------------|--------|
| Wallbox inventory + status (online/offline, connectors, firmware) | ✅ |
| RFID chip management, enrollment via "learning window" (2 min, Authorize intercept) | ✅ |
| Chip → user assignment; a chip without a user is a guest chip | ✅ |
| Remote unlocking for guests, billing to guest label | ✅ |
| Live values during charging (charged kWh, current power, SoC) | ✅ |
| Transaction list, filter by user | ✅ |
| Statistics by month / quarter / year | ✅ |
| Users **are** the employees: chips, sessions and limits hang off the user | ✅ |
| Charging limits per session: target kWh and/or timer, automatic stop | ✅ |
| Per-user defaults for those limits, set by the employee or the admin | ✅ |
| Employee self-service: own sessions live, own limits, stop own session | ✅ |
| Password change, required by an administrator or done by the user | ✅ |
| Monthly report sent by email to whoever charged | ✅ |
| Wallbox messages (status and faults) kept for 60 days, per session and per wallbox | ✅ |
| Multilingual UI (Deutsch, English, Français, Español) | ✅ |

## Getting started

**Prebuilt binaries** (Windows x64, Linux x64) are available under
[Releases](https://github.com/hilman2/easy-ocpp/releases/latest). Unpack,
run `easy-ocpp.exe` or `easy-ocpp`, done (see `INSTALL.md` in the package).

Or build it yourself:

```bash
# once: create a config (optional)
copy config.example.toml config.toml

# Build + start
cargo run --release
```

Then open the UI at <http://localhost:8080>. **Default login on first start:** `admin` / `admin` – please change the password right away under "Users".

Forgot the admin password?

```bash
cargo run --release -- --reset-admin "newPassword123"
```

## Configuring a wallbox

Wallboxes establish a WebSocket connection:

```
ws://<host>:8080/ocpp/<ChargePointId>
```

Subprotocols are negotiated automatically (`ocpp1.6` or `ocpp2.0.1`).

For older OCPP 1.5 devices (SOAP):

```
POST http://<host>:8080/ocpp15
```

### Live meter readings during charging

When an OCPP 1.6 wallbox connects, the server automatically configures it to
report the meter reading, power, and (if available) SoC every 30 seconds during
a charging session (`MeterValueSampleInterval`, `MeterValuesSampledData`).
The interval can be adjusted via `config.toml`; `0` disables the
auto-configuration:

```toml
[ocpp]
meter_interval_s = 30
```

The cockpit and the wallbox detail page automatically refresh active charging
sessions every 10 seconds (htmx polling). If a wallbox does not report power,
it is derived from the last two meter readings.

## Roles and permissions

There is exactly one kind of person in the system: a **user**. An employee is a
user with `role = user`, and their chips, charging sessions and limits hang off
that account directly. There is no separate employee record.

| | Admin | Employee |
|---|---|---|
| Cockpit, wallboxes, chips, user management | ✅ | – |
| Charging sessions of everyone | ✅ | – |
| Own charging sessions (list, CSV, statistics, monthly PDF) | ✅ | ✅ |
| Live view of own running session, stop it | ✅ | ✅ |
| Target kWh / timer on own running session | ✅ | ✅ |
| Default limits, own | ✅ | ✅ |
| Default limits of any employee | ✅ | – |

An employee lands on **`/me`** after logging in, not on the cockpit; the
navigation only shows what they may actually open. Restrictions are enforced
server-side, not just hidden in the UI.

Employees created before this account model existed are carried over **without a
password**. Their charging is recorded, but they cannot sign in until an admin
sets one under "Users".

## Charging limits

A charging session stops automatically once it reaches a **target energy** or a
**timer**, whichever comes first. Both are optional; either can be left empty.

- **Defaults per person**: the employee sets them on their own page, the admin on
  the user detail page. They are copied into every new session at start, with the
  timer counting from the start of charging.
- **Per running session**: the values can still be changed while charging. There
  the timer means "stop in N minutes from now", which is what you want when
  standing in front of the wallbox.

A watchdog checks every 15 seconds and sends `RemoteStopTransaction`. The energy
limit is additionally checked immediately whenever new meter values arrive, so
charging does not continue until the next tick. If the wallbox rejects the stop,
it is retried on the next tick. The session is only marked as handled after the
wallbox has accepted.

## Data storage

Everything lives in **one SQLite file** at `data/easy-ocpp.db` (changeable via `config.toml`). Migrations reside in `migrations/` and are applied automatically at startup.

### Sanity checks on incoming data

- **Timestamps**: >24 h in the future or >10 years in the past are discarded – fallback to the server clock.
- **StartTransaction / StopTransaction**: Idempotent against repeats; decreasing meter values are corrected.
- **StatusNotification**: UPSERT per (wallbox, connector) – no duplicates.
- **MeterValues**: negative values are discarded, SoC validated to 0–100 %, kWh → Wh normalized.
- **Enrollment**: A newly captured tag is assigned to exactly one open learning-window session.

## Project layout

```
src/
  main.rs           – entry point, Tokio runtime, SQLite pool
  config.rs         – TOML configuration
  db.rs             – bootstrap + helpers (Argon2, settings)
  error.rs          – AppError / IntoResponse
  auth/             – session cookies + local login
  domain/           – data models (FromRow)
  ocpp/
    wire.rs         – OCPP JSON frame parser
    hub.rs          – registry of all active connections
    ocpp16.rs       – OCPP 1.6J (complete)
    ocpp20.rs       – OCPP 2.0.1 (bootstrap)
    soap15.rs       – OCPP 1.5 SOAP endpoint
    limits.rs       – watchdog for target kWh / timer
  web/              – axum router, Askama views
templates/          – HTML templates (Askama)
static/             – CSS + htmx shim (embedded via rust-embed)
migrations/         – SQLite migrations
```

## License

MIT
