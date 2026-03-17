# GNOME Online Accounts Provider — LNXDrive

**Status**: Implemented (S5)

## Architecture

The GOA provider is a C shared library implementing the `GoaOAuth2Provider` GObject
subclass, registered with GNOME Online Accounts to provide native Microsoft account
integration in GNOME Settings → Online Accounts.

### Components

| Component | Description |
|-----------|-------------|
| `lnxdrive-goa-module.c` | GIO module entry point — registers the provider type |
| `lnxdrive-goa-provider.c` | `GoaOAuth2Provider` subclass with Microsoft OAuth2 endpoints |
| `lnxdrive-goa-dbus.c` | D-Bus helper — hands off GOA tokens to the daemon |
| Shell extension GOA monitor | Watches `AccountRemoved` signals and calls `Auth.Logout()` |

### Functional Requirements Coverage

| FR | Description | Status |
|----|-------------|--------|
| FR-019 | Provider registration in GNOME Online Accounts | Implemented |
| FR-020 | OAuth2 authentication via GOA | Implemented |
| FR-021 | SSO — reuse existing Microsoft accounts from GOA | Implemented |
| FR-022 | Automatic token refresh via GOA infrastructure | Implemented |
| FR-023 | Account removal propagation to daemon | Implemented |

### D-Bus Integration

The provider uses `Auth.CompleteAuthWithTokens(access_token, refresh_token, expires_at_unix)`
to pass GOA-obtained tokens to the daemon. This method was added specifically for GOA
integration, separate from the existing `CompleteAuth(code, state)` which expects an
authorization code.

Token refresh for GOA-sourced auth is delegated back to GOA via
`org.gnome.OnlineAccounts.OAuth2Based.GetAccessToken()`.

### Dependencies

- `gnome-online-accounts` (libgoa-1.0, libgoa-backend-1.0) >= 3.48
- `webkitgtk-6.0` or `webkit2gtk-6.0` (GOA OAuth2 embedded view)
- `gio-2.0` >= 2.76
- LNXDrive daemon D-Bus interface `com.strangedaystech.LNXDrive.Auth`

### Build

This component is gated behind the `enable_goa` Meson option (default: false):

```bash
meson setup builddir -Denable_goa=true
ninja -C builddir
```

The shared library installs to the GOA provider module directory
(typically `/usr/lib64/goa-1.0/web-extensions/`).

### Testing

```bash
# Load the provider in a test environment
GOA_PROVIDER_MODULE_DIR=./builddir/lnxdrive-gnome/goa-provider gnome-control-center online-accounts
```

---

*LNXDrive GNOME Integration — [Strange Days Tech](https://strangedays.tech)*
