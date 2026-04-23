//! crawlds-vfs: Virtual filesystem - disks, mounts, file operations, search.
//!
//! Modules:
//! - `disk` - UDisks2 block device management (mount/unmount/eject)
//! - `fs` - disk usage monitoring, file operations (list, info)
//! - `search` - Tantivy-based full-text search
//! - `ops` - file operations (copy, move, delete, rename, trash)
//! - `watcher` - file system watching with notify crate

pub mod disk;
pub mod error;
pub mod fs;
pub mod ops;
pub mod preload;
pub mod search;
pub mod types;
pub mod watcher;

pub use error::VfsError;

// Re-export public API
pub use disk::{eject, list_devices, mount, unmount};
pub use fs::{
    cache_dir, config_dir, copy_files, create_directory, create_file, delete_file, exists,
    file_info, get_disk_usage, home_dir, is_directory, list_dir, get_trash_list,
    move_files, rename_file, move_to_trash, Entry, EntryKind,
};
pub use ops::{copy, copy_with_progress, move_entry, delete as delete_path, rename as rename_path, Progress, ProgressKind, ProgressStatus, ProgressSender};
pub use preload::{Preloader, PreloadManager, PreloadStrategy, PreloaderStats};
pub use search::{SearchEngine, SearchHit, search, search_home};
pub use watcher::FsWatcher;

use crawlds_ipc::events::{CrawlEvent, DiskEvent};
use watcher::watcher_to_fs_event;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::info;

// ── Config ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub removable_only: bool,
    pub auto_mount: bool,
    pub search_max_results: usize,
    pub disk_usage_interval_secs: u64,
    pub watch_paths: Vec<String>,
    pub index_path: Option<String>,
}

// ── Domain runner ───────────────────────────────────────────────────────────

pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) -> anyhow::Result<()> {
    info!("crawlds-vfs starting");

    let conn = zbus::Connection::system().await?;

    let obj_manager_proxy = zbus::fdo::ObjectManagerProxy::builder(&conn)
        .destination("org.freedesktop.UDisks2")?
        .path("/org/freedesktop/UDisks2")?
        .build()
        .await?;

    let mut interfaces_added = obj_manager_proxy.receive_interfaces_added().await?;
    let mut interfaces_removed = obj_manager_proxy.receive_interfaces_removed().await?;

    info!("crawlds-vfs: watching UDisks2 for block device events");

    // File system watcher
    let mut fs_watcher = match FsWatcher::new() {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("failed to create fs watcher: {}", e);
            return Err(anyhow::anyhow!("fs watcher init failed"));
        }
    };

    // Watch configured paths
    for watch_path in &cfg.watch_paths {
        if let Err(e) = fs_watcher.watch(PathBuf::from(watch_path).as_ref()) {
            tracing::warn!("failed to watch {}: {}", watch_path, e);
        }
    }

    // Spawn file watcher event forwarder
    let watcher_tx = fs_watcher.tx();
    let tx_clone = tx.clone();
    
    // Create preloader manager
    let preloader = PreloadManager::new(100, 5);
    let preloader_clone = preloader.clone();
    
    tokio::spawn(async move {
        let mut rx = watcher_tx.subscribe();
        loop {
            if let Ok(event) = rx.recv().await {
                let fs_event = watcher_to_fs_event(event.clone());
                
                // Invalidate preloader cache on changes
                if let Some(path) = fs_event.path() {
                    preloader_clone.handle_fs_change(&path).await;
                }
                
                let _ = tx_clone.send(CrawlEvent::Disk(DiskEvent::FsChanged {
                    fs_event,
                }));
            }
        }
    });

    // Spawn disk usage periodic updates
    let disk_cfg = cfg.clone();
    let disk_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(disk_cfg.disk_usage_interval_secs)).await;
            if let Ok(usage) = fs::get_disk_usage().await {
                let _ = disk_tx.send(CrawlEvent::Disk(DiskEvent::DiskUsageUpdated { usage }));
            }
        }
    });

    // Handle device add/remove events
    loop {
        tokio::select! {
            Some(signal) = interfaces_added.next() => {
                let args = signal.args()?;
                let path = args.object_path.to_string();
                if path.contains("/block_devices/") {
                    let dev_result = disk::build_block_device(&conn, &path).await;
                    if let Ok(dev) = dev_result
                        && (!disk_cfg.removable_only || dev.removable)
                    {
                        info!(device = %dev.device, "block device added");
                        let _ = tx.send(CrawlEvent::Disk(DiskEvent::DeviceAdded { device: dev.clone() }));
                        if disk_cfg.auto_mount && dev.removable && !dev.mounted {
                            if let Err(e) = disk::mount(&dev.device).await {
                                tracing::warn!("auto-mount failed: {e}");
                            }
                        }
                    }
                }
            }
            Some(signal) = interfaces_removed.next() => {
                let args = signal.args()?;
                let path = args.object_path.to_string();
                if path.contains("/block_devices/") {
                    info!(path = %path, "block device removed");
                    let _ = tx.send(CrawlEvent::Disk(DiskEvent::DeviceRemoved { device_path: path }));
                }
            }
        }
    }
}