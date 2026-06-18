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

## Release

El workflow `.github/workflows/release.yml` (raíz del monorepo) construye el
bundle y publica el GitHub Release con `SHA256SUMS` al pushear un tag `v*`.

## Licencia

GPL-3.0-or-later
