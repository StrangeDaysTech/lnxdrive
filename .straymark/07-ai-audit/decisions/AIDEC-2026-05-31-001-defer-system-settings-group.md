---
id: AIDEC-2026-05-31-001
title: Posponer el grupo de ajustes "System" del panel (G1) a v0.2
status: accepted
created: 2026-05-31
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: low
tags: [gnome, preferences, settings, scope, deferral, charter-01, phase-3, v0.2]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - phase-3-gtk4-panel-audit
---

# AIDEC: Posponer el grupo "System" (G1) a v0.2

## Context

La auditoría de Fase 3 (`.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md`)
registró el hallazgo **G1**: el Charter-01 nombra cuatro grupos de ajustes
(Account, Folders, Network, System), pero el panel implementa Account, Sync
(≈Folders), Advanced (≈Network) y Conflicts — **no existe un grupo "System"**, y
el daemon **no expone API D-Bus** para sus ajustes candidatos: arranque
automático, gestión de caché y política de deshidratación.

De esos tres, solo el **arranque automático** es implementable sin API D-Bus
nueva (gestionando una unit de usuario de systemd o un `.desktop` de autostart
desde el panel). **Caché** y **deshidratación** requieren extender la interfaz
`Settings` del daemon con métodos nuevos y la lógica para aplicarlos — trabajo
**cruzado** (daemon + panel) y de diseño no trivial.

## Problem

¿Dónde y cuándo abordamos G1, dado que el Charter-01 es estrictamente "Road to
v0.1.0-alpha.1" y G1 mezcla un control trivial (auto-start) con ajustes que no
tienen backend y exceden el MVP del alpha?

## Alternatives Considered

### Alternativa 1 — Implementar G1 completo ahora, dentro de Charter-01

Crear la página "System" con auto-start + caché + deshidratación, añadiendo la
API D-Bus necesaria en el daemon.

**Pros:** cierra el "cuatro grupos" literal del Charter.
**Cons:** caché/deshidratación **no son MVP alpha**; obliga a diseñar y exponer
API D-Bus nueva (superficie + pruebas) bajo presión del release alpha; infla un
Charter cuyo objetivo declarado es el alpha mínimo. Contradice
[[feedback_minimum_viable_plus_tde]].

### Alternativa 2 — Página "System" solo con auto-start ahora, resto diferido

Enviar una página con el único control implementable y dejar caché/deshidratación
para después.

**Pros:** algo de "System" visible en el alpha sin API nueva.
**Cons:** un grupo "System" a medias (un solo toggle) confunde más que ayuda;
mezcla alcance v0.1 y v0.2 en un mismo grupo; habría que rediseñarlo al añadir el
resto. Bajo valor para el usuario alpha.

### Alternativa 3 — Fase nueva dentro de Charter-01 para G1

Añadir una "Fase 7: System settings" al roadmap de Charter-01.

**Pros:** mantiene G1 rastreado en el Charter activo.
**Cons:** **incoherente con el alcance del Charter** — Charter-01 es "Road to
v0.1.0-alpha.1"; una fase de ajustes que requiere API nueva y no es MVP no
pertenece a un Charter de alpha. Diluiría el criterio de "hecho" del alpha.

### Alternativa 4 — Diferir G1 a un Charter v0.2 futuro (ELEGIDA)

Documentar G1 como diferido; abordarlo en un Charter v0.2 (cuando v0.2 arranque),
junto con el resto de ajustes avanzados y su API D-Bus.

**Pros:** respeta el alcance del alpha; agrupa el grupo "System" completo de forma
coherente (auto-start + caché + deshidratación + su API) en el ciclo donde
pertenece; no introduce API D-Bus a medias en el alpha.
**Cons:** el panel del alpha mostrará tres grupos en vez de cuatro — aceptable y
documentado.

## Decision

**Alternativa 4.** G1 (grupo "System") se **pospone a v0.2** y se abordará en un
**Charter v0.2 futuro**, no como fase de Charter-01 ni como implementación parcial
en el alpha. No se crea el Charter v0.2 ahora (sería prematuro y de un solo ítem);
esta AIDEC es la semilla de seguimiento y se promoverá al backlog de v0.2 cuando
ese ciclo comience.

El Charter-01 se actualiza para reflejar que la Fase 3 entrega **tres** grupos de
ajustes wired al daemon (Account, Folders/Sync, Network/Advanced) más Conflicts,
y que el grupo "System" queda **fuera de alcance del alpha** por esta decisión.

## Consequences

- El panel del alpha no tendrá grupo "System"; el arranque automático se gestiona
  por el packaging/systemd del alpha, no por la UI todavía.
- Cuando arranque v0.2, su Charter incluirá: API D-Bus de caché y deshidratación
  en `Settings`, y la página "System" del panel (auto-start + caché +
  deshidratación) que las consume.
- Fase 3 puede cerrarse con los hallazgos **H** (H1–H5) resueltos sin bloquear por
  G1.
