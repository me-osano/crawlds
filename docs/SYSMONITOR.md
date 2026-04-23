# System Monitor Module

CrawlDS System Monitor (sysmon) provides real-time system metrics including CPU, memory, network traffic, GPU status, and disk information. The module runs as an independent domain task within the daemon, broadcasting events via SSE and exposing synchronous query functions for HTTP requests.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           BACKEND (Rust)                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-sysmon                                                       │   │
│  │   ├── Config           (poll_interval_ms, thresholds)               │   │
│  │   ├── NetState         (tracks network interface stats)              │   │
│  │   ├── get_cpu()        → CpuStatus                                   │   │
│  │   ├── get_mem()        → MemStatus                                   │   │
│  │   ├── get_disks()      → Vec<DiskStatus>                             │   │
│  │   ├── get_net()        → NetTraffic                                  │   │
│  │   └── get_gpu()        → Option<GpuStatus>                           │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-daemon HTTP Router                                           │   │
│  │   GET /sysmon/cpu   → crawlds_sysmon::get_cpu()                     │   │
│  │   GET /sysmon/mem   → crawlds_sysmon::get_mem()                     │   │
│  │   GET /sysmon/disk  → crawlds_sysmon::get_disks()                   │   │
│  │   GET /sysmon/net   → crawlds_sysmon::get_net()                     │   │
│  │   GET /sysmon/gpu   → crawlds_sysmon::get_gpu()                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                         (broadcast::Sender<CrawlEvent>)                    │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND (QML)                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CrawlDSService                                                       │   │
│  │   └── Receives SysmonEvent via SSE                                  │   │
│  │       ├── cpuUpdate(cpu: CpuStatus)                                 │   │
│  │       ├── memUpdate(mem: MemStatus)                                 │   │
│  │       ├── netUpdate(traffic: NetTraffic)                            │   │
│  │       ├── gpuUpdate(gpu: GpuStatus)                                │   │
│  │       ├── cpuSpike(usage, threshold)                                │   │
│  │       └── memPressure(used_percent)                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ SystemMonitorService.qml                                              │   │
│  │   ├── Subscribes to CrawlDSService signals                           │   │
│  │   ├── Maintains current state (cpu, mem, net, gpu)                  │   │
│  │   ├── Triggers thresholds for alerts                                 │   │
│  │   └── Provides formatting and display utilities                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Configuration

In `core.toml`:

```toml
[sysmon]
poll_interval_ms = 1000          # Update interval in milliseconds
cpu_spike_threshold = 90.0       # Emit CpuSpike when aggregate CPU exceeds this percent
mem_pressure_threshold = 85.0    # Emit MemPressure when memory usage exceeds this percent
cpu_change_threshold = 2.0        # Only emit CPU updates when change exceeds this %
mem_change_threshold = 1.0       # Only emit MEM updates when change exceeds this %
net_change_threshold = 1024      # Only emit NET updates when change exceeds this bytes
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `poll_interval_ms` | u64 | 1000 | How often to poll system metrics (ms) |
| `cpu_spike_threshold` | f32 | 90.0 | CPU usage % that triggers CpuSpike event |
| `mem_pressure_threshold` | f32 | 85.0 | Memory usage % that triggers MemPressure event |
| `cpu_change_threshold` | f32 | 2.0 | Minimum CPU change % to emit update |
| `mem_change_threshold` | f32 | 1.0 | Minimum memory change % to emit update |
| `net_change_threshold` | u64 | 1024 | Minimum network bytes to emit update |

## HTTP API

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/sysmon/cpu` | GET | Get current CPU status |
| `/sysmon/mem` | GET | Get current memory status |
| `/sysmon/disk` | GET | Get disk information |
| `/sysmon/net` | GET | Get network traffic |
| `/sysmon/gpu` | GET | Get GPU status |

### Response Examples

**GET /sysmon/cpu**
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

**GET /sysmon/mem**
```json
{
  "total_kb": 33554432,
  "used_kb": 16777216,
  "available_kb": 16777216,
  "swap_total_kb": 8388608,
  "swap_used_kb": 1048576
}
```

**GET /sysmon/disk**
```json
[
  {
    "mount": "/",
    "total_bytes": 500000000000,
    "used_bytes": 250000000000,
    "available_bytes": 250000000000,
    "filesystem": "ext4"
  },
  {
    "mount": "/home",
    "total_bytes": 1000000000000,
    "used_bytes": 750000000000,
    "available_bytes": 250000000000,
    "filesystem": "ext4"
  }
]
```

**GET /sysmon/net**
```json
{
  "rx_bytes": 1234567890,
  "tx_bytes": 987654321,
  "rx_bps": 1024000,
  "tx_bps": 512000
}
```

**GET /sysmon/gpu**
```json
{
  "name": "NVIDIA GeForce RTX 3080",
  "temperature_c": 65.0
}
```
or `null` if GPU info unavailable.

### Using curl

```bash
# Get CPU status
curl --unix-socket /run/crawlds.sock http://localhost/sysmon/cpu

# Get memory status
curl --unix-socket /run/crawlds.sock http://localhost/sysmon/mem

# Get disk info
curl --unix-socket /run/crawlds.sock http://localhost/sysmon/disk

# Get network traffic
curl --unix-socket /run/crawlds.sock http://localhost/sysmon/net

# Get GPU status
curl --unix-socket /run/crawlds.sock http://localhost/sysmon/gpu
```

## Events (SSE)

The system monitor broadcasts events at the configured interval. Connect to the SSE stream:

```bash
curl --unix-socket /run/crawlds.sock http://localhost/events
```

### Event Types

#### CpuUpdate
```json
{
  "domain": "sysmon",
  "data": {
    "event": "cpu_update",
    "cpu": {
      "aggregate": 25.5,
      "cores": [30.2, 22.1, 24.8, 25.0],
      "frequency_mhz": [3500, 3500, 3500, 3500],
      "load_avg": { "one": 1.25, "five": 1.10, "fifteen": 0.95 },
      "temperature_c": null
    }
  }
}
```

#### MemUpdate
```json
{
  "domain": "sysmon",
  "data": {
    "event": "mem_update",
    "mem": {
      "total_kb": 33554432,
      "used_kb": 16777216,
      "available_kb": 16777216,
      "swap_total_kb": 8388608,
      "swap_used_kb": 1048576
    }
  }
}
```

#### NetUpdate
```json
{
  "domain": "sysmon",
  "data": {
    "event": "net_update",
    "traffic": {
      "rx_bytes": 1234567890,
      "tx_bytes": 987654321,
      "rx_bps": 1024000,
      "tx_bps": 512000
    }
  }
}
```

#### GpuUpdate
```json
{
  "domain": "sysmon",
  "data": {
    "event": "gpu_update",
    "gpu": {
      "name": "NVIDIA GeForce RTX 3080",
      "temperature_c": 65.0
    }
  }
}
```

#### CpuSpike (threshold alert)
```json
{
  "domain": "sysmon",
  "data": {
    "event": "cpu_spike",
    "usage": 92.5,
    "threshold": 90.0
  }
}
```

#### MemPressure (threshold alert)
```json
{
  "domain": "sysmon",
  "data": {
    "event": "mem_pressure",
    "used_percent": 87.3
  }
}
```

## Data Types

### CpuStatus
```rust
pub struct CpuStatus {
    pub aggregate: f32,           // Average CPU usage across all cores (%)
    pub cores: Vec<f32>,          // Per-core CPU usage (%)
    pub frequency_mhz: Vec<u64>,   // Per-core frequency (MHz)
    pub load_avg: LoadAvg,        // System load averages
    pub temperature_c: Option<f32>, // CPU temperature (if available)
}
```

### LoadAvg
```rust
pub struct LoadAvg {
    pub one: f64,      // 1-minute load average
    pub five: f64,     // 5-minute load average
    pub fifteen: f64,  // 15-minute load average
}
```

### MemStatus
```rust
pub struct MemStatus {
    pub total_kb: u64,       // Total physical memory (KB)
    pub used_kb: u64,        // Used physical memory (KB)
    pub available_kb: u64,   // Available physical memory (KB)
    pub swap_total_kb: u64,  // Total swap space (KB)
    pub swap_used_kb: u64,   // Used swap space (KB)
}
```

### NetTraffic
```rust
pub struct NetTraffic {
    pub rx_bytes: u64,  // Total received bytes (cumulative)
    pub tx_bytes: u64,  // Total transmitted bytes (cumulative)
    pub rx_bps: u64,    // Current receive rate (bytes/sec)
    pub tx_bps: u64,    // Current transmit rate (bytes/sec)
}
```

### GpuStatus
```rust
pub struct GpuStatus {
    pub name: Option<String>,      // GPU driver name
    pub temperature_c: Option<f32>, // GPU temperature (if available)
}
```

### DiskStatus
```rust
pub struct DiskStatus {
    pub mount: String,              // Mount point
    pub total_bytes: u64,          // Total disk space
    pub used_bytes: u64,           // Used disk space
    pub available_bytes: u64,      // Available disk space
    pub filesystem: Option<String>, // Filesystem type
}
```

## Backend Crate

**Location**: `core/crates/crawlds-sysmon/`

### Key Functions

```rust
// Start the system monitor domain task
pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) -> anyhow::Result<()>

// Synchronous query functions (called from HTTP handlers)
pub fn get_cpu() -> CpuStatus
pub fn get_mem() -> MemStatus
pub fn get_disks() -> Vec<DiskStatus>
pub fn get_net() -> NetTraffic
pub fn get_gpu() -> Option<GpuStatus>

// Builder functions (for use in tests or external consumers)
pub fn build_cpu_status(sys: &System) -> CpuStatus
pub fn build_mem_status(sys: &System) -> MemStatus
pub fn build_disk_status() -> Vec<DiskStatus>
```

### Implementation Details

- Uses `sysinfo` crate for CPU and memory metrics
- Reads `/proc/net/dev` directly for network statistics
- GPU detection via `/sys/class/drm/` sysfs paths
- Loopback interface (`lo`) is excluded from network stats
- Network bytes/second calculated from delta between samples

## Dependencies

- `sysinfo` - Cross-platform system information
- `tokio` - Async runtime for the domain task
- `tracing` - Logging

## Troubleshooting

### No GPU info detected

GPU detection is best-effort and reads from sysfs. Check:
```bash
# List DRM devices
ls -la /sys/class/drm/

# Check GPU hwmon for temperature
ls -la /sys/class/drm/card*/device/hwmon/
cat /sys/class/drm/card*/device/hwmon/hwmon*/temp1_input
```

### Network stats show 0

- Check if `/proc/net/dev` exists and is readable
- Loopback interface (`lo`) is intentionally excluded
- Stats may show 0 on first sample (no delta to calculate rate)

### High CPU from polling

1. Reduce `poll_interval_ms` in config:
```toml
[sysmon]
poll_interval_ms = 2000  # 2 seconds instead of 1
```

2. Increase change thresholds to emit less frequently:
```toml
[sysmon]
cpu_change_threshold = 5.0    # Only emit when CPU changes >5%
mem_change_threshold = 2.0   # Only emit when memory changes >2%
net_change_threshold = 10240  # Only emit when network changes >10KB
```

### Events not received

1. Verify daemon is running:
   ```bash
   curl --unix-socket /run/crawlds.sock http://localhost/health
   ```

2. Check SSE connection:
   ```bash
   curl --unix-socket /run/crawlds.sock http://localhost/events
   ```

3. Check domain started:
   ```bash
   journalctl -u crawlds-daemon | grep sysmon
   ```

### Change Detection

Events are only emitted when values change beyond the configured thresholds. This dramatically reduces IPC overhead. The first update always includes all metrics, subsequent updates are filtered:

| Metric | First Update | Subsequent Updates |
|--------|------------|------------------|
| CPU | Always | When change > `cpu_change_threshold` |
| Memory | Always | When change > `mem_change_threshold` |
| Network | Always | When bytes > `net_change_threshold` |
| GPU | Always | Only on change |

### Events not received

1. Verify daemon is running:
   ```bash
   curl --unix-socket /run/crawlds.sock http://localhost/health
   ```

2. Check SSE connection:
   ```bash
   curl --unix-socket /run/crawlds.sock http://localhost/events
   ```

3. Check domain started:
   ```bash
   journalctl -u crawlds-daemon | grep sysmon
   ```
