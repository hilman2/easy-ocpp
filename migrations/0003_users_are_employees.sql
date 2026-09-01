-- Mitarbeiter und Login werden wieder zu EINEM Begriff: dem Benutzer.
--
-- Bis 0002 gab es zwei Tabellen (`users` = Login, `employees` = Person). Die
-- Verknuepfung wurde nur fuer Altdaten einmalig gesetzt und danach nie wieder
-- gepflegt, weshalb ein Nicht-Admin-Login praktisch nie eigene Ladungen sah.
-- Ab hier gilt: ein Mitarbeiter IST ein users-Eintrag mit role='user'.
-- Die Semantik "gehoert zu Person" laeuft wieder ueber user_id (chips,
-- transactions). Die Spalten existieren seit 0001 und werden hier neu befuellt.

-- 1) users bekommt die Mitarbeiter-Felder und die Ladelimit-Vorgaben.
ALTER TABLE users ADD COLUMN department TEXT;
-- Standard-Ladelimits des Mitarbeiters. NULL = kein Limit. Werden beim Start
-- einer Ladung in die Transaktion kopiert; der Mitarbeiter kann sie dort noch
-- fuer die laufende Ladung aendern.
ALTER TABLE users ADD COLUMN default_limit_wh INTEGER;
ALTER TABLE users ADD COLUMN default_limit_minutes INTEGER;

-- 2) Zielnamensraum 'emp<id>' fuer generierte Logins freiraeumen. Kollisionen
--    sind bei realen Benutzernamen nicht zu erwarten; der Guard verhindert nur,
--    dass die Migration am UNIQUE-Index scheitert.
UPDATE users SET username = username || '.old'
 WHERE username IN (SELECT 'emp' || id FROM employees);

-- 3) Jeder Mitarbeiter ohne Login bekommt ein Konto ohne Passwort, damit sich
--    niemand ungewollt anmelden kann. Der Admin vergibt es unter "Benutzer".
INSERT INTO users (username, display_name, email, department, role, auth_source,
                   password_hash, disabled, employee_id)
SELECT 'emp' || e.id, e.display_name, e.email, e.department, 'user', 'local',
       NULL, CASE WHEN e.active = 0 THEN 1 ELSE 0 END, e.id
  FROM employees e
 WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.employee_id = e.id);

-- 4) Bereits verknuepfte Logins erben Abteilung und (falls leer) E-Mail.
UPDATE users SET department = (
        SELECT e.department FROM employees e WHERE e.id = users.employee_id)
 WHERE employee_id IS NOT NULL AND department IS NULL;
UPDATE users SET email = (
        SELECT e.email FROM employees e WHERE e.id = users.employee_id)
 WHERE employee_id IS NOT NULL AND email IS NULL;

-- 5) Zuordnungen von employee_id zurueck auf user_id ueberfuehren.
UPDATE chips SET user_id = (
        SELECT MIN(u.id) FROM users u WHERE u.employee_id = chips.employee_id)
 WHERE employee_id IS NOT NULL;
UPDATE transactions SET user_id = (
        SELECT MIN(u.id) FROM users u WHERE u.employee_id = transactions.employee_id)
 WHERE employee_id IS NOT NULL;

-- 6) employees und alle employee_id-Spalten entfernen. Reihenfolge: erst die
--    Indizes, dann die Spalten (loest die Fremdschluessel mit), dann die Tabelle.
DROP INDEX IF EXISTS idx_chips_employee;
DROP INDEX IF EXISTS idx_tx_employee;
ALTER TABLE chips DROP COLUMN employee_id;
ALTER TABLE transactions DROP COLUMN employee_id;
ALTER TABLE users DROP COLUMN employee_id;
DROP TABLE employees;

-- 7) Ladelimits pro laufender Ladung.
--    limit_wh      – Abschaltung, sobald so viel Energie geladen wurde (NULL = aus)
--    limit_until   – Abschaltung zu diesem Zeitpunkt, RFC3339 (NULL = aus)
--    limit_stopped – 0 = nicht ausgeloest, 1 = wegen Energielimit gestoppt,
--                    2 = wegen Zeitlimit gestoppt. Verhindert wiederholte
--                    RemoteStop-Aufrufe, solange die Box noch laeuft.
ALTER TABLE transactions ADD COLUMN limit_wh INTEGER;
ALTER TABLE transactions ADD COLUMN limit_until TEXT;
ALTER TABLE transactions ADD COLUMN limit_stopped INTEGER NOT NULL DEFAULT 0;

-- 8) Chip-Kategorie ist ab jetzt abgeleitet, nicht mehr frei waehlbar:
--    Chip einem Benutzer zugeordnet = Mitarbeiter-Chip, sonst Gast-Chip.
--    Vorher liessen sich beide Angaben widersprechen (Gast-Chip, der auf einen
--    Mitarbeiter zeigt, und umgekehrt).
--
--    Solange es den Gast-Begriff nicht am Chip gab, wurde er als Pseudo-Person
--    nachgebaut ("Gast"/"Guest"/"Besucher" als Mitarbeiter ohne Login). Deren
--    Chips werden zu echten Gast-Chips. Die Person selbst bleibt erhalten und
--    wird nur deaktiviert, damit die Zuordnung alter Ladungen nicht verloren geht.
UPDATE chips SET user_id = NULL
 WHERE user_id IN (
        SELECT id FROM users
         WHERE password_hash IS NULL
           AND role = 'user'
           AND lower(display_name) IN ('gast', 'gaeste', 'gäste', 'guest', 'guests', 'besucher'));

UPDATE users SET disabled = 1
 WHERE password_hash IS NULL
   AND role = 'user'
   AND lower(display_name) IN ('gast', 'gaeste', 'gäste', 'guest', 'guests', 'besucher');

UPDATE chips SET kind = CASE WHEN user_id IS NULL THEN 'guest' ELSE 'employee' END;
