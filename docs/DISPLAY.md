# Display Architecture

This document describes display control in CrawlDS, including brightness, nightlight, and wallpaper management.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           UI Layer                                          │
│  BrightnessIndicator │ NightlightIndicator │ WallpaperSelector             │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CrawlDSService (QML)                                   │
│  brightnessPercent, nightlightEnabled, wallpaperEvent                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Daemon JSON-RPC Server                                │
│  BrightnessGet/Set │ NightlightEnable/Disable │ WallpaperStatus/Set        │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Backend (crawlds-display)                                │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ brightness.rs: sysfs /sys/class/backlight                            │ │
│  │ nightlight.rs: redshift or wayland-native                            │ │
│  │ wallpaper/: service + backends (swww, dummy)                         │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Backend (`crawlds-display`)

### Modules

| Module | File | Backend | Description |
|--------|------|---------|-------------|
| brightness | `brightness.rs` | sysfs | Backlight brightness control |
| nightlight | `nightlight.rs` | redshift/wayland | Blue light filter |
| wallpaper | `wallpaper/` | swww | Wallpaper management subsystem |

## Brightness Control

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `BrightnessGet` | RPC | Get current brightness status |
| `BrightnessSet` | RPC | Set brightness to specific percent |
| `BrightnessInc` | RPC | Increase brightness by delta |
| `BrightnessDec` | RPC | Decrease brightness by delta |

### Response Format

```json
{
  "status": {
    "device": "intel_backlight",
    "current": 800,
    "max": 1000,
    "percent": 80.0
  }
}
```

### Implementation

Located in `core/crates/crawlds-display/src/brightness.rs`:

- Reads/writes `/sys/class/backlight/<device>/brightness`
- Auto-detects device (prefers intel, then amdgpu, then any)
- Configurable min/max percent bounds
- Emits SSE event on brightness change

## Nightlight Control

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `NightlightStatus` | RPC | Get nightlight status |
| `NightlightEnable` | RPC | Enable nightlight |
| `NightlightDisable` | RPC | Disable nightlight |
| `NightlightSet` | RPC | Set color temperature (K) |

### Response Format

```json
{
  "status": {
    "enabled": true,
    "temperature_k": 4500,
    "available": true
  }
}
```

### Implementation

Located in `core/crates/crawlds-display/src/nightlight.rs`:

- **Primary**: wayland-native protocols (KDE, GNOME, sway)
- **Fallback**: redshift command-line tool
- Temperature range: 1000K (warm) to 10000K (cool)
- Supports smooth transitions

## Wallpaper Management

The wallpaper subsystem follows a clean architecture with pluggable backends.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    WallpaperService (service.rs)                            │
│  - Owns WallpaperState (current, per_monitor)                               │
│  - Orchestrates backends                                                    │
│  - Sends WallpaperEvent on changes                                          │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│               WallpaperBackend (trait)                                      │
│  ┌────────────────┬────────────────┬──────────────┐                        │
│  │   SwwwBackend  │ MpvpaperBackend │  DummyBackend│                        │
│  │  (implemented) │    (future)    │  (fallback)  │                        │
│  └────────────────┴────────────────┴──────────────┘                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Types

| Type | Description |
|------|-------------|
| `WallpaperState` | Current wallpaper path + per-monitor mappings |
| `SetWallpaperRequest` | Path, monitor, mode, transition, duration |
| `WallpaperMode` | Fill, Fit, Stretch, Center, Tile |
| `BackendInfo` | Backend name, availability, daemon status |

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `WallpaperStatus` | RPC | Get current wallpaper state |
| `WallpaperSet` | RPC | Set wallpaper (path, monitor?, transition?) |
| `WallpaperGet` | RPC | Get wallpaper for specific monitor |
| `WallpaperBackends` | RPC | List available backends |

### Request Format

```json
{
  "method": "WallpaperSet",
  "params": {
    "path": "/home/user/wallpapers/arch.jpg",
    "monitor": "eDP-1",
    "transition": "fade"
  }
}
```

### Response Format

```json
{
  "result": { "ok": true }
}
```

### Backend Detection

The service auto-detects the best available backend:

1. Check for `swww` binary
2. Fall back to `DummyBackend` (no-op)

### Module Structure

```
crawlds-display/src/wallpaper/
├── mod.rs           # Module exports + Config
├── models.rs        # SetWallpaperRequest, WallpaperState, WallpaperMode, etc.
├── backend/
│   ├── mod.rs       # WallpaperBackend trait, detect_backend(), list_backends()
│   └── swww.rs      # swww implementation with daemon auto-start
└── service.rs       # WallpaperService, handle_ipc_request_sync()
```

### Design Principles

1. **Backend is dumb** — only executes commands
2. **Service owns state** — tracks current wallpaper per monitor
3. **IPC talks to service** — never directly to backend
4. **Pluggable** — trait allows adding `mpvpaper` or custom renderers later

## Configuration

### `core.toml` Wallpaper Section

```toml
[wallpaper]
swww_bin = "swww"
default_transition = "fade"
transition_duration_ms = 500
```

| Option | Default | Description |
|--------|---------|-------------|
| `swww_bin` | `"swww"` | Path to swww binary |
| `default_transition` | `"fade"` | Default transition type |
| `transition_duration_ms` | `500` | Transition duration in milliseconds |

### Transition Types

swww supports: `fade`, `left`, `right`, `up`, `down`, `center`, `any`, `random`

## Files

| File | Purpose |
|------|---------|
| `crawlds-display/src/lib.rs` | Main module, re-exports |
| `crawlds-display/src/brightness.rs` | Backlight brightness |
| `crawlds-display/src/nightlight.rs` | Blue light filter |
| `crawlds-display/src/wallpaper/` | Wallpaper subsystem |
| `crawlds-display/src/config.rs` | Unified config types |
| `quickshell/Services/UI/WallpaperService.qml` | QML wallpaper service |
| `quickshell/Services/Core/CrawlDSService.qml` | Event handling |

## Roadmap

### High Priority

- [ ] **Backend auto-start**
  - Automatically start swww daemon if not running
  - Detect compositor and use appropriate wallpaper backend

- [ ] **Per-monitor wallpaper profiles**
  - Save different wallpapers per monitor
  - Support monitor hotplugging

- [ ] **Wallpaper rotation scheduler**
  - Time-based wallpaper changes
  - Integration with existing automation system

### Medium Priority

- [ ] **mpvpaper backend**
  - Animated/gif wallpapers support
  - Video wallpaper playback

- [ ] **Custom renderer backend** (future)
  - Wayland-native wallpaper rendering
  - Drop-in replacement for swww

- [ ] **Matugen integration**
  - Auto-generate color scheme on wallpaper change
  - Trigger theming pipeline from wallpaper events

### Lower Priority

- [ ] **Wallpaper caching**
  - Preload wallpapers for faster transitions
  - Background download for remote wallpapers

- [ ] **Wallpaper search**
  - Integration with wallpaper APIs (Wallhaven, etc.)
  - Local wallpaper indexer

- [ ] **DPMS control**
  - Turn screen off on idle
  - Lock screen integration