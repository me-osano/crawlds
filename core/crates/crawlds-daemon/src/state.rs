use crate::config::Config;
use crawlds_greeter::GreeterManager;
use crawlds_ipc::CrawlEvent;
use crawlds_theme::ThemeManager;
use crawlds_webservice::WebserviceState;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

/// Clipboard entry with optional pinned state
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

/// Clipboard history store - manages clipboard entries
#[allow(dead_code)]
#[derive(Clone)]
pub struct ClipboardStore {
    pub entries: Arc<Mutex<VecDeque<ClipboardEntry>>>,
    pub pinned_ids: Arc<Mutex<std::collections::HashSet<String>>>,
    pub max_entries: usize,
    pub max_pinned: usize,
}

#[allow(dead_code)]
impl ClipboardStore {
    pub fn new(max_entries: usize, max_pinned: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(max_entries))),
            pinned_ids: Arc::new(Mutex::new(std::collections::HashSet::new())),
            max_entries,
            max_pinned,
        }
    }

    pub async fn add_entry(&self, entry: ClipboardEntry) {
        let mut entries = self.entries.lock().await;
        // Deduplicate by content hash
        let content_hash = Self::hash_content(&entry.content);
        entries.retain(|e| Self::hash_content(&e.content) != content_hash);
        // Add to front
        entries.push_front(entry);
        // Trim to max
        while entries.len() > self.max_entries {
            if let Some(removed) = entries.pop_back() {
                if !self.is_pinned(&removed.id).await {
                    // Entry was removed from history
                }
            }
        }
    }

    pub async fn get_history(&self, limit: usize) -> Vec<ClipboardEntry> {
        let entries = self.entries.lock().await;
        entries.iter().take(limit).cloned().collect()
    }

    pub async fn delete_entry(&self, id: &str) -> bool {
        if self.is_pinned(id).await {
            return false; // Can't delete pinned entries
        }
        let mut entries = self.entries.lock().await;
        let initial_len = entries.len();
        entries.retain(|e| e.id != id);
        entries.len() < initial_len
    }

    pub async fn clear_history(&self) {
        let mut entries = self.entries.lock().await;
        let pinned_ids = self.pinned_ids.lock().await;
        entries.retain(|e| pinned_ids.contains(&e.id));
    }

    pub async fn pin_entry(&self, id: &str) -> Result<(), &'static str> {
        let mut pinned = self.pinned_ids.lock().await;
        if pinned.len() >= self.max_pinned {
            return Err("max pinned entries reached");
        }
        pinned.insert(id.to_string());
        // Update entry
        let mut entries = self.entries.lock().await;
        for entry in entries.iter_mut() {
            if entry.id == id {
                entry.pinned = true;
                return Ok(());
            }
        }
        Ok(())
    }

    pub async fn unpin_entry(&self, id: &str) {
        let mut pinned = self.pinned_ids.lock().await;
        pinned.remove(id);
        let mut entries = self.entries.lock().await;
        for entry in entries.iter_mut() {
            if entry.id == id {
                entry.pinned = false;
                return;
            }
        }
    }

    pub async fn is_pinned(&self, id: &str) -> bool {
        let pinned = self.pinned_ids.lock().await;
        pinned.contains(id)
    }

    pub async fn pinned_count(&self) -> usize {
        let pinned = self.pinned_ids.lock().await;
        pinned.len()
    }

    pub async fn get_entry(&self, id: &str) -> Option<ClipboardEntry> {
        let entries = self.entries.lock().await;
        entries.iter().find(|e| e.id == id).cloned()
    }

    fn hash_content(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

/// Shared application state — cloned into every axum handler via `.with_state()`.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub event_tx: broadcast::Sender<CrawlEvent>,
    pub notify_store: Arc<crawlds_notify::NotifyStore>,
    pub greeter: Arc<Mutex<GreeterManager>>,
    pub clipboard_store: Arc<ClipboardStore>,
    pub webservice_store: Arc<WebserviceState>,
    pub theme_manager: Arc<Mutex<ThemeManager>>,
}

impl AppState {
    pub fn new(
        config: Config,
        event_tx: broadcast::Sender<CrawlEvent>,
        notify_store: Arc<crawlds_notify::NotifyStore>,
    ) -> Self {
        let webservice_store = Arc::new(WebserviceState::new_with_config(
            event_tx.clone(),
            config.webservice.wallhaven.api_key.clone(),
            config.webservice.wallhaven.clone(),
        ));

        let clipboard_history_size = config.clipboard.history_size;

        let themes_dir = PathBuf::from(&config.assets_dir).join("Themes");
        let cache_dir = config.cache_dir.clone();
        let theme_manager = Arc::new(Mutex::new(ThemeManager::new(themes_dir, cache_dir)));

        Self {
            config: Arc::new(config),
            event_tx,
            notify_store,
            greeter: Arc::new(Mutex::new(GreeterManager::new())),
            clipboard_store: Arc::new(ClipboardStore::new(
                clipboard_history_size,
                25,
            )),
            webservice_store,
            theme_manager,
        }
    }
}


