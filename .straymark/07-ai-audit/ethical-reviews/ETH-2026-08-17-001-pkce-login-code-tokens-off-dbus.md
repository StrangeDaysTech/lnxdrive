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
approved_by: montfort
approved_date: 2026-08-18
---

# ETH: Browser/PKCE login en el daemon — code y tokens fuera del surface D-Bus

> **APPROVED** (2026-08-18). Revisado y aprobado por el operador humano
> (ver `Approval`). Redactado como draft por un agente de IA
> (`AILOG-2026-08-17-002` documenta la implementación). Las tres preguntas
> abiertas quedaron resueltas — ver §"Preguntas abiertas — resoluciones".

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

## Preguntas abiertas — resoluciones (2026-08-18)

Las tres preguntas quedaron resueltas por el revisor humano (montfort) el
2026-08-18:

### 1. Ventana del loopback

**Resolución: mantener 300 s.** La duración de la ventana no es el control
de seguridad; lo son el `state` CSRF + PKCE. Para inyectar un callback
válido, un atacante necesitaría un `code` de Microsoft emitido para nuestro
`client_id` y ligado a un challenge PKCE cuyo verifier solo posee el daemon:
no puede fabricarlo (un code de su propio flujo queda ligado a *su*
verifier y el intercambio con el nuestro falla con `invalid_grant`). Lo
peor alcanzable es un **DoS local del login** (adelantarse al navegador y
consumir el único `accept` con un callback forjado), sin robo de
credenciales. Reducir la ventana perjudica UX (MFA, redes lentas) sin
ganancia de seguridad medible. Mejora diferible a v0.2: no tumbar el
listener si un exchange falla (reintentar dentro de la ventana).

### 2. Cuenta mínima en fallo de Graph (ruta GOA)

**Resolución: mantener la cuenta mínima.** El path de sync no usa
`onedrive_id` para construir peticiones: delta es `/me/drive/root/delta`
(`delta.rs:45`), upload `/me/drive/root:…`, metadata/borrado
`/me/drive/items/{id}` — todo relativo a `me/drive`, resuelto por el token.
El drive id vacío es solo metadato y no rompe nada (el campo es
`TEXT NOT NULL`; string vacío lo satisface). Persistir deja al daemon salir
de `WaitingForAuth` e intentar el sync: si el token GOA no trae scopes
(R1 materializado), el fallo de Graph aparece como error real en
`journalctl` — la señal que se quiere para observar R1. No persistir deja
la inconsistencia opuesta (UI "autenticado", daemon esperando en silencio),
que oculta R1. El `warn!` marca el caso; `onedrive_id` vacío queda
documentado como "token GOA sin scopes Graph utilizables".

### 3. Peer-credentials en D-Bus

**Resolución: diferir a v0.2.** Alineado con el triage M0: #20 (authn/authz
D-Bus, P2) y #22 (rate limiting) ya están en el milestone `v0.2.0-beta`.
En un daemon de sesión, filtrar por UID es un **no-op** (daemon y llamadores
legítimos comparten UID; el filtro no distingue la app de preferencias de un
proceso malicioso del mismo usuario). Un chequeo por identidad Flatpak solo
aporta en el sandbox y no debe meterse con prisa en el alpha. El vector de
mayor impacto (tokens **y** code por el bus) ya está cerrado por RISK-002 +
la retirada de `CompleteAuth`. Riesgo residual acotado: un proceso del mismo
usuario puede forzar login contra una cuenta GOA atacante → como mucho añade
una entrada de keyring **nueva** de **otra** cuenta, sin tocar la legítima
(`ETH-2026-05-29-001` §3). Tracking: #20 y #22.

## Approval

**APPROVED** — revisado y aprobado por el operador humano (`approved_by:
montfort`) el 2026-08-18, junto a `AILOG-2026-08-17-002`. El cambio de
código correspondiente (PR #73) tiene levantado el gate del ETH; el merge
queda pendiente únicamente del gate de capacidad del operador (guion
`new-guide/09` §M1 contra OneDrive real, incluye el gate R1).
