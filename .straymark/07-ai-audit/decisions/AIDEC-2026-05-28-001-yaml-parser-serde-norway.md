---
id: AIDEC-2026-05-28-001
title: YAML parser — migración de serde_yaml (deprecated) a serde_norway
status: accepted
created: 2026-05-28
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [yaml, dependencies, security, billion-laughs, dos, charter-01, issue-002]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-05-28-003
---

# AIDEC: YAML parser — migración a serde_norway

## Context

El parser de configuración (`lnxdrive-core::config::Config::load`) deserializa
YAML con `serde_yaml = "0.9.34+deprecated"`. ISSUE-002 (alias D5 / SIM-L4-003,
P0) requiere endurecer ese parser contra el ataque **billion-laughs**
(bomba de expansión de alias YAML). `serde_yaml` de dtolnay está **archivado y
sin mantenimiento** (README: "no longer maintained"; tracking RustSec
advisory-db #2132) y **no ofrece ninguna protección configurable ni integrada**
contra alias bombs.

## Problem

¿Con qué librería YAML reemplazamos `serde_yaml` para (a) mitigar billion-laughs
y (b) salir del crate deprecated, manteniendo la API serde (`from_str` a structs
tipados) y minimizando la superficie de migración?

## Alternatives Considered

> Investigación con datos verificados contra crates.io API, GitHub API,
> docs.rs y rustsec.org (corte 2026-05-28). Pesos de evaluación: base de
> usuarios 20 %, mantenimiento activo 25 %, madurez+tests 20 %, protección
> billion-laughs 20 %, calidad de commits/PRs 15 %.

### Alternative 1: `serde_yaml_ng` (acatton)

Continuación API-compatible del serde-yaml de dtolnay.

**Pros**: 4.2M descargas; CI con clippy/miri/fuzz; mantenedor responde.
**Cons**: **NO protege contra billion-laughs** (mismo backend `unsafe-libyaml`),
además de un bug O(n²) de anidamiento sin arreglar; release en crates.io
desfasada ~16 meses respecto al repo; relicenció a MIT-only; "no professional
support". **No resuelve el objetivo de ISSUE-002.** Índice 2.95.

### Alternative 2: `serde_yaml_bw` (Bourumir Wyngs)

Fork endurecido con protección de dos capas (pre-check `Budget` +
`DeserializerOptions` con límites de recursión/alias/nodos, activos por defecto).

**Pros**: protección billion-laughs configurable de primera clase; suite de
tests enorme (yaml-test-suite completa + ~60 tests de seguridad) + fuzzing +
Miri; releases muy activos (may-2026). Índice 4.10.
**Cons**: base de usuarios joven (135k descargas, concentradas vía `axoasset`);
desarrollo **auto-mergeado asistido por IA** (poca revisión humana entre pares);
y —decisivo— depende de una pila de **tres crates `0.0.x` del propio autor**
(`saphyr-parser-bw` 0.0.613 → renombrado a `granit-parser` 0.0.1 en mayo 2026)
en la raíz de confianza, con rebrand en curso y versionado pre-release que puede
romper compatibilidad en cualquier publicación. `edition 2024` (Rust ≥1.85), sin
MSRV declarada.

### Alternative 3: `serde_norway` (cafkafk)

Fork mantenido de serde-yaml usado por eza, utoipa, schematic, mago.

**Pros**: mayor base de usuarios de los forks (6.9M descargas; usuarios
marquee); **es el reemplazo que RUSTSEC recomienda** frente al inseguro
`serde_yml` (RUSTSEC-2025-0068); protección billion-laughs **integrada y activa
por defecto** (límite de recursión 128 + cap de repetición de alias
`events.len()*100`, errores `RecursionLimitExceeded`/`RepetitionLimitExceeded`
en `src/de.rs`); suite de tests heredada + fuzzing + CI con `cargo deny` diario;
RUSTSEC limpio; backend `unsafe-libyaml-norway` con versionado `0.2.x` estable;
licencia dual MIT/Apache-2.0; API drop-in (`from_str`/`to_string`).
**Cons**: límites DoS **hardcoded, no configurables**; release estancado desde
dic-2024 (~17 meses); bus factor 1. Índice 3.95.

### Alternative 4 (descartada de raíz): parsers de bajo nivel

`saphyr`/`saphyr-parser` y `yaml-rust2`: **no deserializan directo a structs
tipados** (romperían `from_str::<Config>()`, exigiendo mapeo manual) y **tampoco
protegen contra billion-laughs** (verificado en su `parser.rs`).

## Decision

**Chosen**: Alternative 3 — `serde_norway`.

**Justification**: El input de ISSUE-002 es un **archivo de configuración local**
(`~/.config/lnxdrive/config.yaml`), no un endpoint de red expuesto a atacantes
arbitrarios. Para ese modelo de amenaza, los límites **hardcoded y activos por
defecto** de `serde_norway` (recursión 128 + cap de repetición de alias) mitigan
billion-laughs de sobra; la configurabilidad de `serde_yaml_bw` es
over-engineering aquí. A cambio obtenemos:

1. La mayor base de usuarios y validación de ecosistema de los forks (eza,
   utoipa, …) y el aval explícito de RUSTSEC.
2. Salida del crate deprecated sin introducir **tres dependencias `0.0.x` de un
   único autor con rebrand activo** en la raíz de confianza — justo el tipo de
   dependencia que el `cargo deny`/`cargo audit` del PR de CI-hardening de esta
   misma Fase 1 señalaría.
3. Migración mínima (swap de crate, API compatible) con protección activa sin
   configurar nada.

El estancamiento de releases (~17 meses) se acepta para una dependencia de
parsing estable (fork maduro de serde-yaml, CI de auditoría diaria); la licencia
permisiva permite forkear si hiciera falta un fix.

## Consequences

### Positive
- Billion-laughs mitigado por defecto, sin código de límites propio que mantener.
- Fuera del `serde_yaml` deprecated; backend con versionado estable.
- Cambio de superficie mínimo; API serde idéntica.

### Negative
- Límites DoS no ajustables por configuración (aceptable para un config local).
- Dependemos de un mantenedor único con cadencia de releases lenta.

### Risks
- **Mantenedor único / release estancado** → Mitigación: licencia MIT/Apache
  permite forkear; el cap de tamaño propio (`MAX_CONFIG_BYTES`) añade una capa
  independiente de la librería.
- **Defensa en profundidad**: además de los límites de `serde_norway`,
  `Config::from_yaml_str` rechaza configs > 1 MiB antes de parsear.

## Implementation

```toml
# lnxdrive-engine/Cargo.toml (workspace)
serde_norway = "0.9"
```

`Config::load` → `Config::from_yaml_str` (cap de tamaño + `serde_norway::from_str`).
Regresión: `lnxdrive-engine/tests/security/billion_laughs.yaml` +
`config::tests::test_billion_laughs_rejected` / `_trips_dos_limit` /
`test_oversized_config_rejected` / `test_default_config_still_parses`.

## References

- serde_norway: https://crates.io/crates/serde_norway · https://github.com/cafkafk/serde-norway
- RUSTSEC-2025-0068 (recomienda serde_norway sobre serde_yml): https://rustsec.org
- serde_yaml unmaintained tracking: https://github.com/rustsec/advisory-db/issues/2132
- serde_yaml_bw (alternativa endurecida evaluada): https://github.com/bourumir-wyngs/serde-yaml-bw

---

<!-- Template: StrayMark | https://strangedays.tech -->
