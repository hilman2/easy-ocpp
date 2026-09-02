-- Verlauf der StatusNotification je Anschluss.
--
-- Bisher ueberschrieb jede Meldung die vorherige, gespeichert war nur der
-- aktuelle Zustand. Damit liess sich nicht beantworten, warum eine Ladung
-- abgebrochen ist oder wann eine Wallbox gestoert war.
--
-- Eingetragen wird nur, was sich gegenueber der letzten Meldung desselben
-- Anschlusses geaendert hat. Manche Boxen wiederholen denselben Status im
-- Heartbeat-Takt; ohne diese Bedingung liefe die Tabelle voll mit Kopien.
CREATE TABLE connector_events (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    wallbox_id    INTEGER NOT NULL REFERENCES wallboxes(id) ON DELETE CASCADE,
    connector_id  INTEGER NOT NULL,
    -- Available, Preparing, Charging, SuspendedEV, SuspendedEVSE, Finishing,
    -- Reserved, Unavailable, Faulted
    status        TEXT    NOT NULL,
    -- NoError, GroundFailure, HighTemperature, OverCurrentFailure, ...
    error_code    TEXT,
    -- Freitext der Wallbox, dazu der herstellereigene Fehlercode. Letzterer
    -- steht in keiner Norm, ist aber oft die brauchbarste Diagnose.
    info          TEXT,
    vendor_error  TEXT,
    -- Zeitpunkt aus der Meldung, nicht der Empfang.
    timestamp     TEXT    NOT NULL,
    created_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Fuer die Liste je Wallbox.
CREATE INDEX idx_cev_wallbox ON connector_events(wallbox_id, timestamp DESC);
-- Fuer die Meldungen im Zeitraum einer Ladung.
CREATE INDEX idx_cev_session ON connector_events(wallbox_id, connector_id, timestamp);
