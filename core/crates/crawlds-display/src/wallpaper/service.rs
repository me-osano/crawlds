//! Service layer for wallpaper management.
//!
//! The service owns the state and orchestrates backends.
//! This is the layer that IPC talks to.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::backend::{detect_backend, list_backends, WallpaperBackend};
use super::models::*;
use super::Config;
use crawlds_ipc::events::{CrawlEvent, WallpaperEvent};

/// Wallpaper service - owns state and delegates to backends.
pub struct WallpaperService {
    backend: Arc<RwLock<Box<dyn WallpaperBackend>>>,
    state: Arc<RwLock<WallpaperState>>,
    event_tx: tokio::sync::broadcast::Sender<CrawlEvent>,
    config: Config,
}

impl WallpaperService {
    /// Create a new wallpaper service with config.
    pub fn new(
        event_tx: tokio::sync::broadcast::Sender<CrawlEvent>,
        config: Config,
    ) -> Self {
        let mut backend = detect_backend();
        if let Err(e) = backend.init() {
            warn!("Failed to init wallpaper backend: {}", e);
        }

        let service = Self {
            backend: Arc::new(RwLock::new(backend)),
            state: Arc::new(RwLock::new(WallpaperState::default())),
            event_tx,
            config,
        };

        service
    }

    /// Initialize with default wallpaper if configured.
    pub async fn init_defaults(&self) -> anyhow::Result<()> {
        if let Some(ref default_path) = self.config.default_wallpaper {
            let path = std::path::Path::new(default_path);
            if path.exists() {
                let request = SetWallpaperRequest {
                    path: default_path.clone(),
                    monitor: None,
                    mode: WallpaperMode::default(),
                    transition: self.config.default_transition.clone(),
                    transition_duration_ms: self.config.transition_duration_ms,
                };
                self.set_wallpaper(request).await?;
                info!("Set default wallpaper: {}", default_path);
            }
        }
        Ok(())
    }

    /// Set wallpaper.
    pub async fn set_wallpaper(&self, request: SetWallpaperRequest) -> anyhow::Result<()> {
        // Validate path exists
        if !std::path::Path::new(&request.path).exists() {
            return Err(anyhow::anyhow!("Wallpaper file not found: {}", request.path));
        }

        // Get backend and execute
        let backend = self.backend.read().await;
        backend.set_wallpaper(request.clone())?;

        // Update state
        let mut state = self.state.write().await;
        state.set(request.path.clone(), request.monitor.clone());

        // Send event
        self.send_event(WallpaperEvent::Changed {
            screen: request.monitor.unwrap_or_else(|| "*".to_string()),
            path: request.path,
        }).await;

        Ok(())
    }

    /// Get current wallpaper state.
    pub async fn get_state(&self) -> WallpaperState {
        self.state.read().await.clone()
    }

    /// Get wallpaper for specific monitor or global.
    pub async fn get_wallpaper(&self, monitor: Option<&str>) -> Option<String> {
        let state = self.state.read().await;
        state.get(monitor).map(|s| s.to_string())
    }

    /// Get current backend info.
    pub async fn get_backend_info(&self) -> BackendInfo {
        let backend = self.backend.read().await;
        backend.info()
    }

    /// List all available backends.
    pub fn list_backends() -> Vec<BackendInfo> {
        list_backends()
    }

    /// Switch to a different backend.
    pub async fn switch_backend(&mut self, backend_name: &str) {
        let new_backend: Box<dyn WallpaperBackend> = match backend_name {
            "swww" if which::which("swww").is_ok() => Box::new(super::backend::SwwwBackend::new()),
            _ => {
                // Fallback to auto-detect
                detect_backend()
            }
        };

        let mut backend = self.backend.write().await;
        *backend = new_backend;

        info!("Switched to backend: {}", backend_name);
    }

    /// Preload a wallpaper.
    pub async fn preload(&self, path: &str) -> anyhow::Result<()> {
        let backend = self.backend.read().await;
        backend.preload(std::path::Path::new(path))
    }

    /// Clear wallpaper state.
    pub async fn clear_state(&self) {
        let mut state = self.state.write().await;
        *state = WallpaperState::default();
    }

    async fn send_event(&self, event: WallpaperEvent) {
        let crawl_event = CrawlEvent::Wallpaper(event);
        let _ = self.event_tx.send(crawl_event);
    }
}

/// Handle an IPC request synchronously.
///
/// Note: This creates a minimal runtime for sync operations.
/// For production, prefer the async service methods directly.
pub fn handle_ipc_request_sync(
    service: &WallpaperService,
    request: IpcRequest,
) -> IpcResponse {
    match request {
        IpcRequest::SetWallpaper {
            path,
            monitor,
            mode,
            transition,
        } => {
            let request = SetWallpaperRequest {
                path,
                monitor,
                mode: mode.unwrap_or_default(),
                transition: transition.unwrap_or_default(),
                transition_duration_ms: 500,
            };
            // Validate path
            if !std::path::Path::new(&request.path).exists() {
                return IpcResponse::err("Wallpaper file not found");
            }
            // Execute synchronously using blocking_read
            let backend = service.backend.blocking_read();
            match backend.set_wallpaper(request.clone()) {
                Ok(_) => {
                    // Update state (using blocking_write)
                    let mut state = service.state.blocking_write();
                    state.set(request.path.clone(), request.monitor.clone());
                    IpcResponse::ok()
                }
                Err(e) => IpcResponse::err(e.to_string()),
            }
        }
        IpcRequest::GetState => {
            let state = service.state.blocking_read().clone();
            IpcResponse::with_data(state)
        }
        IpcRequest::ListBackends => {
            IpcResponse::with_data(WallpaperService::list_backends())
        }
        IpcRequest::GetWallpaper { monitor } => {
            let wallpaper = service.state.blocking_read().get(monitor.as_deref()).map(|s| s.to_string());
            let data = serde_json::json!({ "wallpaper": wallpaper });
            IpcResponse::with_data(data)
        }
        IpcRequest::Preload { path } => {
            let backend = service.backend.blocking_read();
            match backend.preload(std::path::Path::new(&path)) {
                Ok(_) => IpcResponse::ok(),
                Err(e) => IpcResponse::err(e.to_string()),
            }
        }
    }
}