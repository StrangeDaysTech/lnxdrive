---
id: ETH-2026-08-17-001
title: "Browser/PKCE login en el daemon — el authorization code y los tokens no cruzan D-Bus"
status: draft
created: 2026-08-17
agent: qwen3.8-max
confidence: high
review_required: true
risk_level: high
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security, data_privacy]
iso_42001_clause: [8]
gdpr_legal_basis: contract
fria_required: false
tags: [security, credentials, oauth, pkce, goa, keyring, dbus, gdpr]
related:
  - AILOG-2026-08-17-002
  - ETH-2026-05-29-001
  - CHARTER-02-road-to-functional-v0-1
  - RISK-002-security-vulns
approved_by: null
approved_date: null
---

# ETH: Browser/PKCE login en el daemon — code y tokens fuera del surface D-Bus

> **IMPORTANTE**: Este documento es un DRAFT creado por un agente de IA.
> Requiere revisión y aprobación humanas antes de mergear el cambio de
> código correspondiente (ver `AILOG-2026-08-17-002` para la
> implementación). Extiende `ETH-2026-05-29-001` a la ruta browser/PKCE.

## Executive Summary

El batch M1 (issue #70) añade una segunda ruta de login al daemon: el flujo
OAuth2 **browser/PKCE**, donde el daemon genera la URL de autorización, el
usuario se autentica en su navegador, y el redirect OAuth llega a un
servidor loopback (`127.0.0.1:8400`) **dentro del propio proceso del
daemon**. Esta revisión ética cubre dos hechos nuevos respecto a
`ETH-2026-05-29-001`:

1. **El authorization code ya no cruza D-Bus.** Hasta ahora existía
   `Auth.CompleteAuth(code, state)`: cualquier proceso del usuario podía
   ver el code en el bus. El método se retira; el code lo captura el
   loopback interno del daemon y se intercambia por tokens en el proceso.
   El invariante de RISK-002 ("ningún secreto cruza el bus como argumento
   de método") se extiende explícitamente al authorization code, que es un
   bearer de un solo uso capaz de convertirse en tokens.
2. **La ruta GOA ahora persiste cuenta y prueba scopes contra Graph.** El
   login GOA enriquece el perfil con `GET /me`+`/me/drive` (prueba empírica
   del riesgo R1) y persiste la cuenta en SQLite + audit, lo que añade
   datos personales (display name, drive id) al alcance del tratamiento.

## Context

LnXDrive sincroniza archivos entre el OneDrive del usuario y el sistema
local. `ETH-2026-05-29-001` revisó la mudanza de los tokens fuera del
surface D-Bus público para la ruta GOA. M1 completa el diagnóstico de
`new-guide/03-diagnostico-login.md`: la GUI no podía loguearse porque
`StartAuth` devolvía una URL sin parámetros, y la única ruta PKCE funcional
estaba en el CLI. La decisión D2 (AIDEC-2026-07-04-001) fija GOA como vía
primaria en GNOME y PKCE loopback como ruta universal.

## Ethical concerns and how the change addresses them

### 1. Confidencialidad del authorization code

**Preocupación.** El authorization code OAuth es un credential bearer de un
solo uso: quien lo captura dentro de su corta ventana puede intercambiarlo
por tokens. El antiguo `Auth.CompleteAuth(code, state)` lo exponía en el
bus de sesión, visible para cualquier proceso del usuario.

**Mitigación.** El método se retira del surface D-Bus. El code lo recibe el
servidor loopback del propio daemon (`LocalCallbackServer`), se valida el
CSRF `state` y se intercambia por tokens dentro del proceso. Ningún cliente
D-Bus ve el code. El leak-test (`lnxdrive-testing/scripts/leak-test-dbus-tokens.sh`)
sigue vigilando el bus.

### 2. Confidencialidad de los tokens (continuidad de ETH-2026-05-29-001)

**Preocupación.** Los tokens de la ruta PKCE (access + refresh) son
long-lived y equivalen a la contraseña del usuario mientras viven.

**Mitigación.** Idéntica a la ruta GOA: los tokens se guardan solo en el
keyring del sistema (`KeyringTokenStorage`, service `lnxdrive`) y en
memoria del daemon. Nunca atraviesan D-Bus como argumentos. `StartAuth`
devuelve solo la URL de autorización y el `state` CSRF (no secretos); el
`pkce_verifier` permanece en `DaemonState` y no sale del proceso.

### 3. Datos personales añadidos — cuenta persistida

**Preocupación.** El login ahora persiste una cuenta en SQLite (e-mail,
display name, drive id, quota) y un audit entry, además del e-mail en el
keyring como clave. Display name y drive id son datos personales nuevos
respecto al ETH previo.

**Mitigación.** Base jurídica `contract` (Art. 6(1)(b)): el usuario inicia
sesión explícitamente para que la app sincronice sus archivos. Minimización
(Art. 5(1)(c)): solo se guardan los campos necesarios para identificar la
cuenta y el drive. Sin telemetría ni analítica. Los datos permanecen en la
máquina del usuario. El audit entry registra e-mail/display name/drive id
(sin tokens), consistente con el patrón de auditoría existente.

### 4. Llamada a Graph con el token GOA (riesgo R1)

**Preocupación.** La ruta GOA ahora usa el token obtenido de GOA para llamar
`GET /me`+`/me/drive`. Si el token no trae scopes utilizables (riesgo R1
del Charter), la llamada falla; si los trae, el daemon contacta Microsoft
Graph durante el login.

**Mitigación.** La llamada es best-effort y solo de lectura del perfil del
propio usuario autenticado (sin datos de terceros). Si falla, el login
degrada a una cuenta mínima y se registra el warning — el fallo de scopes se
superficie en vez de ocultarse. El gate final lo ejecuta el operador contra
OneDrive real (guion M1).

### 5. Proveedor GOA host-side

**Preocupación.** El build por defecto ahora instala un módulo de proveedor
GOA (`enable_goa=true`) que el `goa-daemon` del host carga.

**Mitigación.** El proveedor solo gestiona la cuenta `lnxdrive_microsoft`
del propio usuario en GOA; no introduce nueva superficie de red ni exfiltra
datos. En Flatpak se mantiene `-Denable_goa=false` porque el proveedor debe
ser visible para el `goa-daemon` del host, no del sandbox.

## GDPR fields

- **Base jurídica** (Art. 6): `contract` — el usuario inicia sesión para
  usar la sincronización.
- **Minimización** (Art. 5(1)(c)): e-mail, display name, drive id, quota y
  tokens; nada más.
- **Limitación de conservación** (Art. 5(1)(e)): access token con TTL de
  Microsoft; refresh token hasta `Logout` (`KeyringTokenStorage::clear`).
  La cuenta en SQLite hasta el logout/eliminación.
- **Integridad y confidencialidad** (Art. 5(1)(f)/Art. 32): tokens solo en
  keyring (reposo) y memoria del daemon (activo); code y tokens nunca como
  argumentos de D-Bus.
- **DPIA** (Art. 35): no requerida — tratamiento local, usuario único, sin
  categorías especiales ni monitorización a gran escala.

## Open questions for the reviewer

1. **Ventana del loopback.** El listener loopback vive hasta 300 s tras
   `StartAuth` si el usuario abandona el flujo. ¿Es aceptable esa ventana o
   reducirla (p. ej. 120 s)?
2. **Cuenta mínima en fallo de Graph (ruta GOA).** Al fallar el enriquecido,
   persistimos una cuenta con display_name=e-mail y drive id vacío para que
   el daemon arranque y exponga el error real. ¿Correcto, o preferible no
   persistir hasta que Graph responda?
3. **Peer-credentials en D-Bus.** Sigue pendiente de `ETH-2026-05-29-001`
   (pregunta 3): ¿restringir ya los llamadores de `Auth.*` por app-id/UID,
   o dejarlo para v0.2?

## Approval

Este ETH es `draft`. Flujo de aprobación:

1. El revisor lee `AILOG-2026-08-17-002` junto a este ETH.
2. El revisor aprueba (`status: approved`, rellenar `approved_by`,
   `approved_date`) o pide revisiones.
3. El PR del batch no se mergea sin este ETH aprobado, según
   `AGENT-RULES.md` (código de autenticación).
