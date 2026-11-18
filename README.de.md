# easy-ocpp

🌐 [English](README.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md)

[![CI](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml/badge.svg)](https://github.com/hilman2/easy-ocpp/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hilman2/easy-ocpp)](https://github.com/hilman2/easy-ocpp/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**[Projektseite und Dokumentation](https://hilman2.github.io/easy-ocpp/de/)**

Selbst gehosteter **OCPP-Server (CSMS)** für Ladestationen, gebaut für **KMUs mit 1–10 Wallboxen**. Ladepunkt-Verwaltung, RFID-Chips, Ladelimits und Auswertungen, ohne Cloud-Abo.

- **Ein Binary, eine SQLite-Datei** – keine externe Datenbank, kein Message-Broker.
- **OCPP 1.6J** (vollständig) + **OCPP 2.0.1** (WebSocket-Gerüst, BootNotification / TransactionEvent) + **OCPP 1.5 SOAP** (Gerüst für Boot/Heartbeat).
- **Moderne Web-UI** (Askama + htmx), lokale Konten mit Argon2-Passwörtern.
- Konten und Passwörter liegen in der SQLite-Datei. Eine Anbindung an ein Verzeichnis oder an Single Sign-on gibt es nicht.
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
| Benutzer **sind** die Mitarbeiter: Chips, Ladungen und Limits hängen am Konto | ✅ |
| Ladelimits je Ladung: Ziel-kWh und/oder Timer, automatische Abschaltung | ✅ |
| Standardvorgaben dafür je Person, gesetzt vom Mitarbeiter oder vom Admin | ✅ |
| Mitarbeiter-Selbstbedienung: eigene Ladung live, eigene Limits, selbst beenden | ✅ |
| Passwortwechsel, vom Administrator verlangt oder selbst ausgelöst | ✅ |
| Monatsbericht per E-Mail an alle, die geladen haben | ✅ |
| Wallbox-Meldungen (Status und Störungen), 60 Tage aufgehoben, je Ladung und je Wallbox | ✅ |
| Mehrsprachige UI (Deutsch, English, Français, Español) | ✅ |

## Starten

**Fertige Binaries** (Windows x64, Linux x64) gibt es unter
[Releases](https://github.com/hilman2/easy-ocpp/releases/latest). Entpacken,
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
| Cockpit, Wallboxen, Chips, Benutzerverwaltung | ✅ | – |
| Ladungen aller Personen | ✅ | – |
| Eigene Ladungen (Liste, CSV, Statistik, Monats-PDF) | ✅ | ✅ |
| Eigene laufende Ladung live sehen und beenden | ✅ | ✅ |
| Ziel-kWh / Timer an der eigenen laufenden Ladung | ✅ | ✅ |
| Standardvorgaben, eigene | ✅ | ✅ |
| Standardvorgaben jedes Mitarbeiters | ✅ | – |

Ein Mitarbeiter landet nach dem Login auf **`/me`**, nicht im Cockpit; die
Navigation zeigt ihm nur, was er auch öffnen darf. Die Einschränkungen greifen
serverseitig, nicht bloß durch Ausblenden in der Oberfläche.

Mitarbeiter aus der Zeit vor diesem Kontomodell werden **ohne Passwort**
übernommen. Ihre Ladungen werden erfasst, anmelden können sie sich erst, wenn
ein Admin unter „Benutzer" eines vergibt.

## Ladelimits

Eine Ladung wird automatisch beendet, sobald sie eine **Ziel-Energie** oder einen
**Timer** erreicht, je nachdem was zuerst eintritt. Beides ist optional und einzeln
abschaltbar.

- **Standardvorgaben je Person**: der Mitarbeiter setzt sie auf seiner eigenen
  Seite, der Admin auf der Benutzer-Detailseite. Sie werden beim Start jeder
  Ladung übernommen, der Timer zählt dann ab Ladebeginn.
- **An der laufenden Ladung**: die Werte lassen sich während des Ladens noch
  ändern. Dort bedeutet der Timer „in N Minuten abschalten". Das meint man,
  meint, wenn man vor der Wallbox steht.

Ein Watchdog prüft alle 15 Sekunden und schickt `RemoteStopTransaction`. Das
Energielimit wird zusätzlich sofort beim Eintreffen neuer Messwerte geprüft,
damit nicht bis zum nächsten Takt weitergeladen wird. Lehnt die Wallbox den Stop
ab, wird beim nächsten Takt erneut versucht. Abgehakt wird die Ladung erst,
wenn die Wallbox angenommen hat.

## Überschussladen und Netzbezug begrenzen

Unter **Überschussladen** verwaltet der Administrator die optionale Regelung.
Sie ist zunächst ausgeschaltet. Normale Chips laden nur mit Überschuss;
**GF / Gäste** haben Vorrang. **Netzstrom erlaubt** ist eine getrennte Freigabe
pro Chip für beispielsweise Firmenautos. Sie umgeht niemals das Anschlusslimit.
Bestehende und neue Gast-Chips erhalten hohe Priorität, aber keine Netzfreigabe.
Gleichrangige Autos teilen das Budget, sofern ihre Mindestladeleistung reicht.
Sonst warten einzelne Autos mit 0 W und laufen bei mehr Leistung wieder an.

Das Netzlimit ist mit **150000 W (150 kW)** vorbelegt, der einstellbare Puffer
mit **10000 W**. Daraus ergibt sich ein Regelziel von höchstens 140 kW Bezug.
Gezählt wird der gesamte Standort einschließlich Gebäudelast. Sind allein die
anderen Verbraucher schon über dem Ziel, werden alle Autos pausiert.
Hardwarewerte gelten je Anschluss: Mindest-/Maximalleistung, Phasen und
unterstützte OCPP-Einheit vor Aktivierung unter **Überschussladen** prüfen.
Bei Stromvorgaben wird konservativ abgerundet und mit 230 V je Phase gerechnet.

Die Messwertquelle ist austauschbar. Ein späterer Zähleradapter sendet alle
wenigen Sekunden über HTTPS an `POST /api/energy/meter`, mit
`Authorization: Bearer <meter_token>`. Den zufälligen Token mit mindestens
24 Zeichen unter `[energy]` in `config.toml` setzen und den Server neu starten.
Beispielinhalt (Zeitstempel durch den tatsächlichen Messzeitpunkt ersetzen):

```json
{"timestamp":"2026-10-15T10:00:00Z","grid_power_w":-5000,"charging_power_w":10000}
```

`grid_power_w` ist die vorzeichenbehaftete Leistung am Netzanschluss: positiv
für Bezug, negativ für Einspeisung. `charging_power_w` ist die **zeitgleich
gemessene Summe aller verwalteten Ladeanschlüsse**, bereits in der Netzleistung
enthalten. Keine Energiezählerstände, Sollwerte oder ungleichzeitig abgelesenen
OCPP-Werte einsetzen. Im Beispiel sind ohne Autos 15 kW Überschuss verfügbar.
Der Adapter muss beide Messungen synchron zusammenführen; Modbus/MQTT oder eine
Hersteller-API sind noch nicht angebunden. Alte, wiederholte oder zukünftige
Zeitstempel werden abgewiesen. Messwerte bleiben nur im Arbeitsspeicher und
müssen nach einem Serverneustart neu eintreffen.

Ohne frische Messwerte (standardmäßig höchstens 15 Sekunden alt) wird pausiert;
neue Starts bleiben gesperrt. Mit frischen Werten kann eine autorisierte
Transaktion zunächst mit 0 W auf Überschuss warten. Ein Null-Standardprofil
schützt den Start. Die Regelung senkt zuerst bestehende Freigaben und wartet
vor einer Umverteilung auf Bestätigung und eine neue Messung. Abgelehnte
Profile geben kein Budget frei; zusätzlich wird ein Stop versucht. Freigaben
fallen nach spätestens 30 Sekunden ohne Erneuerung auf 0 zurück.
Die Regelung deaktiviert lokale Vorautorisierung, Offline-Autorisierung und
den Autorisierungscache; diese Einstellungen bleiben beim Ausschalten erhalten.
Ausschalten entfernt gezielt die eigenen Ladeprofile und erlaubt normales Laden.
Ein nie aktiviertes System verändert keine Wallbox-Einstellungen.

**Voraussetzung:** OCPP 1.6J mit getesteter Smart-Charging-Unterstützung,
Null-Limits und korrekter Stationszeit. Andere Protokolle werden bei aktiver
Regelung gesperrt. Die Profile sind mit einem Simulator getestet, müssen aber
mit den konkreten Wallboxen abgenommen werden. OCPP und ein Puffer garantieren
keine harte 150-kW-Grenze bei Lastsprüngen oder Kommunikationsausfall. Dafür
ist eine schnelle lokale Messung und ein lokaler Lastregler erforderlich.

## Docker

```powershell
Copy-Item config.example.toml config.toml
docker compose up -d --build
```

Die Oberfläche ist unter <http://localhost:8080> erreichbar. Die Datenbank liegt
im persistenten Volume `easy-ocpp-data`; `config.toml` wird nur lesbar eingebunden.
`docker compose down` erhält die Daten, `docker compose down -v` löscht sie.

### Hinter einem Reverse-Proxy

In `deploy/` liegt ein Aufbau für einen Host, auf dem ein Caddy mehrere Dienste
bedient. `compose.yaml` hängt easy-ocpp ohne eigenen Port in das Docker-Netz des
Proxys, `easy-ocpp.caddy.example` ist die passende Datei für Caddy.

Zwei Dinge sind für die Wallboxen wichtig. Die meisten sprechen OCPP ohne TLS,
`ws://` über Port 80 muss also direkt bei easy-ocpp ankommen: Ein OCPP-Client
folgt keiner Umleitung auf HTTPS. Das Beispiel lässt auf Port 80 nur `/ocpp/*`
und `/ocpp15` durch und leitet alles andere um. Außerdem trennt ein Reload des
Proxys offene WebSockets sofort, wenn man ihm keine Frist gibt. Das erledigt
`stream_close_delay`.

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
