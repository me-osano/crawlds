# CrawlDS Network

This document covers all network management functionality in CrawlDS: Wi-Fi, Ethernet, and hotspot services. The implementation lives in `core/crates/crawlds-network/src/lib.rs` and communicates with the system via NetworkManager D-Bus.

**Architecture overview:**

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  QML Frontend   │────▶│  crawlds-daemon  │────▶│ NetworkManager  │
│ (NetworkService)│◀────│   (HTTP API)     │◀────│   D-Bus API     │
└─────────────────┘     └──────────────────┘     └─────────────────┘
        │                        │                        │
        │ SSE                    │ SSE                    │ D-Bus
        └────────────────────────┘                        │
                                                              │
        ┌─────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────┐     ┌─────────────────┐
│ hostapd + dnsmasq (optional, for hotspot Wi-Fi+Wi-Fi sharing)
└─────────────────────────────────────────────────────────────┘
```

---

## Table of Contents

1. [Wi-Fi](#wi-fi)
   - [Scan and list networks](#scan-and-list-networks)
   - [Connect and disconnect](#connect-and-disconnect)
   - [Forget a saved network](#forget-a-saved-network)
   - [Active Wi-Fi details](#active-wi-fi-details)
2. [Ethernet](#ethernet)
   - [List interfaces](#list-interfaces)
   - [Interface details](#interface-details)
   - [Connect and disconnect](#ethernet-connectdisconnect)
3. [Hotspot](#hotspot)
   - [Backends](#hotspot-backends)
   - [Backend selection logic](#backend-selection-logic)
   - [Virtual interface support](#virtual-interface-support)
   - [Client tracking](#client-tracking)
   - [NAT and IP forwarding](#nat-and-ip-forwarding)
   - [Config options](#hotspot-config-options)
4. [API Reference](#api-reference)
   - [HTTP endpoints](#http-endpoints)
   - [SSE events](#sse-events)
5. [Architecture notes](#architecture-notes)
   - [TTL caching and background refresh](#ttl-caching-and-background-refresh)
   - [Scan delay and stale result handling](#scan-delay-and-stale-result-handling)
   - [File caching (QML)](#file-caching-qml)
   - [Session resume refresh](#session-resume-refresh)
6. [NM vs hostapd+dnsmasq](#nm-vs-hostapddnsmasq)
7. [Roadmap](#roadmap)

---

## Wi-Fi

All Wi-Fi operations are performed over the system D-Bus connection to NetworkManager (`org.freedesktop.NetworkManager`).

### Scan and list networks

A Wi-Fi scan triggers a background NM scan request. A configurable delay (`wifi_scan_finish_delay_ms`, default 0ms) is applied before emitting the `wifi_scan_finished` event to ensure scan results have propagated through NM's internal state. Results are merged with known/saved profiles to populate the `existing` and `cached` fields.

**Endpoints:**

```
GET  /network/wifi              → [WifiNetwork]
POST /network/wifi/scan         → { ok: true }
GET  /network/wifi/details      → ActiveWifiDetails
```

**Process:**
1. `POST /network/wifi/scan` calls `nmcli device wifi rescan` via shell
2. A background timer waits `wifi_scan_finish_delay_ms`
3. `wifi_scan_started` → `wifi_scan_finished` SSE events are emitted
4. `list_wifi()` queries all visible APs from NM D-Bus (`org.freedesktop.NetworkManager.Device.Wireless`)
5. `list_known_wifi_ssids()` queries saved profiles from NM Settings (`org.freedesktop.NetworkManager.Settings`)
6. Each `WifiNetwork` gets `existing=true` if it matches a saved profile SSID, and `cached=true` if it was in a previous scan result (for hidden networks that drop off the active scan list)

**`WifiNetwork` fields:**

| Field | Type | Description |
|---|---|---|
| `ssid` | `String` | Network name |
| `signal` | `u8` | Signal strength (0-100) |
| `secured` | `bool` | Whether the network requires auth |
| `connected` | `bool` | Currently connected |
| `existing` | `bool` | SSID matches a saved NM profile |
| `cached` | `bool` | Seen in a previous scan (hidden network fallback) |
| `password_required` | `bool` | Network needs a password to connect |
| `security` | `String` | Security type (e.g. `wpa2-psk`, `wpa3-sae`, `open`) |
| `frequency_mhz` | `Option<u32>` | Frequency in MHz |
| `bssid` | `Option<String>` | AP MAC address |
| `last_seen_ms` | `Option<u64>` | Unix timestamp ms of last scan result |

### Connect and disconnect

```
POST /network/wifi/connect    ← { ssid, password? }
POST /network/wifi/disconnect
```

Connecting to an open network does not save the profile. Connecting to a secured network auto-saves the profile to NM. Hidden networks are supported via the `hidden: true` flag passed to NM.

### Forget a saved network

```
POST /network/wifi/forget    ← { ssid }
```

Queries all NM saved connections via D-Bus, matches by SSID, and calls `Delete()` on the `org.freedesktop.NetworkManager.Settings.Connection` object. This removes the profile from both NM and `wpa_supplicant`.

### Active Wi-Fi details

```
GET /network/wifi/details    → ActiveWifiDetails
```

Queries the active Wi-Fi device's `ActiveConnection`, extracts IPv4/IPv6 config from `org.freedesktop.NetworkManager.IP4Config` and `IP6Config` (addresses, gateways, DNS servers), reads the current AP frequency to derive band/channel, and reads link rate. Falls back to `/sys/class/net/<ifname>/speed` for link speed when NM doesn't report it.

**`ActiveWifiDetails` fields:**

| Field | Type | Description |
|---|---|---|
| `ifname` | `Option<String>` | Interface name (e.g. `wlan0`) |
| `ssid` | `Option<String>` | Current SSID |
| `signal` | `Option<u8>` | Signal strength 0-100 |
| `frequency_mhz` | `Option<u32>` | Current frequency |
| `band` | `Option<String>` | `"2.4"` or `"5"` |
| `channel` | `Option<u32>` | Current channel |
| `rate_mbps` | `Option<u32>` | Link rate in Mbps |
| `ip4` | `Option<String>` | IPv4 address with mask (e.g. `192.168.1.10/24`) |
| `ip6` | `Vec<String>` | IPv6 addresses |
| `gateway4` | `Option<String>` | IPv4 default gateway |
| `gateway6` | `Vec<String>` | IPv6 default gateways |
| `dns4` | `Vec<String>` | IPv4 DNS servers |
| `dns6` | `Vec<String>` | IPv6 DNS servers |
| `security` | `Option<String>` | Current AP security type |
| `bssid` | `Option<String>` | Current AP MAC |
| `mac` | `Option<String>` | Station MAC address |

---

## Ethernet

### List interfaces

```
GET /network/eth    → [EthernetInterface]
```

Queries all NM wired devices, returns their names, MAC addresses, link state, and IP addresses.

### Interface details

```
GET /network/eth/details?interface=eth0    → ActiveEthernetDetails
```

Extracts full IPv4/IPv6 config (addresses, gateways, DNS) and link speed from the NM wired device. Speed fallback reads `/sys/class/net/<ifname>/speed` as a signed integer and converts to Mbps string.

**`ActiveEthernetDetails` fields:**

| Field | Type | Description |
|---|---|---|
| `ifname` | `String` | Interface name |
| `speed` | `Option<String>` | Link speed (e.g. `"1000 Mbps"`) |
| `ipv4` | `Option<String>` | IPv4 address with mask |
| `ipv6` | `Vec<String>` | IPv6 addresses |
| `gateway4` | `Option<String>` | IPv4 default gateway |
| `gateway6` | `Vec<String>` | IPv6 default gateways |
| `dns4` | `Vec<String>` | IPv4 DNS servers |
| `dns6` | `Vec<String>` | IPv6 DNS servers |
| `mac` | `Option<String>` | Interface MAC address |

### Ethernet connect/disconnect

```
POST /network/eth/connect    ← { interface? }
POST /network/eth/disconnect ← { interface? }
```

Brings NM wired connections up or down by interface name.

---

## Hotspot

CrawlDS supports two hotspot backends: **NetworkManager (NM)** and **hostapd + dnsmasq**. The choice depends on the upstream internet source and hardware capabilities.

### Hotspot backends

#### Path A: NetworkManager (default)

NM provides a built-in Wi-Fi AP mode via the `wifi-p2p` connection type or by creating a shared connection profile. This path is simple, requires no external binaries, and is managed entirely through D-Bus.

**Process:**
1. Query available Wi-Fi devices
2. Create a new `802-11-wireless` connection profile with `mode=ap`
3. Set the SSID and (optionally) WPA2 password
4. If `connection.autoconnect=true` and upstream is ethernet, set `connection.gateway-ping-timeout=0` to prevent interference
5. Activate the connection

**Requires:** NetworkManager with Wi-Fi AP support (most distributions support this)
**Best for:** Ethernet upstream, single-radio Wi-Fi card, simplicity

#### Path B: hostapd + dnsmasq

For simultaneous Wi-Fi client + Wi-Fi AP sharing, NM's built-in AP mode is insufficient because most consumer Wi-Fi cards can only operate in one mode at a time. The hostapd path creates a **virtual AP interface** (e.g. `wlan0ap`) using the `iw` tool's `type __ap` directive, then runs `hostapd` and `dnsmasq` as separate processes.

**Process:**
1. Detect upstream type (ethernet vs Wi-Fi client)
2. Detect the Wi-Fi physical interface (`phy`) name via `iw dev <iface> info`
3. Check virtual AP support via `iw dev <phy> info | grep "type AP"`
4. Optionally create a virtual interface `iw dev <iface> interface add <iface>ap type __ap`
5. Mark the virtual (or real) interface as unmanaged by NM to prevent interference
6. Assign IP `10.10.10.1/24` to the AP interface
7. Write and spawn `hostapd` with config at `/run/crawlds/hostapd.conf`
8. Write and spawn `dnsmasq` with config at `/run/crawlds/dnsmasq.conf` (DHCP range `10.10.10.10-250`, gateway `10.10.10.1`)
9. Enable IPv4 forwarding and set up iptables NAT/MASQUERADE on the upstream interface
10. Start background monitor loop to track connected clients

**Requires:** `hostapd`, `dnsmasq`, `iw`, `iptables`, `ip` utilities
**Best for:** Wi-Fi upstream (tethering from a phone or another network), simultaneous client+AP mode

### Backend selection logic

`start_hotspot()` auto-selects the backend based on upstream type:

```
if upstream interface starts with "en" or "eth"  → NetworkManager
else if upstream is Wi-Fi client (e.g. "wlan0")  → hostapd
```

The backend can be overridden per-request via `HotspotConfig.backend` or globally via `core.toml`:

```toml
[network]
hotspot_backend = "networkmanager"  # or "hostapd"
hotspot_virtual_iface = true         # create virtual AP for simultaneous client+AP
```

### Virtual interface support

The `supports_virtual_ap()` check queries the phy capabilities:

```bash
iw dev <phy> info | grep "type AP"
```

If this returns output, the driver/firmware supports virtual AP interfaces. Most Intel, MediaTek, and recent Realtek cards support this. Older or more exotic hardware may not.

When `hotspot_virtual_iface = false`, the hostapd path uses the same interface for both client and AP — this will disconnect the client connection.

### Client tracking

For the NM backend, connected clients are read from `/proc/sys/kernel/debug/ieee80211/<phy>/stations` (debugfs). For the hostapd backend, the same debugfs path is used, keyed by the virtual interface name.

Each client record contains:
- `mac`: Client MAC address
- `ip`: IP address (if assigned via DHCP, resolved from ARP table)

A background monitor loop polls every 10 seconds and emits `hotspot_client_joined` / `hotspot_client_left` events.

### NAT and IP forwarding

Both backends enable IPv4 packet forwarding (`/proc/sys/net/ipv4/ip_forward`) and add iptables NAT rules:

```bash
iptables -t nat -A POSTROUTING -o <upstream_iface> -j MASQUERADE
iptables -A FORWARD -i <ap_iface> -o <upstream_iface> -j ACCEPT
iptables -A FORWARD -i <upstream_iface> -o <ap_iface> -m state --state RELATED,ESTABLISHED -j ACCEPT
```

IPv6 forwarding is **not** currently configured.

### Hotspot config options

```toml
[network]
# hotspot_backend: preferred backend
#   "networkmanager" — simple, no external deps, requires ethernet upstream
#   "hostapd"       — supports wifi+wifi sharing, requires hostapd + dnsmasq
hotspot_backend = "networkmanager"

# hotspot_virtual_iface: create virtual AP interface for simultaneous client+AP
#   true  — use a virtual interface (wlan0ap), client stays connected
#   false — use the same interface, client disconnects
hotspot_virtual_iface = true
```

**`HotspotConfig` (HTTP API):**

| Field | Type | Required | Description |
|---|---|---|---|
| `ssid` | `String` | Yes | Hotspot network name |
| `password` | `Option<String>` | No | WPA2 password (if omitted, open network) |
| `iface` | `Option<String>` | No | Specific Wi-Fi interface (auto-detected if omitted) |
| `band` | `Option<String>` | No | `"2.4"` or `"5"` (auto-detected from channel) |
| `channel` | `Option<u32>` | No | Specific channel (auto-detected if omitted) |
| `backend` | `Option<HotspotBackend>` | No | `"networkmanager"` or `"hostapd"` (global config fallback) |

**`HotspotStatus` (response):**

| Field | Type | Description |
|---|---|---|
| `active` | `bool` | Hotspot is running |
| `ssid` | `Option<String>` | Current SSID |
| `iface` | `Option<String>` | AP interface name |
| `band` | `Option<String>` | `"2.4"` or `"5"` |
| `channel` | `Option<u32>` | Current channel |
| `clients` | `Vec<HotspotClient>` | Connected clients |
| `backend` | `HotspotBackend` | Which backend is active |
| `supports_virtual_ap` | `bool` | Whether hardware supports virtual AP |

---

## API Reference

### HTTP endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/network/status` | yes | Overall network status |
| `GET` | `/network/wifi` | yes | List visible + known Wi-Fi networks |
| `GET` | `/network/wifi/details` | yes | Active Wi-Fi connection details |
| `POST` | `/network/wifi/scan` | yes | Trigger Wi-Fi scan |
| `POST` | `/network/wifi/connect` | yes | Connect to Wi-Fi network |
| `POST` | `/network/wifi/disconnect` | yes | Disconnect Wi-Fi |
| `POST` | `/network/wifi/forget` | yes | Remove a saved Wi-Fi profile |
| `POST` | `/network/power` | yes | Enable/disable all network |
| `GET` | `/network/eth` | yes | List Ethernet interfaces |
| `GET` | `/network/eth/details?interface=…` | yes | Ethernet interface details |
| `POST` | `/network/eth/connect` | yes | Bring up Ethernet interface |
| `POST` | `/network/eth/disconnect` | yes | Bring down Ethernet interface |
| `POST` | `/network/hotspot/start` | yes | Start hotspot |
| `POST` | `/network/hotspot/stop` | yes | Stop hotspot |
| `GET` | `/network/hotspot/status` | yes | Get hotspot status |

All endpoints return JSON. Errors use the envelope:

```json
{
  "error": {
    "domain": "network",
    "code": "network_error",
    "message": "Human-readable description"
  }
}
```

### SSE events

Subscribe to `GET /events` with `?domain=network`. Events use the format:

```json
{
  "domain": "network",
  "event": "event_name",
  "data": { ... }
}
```

**All network events:**

| Event | When |
|---|---|
| `connected` | Wi-Fi connected to an AP |
| `disconnected` | Wi-Fi disconnected |
| `ip_changed` | IP address changed |
| `wifi_enabled` | Wi-Fi radio enabled |
| `wifi_disabled` | Wi-Fi radio disabled |
| `wifi_scan_started` | Wi-Fi scan initiated |
| `wifi_scan_finished` | Wi-Fi scan completed |
| `wifi_list_updated` | New/updated network list available |
| `active_wifi_details_changed` | Active Wi-Fi details updated |
| `ethernet_interfaces_changed` | Ethernet interface list changed |
| `active_ethernet_details_changed` | Ethernet details updated |
| `mode_changed` | Device mode changed (station/AP) |
| `connectivity_changed` | Internet connectivity state changed |
| `hotspot_started` | Hotspot activation succeeded |
| `hotspot_stopped` | Hotspot stopped |
| `hotspot_status_changed` | Periodic hotspot status poll |
| `hotspot_client_joined` | Client associated to hotspot |
| `hotspot_client_left` | Client disassociated from hotspot |

---

## Architecture notes

### TTL caching and background refresh

The network backend uses time-to-live (TTL) caching to prevent hammering NM with rapid requests:

| Data | TTL | Behavior |
|---|---|---|
| Wi-Fi list | 5s | 5s after expiry, refresh on next request |
| Wi-Fi details | 5s | 5s after expiry, refresh on next request |
| Ethernet details | 5s | 5s after expiry, refresh on next request |

Every 5 seconds, a background tick fetches the latest state and broadcasts SSE events if data changed. Individual API calls also check TTL and skip the NM query if cache is fresh.

### Scan delay and stale result handling

The `wifi_scan_finish_delay_ms` config (default: 0) adds a delay between triggering the scan and emitting `wifi_scan_finished`. This compensates for NM's internal propagation delay. A value of 500-1000ms is recommended for more reliable result delivery.

The QML frontend (`NetworkService.qml`) uses a `scanDelayTimer` to ignore stale scan results: when `wifi_scan_started` arrives, it marks `scanPending = true` and ignores any `wifi_list_updated` that arrived before the corresponding `wifi_scan_finished`.

### File caching (QML)

`NetworkService.qml` persists the Wi-Fi network list to disk at `Settings.cacheDir + "network.json"` using QML's `FileView` and `JsonAdapter`. This enables instant network display on startup without waiting for a scan. On session resume, the cache is restored and a fresh scan is triggered.

### Session resume refresh

`SessionService.sessionResumed` triggers a full network state refresh:
- `refreshEthernet()`
- `refreshNetworks()` (full scan)
- `refreshActiveWifiDetails()`
- `refreshActiveEthernetDetails()`
- `refreshHotspotStatus()`

This ensures all network state is current after the device wakes from sleep or the session is unlocked.

---

## NM vs hostapd+dnsmasq

| Aspect | NetworkManager | hostapd + dnsmasq |
|---|---|---|
| **Upstream** | Ethernet | Ethernet or Wi-Fi |
| **Simultaneous client+AP** | No (single radio) | Yes (virtual interface) |
| **Dependencies** | None extra | `hostapd`, `dnsmasq`, `iw`, `iptables` |
| **Complexity** | Low (D-Bus only) | High (processes, NAT, virtual ifaces) |
| **NM integration** | Native | NM must mark interface unmanaged |
| **Client tracking** | Debugfs `ieee80211/<phy>/stations` | Same |
| **DHCP server** | NM shared connection | dnsmasq (custom range/gateway) |
| **DNS redirection** | NM shared connection | dnsmasq `address=/#/10.10.10.1` |
| **IPv6 support** | Built-in | Not configured |
| **Band/channel control** | Limited | Full hostapd control |
| **WPA3 support** | Yes | Yes |
| **Requires sudo/polkit** | For connection changes | For virtual iface creation + iptables |
| **Config persistence** | NM profile | Config file at `/run/crawlds/` |
| **Status detection** | Queries NM active connection | Checks for `hostapd` process + virtual iface |

**Recommendation:**
- **Use NM** for the simplest setup with ethernet upstream
- **Use hostapd** when you need Wi-Fi+Wi-Fi sharing (e.g. tethering from a phone) or when you need fine-grained control over AP settings

---

## Roadmap

The following items are identified but not yet implemented:

### Compilation & correctness

- [ ] **`OwnedValue::Str` downcasting** — `list_known_wifi_ssids()` uses `OwnedValue::Str` for D-Bus value matching. Verify the zvariant `Str` type is correctly imported and matched; on older zvariant versions this may need `OwnedValue::U8` or a different variant
- [ ] **`supports_virtual_ap()` on real hardware** — The check uses `iw dev <phy> info | grep "type AP"` which may need verification; some drivers report "type AP" only after the virtual interface is created, not before

### Hotspot

- [ ] **SSID reading from hostapd** — `hotspot_status()` for the hostapd path returns `ssid: None` because hostapd doesn't expose the SSID back. Can be resolved by parsing `iw dev <iface> info`
- [ ] **IPv6 NAT** — `setup_ip_forward()` only configures IPv4 MASQUERADE; IPv6 forwarding may be needed for dual-stack upstream connections
- [ ] **Per-client bandwidth limits** — hostapd supports `wmm_ac_be` / `wmm_ac_bk` / `wmm_ac_vi` / `wmm_ac_vo` access classes; possible via `tc` (traffic control) in the hostapd path for per-device QoS
- [ ] **NM conflict detection for hostapd path** — `nm_mark_unmanaged` uses `nmcli device set ... managed no` which may need polkit authorization; should fall back to writing `/etc/NetworkManager/conf.d/unmanaged.conf`
- [ ] **Runtime dependency declaration** — `hostapd` and `dnsmasq` should be listed as runtime dependencies in packaging (AUR, PKGBUILD, etc.)
- [ ] **Channel auto-selection improvements** — Currently defaults to channel 1; could implement DFS channel scanning for 5GHz to avoid occupied channels
- [ ] **Band steering** — Automatically move clients between 2.4GHz and 5GHz based on signal strength
- [ ] **Hotspot MAC filtering** — Allow/deny specific MAC addresses

### Network (general)

- [ ] **Error handling improvements** — Many functions use `unwrap_or` and silent failures that should surface meaningful errors to the caller
- [ ] **NM unavailable handling** — Consider returning HTTP 503 when NetworkManager D-Bus is not available
- [ ] **Precise HTTP status codes** — Currently all network errors return 400; map `NetError` variants to appropriate codes (404 for not found, 503 for service unavailable, etc.)
- [ ] **Hidden network support improvements** — Ensure `hidden` flag is consistently passed to NM for both connect and scan operations
- [ ] **Enterprise Wi-Fi (802.1X)** — Full support for WPA-Enterprise with EAP methods, CA certificates, anonymous identity
- [ ] **Wi-Fi Direct / P2P** — NM's Wi-Fi P2P mode for direct device-to-device connections
- [ ] **VPN integration** — Connect/disconnect/manages VPNs through NM's `org.freedesktop.NetworkManager.VPN.Plugin` interface
- [ ] **Bond/bridge support** — Create and manage network bonds and bridges through NM

### QML UI

- [ ] **Hotspot settings in dialog** — Add band, channel, and backend selection to the hotspot start dialog in `NetworkPanel.qml`
- [ ] **Forget network hidden-network edge case** — `NetworkService.forget()` correctly removes the network from NM and the local `networks` dict (keyed by SSID). However, a hidden network that was only in the cached scan list (not in the current scan) won't appear in the cache after forgetting since the cache is only updated on successful scan results. Consider clearing the `cached` field for the forgotten SSID on the next scan
- [ ] **Hotspot QR code** — Generate a scannable QR code for the hotspot password
- [ ] **Hotspot clients list improvements** — Show device names (via DHCP hostname), IP addresses, connection duration
- [ ] **Network quality indicators** — Visual feedback for signal degradation, channel congestion, interference
