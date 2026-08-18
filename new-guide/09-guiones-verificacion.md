# Guiones de verificación por milestone (CHARTER-02)

> Regla 2 del replanteo: **cada milestone lleva su guion de verificación,
> escrito al abrirlo**. Este archivo los acumula; el gate de cada batch es que
> el operador ejecute el guion contra su OneDrive real y confirme.

---

## M0 — "Sé qué es verdad" (abierto 2026-07-04)

**Capacidad**: los 23 issues del risk-analysis re-verificados contra el código
actual; fantasmas cerrados con evidencia `file:line`; el resto asignado al
milestone que bloquea.

### Guion

1. Para cada issue abierto con label `from-risk-analysis`: leer el cuerpo,
   localizar el código actual que le corresponde, emitir veredicto
   (RESUELTO / VIGENTE / PARCIAL / NO-APLICA-AÚN) con evidencia `file:line`.
2. Cierres: todo issue cerrado lleva comentario con la evidencia exacta y la
   condición de reapertura si aplica.
3. Asignaciones: todo issue vigente queda en el milestone de capacidad que
   bloquea (M1–M6), en un paraguas de versión (v0.2/v1.0), o en backlog sin
   milestone con justificación.
4. Degradaciones de prioridad: comentadas con el razonamiento (p. ej. session
   bus ⇒ defensa-en-profundidad).

### Verificación (operador)

```bash
# Ningún issue abierto sin decisión de triage:
gh issue list --state open --label from-risk-analysis \
  --json number,title,milestone --jq '.[] | select(.milestone == null)'
# → debe devolver SOLO los ítems de backlog deliberado (#23, #28, #30)

# Los cierres tienen evidencia:
gh issue view 9 --comments | grep "sync_item.rs"
gh issue view 12 --comments | grep "xattr.rs"

# Conteo esperado tras el batch: 17 abiertos (13 con milestone + 3 backlog + #66 nuevo), 7 cerrados hoy
gh issue list --state open --json number | jq length   # → 17
```

**Resultado del batch (2026-07-04)** — *ningún P0 real sobrevivió al triage*:

- 7 cerrados — 4 RESUELTOS (#9, #12, #14, #24) y 3 NO-APLICA-AÚN (#11, #17, #26).
- 13 asignados — M3: #18 · M5: #15, #27 · M6: #13, #19, #25 · v0.2: #8, #16, #20, #21, #22, #31 · v1.0: #29.
- 3 a backlog deliberado (#23, #28, #30, sin milestone).
- 3 degradaciones de prioridad (#8 y #13 P0→P2; #20 P1→P2); #21 estrechado a path-confinement.
- **#66 creado** (refresh de auth token → M5, = FU-016).

---

## M1 — "Puedo entrar" (abierto 2026-08-17)

**Capacidad**: login real con cuenta Microsoft desde la app de preferencias;
tokens SOLO en el keyring; la UI muestra la cuenta conectada. Dos rutas:
**GOA** (primaria, decisión D2) y **browser/PKCE loopback** (fallback
universal). Verifica de paso (riesgo R1) que el token GOA trae scopes
utilizables para Graph.

**Trabajo asociado**: issue #70 (= FU-017) · AIDEC-2026-07-04-001 · riesgo R1
(Charter-02 §Risks). Diagnóstico de partida: `03-diagnostico-login.md`.

### Prerrequisitos del entorno

- Sesión GNOME en la máquina del operador + **cuenta Microsoft real de
  pruebas dedicada** (no la de producción).
- Daemon **real** atendiendo el bus de sesión — regla 1 del replanteo: ningún
  milestone se cierra contra mock (el "funcionaba" de la VM era
  `mock-dbus-daemon.py`, ver `03-diagnostico-login.md` §5):

  ```bash
  pgrep -af mock-dbus-daemon.py                          # → sin proceso
  busctl --user list | grep com.strangedaystech.LNXDrive # → PID del dueño
  ps -o comm= -p <PID>                                   # → lnxdrived
  ```

- App de preferencias con feature `goa` (ya está en `default`,
  `lnxdrive-gnome/preferences/Cargo.toml:29`).
- Proveedor GOA (`lnxdrive-gnome/goa-provider/`) instalado host-side — el
  batch activa `enable_goa=true` y lo instala (issue #70).
- Local checks del Charter-02 §Verification en verde (`cargo test`,
  clippy `-D warnings`, `cargo build --features goa`).

### Guion

Orden de ejecución: A (estado inicial sin cuenta) → B (ruta GOA) → C (gate
R1) → D (ruta browser/PKCE).

**A. Onboarding sin cuenta GOA**

1. En el estado inicial (sin cuenta `lnxdrive_microsoft` en GOA), abrir la
   app de preferencias.
2. Observar: no hay botón GOA; la página ofrece "Sign In" (browser) **y** un
   botón que abre GNOME Online Accounts para crear la cuenta (issue #70,
   "botón para lanzar GNOME Online Accounts cuando no hay cuenta"). Sin
   callejón sin salida visible.

**B. Ruta GOA (primaria, D2)**

1. GNOME Settings → Online Accounts → añadir la cuenta del proveedor
   LNXDrive (tipo `lnxdrive_microsoft`) con la cuenta de pruebas. GOA hace el
   OAuth de Microsoft en su propio diálogo; los tokens quedan en GOA.
2. Reiniciar el daemon y abrir la app de preferencias.
3. Observar: aparece el botón "Use existing Microsoft account" (solo visible
   cuando la cuenta existe).
4. Clic. Observar: NO se abre navegador; estado de espera; transición a la
   página de carpeta; **la UI muestra e-mail y display name de la cuenta**.
5. Con `dbus-monitor` corriendo durante el clic: se emite la señal
   `AuthStateChanged("authenticated")` — antes del batch esta ruta no la
   emitía (issue #70).

**C. Gate R1 — el token GOA opera contra Graph**

Confirmación empírica de R1 (issue #70: el proveedor pide
`Files.ReadWrite.All` por diseño; aquí se demuestra):

```bash
# Precaución: imprime el token en la terminal — SOLO en la máquina del
# operador; limpiar la variable al terminar.
T=$(secret-tool lookup service lnxdrive username <email> | jq -r .access_token)
curl -s -H "Authorization: Bearer $T" \
  https://graph.microsoft.com/v1.0/me/drive | jq '{id, quota}'
unset T
# → HTTP 200 con la quota REAL de la cuenta de pruebas (prueba scopes
#   Files.*; /me prueba User.Read).
# Si devuelve 403/insufficient scopes: R1 se materializa → la capacidad NO
# se cierra por GOA, se intenta la ruta D como fallback y se reabre D2
# (Charter-02 §Risks).
```

**D. Ruta browser/PKCE (fallback universal)**

1. Cerrar la sesión de la ruta B: `lnxdrive auth logout` (vacía el keyring y
   suspende la cuenta), para partir limpio.
2. En la app de preferencias, clic en "Sign In". Observar: el navegador abre
   una URL OAuth v2.0 que ahora **lleva todos los parámetros** — comprobar en
   la barra de direcciones: `client_id`, `response_type=code`, `redirect_uri`,
   `scope`, `state`, `code_challenge`, `code_challenge_method=S256` (antes
   del batch la URL iba vacía; `03-diagnostico-login.md` §3).
3. Observar en la misma URL: `redirect_uri` es el valor único consolidado
   (ya no divergen `127.0.0.1:8400/callback` y `localhost:8400`) y
   `client_id` resuelve aunque `auth.app_id` no esté configurado (origen
   fijado; FU-017).
4. Login en Microsoft. Observar: el redirect llega al loopback (`:8400`); el
   **daemon** captura el `code` directamente (no cruza D-Bus — mejora
   RISK-002), lo intercambia por tokens, los guarda en el keyring y emite
   `AuthStateChanged`.
5. Observar: la app transiciona a autenticada y muestra la cuenta. En el bus
   no se observa ningún método llevando `code`/`state` como argumentos (el
   viejo `CompleteAuth(code, state)` queda retirado/deprecado).

### Verificación (operador)

```bash
# 1. Token SOLO en el keyring (servicio "lnxdrive" — KEYRING_SERVICE,
#    lnxdrive-graph/src/auth.rs:35):
secret-tool search service lnxdrive
# → entrada de la cuenta de pruebas (atributos service/username)

# 2. Nada sensible cruza D-Bus en NINGUNA ruta (B y D):
dbus-monitor --session > /tmp/m1-bus.txt &   # arrancar ANTES de los clics
# ... ejecutar las rutas del guion ...
kill %1
grep -cE "Bearer|eyJ" /tmp/m1-bus.txt        # → 0
grep -E "member=(StartAuth|CompleteAuth)" /tmp/m1-bus.txt
# → StartAuth visible (devuelve la URL); ningún CompleteAuth con code/tokens

# 3. Sin tokens en los logs del daemon:
journalctl --user -u lnxdrive | grep -cE "Bearer|eyJ"   # → 0

# 4. Gate R1: sección C del guion.
```

**Resultado del batch** — *(se llena al cierre: rutas verificadas, evidencia
del gate R1, desviaciones encontradas y su corrección atómica.)*

## M2 — "Veo mis archivos" (pendiente de abrir)

## M3 — "Abro un archivo" (pendiente de abrir)

## M4 — "Mis cambios viajan" (pendiente de abrir)

## M5 — "Sobrevive el tiempo" (pendiente de abrir)

## M6 — "No destruye datos" (pendiente de abrir)

---

*new-guide · documento de trabajo del CHARTER-02.*
