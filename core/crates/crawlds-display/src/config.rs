//! Display configuration types.
//!
//! Contains config structs for brightness, nightlight, and wallpaper.

use serde::{Deserialize, Serialize};

/// Brightness configuration.
pub use super::brightness::Config as BrightnessConfig;

/// Nightlight configuration.
pub use super::nightlight::Config as NightlightConfig;

/// Wallpaper configuration.
pub use super::wallpaper::Config as WallpaperConfig;

pub mod all {
    use super::*;

    /// Unified display config for backward compatibility with existing code.
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct Config {
        #[serde(default)]
        pub brightness: BrightnessConfig,
        #[serde(default)]
        pub nightlight: NightlightConfig,
    }
}
