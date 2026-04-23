# Project Structure

```
crawlds/
├── Cargo.toml                   # workspace manifest
├── config/
│   └── core.toml               # annotated example configuration
├── systemd/
│   └── crawlds.service            # systemd user service unit
├── pkg/
│   └── PKGBUILD                 # Arch Linux package
└── crates/
    ├── crawlds-ipc/               # shared types, events, error envelope
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs         # CrawlError, ErrorEnvelope
    │       ├── events.rs        # CrawlEvent enum (all domain events)
    │       └── types.rs         # BluetoothDevice, BatteryStatus, GeoLocation, etc.
    │
    ├── crawlds-daemon/            # main binary — axum server over Unix socket
    │   └── src/
    │       ├── main.rs          # startup, spawn_domains()
    │       ├── config.rs        # figment config loading
    │       ├── state.rs         # AppState (Arc<Config> + broadcast tx)
    │       ├── router.rs        # all axum routes
    │       └── sse.rs           # GET /events SSE handler
    │
    ├── crawlds-cli/               # crawlds binary — thin HTTP client + clap
    │   └── src/
    │       ├── main.rs          # Cli + Commands dispatch
    │       ├── client.rs        # CrawlClient (hyper over Unix socket)
    │       ├── output.rs        # terminal formatting helpers
    │       └── cmd/
    │           ├── brightness.rs
    │           ├── bluetooth.rs
    │           ├── audio.rs
    │           ├── network.rs
    │           ├── notify.rs
    │           ├── clipboard.rs
    │           ├── proc_.rs
    │           ├── power.rs
    │           ├── disk.rs
    │           ├── sysmon.rs
    │           └── daemon.rs
    │
    ├── crawlds-bluetooth/                # bluer + BlueZ D-Bus
    ├── crawlds-network/               # zbus + NetworkManager
    ├── crawlds-notify/            # zbus — implements org.freedesktop.Notifications
    ├── crawlds-clipboard/         # wl-clipboard-rs — Wayland clipboard
    ├── crawlds-sysmon/            # sysinfo — CPU, memory, disk
    ├── crawlds-brightness/        # sysfs /sys/class/backlight
    ├── crawlds-proc/              # sysinfo — process list/kill
    ├── crawlds-power/             # zbus — UPower battery
    ├── crawlds-disk/              # zbus — UDisks2 block devices
```
