//! crawlds-display: Display control (brightness, nightlight, etc.)
//!
//! This crate handles display-related functionality:
//! - Brightness control via sysfs backlight
//! - Nightlight/blue light filter via wayland-native or redshift

pub mod brightness;
pub mod nightlight;

pub use brightness::{Backlight, BrightnessError, Config as BrightnessConfig};
pub use nightlight::{NightlightError, Config as NightlightConfig};

use tokio::sync::broadcast;

pub use brightness::run as run_brightness;
pub use brightness::Config as Config;

/// Run the display subsystem (brightness polling).
/// For now, only brightness is reactive; nightlight is query-only.
pub async fn run(cfg: Config, tx: broadcast::Sender<crawlds_ipc::CrawlEvent>) -> anyhow::Result<()> {
    brightness::run(cfg, tx).await
}
