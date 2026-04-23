# CrawlDS Core

```
  ____ ____      ___        ___
 / ___|  _ \    / \ \      / / |
| |   | |_) |  / _ \ \ /\ / /| |
| |___|  _ <  / ___ \ V  V / | |___
 \____|_| \_\/_/   \_\_/\_/  |_____|
```

A fast, modular system services daemon and CLI for Linux Wayland desktops —
built in Rust for use with [niri](https://github.com/YaLTeR/niri),
[Quickshell](https://quickshell.outfoxxed.me/), and any compositor that
doesn't hand you a desktop environment.

## What crawlds is

**`crawlds-daemon`** — a long-running Rust process that owns system-level
concerns and exposes them over a Unix socket at
`$XDG_RUNTIME_DIR/crawlds.sock`. Each domain runs as an independent `tokio` task.
Events from all domains are broadcast over a single SSE stream.

**`crawlds`** (the CLI) — a thin client that sends HTTP-over-socket requests
to the daemon and formats the response for the terminal.

## Domains

| Domain | Backend | What it owns |
|---|---|---|
| Bluetooth | `bluer` / BlueZ D-Bus | Discovery, pair, connect/disconnect |
| Network | `zbus` / NetworkManager | WiFi scan, connect, hotspot |
| Notifications | `zbus` / D-Bus | Full notification daemon |
| Clipboard | `wl-clipboard-rs` | Clipboard + history |
| System monitor | `sysinfo` | CPU, memory, disk |
| Brightness | sysfs | Backlight read/write |
| Processes | `sysinfo` | Process list, search, kill |
| Power | `zbus` / UPower | Battery, time estimates |
| Disk | `zbus` / UDisks2 | Mount, unmount, eject |

## Installation

### Dependencies

**Runtime:**
```
bluez  networkmanager  udisks2  upower  pipewire  pipewire-pulse  libpulse
```

**Build:**
```
rust (stable)  cargo  pkg-config  clang
```

On Arch:
```bash
sudo pacman -S bluez bluez-libs networkmanager udisks2 upower \
               pipewire pipewire-pulse libpulse \
               rust pkg-config clang
```

### From source

```bash
cargo build --release --workspace --bins

# Install binaries
sudo install -Dm755 target/release/crawlds-daemon /usr/local/bin/
sudo install -Dm755 target/release/crawlds /usr/local/bin/

# Install service
mkdir -p ~/.config/systemd/user
cp systemd/crawlds.service ~/.config/systemd/user/
```

## Setup

### Brightness permissions

**udev rule (recommended):**
```
# /etc/udev/rules.d/90-crawlds-backlight.rules
ACTION=="add", SUBSYSTEM=="backlight", \
    RUN+="/usr/bin/chgrp video /sys/class/backlight/%k/brightness", \
    RUN+="/usr/bin/chmod g+w /sys/class/backlight/%k/brightness"
```

```bash
sudo usermod -aG video $USER
```

### Bluetooth

```bash
sudo systemctl enable --now bluetooth
```

### systemd user service

```bash
systemctl --user enable --now crawlds
journalctl --user -u crawlds -f
```

## Configuration

Config: `$XDG_CONFIG_HOME/crawlds/core.toml`

Environment variables use double-underscore separators:
```bash
CRAWLDS__DAEMON__LOG_LEVEL=debug crawlds-daemon
CRAWLDS__SYSMON__POLL_INTERVAL_MS=2000 crawlds-daemon
```

```toml
[daemon]
log_level = "info"

[network]
wifi_scan_on_start = false

[notifications]
replace_daemon = true

[sysmon]
poll_interval_ms = 1000
```

## CLI Reference

All commands accept `--json` for raw JSON output.

### brightness
```bash
crawlds brightness --set=80
crawlds brightness --inc=5
crawlds brightness --dec=10
```

### sysmon
```bash
crawlds sysmon --cpu
crawlds sysmon --mem
crawlds sysmon --watch
```

### bluetooth
```bash
crawlds bluetooth --scan
crawlds bluetooth --connect=AA:BB:CC:DD:EE:FF
```

### network
```bash
crawlds network --wifi --list
crawlds network --wifi --connect --ssid=xxx --password=xxx
```

### power
```bash
crawlds power
```

### notify
```bash
crawlds notify --title="Hello" --body="World"
```

### clipboard
```bash
crawlds clipboard --get
crawlds clipboard --history
```

### proc
```bash
crawlds proc
crawlds proc --find=firefox
crawlds proc --kill=1234
```

### disk
```bash
crawlds disk --mount=/dev/sdb1
crawlds disk --unmount=/dev/sdb1
```

## License

MIT — see [LICENSE](../LICENSE).