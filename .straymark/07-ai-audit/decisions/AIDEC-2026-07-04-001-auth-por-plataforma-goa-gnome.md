---
id: AIDEC-2026-07-04-001
title: "Auth por plataforma: GOA como vía de login en GNOME, PKCE loopback como ruta universal"
status: accepted
created: 2026-07-04
agent: claude-code-v1.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [8]
tags: [auth, oauth, goa, pkce, gnome, multi-desktop, replanteo]
related: [ADR-2026-07-04-001, ADR-2026-07-04-002, CHARTER-02-road-to-functional-v0-1, AILOG-2026-07-04-001]
---

# AIDEC: Auth por plataforma — GOA en GNOME, PKCE loopback universal

## Context

El replanteo del 2026-07-04 (`new-guide/03-diagnostico-login.md`,
`06-catalogo-desviaciones.md` C1) encontró el login GUI roto: el daemon devuelve
en `StartAuth` una URL OAuth sin parámetros. En el código existen **dos
mecanismos de auth reales y completos** que nunca se cablearon end-to-end desde
la GUI: (a) el flujo OAuth2 Authorization Code + PKCE con callback loopback
(`lnxdrive-graph/src/auth.rs`, hoy solo usado por el CLI) y (b) el backend GOA
(GNOME Online Accounts) inyectado en el daemon
(`goa_auth_backend.rs`, endurecido en RISK-002 para no exponer tokens por D-Bus).
Al replantear M1 ("puedo entrar") hubo que decidir cuál es la vía primaria de la
GUI de GNOME, considerando que el proyecto apunta a múltiples escritorios en
v1.0 (KDE, COSMIC, XFCE/MATE — sin GOA).

## Problem

¿Qué ruta de autenticación dispara el botón "Sign In" de la app de preferencias
de GNOME, y qué papel juega la otra ruta — sabiendo que cada escritorio futuro
tendrá artefactos ejecutables/de configuración específicos de su plataforma?

## Alternatives Considered

### Alternativa A — PKCE loopback como vía primaria en todas las UIs
- **Pros**: un solo flujo que controlamos por completo; idéntico en todos los
  escritorios (sirve directo a v1.0); no depende de la integración
  Microsoft↔GOA de cada versión de GNOME.
- **Cons**: en GNOME ignora el SSO nativo (si el usuario ya tiene su cuenta
  Microsoft en Online Accounts, le pedimos loguearse otra vez en un navegador);
  experiencia menos integrada al escritorio; la guía prescribía GOA para GNOME.

### Alternativa B — GOA como vía GNOME; PKCE loopback como ruta universal (elegida)
- **Pros**: cada escritorio usa sus artefactos nativos (principio del proyecto
  multi-escritorio: en GNOME eso es GOA; en KDE será KAccounts o equivalente);
  SSO real si la cuenta ya existe en Online Accounts; coherente con la guía y
  con el endurecimiento RISK-002 ya invertido en el backend GOA; PKCE queda
  intacto para CLI y para plataformas sin GOA.
- **Cons**: dependemos de que los tokens GOA traigan scopes utilizables para
  Graph (`Files.ReadWrite.All`) — riesgo R1 de CHARTER-02, verificado como
  primer gate de M1; dos rutas vivas exigen mantener ambas probadas.

## Decision

**Alternativa B** — decisión **D2 del operador** (2026-07-04): "debemos
considerar, respecto de los otros escritorios, que se deberán diseñar artefactos
ejecutables o de configuración específicos de esa plataforma; así que podemos
optar, en el caso de GNOME, por GOA".

Concreción:
1. El botón de login de la app de preferencias GNOME usa la vía **GOA**
   (`CompleteAuthViaGOA`, feature `goa` activa en el build shippeado).
2. El flujo **PKCE loopback** es la ruta universal: CLI hoy; vía por defecto de
   los escritorios sin servicio de cuentas nativo en v1.0. El placeholder roto
   de `StartAuth` se arregla poblando la URL PKCE real (deja de mentir).
3. `redirect_uri` canónico: `http://127.0.0.1:8400/callback` (se elimina la
   variante `localhost:8400` de `authenticate.rs:20`).
4. Si M1 demuestra que los tokens GOA no sirven para Graph (riesgo R1), el
   fallback es PKCE como vía GUI y esta decisión se re-abre con el operador.

## Consequences

**Positivas**: M1 tiene vía definida y verificable; el diseño por plataforma
queda enunciado como principio (cada UI usa los artefactos nativos de su
escritorio); las dos inversiones existentes (GOA endurecido + PKCE completo)
se aprovechan en lugar de elegir-y-tirar.

**Negativas**: matriz de prueba de auth ×2 (GOA y PKCE deben mantenerse
funcionales); la promesa GNOME depende de un componente externo (GOA) cuyo
comportamiento por versión hay que vigilar.

**Riesgos**: R1 de CHARTER-02 (scopes GOA↔Graph) — gate temprano de M1 con
fallback definido. La documentación de la guía se actualiza en el plan B7
(`new-guide/08-plan-actualizacion-guia.md`).

---

<!-- Template: StrayMark | https://strangedays.tech -->
