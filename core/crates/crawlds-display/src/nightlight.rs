//! Nightlight (blue light filter) control via wayland-native or redshift.
//!
//! Supports:
//! - wayland-native (KDE, GNOME, sway)
//! - redshift (X11/Wayland)

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub temperature_k: u32,
    pub auto_adjust: bool,
    pub transition_secs: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            temperature_k: 4500,
            auto_adjust: true,
            transition_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightlightStatus {
    pub enabled: bool,
    pub temperature_k: u32,
    pub available: bool,
}

#[derive(Debug, Error)]
pub enum NightlightError {
    #[error("nightlight not available")]
    NotAvailable,
    #[error("failed to adjust nightlight: {0}")]
    AdjustFailed(String),
    #[error("backend error: {0}")]
    Backend(String),
}

static CURRENT_TEMP: std::sync::OnceLock<tokio::sync::RwLock<u32>> = std::sync::OnceLock::new();

fn current_temp() -> &'static tokio::sync::RwLock<u32> {
    CURRENT_TEMP.get_or_init(|| tokio::sync::RwLock::new(6500))
}

pub async fn get_status() -> Result<NightlightStatus, NightlightError> {
    let temp = *current_temp().read().await;
    let available = wayland_available() || redshift_available();

    Ok(NightlightStatus {
        enabled: temp != 6500,
        temperature_k: temp,
        available,
    })
}

pub async fn set_temperature(kelvin: u32) -> Result<(), NightlightError> {
    let temp = kelvin.clamp(1000, 10000);

    if let Err(e) = try_wayland_native(temp).await {
        tracing::debug!("wayland-native failed: {}, trying redshift", e);
        try_redshift(temp)?;
    }

    *current_temp().write().await = temp;
    Ok(())
}

pub async fn enable() -> Result<(), NightlightError> {
    let temp = Config::default().temperature_k;
    set_temperature(temp).await
}

pub async fn disable() -> Result<(), NightlightError> {
    set_temperature(6500).await
}

fn wayland_available() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}

fn redshift_available() -> bool {
    which::which("redshift").is_ok()
}

async fn try_wayland_native(_temp: u32) -> Result<(), NightlightError> {
    #[cfg(feature = "wayland")]
    {
        // TODO: Implement wayland-native via kanshi or compositor-specific protocols
        // For KDE: org.kde.KWin.TempFilter
        // For GNOME: org.gnome.SettingsDaemon.Color.Temperature
        // For sway: i3 IPC or swaymsg
    }

    #[cfg(not(feature = "wayland"))]
    {
        let _ = _temp;
        Err(NightlightError::NotAvailable)
    }
}

fn try_redshift(temp: u32) -> Result<(), NightlightError> {
    use std::process::Command;

    // Check if redshift is running
    let running = Command::new("pgrep")
        .arg("-x")
        .arg("redshift")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if running {
        // Send D-Bus signal to adjust temperature
        let output = Command::new("redshift")
            .args(["-O", &temp.to_string()])
            .output()
            .map_err(|e| NightlightError::Backend(e.to_string()))?;

        if !output.status.success() {
            return Err(NightlightError::AdjustFailed("redshift -O failed".into()));
        }
    } else {
        // Start redshift with one-shot mode
        let output = Command::new("redshift")
            .args(["-O", &temp.to_string(), "-P"])
            .output()
            .map_err(|e| NightlightError::Backend(e.to_string()))?;

        if !output.status.success() {
            return Err(NightlightError::AdjustFailed("redshift -O -P failed".into()));
        }
    }

    Ok(())
}
