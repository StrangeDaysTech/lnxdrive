---
last_scan: 2026-06-04
schema_version: v1
total_open: 2
total_promoted: 0
total_closed_in_session: 4
total_phase_blocked: 0
total_suspected_closed: 0
buckets:
  - ready
  - time-triggered
  - charter-triggered
  - phase-blocked
  - operational
fully_extracted_ailogs:
  - AILOG-2026-05-28-001
  - AILOG-2026-05-28-002
  - AILOG-2026-06-04-001
  - AILOG-2026-06-04-002
---

# Follow-ups Backlog

> Central registry of `§Follow-ups` and `R<N> (new, not in Charter)` entries across AILOGs.
> Maintained by `straymark followups drift --apply`; counters are CLI-owned.
> Convention: `.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` ·
> Schema: `.straymark/schemas/follow-ups-backlog.schema.v1.json`

<!--
Entry shape (v1 — optional fields marked):

### FU-NNN — <short description>
- **Origin**: AILOG-NNNN-NN-NN-NNN <pointer to source section>
- **Origin-class**: ex-ante-planning | testing | telemetry | staging | real-env-bug   (optional)
- **Status**: open | in-progress | suspected-closed | closed | superseded | promoted
- **Severity**: normal | blocking                                                     (optional)
- **Trigger**: ready | <calendar date> | when <X> | <other>
- **Destination**: chore | mini-charter | charter-replanning | operations | <charter-id> | <TDE id>
- **Cost**: <effort estimate>
- **Labels**: <free tags, comma-separated>                                            (optional)
- **Notes**: <free-form context>
-->

## Bucket: ready

### FU-001 — R6 — Charter scope drift: RISK-003 real change surface was not `write_serializer.rs`
- **Origin**: AILOG-2026-05-28-001 §R6 (new, not in Charter)
- **Origin-class**: ex-ante-planning
- **Status**: closed
- **Trigger**: ready
- **Destination**: CHARTER-01
- **Cost**: 0 (resolved at source)
- **Notes**: Charter drift already remediated atomically in the source PR — the `## Files to modify` row for RISK-003 was corrected from the fully-implemented `write_serializer.rs` stub to the real surface (`inode_entry.rs`, `filesystem.rs`, `hydration.rs`, `tests/integration_write_during_hydration.rs`). Closed at registry adoption (2026-06-04); no pending work.

### FU-002 — R7 — Charter scope drift: RISK-001 touched `main.rs` and cross-crate `lnxdrive-ipc`
- **Origin**: AILOG-2026-05-28-002 §R7 (new, not in Charter)
- **Origin-class**: ex-ante-planning
- **Status**: closed
- **Trigger**: ready
- **Destination**: CHARTER-01
- **Cost**: 0 (resolved at source)
- **Notes**: Charter drift already remediated atomically in the source PR — the `## Files to modify` row for RISK-001 (only `health.rs`) was extended to `lnxdrive-daemon/src/main.rs` and cross-crate `lnxdrive-ipc/src/service.rs` (`dbus_health` state field + property). Closed at registry adoption (2026-06-04); no pending work.

### FU-003 — **Vendoring de crates para Flathub**: el manifiesto usa `build-args:
- **Origin**: AILOG-2026-06-04-001 §Follow-ups
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04.

### FU-004 — **`lnxdrive-packaging/README.md` desactualizado**: promete subdirectorios
- **Origin**: AILOG-2026-06-04-001 §Follow-ups
- **Status**: closed
- **Trigger**: resolved
- **Destination**: CHARTER-01
- **Cost**: S
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04. Resuelto en Charter-01 Fase 5 (AILOG-2026-06-04-002): README de packaging realineado con la realidad del alpha (Flatpak only, formatos diferidos a v0.2.0-beta).

### FU-005 — **Nombres canónicos de screenshots para Fase 5**: el metainfo referencia
- **Origin**: AILOG-2026-06-04-001 §Follow-ups
- **Status**: closed
- **Trigger**: Fase 5 release assets
- **Destination**: docs/screenshots/ (PR #49)
- **Cost**: S
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04. Closed 2026-06-17: los 6 PNG con nombres canónicos (preferences-window, onboarding-wizard, conflict-dialog, shell-indicator, status-menu, nautilus-overlays) capturados en VM Nivel-5 (GNOME Wayland, mock daemon) y añadidos a `docs/screenshots/`; coinciden con README raíz y metainfo AppStream.

### FU-006 — **`lnxdrive-engine/config/lnxdrive-autostart.desktop` apunta a
- **Origin**: AILOG-2026-06-04-002 §Follow-ups
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04.
## Bucket: time-triggered

## Bucket: charter-triggered

## Bucket: phase-blocked

## Bucket: operational
