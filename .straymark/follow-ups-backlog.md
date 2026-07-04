---
last_scan: 2026-07-03
schema_version: v1
total_open: 6
total_promoted: 0
total_closed_in_session: 9
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
  - AILOG-2026-07-03-001
  - AILOG-2026-07-03-002
  - AILOG-2026-07-03-003
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
- **Status**: closed
- **Trigger**: resolved
- **Destination**: lnxdrive-engine/config/ (PR #52)
- **Cost**: XS
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04. Closed 2026-07-02: las tres unidades (`.service` D-Bus, `lnxdrive.service` systemd, `.desktop` autostart) apuntaban a `/usr/bin/lnxdrive-daemon`; el binario real es `lnxdrived` (`[[bin]] name` en lnxdrive-daemon/Cargo.toml; el manifiesto Flatpak instala `target/release/lnxdrived`). Corregido en PR #52.

### FU-007 — **Wiring de comandos files-on-demand (`pin`/`unpin`/`hydrate`/`dehydrate`) al
- **Origin**: AILOG-2026-07-03-001 §Follow-ups
- **Source-hash**: cd5882682e75
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-07-03.

### FU-008 — **M2 — alinear el app-id del Flatpak (`com.strangedaystech.LNXDrive`) con el
- **Origin**: AILOG-2026-07-03-001 §Follow-ups
- **Source-hash**: 16387869678e
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-07-03.

### FU-009 — **RD-2 — `lnxdrive-gnome` depende de `lnxdrive-ipc` vía git remoto**
- **Origin**: AILOG-2026-07-03-001 §Follow-ups
- **Source-hash**: 9efc95818794
- **Status**: closed
- **Trigger**: ready
- **Destination**: lnxdrive-gnome/Cargo.toml (rama `chore/gnome-ipc-path-dep`)
- **Cost**: XS
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-07-03. Closed 2026-07-03 (`e8c4a00`): la dep pasó de `git = ".../lnxdrive.git"` (pineada a un commit viejo — `lnxdrive-core 0.1.0` / `serde_yaml`, desalineado del monorepo) a `path = "../lnxdrive-engine/crates/lnxdrive-ipc"`. Verificado con `cargo metadata` (resuelve a source=None / manifest local, grafo limpio).

### FU-010 — **RD-3 — actualizar la fecha y el link `[0.1.0-alpha.1]` del `CHANGELOG.md`**
- **Origin**: AILOG-2026-07-03-001 §Follow-ups
- **Source-hash**: c5d32fcd008f
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-07-03.

### FU-011 — **RD-4 — actualizar `<release … date="2026-06-04">` del metainfo**
- **Origin**: AILOG-2026-07-03-001 §Follow-ups
- **Source-hash**: 491716884814
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-07-03.

### FU-012 — R1 (new, not in Charter) — copy overstatement en assets de release.
- **Origin**: AILOG-2026-07-03-001 §R1 (new, not in Charter)
- **Source-hash**: f7fb2759b72e
- **Status**: closed
- **Trigger**: resolved
- **Destination**: AILOG-2026-07-03-001 §R1
- **Cost**: —
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-07-03. Closed 2026-07-03: ya mitigado en su AILOG de origen — los tres defectos de veracidad se corrigieron a copy fiel antes del tag; el patrón "declared but not wired" queda trackeado upstream (straymark#209). No había trabajo pendiente; entrada de registro cerrada en triage.

### FU-013 — **La extensión de GNOME Shell no está en el build de Meson ni se instala vía
- **Origin**: AILOG-2026-07-03-002 §Follow-ups
- **Source-hash**: 4ee94a61be90
- **Status**: closed
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Premisa original inexacta — el target de Meson ya existía y funciona (verificado `meson setup -Denable_shell=true` + `meson install`). La deuda real (instalación host-side no documentada) se resolvió con la sección host-side en `lnxdrive-packaging/README.md`. Ver AILOG-2026-07-03-003 / PR de la rama `fix/shell-extension-hostside-and-metadata`.

### FU-014 — **`metadata.json` declara `settings-schema: com.strangedaystech.LNXDrive.Indicator`
- **Origin**: AILOG-2026-07-03-002 §Follow-ups
- **Source-hash**: a23bd62d0851
- **Status**: closed
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: `settings-schema` muerto eliminado del `metadata.json` (`prefs.js` no usa `getSettings()`; no se necesita gschema). Ver AILOG-2026-07-03-003.

### FU-015 — **Limitación del validador de StrayMark con AILOGs en subdirectorios por
- **Origin**: AILOG-2026-07-03-003 §Follow-ups
- **Source-hash**: ea6eb87d48af
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-07-03.
## Bucket: time-triggered

## Bucket: charter-triggered

## Bucket: phase-blocked

## Bucket: operational
