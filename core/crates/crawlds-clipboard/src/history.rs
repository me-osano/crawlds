//! Clipboard history management with hash deduplication

use crawlds_ipc::types::ClipEntry;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const HASH_SEED: u64 = 0x1234567890abcdef;

fn compute_fnv_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = HASH_SEED;
    for &byte in data {
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= byte as u64;
    }
    hash
}

#[derive(Clone)]
pub struct ClipHistory {
    inner: Arc<Mutex<InnerHistory>>,
    capacity: usize,
}

struct InnerHistory {
    entries: VecDeque<Entry>,
    hash_set: std::collections::HashSet<u64>,
}

#[derive(Clone)]
pub struct Entry {
    pub id: u64,
    pub content: String,
    pub mime: String,
    pub preview: String,
    pub size: usize,
    pub timestamp_ms: u64,
    pub is_image: bool,
    pub hash: u64,
}

impl ClipHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerHistory {
                entries: VecDeque::with_capacity(capacity),
                hash_set: std::collections::HashSet::new(),
            })),
            capacity,
        }
    }

    pub fn push(&self, content: String, mime: String) {
        let hash = compute_fnv_hash(content.as_bytes());
        let size = content.len();

        let mut inner = self.inner.lock().unwrap();

        // Skip if identical hash exists
        if inner.hash_set.contains(&hash) {
            return;
        }

        // Enforce size limit
        if size > 5 * 1024 * 1024 {
            // 5MB
            return;
        }

        // Remove oldest entries if at capacity
        while inner.entries.len() >= self.capacity {
            if let Some(removed) = inner.entries.pop_back() {
                inner.hash_set.remove(&removed.hash);
            }
        }

        // Create entry
        let entry = Entry {
            id: 0, // Will be set by storage
            content: content.clone(),
            mime: mime.clone(),
            preview: make_preview(&content, &mime),
            size,
            timestamp_ms: now_ms(),
            is_image: is_image_mime_type(&mime),
            hash,
        };

        inner.hash_set.insert(hash);
        inner.entries.push_front(entry);
    }

    pub fn list(&self) -> Vec<ClipEntry> {
        let inner = self.inner.lock().unwrap();
        inner
            .entries
            .iter()
            .map(|e| ClipEntry {
                content: e.content.clone(),
                mime: e.mime.clone(),
                timestamp_ms: e.timestamp_ms,
            })
            .collect()
    }

    pub fn latest(&self) -> Option<ClipEntry> {
        let inner = self.inner.lock().unwrap();
        inner.entries.front().map(|e| ClipEntry {
            content: e.content.clone(),
            mime: e.mime.clone(),
            timestamp_ms: e.timestamp_ms,
        })
    }

    pub fn contains_hash(&self, hash: u64) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.hash_set.contains(&hash)
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.entries.clear();
        inner.hash_set.clear();
    }

    pub fn remove_by_hash(&self, hash: u64) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if !inner.hash_set.contains(&hash) {
            return false;
        }
        inner.entries.retain(|e| e.hash != hash);
        inner.hash_set.remove(&hash);
        true
    }
}

fn make_preview(content: &str, mime: &str) -> String {
    if is_image_mime_type(mime) {
        let size = content.len();
        return format!("[[ image {} ]]", format_size(size));
    }

    let text = content.trim();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if text.len() > 100 {
        text[..100].to_string() + "…"
    } else {
        text.to_string()
    }
}

fn is_image_mime_type(mime: &str) -> bool {
    mime.len() > 6 && mime.starts_with("image/")
}

fn format_size(size: usize) -> String {
    let units = ["B", "KiB", "MiB"];
    let mut fsize = size as f64;
    let mut i = 0;
    while fsize >= 1024.0 && i < units.len() - 1 {
        fsize /= 1024.0;
        i += 1;
    }
    format!("{:.0} {}", fsize, units[i])
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
