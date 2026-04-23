# Command-Line Interface (CLI)

The CrawlDS CLI (`crawlds`) provides a command-line interface to the CrawlDS daemon. It communicates with the daemon over a Unix socket, exposing all daemon functionality for scripting, automation, and terminal usage.

## Installation

```bash
# Build from source
cargo build --release -p crawlds-cli

# The binary will be at target/release/crawlds
sudo cp target/release/crawlds ~/.local/bin/
```

## Synopsis

```bash
crawlds [OPTIONS] <COMMAND>

# Global options
crawlds --socket <PATH>     # Override daemon socket path
crawlds --json, -j         # Output raw JSON instead of formatted output
crawlds --help              # Show help
crawlds --version          # Show version
```

## Global Options

| Option | Description |
|--------|-------------|
| `--socket, -s <PATH>` | Override the daemon socket path (env: `CRAWLDS_SOCKET`) |
| `--json, -j` | Output raw JSON instead of human-readable formatted output |
| `--help, -h` | Show help information |
| `--version, -V` | Print version |

### Socket Path

By default, the CLI connects to:
```
$XDG_RUNTIME_DIR/crawlds.sock
```

Common paths:
- `/run/user/<uid>/crawlds.sock` (XDG_RUNTIME_DIR set)
- `/run/crawlds.sock` (fallback)

Override with `--socket`:
```bash
crawlds --socket /tmp/crawlds.sock sysmon --cpu
```

## Commands

### run

Run the Quickshell desktop shell.

```bash
crawlds run [OPTIONS] [-- <ARGS>...]
```

| Option | Description |
|--------|-------------|
| `-c, --config <PATH>` | Quickshell config directory (default: `~/.config/quickshell/crawlds`) |
| `-d` | Run as daemon |

**Examples:**
```bash
# Run Quickshell interactively
crawlds run

# Run specific config
crawlds run -c ~/.config/quickshell/my-config

# Run as daemon
crawlds run -d
```

### restart

Restart the running shell.

```bash
crawlds restart
```

Sends SIGUSR1 to all running shell instances to trigger a restart. If no shell is running, starts a new daemon.

**Examples:**
```bash
crawlds restart
```

### kill

Kill the running shell.

```bash
crawlds kill
```

Sends SIGKILL to all running shell instances and cleans up PID files.

**Examples:**
```bash
crawlds kill
```

### ipc

Send IPC commands to the running shell.

```bash
crawlds ipc <TARGET> <FUNCTION> [ARGS...]
```

**Examples:**
```bash
# List available IPC targets
crawlds ipc

# Call an IPC function
crawlds ipc system brightness Get
```

### update

Update CrawlDS to the latest release.

```bash
crawlds update [OPTIONS] [-- <ARGS>...]
```

| Option | Description |
|--------|-------------|
| `--dry-run` | Check for updates without installing |
| `--pass-through <ARGS>` | Pass args to the updater script |

**Examples:**
```bash
# Check for updates
crawlds update --dry-run

# Install update
crawlds update
```

### version

Show version information.

```bash
crawlds version [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-j, --json` | Output JSON |

**Examples:**
```bash
crawlds version
crawlds version --json
```

### bluetooth

Bluetooth device management.

```bash
crawlds bluetooth [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--list` | List paired/known devices |
| `--scan` | Start device discovery scan |
| `--connect <ADDRESS>` | Connect to a device |
| `--disconnect <ADDRESS>` | Disconnect a device |
| `--power <on\|off>` | Power the adapter on/off |
| `--status` | Show adapter status |
| `--discoverable <on\|off>` | Set discoverable mode |
| `--pairable <on\|off>` | Set pairable mode |
| `--alias <ADDRESS> <NAME>` | Set device alias |
| `--pair <ADDRESS>` | Pair with a device |
| `--trust <ADDRESS> <on\|off>` | Trust or untrust a device |
| `--remove <ADDRESS>` | Remove/forget a device |

**Examples:**
```bash
# Show Bluetooth status
crawlds bluetooth

# List devices
crawlds bluetooth --list

# Scan for devices
crawlds bluetooth --scan

# Connect to a device
crawlds bluetooth --connect AA:BB:CC:DD:EE:FF

# Power off adapter
crawlds bluetooth --power off

# JSON output
crawlds bluetooth --list --json
```

### network

Network and WiFi management.

```bash
crawlds network [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--status` | Show network status |
| `--wifi-list` | List available WiFi networks |
| `--wifi-connect <SSID>` | Connect to WiFi |
| `--wifi-disconnect` | Disconnect WiFi |
| `--ethernet-list` | List ethernet interfaces |
| `--mode <station\|ap>` | Set network mode |
| `--hotspot-start` | Start hotspot |
| `--hotspot-stop` | Stop hotspot |

**Examples:**
```bash
# Show network status
crawlds network

# List WiFi networks
crawlds network --wifi-list

# Connect to WiFi
crawlds network --wifi-connect "MyNetwork"

# Start hotspot
crawlds network --hotspot-start
```

### notify

Notification control.

```bash
crawlds notify [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--list` | List active notifications |
| `--dismiss <ID>` | Dismiss a notification |
| `--title <TEXT>` | Set notification title |
| `--body <TEXT>` | Set notification body |
| `--urgency <low\|normal\|critical>` | Set urgency level |

**Examples:**
```bash
# List notifications
crawlds notify --list

# Send a notification
crawlds notify --title "Hello" --body "World"

# Send critical notification
crawlds notify --title "Alert" --body "Something happened" --urgency critical

# Dismiss notification
crawlds notify --dismiss 5
```

### clip (Clipboard)

Clipboard access and management.

```bash
crawlds clip [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--get` | Get current clipboard content |
| `--set <TEXT>` | Set clipboard content |
| `--history` | Show clipboard history |

**Examples:**
```bash
# Get clipboard
crawlds clip --get

# Set clipboard
crawlds clip --set "Hello, World!"

# Show history
crawlds clip --history

# JSON output
crawlds clip --history --json
```

### sysmon (System Monitor)

System monitoring information.

```bash
crawlds sysmon [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--cpu` | Show CPU usage |
| `--mem` | Show memory usage |
| `--disk` | Show disk usage |
| `--net` | Show network throughput |
| `--gpu` | Show GPU info |
| `--watch` | Stream live updates |

**Examples:**
```bash
# Default (CPU)
crawlds sysmon

# CPU with details
crawlds sysmon --cpu

# Memory
crawlds sysmon --mem

# Disk usage
crawlds sysmon --disk

# GPU info
crawlds sysmon --gpu

# Watch mode (live updates)
crawlds sysmon --watch

# Watch specific metrics
crawlds sysmon --cpu --mem --watch

# JSON output
crawlds sysmon --cpu --json
```

### brightness

Display brightness control.

```bash
crawlds brightness [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--get` | Get current brightness |
| `--set <PERCENT>` | Set brightness (0-100) |
| `--inc <PERCENT>` | Increase brightness |
| `--dec <PERCENT>` | Decrease brightness |

**Examples:**
```bash
# Get brightness
crawlds brightness --get

# Set to 50%
crawlds brightness --set 50

# Increase by 10%
crawlds brightness --inc 10

# Decrease by 20%
crawlds brightness --dec 20
```

### proc (Processes)

Process listing and management.

```bash
crawlds proc [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--list` | List processes |
| `--find <NAME>` | Find processes by name |
| `--kill <PID>` | Kill a process |
| `--watch <PID>` | Watch for process exit |

**Examples:**
```bash
# List top processes
crawlds proc --list

# Find Firefox
crawlds proc --find firefox

# Kill process
crawlds proc --kill 12345

# Watch for exit
crawlds proc --watch 12345
```

### power

Battery and power status.

```bash
crawlds power [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--status` | Show power status |
| `--battery` | Show battery details |

**Examples:**
```bash
# Power status
crawlds power

# Battery details
crawlds power --battery
```

### disk

Disk and removable media management.

```bash
crawlds disk [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--list` | List block devices |
| `--mount <DEVICE>` | Mount a device |
| `--unmount <DEVICE>` | Unmount a device |

**Examples:**
```bash
# List devices
crawlds disk --list

# Mount device
crawlds disk --mount /dev/sdb1

# Unmount device
crawlds disk --unmount /dev/sdb1
```

### daemon

Daemon control.

```bash
crawlds daemon [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--status` | Show daemon status |
| `--restart` | Restart the daemon |
| `--stop` | Stop the daemon |

**Examples:**
```bash
# Status
crawlds daemon --status

# Restart
crawlds daemon --restart
```

### greeter

Greeter (greetd) management.

```bash
crawlds greeter [OPTIONS]
```

**Examples:**
```bash
# Start greeter
crawlds greeter

# With specific command
crawlds greeter <command>
```

## Output Formats

### Human-Readable (Default)

```
CPU
  usage     25.5%
  load avg  1.25  1.10  0.95
  cores     30.2%  22.1%  24.8%  25.0%
```

### JSON (`--json`)

```json
{
  "aggregate": 25.5,
  "cores": [30.2, 22.1, 24.8, 25.0],
  "frequency_mhz": [3500, 3500, 3500, 3500],
  "load_avg": {
    "one": 1.25,
    "five": 1.10,
    "fifteen": 0.95
  },
  "temperature_c": null
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CRAWLDS_SOCKET` | Daemon socket path (overridden by `--socket`) |
| `CRAWLDS_CONFIG_PATH` | Quickshell config path (used by shell commands) |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Connection error |
| 3 | Invalid arguments |

## Examples

### Daily Workflow

```bash
#!/bin/bash

# Check system status
echo "=== System Status ==="
crawlds sysmon --cpu --mem --disk

# Battery check
echo "=== Battery ==="
crawlds power --battery

# Network status
echo "=== Network ==="
crawlds network --status

# Clipboard history
echo "=== Recent Clipboard ==="
crawlds clip --history
```

### Scripting

```bash
#!/bin/bash
# Monitor script

# Get CPU usage
CPU=$(crawlds sysmon --cpu --json | jq -r '.aggregate')
if (( $(echo "$CPU > 90" | bc -l) )); then
    crawlds notify --title "High CPU" --body "CPU at ${CPU}%" --urgency critical
fi

# Get memory usage
MEM=$(crawlds sysmon --mem --json | jq -r '.used_kb')
TOTAL=$(crawlds sysmon --mem --json | jq -r '.total_kb')
```

### Automation

```bash
# Cron job - hourly system report
0 * * * * crawlds sysmon --cpu --mem --disk >> /var/log/crawlds-report.log

# Auto-brightness based on time (with external tool)
0 6 * * * crawlds brightness --set 80
0 22 * * * crawlds brightness --set 30

# Battery warning script
#!/bin/bash
BATTERY=$(crawlds power --battery --json | jq -r '.percent')
if (( $(echo "$BATTERY < 20" | bc -l) )); then
    crawlds notify --title "Low Battery" --body "${BATTERY}% remaining"
fi
```

### Quick Commands

```bash
# Copy to clipboard
echo "text" | xclip -selection clipboard && crawlds clip --set "$(xclip -o)"

# Find resource-heavy processes
crawlds proc --list | head -20

# Quick system check
crawlds sysmon --cpu --mem -j | jq

# Toggle Bluetooth
crawlds bluetooth --power off  # Turn off
crawlds bluetooth --power on   # Turn on
```

## Troubleshooting

### Connection Refused

```bash
# Check if daemon is running
systemctl --user status crawlds-daemon

# Start daemon
systemctl --user start crawlds-daemon

# Check socket exists
ls -la /run/user/$UID/crawlds.sock

# Use correct socket path
crawlds --socket /run/user/$UID/crawlds.sock sysmon
```

### Permission Denied

```bash
# Check user permissions
id

# For system-wide socket:
ls -la /run/crawlds.sock
sudo usermod -aG crawlds $USER
```

### Timeout or Slow Response

```bash
# Use timeout
timeout 5 crawlds sysmon --cpu

# Check daemon logs
journalctl --user -u crawlds-daemon -f
```

## See Also

- [IPC.md](IPC.md) - Event and type documentation
- [SYSMONITOR.md](SYSMONITOR.md) - System monitor details
- [BLUETOOTH.md](BLUETOOTH.md) - Bluetooth system details
- [NETWORK.md](NETWORK.md) - Network system details
