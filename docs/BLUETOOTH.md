# Bluetooth Architecture

This document describes how Bluetooth control is implemented in CrawlDS, from backend to UI.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           UI Layer                                         │
│  BluetoothIndicator (bar widget)                                          │
│  BluetoothPanel (quick settings)                                          │
│  BluetoothTab (settings)                                                  │
│  BatteryPopout (show paired devices)                                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      BluetoothService                                     │
│  - Device filtering/sorting                                               │
│  - Device icon mapping                                                     │
│  - Signal strength calculation                                            │
│  - Delegates all operations to CrawlDSService                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CrawlDSService                                       │
│  - Bluetooth state (powered, discovering, devices)                       │
│  - SSE event handling                                                     │
│  - POST wrappers for all operations                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CrawlDS Backend                                      │
│  - Uses bluer crate (BlueZ wrapper)                                       │
│  - Manages adapter, devices, pairing                                      │
│  - Publishes SSE events                                                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
                        ┌─────────────────────┐
                        │  BlueZ (bluer)       │
                        │  org.bluez via D-Bus │
                        └─────────────────────┘
```

## Backend (crawlds-bluetooth)

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/bluetooth/status` | GET | Get adapter status |
| `/bluetooth/devices` | GET | Get paired devices |
| `/bluetooth/scan` | POST | Start device discovery |
| `/bluetooth/power` | POST | Enable/disable adapter |
| `/bluetooth/connect` | POST | Connect to device |
| `/bluetooth/disconnect` | POST | Disconnect device |
| `/bluetooth/pair` | POST | Pair with device |
| `/bluetooth/trust` | POST | Set device trusted (auto-connect) |
| `/bluetooth/remove` | POST | Remove/forget device |
| `/bluetooth/alias` | POST | Set device alias |
| `/bluetooth/discoverable` | POST | Set discoverable |
| `/bluetooth/pairable` | POST | Set pairable |

### Request/Response Formats

```bash
# GET /bluetooth/status
Response:
{
  "powered": true,
  "discovering": false,
  "devices": [
    {
      "address": "AA:BB:CC:DD:EE:FF",
      "name": "AirPods Pro",
      "connected": true,
      "paired": true,
      "rssi": -45,
      "battery": 85,
      "icon": "audio-headphones"
    }
  ]
}

# POST /bluetooth/power
Body: { "on": true }
Response: { "ok": true }

# POST /bluetooth/connect
Body: { "address": "AA:BB:CC:DD:EE:FF" }
Response: { "ok": true }

# POST /bluetooth/disconnect
Body: { "address": "AA:BB:CC:DD:EE:FF" }
Response: { "ok": true }

# POST /bluetooth/pair
Body: { "address": "AA:BB:CC:DD:EE:FF" }
Response: { "ok": true }

# POST /bluetooth/trust
Body: { "address": "AA:BB:CC:DD:EE:FF", "trusted": true }
Response: { "ok": true }

# POST /bluetooth/remove
Body: { "address": "AA:BB:CC:DD:EE:FF" }
Response: { "ok": true }

# POST /bluetooth/discoverable
Body: { "on": true }
Response: { "ok": true }
```

### Implementation

Located in `core/crates/crawlds-bluetooth/src/lib.rs`:

```rust
pub struct Config {
    pub auto_power: bool,
    pub scan_timeout_secs: u64,
}

pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) -> anyhow::Result<()> {
    // 1. Connect to BlueZ session
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;

    // 2. Register agent for PIN handling
    let _agent_handle = register_agent(&session).await?;

    // 3. Publish existing devices
    for addr in adapter.device_addresses().await? {
        let dev = adapter.device(addr)?;
        let bt_dev = device_to_ipc(&dev).await;
        tx.send(CrawlEvent::Bluetooth(BtEvent::DeviceDiscovered { device: bt_dev }));
        tokio::spawn(watch_device(dev, addr.to_string(), tx));
    }

    // 4. Watch for adapter events (device added/removed)
    let mut adapter_events = adapter.events().await?;
    while let Some(event) = adapter_events.next().await {
        match event {
            AdapterEvent::DeviceAdded(addr) => {
                // Publish new device
            }
            AdapterEvent::DeviceRemoved(addr) => {
                // Publish device removed
            }
        }
    }
}

pub async fn get_status() -> Result<BtStatus, BtError>;
pub async fn get_devices() -> Result<Vec<BtDevice>, BtError>;
pub async fn set_powered(on: bool) -> Result<(), BtError>;
pub async fn scan() -> Result<(), BtError>;
pub async fn connect(address: &str) -> Result<(), BtError>;
pub async fn disconnect(address: &str) -> Result<(), BtError>;
pub async fn pair(address: &str) -> Result<(), BtError>;
pub async fn set_trusted(address: &str, trusted: bool) -> Result<(), BtError>;
pub async fn remove_device(address: &str) -> Result<(), BtError>;
pub async fn set_discoverable(on: bool) -> Result<(), BtError>;
```

### SSE Events

The backend publishes Bluetooth events:

```json
{
  "domain": "bluetooth",
  "data": {
    "event": "adapter_powered",
    "on": true
  }
}

{
  "domain": "bluetooth",
  "data": {
    "event": "device_connected",
    "device": {
      "address": "AA:BB:CC:DD:EE:FF",
      "name": "AirPods Pro",
      "connected": true
    }
  }
}

{
  "domain": "bluetooth",
  "data": {
    "event": "device_disconnected",
    "address": "AA:BB:CC:DD:EE:FF"
  }
}

{
  "domain": "bluetooth",
  "data": {
    "event": "scan_started"
  }
}

{
  "domain": "bluetooth",
  "data": {
    "event": "scan_stopped"
  }
}
```

### Types

```rust
// In core/crates/crawlds-ipc/src/types.rs
pub struct BtDevice {
    pub address: String,
    pub name: Option<String>,
    pub connected: bool,
    pub paired: bool,
    pub rssi: Option<i16>,
    pub battery: Option<u8>,
    pub icon: Option<String>,
}

pub struct BtStatus {
    pub powered: bool,
    pub discovering: bool,
    pub devices: Vec<BtDevice>,
}
```

## CrawlDSService

Located in `quickshell/services/CrawlDSService.qml`:

### Properties

```qml
// ── Bluetooth ─────────────────────────────────────────────────────────
property bool   btPowered:         false
property bool   btDiscovering:     false
property var    btDevices:         []
property int    btConnectedCount:  0
```

### Functions

```qml
// ── Bluetooth control ───────────────────────────────────────────────────────
function setBtPowered(on)     { crawlPost("/bluetooth/power", { on: on }) }
function startBtScan()         { crawlPost("/bluetooth/scan", {}) }
function setBtDiscoverable(on){ crawlPost("/bluetooth/discoverable", { on: on }) }
function setBtPairable(on)     { crawlPost("/bluetooth/pairable", { on: on }) }

function connectBtDevice(address)  { crawlPost("/bluetooth/connect", { address: address }) }
function disconnectBtDevice(address) { crawlPost("/bluetooth/disconnect", { address: address }) }
function pairBtDevice(address) { crawlPost("/bluetooth/pair", { address: address }) }
function forgetBtDevice(address) { crawlPost("/bluetooth/remove", { address: address }) }
function setBtTrusted(address, trusted) { crawlPost("/bluetooth/trust", { address: address, trusted: trusted }) }
```

### SSE Event Handling

```qml
function _dispatch(evt) {
    switch (evt.domain) {
    case "bluetooth": _handleBtEvent(evt.data); break
    }
}

function _handleBtEvent(data) {
    switch (data.event) {
    case "adapter_powered": root.btPowered = data.on; break
    case "device_connected":
    case "device_disconnected":
        _fetchInitial("/bluetooth/status", _handleBt)
        break
    case "scan_started": root.btDiscovering = true;  break
    case "scan_stopped": root.btDiscovering = false; break
    }
}
```

## BluetoothService

Located in `quickshell/services/BluetoothService.qml`:

### Architecture

The simplified service now delegates all operations to CrawlDSService:

```
BluetoothService
         │
         ▼
CrawlDSService
         │
         ▼
  HTTP POST requests
         │
         ▼
CrawlDS Backend
         │
         ▼
     bluer
```

### Properties

```qml
readonly property bool bluetoothAvailable: true
readonly property bool enabled: CrawlDSService.btPowered
readonly property bool scanningActive: CrawlDSService.btDiscovering
property bool discoverable: false

readonly property var devices: CrawlDSService.btDevices || []
readonly property var devicesList: // Array from devices
readonly property var connectedDevices: // Filtered by connected
```

### Control Functions

```qml
function setBluetoothEnabled(state) {
    CrawlDSService.setBtPowered(state);
}

function setScanActive(active) {
    if (active) {
        CrawlDSService.startBtScan();
    } else {
        CrawlDSService.setBtDiscoverable(false);
    }
}

function setDiscoverable(state) {
    CrawlDSService.setBtDiscoverable(state);
}
```

### Device Operations

```qml
function connectDeviceWithTrust(device) {
    var addr = macFromDevice(device);
    CrawlDSService.setBtTrusted(addr, true);
    CrawlDSService.connectBtDevice(addr);
}

function disconnectDevice(device) {
    CrawlDSService.disconnectBtDevice(macFromDevice(device));
}

function pairDevice(device) {
    CrawlDSService.pairBtDevice(macFromDevice(device));
}

function forgetDevice(device) {
    CrawlDSService.forgetBtDevice(macFromDevice(device));
}

function setDeviceAutoConnect(device, enabled) {
    CrawlDSService.setBtTrusted(macFromDevice(device), enabled);
}
```

### Helper Functions

```qml
function sortDevices(devList)      // Sort by signal strength
function dedupeDevices(list)       // Remove duplicates
function macFromDevice(device)    // Extract MAC address
function deviceKey(device)        // Unique device identifier
function getDeviceIcon(device)  // Map to icon name
function getSignalPercent(device) // RSSI → percentage
function getSignalIcon(device)    // Signal quality icon
function getBatteryPercent(device) // Device battery
```

## UI Components

### BluetoothIndicator

Bar widget showing connected device count and toggle:

```qml
// quickshell/modules/bar/widgets/BluetoothIndicator.qml
readonly property int connected: BluetoothService.connectedDevices.length

onClicked: BluetoothService.setBluetoothEnabled(!BluetoothService.enabled)
```

### BluetoothPanel

Quick settings popout:

```qml
// quickshell/modules/panels/bluetooth/BluetoothPanel.qml
- Toggle adapter on/off
- Show connected devices
- Quick disconnect
```

### BluetoothTab

Full settings page:

```qml
// quickshell/modules/settings/tabs/Bluetooth/BluetoothTab.qml
- Device on/off toggle
- Discoverable toggle
- Scan button
- Device list (connected, paired, available)
- Device details expand
- Pair/Connect/Forget actions
- Auto-connect toggle
```

### BatteryPopout

Shows paired Bluetooth devices with battery:

```qml
// quickshell/modules/bar/popouts/battery/BatteryPopout.qml
icon: BluetoothService.getDeviceIcon(modelData)
```

## Historical Context

### Before Simplification

The original BluetoothService had:
- Native BluetoothAdapter (Quickshell.Bluetooth)
- bluetoothctl fallback polling
- RSSI round-robin polling via bluetoothctl
- Airplane mode handling
- Device pairing with PIN input
- Direct rfkill calls

This created:
- Duplicate state (backend + native + ctl)
- Complex fallback logic
- Potential state divergence

### After Simplification

Now:
- Single source of truth (CrawlDSService)
- All operations via backend
- Simpler code (755 → 272 lines)
- Consistent architecture

## Files Reference

| File | Purpose |
|------|---------|
| `core/crates/crawlds-bluetooth/src/lib.rs` | Backend (bluer) |
| `core/crates/crawlds-ipc/src/types.rs` | IPC types |
| `core/crates/crawlds-daemon/src/router.rs` | HTTP endpoints |
| `quickshell/services/CrawlDSService.qml` | State + HTTP wrappers |
| `quickshell/services/BluetoothService.qml` | UI logic |
| `quickshell/modules/bar/widgets/BluetoothIndicator.qml` | Bar widget |
| `quickshell/modules/panels/bluetooth/BluetoothPanel.qml` | Quick settings |
| `quickshell/modules/settings/tabs/Bluetooth/BluetoothTab.qml` | Settings |

## Future Improvements

1. **Per-device brightness**: Add battery percentage endpoints
2. **PIN pairing**: Add proper Pine input handling in backend
3. **Audio profiles**: A2DP, HFP, etc.
4. **LE Audio**: Bluetooth LE Audio support
5. **Multi-adapter**: Support for multiple adapters