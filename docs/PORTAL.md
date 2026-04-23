# XDG Desktop Portal Backend - CrawlDS

## Overview

`crawlds-portal` is a custom XDG Desktop Portal backend that provides a unified interface for sandboxed applications (Flatpak, Snap, etc.) to interact with the system. Unlike existing backends that tie to specific toolkits (GTK, Qt, GNOME), `crawlds-portal` is designed to work with CrawlDS's native component architecture.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  xdg-desktop-portal (frontend daemon)                │
│  Bus name: org.freedesktop.portal.Desktop           │
│  Handles: permissions, document store, lifecycle │
└─────────────────────────────────────────────────────────┘
                          ↕ D-Bus
┌─────────────────────────────────────────────────────────┐
│  crawlds-portal (backend)                          │
│  Bus name: org.freedesktop.impl.portal.desktop  │
│  Implements: org.freedesktop.impl.portal.*     │
└─────────────────────────────────────────────────────────┘
```

### Key Components

1. **Portal File** (`/usr/share/xdg-desktop-portal/portals/crawlds.portal`)
2. **Configuration** (`/etc/xdg-desktop-portal/crawlds-portals.conf`)
3. **D-Bus Service** - Implements backend interfaces

---

## Portal Interfaces

### Core Interfaces (High Priority)

| Interface | Description | Methods | CrawlDS Integration |
|-----------|-------------|---------|-------------------|
| `FileChooser` | File open/save dialogs | OpenFile, SaveFile, SaveFiles | `crawlds-vfs` |
| `OpenURI` | Open URLs in browser | OpenURI | External handler |
| `Print` | Print dialogs | PreparePrint, Print | CUPS/spooler |
| `Notification` | System notifications | AddNotification, RemoveNotification | `crawlds-notify` |

### Desktop Integration (Medium Priority)

| Interface | Description | Methods |
|-----------|------------|---------|
| `Wallpaper` | Set desktop background | SetWallpaperURI |
| `Screenshot` | Take screenshots | Screenshot |
| `ScreenCast` | Screen/window recording | CreateSession, SelectSources, Start |
| `Background` | Run in background | Background |

### Access & Permissions

| Interface | Description |
|-----------|------------|
| `Account` | User info (name, avatar) |
| `Clipboard` | Clipboard access |
| `Settings` | dconf/registry access |
| `Lockdown` | Restrict features |

### Advanced Features (Lower Priority)

| Interface | Description |
|-----------|------------|
| `RemoteDesktop` | Remote desktop sessions |
| `Camera` | Camera access |
| `Email` | Email client integration |
| `DynamicLauncher` | Add to application menu |
| `Inhibit` | Lock screen/suspend |
| `Location` | GPS/location access |
| `GlobalShortcuts` | Register shortcuts |

---

## Implementation Details

### D-Bus Service Registration

The backend must register with D-Bus using the well-known name:
```
org.freedesktop.impl.portal.desktop
```

### Portal File Format

```ini
[portal]
DBusName=org.freedesktop.impl.portal.desktop
Interfaces=org.freedesktop.impl.portal.FileChooser;org.freedesktop.impl.portal.Notification;org.freedesktop.impl.portal.Print;org.freedesktop.impl.portal.Wallpaper
UseIn=crawlds
```

### Configuration File

```ini
[preferred]
default=crawlds
org.freedesktop.impl.portal.FileChooser=crawlds
org.freedesktop.impl.portal.Notification=crawlds
org.freedesktop.impl.portal.Print=crawlds
org.freedesktop.impl.portal.Wallpaper=crawlds
```

---

## Common Conventions

### Request Pattern

All portal methods follow this pattern:

```rust
fn method_name(
    handle: ObjectPath,      // Request object handle
    app_id: String,        // Application ID
    // ... other params
    options: Dict,         // Additional options
) -> (response: u32, results: Dict);
```

Response codes:
- `0` - Success
- `1` - Cancelled
- `2` - Error

### Session Pattern

Long-running operations use sessions:

```rust
fn create_session(handle, session_handle, app_id, options) -> (response, results)
// Returns session_id in results
```

### Window Identifiers

For positioning dialogs relative to app windows. Format:
```
"x11:XXXX" for X11
"wayland:APP-ID:window-id" for Wayland
```

---

## Integration with CrawlDS Components

### Existing Components to Leverage

```
crawlds-vfs      → FileChooser, OpenURI
crawlds-notify    → Notification
crawlds-clipboard → Clipboard
crawlds-power    → Inhibit (lock/suspend)
```

### New Components Needed

1. **File Chooser Dialog** - QML-based file picker
2. **Print Dialog** - QML-based print settings
3. **Screenshot Handler** - Using grim/ASCII or similar
4. **Screen Cast** - PipeWire integration

---

## Roadmap

### Phase 1: Core Portals (MVP)

- [ ] Set up D-Bus service structure
- [ ] FileChooser portal (open/save files)
- [ ] OpenURI portal (open URLs)
- [ ] Notification portal
- [ ] Portal file installation
- [ ] Configuration

### Phase 2: Desktop Integration

- [ ] Wallpaper portal
- [ ] Screenshot portal
- [ ] Print portal
- [ ] Background portal

### Phase 3: Advanced Features

- [ ] ScreenCast portal (screen recording)
- [ ] Clipboard portal
- [ ] Account portal
- [ ] Settings portal

### Phase 4: Additional Portals

- [ ] RemoteDesktop portal
- [ ] Camera portal
- [ ] Location portal
- [ ] Inhibit portal

---

## Technical Considerations

### Dependencies

```toml
[dependencies]
zbus = "5"           # D-Bus bindings
zvariant = "5"         # D-Bus type serialization
tokio = "1"           # Async runtime
serde = "1"            # Serialization
thiserror = "1"          # Error handling
tracing = "0.1"        # Logging

[build-dependencies]
zbus-build = "5"        # D-Bus code generation
```

### Security

1. **File Access Validation** - Only allow file:// URIs or FDs
2. **Window Tracking** - Track parent windows for dialogs
3. **Permission Store** - Use xdg-desktop-portal's permission store
4. **App ID Verification** - Verify caller identity

### Testing

- Use `bush` or similar for D-Bus testing
- Test with Flatpak applications
- Test against xdg-desktop-portal frontend

---

## Reference

- [XDG Desktop Portal Specification](https://flatpak.github.io/xdg-desktop-portal/)
- [ashpd](https://github.com/bilelmoussaoui/ashpd) - Rust wrapper library
- [xdg-desktop-portal-gtk4](https://github.com/mahkoh/xdg-desktop-portal-gtk4) - GTK4 reference implementation

### Related Documentation

- [VFS](FILESYSTEM.md) - Virtual filesystem
- [Notification](NOTIFICATION.md) - Notification system
- [Clipboard](CLIPBOARD.md) - Clipboard management
- [Power](POWER.md) - Power management

---

## Implementation Status

| Portal | Status | Notes |
|--------|--------|-------|
| FileChooser | Not started | Requires QML file dialog |
| OpenURI | Not started | External browser handler |
| Notification | Not started | Leverage crawlds-notify |
| Print | Not started | CUPS integration |
| Wallpaper | Not started | Use crawlds-vfs |
| Screenshot | Not started | grim/ASCII integration |
| ScreenCast | Not started | PipeWire integration |
| Clipboard | Not started | Leverage crawlds-clipboard |