# GNOME Online Accounts provider for LNXDrive
# Implements FR-019 through FR-023
#
# Enable with: meson setup builddir -Denable_goa=true

goa_dep = dependency('goa-1.0', version: '>= 3.48')
goa_backend_dep = dependency('goa-backend-1.0', version: '>= 3.48')
gio_dep = dependency('gio-2.0', version: '>= 2.76')

# WebKitGTK naming varies between distros
webkit_dep = dependency('webkitgtk-6.0', required: false)
if not webkit_dep.found()
  webkit_dep = dependency('webkit2gtk-6.0')
endif

# GOA provider module install directory (from pkg-config)
goa_provider_dir = goa_backend_dep.get_variable(
  pkgconfig: 'goabackenddir',
  default_value: get_option('libdir') / 'goa-1.0' / 'web-extensions',
)

shared_library('lnxdrive-goa-provider',
  files(
    'src/lnxdrive-goa-module.c',
    'src/lnxdrive-goa-provider.c',
    'src/lnxdrive-goa-dbus.c',
  ),
  dependencies: [goa_dep, goa_backend_dep, webkit_dep, gio_dep],
  c_args: [
    '-DGOA_API_IS_SUBJECT_TO_CHANGE',
    '-DGOA_BACKEND_API_IS_SUBJECT_TO_CHANGE',
    '-DGETTEXT_PACKAGE="lnxdrive-gnome"',
  ],
  install: true,
  install_dir: goa_provider_dir,
)
