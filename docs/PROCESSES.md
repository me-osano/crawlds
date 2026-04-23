# Process Management Module

CrawlDS Process Management (proc) provides process listing, search, and management via sysinfo. Uses caching with incremental updates for efficiency.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           BACKEND (Rust)                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ ProcessCache (global)                                          │   │
│  │   ├── by_pid: HashMap<u32, ProcessInfo>                        │   │
│  │   ├── sorted_by_cpu: VecDeque (top 100)                      │   │
│  │   └── sorted_by_mem: VecDeque (top 100)                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Domain Task (crawlds-proc::run)                              │   │
│  │   Every 1s: refresh_top() → emit TopUpdate event            │   │
│  │   Every 5s: full_refresh() for search/kill                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────���─��─────────┐   │
│  │ JSON-RPC Server                                             │   │
│  │   ProcList → list_processes() (uses cache)                   │   │
│  │   ProcTop → cached top-N (no scan)                        │   │
│  │   ProcFind → find_processes() (uses cache)                  │   │
│  │   ProcKill → kill_process() (fresh scan)                    │   │
│  │   ProcWatch → watch_pid() (async)                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND (QML)                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CrawlDSService                                              │   │
│  │   ├── procList()  → JSON-RPC → Vec<ProcessInfo>              │   │
│  │   ├── procTop()  → JSON-RPC → {top_by_cpu, top_by_mem}      │   │
│  │   ├── procFind() → JSON-RPC → Vec<ProcessInfo>              │   │
│  │   ├── procKill()→ JSON-RPC → ok/error                   │   │
│  │   └── procWatch()→ JSON-RPC → wait for exit               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Subscribed event stream                                      │   │
│  │   ProcEvent.TopUpdate (every 1s with top 10 by CPU/MEM)    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           BACKEND (Rust)                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-proc                                                          │   │
│  │   ├── Config           (default_sort, default_top)                   │   │
│  │   ├── list_processes() → Vec<ProcessInfo>                           │   │
│  │   ├── find_processes() → Vec<ProcessInfo>                           │   │
│  │   ├── kill_process()  → Result<(), ProcError>                       │   │
│  │   └── watch_pid()     → Result<String, ProcError>                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-daemon HTTP Router                                           │   │
│  │   GET  /proc/list        → crawlds_proc::list_processes()           │   │
│  │   GET  /proc/find        → crawlds_proc::find_processes()           │   │
│  │   POST /proc/:pid/kill   → crawlds_proc::kill_process()            │   │
│  │   GET  /proc/watch/:pid  → crawlds_proc::watch_pid() (async)       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND (QML)                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CrawlDSService                                                       │   │
│  │   ├── procList(sort, top)       → HTTP → Vec<ProcessInfo>          │   │
│  │   ├── procFind(name)            → HTTP → Vec<ProcessInfo>          │   │
│  │   ├── procKill(pid, force)      → HTTP → ok/error                  │   │
│  │   └── procWatch(pid)            → HTTP → wait for exit              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ ProcessService.qml (hypothetical)                                     │   │
│  │   ├── listProcesses(sort, top)                                       │   │
│  │   ├── searchProcesses(query)                                         │   │
│  │   ├── killProcess(pid, force)                                        │   │
│  │   └── watchProcess(pid)                                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Configuration

In `core.toml`:

```toml
[processes]
default_sort = "cpu"        # cpu | mem | pid | name
default_top = 30             # number of processes returned by default
include_cmd = false          # include command line (expensive)
top_interval_ms = 1000      # interval for top-N tracking
full_interval_ms = 5000       # interval for full scan
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `default_sort` | String | `"cpu"` | Default sort field for process list |
| `default_top` | usize | `30` | Default number of processes to return |
| `include_cmd` | bool | `false` | Include command line args (expensive) |
| `top_interval_ms` | u64 | `1000` | Interval for top-N tracking events |
| `full_interval_ms` | u64 | `5000` | Interval for full process scan |

### Sort Options

| Value | Description |
|-------|-------------|
| `cpu` | Sort by CPU usage (highest first) - **default** |
| `mem` | Sort by memory usage (highest first) |
| `pid` | Sort by PID (ascending) |
| `name` | Sort alphabetically by process name |

## JSON-RPC API

### Commands

| Command | Params | Description |
|---------|--------|-------------|
| `ProcList` | `{sort?, top?}` | List processes (uses cache) |
| `ProcTop` | `{limit?}` | Get cached top-N (no scan) |
| `ProcFind` | `{name}` | Find by name (uses cache) |
| `ProcKill` | `{pid, force}` | Kill process |
| `ProcWatch` | `{pid}` | Watch process exit |

### ProcList

List processes with sorting. Uses cached data.

**Params:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `sort` | string | `cpu` | Sort field: `cpu`, `mem`, `pid`, `name` |
| `top` | usize | `30` | Maximum processes to return |

**Example:**
```bash
echo '{"jsonrpc": "2.0", "method": "ProcList", "params": {"sort": "cpu", "top": 20}, "id": 1}' | \
  socat - unix:/run/user/1000/crawlds.sock
```

**Response:**
```json
{"jsonrpc": "2.0", "result": [...], "id": 1}
```

### ProcTop

Get cached top processes without full scan. Use for real-time UI.

**Params:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | usize | `10` | Number per list |

**Example:**
```bash
echo '{"jsonrpc": "2.0", "method": "ProcTop", "params": {"limit": 10}, "id": 1}' | \
  socat - unix:/run/user/1000/crawlds.sock
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "top_by_cpu": [...],
    "top_by_mem": [...]
  },
  "id": 1
}
```

### ProcFind

Search for processes by name (case-insensitive).

**Params:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Process name |

### ProcKill

Send signal to process. SIGTERM by default, SIGKILL if `force: true`.

**Params:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `pid` | u32 | Process ID |
| `force` | bool | Use SIGKILL |

### ProcWatch

Wait for process to exit (polls every 500ms).

## Data Types

### ProcessInfo

```rust
pub struct ProcessInfo {
    pub pid: u32,           // Process ID
    pub name: String,        // Process name
    pub cpu_percent: f32,    // CPU usage percentage
    pub mem_rss_kb: u64,     // Resident memory in KB
    pub status: String,      // Process status (e.g., "Run", "Sleep")
    pub user: Option<String>, // Username (currently None)
    pub cmd: Vec<String>,    // Command line arguments
}
```

### Process Status Values

Common status values from sysinfo:

| Status | Description |
|--------|-------------|
| `Run` | Running or ready to run |
| `Sleep` | Sleeping |
| `Stop` | Stopped by job control |
| `Zombie` | Zombie process |
| `Idle` | Idle |

### ProcError

```rust
pub enum ProcError {
    #[error("process not found: PID {0}")]
    NotFound(u32),

    #[error("permission denied killing PID {0}")]
    PermissionDenied(u32),

    #[error("signal failed: {0}")]
    SignalFailed(String),
}
```

## Events

The process domain emits events via Subscribe command.

```json
{"jsonrpc": "2.0", "method": "Subscribe", "id": 1}
```

Then events arrive as NDJSON:

```json
{"jsonrpc": "2.0", "method": "event", "params": {"domain": "proc", "data": {...}}}
```

### ProcEvent Types

#### TopUpdate

Emitted every 1s (configurable) with top processes:

```json
{
  "domain": "proc",
  "data": {
    "event": "top_update",
    "top_by_cpu": [...],
    "top_by_mem": [...]
  }
}
```

This provides real-time top 10 processes without polling.

## Backend Crate

**Location**: `core/crates/crawlds-proc/`

### ProcessCache

Global cache with incremental updates:

```rust
static PROCESS_CACHE: Lazy<Mutex<ProcessCache>> = Lazy::new(...);

struct ProcessCache {
    by_pid: HashMap<u32, ProcessInfo>,      // All processes
    sorted_by_cpu: VecDeque<ProcessInfo>,  // Top 100 by CPU
    sorted_by_mem: VecDeque<ProcessInfo>,  // Top 100 by MEM
    include_cmd: bool,
}
```

### Key Functions

```rust
// Start domain task (active)
pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) -> anyhow::Result<()>

// List processes (uses cache)
pub fn list_processes(sort_by: &str, top: usize) -> Vec<ProcessInfo>

// Find by name (uses cache)
pub fn find_processes(name: &str) -> Vec<ProcessInfo>

// Force fresh scan
pub fn list_processes_fresh(sort_by: &str, top: usize) -> Vec<ProcessInfo>

// Kill process
pub fn kill_process(pid: u32, force: bool) -> Result<(), ProcError>

// Watch exit
pub async fn watch_pid(pid: u32) -> Result<String, ProcError>
```

### Implementation Details

- Global cache (`OnceCell` + `Mutex`) shared across requests
- Domain task runs two update loops:
  - `full_refresh()` every 5s (full `/proc` scan)
  - `refresh_top()` every 1s (update existing PIDs, re-sort)
- Emits `TopUpdate` events every second with top 10 by CPU/mem
- `include_cmd: false` skips expensive cmdline parsing
- Signal handling uses SIGTERM/SIGKILL

## Dependencies

- `sysinfo` - Cross-platform process and system information
- `tokio` - Async runtime for the domain task and watch_pid
- `tracing` - Logging

## QML Usage

### Example: Process List Component

```qml
// Fetch top processes by CPU
function loadTopProcesses() {
    CrawlDSService.procList("cpu", 20, function(results) {
        processes = results
    })
}

// Search for processes
function searchProcesses(query) {
    CrawlDSService.procFind(query, function(results) {
        processes = results
    })
}

// Kill a process
function killProcess(pid) {
    CrawlDSService.procKill(pid, false, function(response) {
        if (response.ok) {
            console.log("Process", pid, "terminated")
            loadTopProcesses()
        }
    })
}

// Force kill a process
function forceKillProcess(pid) {
    CrawlDSService.procKill(pid, true, function(response) {
        if (response.ok) {
            console.log("Process", pid, "killed")
        }
    })
}
```

### Example: Watch Process Exit

```qml
// Watch for a process to exit
function watchProcess(pid) {
    CrawlDSService.procWatch(pid, function(response) {
        if (response.exit_code !== undefined) {
            console.log("Process", response.name, "exited")
        }
    }, function(error) {
        console.log("Error watching process:", error.message)
    })
}
```

## Troubleshooting

### Permission Denied When Killing Process

Processes owned by root or other users cannot be killed without elevated privileges. Try:

1. Running the daemon as root (not recommended for security)
2. Using polkit for authorization
3. Using `sudo` for specific operations

### Process Not Found

The process may have already exited. This is common when:

1. The process terminates between listing and killing
2. The PID was recycled to a new process
3. The process is in a different PID namespace

### watch_pid Never Returns

The process may be immortal or running in a different context. Consider:

1. Adding a timeout to the watch operation
2. Running watch in background with timeout

```bash
# With timeout (5 seconds)
timeout 5 curl "http://localhost/proc/watch/1234" \
  --unix-socket /run/crawlds.sock
```

### High Memory Usage from Process Listing

The cache uses ~100KB per process. With 500 processes:
- `by_pid`: ~50MB
- `sorted_by_cpu` (100): ~5MB
- `sorted_by_mem` (100): ~5MB

Mitigate by:
- Setting `include_cmd = false` (default)
- Reducing `full_interval_ms` to only scan when needed

### Zombie Processes Showing

Zombie processes (status: "Zombie") show with 0% CPU and memory. They cannot be killed directly - the parent process must reap them via `wait()`. To resolve:

1. Find and restart/kill the parent process
2. Kill the parent process to orphan the zombie (init will reap it)
