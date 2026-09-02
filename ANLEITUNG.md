# easy-ocpp v0.5.0 – Installation & Erststart (Windows)

🌐 [English](INSTALL.md) · [Deutsch](ANLEITUNG.md) · [Français](INSTALL.fr.md) · [Español](INSTALL.es.md)

Management-Tool für Wallboxen (OCPP 1.5/1.6/2.0.1). Ein Binary, eine SQLite-Datei.

## Was ist neu in v0.5.0

- **Passwort selbst aendern.** Jeder Benutzer kann sein Passwort ueber seine
  eigene Seite aendern. Dabei wird das bisherige abgefragt.
- **Passwortwechsel erzwingen.** Wer im Bereich „Benutzer" ein Passwort
  vergibt, kann ankreuzen, dass der Benutzer es beim naechsten Anmelden selbst
  aendern muss. Der Haken ist vorausgewaehlt, weil ein vom Administrator
  vergebenes Passwort ueber einen zweiten Weg weitergegeben wird. Bis der
  Wechsel erfolgt ist, fuehrt jede Seite auf das Wechselformular.
- **Monatsberichte per E-Mail.** Am Monatsersten bekommt jeder Benutzer, der im
  Vormonat geladen hat und eine E-Mail-Adresse hinterlegt hat, seinen Bericht
  als PDF zugeschickt. Wer nicht geladen hat, bekommt nichts. Der Versand ist
  ausgeschaltet, solange kein `[mail]`-Abschnitt in der `config.toml` steht;
  ein Beispiel liegt in `config.example.toml`.
- **Fehler behoben:** Unter „Meine Ladungen" stand bei der gerade laufenden
  Ladung 0 kWh statt des aktuellen Standes. Ausserdem wurden Nachkommastellen
  abgeschnitten, aus 2,7 kWh wurde 2.

**Zum Mailversand:** Lief der Server am Monatsersten nicht, wird der Versand
spaeter im Monat nachgeholt. Wer ihn neu einschaltet, bekommt deshalb einmalig
den Bericht des Vormonats. Das ist zugleich der einfachste Test.

## Was ist neu in v0.4.0

- **Mitarbeiter sind jetzt Benutzer.** Bisher gab es zwei getrennte Begriffe:
  ein Login und daneben eine Mitarbeiter-Karteikarte. Das ist zusammengefuehrt:
  ein Mitarbeiter ist ein Benutzer, seine Chips und Ladungen haengen direkt an
  diesem Konto. Bestehende Mitarbeiter werden uebernommen; wer bisher kein Login
  hatte, bekommt ein Konto ohne Passwort und kann sich erst anmelden, sobald ein
  Administrator unter „Benutzer" eines vergibt.
- **Mitarbeiter sehen nur ihre eigenen Ladungen.** Nach dem Anmelden landen sie
  auf einer eigenen Seite mit den laufenden Ladungen ihrer Chips. Cockpit,
  Wallboxen, Chips und Benutzerverwaltung bleiben dem Administrator vorbehalten.
- **Ladelimits.** Eine Ladung schaltet automatisch ab, sobald sie eine Ziel-kWh
  oder einen Timer erreicht, je nachdem was zuerst eintritt. Jede Person hat
  Standardvorgaben, die fuer jede neue Ladung gelten; setzen kann sie der
  Mitarbeiter selbst oder der Administrator. An der laufenden Ladung sind die
  Werte weiterhin aenderbar, und der Mitarbeiter kann seine Ladung selbst beenden.
- **Chips sind eindeutig.** Ein Chip gehoert entweder zu einer Person oder er ist
  ein Gast-Chip. Die frueher getrennt gepflegte Kategorie ergibt sich jetzt aus
  der Zuordnung und kann ihr nicht mehr widersprechen.
- **Das Programm heisst jetzt `easy-ocpp`**, Einzelheiten dazu weiter unten.

## Was ist neu in v0.3.0

- **Mehrsprachige Web-UI:** Die Oberfläche gibt es jetzt auf Deutsch, English,
  Français und Español. Die Sprache wird automatisch anhand der
  Browser-Einstellung erkannt und lässt sich jederzeit über den Umschalter in
  der Kopfzeile ändern.
- **GitHub-Repository mit CI und automatischen Releases:** Der Quellcode liegt
  jetzt auf <https://github.com/hilman2/easy-ocpp>. Jede Version wird per CI
  automatisch gebaut und als Release veröffentlicht.

## Was ist neu in v0.2.0

- **Live-Werte während der Ladung:** Cockpit und Wallbox-Detailseite zeigen für
  laufende Ladungen die bisher geladenen **kWh**, die aktuelle **Leistung (kW)**
  und, falls die Wallbox ihn meldet, den **SoC** des Fahrzeugs. Die Anzeige
  aktualisiert sich alle 10 Sekunden automatisch.
- **Automatische Wallbox-Konfiguration:** Beim Verbinden richtet der Server die
  Wallbox so ein, dass sie alle 30 Sekunden Messwerte meldet
  (`MeterValueSampleInterval`, `MeterValuesSampledData`), einstellbar über die
  neue `[ocpp]`-Sektion in der `config.toml`.
- Diverse Robustheits-Fixes bei der Messwert-Verarbeitung (Phasenwerte,
  fehlerhafte Werte, veraltete Anzeigen).

**Update von v0.1.0:** Einfach die `easy-ocpp.exe` über die bestehende Datei
kopieren und den Dienst neu starten. Die Datenbank (`data\easy-ocpp.db`) bleibt
unverändert; es gibt keine neuen Migrationen. Die Live-Werte erscheinen ab der
nächsten Ladung, nachdem sich die Wallbox neu verbunden hat.

**Umbenannt: `easy-occp` heisst jetzt `easy-ocpp`.** Im alten Namen steckte
ein Dreher, das Protokoll heisst OCPP. Beim Update:

- Das Programm heisst jetzt `easy-ocpp.exe`. Die alte `easy-occp.exe` loeschen,
  sonst liegen zwei Programme im Ordner.
- **Die Datenbank bleibt erhalten.** Findet das Programm nur die alte
  `data\easy-occp.db`, arbeitet es unveraendert mit ihr weiter und schreibt beim
  Start einen Hinweis ins Log. Wer umstellen moechte: Programm stoppen, Datei in
  `easy-ocpp.db` umbenennen, neu starten.
- Ein eingerichteter Dienst (nssm, Aufgabenplanung) muss auf den neuen
  Programmnamen zeigen.
- Alle Anmeldungen sind einmalig ungueltig, weil auch der Sitzungs-Cookie neu
  heisst. Einfach neu anmelden.

## Inhalt dieses Ordners

```
easy-ocpp.exe          Hauptprogramm (Windows x64, alles inkl.)
config.example.toml    Vorlage für die Konfiguration
README.md              Projekt-Übersicht
ANLEITUNG.md           Diese Datei
```

Die `.exe` enthält Datenbank-Schema, Web-UI und statische Assets. **Nichts weiter muss mitgeliefert werden.**

## 1. Installation

1. Diesen Ordner an einen festen Ort kopieren, z. B. `C:\easy-ocpp\`.
2. Optional: `config.example.toml` nach `config.toml` kopieren und anpassen (siehe unten). Ohne `config.toml` laufen die Defaults (Port 8080, Datenverzeichnis `./data`).
3. Sicherstellen, dass Port **8080** auf dem Host frei ist und in der Windows-Firewall eingehend freigegeben ist. Sonst erreichen die Wallboxen den Server nicht.

## 2. Erststart

Im Ordner eine **PowerShell** öffnen und starten:

```powershell
.\easy-ocpp.exe
```

Beim ersten Start erledigt die Anwendung alles selbst:

- legt das Verzeichnis `data\` an,
- erzeugt die SQLite-Datei `data\easy-ocpp.db`,
- spielt das komplette Schema ein (Migrationen sind in die Binary eingebettet),
- legt den Admin-Benutzer an.

Danach im Browser öffnen:

<http://localhost:8080>

**Default-Login:** `admin` / `admin`. Passwort bitte sofort unter „Benutzer" ändern.

Beenden mit `Strg+C` im PowerShell-Fenster.

## 3. Konfiguration (`config.toml`)

Minimal reicht Folgendes:

```toml
[http]
bind = "0.0.0.0:8080"
public_base_url = "http://<server-ip-oder-hostname>:8080"

[storage]
data_dir = "data"
db_file  = "easy-ocpp.db"

[ocpp]
# Intervall in Sekunden, in dem Wallboxen während einer Ladung Zählerstand und
# Leistung melden. Wird beim Verbinden automatisch in der Wallbox gesetzt.
# 0 = Auto-Konfiguration deaktivieren.
meter_interval_s = 30
```

- `bind = "0.0.0.0:8080"`: hört auf allen Netzwerkadaptern (nötig, damit Wallboxen verbinden können).
- `public_base_url`: öffentliche Adresse, unter der der Server aus Sicht der Wallboxen erreichbar ist.
- `meter_interval_s`: Meldeintervall der Live-Messwerte (Standard 30 s).

## 4. Wallbox einrichten

Wallbox-Backend-URL in der jeweiligen Geräte-Konfiguration (Herstellerportal oder Web-UI der Box) eintragen:

**OCPP 1.6 / 2.0.1 (WebSocket):**
```
ws://<server-ip>:8080/ocpp/<ChargePointId>
```

`<ChargePointId>` ist die eindeutige ID der Wallbox (frei wählbar, z. B. `WB-Halle-01`). Subprotokoll (`ocpp1.6` / `ocpp2.0.1`) wird automatisch ausgehandelt.

**OCPP 1.5 (SOAP, nur Altgeräte):**
```
http://<server-ip>:8080/ocpp15
```

Sobald die Wallbox verbindet, erscheint sie unter **Wallboxen** mit Status *online*. Kurz nach dem Verbinden konfiguriert der Server die Box automatisch für die Live-Messwerte (nur OCPP 1.6).

## 5. Chips anlernen

1. In der UI → **Chips → Lernfenster öffnen** (2 Minuten aktiv).
2. An der Wallbox mit dem neuen RFID-Chip authentifizieren.
3. Der Chip erscheint in der Liste und kann einem Mitarbeiter zugeordnet oder als Gast-Chip markiert werden (inkl. Ablaufdatum).

## 6. Als Windows-Dienst dauerhaft laufen lassen (optional)

`easy-ocpp.exe` ist ein gewöhnliches Konsolenprogramm. Für Autostart/Service:

**Variante A: Aufgabenplanung**

- Aufgabenplanung → *Aufgabe erstellen* → Trigger *Beim Systemstart* → Aktion *Programm starten* → `C:\easy-ocpp\easy-ocpp.exe` → *Starten in:* `C:\easy-ocpp\`.
- *„Unabhängig von der Benutzeranmeldung ausführen"* aktivieren.

**Variante B: NSSM (Non-Sucking Service Manager)**

```powershell
nssm install easy-ocpp "C:\easy-ocpp\easy-ocpp.exe"
nssm set easy-ocpp AppDirectory "C:\easy-ocpp"
nssm start easy-ocpp
```

## 7. Admin-Passwort zurücksetzen

Falls das Admin-Passwort verloren ist:

```powershell
.\easy-ocpp.exe --reset-admin "neuesPasswort123"
```

Setzt das Passwort des `admin`-Accounts neu und beendet sich.

## 8. Backup

Alles Relevante liegt in **einer einzigen Datei**: `data\easy-ocpp.db`.

Für ein konsistentes Backup den Dienst kurz stoppen und die Datei (inkl. evtl. `-wal` / `-shm`) wegsichern, fertig.

## 9. Fehlersuche

| Problem | Ursache / Lösung |
|---------|------------------|
| Browser zeigt „Seite nicht erreichbar" | Port 8080 belegt oder Firewall blockiert. `netstat -ano \| findstr :8080` prüfen. |
| Wallbox verbindet nicht | `public_base_url` / Firewall / `ChargePointId`-URL prüfen. Logs in der Konsole zeigen eingehende WS-Verbindungen. |
| Keine Live-Werte während der Ladung | Wallbox neu verbinden lassen (Konfiguration wird beim Connect gesetzt). Konsolen-Log prüfen: `MeterValueSampleInterval` sollte als „gesetzt" erscheinen. Manche Boxen brauchen einen Neustart, andere unterstützen keine Leistungsmessung, dann wird die Leistung aus den Zählerständen abgeleitet. |
| „database is locked" | Kein zweites `easy-ocpp.exe` gleichzeitig auf derselben DB starten. |
| Mehr Logs gewünscht | Vor dem Start: `$env:RUST_LOG="debug"` setzen. |

## 10. Deinstallation

Dienst/Aufgabe entfernen, Ordner löschen. Es gibt keine Registry-Einträge und keine externen Abhängigkeiten.

---

Version 0.5.0 · Lizenz: MIT
