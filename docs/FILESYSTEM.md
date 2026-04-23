# CrawlDS VFS (Virtual Filesystem)

The CrawlDS VFS system provides file management, disk operations, and search functionality through a Rust backend and QML frontend integration.

## Architecture Overview

```
Quickshell/QML (UI Layer)
        ↕ HTTP/Unix Socket
crawlds-vfs (Rust Library)
        ↕
  ├─ disk/     → UDisks2 (D-Bus)
  ├─ fs/       → tokio::fs
  ├─ ops/      → file operations
  ├─ search/   → Tantivy search
  └─ watcher/  → notify crate
```

## Crate: `crawlds-vfs`

### Modules

| Module | Purpose |
|--------|---------|
| `disk` | UDisks2 D-Bus integration for block devices |
| `fs` | Directory listing, disk usage, Entry type |
| `ops` | Copy, move, delete, rename, trash operations |
| `search` | Tantivy-powered full-text search |
| `watcher` | File system watching with notify crate |
| `error` | VfsError enum |
| `types` | Re-exports from crawlds-ipc |

### Public API

```rust
// Disk management (UDisks2)
pub use disk::{list_devices, mount, unmount, eject};

// File operations
pub use fs::{
    list_dir,           // List directory contents
    get_disk_usage,    // Get disk usage for all mounts
    copy_files,        // Copy files/directories
    move_files,        // Move files/directories
    delete_file,       // Delete files/directories
    rename_file,       // Rename files/directories
    trash_files,       // Move to trash
    create_directory,  // Create directory
    Entry,             // File/directory entry type
    EntryKind,         // File, Directory, Symlink
};

// Search
pub use search::{search, search_home, SearchEngine, SearchHit};

// Configuration
pub use watcher::{FsWatcher, FsEvent};
```

### Configuration

```rust
pub struct Config {
    pub removable_only: bool,          // Only track removable devices
    pub auto_mount: bool,              // Auto-mount removable devices
    pub search_max_results: usize,      // Maximum search results
    pub disk_usage_interval_secs: u64,  // Disk usage update interval
    pub watch_paths: Vec<String>,      // Paths to watch for changes
    pub index_path: Option<String>,    // Tantivy index path
}
```

### Events

The VFS module emits `DiskEvent` via the crawler event system:

```rust
pub enum DiskEvent {
    DeviceMounted { device: BlockDevice },
    DeviceUnmounted { device_path: String },
    DeviceAdded { device: BlockDevice },
    DeviceRemoved { device_path: String },
    DiskUsageUpdated { usage: Vec<DiskUsage> },
    FsChanged { fs_event: FsEvent },  // File system change events
    OperationProgress {
        operation_id: String,
        operation_kind: String,
        current_file: String,
        processed_bytes: u64,
        total_bytes: u64,
        files_processed: u32,
        total_files: u32,
        percent: f64,
        status: String,
    },
}
```

### Progress Reporting

File operations (copy, move, delete) emit progress events via SSE:

```rust
// Backend emits progress during operations
DiskEvent::OperationProgress {
    operation_id: "copy_abc123",
    operation_kind: "Copy",
    current_file: "/home/user/file.txt",
    processed_bytes: 1024,
    total_bytes: 10240,
    files_processed: 1,
    total_files: 5,
    percent: 10.0,
    status: "InProgress",
}
```

**Frontend consumption:**

```qml
Connections {
    target: CrawlDSService
    function onFsEvent(data) {
        if (data.event === "operation_progress") {
            // Update progress UI
            updateProgress(data.percent, data.current_file)
        }
    }
}
```

## Daemon Integration

### Routes

| Method | Path | Description |
|--------|------|-------------|
| GET | `/disk/list` | List removable devices |
| POST | `/disk/mount` | Mount a device |
| POST | `/disk/unmount` | Unmount a device |
| POST | `/disk/eject` | Eject a device |
| GET | `/vfs/disk-usage` | Get disk usage for all mounts |
| GET | `/vfs/list?path=<path>` | List directory contents |
| GET | `/vfs/search?q=<query>&root=<path>&max_results=<n>` | Search files |
| POST | `/vfs/copy` | Copy files |
| POST | `/vfs/move` | Move files |
| POST | `/vfs/delete` | Delete files |
| POST | `/vfs/trash` | Move to trash |
| POST | `/vfs/rename` | Rename file |
| POST | `/vfs/mkdir` | Create directory |

### Example Request/Response

```bash
# List directory
curl --unix-socket /run/user/1000/crawlds.sock http://localhost/vfs/list?path=%2Fhome%2Fenosh

# Search
curl --unix-socket /run/user/1000/crawlds.sock "http://localhost/vfs/search?q=config&max_results=20"

# Copy file
curl -X POST --unix-socket /run/user/1000/crawlds.sock http://localhost/vfs/copy \
  -H "Content-Type: application/json" \
  -d '{"source": "/home/enosh/file.txt", "destination": "/home/enosh/backup/"}'
```

## Quickshell Integration

### CrawlDSService

The main `CrawlDSService` provides VFS properties and handlers:

```qml
// Properties
property var diskUsage: []           // Disk usage for all mounts
property var removableDevices: []    // Removable devices
signal fsChanged(var event)          // File system change events

// Handlers
function _handleDiskUsage(data)      // Handle disk usage updates
function _handleRemovableDevices(data) // Handle device list
function _handleVfsEvent(data)       // Handle VFS events
```

### VFSService

A dedicated singleton service for file operations:

```qml
// Navigation
function navigate(path)   // Navigate to directory
function refresh()       // Refresh current directory
function goUp()          // Navigate to parent
function goHome()        // Navigate to home

// Operations
function copyFile(src, dst)
function moveFile(src, dst)
function deleteFile(path)
function trashFile(path)
function renameFile(path, newName)
function createFolder(name)

// Search
function search(query, maxResults)
function searchInFolder(query, folder, maxResults)

// Utility
function formatSize(bytes)  // Format byte size
function openPath(path)    // Open with xdg-open

// Progress support
signal progressChanged(var data)
```

### Progress Events

```qml
// Listen for file operation progress
Connections {
    target: VFSService
    function onProgressChanged(progressData) {
        // progressData: { percent, currentFile, bytesCopied, totalBytes, ... }
    }
}
```

### Usage Example

```qml
import qs.service.core

Rectangle {
    Button {
        text: "Open Home"
        onClicked: VFSService.goHome()
    }
    
    Button {
        text: "Create Folder"
        onClicked: VFSService.createFolder("New Folder")
    }
    
    TextInput {
        onAccepted: VFSService.search(text, 50)
    }
    
    Connections {
        target: VFSService
        function onEntriesChanged(entries) {
            // Update file list view
        }
    }
}
```

## File Watching

The VFS module watches configured paths for file system changes:

```rust
// Config
let config = Config {
    watch_paths: vec![
        std::env::var("HOME").unwrap(),
        "/home/enosh/Documents".to_string(),
    ],
    ..Default::default()
};
```

Changes are pushed to the frontend via SSE events with the `fs_changed` event type.

## Search Engine

Uses Tantivy for full-text search with the following schema:

| Field | Type | Description |
|-------|------|-------------|
| path | STRING | Full file path |
| name | TEXT | File name (searchable) |
| body | TEXT | File content (optional) |
| ext | STRING | File extension |
| mime | STRING | MIME type |
| size | U64 | File size |
| mtime | I64 | Modification timestamp |

### Index Location

Default index path: `$XDG_DATA_HOME/crawlds/search_index` (or `~/.local/share/crawlds/search_index`)

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `zbus` | D-Bus (UDisks2) |
| `notify` | File watching |
| `tantivy` | Search engine |
| `trash` | Trash integration |
| `infer` | MIME type detection |
| `walkdir` | Directory traversal |

## Error Handling

```rust
pub enum VfsError {
    DBus(#[from] zbus::Error),
    NotFound(String),
    MountFailed(String),
    UnmountFailed(String),
    Io(#[from] std::io::Error),
    SearchFailed(String),
    WatchError(String),
    OperationFailed(String),
    TrashFailed(String),
    IndexError(String),
}
```

All errors are converted to API responses:

```json
{
  "error": "vfs",
  "code": "operation_failed",
  "message": "File operation failed: permission denied"
}
```

---

## Roadmap: VFS-Only Files UI

### Current State

The Files UI (`quickshell/modules/files/`) supports two data sources:
- `dataSource: "qml"` - QML FolderListModel (default, working)
- `dataSource: "vfs"` - VFS backend via VFSService (scaffolded, not wired)

### Goal

Migrate entirely to VFS backend for:
- Unified file operations via Rust backend
- Progress reporting for copy/move
- File watcher integration
- Search integration

### TODO

#### Phase 1: VFS Model Wiring (Priority: High)

- [ ] Change `dataSource` default from `"qml"` to `"vfs"`
- [ ] Update GridView delegate to use VFS model properties
  - `fileName` → `fileName` ✓
  - `filePath` → `filePath` ✓
  - `fileIsDir` → `fileIsDir` ✓
  - Add: `fileSize`, `fileModified`, `fileExtension`
- [ ] Update ListView delegate similarly
- [ ] Connect folder model count to `vfsModel.count`

#### Phase 2: File Operations (Priority: High)

- [ ] Connect copy operation to VFSService.copyFile()
- [ ] Connect move operation to VFSService.moveFile()
- [ ] Connect delete to VFSService.deleteFile()
- [ ] Connect trash to VFSService.trashFile()
- [ ] Connect rename to VFSService.renameFile()
- [ ] Connect create folder to VFSService.createFolder()

#### Phase 3: Progress Integration (Priority: Medium)

- [ ] Connect VFSService.progressChanged signal to UI
- [ ] Add progress overlay component
- [ ] Show progress during copy/move operations

#### Phase 4: Search Integration (Priority: Medium)

- [ ] Add search bar to FilesContent
- [ ] Connect to VFSService.search()
- [ ] Display search results in file browser

#### Phase 5: File Watcher Integration (Priority: Low)

- [ ] Connect CrawlDSService.fsChanged signal
- [ ] Auto-refresh on file system changes
- [ ] Show notification on external changes

#### Phase 6: Cleanup (Priority: Medium)

- [ ] Remove FolderListModel dependency
- [ ] Remove `dataSource` property
- [ ] Verify all QML folder list references use vfsModel

### Files to Modify

```
quickshell/modules/files/
  FilesContent.qml       # dataSource default, model wiring
  FilesGridDelegate.qml    # VFS property mapping
  FilesListDelegate.qml    # VFS property mapping
  FilesInfo.qml          # Use VFS file info API
  FilesNavigation.qml    # Connect to VFSService
```

### Testing Checklist

- [ ] Navigate to directories via VFS
- [ ] Sort files (name/size/modified/type)
- [ ] Copy files with progress
- [ ] Delete files
- [ ] Search files
- [ ] Large directories (1000+ files) performance