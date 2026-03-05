Estructura base             
                                                                                                                                                                      
  - testing/.gitignore - Ignora logs, imágenes de VM, artifacts de build
  - testing/logs/.gitkeep - Mantiene el directorio en git
  - testing/Makefile - Orquestador maestro con todos los targets

  Scripts (testing/scripts/)
  ┌───────────────────────────────┬───────────────────────────────────────────────────────────────────┐
  │            Script             │                            Descripción                            │
  ├───────────────────────────────┼───────────────────────────────────────────────────────────────────┤
  │ 00-check-deps.sh              │ Verifica dependencias del host, reporta niveles disponibles       │
  ├───────────────────────────────┼───────────────────────────────────────────────────────────────────┤
  │ 01-build-and-test.sh          │ Compila ambos repos + todos los tests en container Podman         │
  ├───────────────────────────────┼───────────────────────────────────────────────────────────────────┤
  │ 02-test-dbus-integration.sh   │ Tests D-Bus enfocados: daemon real + mock + no-daemon             │
  ├───────────────────────────────┼───────────────────────────────────────────────────────────────────┤
  │ 03-gnome-nested-session.sh    │ Sesión GNOME anidada en host con mock daemon y archivos de prueba │
  ├───────────────────────────────┼───────────────────────────────────────────────────────────────────┤
  │ 04-gnome-desktop-container.sh │ Container GNOME completo con VNC en puerto 5900                   │
  ├───────────────────────────────┼───────────────────────────────────────────────────────────────────┤
  │ container-test-runner.sh      │ Script entrypoint interno del container (7 fases secuenciales)    │
  ├───────────────────────────────┼───────────────────────────────────────────────────────────────────┤
  │ collect-logs.sh               │ Recolecta y genera resumen unificado de todos los logs            │
  └───────────────────────────────┴───────────────────────────────────────────────────────────────────┘
  Containers (testing/containers/)
  ┌─────────────────────────────┬─────────────────────────────────────────────────────────┐
  │        Containerfile        │                       Descripción                       │
  ├─────────────────────────────┼─────────────────────────────────────────────────────────┤
  │ Containerfile.build-test    │ Fedora 41 + systemd + Rust + meson + Python D-Bus + GJS │
  ├─────────────────────────────┼─────────────────────────────────────────────────────────┤
  │ Containerfile.gnome-desktop │ GNOME completo + Xvfb + VNC + mock daemon               │
  └─────────────────────────────┴─────────────────────────────────────────────────────────┘
  VM (testing/vm/)
  ┌──────────────────────┬─────────────────────────────────────────────────────────────────────┐
  │       Archivo        │                             Descripción                             │
  ├──────────────────────┼─────────────────────────────────────────────────────────────────────┤
  │ create-test-vm.sh    │ Descarga Fedora Cloud, crea VM QEMU con cloud-init                  │
  ├──────────────────────┼─────────────────────────────────────────────────────────────────────┤
  │ cloud-init/meta-data │ Metadata de la instancia                                            │
  ├──────────────────────┼─────────────────────────────────────────────────────────────────────┤
  │ cloud-init/user-data │ Provisioning: instala GNOME, compila LNXDrive, configura auto-login │
  └──────────────────────┴─────────────────────────────────────────────────────────────────────┘
  Uso Rápido

  cd /E/Proyectos/Strange Days Tech/lnxdrive-project/testing
  make check-deps          # Verifica qué está disponible
  make build-test          # Compila + tests completos (~10-15 min primera vez)
  make test-dbus           # Solo tests D-Bus
  make gnome-nested        # Ventana GNOME anidada (más rápida)
  make gnome-container     # GNOME + VNC (más robusta)
  make vm-create           # VM completa (máximo aislamiento)
  make logs                # Ver resumen de logs
  make clean               # Limpiar todo
