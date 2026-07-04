# LNXDrive Packaging

Scripts y configuraciones de empaquetamiento para LNXDrive.

## Estado actual (v0.1.0-alpha)

El alpha distribuye **únicamente Flatpak** (decisión de alcance del
Charter-01). Los demás formatos están diferidos al milestone `v0.2.0-beta`:

| Formato | Estado |
|---------|--------|
| **Flatpak** (`flatpak/`) | ✅ Activo — bundle publicado en GitHub Releases |
| RPM (Fedora, RHEL, openSUSE) | Diferido a `v0.2.0-beta` |
| DEB (Debian, Ubuntu, Mint) | Diferido a `v0.2.0-beta` |
| AUR (Arch Linux) | Diferido a `v0.2.0-beta` |
| AppImage | Diferido a `v0.2.0-beta` |
| Flathub | Diferido a `v0.2.0-beta` (requiere vendoring de crates) |

## Estructura

```
lnxdrive-packaging/
├── flatpak/          # Manifiesto Flatpak (com.strangedaystech.LNXDrive.yaml)
└── scripts/          # Scripts de build y release (placeholder)
```

Los subdirectorios `rpm/`, `debian/`, `aur/` y `appimage/` se crearán cuando
sus formatos se activen en `v0.2.0-beta`.

## Build local (Flatpak)

Desde la **raíz del monorepo** (las sources del manifiesto son rutas `dir`
relativas):

```bash
flatpak-builder --user --install --force-clean build-dir \
  lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
```

O con el builder de Flathub si `flatpak-builder` no está empaquetado en tu
distro:

```bash
flatpak install --user flathub org.flatpak.Builder
flatpak run org.flatpak.Builder --user --install --force-clean build-dir \
  lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
```

El bundle incluye `lnxdrived` (daemon), `lnxdrive` (CLI) y
`lnxdrive-preferences` (panel GTK4). Las extensiones de Nautilus/Shell y el
provider GOA son componentes host-side y quedan fuera del sandbox.

## Componentes host-side (extensión de GNOME Shell)

El indicador del panel es una **extensión de GNOME Shell** que se carga dentro
del proceso `gnome-shell` del host: no puede vivir en el sandbox del Flatpak, así
que el manifiesto la excluye (`-Denable_shell=false`). Por eso, **instalar solo el
Flatpak no coloca el indicador** — hay que instalar la extensión aparte, en el
host, con Meson. Compatibilidad declarada: GNOME Shell 45–50.

Instalación en el **prefix de usuario** (sin root, recomendado para probar):

```bash
cd lnxdrive-gnome
meson setup build --prefix ~/.local \
  -Denable_shell=true \
  -Denable_nautilus=false -Denable_preferences=false -Denable_goa=false
meson install -C build
# → ~/.local/share/gnome-shell/extensions/lnxdrive-indicator@strangedaystech.com/
```

Instalación **a nivel de sistema** (para empaquetado distro):

```bash
cd lnxdrive-gnome
meson setup build -Denable_shell=true \
  -Denable_nautilus=false -Denable_preferences=false -Denable_goa=false
sudo meson install -C build
```

Tras instalar, habilitar la extensión y **reiniciar la sesión de GNOME**
(en Wayland es obligatorio logout/login; el Shell solo reescanea extensiones al
iniciar sesión):

```bash
gnome-extensions enable lnxdrive-indicator@strangedaystech.com
```

El indicador se conecta al daemon (`lnxdrived`) por el bus de sesión D-Bus
(`com.strangedaystech.LNXDrive`), que el Flatpak arranca por activación al primer
connect. La extensión de Nautilus se instala por el mismo mecanismo
(`-Denable_nautilus=true`); el provider GOA está diferido.

## Release

El workflow `.github/workflows/release.yml` (raíz del monorepo) construye el
bundle y publica el GitHub Release con `SHA256SUMS` al pushear un tag `v*`.

## Licencia

GPL-3.0-or-later
