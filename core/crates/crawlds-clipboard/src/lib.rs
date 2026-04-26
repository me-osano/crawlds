//! crawlds-clipboard: Wayland clipboard access
//!
//! Features:
//! - Fast polling clipboard monitoring (50ms default)
//! - FNV hash-based deduplication
//! - Persistent storage (sled)
//! - wl-clipboard-rs integration
//!
//! Note: True event-driven via Wayland ext_data_control requires complex
//! wayland-client dispatch setup. Current implementation uses 50ms polling
//! which provides near-instant detection with simpler code.

pub mod config;
pub mod storage;
pub mod wayland;

pub use config::Config;
pub use storage::{Entry, Storage};

use crawlds_ipc::events::{ClipboardEvent, CrawlEvent};
use crawlds_ipc::types::ClipEntry;
use fnv::FnvHasher;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) -> anyhow::Result<()> {
    run_with_scheduler(cfg, tx).await
}

pub async fn run_with_scheduler(
    cfg: Config,
    tx: broadcast::Sender<CrawlEvent>,
) -> anyhow::Result<()> {
    info!("crawlds-clipboard starting (persistent={}, event_driven={})", 
        cfg.persistent, cfg.event_driven);

    // Check Wayland display
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        warn!("WAYLAND_DISPLAY not set — clipboard domain will be inactive");
        std::future::pending::<()>().await;
        return Ok(());
    }

    // Initialize storage
    let storage: Option<Arc<Storage>> = if cfg.persistent {
        let data_dir = cfg.data_dir
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| get_default_data_dir());
        
        Some(Arc::new(
            Storage::new(data_dir, cfg.history_size, cfg.max_entry_size as usize)?
        ))
    } else {
        None
    };

    // In-memory history for quick lookups (sync Mutex for thread safety)
    let history: Arc<std::sync::Mutex<HashSet<u64, fnv::FnvBuildHasher>>> = 
        Arc::new(std::sync::Mutex::new(HashSet::default()));

    // Load existing entries into hash set
    if let Some(ref store) = storage {
        if let Ok(entries) = store.list() {
            let mut hist = history.lock().unwrap();
            for entry in entries {
                hist.insert(entry.hash);
            }
            debug!("clipboard: loaded {} existing entries", hist.len());
        }
    }

    // Create channel for clipboard events
    let (clip_tx, clip_rx) = crossbeam::channel::bounded(32);

    // Spawn clipboard monitoring thread
    let history_clone = history.clone();
    let storage_clone = storage.clone();
    
    thread::spawn(move || {
        if cfg.event_driven {
            // True event-driven via Wayland
            crate::wayland::run_wayland_listener(history_clone, storage_clone, clip_tx);
        } else {
            // Polling fallback
            crate::wayland::run_poll_fallback(history_clone, storage_clone, clip_tx);
        }
    });

    // Main loop: receive clipboard events and forward to broadcast
    loop {
        match clip_rx.recv() {
            Ok(entry) => {
                let _ = tx.send(CrawlEvent::Clipboard(ClipboardEvent::Changed { entry }));
            }
            Err(_) => {
                debug!("clipboard: channel closed, exiting");
                break;
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn compute_hash(data: &[u8]) -> u64 {
    let mut hasher = FnvHasher::default();
    use std::hash::Hasher;
    hasher.write(data);
    hasher.finish()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn get_default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crawlds")
        .join("clipboard")
}

pub fn get() -> Result<Option<ClipEntry>, Box<dyn std::error::Error + Send + Sync>> {
    use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
    use std::io::Read;
    
    let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);
    match result {
        Ok((mut reader, mime)) => {
            let mut content = String::new();
            reader.read_to_string(&mut content)?;
            Ok(Some(ClipEntry {
                content,
                mime,
                timestamp_ms: now_ms(),
            }))
        }
        Err(_) => Ok(None),
    }
}

pub async fn set(text: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use wl_clipboard_rs::copy::{copy, Options, Source};
    
    let options = Options::new();
    let source = Source::Bytes(text.into_bytes().into());
    copy(options, source, wl_clipboard_rs::copy::MimeType::Text)?;
    Ok(())
}

pub fn set_sync(text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use wl_clipboard_rs::copy::{copy, Options, Source};
    
    let options = Options::new();
    let source = Source::Bytes(text.as_bytes().to_vec().into());
    copy(options, source, wl_clipboard_rs::copy::MimeType::Text)?;
    Ok(())
}