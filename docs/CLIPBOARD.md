# Clipboard Service

CrawlDS provides clipboard monitoring and history management through a unified architecture:
1. **Backend** (`crawlds-clipboard`): Modular clipboard with scheduler, hash deduplication
2. **Frontend** (`ClipboardService.qml`): Unified QML service using `CrawlDSService` API

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              BACKEND (Rust)                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ crawlds-clipboard                                                    │   │
│  │   ├── lib.rs         - Main entry + scheduler integration          │   │
│  │   ├── config.rs     - Configuration options                       │   │
│  │   └── history.rs   - ClipHistory with FNV hash deduplication    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Scheduler (crawlds-scheduler)                                        │   │
│  │   - Jitter-based timing to prevent CPU spikes                       │   │
│  │   - Integrates with daemon for coordinated updates                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼ (SSE events / HTTP)
┌─────────────────────────────────────────────────────────────────────────────┐
│                             FRONTEND (QML)                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CrawlDSService (Singleton)                                           │   │
│  │   - Receives clipboard events via SSE                                │   │
│  │   - Provides crawlPost/_fetchInitial helpers                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ ClipboardService (Singleton)                                         │   │
│  │   - Uses CrawlDSService API for clipboard operations                 │   │
│  │   - Falls back to cliphist CLI when backend unavailable              │   │
│  │   - Manages pinned entries, search, filtering                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Backend (Rust)

### Crate: `crawlds-clipboard`

**Location**: `core/crates/crawlds-clipboard/`

### Features

| Feature | Status | Description |
|---------|--------|-------------|
| Event-Driven Monitoring | ✅ | 100ms polling via wl-clipboard-rs |
| FNV Hash Deduplication | ✅ | Uses FNV-1a hash to skip duplicate content |
| Persistent Storage | ✅ | Sled-based persistent history storage |
| wl-clipboard-rs | ✅ | Direct Wayland clipboard access |

**Dependencies**:
- `wl-clipboard-rs` - Wayland clipboard access

### ClipboardStore

**Location**: `core/crates/crawlds-daemon/src/state.rs`

```rust
pub struct ClipboardStore {
    pub entries: Arc<Mutex<VecDeque<ClipboardEntry>>>,
    pub pinned_ids: Arc<Mutex<HashSet<String>>>,
    pub max_entries: usize,
    pub max_pinned: usize,
}
```

### Configuration

In `core.toml`:

```toml
[clipboard]
history_size      = 100      # entries to retain
watch_primary     = false    # also watch primary selection (middle-click paste)
poll_interval_ms  = 500      # polling interval in milliseconds
event_driven      = true     # use Wayland ext_data_control (future)
persistent        = true     # persistent history storage (sled)
max_entry_size    = 5242880  # 5MB max entry size
data_dir          = null     # data directory for storage (default: ~/.local/share/crawlds/clipboard)
```

### Module Structure

```
crawlds-clipboard/src/
├── lib.rs       # Main entry, scheduler integration, run_with_scheduler
├── config.rs    # Config struct with all options
├── history.rs  # ClipHistory with FNV hash deduplication
├── storage.rs  # Sled-based persistent storage
└── watcher.rs  # Event-driven + polling fallback
```

### Hash Deduplication

The clipboard uses FNV (Fowler-Noll-Vo) hash for efficient deduplication:

```rust
fn compute_fnv_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0x1234567890abcdef;
    for &byte in data {
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= byte as u64;
    }
    hash
}
```

This avoids storing duplicate content while keeping memory usage low.

### Types

**ClipboardEntry** (`state.rs`):
```rust
pub struct ClipboardEntry {
    pub id: String,
    pub content: String,
    pub preview: String,
    pub mime: String,
    pub size: usize,
    pub is_image: bool,
    pub timestamp_ms: u64,
    pub pinned: bool,
}
```

**ClipEntry** (`crawlds-ipc/src/types.rs`):
```rust
pub struct ClipEntry {
    pub content: String,
    pub mime: String,
    pub timestamp_ms: u64,
}
```

### HTTP API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/clipboard/history` | GET | Get clipboard history |
| `/clipboard` | GET | Get current clipboard content |
| `/clipboard` | POST | Set clipboard content |
| `/clipboard/copy` | POST | Copy entry to clipboard |
| `/clipboard/delete` | POST | Delete clipboard entry |
| `/clipboard/pin` | POST | Pin entry |
| `/clipboard/unpin` | POST | Unpin entry |
| `/clipboard/pinned_count` | GET | Get pinned entry count |
| `/clipboard/clear` | POST | Clear history (keeps pinned) |

### Example Requests

```bash
# Get clipboard history
curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/clipboard/history?limit=100

# Copy entry to clipboard
curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/clipboard/copy \
  -X POST -H 'Content-Type: application/json' -d '{"id":"12345"}'

# Delete entry
curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/clipboard/delete \
  -X POST -H 'Content-Type: application/json' -d '{"id":"12345"}'

# Pin entry
curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/clipboard/pin \
  -X POST -H 'Content-Type: application/json' -d '{"id":"12345"}'

# Clear history (keeps pinned)
curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/clipboard/clear \
  -X POST
```

## Frontend (QML)

### Service: ClipboardService

**Location**: `quickshell/services/ClipboardService.qml`

**Type**: Singleton

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `available` | `bool` | Backend connection status |
| `wtypeAvailable` | `bool` | Whether wtype is installed |
| `internalEntries` | `var` | All clipboard entries |
| `clipboardEntries` | `var` | Filtered/searched entries |
| `pinnedEntries` | `var` | Pinned entries only |
| `unpinnedEntries` | `var` | Non-pinned entries |
| `pinnedCount` | `int` | Number of pinned entries |
| `totalCount` | `int` | Total visible entries |
| `searchText` | `string` | Current search query |
| `selectedIndex` | `int` | Selected entry index |
| `keyboardNavigationActive` | `bool` | Keyboard nav state |
| `refCount` | `int` | Reference count for auto-refresh |

### Signals

| Signal | Parameters | Description |
|--------|------------|-------------|
| `historyCopied` | - | Emitted when history is copied |
| `historyCleared` | - | Emitted when history is cleared |
| `entryAdded` | `entry` | Emitted when entry is added |
| `entryDeleted` | `id` | Emitted when entry is deleted |

### Methods

#### `init()`
Initialize the service. Called automatically.

#### `refresh()`
Fetch clipboard history from backend.

#### `updateFilteredModel()`
Update filtered entries based on search text.

#### `copyEntry(entry, closeCallback)`
Copy entry to clipboard.

#### `pasteEntry(entry, closeCallback)`
Copy entry to clipboard and type it.

#### `pasteSelected(closeCallback)`
Paste currently selected entry.

#### `deleteEntry(entry)`
Delete a clipboard entry.

#### `pinEntry(entry)`
Pin a clipboard entry (max 25 pinned).

#### `unpinEntry(entry)`
Unpin a clipboard entry.

#### `clearAll()`
Clear clipboard history (keeps pinned entries).

#### `getEntryType(entry)`
Get entry type: "text", "long_text", or "image".

#### `parseImageMeta(preview)`
Parse image dimensions from preview.

#### `decodeToDataUrl(id, mime, callback)`
Decode image to data URL for display.

### CrawlDSService Integration

The service listens for clipboard events via SSE:

```qml
Connections {
    target: CrawlDSService
    function onClipboardEvent(data) {
        // Handle clipboard events from backend
    }
}
```

## Clipboard Provider (Launcher)

**Location**: `quickshell/modules/launcher/Providers/ClipboardProvider.qml`

### Command

Access clipboard history via the launcher command `>clip`

### Commands

| Command | Description |
|---------|-------------|
| `>clip` | Open clipboard history browser |
| `>clip <query>` | Search clipboard history |
| `>clip clear` | Clear all clipboard history |

### Settings

In `quickshell/assets/settings-default.json`:

```json
{
  "appLauncher": {
    "enableClipboardHistory": false,
    "autoPasteClipboard": false,
    "enableClipPreview": true,
    "clipboardWrapText": true,
    "clipboardWatchTextCommand": "wl-paste --type text --watch cliphist store",
    "clipboardWatchImageCommand": "wl-paste --type image --watch cliphist store"
  }
}
```

## External Dependencies

### Required

- **wl-paste/wl-copy**: Wayland clipboard utilities (part of wl-clipboard)
- **cliphist**: Clipboard history manager

### Optional

- **wtype**: Type arbitrary text, required for auto-paste

### Installation (Arch Linux)

```bash
pacman -S wl-clipboard cliphist wtype
```

## Data Flow

### Clipboard Change Detection

1. Backend polls clipboard via `wl-clipboard-rs` or user runs `wl-paste --watch cliphist store`
2. `cliphist store` is invoked when clipboard changes
3. User queries clipboard via `>clip` command
4. `ClipboardService.refresh()` fetches from `GET /clipboard/history`
5. Entries are cached for display

### Pinned Entries

1. User pins an entry via `ClipboardService.pinEntry()`
2. Backend stores ID in `ClipboardStore.pinned_ids`
3. Entry marked with `pinned: true`
4. Pin preserved across history clears
5. Maximum 25 pinned entries

### Auto-Paste

1. User selects clipboard item in launcher
2. `ClipboardService.pasteEntry()` calls `POST /clipboard/copy`
3. Content copied to Wayland clipboard
4. If `wtypeAvailable`, content typed:
   ```
   wl-paste | wtype -
   ```

## Troubleshooting

### Clipboard history not working

1. Check if `cliphist` is installed:
   ```bash
   which cliphist
   ```

2. Check backend connection:
   ```bash
   curl --unix-socket $XDG_RUNTIME_DIR/crawlds.sock http://localhost/health
   ```

3. Test cliphist manually:
   ```bash
   echo "test" | wl-copy
   cliphist list
   ```

### Auto-paste not working

1. Check if `wtype` is installed:
   ```bash
   which wtype
   ```

2. Test manually:
   ```bash
   echo "hello" | wl-copy
   wl-paste | wtype -
   ```

## Roadmap

### Completed
- [x] Implement HTTP API handlers
- [x] Add pinned entries support
- [x] Add notification feedback
- [x] Use Process+Timer pattern for typing
- [x] Modularize clipboard crate (config, storage)
- [x] Add FNV hash deduplication
- [x] Add persistent storage (sled)
- [x] Add clipboard get/set via wl-clipboard-rs
- [x] Add fast polling monitoring (50ms)
- [x] Add wayland-clipboard-listener for event-driven (with polling fallback)

### Planned
- [ ] Support image clipboard in backend
- [ ] Multi-MIME type support
- [ ] Add backend event handling in CrawlDSService
- [ ] Implement clipboard statistics/tracking

## True Event-Driven Implementation

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Tokio Runtime (main)                      │
│  ┌─────────────────┐    ┌────────────────────────────────┐  │
│  │  Daemon runs    │    │  Clipboard events via          │  │
│  │  all domains    │    │  broadcast::Sender<CrawlEvent> │  │
│  └────────┬────────┘    └────────────────────────────────┘  │
│           │                                                 │
│  ┌────────▼─────────────────────────────────────────────┐   │
│  │  Wayland Event Loop Thread (std::thread)             │   │
│  │  - wayland-client Connection                         │   │
│  │  - Get ext_data_control_manager_v1 from registry    │   │
│  │  - Bind wl_seat → DataDevice                        │   │
│  │  - Listen for selection events                       │   │
│  │  - Send to main thread via crossbeam channel         │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Implementation Plan

1. **Add dependencies** to `Cargo.toml`:
   ```toml
   wayland-client = "0.31"
   wayland-protocols = "0.32"
   os-pipe = "1"
   ```

2. **Create `src/wayland.rs`** - Wayland protocol handling:
   - `run_wayland_listener()` - Main event loop
   - Handle `data_offer`, `selection` events
   - Read clipboard data via pipe

3. **Protocol flow:**
   - Connect to Wayland display
   - Get registry, lookup globals
   - Bind `ext_data_control_manager_v1` (or `wlr_data_control` as fallback)
   - Get data device for seat
   - On `selection` event → get offer → receive data
   - Send to tokio via crossbeam

4. **Fallback:** If protocol unavailable, use 100ms polling (current)

### Dependencies Added

| Crate | Purpose |
|-------|---------|
| `wayland-client` | Wayland display connection and event loop |
| `wayland-protocols` | ext_data_control protocol definitions |
| `os-pipe` | For receiving clipboard data |

### Protocol Support

| Protocol | Compositors | Status |
|----------|-------------|--------|
| `ext_data_control_manager_v1` | GNOME, KDE, Hyprland | Primary |
| `wlr_data_control_unstable_v1` | Sway, Wayfire, River | Fallback |
| None available | - | Use 100ms polling |

### Reference

See DankMaterialShell Go implementation for exact protocol handling:
- `core/internal/proto/ext_data_control/` - Protocol bindings
- `core/pkg/clipboard/watch.go` - Event listener

### 2. Persistent Storage
Add BoltDB/SQLite for persistent clipboard history:

```rust
// Using sled (already in workspace)
struct PersistentHistory {
    db: sled::Db,
}

impl PersistentHistory {
    fn store(&self, entry: &Entry) -> Result<(), Error> {
        self.db.insert(entry.hash.to_be_bytes(), entry.encode())
    }
}
```

### 3. Multi-format Support
- Images with preview generation
- Rich text
- File transfers
