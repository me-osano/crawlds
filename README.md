# CrawlDS

```
    █████████  ███████████     █████████   █████   ███   █████ █████       ██████████    █████████
  ███░░░░░███░░███░░░░░███   ███░░░░░███ ░░███   ░███  ░░███ ░░███       ░░███░░░░███  ███░░░░░███
 ███     ░░░  ░███    ░███  ░███    ░███  ░███   ░███   ░███  ░███        ░███   ░░███░███    ░░░ 
░███          ░██████████   ░███████████  ░███   ░███   ░███  ░███        ░███    ░███░░█████████ 
░███          ░███░░░░░███  ░███░░░░░███  ░░███  █████  ███   ░███        ░███    ░███ ░░░░░░░░███
░░███     ███ ░███    ░███  ░███    ░███   ░░░█████░█████░    ░███      █ ░███    ███  ███    ░███
░░█████████  █████   █████ █████   █████    ░░███ ░░███      ███████████ ██████████  ░░█████████ 
░░░░░░░░░  ░░░░░   ░░░░░ ░░░░░   ░░░░░      ░░░   ░░░      ░░░░░░░░░░░  ░░░░░░░░░░    ░░░░░░░░░  
```

A fast, modular system services daemon and CLI for Linux Wayland desktops —
built in Rust for use with [niri](https://github.com/YaLTeR/niri),
[Quickshell](https://quickshell.outfoxxed.me/), and any compositor that
doesn't hand you a desktop environment.

## Table of Contents

- [What crawlds is](#what-crawlds-is)
- [Domains](#domains)
- [Installation](#installation)
- [Setup](#setup)
- [Configuration](#configuration)
- [CLI Reference](#cli-reference)
- [License](#license)

---

## What crawlds is

`crawlds` is two things:

**`crawlds-daemon`** — a long-running Rust process that owns system-level
concerns and exposes them over a Unix socket at
`$XDG_RUNTIME_DIR/crawlds.sock`. Each domain (Bluetooth, audio, brightness,
etc.) runs as an independent `tokio` task. Events from all domains are
broadcast over a single SSE stream that any number of clients can subscribe
to simultaneously.

**`crawlds`** (the CLI) — a thin client that sends HTTP-over-socket requests
to the daemon and formats the response for the terminal.

```
crawlds CLI ─────┐                  HTTP over Unix socket
crawlds-bluetooth ──┤  broadcast    $XDG_RUNTIME_DIR/crawlds.sock
crawlds-network ───┤  channel       ──► GET /events (SSE stream)
crawlds-sysmon ───┤  (CrawlEvent)
crawlds-audio ────┤                  Quickshell QML
crawlds-power ────┤                  DataStream / NetworkRequest
...            ──┘
              └── crawlds-daemon (axum)
```

## Domains

| Domain | Backend | What it owns |
|---|---|---|
| Bluetooth | `bluer` / BlueZ D-Bus | Discovery, pair, connect/disconnect |
| Network | `zbus` / NetworkManager | WiFi scan, connect, hotspot |
| Notifications | `zbus` / D-Bus | Full notification daemon |
| Clipboard | `wl-clipboard-rs` | Clipboard + history |
| System monitor | `sysinfo` | CPU, memory, disk, GPU |
| Brightness | sysfs | Backlight read/write |
| Processes | `sysinfo` | Process list, search, kill |
| Power | `zbus` / UPower | Battery, time estimates |
| Disk | `zbus` / UDisks2 | Mount, unmount, eject |
| Webservice | `reqwest` | RSS feeds, Wallhaven wallpaper |
| Greeter | `greetd_ipc` | Login screen, PAM |

---

## Installation

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/me-osano/crawlds/master/install.sh | sh
```

Options:
- `--daemon-only` — Install only the Rust daemon
- `--shell-only` — Install only the Quickshell shell
- `--system` — System-wide install (requires sudo)
- `--enable` — Enable and start the daemon service

### Dependencies

**Runtime:**
```
bluez  networkmanager  udisks2  upower  pipewire  pipewire-pulse  libpulse  quickshell
```

**Build:**
```
rust (stable)  cargo  pkg-config  clang
```

On Arch:
```bash
sudo pacman -S bluez bluez-libs networkmanager udisks2 upower \
               pipewire pipewire-pulse libpulse \
               rust pkg-config clang quickshell-git
```

### Manual Build

```bash
git clone https://github.com/me-osano/crawlds
cd crawlds/core
cargo build --release --workspace --bins

# Install binaries
sudo install -Dm755 target/release/crawlds-daemon ~/.local/bin/
sudo install -Dm755 target/release/crawlds ~/.local/bin/

# Install service
mkdir -p ~/.config/systemd/user
cp systemd/crawlds.service ~/.config/systemd/user/
```

---

## Setup

### Brightness permissions

crawlds reads/writes `/sys/class/backlight/<device>/brightness` directly.

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

### Notifications

Disable your existing notification daemon (mako/dunst):
```bash
systemctl --user disable --now mako
```

### systemd user service

```bash
systemctl --user enable --now crawlds
systemctl --user status crawlds
journalctl --user -u crawlds -f
```

---

## Configuration

Config: `$XDG_CONFIG_HOME/crawlds/core.toml` (default `~/.config/crawlds/core.toml`)

Environment variable overrides:
```bash
CRAWLDS__DAEMON__LOG_LEVEL=debug crawlds-daemon
CRAWLDS__SYSMON__POLL_INTERVAL_MS=2000 crawlds-daemon
```

Key settings:
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

---

## CLI Reference

All commands accept `--json` for raw JSON output.

### run
```bash
crawlds run              # Run Quickshell shell
crawlds run -c /path     # Custom config
```

### brightness
```bash
crawlds brightness --set=80  # set to 80%
crawlds brightness --inc=5   # increase 5%
crawlds brightness --dec=10  # decrease 10%
```

### sysmon
```bash
crawlds sysmon --cpu        # CPU usage
crawlds sysmon --mem        # memory
crawlds sysmon --disk       # disk usage
crawlds sysmon --watch      # live updates (SSE)
```

### bluetooth
```bash
crawlds bluetooth --scan
crawlds bluetooth --connect=AA:BB:CC:DD:EE:FF
crawlds bluetooth --power=on
```

### network
```bash
crawlds network --wifi --list
crawlds network --wifi --scan
crawlds network --wifi --connect --ssid=MySSID --password=xxx
crawlds network --hotspot --connect --ssid=xxx --password=xxx
```

### audio
```bash
crawlds audio --output --volume=70
crawlds audio --output --mute
crawlds audio --input --list
```

### media
```bash
crawlds media --play/--pause/--next/--prev
crawlds media --list            # all MPRIS players
crawlds media --player=spotify
```

### power
```bash
crawlds power                  # battery status
```

### notify
```bash
crawlds notify --title="Build done" --body="Success"
crawlds notify --dismiss=42
```

### clipboard
```bash
crawlds clipboard --get             # current content
crawlds clipboard --set="text"      # write
crawlds clipboard --history         # history
```

### proc
```bash
crawlds proc --sort=mem --top=10
crawlds proc --find=firefox
crawlds proc --kill=1234
```

### disk
```bash
crawlds disk --mount=/dev/sdb1
crawlds disk --unmount=/dev/sdb1
crawlds disk --eject=/dev/sdb
```

### daemon
```bash
crawlds daemon                 # status
crawlds daemon --restart/--stop
```

---

## License

MIT — see [LICENSE](LICENSE).