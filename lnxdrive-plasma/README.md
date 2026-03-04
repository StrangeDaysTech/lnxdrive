# LNXDrive Plasma

Integración nativa de LNXDrive para KDE Plasma.

## Descripción

Este repositorio proporciona la experiencia de usuario completa para KDE Plasma, incluyendo:

- **Aplicación principal**: Interfaz Qt6/QML para configuración
- **Plugin Dolphin**: Overlay icons y menús contextuales
- **Plasmoid**: Widget para el panel de Plasma

## Requisitos

- KDE Plasma 5.27+ / 6.0+
- Qt 6.6+
- KDE Frameworks 6.0+
- CMake 3.16+
- lnxdrive-daemon en ejecución

## Compilación

```bash
mkdir build && cd build
cmake .. -DCMAKE_INSTALL_PREFIX=/usr
make
sudo make install
```

## Estructura

```
lnxdrive-plasma/
├── src/              # Aplicación principal
├── qml/              # Componentes QML
├── dolphin-plugin/   # Plugin para Dolphin
└── plasmoid/         # Widget de panel
```

## Comunicación con el daemon

La aplicación se comunica con `lnxdrive-daemon` directamente a través de D-Bus usando Qt6 DBus.

## Licencia

GPL-3.0-or-later
