//! Config for clipboard domain

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum number of clipboard history entries to retain
    pub history_size: usize,
    /// Also watch the primary selection (middle-click paste)
    pub watch_primary: bool,
    /// Polling interval in milliseconds (fallback when event-driven unavailable)
    pub poll_interval_ms: u64,
    /// Use event-driven monitoring (Wayland ext_data_control)
    pub event_driven: bool,
    /// Enable persistent history storage
    pub persistent: bool,
    /// Maximum entry size in bytes
    pub max_entry_size: u64,
    /// Data directory for persistent storage
    pub data_dir: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            history_size: 100,
            watch_primary: false,
            poll_interval_ms: 500,
            event_driven: true,
            persistent: true,
            max_entry_size: 5 * 1024 * 1024, // 5MB
            data_dir: None,
        }
    }
}
