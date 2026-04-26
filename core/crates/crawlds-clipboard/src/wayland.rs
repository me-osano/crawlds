//! Wayland event-driven clipboard monitoring via ext_data_control protocol
//!
//! Uses wayland-clipboard-listener crate for true event-driven monitoring.

use crate::storage::Storage;
use crate::ClipEntry;
use fnv::FnvHasher;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use tracing::{debug, error, info};

pub fn run_wayland_listener(
    history: Arc<std::sync::Mutex<HashSet<u64, fnv::FnvBuildHasher>>>,
    storage: Option<Arc<Storage>>,
    tx: crossbeam::channel::Sender<ClipEntry>,
) {
    info!("clipboard: starting Wayland event-driven listener (ext_data_control)");

    // Try to use wayland-clipboard-listener for true event-driven
    match run_event_driven(&history, &storage, &tx) {
        Ok(_) => {
            info!("clipboard: event-driven listener exited normally");
        }
        Err(e) => {
            error!(
                "clipboard: event-driven failed: {:?}, falling back to polling",
                e
            );
            run_poll_fallback(history, storage, tx);
        }
    }
}

fn run_event_driven(
    history: &Arc<std::sync::Mutex<HashSet<u64, fnv::FnvBuildHasher>>>,
    storage: &Option<Arc<Storage>>,
    tx: &crossbeam::channel::Sender<ClipEntry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use wayland_clipboard_listener::{WlClipboardPasteStream, WlListenType};

    info!("clipboard: initializing wayland-clipboard-listener");

    let mut stream = WlClipboardPasteStream::init(WlListenType::ListenOnCopy)?;

    for _context_result in stream.paste_stream().flatten() {
        // Use wl-clipboard-rs to read the actual clipboard content
        // The wayland-clipboard-listener just signals when clipboard changes
        use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};

        let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);

        let (content, mime) = match result {
            Ok((mut reader, mime)) => {
                use std::io::Read;
                let mut content = String::new();
                match reader.read_to_string(&mut content) {
                    Ok(_) if !content.is_empty() => (content, mime.to_string()),
                    _ => continue,
                }
            }
            Err(e) => {
                debug!("clipboard: failed to read content: {:?}", e);
                continue;
            }
        };

        let hash = compute_hash(content.as_bytes());

        // Check in-memory duplicate
        {
            let hist = history.lock().unwrap();
            if hist.contains(&hash) {
                debug!("clipboard: duplicate in memory, skipping");
                continue;
            }
        }

        // Check storage duplicate
        if let Some(ref store) = storage.as_ref() {
            if store.contains_hash(hash) {
                debug!("clipboard: duplicate in storage, skipping");
                continue;
            }
        }

        debug!("clipboard: event-driven change ({} bytes)", content.len());

        // Update in-memory hash
        {
            let mut hist = history.lock().unwrap();
            hist.insert(hash);
        }

        // Store to persistent storage
        let entry = if let Some(ref store) = storage.as_ref() {
            match store.store(content.clone(), mime.clone()) {
                Ok(Some(e)) => ClipEntry {
                    content: e.content,
                    mime: e.mime,
                    timestamp_ms: e.timestamp_ms,
                },
                _ => continue,
            }
        } else {
            ClipEntry {
                content: content.clone(),
                mime,
                timestamp_ms: now_ms(),
            }
        };

        if tx.send(entry).is_err() {
            debug!("clipboard: receiver dropped");
            break;
        }
    }

    Ok(())
}

pub fn run_poll_fallback(
    history: Arc<std::sync::Mutex<HashSet<u64, fnv::FnvBuildHasher>>>,
    storage: Option<Arc<Storage>>,
    tx: crossbeam::channel::Sender<ClipEntry>,
) {
    info!("clipboard: starting polling fallback (500ms)");

    let mut last_content = String::new();
    let poll_interval = std::time::Duration::from_millis(500);
    let storage = storage.as_ref();

    loop {
        thread::sleep(poll_interval);

        use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};

        let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);

        if let Ok((mut reader, mime)) = result {
            use std::io::Read;
            let mut content = String::new();

            if reader.read_to_string(&mut content).is_ok()
                && !content.is_empty()
                && content != last_content
            {
                last_content = content.clone();

                let hash = compute_hash(content.as_bytes());

                {
                    let hist = history.lock().unwrap();
                    if hist.contains(&hash) {
                        continue;
                    }
                }

                if let Some(ref store) = storage {
                    if store.contains_hash(hash) {
                        continue;
                    }
                }

                debug!("clipboard: polling change ({} bytes)", content.len());

                {
                    let mut hist = history.lock().unwrap();
                    hist.insert(hash);
                }

                let entry = if let Some(ref store) = storage {
                    match store.store(content.clone(), mime.to_string()) {
                        Ok(Some(e)) => ClipEntry {
                            content: e.content,
                            mime: e.mime,
                            timestamp_ms: e.timestamp_ms,
                        },
                        _ => continue,
                    }
                } else {
                    ClipEntry {
                        content: content.clone(),
                        mime,
                        timestamp_ms: now_ms(),
                    }
                };

                let _ = tx.send(entry);
            }
        }
    }
}

fn compute_hash(data: &[u8]) -> u64 {
    let mut hasher = FnvHasher::default();
    use std::hash::Hasher;
    hasher.write(data);
    hasher.finish()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
