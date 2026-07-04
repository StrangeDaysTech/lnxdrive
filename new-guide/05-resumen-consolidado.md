# LNXDrive — Resumen consolidado (2026-07-03)

Síntesis de los cuatro informes. Todo verificado contra el código del repositorio.

## 1. Arquitectura — "backend común + UIs delgadas" (confirmado)

El patrón es **un daemon común (`lnxdrived`) que habla D-Bus, y UIs por escritorio que son clientes delgados**.

- Backend: workspace Cargo en `lnxdrive-engine/` con **11 crates** en arquitectura hexagonal.
  - Núcleo de dominio puro: `lnxdrive-core` (puertos/traits + casos de uso).
  - Adaptadores: `lnxdrive-graph` (OneDrive), `lnxdrive-cache` (SQLite), `lnxdrive-fuse` (files-on-demand), `lnxdrive-sync` (motor).
  - Binario/IPC: `lnxdrive-daemon` + `lnxdrive-ipc` (9 interfaces D-Bus en `com.strangedaystech.LNXDrive`).
  - Stubs vacíos: `lnxdrive-conflict`, `lnxdrive-audit`, `lnxdrive-telemetry` (la lógica de conflict/audit vive en `lnxdrive-core`).
- El puerto multi-proveedor **ya existe y es agnóstico**: trait `ICloudProvider` con un solo adaptador hoy (OneDrive/Graph). Diseñado para que Google Drive/Dropbox sean "otro impl del trait".
- UIs: solo GNOME es real (GTK4 preferences + indicador Shell + extensión Nautilus). Las de `experimental/` (GTK3, Plasma, COSMIC) son stubs → v1.0.0.

## 2. El bug del login — cable suelto, no funcionalidad ausente

Al dar clic en "Sign In", la app llama por D-Bus a `StartAuth()` del daemon, que devuelve una URL placeholder **sin ningún parámetro** (`service.rs:853`). Microsoft la rechaza por falta de `client_id` (primer obligatorio ausente).

El generador correcto (`PKCEFlow::generate_auth_url()`, `auth.rs:192`) **existe y está completo**, pero el daemon nunca lo invoca — solo lo usa el CLI. Antes "funcionaba" en VM porque corría el mock D-Bus en Python (`mock-dbus-daemon.py:570`) que devolvía una URL falsa y auto-emitía `authenticated`.

**Fix:** cablear en el daemon la ruta PKCE real (generar URL, levantar callback loopback `127.0.0.1:8400`, completar `code → tokens` → keyring). Bug de integración acotado.

## 3. ¿Qué tanto falta de OneDrive?

El transporte con Graph está prácticamente terminado. Los huecos que impiden un uso end-to-end sin vigilancia:

| # | Hueco | Impacto | Tipo |
|---|-------|---------|------|
| 1 | Login GUI no cableado | Bloquea onboarding desde la app | Integración (acotado) |
| 2 | Refresh de token en runtime ausente | El daemon falla al expirar el token (~1h) hasta reiniciar | Integración (acotado) |
| 3 | Detección de conflictos ausente | Riesgo de last-writer-wins; crate `lnxdrive-conflict` vacío | Feature (trabajo real) |
| 4 | Dos rutas de auth divergentes | El use case llama a un `bail!` stub; CLI exige app_id | Consolidación |
| 5 | Tenant fijo a `consumers` | No soporta cuentas de organización (Business) | Config |
| 6 | Upload grande sin progreso | Cosmético (`progress = None`) | Menor |

**Hecho y con tests:** todas las operaciones de archivo del trait (delta con paginación y 410-Gone→resync, download + Range, upload simple y resumable en chunks, metadata, quota, delete), OAuth2 PKCE SHA256 completo, ruta GOA, `client_id` real por defecto, rate limiting + retry 429, motor de sync end-to-end.

**Lectura de conjunto:** los puntos 1-3 son los que hacen que "no funcione end-to-end desde la GUI". 1 y 2 son cableado; 3 es feature genuina. El punto 2 (refresh de token) es el más crítico para presentar el alpha como "motor OneDrive funcional".

## 4. Roadmap

- **v0.1.0-alpha.1 — ACTUAL, recta final** (Charter-01 `in-progress`, falta solo la Fase 6: cortar el tag). Motor OneDrive + CLI + stack GNOME + Flatpak. Solo GNOME.
- **v0.2.0-beta:** grupo System del panel (+ API D-Bus), packaging RPM/DEB/AUR/AppImage, Flathub (+ vendoring), fallback D-Bus Unix-socket, i18n, telemetría opt-in.
- **v1.0.0:** reactivación de UIs experimental (Plasma → COSMIC → GTK3, cada una su Charter), multi-proveedor (Google Drive, Dropbox), 5+ idiomas.

## 5. Nota de honestidad de release

Los huecos #1-#3 de OneDrive están declarados en el alcance del alpha actual pero no funcionan end-to-end. Conviene decidir conscientemente si el alpha se tagea con esos límites documentados o si se resuelven antes del tag — sobre todo el #2 (refresh de token), porque un daemon que muere a la hora es difícil de presentar como "motor OneDrive funcional".

## 6. Próximos pasos propuestos

1. **Arreglar el login GUI** (#1): cablear `StartAuth` del daemon a la ruta PKCE real. Desbloqueo más visible y acotado.
2. **Registrar como follow-ups/issues** los huecos #2, #3, #4 para que entren al backlog con trazabilidad.
