# Working rules for this repository

## Changelog

`CHANGELOG.md` is **written in English for the people who use easy-ocpp, not
for developers**. An entry describes what changes for someone running the
program: what they can now do, what looks different, what they need to watch
out for when updating.

Keep file names, function names, module structure and invisible refactorings
out of it, along with words like "refactored", "bumped" or "implemented".
Anyone who needs the technical view reads the commits.

    Good: Charging sessions now stop on their own once they reach a
          target amount of energy or a time you set.
    Bad:  Added limit_wh and limit_until columns plus a watchdog task.

Every released version gets a section with its number and date. Lead with what
affects the most people. Notes about updating go at the end of the section,
because they only matter once someone has decided to install it.

## Language and style

- **No em dashes (—)** in code, comments, documentation, commit messages or on
  the website. Rewrite the sentence instead of swapping the character for
  another one.
- Commit messages are written in German, as they have been so far. They explain
  the why, not just the what.
- The state of a development machine does not belong in the project history.
  Discuss tooling problems in the chat or in the pull request.

## Migrations

Files in `migrations/` are **immutable once they have been released**. sqlx
embeds their content at compile time, builds a SHA-384 checksum over it and
compares that against `_sqlx_migrations` at startup. Any change to the content,
including a comment, makes existing installations stop with "migration N was
previously applied but has been modified".

Changes always go into a new migration. `db::repair_line_ending_checksums`
only covers differing line endings and nothing else.

`.gitattributes` enforces `*.sql text eol=lf`. It has to stay that way, or
Windows and Linux runners produce different checksums for the same file.

## Website

`docs/` is **generated**. Never edit it by hand, run `python
tools/build_pages.py` instead. The generator writes twelve HTML files, the
stylesheet, the sitemap and robots.txt.

The site loads **nothing from third-party servers**: no fonts, no scripts, no
images, no tracking. That is deliberate. It keeps the site fast and the privacy
policy short. Anyone adding an external resource has to update the privacy
policy and may need a consent banner.

## Four languages

German, English, French and Spanish. Changing a text means changing all four:
`README*.md`, `INSTALL*.md` together with `ANLEITUNG.md`, the table in
`src/i18n.rs`, and the content in `tools/build_pages.py`.

## Raising the version

Always change `Cargo.toml` and `Cargo.lock` together, otherwise CI fails on
`cargo build --locked`. Then update the title and footer lines of the four
installation guides and add a section to `CHANGELOG.md`.

Pushing a tag `vX.Y.Z` triggers the release workflow, which builds and
publishes the binaries for Windows and Linux.
