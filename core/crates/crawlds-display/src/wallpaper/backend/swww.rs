//! swww wallpaper backend implementation.

use std::path::Path;
use std::process::{Command, Stdio};
use tracing::{debug, error, info};

use super::super::models::{SetWallpaperRequest, WallpaperMode};
use super::WallpaperBackend;
use anyhow::Context;

/// swww backend configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub swww_bin: String,
    pub default_transition: String,
    pub transition_duration_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            swww_bin: "swww".to_string(),
            default_transition: "fade".to_string(),
            transition_duration_ms: 500,
        }
    }
}

/// swww wallpaper backend.
pub struct SwwwBackend {
    config: Config,
}

impl SwwwBackend {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn with_config(config: Config) -> Self {
        Self { config }
    }

    fn ensure_daemon(&self) -> anyhow::Result<()> {
        // Check if daemon is already running
        if self.is_daemon_running() {
            return Ok(());
        }

        info!("Starting swww daemon");

        // Start daemon in background
        let child = Command::new(&self.config.swww_bin)
            .arg("daemon")
            .arg("--daemonize")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn swww daemon")?;

        // Don't wait for it - let it run in background
        drop(child);

        // Small delay to let daemon start
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(())
    }
}

impl Default for SwwwBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperBackend for SwwwBackend {
    fn name(&self) -> &'static str {
        "swww"
    }

    fn init(&mut self) -> anyhow::Result<()> {
        if self.is_available() {
            // Try to ensure daemon is running
            self.ensure_daemon().ok();
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        which::which(&self.config.swww_bin).is_ok()
    }

    fn is_daemon_running(&self) -> bool {
        Command::new(&self.config.swww_bin)
            .args(["daemon", "status"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn set_wallpaper(&self, request: SetWallpaperRequest) -> anyhow::Result<()> {
        if !self.is_available() {
            return Err(anyhow::anyhow!("swww not available"));
        }

        // Ensure daemon is running
        self.ensure_daemon()?;

        let path = Path::new(&request.path);
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Wallpaper file not found: {}",
                request.path
            ));
        }

        let transition = if request.transition.is_empty() {
            &self.config.default_transition
        } else {
            &request.transition
        };

        debug!(
            "swww: setting wallpaper path={} monitor={:?} transition={}",
            request.path, request.monitor, transition
        );

        let mut cmd = Command::new(&self.config.swww_bin);
        cmd.arg("set").arg(&request.path);

        // Add monitor target
        if let Some(monitor) = &request.monitor {
            if !monitor.is_empty() {
                cmd.arg("--output").arg(monitor);
            }
        }

        // Add transition options
        cmd.arg("--transition-type").arg(transition);
        cmd.arg("--transition-duration")
            .arg(self.config.transition_duration_ms.to_string());

        // Add fill mode mapping
        let swww_fill_mode = match request.mode {
            WallpaperMode::Fill => "fill",
            WallpaperMode::Fit => "fit",
            WallpaperMode::Stretch => "stretch",
            WallpaperMode::Center => "center",
            WallpaperMode::Tile => "tile",
        };
        cmd.arg("--transition-pos").arg(swww_fill_mode);

        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().context("Failed to execute swww set")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("swww set failed: {}", stderr);
            return Err(anyhow::anyhow!("swww set failed: {}", stderr));
        }

        Ok(())
    }

    fn supports_animations(&self) -> bool {
        true
    }
}
