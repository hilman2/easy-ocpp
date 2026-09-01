-- Erzwungener Passwortwechsel und Versandprotokoll der Monatsberichte.

-- 1 = der Benutzer muss beim naechsten Aufruf ein neues Passwort setzen.
-- Wird gesetzt, wenn ein Administrator ein Passwort vergibt und den Haken
-- dafuer anklickt, und beim erfolgreichen Wechsel wieder geloescht.
ALTER TABLE users ADD COLUMN must_change_password INTEGER NOT NULL DEFAULT 0;

-- Haelt fest, welcher Monatsbericht an wen bereits herausgegangen ist. Ohne
-- das wuerde ein Neustart oder ein zweiter Versuch nach einem Teilfehler
-- dieselbe Mail erneut verschicken.
CREATE TABLE report_mails (
    user_id  INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Berichtszeitraum als 'YYYY-MM'.
    period   TEXT    NOT NULL,
    sent_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, period)
);
