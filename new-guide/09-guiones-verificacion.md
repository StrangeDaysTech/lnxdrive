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

## M1 — "Puedo entrar" (pendiente de abrir)

*(Guion se escribe al abrir el batch. Esqueleto esperado: instalar el bundle,
Sign In vía GOA con cuenta real de pruebas, verificar cuenta visible en UI,
`secret-tool search service lnxdrive` con token presente,
`dbus-monitor --session | grep -cE "Bearer|eyJ"` = 0.)*

## M2 — "Veo mis archivos" (pendiente de abrir)

## M3 — "Abro un archivo" (pendiente de abrir)

## M4 — "Mis cambios viajan" (pendiente de abrir)

## M5 — "Sobrevive el tiempo" (pendiente de abrir)

## M6 — "No destruye datos" (pendiente de abrir)

---

*new-guide · documento de trabajo del CHARTER-02.*
