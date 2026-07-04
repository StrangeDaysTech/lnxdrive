---
id: AILOG-2026-07-03-005
title: "Chore: lnxdrive-gnome usa path local para la dep lnxdrive-ipc (FU-009 / RD-2)"
status: accepted
created: 2026-07-03
agent: claude-opus-4-8-v1.0
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [8]
lines_changed: 45
files_modified:
  - lnxdrive-gnome/Cargo.toml
  - lnxdrive-gnome/Cargo.lock
observability_scope: none
tags: [gnome, cargo, dependency, chore, monorepo, follow-up]
related:
  - AILOG-2026-07-03-001-fases-4-5-external-audit-remediation
---

# AILOG: Chore — path local para `lnxdrive-ipc` en `lnxdrive-gnome`

## Summary

Cierre de **FU-009 / RD-2** (backlog de follow-ups). El crate `lnxdrive-gnome`
declaraba su dependencia `lnxdrive-ipc` vía **git remoto**
(`git = "https://github.com/strangedaystech/lnxdrive.git"`) en lugar de un path
local, pese a que el crate vive en el mismo monorepo
(`lnxdrive-engine/crates/lnxdrive-ipc/`). Se cambió a
`path = "../lnxdrive-engine/crates/lnxdrive-ipc"`.

El `git` dep no tenía `rev`/`branch`, así que resolvía a la rama por defecto del
remoto y quedó **pineado en el `Cargo.lock` a un commit viejo** (`d3a6b0c2`:
`lnxdrive-core 0.1.0`, `serde_yaml`), desalineado del estado local del monorepo
(`0.1.0-alpha.1`, `serde_norway`). El path local hace que la dep siga el working
tree, que es el comportamiento correcto para una dependencia in-repo.

## Context

Follow-up heredado del round de auditoría externa Fases 4–5
(AILOG-2026-07-03-001 §Follow-ups, RD-2). Clasificado ahí como bajo impacto
—el crate `lnxdrive-ipc` es un stub que no se construye dentro del bundle Flatpak—
con trigger `ready` y destino `chore`.

## Actions Performed

1. **`lnxdrive-gnome/Cargo.toml`**: `lnxdrive-ipc = { git = … }` →
   `lnxdrive-ipc = { path = "../lnxdrive-engine/crates/lnxdrive-ipc" }`.
2. **`lnxdrive-gnome/Cargo.lock`**: regenerado por Cargo al resolver el grafo;
   `lnxdrive-ipc` y `lnxdrive-core` pasan de `source = git+…` a las versiones
   locales (`0.1.0-alpha.1`), reflejando la cadena real del monorepo
   (`serde_yaml` → `serde_norway`, `async-trait` añadido a `lnxdrive-ipc`).

## Risk

- **Sin riesgo de runtime**: el crate no entra al bundle; el cambio solo afecta
  cómo Cargo resuelve la fuente de la dep en builds locales/CI del componente.
- **Herencia de workspace correcta**: `lnxdrive-ipc` usa `version.workspace`,
  `lnxdrive-core.workspace`, etc.; Cargo resuelve esa herencia siguiendo el
  `[workspace]` ancestro del path del crate, así que el path dep cruzando
  componentes funciona sin tocar el workspace destino.

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `lnxdrive-gnome/Cargo.toml` | +2/-2 | git dep → path dep local |
| `lnxdrive-gnome/Cargo.lock` | +20/-21 | Resolución regenerada a fuentes locales |

## Decisions Made

- **Path relativo vs git pineado**: un `git` dep sin `rev` es no-determinista
  (sigue la rama por defecto del remoto) y ya había divergido del monorepo; el
  path local es la forma canónica de una dep in-repo y elimina el lockfile stale.

## Impact

- **Functionality**: N/A (mismo API; solo cambia el origen de la dep).
- **Performance**: N/A.
- **Security**: leve mejora — deja de traer código desde un remoto/commit no
  fijado; usa el fuente auditado del propio repo.
- **Privacy**: N/A.
- **Environmental**: N/A.

## Verification

- [x] `cargo metadata` resuelve `lnxdrive-ipc` a `source = None` con `manifest_path`
  apuntando al crate local del monorepo.
- [x] El grafo de dependencias completo resuelve limpio (`cargo metadata`,
  `exit=0`).
- [x] `Cargo.lock` regenerado sin entradas `git+…` para `lnxdrive-ipc` /
  `lnxdrive-core`.

## Follow-ups

Ninguno. Este AILOG cierra un follow-up existente (FU-009) en vez de abrir uno
nuevo; el estado vive en el backlog de follow-ups.

## Additional Notes

- Cierra FU-009 (RD-2) del registro `.straymark/follow-ups-backlog.md`.

---

<!-- Template: StrayMark | https://strangedays.tech -->
