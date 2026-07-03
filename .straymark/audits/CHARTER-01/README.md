# Audits — CHARTER-01 (layout convention)

This Charter is **multi-phase**, so it has **more than one external-audit round**.
StrayMark's audit tooling (`straymark charter audit`, `/straymark-audit-execute`,
`/straymark-audit-review`) currently assumes **one audit round per Charter**: it
reads/writes the round triad at **fixed, flat paths** and globs `report-*.md`
**non-recursively** at this directory's top level. There is no native `--round`
namespacing yet (upstream tracking: see `upstream-feedback-drafts.md`, related to
[straymark#208](https://github.com/StrangeDaysTech/straymark/issues/208)).

To keep rounds from colliding — and to stop the flat `report-*.md` glob from
merging a **previous** round's reports into the **current** review/telemetry — we
use this local convention:

## Convention

- **The current (in-progress) round lives flat**, at this directory's top level
  (`audit-prompt.md`, `report-*.md`, `review.md`, `external-audit-pending.yaml`),
  because the CLI and the `audit-execute`/`audit-review` skills read/write those
  fixed paths. The non-recursive glob then sees **only** the current round.
- **When a round closes**, archive its triad into a **per-round subfolder** with
  canonical (unsuffixed) names:

  ```
  fase-1/   { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
  fase-3/   { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
  ```

- **Charter/phase-level analysis docs stay flat** — they are not round-triad
  files and are referenced by immutable AILOG/AIDEC records:
  - `phase-3-gtk4-panel-audit.md` — the internal Fase-3 panel audit (source doc
    for AILOG-2026-05-31-002 / AIDEC-2026-05-31-001).
  - `upstream-feedback-drafts.md` — charter-wide ledger of adopter feedback owed
    upstream; filed at Charter close.

## Rounds so far

| Round     | Scope                                   | Location            |
|-----------|-----------------------------------------|---------------------|
| Fase 1    | Security (RISK-002 tokens, CI hardening)| `fase-1/`           |
| Fase 3    | GTK4 preferences panel                  | `fase-3/`           |
| Fases 4–5 | Flatpak packaging + release infra       | *(flat — current)*  |

> This is an adopter stopgap. If StrayMark ships native per-round audit
> namespacing (`--round <label>` + per-round subfolders + round-scoped
> `--merge-reports`), migrate to it and delete this convention.
