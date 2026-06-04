---
id: AIDEC-2026-06-04-001
title: Arquitectura del manifiesto Flatpak para v0.1.0-alpha (runtime 49, sources dir, bus scoped)
status: accepted
created: 2026-06-04
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [packaging, flatpak, runtime, sandbox, dbus, charter-01, phase-4]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-06-04-001
---

# AIDEC: Arquitectura del manifiesto Flatpak para v0.1.0-alpha

## Context

El Charter-01 (Fase 4, scope item 5) pide completar
`lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` con install
stages, permisos correctos y target `org.gnome.Platform 47`. El manifiesto
heredado tenía cuatro defectos estructurales:

1. Apuntaba a **dos repos git separados** (`lnxdrive.git`, `lnxdrive-gnome.git`
   con tag `v0.1.0`) que no existen — el proyecto es un **monorepo** sin tags.
2. `command: lnxdrive-gnome` ejecuta un **stub** (`src/main.rs` imprime
   "Not yet implemented"); la GUI real es `lnxdrive-preferences`.
3. Runtime `org.gnome.Platform 45` (EOL); el Charter declara 47, que también
   alcanzó EOL en 2025 — antes incluso de la firma del Charter (2026-05-29).
4. Sin install stages para iconos, `.desktop`, metainfo ni schema GSettings, y
   con `--socket=session-bus` (bus de sesión sin restricción).

## Problem

Definir runtime objetivo, mecanismo de sources, comando principal, alcance de
módulos y política de sandbox para el bundle del alpha, respetando el scope del
Charter y la postura de seguridad de RISK-002 (superficie D-Bus mínima).

## Alternatives Considered

### Runtime objetivo
- **A. `org.gnome.Platform 47`** (literal del Charter): EOL desde 2025 — sin
  parches de seguridad; además libadwaita 1.6 justo en el límite del feature
  gate `v1_6` del panel. Descartada: publicar un alpha sobre runtime EOL
  contradice el espíritu de la Fase 1 (cierre de riesgos).
- **B. `org.gnome.Platform 50`** (estable actual, mar 2026): válida, pero
  reduce la ventana de compatibilidad para early adopters en distros LTS.
- **C. `org.gnome.Platform 49`** ✅: el runtime soportado más antiguo en
  jun 2026; satisface gtk4 `v4_14` + libadwaita `v1_6`; ya instalado en la
  máquina de verificación Nivel-5.

### Sources de los módulos
- **A. Git remoto con tags** (heredado): los repos/tags no existen; rompería
  el build local y el de release.
- **B. `type: dir` relativo al manifiesto** ✅: construye siempre desde el
  checkout local del monorepo — sirve igual para la verificación local del
  operador y para `release.yml` (Fase 5). `skip: [target]` evita copiar
  artefactos de cargo.
- **C. `type: git` con `path:` local**: solo archivos commiteados — incómodo
  para iterar (cada ajuste exige commit previo) sin aportar nada al alpha.

### Comando principal y módulos
- `command: lnxdrive-preferences` (la GUI real). El daemon se lanza con
  `flatpak run --command=lnxdrived com.strangedaystech.LNXDrive`.
- Módulo engine: `cargo build --release --locked -p lnxdrive-daemon -p
  lnxdrive-cli` (los dos binarios reales: `lnxdrived`, `lnxdrive`).
- Módulo gnome: **meson** con `-Denable_nautilus=false -Denable_shell=false`
  — las extensiones de Nautilus/Shell y el provider GOA se cargan en procesos
  del *host* y no pueden vivir dentro del sandbox; el meson existente ya
  instala panel, iconos, `.desktop`, metainfo y schema (los "install stages"
  que pedía el Charter, sin duplicarlos a mano).

### Política de sandbox (finish-args)
- Se elimina `--socket=session-bus` (acceso sin restricción) en favor de
  **nombres scoped**: `--own-name=com.strangedaystech.LNXDrive` +
  `--talk-name=org.freedesktop.secrets` + `--talk-name=org.gnome.OnlineAccounts`
  — alineado con RISK-002 (superficie D-Bus mínima).
- `--device=all` para `/dev/fuse` (no existe clase de device más fina en
  Flatpak); el riesgo R2 del Charter ya prevé el smoke-test de FUSE bajo
  sandbox en VM antes de publicar.
- `--filesystem=home` literal del Charter (la raíz de sync vive en `$HOME`).

### Red durante el build
- cargo descarga crates.io vía `build-args: --share=network`. Una submission a
  Flathub exigiría sources vendorizados (`flatpak-cargo-generator`); se difiere
  — el alpha distribuye bundle por GitHub Releases (registrado como follow-up).

## Decision

Runtime **`org.gnome.Platform 49`** (drift documentado vs el "47" del Charter,
con atomic update de la tabla Files-to-modify), sources **`type: dir`** por
módulo con `skip`, `command: lnxdrive-preferences`, módulo gnome vía **meson**
con extensiones host-side deshabilitadas, bus de sesión **scoped** (own-name +
talk-names), `--device=all` para FUSE y red solo en build para cargo.

## Consequences

- El bundle construye reproduciblemente desde cualquier checkout del monorepo
  sin depender de tags inexistentes; `release.yml` (Fase 5) lo reutiliza tal cual.
- Early adopters necesitan `org.gnome.Platform//49` (descarga automática al
  instalar el bundle).
- La integración Nautilus/Shell/GOA queda fuera del Flatpak del alpha — se
  documentará como limitación conocida en el README/release notes (Fase 5).
- Flathub queda explícitamente fuera del alcance del alpha (follow-up FU en el
  registro).
- Si el smoke-test de R2 muestra que FUSE no monta bajo el sandbox ni con
  `--device=all`, aplica la mitigación ya prevista: v0.1.0-alpha.2 con el
  permiso/portal faltante.
