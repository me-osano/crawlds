//! Wallpaper management subsystem.
//!
//! Architecture:
//! - `backend/` - Pluggable backend implementations (swww, etc.)
//! - `models.rs` - Shared domain types
//! - `service.rs` - Service layer that owns state and orchestrates backends
//!
//! Design principles:
//! - Backend is dumb (only executes commands)
//! - Service owns state (current wallpaper, monitor mapping)
//! - IPC talks only to service

pub mod backend;
pub mod models;
pub mod service;

use serde::{Deserialize, Serialize};

pub use backend::WallpaperBackend;
pub use service::WallpaperService;
pub use service::handle_ipc_request_sync;

// Re-export models at crate root for convenience
pub use models::BackendInfo;
pub use models::IpcRequest;
pub use models::IpcResponse;
pub use models::SetWallpaperRequest;
pub use models::WallpaperMode;
pub use models::WallpaperState;

/// Wallpaper configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_swww_bin")]
    pub swww_bin: String,
    #[serde(default = "default_transition")]
    pub default_transition: String,
    #[serde(default = "default_duration")]
    pub transition_duration_ms: u64,
    #[serde(default)]
    pub default_wallpaper: Option<String>,
    #[serde(default)]
    pub auto_generate_theme: bool,
}

fn default_swww_bin() -> String {
    "swww".to_string()
}

fn default_transition() -> String {
    "fade".to_string()
}

fn default_duration() -> u64 {
    500
}

impl Default for Config {
    fn default() -> Self {
        Self {
            swww_bin: default_swww_bin(),
            default_transition: default_transition(),
            transition_duration_ms: default_duration(),
            default_wallpaper: None,
            auto_generate_theme: true,
        }
    }
}