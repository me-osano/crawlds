//! Rainbow scheme implementation
//!
//! Chromatic accents with grayscale neutrals.

use crate::dynamic::scheme::Scheme;
use crate::dynamic::scheme::SchemeColors;
use crate::dynamic::scheme::SchemeTonalSpot;

pub struct SchemeRainbow {
    tonal_spot: SchemeTonalSpot,
}

impl SchemeRainbow {
    pub fn new(source_hue: f32) -> Self {
        Self {
            tonal_spot: SchemeTonalSpot::new(source_hue),
        }
    }

    pub fn get_dark(&self) -> SchemeColors {
        let mut colors = self.tonal_spot.get_dark();
        colors.surface = "#000000".to_string();
        colors.background = "#000000".to_string();
        colors.surface_container_lowest = "#000000".to_string();
        colors.surface_container_low = "#000000".to_string();
        colors.surface_container = "#000000".to_string();
        colors.surface_container_high = "#000000".to_string();
        colors.surface_container_highest = "#000000".to_string();
        colors
    }

    pub fn get_light(&self) -> SchemeColors {
        let mut colors = self.tonal_spot.get_light();
        colors.surface = "#ffffff".to_string();
        colors.background = "#ffffff".to_string();
        colors.surface_container_lowest = "#ffffff".to_string();
        colors.surface_container_low = "#ffffff".to_string();
        colors.surface_container = "#ffffff".to_string();
        colors.surface_container_high = "#ffffff".to_string();
        colors.surface_container_highest = "#ffffff".to_string();
        colors
    }
}
