# System Information (`crawlds-system`)

This document describes the system information aggregator for CrawlDS.

## Overview

`crawlds-system` provides a **single source of truth** for read-only system state including:

- Compositor detection and capabilities
- Operating system information
- Session information
- Hardware information
- Display/monitor information

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SystemService                                      │
│  Provides cached SystemInfo snapshot on demand                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                    ┌────────────────┼────────────────┐
                    ▼                ▼                ▼
            ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
            │  compositor  │ │      os       │ │   hardware   │
            │  detection   │ │  /proc/...    │ │  /proc/cpuinfo│
            └──────────────┘ └──────────────┘ └──────────────┘
```

## Modules

### `compositor.rs` — Compositor Detection

Detects the current Wayland compositor via environment variables.

**Detection Order:**
1. `HYPRLAND_INSTANCE_SIGNATURE` → Hyprland
2. `SWAYSOCK` → Sway (or Scroll if `XDG_CURRENT_DESKTOP` contains "scroll")
3. `NIRI_SOCKET` → Niri
4. `LABWC_PID` → Labwc
5. `XDG_CURRENT_DESKTOP` contains "mango" → Mango
6. Fallback → Unknown

**Compositor Capabilities:**

| Capability | Hyprland | Sway | Niri | Mango | Labwc | Scroll |
|------------|----------|------|------|-------|-------|--------|
| `layer_shell` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `blur` | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| `screencopy` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `wallpaper_control` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `dpms` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `socket_ipc` | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `http_ipc` | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |

### `os.rs` — Operating System Info

Reads from:
- `/etc/os-release` — OS name, ID, pretty name
- `/proc/version` — Kernel version
- `/etc/hostname` — System hostname

### `session.rs` — Session Information

Detects session type:
- `XDG_SESSION_TYPE` environment variable
- Fallback to `WAYLAND_DISPLAY`, `DISPLAY` vars

### `hardware.rs` — Hardware Info

Reads from:
- `/proc/cpuinfo` — CPU model, core count
- `/proc/meminfo` — Total memory
- `lspci -d ::0300` — GPU info

### `display.rs` — Monitor Information

Reads from:
- `/sys/class/drm` — Monitor status, dimensions, position
- `wlr-randr --json` — Display scales (when available)

## Models

### `SystemInfo` (root)

```rust
pub struct SystemInfo {
    pub compositor: CompositorInfo,
    pub os: OsInfo,
    pub session: SessionInfo,
    pub hardware: HardwareInfo,
    pub display: DisplayInfo,
}
```

### `CompositorInfo`

```rust
pub struct CompositorInfo {
    pub compositor_type: CompositorType,
    pub name: String,
    pub capabilities: CompositorCapabilities,
}

pub enum CompositorType {
    Hyprland, Sway, Niri, Mango, Labwc, Scroll, Unknown,
}
```

### `CompositorCapabilities`

```rust
pub struct CompositorCapabilities {
    pub layer_shell: bool,        // Required for panels/wallpapers
    pub blur: bool,                // Blur effects
    pub screencopy: bool,          // Screenshot support
    pub wallpaper_control: bool,   // swww/hyprpaper compatible
    pub dpms: bool,               // Monitor power control
    pub socket_ipc: bool,         // Uses Unix socket IPC
    pub http_ipc: bool,            // Uses HTTP IPC
}
```

### `OsInfo`

```rust
pub struct OsInfo {
    pub name: String,        // e.g., "Arch Linux"
    pub kernel: String,      // e.g., "6.8.1-arch1"
    pub pretty_name: String, // e.g., "Arch Linux"
    pub hostname: String,    // e.g., "hostname"
    pub id: String,          // e.g., "arch"
}
```

### `SessionInfo`

```rust
pub struct SessionInfo {
    pub session_type: SessionType,  // Wayland, X11, Tty
    pub user: String,
    pub seat: Option<String>,
    pub home: String,
}
```

### `HardwareInfo`

```rust
pub struct HardwareInfo {
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub memory_total: u64,  // bytes
    pub gpu: Option<String>,
}
```

### `DisplayInfo`

```rust
pub struct DisplayInfo {
    pub monitors: Vec<MonitorInfo>,
    pub scales: HashMap<String, f32>,  // name → scale
}

pub struct MonitorInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale: f32,
    pub refresh_rate: f32,
    pub focused: bool,
    pub active: bool,
}
```

## IPC Commands

### `SystemInfo`

Returns full system snapshot.

**Request:**
```json
{ "method": "SystemInfo" }
```

**Response:**
```json
{
  "result": {
    "compositor": {
      "type": "hyprland",
      "name": "hyprland",
      "capabilities": {
        "layer_shell": true,
        "blur": true,
        "wallpaper_control": true,
        "dpms": true,
        "socket_ipc": false,
        "http_ipc": true
      }
    },
    "os": {
      "name": "Arch Linux",
      "kernel": "6.8.1-arch1",
      "pretty_name": "Arch Linux",
      "hostname": "hostname",
      "id": "arch"
    },
    "session": {
      "type": "wayland",
      "user": "username",
      "seat": "Seat0",
      "home": "/home/username"
    },
    "hardware": {
      "cpu_model": "Intel i7-12700K",
      "cpu_cores": 12,
      "memory_total": 33554432000,
      "gpu": "NVIDIA GeForce RTX 3080"
    },
    "display": {
      "monitors": [
        {
          "name": "DP-1",
          "width": 2560,
          "height": 1440,
          "scale": 1.0
        }
      ],
      "scales": {
        "DP-1": 1.0
      }
    }
  }
}
```

### `CompositorCapabilities`

Returns just compositor capabilities.

**Request:**
```json
{ "method": "CompositorCapabilities" }
```

**Response:**
```json
{
  "result": {
    "layer_shell": true,
    "blur": true,
    "wallpaper_control": true,
    "dpms": true,
    "socket_ipc": false,
    "http_ipc": true
  }
}
```

## QML Integration

```qml
// CrawlDSService properties (set on bootstrap)
property string compositorName
property bool supportsWallpaperControl
property bool supportsBlur
property bool supportsLayerShell

// Check capabilities before showing features
if (CrawlDSService.supportsBlur) {
    showBlurSettings()
}

if (CrawlDSService.supportsWallpaperControl) {
    showWallpaperPanel()
}
```

## Service Interface

```rust
use crawlds_system::SystemService;

// Create service (collects snapshot on creation)
let system = SystemService::new();

// Query data
let info = system.get_info();
let compositor = system.compositor();
let capabilities = &compositor.capabilities;

// Refresh (rarely needed)
system.refresh();
```

## Design Principles

1. **Read-only** — Never modifies system state
2. **Snapshot-based** — Cached on creation, refreshed on demand
3. **Single source of truth** — Other crates query through this service
4. **No polling** — Static data never changes; display info updates on hotplug

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Module exports |
| `src/models.rs` | All shared types |
| `src/compositor.rs` | Compositor detection + capabilities |
| `src/os.rs` | OS/kernel/hostname |
| `src/session.rs` | Session type detection |
| `src/hardware.rs` | CPU/memory/GPU info |
| `src/display.rs` | Monitor/scales info |
| `src/service.rs` | SystemService entry point |

## Roadmap

### High Priority
- [ ] Real-time display events (monitor hotplug)
- [ ] GPU detection improvements

### Medium Priority
- [ ] Battery info for laptops
- [ ] Temperature sensors (if available)

### Lower Priority
- [ ] Subscription-based updates for QML
- [ ] Integration with sysmon for runtime metrics