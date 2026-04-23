# CrawlDS Daemon

The CrawlDS daemon (`crawlds-daemon`) is the central system service that manages all desktop services including Bluetooth, network, clipboard, system monitoring, themes, and more. It runs as a long-lived background process and exposes functionality via JSON-RPC 2.0 over a Unix socket.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          crawlds-daemon                                   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                         Configuration                                │ │
│  │                    (core.toml / environment)                      │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                        AppState                                       │ │
│  │   • config              (merged config with defaults)                │ │
│  │   • event_tx            (broadcast channel for events)               │ │
│  │   • notify_store        (notification history)                      │ │
│  │   • clipboard_store      (clipboard history with pinning)           │ │
│  │   • webservice_store    (RSS feeds, Wallhaven state)                │ │
│  │   • greeter             (GreeterManager for greetd)                  │ │
│  │   • theme_manager       (ThemeManager for theming)                   │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                     Domain Tasks (async)                              │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │ │
│  │  │bluetooth │ │ network  │ │clipboard │ │  sysmon  │ │  display │ │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐            │ │
│  │  │   proc   │ │  power   │ │  notify  │ │webservice│            │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘            │ │
│  │                                                                      │ │
│  │  Each domain:                                                         │ │
│  │  • Has its own config section                                        │ │
│  │  • Runs as independent tokio task                                    │ │
│  │  • Publishes events via event_tx                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                  JSON-RPC 2.0 Server (Unix Socket)                  │ │
│  │                                                                      │ │
│  │   Unix Socket                      TCP Bridge (optional)             │ │
│  │   /run/user/<uid>/crawlds.sock    127.0.0.1:9280                   │ │
│  │                                                                      │ │
│  │   Protocol: JSON-RPC 2.0                                            │ │
│  │   Events: NDJSON (after Subscribe)                                  │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                     ┌────────────────┼────────────────┐
                     ▼                ▼                ▼
           ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
           │   CLI       │  │  Quickshell │  │   Scripts   │
           │ crawlds    │  │ CrawlDSService │  │    ...     │
           └─────────────┘  └─────────────┘  └─────────────┘
```

## Module Structure

```
core/crates/crawlds-daemon/src/
├── main.rs          # Entry point, domain spawning, socket binding
├── config.rs        # Configuration loading (figment)
├── state.rs         # AppState, ClipboardStore, WebserviceState, ThemeManager
└── json_server.rs   # JSON-RPC 2.0 command handlers
```

> **Note**: Greeter functionality has been moved to `crawlds-greeter` crate.
> The daemon now imports `crawlds_greeter::{GreeterManager, GreeterSession}` for greetd integration.

### main.rs

The entry point handles:

1. **Logging setup** - Configures tracing with `EnvFilter`
2. **Config loading** - Loads `core.toml` with environment variable overrides
3. **Broadcast channel** - Creates `event_tx` for domain communication (buffer size: 100)
4. **AppState creation** - Initializes all shared state including stores and managers
5. **Domain spawning** - Starts all domain tasks (bluetooth, network, clipboard, sysmon, display, proc, power, notify, webservice, idle)
6. **Socket binding** - Binds Unix socket for JSON-RPC connections
7. **JSON-RPC server** - Accepts connections and processes commands

### config.rs

Configuration loading using figment:

```rust
pub fn load() -> anyhow::Result<Config> {
    // Config sources (in order of precedence):
    // 1. Environment variables (CRAWLDS_*)
    // 2. $XDG_CONFIG_HOME/crawlds/core.toml
    // 3. Defaults
    //
    // Also sets default assets_dir and cache_dir if not configured
}
```

### json_server.rs

JSON-RPC 2.0 server with command handlers:
- Uses `UnixListener` for socket connections
- Each line is a JSON-RPC request, each response is a line
- Events delivered as NDJSON after `Subscribe` command

### state.rs

Shared application state:

```rust
pub struct AppState {
    pub config: Arc<Config>,              // Full merged config
    pub event_tx: broadcast::Sender<CrawlEvent>,  // Event broadcaster
    pub notify_store: Arc<NotifyStore>,   // Notification history
    pub greeter: Arc<Mutex<GreeterManager>>, // greetd sessions
    pub clipboard_store: Arc<ClipboardStore>, // Clipboard with pinning
    pub webservice_store: Arc<WebserviceState>, // RSS/Wallhaven state
    pub theme_manager: Arc<Mutex<ThemeManager>>, // Theme management
}
```

Also includes `ClipboardStore` for clipboard history with pinning support.

### Greeter Integration

The daemon uses `crawlds-greeter` crate for greetd integration:

```rust
use crawlds_greeter::{GreeterManager, GreeterSession};

pub struct AppState {
    // ...
    pub greeter: Arc<Mutex<GreeterManager>>,
}
```

See [GREETER.md](GREETER.md) for full documentation.

### Theme Manager

The daemon includes `ThemeManager` for dynamic theming:

```rust
pub struct AppState {
    // ...
    pub theme_manager: Arc<Mutex<ThemeManager>>,
}
```

Themes are loaded from `$assets_dir/Themes` and cached in `$cache_dir`.

## Domain Tasks

All domains run as independent `tokio::spawn` tasks:

| Domain | Crate | Description |
|--------|-------|-------------|
| bluetooth | `crawlds-bluetooth` | Bluetooth device management |
| network | `crawlds-network` | WiFi, ethernet, hotspot |
| clipboard | `crawlds-clipboard` | Clipboard monitoring |
| sysmon | `crawlds-sysmon` | CPU, memory, disk, network |
| display | `crawlds-display` | Brightness control |
| proc | `crawlds-proc` | Process listing/killing |
| power | `crawlds-power` | Battery, power profiles, idle |
| notify | `crawlds-notify` | Notifications |
| webservice | `crawlds-webservice` | RSS, Wallhaven |

### Spawning Pattern

```rust
async fn spawn_domains(state: &Arc<AppState>) {
    let tx = state.event_tx.clone();
    let cfg = state.config.clone();

    tokio::spawn(crawlds_bluetooth::run(cfg.bluetooth.clone(), tx.clone()));
    tokio::spawn(crawlds_network::run(cfg.network.clone(), tx.clone()));
    // ... each domain as independent task
}
```

### Domain Run Signature

Each domain crate exports:

```rust
pub async fn run(
    cfg: DomainConfig,
    tx: broadcast::Sender<CrawlEvent>
) -> anyhow::Result<()>
```

The domain:
1. Reads its config section
2. Sets up internal state
3. Loops, emitting events via `tx.send()`
4. Handles requests via shared state

## JSON-RPC API Reference

### Protocol

JSON-RPC 2.0 over Unix socket. Each request is a JSON object per line:

```json
{"jsonrpc": "2.0", "method": "CommandName", "params": {...}, "id": 1}
```

Response:

```json
{"jsonrpc": "2.0", "result": {...}, "id": 1}
```

### Socket Path

```
$ XDG_RUNTIME_DIR/crawlds.sock
```

### Commands

#### System
| Command | Params | Description |
|---------|--------|-------------|
| `Hello` | `{client?, version?}` | Version handshake |
| `Ping` | - | Keep-alive |
| `Get` | `{key?}` | Get config value |
| `Set` | `{key, value}` | Set config value |
| `Health` | - | Health check |
| `Subscribe` | - | Subscribe to events |

#### Theme
| Command | Params | Description |
|---------|--------|-------------|
| `ThemeList` | - | List themes |
| `ThemeCurrent` | - | Get current theme |
| `ThemeSet` | `{name}` | Set theme |
| `ThemeGet` | `{name?}` | Get theme details |

#### System Monitor
| Command | Params | Description |
|---------|--------|-------------|
| `SysmonCpu` | - | CPU status |
| `SysmonMem` | - | Memory status |
| `SysmonDisk` | - | Disk info |
| `SysmonNet` | - | Network traffic |
| `SysmonGpu` | - | GPU status |

#### Power
| Command | Params | Description |
|---------|--------|-------------|
| `PowerBattery` | - | Battery status |
| `PowerProfileGet` | - | Get profile |
| `PowerProfileSet` | `{profile}` | Set profile |
| Command | Params | Description |
|---------|--------|-------------|
| `NetStatus` | - | Network status |
| `NetWifiList` | - | List WiFi networks |
| `NetWifiDetails` | - | Current WiFi details |
| `NetWifiScan` | - | Scan for networks |
| `NetWifiConnect` | `{ssid, password?}` | Connect to WiFi |
| `NetWifiDisconnect` | - | Disconnect WiFi |
| `NetWifiForget` | `{ssid}` | Forget network |
| `NetPower` | `{enabled}` | WiFi on/off |
| `NetEthList` | - | List ethernet |
| `NetEthConnect` | `{interface}` | Connect ethernet |
| `NetEthDisconnect` | - | Disconnect ethernet |
| `NetEthDetails` | `{interface}` | Ethernet details |
| `NetHotspotStart` | - | Start hotspot |
| `NetHotspotStop` | - | Stop hotspot |
| `NetHotspotStatus` | - | Hotspot status |

#### Bluetooth
| Command | Params | Description |
|---------|--------|-------------|
| `BtStatus` | - | Adapter status |
| `BtDevices` | - | List devices |
| `BtScan` | - | Start discovery |
| `BtConnect` | `{address}` | Connect device |
| `BtDisconnect` | `{address}` | Disconnect device |
| `BtPower` | `{enabled}` | Power on/off |
| `BtPair` | `{address}` | Pair device |
| `BtRemove` | `{address}` | Remove device |
| `BtDiscoverable` | `{enabled}` | Set discoverable |
| `BtTrust` | `{address, trusted}` | Trust device |
| `BtAlias` | `{address, alias}` | Set device alias |
| `BtPairable` | `{enabled}` | Set pairable |

#### Notifications
| Command | Params | Description |
|---------|--------|-------------|
| `NotifyList` | - | List notifications |
| `NotifySend` | `{title, body}` | Send notification |
| `NotifyDismiss` | `{id}` | Dismiss notification |

#### Clipboard
| Command | Params | Description |
|---------|--------|-------------|
| `ClipGet` | - | Get current |
| `ClipSet` | `{text}` | Set clipboard |
| `ClipHistory` | `{limit?}` | History |
| `ClipDelete` | `{id}` | Delete entry |
| `ClipPin` | `{id}` | Pin entry |
| `ClipUnpin` | `{id}` | Unpin entry |
| `ClipPinnedCount` | - | Count pinned |
| `ClipClear` | - | Clear history |

#### Brightness
| Command | Params | Description |
|---------|--------|-------------|
| `BrightnessGet` | - | Get brightness |
| `BrightnessSet` | `{value}` | Set brightness |
| `BrightnessInc` | `{value}` | Increase |
| `BrightnessDec` | `{value}` | Decrease |

#### Processes
| Command | Params | Description |
|---------|--------|-------------|
| `ProcList` | `{sort?, top?}` | List processes |
| `ProcFind` | `{name}` | Find by name |
| `ProcKill` | `{pid, force}` | Kill process |
| `ProcWatch` | `{pid}` | Watch exit |

#### VFS
| Command | Params | Description |
|---------|--------|-------------|
| `VfsList` | `{path?}` | List directory |
| `VfsSearch` | `{query}` | Search home |
| `VfsMkdir` | `{path}` | Create directory |
| `VfsDelete` | `{path}` | Delete file |
| `VfsCopy` | `{from, to}` | Copy files |
| `VfsMove` | `{from, to}` | Move files |
| `VfsRename` | `{path, name}` | Rename file |
| `VfsTrash` | `{path}` | Move to trash |
| `VfsDiskUsage` | - | Disk usage |
| `DiskList` | - | List devices |
| `DiskMount` | `{path}` | Mount device |
| `DiskUnmount` | `{path}` | Unmount |
| `DiskEject` | `{path}` | Eject |

#### Idle
| Command | Params | Description |
|---------|--------|-------------|
| `IdleStatus` | - | Idle status |
| `IdleActivity` | - | Reset idle time |
| `IdleInhibit` | `{why}` | Inhibit idle |
| `IdleUninhibit` | `{id}` | Uninhibit idle |

#### Greeter
| Command | Params | Description |
|---------|--------|-------------|
| `GreeterStatus` | - | Session status |
| `GreeterSession` | `{username?}` | Create session |
| `GreeterLaunch` | - | Launch session (deprecated) |
| `GreeterRespond` | `{response}` | Auth response |
| `GreeterCancel` | - | Cancel session |
| `GreeterCreate` | - | Create new session |
| `GreeterPamInfo` | - | PAM info |
| `GreeterExternalAuth` | `{user, auth}` | External auth |

#### Webservice (RSS/Wallhaven)
| Command | Params | Description |
|---------|--------|-------------|
| `RssFeeds` | - | List feeds |
| `RssAdd` | `{url}` | Add feed |
| `RssRemove` | `{url}` | Remove feed |
| `RssRefresh` | - | Refresh feeds |
| `WallhavenSearch` | `{query}` | Search wallpapers |
| `WallhavenRandom` | - | Random wallpapers |

## Configuration

### Config File Location

```
$XDG_CONFIG_HOME/crawlds/core.toml
~/.config/crawlds/core.toml
```

### Environment Variables

All config can be overridden with `CRAWLDS_` prefix:

```bash
# Example environment overrides
CRAWLDS__DAEMON__LOG_LEVEL=debug
CRAWLDS__DAEMON__SOCKET_PATH=/tmp/crawlds.sock
CRAWLDS__BLUETOOTH__AUTO_POWER=true
CRAWLDS__SYSMON__POLL_INTERVAL_MS=2000
```

### Configuration Sections

```toml
# Daemon settings
[daemon]
socket_path = ""        # Default: $XDG_RUNTIME_DIR/crawlds.sock
tcp_addr = ""           # Optional TCP bridge (e.g., "127.0.0.1:9280")
log_level = "info"     # trace, debug, info, warn, error

# Theme settings
[theme]
current = ""          # Current theme name
variant = ""         # Theme variant

# Greeter (greetd)
[greeter]
greetd_socket = "/run/greetd.sock"
session_ttl_secs = 60

# Idle/DPMS settings
[idle]
idle_timeout_secs = 300
dim_timeout_secs = 60
sleep_timeout_secs = 600
screen_off_timeout_secs = 600
lock_timeout_secs = 660
suspend_timeout_secs = 1800
fade_duration_secs = 5
screen_off_command = ""   # Command to turn off screen
lock_command = ""       # Command to lock screen
suspend_command = ""    # Command to suspend

# Domain configs...
[bluetooth]
[network]
[notifications]
[clipboard]
[sysmon]
[brightness]
[processes]
[power]
[vfs]
[webservice]
```

### Default Config Generation

See `core/config/core.toml` for all available options with documentation.

## Socket Communication

### Unix Socket

Default path:
```
$XDG_RUNTIME_DIR/crawlds.sock
```

Fallback (if `XDG_RUNTIME_DIR` not set):
```
/run/user/<uid>/crawlds.sock
```

### TCP Bridge (Optional)

Enable in config:
```toml
[daemon]
tcp_addr = "127.0.0.1:9280"
```

### Permissions

The socket is created with permissions allowing access to the user:
```bash
srwxr-xr-x 1 enosh enosh 0 Apr 16 08:00 /run/user/1000/crawlds.sock
```

## Running the Daemon

### Systemd Service

```ini
[Unit]
Description=CrawlDS Desktop Service
After=network.target

[Service]
Type=simple
ExecStart=~/.local/bin/crawlds-daemon
Environment=RUST_LOG=info
Restart=on-failure

[Install]
WantedBy=default.target
```

```bash
systemctl --user enable crawlds-daemon
systemctl --user start crawlds-daemon
```

### Manual Start

```bash
# Default logging
crawlds-daemon

# Debug logging
RUST_LOG=debug crawlds-daemon

# Custom config
RUST_LOG=info crawlds-daemon

# Environment override
CRAWLDS__DAEMON__LOG_LEVEL=trace crawlds-daemon
```

## Event Broadcasting

All domains publish to a single broadcast channel (buffer size: 100):

```rust
let (event_tx, _) = broadcast::channel::<CrawlEvent>(100);
```

### Subscribe Command

Connect and subscribe to receive all events:

```json
{"jsonrpc": "2.0", "method": "Subscribe", "params": {}, "id": 1}
```

After subscribing, events are delivered as NDJSON:

```json
{"jsonrpc": "2.0", "method": "event", "params": {"domain": "sysmon", "data": {...}}}
```

### Event Format

```json
{
    "domain": "sysmon",
    "data": {
        "event": "cpu_update",
        "cpu": { ... }
    }
}
```

### Filtering

The event stream broadcasts all events. Clients filter by `domain` field.

## Error Handling

### JSON-RPC Errors

All errors return JSON envelope:

```json
{
    "jsonrpc": "2.0",
    "error": {
        "code": -32602,
        "message": "Invalid params: ..."
    },
    "id": 1
}
```

### Error Codes

| Code | Meaning |
|------|---------|
| -32600 | Invalid JSON-RPC request |
| -32601 | JSON value is not a valid method |
| -32602 | Invalid params |
| -32000 | Server error |

### Domain Task Failures

When a domain task panics or errors:
1. Error is logged with domain name
2. Other domains continue running
3. Daemon remains alive
4. Events cease for that domain

### VFS

The daemon uses `crawlds-vfs` for file system operations including:
- File listing and searching
- Directory operations (mkdir, delete, copy, move, rename)
- Trash operations
- Removable media (mount, unmount, eject)

## Dependencies

### System Dependencies

| Dependency | Package | Required | Used by |
|------------|---------|----------|---------|
| D-Bus | dbus | Yes | Bluetooth, Network, GeoClue |
| GeoClue2 | geoclue-2.0 | No | Geolocation |
| NetworkManager | NetworkManager | No | Network |
| UPower | upower | No | Power |
| UDisks2 | udisks2 | No | Disk |
| greetd | greetd | No | Greeter |

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `figment` | Configuration |
| `tracing` | Logging |
| `serde` | Serialization |
| `zbus` | D-Bus client |
| `sysinfo` | System info |

## Logging

### Log Levels

```toml
[daemon]
log_level = "info"  # trace, debug, info, warn, error
```

### Environment Variable

```bash
RUST_LOG=debug crawlds-daemon
```

### Log Output

Logs go to stderr (journald on systemd):
```bash
journalctl --user -u crawlds-daemon -f
```

### Per-Domain Logging

```bash
RUST_LOG=crawlds::network=debug,crawlds::bluetooth=trace crawlds-daemon
```

## Troubleshooting

### Daemon Won't Start

1. **Socket already exists**
   ```bash
   rm /run/user/$UID/crawlds.sock
   systemctl --user restart crawlds-daemon
   ```

2. **Config error**
   ```bash
   RUST_LOG=debug crawlds-daemon
   # Check for config parsing errors
   ```

3. **Missing dependencies**
   ```bash
   # Check D-Bus
   dbus-send --system --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.ListNames
   ```

### Connection Refused

1. **Daemon not running**
   ```bash
   systemctl --user status crawlds-daemon
   ```

2. **Wrong socket path**
   ```bash
   ls -la /run/user/$UID/crawlds.sock
   ```

3. **Permission issue**
   ```bash
   # Check socket permissions
   ls -la /run/user/$UID/crawlds.sock
   # Should be srwxr-xr-x
   ```

### Domain Not Working

1. **Check domain started**
   ```bash
   journalctl --user -u crawlds-daemon -f | grep <domain>
   ```

2. **Check config**
   ```bash
   cat ~/.config/crawlds/core.toml | grep <domain>
   ```

3. **Check system service**
   ```bash
   # Bluetooth
   systemctl status bluetooth
   
   # Network
   systemctl status NetworkManager
   
   # Geolocation
   systemctl status geoclue2
   ```

### High CPU/Memory

1. **Check sysmon interval**
   ```toml
   [sysmon]
   poll_interval_ms = 2000  # Increase from 1000
   ```

2. **Disable unused domains** in config

3. **Check for leaks**
   ```bash
   # Monitor memory growth
   top -p $(pidof crawlds-daemon)
   ```

### Events Not Received

1. **Subscribe not called**
   - Client must send `Subscribe` command to receive events

2. **Domain crashed**
   ```bash
   journalctl --user -u crawlds-daemon | grep <domain>
   ```

3. **Check broadcast channel**
   - Internal max: 100 events
   - If full, oldest events dropped

## Security Considerations

### Socket Permissions

- Socket created with user permissions only
- Only same user can connect
- No world/group access

### D-Bus Permissions

Bluetooth and network require D-Bus permissions. Common issues:

```bash
# Check Bluetooth permissions
id

# Add to required groups
sudo usermod -aG bluetooth $USER
sudo usermod -aG network $USER
```

### No root Required

Daemon runs as regular user. Operations requiring root (mount, etc.) use:
- Polkit for authorization
- UDisks2 for mounting (user session)
- greetd for login (runs greeter as root)

## Performance Optimizations

### Event Buffer Size

```rust
broadcast::channel::<CrawlEvent>(100)
```
- 100 pending events max
- Old events dropped if full
- Adjust if high event rate needed

### Connection Handling

- Each connection spawns new tokio task
- No connection limit configured
- Monitor with:
  ```bash
  ss -x | grep crawlds
  ```

### Domain Task Memory

Each domain maintains its own state:
- sysmon: System snapshot every poll
- clipboard: Ring buffer with configurable max entries
- notifications: History up to configured max items

## Extensions

### Adding New Domain

1. Create crate in `core/crates/crawlds-<name>/`
2. Add to workspace `Cargo.toml`
3. Add config type to `Config` struct
4. Add to `spawn_domains()` in `main.rs`
5. Add JSON-RPC command to `json_server.rs`
6. Document in new section

### Custom JSON-RPC Commands

Add to `json_server.rs`:

```rust
// Add to Command enum
pub enum Command {
    // ... existing commands
    MyCommand { param: String },
}

// Add to execute match
Command::MyCommand { param } => self.my_handler(param).await,
```

## See Also

- [IPC.md](IPC.md) - Event types and serialization
- [CLI.md](CLI.md) - Command-line client
- [SYSMONITOR.md](SYSMONITOR.md) - System monitoring
- [BLUETOOTH.md](BLUETOOTH.md) - Bluetooth system
- [NETWORK.md](NETWORK.md) - Network system
- [CLIPBOARD.md](CLIPBOARD.md) - Clipboard system
- [THEMING.md](THEMING.md) - Theme management
