# Hoja de Ruta

> **Ubicación:** `09-Referencia/02-hoja-de-ruta.md`
> **Relacionado:** [Resumen Ejecutivo](../01-Vision/01-resumen-ejecutivo.md), [Estrategia de Testing](../06-Testing/01-estrategia-testing.md), [Estructura de Repositorios](../08-Distribucion/01-estructura-repositorios.md)
> **Reescrito:** 2026-07-04 (replanteo aprobado — ver §Historial al final)

---

## Visión General

La hoja de ruta de LNXDrive se organiza en **milestones por capacidad
demostrable**, no en fases de implementación ni en tags de versión:

> **Un milestone es una capacidad que el operador puede demostrar en su máquina,
> contra su proveedor de nube real, siguiendo un guion escrito. El tag de versión
> es una consecuencia de los milestones, nunca el milestone mismo.**

Este principio corrige el defecto raíz del esquema anterior (fases como listas
de features con checkboxes, hitos como "fases completadas"), que permitió
acumular avance real sin llegar a un sistema funcional end-to-end. El
diagnóstico completo está en el catálogo de desviaciones del replanteo
(`new-guide/06-catalogo-desviaciones.md`) y las decisiones que lo acompañan en
[ADR-2026-07-04-001](../../.straymark/02-design/decisions/ADR-2026-07-04-001-restaurar-delimitacion-crates.md),
[ADR-2026-07-04-002](../../.straymark/02-design/decisions/ADR-2026-07-04-002-telemetria-interna-only.md)
y [AIDEC-2026-07-04-001](../../.straymark/07-ai-audit/decisions/AIDEC-2026-07-04-001-auth-por-plataforma-goa-gnome.md).

### Las tres reglas

1. **Ningún milestone de capacidad se cierra contra mock.** Los mocks siguen
   siendo la base del CI de componentes; no cierran hitos. Ver el tier
   [E2E-real](../06-Testing/01-estrategia-testing.md) de la estrategia de testing.
2. **Cada milestone lleva su guion de verificación**, escrito al abrirlo (no al
   cerrarlo): "haz esto, observa esto".
3. **Cada milestone cabe en 1–3 semanas.** Si no cabe, está mal cortado.

### Estado actual (2026-07-04)

Lo construido hasta hoy (Charter-01, cerrado): motor de sincronización con
Microsoft Graph (delta, upload/download, rate limiting), files-on-demand FUSE,
stack GNOME (panel GTK4, indicador Shell, extensión Nautilus, backend GOA),
CLI, packaging Flatpak, infraestructura de release, 4 riesgos P0 de seguridad
cerrados. **Lo que falta para un sistema funcional**: login GUI end-to-end,
refresh de token en runtime y detección de conflictos — exactamente lo que
cubre la escalera M1–M6.

---

## Camino a `v0.1.0-alpha.1` — la escalera de capacidades

Ejecutada por [CHARTER-02 (Road to functional v0.1)](../../.straymark/charters/02-road-to-functional-v0-1.md);
cada milestone es un batch del Charter y un
[milestone de GitHub](https://github.com/StrangeDaysTech/lnxdrive/milestones)
al que se asignan los issues que lo bloquean.

| M | Capacidad ("puedo…") | Criterio de aceptación (resumen del guion) |
|---|---|---|
| **M0** | *Sé qué es verdad* | Los 23 issues del risk-analysis re-verificados contra el código actual; fantasmas cerrados con evidencia `file:line`; el resto asignado al milestone que bloquea |
| **M1** | **Puedo entrar** | Desde la app de preferencias instalada, login real con cuenta Microsoft vía GOA; tokens solo en keyring (cero tokens en tráfico D-Bus); la UI muestra la cuenta conectada |
| **M2** | **Veo mis archivos** | Tras el login, el delta inicial puebla placeholders del OneDrive real; `lnxdrive status` refleja cuenta y conteo |
| **M3** | **Abro un archivo** | Clic en un placeholder en Nautilus → hidrata → abre; `user.lnxdrive.state` = `hydrated` |
| **M4** | **Mis cambios viajan** | Edición local visible en OneDrive web; edición remota baja — bidireccional demostrado |
| **M5** | **Sobrevive el tiempo** | Daemon 24h+ con sesión activa: refresh automático de token, delta continuo, sin reinicios manuales |
| **M6** | **No destruye datos** | Conflicto simultáneo local+remoto detectado y materializado sin pérdida silenciosa (mínimo: keep-both). Implementado en `lnxdrive-conflict` (ADR-2026-07-04-001). P0 de data-integrity verificados o resueltos |

**Política de tag:** `v0.1.0-alpha.1` se corta cuando M1–M6 son demostrables.
El alcance del alpha no cambia (motor OneDrive + stack GNOME + Flatpak,
GNOME-only); lo que cambia es el gate: capacidad verificada, no checklist.

Los guiones de verificación completos viven en `new-guide/09-guiones-verificacion.md`
(se escriben al abrir cada batch).

---

## `v0.2.0-beta` — dos epics

### Epic "distribuible"

Los diferimientos heredados de Charter-01, sin cambios de fondo:

- **Flathub**: vendoring de crates cargo (FU-003) + alineación del app-id
  Flatpak↔AppStream↔GSettings (FU-008) + submission.
- **Packaging**: RPM, DEB, AUR, AppImage.
- **Panel GTK4**: grupo **System** (auto-start, cache, política de
  deshidratación) — requiere nueva API D-Bus del daemon (AIDEC-2026-05-31-001).
- **D-Bus**: fallback completo por Unix-socket (arrastra bump zbus 5.x,
  TDE-2026-05-28-002).
- **CLI files-on-demand**: wiring de `pin/unpin/hydrate/dehydrate` al motor FUSE
  vía IPC (FU-007 — hoy son stubs).
- **Seguridad D-Bus**: authn/authz (#20) y rate limiting (#22);
  `GraphClient`/TokenSource (#31, TDE-2026-05-29-001).
- **i18n**: estructura de internacionalización (las traducciones van a v1.0.0).
- Landing page; cobertura formal `cargo tarpaulin`.

### Epic "restaurar delimitación de crates" (ADR-2026-07-04-001)

- Extraer la lógica de auditoría de `lnxdrive-core` a `lnxdrive-audit`
  (la persistencia sigue en la SQLite compartida vía `IStateRepository`).
- Implementar `lnxdrive-telemetry` **interna-only** (ADR-2026-07-04-002):
  auto-observación — el sistema conoce su estado, avisa y reacciona localmente.
  **Sin export externo de ningún tipo** (el diseño OTLP→Google Cloud del
  documento original de telemetría queda descartado).
- (`lnxdrive-conflict` ya quedó poblado por M6.)

---

## `v1.0.0`

- **Multi-escritorio**: reactivación de las UIs de `experimental/` en secuencia
  Plasma → COSMIC → GTK3 (XFCE/MATE), cada una con su propio Charter.
  Prerequisito: el documento de contrato D-Bus real
  ([Comunicación D-Bus](../08-Distribucion/02-comunicacion-dbus.md)).
  Principio de auth por plataforma: cada escritorio usa sus artefactos nativos
  (AIDEC-2026-07-04-001 — GOA en GNOME; PKCE loopback como ruta universal).
- **Multi-proveedor**: Google Drive y Dropbox como adaptadores de
  `ICloudProvider`, evolucionando el trait hacia la firma rica de
  [Puerto ICloudProvider](../07-Extensibilidad/02-puerto-icloudprovider.md)
  (capabilities, registry con feature flags).
- **i18n**: 5+ idiomas.
- **Multi-cuenta**: namespaces `{provider}:{alias}`
  ([Multi-Cuenta](../07-Extensibilidad/03-multi-cuenta-namespaces.md)).

---

## Horizonte post-1.0

Sin calendario comprometido; se planifican con Charter propio cuando se activen:

- **OneDrive avanzado**: SharePoint/Business (requiere tenant configurable — hoy
  fijo a `consumers`), shared folders, historial de versiones, share links.
- **Más proveedores**: Nextcloud (WebDAV), otros según demanda.
- **Publicación de crates** en crates.io (`lnxdrive-fuse`, `lnxdrive-audit`,
  `lnxdrive-conflict`, `lnxdrive-ratelimit`) con docs.rs — la delimitación de
  crates restaurada por ADR-2026-07-04-001 es el prerequisito estructural.

---

## Mecanismo de seguimiento

- **GitHub Milestones** = capacidad (M0–M6) o paraguas de versión (v0.2, v1.0).
  Cada issue vive en el milestone que bloquea.
- **GitHub Issues** = risk backlog público del proyecto.
- **StrayMark Charters** = unidad de ejecución; las fases del Charter activo son
  los milestones de capacidad en curso. Telemetría ex-post al cierre.
- **Registro de follow-ups** (`.straymark/follow-ups-backlog.md`) = lo pendiente
  fino que no amerita issue.

---

## Historial

- **2026-07-04 — Reescritura por capacidades.** El esquema original de este
  documento (11 fases 0–10 con checkboxes; hitos "MVP/1.0/2.0 = fases
  completadas") queda **superado**: definía avance por features marcadas, sin
  criterios de aceptación, y permitió que Charter-01 llegara a su fase de tag
  con el login roto. La historia completa está en git y el diagnóstico en
  `new-guide/06-catalogo-desviaciones.md`. Mapa del contenido anterior:
  fases 0–3 → ejecutadas (Charter-01 y anteriores); fase 4 (observabilidad) →
  epic estructural v0.2 (redefinida por ADR-2026-07-04-002); fase 5
  (conflictos) → M6; fase 6 (multi-cuenta) → v1.0.0; fase 7 (escritorios) →
  v1.0.0; fase 8 (multi-proveedor) → v1.0.0; fase 9 (OneDrive avanzado) →
  post-1.0; fase 10 (crates.io) → post-1.0.

---

## Ver también

- [Resumen Ejecutivo](../01-Vision/01-resumen-ejecutivo.md) - Visión del proyecto
- [Estrategia de Testing](../06-Testing/01-estrategia-testing.md) - Tier E2E-real y regla anti-mock
- [Estructura de Repositorios](../08-Distribucion/01-estructura-repositorios.md) - Organización del código
- [Artefactos Reutilizables](../07-Extensibilidad/04-artefactos-reutilizables.md) - Crates independientes
- [Gobernanza del Proyecto](../08-Distribucion/03-gobernanza-proyecto.md) - Modelo de contribución
