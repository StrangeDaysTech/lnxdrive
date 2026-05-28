# LNXDrive COSMIC

Integración nativa de LNXDrive para COSMIC Desktop (System76).

## Descripción

Este repositorio proporciona la experiencia de usuario para COSMIC Desktop, el nuevo escritorio de System76 escrito en Rust.

- **100% Rust**: Comparte código con el daemon
- **UI moderna**: Construida con iced + libcosmic
- **Integración nativa**: Diseñada específicamente para COSMIC

## Requisitos

- COSMIC Desktop
- Rust 1.75+
- libcosmic
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
lnxdrive-cosmic/
├── src/              # Código fuente Rust
└── assets/           # Recursos (iconos, etc.)
```

## Comunicación con el daemon

La aplicación se comunica con `lnxdrive-daemon` a través de D-Bus usando la librería `lnxdrive-ipc`, que puede compartir código con el daemon al ser ambos proyectos Rust.

## Licencia

GPL-3.0-or-later
