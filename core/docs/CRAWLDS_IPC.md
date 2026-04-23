# CrawlDS IPC / HTTP API

This document describes the HTTP-over-Unix-socket API exposed by the daemon and
the SSE event stream used for live updates.

---

## Overview

The daemon exposes a standard HTTP/1.1 API over a Unix socket. You can talk to
it with any HTTP client that supports Unix sockets.

Socket path: `$XDG_RUNTIME_DIR/crawlds.sock`

### Quick examples

```bash
curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/health
curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/sysmon/cpu
curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock \
     -X POST -H 'Content-Type: application/json' \
     -d '{"value":80}' http://localhost/brightness/set
```

With `socat`:

```bash
echo -e 'GET /sysmon/cpu HTTP/1.0\r\n\r\n' | \
    socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/crawlds.sock
```

---

## Request/response

All responses are JSON. Errors use the standard envelope:

```json
{
  "error": {
    "domain": "bluetooth",
    "code":   "not_powered",
    "message": "Bluetooth adapter is not powered"
  }
}
```

### Health

```
GET  /health
→ { "status": "ok", "version": "0.1.0" }
```

### Bluetooth

```
GET  /bluetooth/status          → BluetoothStatus
GET  /bluetooth/devices         → [BluetoothDevice]
POST /bluetooth/scan            → {}
POST /bluetooth/connect         ← { "address": "AA:BB:CC:DD:EE:FF" }
POST /bluetooth/disconnect      ← { "address": "AA:BB:CC:DD:EE:FF" }
POST /bluetooth/power           ← { "on": true }
POST /bluetooth/pair            ← { "address": "AA:BB:CC:DD:EE:FF" }
POST /bluetooth/trust           ← { "address": "AA:BB:CC:DD:EE:FF", "trusted": true }
POST /bluetooth/remove          ← { "address": "AA:BB:CC:DD:EE:FF" }
POST /bluetooth/alias           ← { "address": "AA:BB:CC:DD:EE:FF", "alias": "Headphones" }
POST /bluetooth/discoverable    ← { "on": true }
POST /bluetooth/pairable        ← { "on": true }
```

### Network

```
GET  /network/status              → NetStatus (includes `mode`)
GET  /network/wifi                → [WifiNetwork]
GET  /network/wifi/details        → ActiveWifiDetails
POST /network/wifi/scan           ← {}
POST /network/wifi/connect        ← { "ssid": "MyWifi", "password": "..." }
POST /network/wifi/disconnect     ← {}
POST /network/wifi/forget         ← { "ssid": "MyWifi" }
POST /network/power               ← { "on": true }
GET  /network/eth                 → [EthernetInterface]
GET  /network/eth/details          → ActiveEthernetDetails (?interface=enp3s0)
POST /network/eth/connect         ← { "interface": "enp3s0" }  # optional: auto-select if omitted
POST /network/eth/disconnect      ← { "interface": "enp3s0" }  # optional: auto-select active if omitted
GET  /network/hotspot/status      → HotspotStatus
POST /network/hotspot/start       ← { "ssid": "MyHotspot", "password": "...", "band": "5GHz", "channel": 36, "backend": "networkmanager" }
POST /network/hotspot/stop       ← {}
```

Hotspot backends:
- `networkmanager` — uses NM AP mode. Best for ethernet upstream or a dedicated radio. Simple, NM handles DHCP+NAT.
- `hostapd` — uses hostapd + dnsmasq + iptables with a virtual AP interface (`<iface>ap`). Required for Wi-Fi+Wi-Fi sharing on a single radio. Supports client tracking and NAT.

`HotspotStatus` additionally includes `backend`, `supports_virtual_ap` fields.

Network config (`core.toml`):

```toml
[network]
# Trigger a Wi-Fi scan on daemon startup
wifi_scan_on_start = true
# Delay before emitting wifi_scan_finished (ms)
wifi_scan_finish_delay_ms = 1000
# hotspot_backend = "networkmanager"   # auto-detected if omitted
```

### Notifications

```
GET    /notify/list      → [Notification]
POST   /notify/send      ← { "title": "...", "body": "...", "urgency": "normal" }
DELETE /notify/:id       → {}
```

### Clipboard

```
GET  /clipboard          → ClipEntry
POST /clipboard          ← { "content": "text" }
GET  /clipboard/history  → [ClipEntry]
```

### Sysmon

```
GET  /sysmon/cpu         → CpuStatus
GET  /sysmon/mem         → MemStatus
GET  /sysmon/disk        → [DiskStatus]
GET  /sysmon/net         → NetTraffic
GET  /sysmon/gpu         → GpuStatus?
```

### Brightness

```
GET  /brightness         → BrightnessStatus
POST /brightness/set     ← { "value": 80 }
POST /brightness/inc     ← { "value": 5 }
POST /brightness/dec     ← { "value": 5 }
```

### Processes

```
GET  /proc/list          → [ProcessInfo]   (?sort=cpu&top=20)
GET  /proc/find          → [ProcessInfo]   (?name=firefox)
GET  /proc/watch/:pid    → { "pid": 1234, "name": "bash", "exit_code": null }
POST /proc/:pid/kill     ← { "force": false }
```

### Power

```
GET  /power/battery      → BatteryStatus
```

### Disk

```
GET  /disk/list          → [BlockDevice]
POST /disk/mount         ← { "device": "/org/freedesktop/UDisks2/block_devices/sdb1" }
POST /disk/unmount       ← { "device": "..." }
POST /disk/eject         ← { "device": "..." }
```

## SSE event stream

```
GET /events
Content-Type: text/event-stream
```

Each event is a JSON object with `domain` and `data` fields:

```
data: {"domain":"sysmon","data":{"event":"cpu_update","cpu":{"aggregate":34.2,...}}}

data: {"domain":"bluetooth","data":{"event":"device_connected","device":{...}}}

data: {"domain":"notify","data":{"event":"new","notification":{...}}}

data: {"domain":"power","data":{"event":"battery_update","status":{...}}}

: keep-alive
```

**All domains and their events:**

| Domain | Events |
|---|---|
| `bluetooth` | `device_discovered`, `device_connected`, `device_disconnected`, `device_removed`, `adapter_powered`, `scan_started`, `scan_stopped` |
| `network` | `connected`, `disconnected`, `ip_changed`, `wifi_enabled`, `wifi_disabled`, `wifi_scan_started`, `wifi_scan_finished`, `wifi_list_updated`, `active_wifi_details_changed`, `ethernet_interfaces_changed`, `active_ethernet_details_changed`, `connectivity_changed`, `mode_changed`, `hotspot_started`, `hotspot_stopped`, `hotspot_status_changed`, `hotspot_client_joined`, `hotspot_client_left` |
| `notify` | `new`, `closed`, `action_invoked`, `replaced` |
| `clipboard` | `changed`, `primary_changed` |
| `sysmon` | `cpu_update`, `mem_update`, `net_update`, `gpu_update`, `cpu_spike`, `mem_pressure` |
| `brightness` | `changed` |
| `proc` | `spawned`, `exited` |
| `power` | `battery_update`, `ac_connected`, `ac_disconnected`, `low_battery`, `critical` |
| `disk` | `device_mounted`, `device_unmounted`, `device_added`, `device_removed` |
| `daemon` | `started`, `stopping`, `domain_error` |

Consume with curl:

```bash
curl --no-buffer --unix-socket $XDG_RUNTIME_DIR/crawlds.sock \
     http://localhost/events
```

Filter a single domain:

```bash
curl --no-buffer --unix-socket $XDG_RUNTIME_DIR/crawlds.sock \
     http://localhost/events | grep '"domain":"power"'
```
