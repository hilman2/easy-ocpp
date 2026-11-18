-- Berechtigungen sind vom Benutzerkonto und seiner Rolle unabhaengig.
ALTER TABLE chips ADD COLUMN charging_priority INTEGER NOT NULL DEFAULT 0
    CHECK (charging_priority IN (0, 1));
ALTER TABLE chips ADD COLUMN allow_grid INTEGER NOT NULL DEFAULT 0
    CHECK (allow_grid IN (0, 1));
UPDATE chips SET charging_priority = 1 WHERE kind = 'guest';

-- Werte gelten je Anschluss dieser Wallbox. Vor Aktivierung pruefen.
ALTER TABLE wallboxes ADD COLUMN charging_min_w INTEGER NOT NULL DEFAULT 4140;
ALTER TABLE wallboxes ADD COLUMN charging_max_w INTEGER NOT NULL DEFAULT 11040;
ALTER TABLE wallboxes ADD COLUMN charging_phases INTEGER NOT NULL DEFAULT 3;
ALTER TABLE wallboxes ADD COLUMN charging_rate_unit TEXT NOT NULL DEFAULT 'W'
    CHECK (charging_rate_unit IN ('W', 'A'));
