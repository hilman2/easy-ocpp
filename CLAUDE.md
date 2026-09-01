# Arbeitsregeln für dieses Repository

## Changelog

`CHANGELOG.md` ist **auf Englisch** und richtet sich an **Anwender, nicht an
Entwickler**. Ein Eintrag beschreibt, was sich für jemanden ändert, der das
Programm benutzt: was er jetzt tun kann, was anders aussieht, was er beim
Update beachten muss.

Nicht hinein gehören Dateinamen, Funktionsnamen, Modulstruktur, Refactorings
ohne sichtbare Wirkung oder Formulierungen wie „refactored", „bumped",
„implemented". Wer die technische Sicht braucht, liest die Commits.

    Gut:     Charging sessions now stop on their own once they reach a
             target amount of energy or a time you set.
    Schlecht: Added limit_wh and limit_until columns plus a watchdog task.

Jede veröffentlichte Version bekommt einen Abschnitt mit Versionsnummer und
Datum. Zuerst das, was am meisten Leute betrifft. Hinweise zum Update kommen
ans Ende des Abschnitts.

## Sprache und Stil

- **Keine Gedankenstriche (—)** in Code, Kommentaren, Doku, Commit-Nachrichten
  oder auf der Webseite. Satz umbauen statt Zeichen tauschen.
- Commit-Nachrichten auf Deutsch, wie im bisherigen Verlauf. Sie erklären das
  Warum, nicht nur das Was.
- Der Zustand der Entwicklungsmaschine gehört nicht in die Projekthistorie.
  Werkzeugprobleme bespricht man im Chat oder im Pull Request.

## Migrationen

`migrations/*.sql` sind **unveränderlich, sobald sie veröffentlicht wurden**.
sqlx bildet beim Kompilieren eine SHA-384-Summe über den Dateiinhalt und
vergleicht sie beim Start mit `_sqlx_migrations`. Jede inhaltliche Änderung,
auch an einem Kommentar, bricht bestehende Installationen mit
„migration N was previously applied but has been modified" ab.

Änderungen kommen immer als neue Migration. `db::repair_line_ending_checksums`
fängt ausschließlich unterschiedliche Zeilenenden ab, nichts weiter.

`.gitattributes` erzwingt `*.sql text eol=lf`. Das muss so bleiben, sonst
erzeugen Windows- und Linux-Runner verschiedene Prüfsummen.

## Webseite

`docs/` ist **erzeugt**. Nie von Hand bearbeiten, sondern
`python tools/build_pages.py` aufrufen. Der Generator baut zwölf HTML-Dateien,
das Stylesheet, die Sitemap und robots.txt.

Die Seite lädt **nichts von fremden Servern**: keine Schriften, keine Skripte,
keine Bilder, kein Tracking. Das ist Absicht, hält die Seite schnell und die
Datenschutzerklärung kurz. Wer eine externe Ressource einbaut, muss die
Datenschutzerklärung anpassen und braucht je nach Fall ein Einwilligungsbanner.

## Mehrsprachigkeit

Deutsch, Englisch, Französisch, Spanisch. Wer einen Text ändert, ändert alle
vier Fassungen: `README*.md`, `INSTALL*.md` samt `ANLEITUNG.md`, die Tabelle in
`src/i18n.rs` und die Inhalte in `tools/build_pages.py`.

## Version anheben

`Cargo.toml` und `Cargo.lock` immer zusammen, sonst scheitert die CI an
`cargo build --locked`. Dazu die Kopf- und Fußzeilen der vier
Installationsanleitungen und ein neuer Abschnitt in `CHANGELOG.md`.

Ein Tag `vX.Y.Z` löst den Release-Workflow aus, der die Binaries für Windows
und Linux baut und veröffentlicht.
