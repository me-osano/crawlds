# Network Architecture

This document describes how network management (Wi-Fi, Ethernet, Hotspot) is implemented in CrawlDS, from backend to UI.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           UI Layer                                       │
│  NetworkIndicator (bar widget)                                        │
│  NetworkPanel (quick settings)                                    │
│  WifiSubTab, EthernetSubTab (settings)                              │
│  HotspotDialog                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      NetworkService                                      │
│  - Wi-Fi/Ethernet state management                                     │
│  - Network list with caching                                       │
│  - Scan management                                              │
│  - Hotspot control                                               │
│  - Connection state                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CrawlDSService                                       │
│  - Network state (connectivity, wifi_enabled, etc.)                     │
│  - SSE event handling                                             │
│  - POST wrappers for all operations                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CrawlDS Backend                                      │
│  - Uses NetworkManager D-Bus API                                   │
│  - Wi-Fi scan/connect/disconnect                                   │
│  - Ethernet management                                          │
│  - Hotspot (NM or hostapd+dnsmasq)                               │
│  - Publishes SSE events                                         │
└─────────────────────────────────────���───────────────────────────────────────┘
                                   │
                                   ▼
                        ┌─────────────────────┐
                        │  NetworkManager     │
                        │  org.freedesktop   │
                        └─────────────────────┘
```

## Backend (crawlds-network)

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/network/status` | GET | Overall network status |
| `/network/wifi` | GET | List visible + known Wi-Fi networks |
| `/network/wifi/details` | GET | Active Wi-Fi connection details |
| `/network/wifi/scan` | POST | Trigger Wi-Fi scan |
| `/network/wifi/connect` | POST | Connect to Wi-Fi network |
| `/network/wifi/disconnect` | POST | Disconnect Wi-Fi |
| `/network/wifi/forget` | POST | Remove a saved Wi-Fi profile |
| `/network/power` | POST | Enable/disable all network |
| `/network/eth` | GET | List Ethernet interfaces |
| `/network/eth/details` | GET | Ethernet interface details |
| `/network/eth/connect` | POST | Bring up Ethernet interface |
| `/network/eth/disconnect` | POST | Bring down Ethernet interface |
| `/network/hotspot/start` | POST | Start hotspot |
| `/network/hotspot/stop` | POST | Stop hotspot |
| `/network/hotspot/status` | GET | Get hotspot status |

### Request/Response Formats

#### Network Status

```bash
# GET /network/status
Response:
{
  "connectivity": "full",  // "full" | "limited" | "none"
  "wifi_enabled": true,
  "network_enabled": true,
  "wifi_available": true,
  "ethernet_available": true,
  "mode": "station",
  "active_ssid": "MyNetwork",
  "interfaces": [...]
}
```

#### Wi-Fi

```bash
# GET /network/wifi
Response: [
  {
    "ssid": "MyNetwork",
    "signal": 85,
    "secured": true,
    "connected": false,
    "existing": true,
    "cached": false,
    "password_required": true,
    "security": "wpa2-psk",
    "frequency_mhz": 5180,
    "bssid": "AA:BB:CC:DD:EE:FF",
    "last_seen_ms": 1713248000000
  }
]

# POST /network/wifi/connect
Body: { "ssid": "MyNetwork", "password": "secret123" }
Response: { "ok": true }

# POST /network/wifi/scan
Response: { "ok": true }

# POST /network/wifi/forget
Body: { "ssid": "MyNetwork" }
Response: { "ok": true }
```

#### Ethernet

```bash
# GET /network/eth
Response: [
  {
    "name": "eth0",
    "state": "connected",
    "ip4": "192.168.1.10/24",
    "ip6": [],
    "mac": "AA:BB:CC:DD:EE:FF"
  }
]

# GET /network/eth/details?interface=eth0
Response: {
  "ifname": "eth0",
  "speed": "1000 Mbps",
  "ipv4": "192.168.1.10/24",
  "ipv6": [],
  "gateway4": "192.168.1.1",
  "gateway6": [],
  "dns4": ["8.8.8.8", "8.8.4.4"],
  "dns6": [],
  "mac": "AA:BB.CC:DD:EE:FF"
}
```

#### Hotspot

```bash
# POST /network/hotspot/start
Body: { "ssid": "MyHotspot", "password": "secret123", "band": "5" }
Response: { "ok": true }

# POST /network/hotspot/stop
Response: { "ok": true }

# GET /network/hotspot/status
Response: {
  "active": true,
  "ssid": "MyHotspot",
  "iface": "wlan0",
  "band": "5",
  "channel": 36,
  "clients": [
    { "mac": "11:22:33:44:55:66", "ip": "10.10.10.50" }
  ],
  "backend": "networkmanager",
  "supports_virtual_ap": true
}
```

### Implementation

Located in `core/crates/crawlds-network/src/lib.rs`:

```rust
pub struct Config {
    pub wifi_scan_on_start: bool,
    pub wifi_scan_finish_delay_ms: u64,
    pub hotspot_backend: HotspotBackend,
    pub hotspot_virtual_iface: bool,
}
```

### Polling Architecture

The network module uses two consolidated tickers instead of five:

| Ticker | Interval | Updates |
|--------|----------|---------|
| `fast_ticker` | 5s | Snapshot, Ethernet, Interfaces |
| `slow_ticker` | 30s | WiFi scan, WiFi details, Hotspot status |

**Before:** 5 separate D-Bus calls every 3-30s
**After:** 2 tickers with ~60% fewer D-Bus calls

This reduces D-Bus overhead significantly while maintaining responsive updates for connectivity changes.

pub struct NetInterface {
    pub name: String,
    pub state: String,
    pub ip4: Option<String>,
    pub ip6: Vec<String>,
    pub mac: Option<String>,
}

pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub secured: bool,
    pub connected: bool,
    pub existing: bool,
    pub cached: bool,
    pub password_required: bool,
    pub security: String,
    pub frequency_mhz: Option<u32>,
    pub bssid: Option<String>,
    pub last_seen_ms: Option<u64>,
}

pub struct ActiveWifiDetails {
    pub ifname: Option<String>,
    pub ssid: Option<String>,
    pub signal: Option<u8>,
    pub frequency_mhz: Option<u32>,
    pub band: Option<String>,
    pub channel: Option<u32>,
    pub rate_mbps: Option<u32>,
    pub ip4: Option<String>,
    pub ip6: Vec<String>,
    pub gateway4: Option<String>,
    pub gateway6: Vec<String>,
    pub dns4: Vec<String>,
    pub dns6: Vec<String>,
    pub security: Option<String>,
    pub bssid: Option<String>,
    pub mac: Option<String>,
}

pub enum HotspotBackend {
    NetworkManager,
    Hostapd,
}
```

### Wi-Fi Scanning

1. `POST /network/wifi/scan` triggers `nmcli device wifi rescan`
2. Background timer waits `wifi_scan_finish_delay_ms` (configurable)
3. SSE events: `wifi_scan_started` → `wifi_scan_finished`
4. `list_wifi()` queries all visible APs from NM D-Bus
5. `list_known_wifi_ssids()` queries saved profiles
6. Merges to populate `existing` and `cached` fields

### Hotspot Backends

#### NetworkManager (default)

- Simple, no external dependencies
- Requires ethernet upstream
- Creates shared connection profile with `mode=ap`

#### hostapd + dnsmasq

- For Wi-Fi upstream (tethering)
- Creates virtual AP interface via `iw`
- Runs hostapd and dnsmasq processes
- Enables iptables NAT/MASQUERADE

### SSE Events

```json
{
  "domain": "network",
  "data": {
    "event": "connected",
    "ssid": "MyNetwork"
  }
}

{
  "domain": "network",
  "data": {
    "event": "disconnected"
  }
}

{
  "domain": "network",
  "data": {
    "event": "wifi_scan_started"
  }
}

{
  "domain": "network",
  "data": {
    "event": "wifi_scan_finished"
  }
}

{
  "domain": "network",
  "data": {
    "event": "wifi_list_updated",
    "networks": [...]
  }
}

{
  "domain": "network",
  "data": {
    "event": "hotspot_started",
    "status": { ... }
  }
}

{
  "domain": "network",
  "data": {
    "event": "hotspot_client_joined",
    "client": { "mac": "...", "ip": "..." }
  }
}
```

## CrawlDSService

Located in `quickshell/services/CrawlDSService.qml`:

### Properties

```qml
// ── Network ─────────────────────────────────────────────────────────
property bool   netWifiEnabled:    false
property bool   netWifiAvailable:  false
property bool   netEthernetAvailable: false
property string netConnectivity:   "unknown"    // full | limited | none
property string netActiveSsid:     ""
property int    netSignal:         0

signal netWifiListUpdated(var networks)
signal netWifiDetailsUpdated(var details)
signal netEthernetListUpdated(var interfaces)
signal netEthernetDetailsUpdated(var details)
signal netWifiScanStarted()
signal netWifiScanFinished()
signal netHotspotStatusChanged(var status)
signal netHotspotStarted(var status)
signal netHotspotStopped()
```

### Functions

```qml
// Network operations via CrawlDSService
function crawlPost("/network/wifi/scan", {})
function crawlPost("/network/wifi/connect", { ssid, password })
function crawlPost("/network/wifi/disconnect", {})
function crawlPost("/network/wifi/forget", { ssid })
function crawlPost("/network/power", { on: true/false })
function crawlPost("/network/eth/connect", { interface })
function crawlPost("/network/eth/disconnect", { interface })
function crawlPost("/network/hotspot/start", { ssid, password, band, channel })
function crawlPost("/network/hotspot/stop", {})
```

## NetworkService

Located in `quickshell/services/NetworkService.qml`:

### Architecture

The NetworkService manages:
1. Wi-Fi scanning and list
2. Ethernet interfaces
3. Hotspot control
4. File caching for instant display

```
NetworkService
    │
    ├── Wi-Fi Management
    │   ├── scan() → CrawlDSService
    │   ├── refreshNetworks() → CrawlDSService._fetchInitial
    │   └── connect(ssid, password) → CrawlDSService.crawlPost
    │
    ├── Ethernet Management
    │   ├── refreshEthernet()
    │   ├── refreshActiveEthernetDetails()
    │   └── connectEthernet/disconnectEthernet
    │
    ├── Hotspot Control
    │   ├── startHotspot(...)
    │   ├── stopHotspot()
    │   └── refreshHotspotStatus()
    │
    └── File Caching
        ├── FileView (network.json)
        └── JsonAdapter
```

### Properties

```qml
property var networks: ({})          // SSID → network mapping
property bool scanning: false
property bool scanningActive: false
property string connectingTo: ""
property bool connecting: false
property string disconnectingFrom: ""
property string forgettingNetwork: ""
property string lastError: ""

property bool wifiAvailable: false
property bool ethernetConnected: false
property var ethernetInterfaces: ([])
property string activeWifiIf: ""
property string activeEthernetIf: ""
property var activeWifiDetails: ({})
property var activeEthernetDetails: ({})

// Hotspot
property bool hotspotActive: false
property var hotspotStatus: ({})
property bool hotspotStarting: false
property bool hotspotStopping: false

// TTL caching
property int activeWifiDetailsTtlMs: 5000
property int activeEthernetDetailsTtlMs: 5000
```

### Key Properties (from CrawlDSService)

```qml
readonly property bool wifiEnabled: CrawlDSService.netWifiEnabled
readonly property string networkConnectivity: CrawlDSService.netConnectivity
readonly property bool internetConnectivity: CrawlDSService.netConnectivity === "full"
readonly property string activeSsid: CrawlDSService.netActiveSsid
readonly property bool wifiAvailableFromDaemon: CrawlDSService.netWifiAvailable
readonly property bool ethernetAvailableFromDaemon: CrawlDSService.netEthernetAvailable

readonly property bool airplaneModeEnabled: Settings.data.network.airplaneModeEnabled
```

### Functions

#### Wi-Fi

```qml
function setWifiEnabled(on) {
    Settings.data.network.wifiEnabled = on
    CrawlDSService.crawlPost("/network/power", { on: on })
    if (on) { scan(); refreshActiveWifiDetails() }
}

function setAirplaneMode(on) {
    Settings.data.network.airplaneModeEnabled = on
    if (typeof BluetoothService !== "undefined") {
        BluetoothService.setAirplaneMode(on)
    }
    if (on) { setWifiEnabled(false) }
}

function scan() {
    scanning = true
    scanningActive = true
    ignoreScanResults = false
    CrawlDSService.crawlPost("/network/wifi/scan", {})
    refreshNetworks()
}

function refreshNetworks() {
    CrawlDSService._fetchInitial("/network/wifi", function(list) {
        applyWifiList(list)
        scanning = false
    })
}

function connect(ssid, password, hidden, securityKey, identity, enterprise) {
    connecting = true
    connectingTo = ssid
    CrawlDSService.crawlPost("/network/wifi/connect", { ssid, password })
    refreshNetworks()
}

function disconnect(ssid) {
    disconnectingFrom = ssid
    CrawlDSService.crawlPost("/network/wifi/disconnect", {})
    refreshNetworks()
}

function forget(ssid) {
    forgettingNetwork = ssid
    CrawlDSService.crawlPost("/network/wifi/forget", { ssid })
    // Remove from local cache
}
```

#### Ethernet

```qml
function refreshEthernet() {
    CrawlDSService._fetchInitial("/network/eth", function(list) {
        ethernetInterfaces = list || []
        ethernetConnected = ethernetInterfaces.some(i => i.connected)
    })
}

function connectEthernet(iface) {
    CrawlDSService.crawlPost("/network/eth/connect", { interface: iface })
    refreshEthernet()
}

function disconnectEthernet(iface) {
    CrawlDSService.crawlPost("/network/eth/disconnect", { interface: iface })
    refreshEthernet()
}
```

#### Hotspot

```qml
function startHotspot(ssid, password, iface, band, channel, backend) {
    hotspotStarting = true
    const payload = { ssid: ssid || "CrawlDS-Hotspot" }
    if (password) payload.password = password
    if (iface) payload.iface = iface
    if (band) payload.band = band
    if (channel) payload.channel = channel
    if (backend) payload.backend = backend
    CrawlDSService.crawlPost("/network/hotspot/start", payload)
    refreshHotspotStatus()
}

function stopHotspot() {
    hotspotStopping = true
    CrawlDSService.crawlPost("/network/hotspot/stop", {})
    // Update state after delay
}

function refreshHotspotStatus() {
    CrawlDSService._fetchInitial("/network/hotspot/status", function(status) {
        hotspotActive = status && status.active
        hotspotStatus = status || {}
    })
}
```

### Connection State Management

```qml
function applyWifiList(list) {
    // Convert array to SSID-keyed object
    const mapped = {}
    for (network of list) {
        mapped[network.ssid] = {
            ssid: network.ssid,
            signal: network.signal || 0,
            secured: network.secured || false,
            connected: network.connected || false,
            existing: network.existing || false,
            cached: network.cached || false,
            passwordRequired: network.password_required || false,
            security: network.security || "open",
            frequency: network.frequency_mhz || null,
            bssid: network.bssid || "",
            lastSeen: network.last_seen_ms || 0
        }
    }
    networks = mapped
    updateCache(mapped)
}
```

### Helper Functions

```qml
function isSecured(security) {
    return security && security !== "open"
}

function isEnterprise(security) {
    return security && security.indexOf("eap") !== -1
}

function getSignalStrengthLabel(signal) {
    if (signal >= 80) return "Excellent"
    if (signal >= 60) return "Good"
    if (signal >= 40) return "Fair"
    if (signal >= 20) return "Weak"
    return "Poor"
}

function signalIcon(signal, connected) {
    if (!connected) return "wifi-off"
    if (signal >= 70) return "wifi-2"
    if (signal >= 35) return "wifi-1"
    return "wifi-0"
}
```

### File Caching

```qml
property string cacheFile: Settings.cacheDir + "network.json"
readonly property var cachedNetworks: cacheAdapter.networks

function updateCache(mapped) {
    cacheAdapter.networks = mapped
    cacheSaveDebounce.restart()
}

function restoreCache() {
    if (cachedNetworks && Object.keys(cachedNetworks).length > 0) {
        networks = cachedNetworks
    }
}

// FileView + JsonAdapter persists to network.json
// Restored on Component.onCompleted
// Updates on scan results
```

### SSE Event Connections

```qml
Connections {
    target: CrawlDSService
    
    function onNetConnectivityChanged() { refreshEthernet() }
    function onNetWifiListUpdated(networks) { applyWifiList(networks); scanning = false }
    function onNetWifiScanStarted() { scanning = true; scanningActive = true }
    function onNetWifiScanFinished() { scanning = false }
    function onNetWifiDetailsUpdated(details) { ... }
    function onNetEthernetListUpdated(interfaces) { ... }
    function onNetEthernetDetailsUpdated(details) { ... }
    function onNetWifiEnabledChanged() { ... }
    function onNetWifiAvailableChanged() { ... }
    function onNetHotspotStatusChanged(data) { ... }
    function onNetHotspotStarted(status) { ... }
    function onNetHotspotStopped() { ... }
}

Connections {
    target: SessionService
    function onSessionResumed() {
        // Full refresh on wake
        refreshEthernet()
        refreshNetworks()
        refreshActiveWifiDetails()
        refreshActiveEthernetDetails()
        refreshHotspotStatus()
    }
}
```

### TTL Caching

The service uses TTL-based caching to prevent excessive NM queries:

| Data | TTL | Behavior |
|---|---|---|
| Wi-Fi list | 5s | Refresh on next request after expiry |
| Wi-Fi details | 5s | Refresh on next request after expiry |
| Ethernet details | 5s | Refresh on next request after expiry |

A background tick fetches latest state every 5 seconds and broadcasts SSE events if data changed.

### Scan Delay and Stale Result Handling

```qml
// scanDelayTimer ignores results that arrived before scan was triggered
function scan() {
    if (scanning) {
        ignoreScanResults = true
        scanPending = true
        return
    }
    scanning = true
    scanningActive = true
    ignoreScanResults = false
    CrawlDSService.crawlPost("/network/wifi/scan", {})
    refreshNetworks()
}
```

## UI Components

### NetworkIndicator

Bar widget showing connection status:

```qml
// quickshell/modules/bar/widgets/NetworkIndicator.qml
property bool connected: NetworkService.internetConnectivity
property string ssid: NetworkService.activeSsid

onClicked: NetworkService.scan()
```

### NetworkPanel

Quick settings popout:

```qml
// quickshell/modules/panels/network/NetworkPanel.qml
- Wi-Fi toggle
- Current network details
- Quick disconnect
- Scan button
- Signal strength
```

### WifiSubTab

Full Wi-Fi settings:

```qml
// quickshell/modules/settings/tabs/Network/WifiSubTab.qml
- Wi-Fi on/off toggle
- Network list (cached + visible)
- Connect dialog (password input)
- Security type handling
- Hidden network support
- Forget network option
```

### EthernetSubTab

Ethernet settings:

```qml
// quickshell/modules/settings/tabs/Network/EthernetSubTab.qml
- Interface list
- Connection status
- Details (speed, IP, gateway, DNS)
- Connect/disconnect
```

### HotspotDialog

Hotspot control:

```qml
// Start hotspot with SSID, password, band, channel, backend
function startHotspot(config) { ... }
function stopHotspot() { ... }
```

## Configuration

### Backend Config (core.toml)

```toml
[network]
wifi_scan_finish_delay_ms = 500
hotspot_backend = "networkmanager"  # or "hostapd"
hotspot_virtual_iface = true
```

### Settings (QML)

```qml
Settings.data.network.wifiEnabled
Settings.data.network.airplaneModeEnabled
Settings.data.network.hotspotEnabled
```

## Limitations and Roadmap

### Current Limitations

1. **Enterprise Wi-Fi** - Partial support (can connect but limited config)
2. **IPv6 NAT** - Not configured for hotspot
3. **Per-client bandwidth limits** - Not implemented
4. **Hotspot QR code** - Not generated
5. **Wi-Fi Direct/P2P** - Not supported

### Backend Could Support

1. Full 802.1X enterprise with certificates
2. Wi-Fi Direct device-to-device
3. VPN integration via NM
4. Network bonds and bridges
5. Band steering between 2.4/5GHz

## Files Reference

| File | Purpose |
|------|---------|
| `core/crates/crawlds-network/src/lib.rs` | Backend implementation |
| `core/crates/crawlds-ipc/src/types.rs` | IPC types |
| `core/crates/crawlds-daemon/src/router.rs` | HTTP endpoints |
| `quickshell/services/CrawlDSService.qml` | State + HTTP wrappers |
| `quickshell/services/NetworkService.qml` | Full network logic |
| `quickshell/modules/bar/widgets/NetworkIndicator.qml` | Bar widget |
| `quickshell/modules/panels/network/NetworkPanel.qml` | Quick settings |
| `quickshell/modules/settings/tabs/Network/*.qml` | Settings tabs |

## Historical Context

### Evolution

The network implementation has evolved:
- Originally: Direct socket calls from UI → NetworkManager
- Current: UI → NetworkService → CrawlDSService → CrawlDS Backend → NetworkManager

This follows the same pattern as Bluetooth - single source of truth via the backend.

### Airplane Mode

NetworkService also manages airplane mode, which affects:
- Wi-Fi (via `/network/power`)
- Bluetooth (via `BluetoothService.setAirplaneMode()`)