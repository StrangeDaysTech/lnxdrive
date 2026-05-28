# LNXDrive GTK3

Cliente GTK3 unificado de LNXDrive para escritorios ligeros.

## Descripción

Este repositorio proporciona una interfaz GTK3 que funciona en múltiples entornos de escritorio:

- **XFCE**: Integración con Thunar
- **Cinnamon**: Integración con Nemo
- **MATE**: Integración con Caja

## Características

- **System tray**: Indicador AppIndicator/StatusNotifier
- **Diálogo de preferencias**: Configuración completa en GTK3
- **Menús contextuales**: Integración con gestores de archivos

## Requisitos

- GTK3 3.24+
- libappindicator3
- Rust 1.75+
- lnxdrive-daemon en ejecución

## Compilación

```bash
cargo build --release
```

## Instalación

```bash
cargo install --path .
```

## Estructura

```
lnxdrive-gtk3/
├── src/                  # Código fuente Rust
└── data/
    ├── icons/           # Iconos de la aplicación
    └── ui/              # Archivos .ui (Glade)
```

## Comunicación con el daemon

La aplicación se comunica con `lnxdrive-daemon` a través de D-Bus usando la librería `lnxdrive-ipc`.

## Licencia

GPL-3.0-or-later
