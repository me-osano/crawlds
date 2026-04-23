# Inter-Process Communication (IPC)

The CrawlDS IPC crate (`crawlds-ipc`) provides shared types, event models, and error handling used across all CrawlDS components. It has no system dependencies and is designed to be usable from any crate including the QML bridge.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        crawlds-ipc crate                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ types.rs                                                              │   │
│  │   └── All shared data structures (BtDevice, WifiNetwork, CpuStatus...) │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ events.rs                                                             │   │
│  │   └── CrawlEvent enum (all domain events)                             │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ error.rs                                                              │   │
│  │   └── CrawlError, ErrorEnvelope                                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         ▼                           ▼                           ▼
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│ crawlds-daemon  │       │  crawlds-cli   │       │  quickshell/    │
│                 │       │                 │       │  CrawlDSService │
│ • Emits events  │       │ • Consumes API │       │                 │
│ • Handles HTTP │       │ • Watch mode   │       │ • SSE listener  │
│ • Type library  │       │ • Type library │       │ • HTTP calls    │
└─────────────────┘       └─────────────────┘       └─────────────────┘
```

## CrawlEvent

The `CrawlEvent` enum is the root event type, tagged by domain. It uses serde's adjacently-tagged representation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "domain", content = "data", rename_all = "snake_case")]
pub enum CrawlEvent {
    Bluetooth(BtEvent),
    Network(NetEvent),
    Notify(NotifyEvent),
    Clipboard(ClipboardEvent),
    Sysmon(SysmonEvent),
    Brightness(BrightnessEvent),
    Proc(ProcEvent),
    Power(PowerEvent),
    Disk(DiskEvent),
    Greeter(GreeterEvent),
    Webservice(WebserviceEvent),
    Daemon(DaemonEvent),
}
```

### JSON Format

Events serialize as tagged JSON:

```json
{
    "domain": "sysmon",
    "data": {
        "event": "cpu_update",
        "cpu": { "..." : "..." }
    }
}
```

## Domain Events

### Bluetooth (BtEvent)

```rust
pub enum BtEvent {
    DeviceDiscovered { device: BtDevice },
    DeviceConnected { device: BtDevice },
    DeviceDisconnected { address: String },
    DeviceRemoved { address: String },
    AdapterPowered { on: bool },
    ScanStarted,
    ScanStopped,
}
```

### Network (NetEvent)

```rust
pub enum NetEvent {
    Connected { ssid: Option<String>, iface: String },
    Disconnected { iface: String },
    IpChanged { iface: String, ip: String },
    WifiEnabled,
    WifiDisabled,
    WifiScanStarted,
    WifiScanFinished,
    WifiListUpdated { networks: Vec<WifiNetwork> },
    ActiveWifiDetailsChanged { details: ActiveWifiDetails },
    EthernetInterfacesChanged { interfaces: Vec<EthernetInterface> },
    ActiveEthernetDetailsChanged { details: ActiveEthernetDetails },
    ModeChanged { mode: NetMode },
    ConnectivityChanged { state: String },
    HotspotStarted { status: HotspotStatus },
    HotspotStopped,
    HotspotStatusChanged { status: HotspotStatus },
    HotspotClientJoined { client: HotspotClient },
    HotspotClientLeft { mac: String },
}
```

### Notifications (NotifyEvent)

```rust
pub enum NotifyEvent {
    New { notification: Notification },
    Closed { id: u32, reason: u32 },
    ActionInvoked { id: u32, action_key: String },
    Replaced { notification: Notification },
}
```

### Clipboard (ClipboardEvent)

```rust
pub enum ClipboardEvent {
    Changed { entry: ClipEntry },
    PrimaryChanged { entry: ClipEntry },
}
```

### System Monitor (SysmonEvent)

```rust
pub enum SysmonEvent {
    CpuUpdate { cpu: CpuStatus },
    MemUpdate { mem: MemStatus },
    NetUpdate { traffic: NetTraffic },
    GpuUpdate { gpu: GpuStatus },
    CpuSpike { usage: f32, threshold: f32 },
    MemPressure { used_percent: f32 },
}
```

### Brightness (BrightnessEvent)

```rust
pub enum BrightnessEvent {
    Changed { status: BrightnessStatus },
}
```

### Processes (ProcEvent)

```rust
pub enum ProcEvent {
    Spawned { pid: u32, name: String },
    Exited { pid: u32, name: String, exit_code: Option<i32> },
}
```

### Power (PowerEvent)

```rust
pub enum PowerEvent {
    BatteryUpdate { status: BatteryStatus },
    AcConnected,
    AcDisconnected,
    LowBattery { percent: f64 },
    Critical { percent: f64 },
}
```

### Disk (DiskEvent)

```rust
pub enum DiskEvent {
    DeviceMounted { device: BlockDevice },
    DeviceUnmounted { device_path: String },
    DeviceAdded { device: BlockDevice },
    DeviceRemoved { device_path: String },
}
```

### Greeter (GreeterEvent)

```rust
pub enum GreeterEvent {
    StateChanged { status: GreeterStatus },
    AuthMessage { message: String, message_type: GreeterMessageType },
    AuthSuccess,
    AuthFailure { message: String },
    SessionStarted,
}
```

### Webservice (WebserviceEvent)

```rust
pub enum WebserviceEvent {
    RssFeedUpdated { feed_url: String, items: Vec<RssItem> },
    RssFeedError { feed_url: String, message: String },
    RssFeedsRefreshed,
    WallhavenResults { walls: Vec<Wallpaper> },
    WallhavenError { message: String },
}
```

### Daemon (DaemonEvent)

```rust
pub enum DaemonEvent {
    Started,
    Stopping,
    DomainError { domain: String, message: String },
}
```

## Error Handling

### CrawlError

```rust
pub enum CrawlError {
    Bluetooth(String),
    Network(String),
    Notification(String),
    Clipboard(String),
    Sysmon(String),
    Brightness(String),
    Process(String),
    Power(String),
    Disk(String),
    DBus(String),
    NotFound(String),
    PermissionDenied(String),
    Internal(String),
}
```

### ErrorEnvelope

All API errors return a standardized JSON envelope:

```rust
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

pub struct ErrorBody {
    pub domain: String,   // Error domain (e.g., "bluetooth", "network")
    pub code: String,     // Error code (e.g., "not_found", "permission_denied")
    pub message: String,  // Human-readable message
}
```

### JSON Format

```json
{
    "error": {
        "domain": "bluetooth",
        "code": "not_found",
        "message": "device not found: AA:BB:CC:DD:EE:FF"
    }
}
```

## SSE Connection

Connect to the SSE stream to receive real-time events:

```bash
curl --unix-socket /run/crawlds.sock http://localhost/events
```

### Event Format

SSE events are newline-delimited JSON:

```
event: sysmon
data: {"domain":"sysmon","data":{"event":"cpu_update","cpu":{...}}}
data: {"domain":"sysmon","data":{"event":"mem_update","mem":{...}}}
```

### Filtering

The CLI and QML frontend filter events by domain. Raw SSE includes all events.

## Data Types Summary

### Bluetooth Types
- `BtDevice` - Bluetooth device info
- `BtStatus` - Adapter and device list

### Network Types
- `WifiNetwork` - Available WiFi network
- `ActiveWifiDetails` - Current WiFi connection
- `EthernetInterface` - Ethernet interface
- `ActiveEthernetDetails` - Active ethernet connection
- `NetInterface` - Generic network interface
- `NetStatus` - Overall network status
- `HotspotStatus` - Hotspot configuration
- `HotspotClient` - Connected hotspot client

### Clipboard Types
- `ClipEntry` - Clipboard history entry

### System Monitor Types
- `CpuStatus` - CPU metrics
- `LoadAvg` - Load averages
- `MemStatus` - Memory metrics
- `NetTraffic` - Network traffic
- `GpuStatus` - GPU info
- `DiskStatus` - Disk metrics

### Process Types
- `ProcessInfo` - Process information

### Power Types
- `BatteryStatus` - Battery state
- `Urgency` - Notification urgency level

### Disk Types
- `BlockDevice` - Block device info

### Greeter Types
- `GreeterStatus` - Greeter state
- `GreeterState` - State enum
- `GreeterMessageType` - Message type enum

### Webservice Types
- `RssItem` - RSS article
- `Wallpaper` - Wallhaven wallpaper

## Module Structure

```rust
pub mod error;      // CrawlError, ErrorEnvelope
pub mod events;     // CrawlEvent, all domain events
pub mod types;      // All shared data types

pub use error::{CrawlError, CrawlResult, ErrorEnvelope};
pub use events::CrawlEvent;
```

## Usage in Rust

```rust
use crawlds_ipc::{CrawlEvent, events::SysmonEvent, ErrorEnvelope};

// Parse an event
let json = r#"{"domain":"sysmon","data":{"event":"cpu_update","cpu":{...}}}"#;
let event: CrawlEvent = serde_json::from_str(json)?;

// Match on domain
match event {
    CrawlEvent::Sysmon(SysmonEvent::CpuUpdate { cpu }) => {
        println!("CPU: {}%", cpu.aggregate);
    }
    CrawlEvent::Bluetooth(event) => { /* ... */ }
    CrawlEvent::Notify(event) => { /* ... */ }
    _ => { /* Other domains */ }
}

// Create an error
let error = ErrorEnvelope::new("bluetooth", "not_found", "device not found");
```

## Usage in QML

The CrawlDSService handles parsing events from SSE:

```qml
Connections {
    target: CrawlDSService

    function onSysmonCpuUpdate(data) {
        console.log("CPU:", data.aggregate + "%")
    }

    function onSysmonMemUpdate(data) {
        console.log("Memory:", data.used_kb, "/", data.total_kb)
    }

    function onBluetoothDeviceConnected(device) {
        console.log("Connected to:", device.name)
    }
}
```

## Event Naming Convention

Domain events follow snake_case naming:
- `device_discovered` (not `deviceDiscovered`)
- `wifi_enabled` (not `wifiEnabled`)
- `cpu_spike` (not `cpuSpike`)

The `CrawlEvent` tag is `domain` with snake_case values, while inner event tags use `event` with snake_case.

## Versioning

The IPC crate is versioned independently from the daemon and CLI. Breaking changes to event schemas or types will increment the IPC crate version.
