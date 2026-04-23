# Display Architecture

This document describes display control in CrawlDS, including brightness and nightlight.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           UI Layer                                          │
│  BrightnessIndicator │ NightlightIndicator │ DisplaySettings                │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      BrightnessService / NightlightService                   │
│  BrightnessService: DDC/ddcutil → CrawlDS → /sys/class/backlight          │
│  NightlightService: wayland-native → redshift (fallback)                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CrawlDSService                                           │
│  brightnessDevice, brightnessPercent                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Backend (crawlds-display)                             │
│  brightness.rs: sysfs /sys/class/backlight                                │
│  nightlight.rs: redshift or wayland-native protocols                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Backend (`crawlds-display`)

### Modules

| Module | File | Backend | Description |
|--------|------|---------|-------------|
| brightness | `brightness.rs` | sysfs | Backlight brightness control |
| nightlight | `nightlight.rs` | redshift/wayland | Blue light filter |

### Brightness Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/brightness` | GET | Get current brightness status |
| `/brightness/set` | POST | Set brightness to specific percent |
| `/brightness/inc` | POST | Increase brightness by delta |
| `/brightness/dec` | POST | Decrease brightness by delta |

### Brightness Response Format

```bash
# GET /brightness
Response:
{
  "device": "intel_backlight",
  "current": 800,
  "max": 1000,
  "percent": 80.0
}
```

### Nightlight Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/nightlight/status` | GET | Get nightlight status |
| `/nightlight/enable` | POST | Enable nightlight |
| `/nightlight/disable` | POST | Disable nightlight |
| `/nightlight/temperature` | POST | Set color temperature (K) |

### Nightlight Response Format

```bash
# GET /nightlight/status
Response:
{
  "enabled": true,
  "temperature_k": 4500,
  "available": true
}
```

## Brightness Implementation

Located in `core/crates/crawlds-display/src/brightness.rs`:

- Reads/writes `/sys/class/backlight/<device>/brightness`
- Auto-detects device (prefers intel, then amdgpu, then any)
- Configurable min/max percent bounds
- Reactive: emits SSE event on brightness change

## Nightlight Implementation

Located in `core/crates/crawlds-display/src/nightlight.rs`:

- **Primary**: wayland-native protocols (KDE, GNOME, sway)
- **Fallback**: redshift command-line tool
- Temperature range: 1000K (warm) to 10000K (cool)
- Supports smooth transitions

### Wayland-Native Support (TODO)

| Compositor | Protocol | Interface |
|------------|----------|-----------|
| KDE | KWin | `org.kde.KWin.TempFilter` |
| GNOME | gsd-color | `org.gnome.SettingsDaemon.Color.Temperature` |
| sway | i3msg | custom |

## Files

| File | Purpose |
|------|---------|
| `core/crates/crawlds-display/src/lib.rs` | Main module, combined config |
| `core/crates/crawlds-display/src/brightness.rs` | Backlight brightness |
| `core/crates/crawlds-display/src/nightlight.rs` | Blue light filter |
| `quickshell/services/BrightnessService.qml` | Full brightness control (DDC, fallback) |
| `quickshell/services/NightlightService.qml` | Nightlight service (TODO) |

## Roadmap

### High Priority

- [ ] **Nightlight D-Bus integration**
  - KDE: `org.kde.KWin.TempFilter` interface
  - GNOME: `org.gnome.SettingsDaemon.Color.Temperature`
  - Auto-detect compositor and use appropriate protocol

- [ ] **Smooth transitions**
  - Gradual brightness changes over 500ms-1s
  - Nightlight temperature transitions during schedule changes

- [ ] **Nightlight scheduling**
  - Automatic enable/disable based on time of day
  - Sunset to sunrise calculation based on geolocation
  - Manual override with auto-resume

- [ ] **NightlightService.qml**
  - QML service to complement BrightnessService
  - Integration with CrawlDSService for SSE events

### Medium Priority

- [ ] **Ambient light sensor**
  - Read ALS from `/sys/class/backlight/.../als`
  - Auto-adjust brightness based on ambient light
  - Hysteresis to prevent oscillation

- [ ] **Multi-display support**
  - Handle multiple backlight devices
  - Per-display brightness control
  - Sync or independent brightness modes

- [ ] **Keyboard backlight**
  - Support for keyboard backlight (`/sys/class/leds`)
  - Separate service or merged into display

- [ ] **Per-monitor brightness in backend**
  - Backend could expose DDC/CI control
  - Backend could enumerate all backlight devices
  - UI layer already handles this, but backend could lead

### Lower Priority

- [ ] **DPMS control**
  - Turn screen off on idle
  - Lock screen integration
  - Configurable timeout per power profile

- [ ] **Color profile management**
  - ICC profile switching
  - Per-app color profiles
  - Automatic switching based on content

- [ ] **Gamma correction**
  - Manual RGB adjustment
  - Color blindness modes
  - High contrast modes

- [ ] **Backend refactor for config**
  - Currently uses `crawlds_display::Config` in daemon config
  - Need to align with `brightness.rs` module structure
  - Nightlight config not yet wired in daemon

### Nice to Have

- [ ] **Nightlight sunrise mode**
  - Gradual temperature shift in morning
  - Mimics natural light cycle

- [ ] **App-specific rules**
  - Auto-dim for certain apps
  - Nightlight activation for specific apps

- [ ] **OSD notifications**
  - Show brightness/nightlight changes
  - Customizable OSD style

- [ ] **Temperature-aware brightness curves**
  - Auto-dim at night (reduce blue light)
  - Outdoor mode: boost brightness and warmth

- [ ] **Nightlight automatic temperature**
  - Based on geolocation, calculate sunset/sunrise times
  - Smooth transitions between day/night temperatures
