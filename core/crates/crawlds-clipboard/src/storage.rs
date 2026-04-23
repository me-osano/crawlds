//! Clipboard persistent storage using sled

use fnv::FnvHasher;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

const DB_NAME: &str = "clipboard";

#[derive(Clone, Debug)]
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

pub struct Storage {
    db: sled::Db,
    max_history: usize,
    max_entry_size: usize,
}

impl Storage {
    pub fn new(
        data_dir: PathBuf,
        max_history: usize,
        max_entry_size: usize,
    ) -> anyhow::Result<Self> {
        let path = data_dir.join(DB_NAME);

        // Create directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = sled::open(&path)?;
        debug!("clipboard storage opened at {:?}", path);

        Ok(Self {
            db,
            max_history,
            max_entry_size,
        })
    }

    pub fn store(&self, content: String, mime: String) -> anyhow::Result<Option<Entry>> {
        let data = content.as_bytes();
        let size = data.len();

        if size == 0 || size > self.max_entry_size {
            return Ok(None);
        }

        let hash = compute_hash(data);
        let now = now_ms();
        let is_image = is_image_mime_type(&mime);
        let preview = make_preview(&content, &mime);

        // Dedup: remove existing entry with same hash
        let mut to_remove: Vec<sled::IVec> = Vec::new();
        for entry in self.db.iter().values() {
            if let Ok(entry) = entry {
                if let Some(existing_hash) = extract_hash(&entry) {
                    if existing_hash == hash {
                        to_remove.push(entry);
                    }
                }
            }
        }
        for entry in to_remove {
            let _ = self.db.remove(&entry);
        }

        // Generate ID
        let id = self.db.generate_id()?;

        // Build entry
        let entry = Entry {
            id,
            content,
            mime,
            preview,
            size,
            timestamp_ms: now,
            is_image,
            hash,
        };

        // Encode and store
        let key = id.to_be_bytes();
        let value = encode_entry(&entry)?;
        self.db.insert(key, value)?;

        // Trim to max history
        self.trim();

        debug!("clipboard stored id={} size={}", id, size);

        Ok(Some(entry))
    }

    fn trim(&self) {
        let mut entries: Vec<(u64, sled::IVec)> = Vec::new();

        for result in self.db.iter() {
            if let Ok((key, value)) = result {
                if let Ok(id) = key.as_ref().try_into() {
                    let id = u64::from_be_bytes(id);
                    entries.push((id, value));
                }
            }
        }

        entries.sort_by(|a, b| b.0.cmp(&a.0));

        for (i, (_, value)) in entries.iter().enumerate() {
            if i >= self.max_history {
                let _ = self.db.remove(value.as_ref());
            }
        }
    }

    pub fn list(&self) -> anyhow::Result<Vec<Entry>> {
        let mut entries: Vec<Entry> = Vec::new();

        for result in self.db.iter().values() {
            if let Ok(value) = result {
                if let Ok(entry) = decode_entry(&value) {
                    entries.push(entry);
                }
            }
        }

        entries.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
        Ok(entries)
    }

    pub fn get(&self, id: u64) -> anyhow::Result<Option<Entry>> {
        let key = id.to_be_bytes();

        if let Some(value) = self.db.get(key)? {
            let entry = decode_entry(&value)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, id: u64) -> anyhow::Result<bool> {
        let key = id.to_be_bytes();

        if let Some(value) = self.db.get(key)? {
            let _ = self.db.remove(value.as_ref())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        self.db.clear()?;
        Ok(())
    }

    pub fn contains_hash(&self, hash: u64) -> bool {
        for result in self.db.iter().values() {
            if let Ok(value) = result {
                if let Some(existing_hash) = extract_hash(&value) {
                    if existing_hash == hash {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn compute_hash(data: &[u8]) -> u64 {
    let mut hasher = FnvHasher::default();
    use std::hash::Hasher;
    hasher.write(data);
    hasher.finish()
}

fn encode_entry(e: &Entry) -> anyhow::Result<Vec<u8>> {
    use std::io::Write;

    let mut buf = Vec::new();

    buf.write_all(&e.id.to_be_bytes())?;
    buf.write_all(&(e.size as u32).to_be_bytes())?;
    buf.write_all(e.content.as_bytes())?;
    buf.write_all(&[0])?;
    buf.write_all(&(e.mime.len() as u32).to_be_bytes())?;
    buf.write_all(e.mime.as_bytes())?;
    buf.write_all(&(e.preview.len() as u32).to_be_bytes())?;
    buf.write_all(e.preview.as_bytes())?;
    buf.write_all(&e.timestamp_ms.to_be_bytes())?;
    buf.write_all(&[if e.is_image { 1 } else { 0 }])?;
    buf.write_all(&e.hash.to_be_bytes())?;

    Ok(buf)
}

fn decode_entry(data: &[u8]) -> anyhow::Result<Entry> {
    use std::io::Read;

    let mut cursor = std::io::Cursor::new(data);

    let mut id_buf = [0u8; 8];
    cursor.read_exact(&mut id_buf)?;
    let id = u64::from_be_bytes(id_buf);

    let mut size_buf = [0u8; 4];
    cursor.read_exact(&mut size_buf)?;
    let _size = u32::from_be_bytes(size_buf) as usize;

    let remaining = cursor.into_inner();
    let null_pos = remaining
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(remaining.len());
    let content = String::from_utf8_lossy(&remaining[..null_pos]).to_string();

    let after_content = &remaining[null_pos + 1..];
    let mut cursor = std::io::Cursor::new(after_content);

    let mut mime_len_buf = [0u8; 4];
    cursor.read_exact(&mut mime_len_buf)?;
    let mime_len = u32::from_be_bytes(mime_len_buf) as usize;

    let mut mime_buf = vec![0u8; mime_len];
    cursor.read_exact(&mut mime_buf)?;
    let mime = String::from_utf8_lossy(&mime_buf).to_string();

    let mut preview_len_buf = [0u8; 4];
    cursor.read_exact(&mut preview_len_buf)?;
    let preview_len = u32::from_be_bytes(preview_len_buf) as usize;

    let mut preview_buf = vec![0u8; preview_len];
    cursor.read_exact(&mut preview_buf)?;
    let preview = String::from_utf8_lossy(&preview_buf).to_string();

    let mut ts_buf = [0u8; 8];
    cursor.read_exact(&mut ts_buf)?;
    let timestamp_ms = u64::from_be_bytes(ts_buf);

    let mut is_image_buf = [0u8; 1];
    cursor.read_exact(&mut is_image_buf)?;
    let is_image = is_image_buf[0] == 1;

    let mut hash_buf = [0u8; 8];
    cursor.read_exact(&mut hash_buf)?;
    let hash = u64::from_be_bytes(hash_buf);

    let size = content.len();

    Ok(Entry {
        id,
        content,
        mime,
        preview,
        size,
        timestamp_ms,
        is_image,
        hash,
    })
}

fn extract_hash(data: &[u8]) -> Option<u64> {
    if data.len() < 8 {
        return None;
    }
    let hash_start = data.len() - 8;
    let hash_bytes = &data[hash_start..];
    Some(u64::from_be_bytes(hash_bytes.try_into().ok()?))
}

fn is_image_mime_type(mime: &str) -> bool {
    mime.len() > 6 && mime.starts_with("image/")
}

fn make_preview(content: &str, mime: &str) -> String {
    if is_image_mime_type(mime) {
        return format!("[[ image {} ]]", format_size(content.len()));
    }

    let text = content.trim();
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if text.len() > 100 {
        text[..100].to_string() + "…"
    } else {
        text.to_string()
    }
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
