//! Wallpaper backend implementations and trait.

mod swww;

pub use swww::SwwwBackend;

use std::path::Path;

use super::models::{BackendInfo, SetWallpaperRequest};

/// List all available backends with their status.
pub fn list_backends() -> Vec<BackendInfo> {
    let mut backends = Vec::new();

    // swww
    let swww_available = which::which("swww").is_ok();
    let swww_running = std::process::Command::new("swww")
        .args(["daemon", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    backends.push(BackendInfo {
        name: "swww".to_string(),
        available: swww_available,
        daemon_running: swww_running,
        supports_animations: true,
    });

    backends
}

/// Detect and return the best available wallpaper backend.
pub fn detect_backend() -> Box<dyn WallpaperBackend> {
    if which::which("swww").is_ok() {
        return Box::new(SwwwBackend::new());
    }

    // Fallback to dummy backend
    Box::new(DummyBackend)
}

/// Trait that all wallpaper backends must implement.
///
/// Backends are intentionally dumb:
/// - Only execute commands
/// - No state management
/// - No IPC
pub trait WallpaperBackend: Send + Sync {
    /// Backend name (e.g., "swww", "mpvpaper").
    fn name(&self) -> &'static str;

    /// Initialize the backend (e.g., start daemon).
    fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Check if the backend is available (binary exists).
    fn is_available(&self) -> bool {
        which::which(self.name()).is_ok()
    }

    /// Check if the backend daemon is running.
    fn is_daemon_running(&self) -> bool {
        false
    }

    /// Set wallpaper using the backend.
    fn set_wallpaper(&self, request: SetWallpaperRequest) -> anyhow::Result<()>;

    /// Preload a wallpaper into cache (optional).
    fn preload(&self, _path: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    /// Whether this backend supports animated wallpapers.
    fn supports_animations(&self) -> bool {
        false
    }

    /// Get backend info.
    fn info(&self) -> BackendInfo {
        BackendInfo {
            name: self.name().to_string(),
            available: self.is_available(),
            daemon_running: self.is_daemon_running(),
            supports_animations: self.supports_animations(),
        }
    }
}

/// Dummy/no-op backend for when no real backend is available.
pub struct DummyBackend;

impl WallpaperBackend for DummyBackend {
    fn name(&self) -> &'static str {
        "dummy"
    }

    fn set_wallpaper(&self, _request: SetWallpaperRequest) -> anyhow::Result<()> {
        tracing::warn!("Dummy backend: wallpaper set called but no real backend available");
        Ok(())
    }
}
