# Milestones por capacidad demostrable

**Fecha:** 2026-07-04 · **Estado:** propuesta aprobada en dirección (decisión D4);
detalles de corte afinables · **Reemplaza:** el esquema "hito = fases completadas
por checkbox + tag SemVer" de `lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md`.

---

## 1. Filosofía

> **Un milestone es una capacidad que el operador puede demostrar en su máquina,
> contra su OneDrive real, siguiendo un guion escrito. El tag de versión es una
> consecuencia de los milestones, nunca el milestone mismo.**

### Tres reglas duras

1. **Ningún milestone de capacidad se cierra contra mock.** Los mocks
   (`wiremock`, `mock-graph`, `mock-dbus-daemon.py`) siguen siendo excelentes
   para CI de componentes; no cierran hitos.
2. **Cada milestone lleva su guion de verificación** — manual está bien: "haz
   esto, observa esto". El guion se escribe al abrir el milestone, no al cerrarlo.
3. **Cada milestone cabe en 1–3 semanas.** Si no cabe, está mal cortado.

---

## 2. La escalera hacia el v0.1 funcional

| M | Capacidad ("puedo…") | Trabajo principal | Issues/FUs que arrastra |
|---|---|---|---|
| **M0** | *"Sé qué es verdad"* — triage de re-verificación: los 23 issues contrastados contra el código actual; cerrar fantasmas, re-priorizar el resto | Solo lectura/análisis; barato | Sospechosos: #9, #14; solapamiento #21↔ISSUE-002 (ver catálogo §6) |
| **M1** | **"Puedo entrar"** — login real con cuenta Microsoft desde la app de preferencias (vía **GOA**, decisión D2); tokens al keyring; la UI muestra la cuenta conectada | Cablear el camino GOA end-to-end en GNOME; arreglar el `StartAuth` placeholder (PKCE como ruta universal/CLI); resolver redirect_uri inconsistente y origen del app_id | El bug de login observado; divergencia de rutas de auth. **Verifica de paso**: scopes de tokens GOA utilizables para Graph |
| **M2** | **"Veo mis archivos"** — tras login, el delta inicial puebla placeholders de mi OneDrive real; `lnxdrive status` refleja cuenta y conteo | Integración login→engine; primera pasada real contra Graph | — |
| **M3** | **"Abro un archivo"** — clic en placeholder en Nautilus → hidrata → abre; xattr/estado correctos | Ya mayormente implementado; verificarlo contra nube real | #18 (race dehydration) entra al radar |
| **M4** | **"Mis cambios viajan"** — edición local aparece en OneDrive web; edición remota baja. Bidireccional demostrado | Ya mayormente implementado; verificación real | #16 (inotify overflow), #17 (observer blocking) |
| **M5** | **"Sobrevive el tiempo"** — daemon 24h+ con sesión activa: refresh automático de token, delta continuo, sin reinicios | Cablear `refresh_if_needed` + manejo de 401 en el loop | #27 (resource leaks); #14 si sobrevive al triage M0 |
| **M6** | **"No destruye datos"** — conflicto simultáneo local+remoto detectado y materializado sin pérdida silenciosa (resolución cruda OK: keep-both) | Implementar detección de conflictos **en `lnxdrive-conflict`** (decisión D1) | #19, #25; P0 data-integrity #8/#12/#13 verificados o resueltos |

### Política de tags

- **`v0.1.0-alpha.1` se corta cuando M1–M6 son demostrables** (decisión D4).
  Ahí se resuelven FU-010/FU-011 (fecha real en CHANGELOG y metainfo).
- Lectura honesta asumida: el alpha no estaba "a un tag de distancia"; estaba a
  ~6 capacidades de distancia. El trabajo ya hecho (transporte Graph, FUSE,
  stack GNOME) es lo que hace que M2–M4 sean cortos.

---

## 3. Después del v0.1 funcional

### v0.2.0-beta — dos epics

**Epic "distribuible"** (lo ya diferido por Charter-01, sin cambios):
- Flathub: FU-003 (vendoring de crates) + FU-008 (alineación app-id)
- Packaging RPM/DEB/AUR/AppImage
- Grupo System del panel (+ nueva API D-Bus; AIDEC-2026-05-31-001)
- D-Bus Unix-socket fallback (+ TDE-2026-05-28-002, bump zbus 5.x)
- Estructura i18n; landing page; cobertura formal tarpaulin
- FU-007: IPC FoD para los comandos del CLI (`pin/hydrate/…`)
- TDE-2026-05-29-001 / issue #31 (GraphClient TokenSource)
- P1 de seguridad D-Bus: #20 (authn/authz), #22 (rate limiting)

**Epic "restaurar delimitación de crates"** (nuevo, por decisión D1):
- Extraer la lógica de audit de `lnxdrive-core` a `lnxdrive-audit`
  (la persistencia puede seguir en la SQLite compartida vía `IStateRepository`,
  como la propia guía prescribe)
- Implementar `lnxdrive-telemetry` **redefinida** (decisión D3): agente de
  auto-observación interna — el sistema determina su propio estado, emite avisos
  y puede reaccionar. **Sin export externo de ningún tipo**
- (`lnxdrive-conflict` ya quedó poblado por M6)

### v1.0.0 — sin cambios de fondo

- Reactivación de UIs de `experimental/` (Plasma → COSMIC → GTK3), cada una con
  su Charter y **con el doc de contrato D-Bus como prerequisito** (que ya existirá,
  ver plan de actualización §2)
- Multi-proveedor (Google Drive, Dropbox) — evolucionando `ICloudProvider` hacia
  la firma rica de `07-Extensibilidad/02`
- 5+ idiomas

---

## 4. Mecanismo de tracking

**GitHub Milestones** sobre los issues existentes (no se recrea el Project,
consistente con el modelo de tracking del proyecto):

- M0…M6 como milestones de GitHub; cada issue asignado al hito que bloquea.
- Los issues que M0 confirme como fantasma se cierran con nota de verificación.
- Charter-02 ("Road to functional v0.1") usa M0–M6 como fases; cada fase de
  Charter = milestone de GitHub. Riesgo, roadmap y gobernanza quedan conectados
  (cierra la Desconexión 3).

---

## 5. El tier de testing que falta: E2E-real

Nuevo nivel en la estrategia de testing: pruebas contra una **cuenta OneDrive
real de pruebas**.

- No es CI-able fácilmente — igual que FUSE, que ya tiene este patrón en el
  proyecto (test T101 `#[ignore]`, gate local). Mismo trato: tests `#[ignore]`
  + guiones manuales que el operador corre localmente como gate de milestone.
- La guía documentará: cuenta de pruebas dedicada, guiones existentes, y la
  regla "mock no cierra hitos".
- Es la vacuna estructural contra recaer en el teatro del mock (Desconexión 2).

---

*new-guide · documento de trabajo — no canónico. La hoja de ruta canónica se
reescribe según `08-plan-actualizacion-guia.md` §1.*
