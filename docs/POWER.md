# Power & Battery Architecture

This document describes how power and battery management is implemented in CrawlDS.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           UI Layer                                       │
│  BatteryWidget (bar widget)                                              │
│  BatteryPopout                                                       │
│  BatterySettings, PowerProfileSettings                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
               ┌──────────────────────┴──────────────────────┐
               ▼                                               ▼
┌─────────────────────────────┐               ┌─────────────────────────────┐
│    BatteryService        │               │   PowerProfileService     │
│ (native UPower D-Bus)    │               │  (client-side only)        │
│                         │               │                        │
│ - Multiple batteries     │               │  - Profile cycling     │
│ - Bluetooth batteries    │               │                        │
│ - Notifications         │               │                       │
└─────────────────────────────┘               └─────────────────────────────┘
               │                                               │
               ▼                                               ▼
┌─────────────────────────────┐               ┌─────────────────────────────┐
│    PowerService            │               │      (none)            │
│ (via CrawlDSService)     │               │                        │
│                         │               │  Posts to /power/prof │
│ - Simple state wrapper   │               │  but endpoint exists   │
│ - Notifications          │               │                       │
└─────────────────────────────┘               └─────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CrawlDSService                                       │
│  - batteryPercent, batteryState, batteryOnAc                                  │
│  - batteryTimeToEmpty, batteryTimeToFull                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CrawlDS Backend                                      │
│  - crawlds-power: UPower D-Bus → battery status                     │
│  - crawlds-power/idle.rs: ScreenSaver D-Bus → idle detection         │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │  org.freedesktop    │
                         │  UPower (D-Bus)     │
                         └─────────────────────┘
```

## Backend (crawlds-power)

### Battery Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/power/battery` | GET | Get battery status |
| `/power/profile` | GET | Get current power profile |
| `/power/profile` | POST | Set power profile (0=balanced, 1=power-saver, 2=performance) |

### Battery Request/Response Formats

```bash
# GET /power/battery
Response:
{
  "percent": 75.0,
  "state": "charging",      // "charging" | "discharging" | "fully_charged" | "empty" | "unknown"
  "time_to_empty_secs": null,
  "time_to_full_secs": 1800,
  "energy_rate_w": 15.5,
  "voltage_v": 12.1,
  "temperature_c": 35.2,
  "on_ac": true
}
```

### Battery Implementation

Located in `core/crates/crawlds-power/src/lib.rs`:

```rust
pub struct Config {
    pub low_battery_threshold: f64,
    pub critical_threshold: f64,
}

pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) {
    // 1. Connect to UPower D-Bus
    let conn = Connection::system().await?;
    let upower = UPowerProxy::new(&conn).await?;

    // 2. Find primary battery device
    let devices = upower.enumerate_devices().await?;
    let battery_path = find_battery(&conn, &devices).await;

    // 3. Poll every 30 seconds
    loop {
        let status = read_battery_status(&bat_proxy, &upower).await;
        tx.send(CrawlEvent::Power(PowerEvent::BatteryUpdate { status }));

        // Emit LowBattery/Critical events
        if pct <= cfg.critical_threshold {
            tx.send(CrawlEvent::Power(PowerEvent::Critical { percent: pct }));
        } else if pct <= cfg.low_battery_threshold {
            tx.send(CrawlEvent::Power(PowerEvent::LowBattery { percent: pct }));
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

pub async fn get_battery() -> Result<BatteryStatus, PowerError>;
```

### Battery SSE Events

```json
{
  "domain": "power",
  "data": {
    "event": "battery_update",
    "status": {
      "percent": 75.0,
      "state": "charging",
      "on_ac": true
    }
  }
}

{
  "domain": "power",
  "data": {
    "event": "low_battery",
    "percent": 15
  }
}

{
  "domain": "power",
  "data": {
    "event": "critical_battery",
    "percent": 5
  }
}

{
  "domain": "power",
  "data": {
    "event": "ac_connected"
  }
}

{
  "domain": "power",
  "data": {
    "event": "ac_disconnected"
  }
}
```

### Battery Types

```rust
pub enum BatteryState {
    Charging,
    Discharging,
    FullyCharged,
    Empty,
    Unknown,
}

pub struct BatteryStatus {
    pub percent: f64,
    pub state: BatteryState,
    pub time_to_empty_secs: Option<i64>,
    pub time_to_full_secs: Option<i64>,
    pub energy_rate_w: Option<f64>,
    pub voltage_v: Option<f64>,
    pub temperature_c: Option<f64>,
    pub on_ac: bool,
}
```

## Idle Detection

Idle detection is configured in the core config file (`~/.config/crawlds/core.toml`) under the `[idle]` section.

### Configuration

```toml
[idle]
idle_timeout_secs = 300        # seconds before considered idle
dim_timeout_secs = 60          # seconds before dimming (0 = disabled)
sleep_timeout_secs = 600        # seconds before sleeping (0 = disabled)
screen_off_timeout_secs = 600   # seconds before screen off (0 = disabled)
lock_timeout_secs = 660          # seconds before lock (0 = disabled)
suspend_timeout_secs = 1800     # seconds before suspend (0 = disabled)
fade_duration_secs = 5          # seconds of fade-to-black before action

# Commands to execute when thresholds are reached
screen_off_command = ""        # e.g., "loginctl lock-session"
lock_command = ""               # e.g., "loginctl lock-session"
suspend_command = ""             # e.g., "systemctl suspend"
```

### Idle Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/idle/status` | GET | Get current idle time and state |
| `/idle/activity` | POST | Simulate user activity |
| `/idle/inhibit` | POST | Prevent idle (with reason) |
| `/idle/uninhibit` | DELETE | Allow idle again |

### Idle Implementation

Located in `core/crates/crawlds-power/src/idle.rs`:

```rust
pub struct Config {
    pub idle_timeout_secs: u64,          // Time before considered idle
    pub dim_timeout_secs: u64,           // Time before dimming (0 = disabled)
    pub sleep_timeout_secs: u64,          // Time before sleeping (0 = disabled)
    pub screen_off_timeout_secs: u64,     // Time before screen off (0 = disabled)
    pub lock_timeout_secs: u64,           // Time before lock (0 = disabled)
    pub suspend_timeout_secs: u64,        // Time before suspend (0 = disabled)
    pub fade_duration_secs: u64,          // Fade overlay duration
    pub screen_off_command: String,       // Command to run on screen off
    pub lock_command: String,             // Command to run on lock
    pub suspend_command: String,          // Command to run on suspend
}

pub async fn run_idle(cfg: Config, tx: broadcast::Sender<CrawlEvent>) {
    // Use org.freedesktop.ScreenSaver for idle detection
    let conn = Connection::session().await?;
    let ss = ScreenSaverProxy::new(&conn).await?;

    loop {
        let idle_time = ss.get_session_idle_time().await?;

        // Calculate state and pending actions
        let (state, pending_action) = calculate_state(idle_time, &cfg, ...);

        // Emit events based on thresholds
        tx.send(CrawlEvent::Idle(IdleEvent {
            event: event_name,
            idle_time_secs: idle_time,
            pending_action: pending_action.clone(),
        }));

        // If action pending, emit separate event for fade overlay
        if let Some(action) = pending_action {
            tx.send(CrawlEvent::Idle(IdleEvent {
                event: "action_pending",
                idle_time_secs: idle_time,
                pending_action: Some(action),
            }));
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

### Idle SSE Events

```json
// State changes
{ "domain": "idle", "data": { "event": "idle_detected", "idle_time_secs": 300 } }
{ "domain": "idle", "data": { "event": "resumed", "idle_time_secs": 0 } }
{ "domain": "idle", "data": { "event": "dimming", "idle_time_secs": 60 } }
{ "domain": "idle", "data": { "event": "sleeping", "idle_time_secs": 600 } }

// Action pending (for fade overlay)
{ "domain": "idle", "data": { "event": "action_pending", "idle_time_secs": 600, "pending_action": "screen_off" } }
{ "domain": "idle", "data": { "event": "action_pending", "idle_time_secs": 660, "pending_action": "lock" } }
{ "domain": "idle", "data": { "event": "action_pending", "idle_time_secs": 1800, "pending_action": "suspend" } }

// Action executed
{ "domain": "idle", "data": { "event": "screen_off", "idle_time_secs": 605 } }
{ "domain": "idle", "data": { "event": "locked", "idle_time_secs": 665 } }
{ "domain": "idle", "data": { "event": "suspended", "idle_time_secs": 1805 } }
```

### Idle Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IdleState {
    Active,      // User is active
    Idle,        // Idle timeout reached
    Dimming,     // Dim timeout reached
    Sleeping,    // Sleep timeout reached
    ScreenOff,   // Screen off action executed
    Locked,      // Lock action executed
    Suspended,   // Suspend action executed
}

pub struct IdleStatus {
    pub idle_time_secs: u64,
    pub state: IdleState,
    pub inhibited: bool,
    pub pending_action: Option<String>,  // "screen_off", "lock", "suspend"
}

pub struct IdleEvent {
    pub event: String,
    pub idle_time_secs: u64,
    pub pending_action: Option<String>,
}
```

---

## CrawlDSService

Located in `quickshell/services/core/CrawlDSService.qml`:

### Properties

```qml
// ── Battery ─────────────────────────────────────────────────────────
property real   batteryPercent:    100
property string batteryState:      "unknown"    // charging | discharging | full | empty
property bool   batteryOnAc:       true
property int    batteryTimeToEmpty: 0            // seconds
property int    batteryTimeToFull:  0            // seconds
```

### Event Handling

```qml
function _handleBattery(data) {
    root.batteryPercent    = data.percent         ?? root.batteryPercent
    root.batteryState      = data.state           ?? root.batteryState
    root.batteryOnAc       = data.on_ac           ?? root.batteryOnAc
    root.batteryTimeToEmpty = data.time_to_empty_secs ?? 0
    root.batteryTimeToFull = data.time_to_full_secs  ?? 0
}

function _handlePowerEvent(data) {
    switch (data.event) {
    case "battery_update": _handleBattery(data.status); break
    case "ac_connected":   root.batteryOnAc = true;  break
    case "ac_disconnected":root.batteryOnAc = false; break
    }
}
```

---

## BatteryService

Located in `quickshell/services/BatteryService.qml`:

### Architecture

BatteryService uses **native UPower** (not via CrawlDS):

```
BatteryService → Quickshell.UPower (D-Bus) → org.freedesktop.UPower
```

### Properties

```qml
readonly property var primaryDevice: _laptopBattery || _bluetoothBattery || null
readonly property real batteryPercentage: getPercentage(primaryDevice)
readonly property bool batteryCharging: isCharging(primaryDevice)
readonly property bool batteryPluggedIn: isPluggedIn(primaryDevice)
readonly property bool batteryReady: isDeviceReady(primaryDevice)
readonly property bool batteryPresent: isDevicePresent(primaryDevice)

readonly property var laptopBatteries: UPower.devices.values.filter(...)
readonly property var bluetoothBatteries: BluetoothService.devices filter

property var deviceModel: [...]  // For device selection
```

### Device Priority

1. **Laptop battery** (via UPower)
2. **Bluetooth battery** (from connected BT devices)

### Functions

```qml
function getPercentage(device)
function isCharging(device)
function isPluggedIn(device)
function isDeviceReady(device)
function isDevicePresent(device)
function isLowBattery(device)
function isCriticalBattery(device)
function getIcon(percent, charging, pluggedIn, isReady)
function getRateText(device)
function getTimeRemainingText(device)
function getDeviceName(device)
```

### Battery Notifications

```qml
function checkDevice(device) {
    if (percentage > warningThreshold) { ... }
    if (percentage > criticalThreshold) { ... }
    if (level) { notify(device, level) }
}

function notify(device, level) {
    ToastService.showNotice(title, desc, icon, 6000)
}
```

---

## PowerService

Located in `quickshell/services/PowerService.qml`:

### Architecture

PowerService uses **CrawlDSService** (not native UPower):

```
PowerService → CrawlDSService → backend
```

### Properties

```qml
readonly property real batteryPercent: CrawlDSService.batteryPercent
readonly property string batteryState: CrawlDSService.batteryState
readonly property bool batteryOnAc: CrawlDSService.batteryOnAc
readonly property int batteryTimeToEmpty: CrawlDSService.batteryTimeToEmpty
readonly property int batteryTimeToFull: CrawlDSService.batteryTimeToFull

readonly property bool batteryCharging: batteryState === "charging"
readonly property bool batteryFull: batteryState === "full"
readonly property bool batteryDischarging: batteryState === "discharging"
readonly property bool batteryEmpty: batteryState === "empty"
```

### Functions

```qml
function isLowBattery()
function isCriticalBattery()
function getIcon()
function getChargingIcon()
function batteryReady()
function getTimeRemainingText()
```

---

## IdleService

Located in `quickshell/services/core/IdleService.qml`:

IdleService now handles both idle detection and inhibition. Configuration is in the core config file, not shell settings.

### Properties

```qml
// Read-only state (from daemon)
readonly property int idleTime: _lastEvent?.idle_time_secs ?? 0
readonly property int idleSeconds: idleTime
readonly property string state: _calculateState()  // "active" | "idle" | "dimming" | "sleeping"

// Fade overlay (triggered by action_pending events)
property string fadePending: ""     // "screen_off" | "lock" | "suspend"
readonly property int fadeDuration: 5  // from daemon config

// Inhibition (prevent idle/sleep)
property bool inhibited: false      // True when apps prevent idle
property bool isInhibited: activeInhibitors.length > 0
property var activeInhibitors: []    // List of active inhibitor IDs

// Native Wayland inhibitor availability
property bool nativeInhibitorAvailable: false
```

### Signals

```qml
signal idleDetected()
signal resumed()
signal dimming()
signal sleeping()
signal actionPending(string action)    // Fade overlay should show
signal actionExecuting(string action)   // Action is being executed
```

### Functions

```qml
// Inhibition
function addInhibitor(id, reason)      // Add app-specific inhibitor
function removeInhibitor(id)            // Remove inhibitor
function inhibit(reason)               // Prevent idle (calls daemon)
function uninhibit()                   // Allow idle (calls daemon)

// Manual user control
function manualToggle()                // Toggle manual keep-awake
function changeTimeout(delta)          // Adjust timeout
function addManualInhibitor(timeoutSec) // Enable with timeout
function removeManualInhibitor()       // Disable

// Activity
function resetIdleTime()               // Simulate user activity
```

### Configuration

Idle settings are in `~/.config/crawlds/core.toml`:

```toml
[idle]
idle_timeout_secs = 300
dim_timeout_secs = 60
sleep_timeout_secs = 600
screen_off_timeout_secs = 600
lock_timeout_secs = 660
suspend_timeout_secs = 1800
fade_duration_secs = 5

screen_off_command = "loginctl lock-session"
lock_command = "loginctl lock-session"
suspend_command = "systemctl suspend"
```

The QML shell settings only have an "enabled" toggle - all timeouts and commands are in the daemon config.

---

## PowerProfileService

Located in `quickshell/services/PowerProfileService.qml`:

### Architecture

**Client-side + backend** - GET/POST to `/power/profile`:

```
PowerProfileService → CrawlDSService → /power/profile endpoint → crawlds-power
```

### Properties

```qml
property int profile: 0        // 0=Balanced, 1=PowerSaver, 2=Performance
readonly property bool available: _profileAvailable
readonly property bool hasPerformanceProfile: _hasPerformance
```

### Functions

```qml
function setProfile(p) {
    CrawlDSService.crawlPost("/power/profile", { profile: p })
    root.profile = p
}

function cycleProfile() {
    // Full cycle: 0 -> 2 -> 1 -> 0 (with performance)
    // Or: 0 <-> 1 (without performance)
}
```

### Profile Values

| Value | Name | Icon |
|-------|------|------|
| 0 | Balanced | "balanced" |
| 1 | Power saver | "powersaver" |
| 2 | Performance | "performance" |

---

## UI Components

### BatteryWidget

```qml
// quickshell/modules/bar/widgets/BatteryWidget.qml
// Uses BatteryService.primaryDevice
```

### BatteryPopout

```qml
// quickshell/modules/bar/popouts/battery/BatteryPopout.qml
// Shows multiple batteries, Bluetooth devices
```

### PowerButton

```qml
// quickshell/modules/bar/widgets/PowerButton.qml
// Power menu trigger
```

### PowerProfileSettings

```qml
// quickshell/modules/settings/bar/widgetSettings/PowerProfileSettings.qml
// Profile selector (Balanced/PowerSaver/Performance)
```

---

## Files Reference

| File | Purpose |
|------|---------|
| `core/config/crawl.toml` | Core config with `[idle]` section |
| `core/crates/crawlds-power/src/lib.rs` | Battery (UPower D-Bus), exports `run_idle()` |
| `core/crates/crawlds-power/src/idle.rs` | Idle detection (ScreenSaver D-Bus), actions |
| `core/crates/crawlds-ipc/src/events.rs` | `CrawlEvent::Power`, `CrawlEvent::Idle` event types |
| `core/crates/crawlds-daemon/src/config.rs` | `IdleConfig` struct |
| `core/crates/crawlds-daemon/src/main.rs` | Spawns idle task via `run_idle()` |
| `quickshell/services/BatteryService.qml` | Full battery (native UPower) |
| `quickshell/services/PowerService.qml` | Simple battery (CrawlDS) |
| `quickshell/services/PowerProfileService.qml` | Power profiles |
| `quickshell/services/core/IdleService.qml` | Idle detection, inhibition, fade overlay |
| `quickshell/modules/screen/wallpaper/FadeOverlay.qml` | Fade-to-black before idle actions |
| `quickshell/services/core/CrawlDSService.qml` | CrawlDS bridge |
| `quickshell/modules/bar/widgets/BatteryWidget.qml` | Bar widget |
| `quickshell/modules/bar/widgets/PowerButton.qml` | Power menu |

---

## Recommendations

1. **Idle integration** - Connect IdleService to PowerProfileService for auto power-saving
2. **Idle integration** - Connect IdleService to BrightnessService for auto-dimming
3. **Idle integration** - Connect IdleService to LockScreen for auto-lock on sleep
