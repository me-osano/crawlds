//! crawlds-display: Display control (brightness, nightlight, wallpaper, etc.)
//!
//! This crate handles display-related functionality:
//! - Brightness control via sysfs backlight
//! - Nightlight/blue light filter via wayland-native or redshift
//! - Wallpaper management via swww backend

pub mod brightness;
pub mod config;
pub mod nightlight;
pub mod wallpaper;

pub use brightness::{Backlight, BrightnessError};
pub use config::all::Config as DisplayConfig;
pub use config::{BrightnessConfig, NightlightConfig, WallpaperConfig};
pub use nightlight::NightlightError;
pub use wallpaper::{WallpaperService, WallpaperMode};

use tokio::sync::broadcast;

pub use brightness::run as run_brightness;

/// Run the display subsystem (brightness polling).
/// For now, only brightness is reactive; nightlight is query-only.
pub async fn run(cfg: DisplayConfig, tx: broadcast::Sender<crawlds_ipc::CrawlEvent>) -> anyhow::Result<()> {
    brightness::run(cfg.brightness, tx).await
}
