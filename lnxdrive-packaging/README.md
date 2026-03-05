# LNXDrive Packaging

Scripts y configuraciones de empaquetamiento para LNXDrive.

## Descripción

Este repositorio centraliza todo el empaquetamiento de LNXDrive para diferentes formatos y distribuciones:

- **Flatpak**: Distribución universal (recomendado)
- **RPM**: Fedora, RHEL, openSUSE
- **DEB**: Debian, Ubuntu, Linux Mint
- **AUR**: Arch Linux
- **AppImage**: Ejecutable portable

## Estructura

```
lnxdrive-packaging/
├── flatpak/          # Manifiestos Flatpak
├── rpm/              # Especificaciones RPM
├── debian/           # Empaquetamiento Debian
├── aur/              # PKGBUILD para AUR
├── appimage/         # Configuración AppImage
└── scripts/          # Scripts de build y release
```

## Uso

### Flatpak

```bash
cd flatpak
flatpak-builder --user --install builddir org.strangedaystech.LNXDrive.yaml
```

### RPM (Fedora)

```bash
cd rpm
rpmbuild -ba lnxdrive.spec
```

### DEB (Debian/Ubuntu)

```bash
cd debian
dpkg-buildpackage -us -uc
```

### AUR

```bash
cd aur
makepkg -si
```

### AppImage

```bash
cd appimage
./build-appimage.sh
```

## Release

El script `scripts/release.sh` automatiza la creación de todos los paquetes para una nueva versión.

## Licencia

GPL-3.0-or-later
