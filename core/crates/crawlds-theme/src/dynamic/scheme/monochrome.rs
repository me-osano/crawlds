//! Monochrome scheme implementation
//!
//! Pure grayscale theme with vibrant error color.

use crate::dynamic::hct::TonalPalette;
use crate::dynamic::scheme::tones::ToneValues;
use crate::dynamic::scheme::tones::MONOCHROME_DARK_TONES;
use crate::dynamic::scheme::tones::MONOCHROME_LIGHT_TONES;
use crate::dynamic::scheme::SchemeColors;

pub struct SchemeMonochrome {
    primary_palette: TonalPalette,
    error_palette: TonalPalette,
}

impl SchemeMonochrome {
    pub fn new(source_hue: f32) -> Self {
        Self {
            primary_palette: TonalPalette::new(source_hue, 0.0),
            error_palette: TonalPalette::new(25.0, 84.0),
        }
    }

    fn generate(&self, tones: &ToneValues) -> SchemeColors {
        SchemeColors {
            primary: self.primary_palette.tone_hex(tones.primary as f32),
            on_primary: self.primary_palette.tone_hex(tones.on_primary as f32),
            primary_container: self
                .primary_palette
                .tone_hex(tones.primary_container as f32),
            on_primary_container: self
                .primary_palette
                .tone_hex(tones.on_primary_container as f32),
            secondary: self.primary_palette.tone_hex(tones.secondary as f32),
            on_secondary: self.primary_palette.tone_hex(tones.on_secondary as f32),
            secondary_container: self
                .primary_palette
                .tone_hex(tones.secondary_container as f32),
            on_secondary_container: self
                .primary_palette
                .tone_hex(tones.on_secondary_container as f32),
            tertiary: self.primary_palette.tone_hex(tones.tertiary as f32),
            on_tertiary: self.primary_palette.tone_hex(tones.on_tertiary as f32),
            tertiary_container: self
                .primary_palette
                .tone_hex(tones.tertiary_container as f32),
            on_tertiary_container: self
                .primary_palette
                .tone_hex(tones.on_tertiary_container as f32),
            error: self.error_palette.tone_hex(tones.error as f32),
            on_error: self.error_palette.tone_hex(tones.on_error as f32),
            error_container: self.error_palette.tone_hex(tones.error_container as f32),
            on_error_container: self.error_palette.tone_hex(tones.on_error_container as f32),
            surface: self.primary_palette.tone_hex(tones.surface as f32),
            on_surface: self.primary_palette.tone_hex(tones.on_surface as f32),
            surface_variant: self.primary_palette.tone_hex(tones.surface_variant as f32),
            on_surface_variant: self
                .primary_palette
                .tone_hex(tones.on_surface_variant as f32),
            surface_container_lowest: self
                .primary_palette
                .tone_hex(tones.surface_container_lowest as f32),
            surface_container_low: self
                .primary_palette
                .tone_hex(tones.surface_container_low as f32),
            surface_container: self
                .primary_palette
                .tone_hex(tones.surface_container as f32),
            surface_container_high: self
                .primary_palette
                .tone_hex(tones.surface_container_high as f32),
            surface_container_highest: self
                .primary_palette
                .tone_hex(tones.surface_container_highest as f32),
            outline: self.primary_palette.tone_hex(tones.outline as f32),
            outline_variant: self.primary_palette.tone_hex(tones.outline_variant as f32),
            shadow: self.primary_palette.tone_hex(tones.shadow as f32),
            scrim: self.primary_palette.tone_hex(tones.scrim as f32),
            inverse_surface: self.primary_palette.tone_hex(tones.inverse_surface as f32),
            inverse_on_surface: self
                .primary_palette
                .tone_hex(tones.inverse_on_surface as f32),
            inverse_primary: self.primary_palette.tone_hex(tones.inverse_primary as f32),
            surface_tint: self.primary_palette.tone_hex(tones.primary as f32),
            background: self.primary_palette.tone_hex(tones.background as f32),
            on_background: self.primary_palette.tone_hex(tones.on_background as f32),
            surface_dim: self.primary_palette.tone_hex(tones.surface_dim as f32),
            surface_bright: self.primary_palette.tone_hex(tones.surface_bright as f32),
        }
    }

    pub fn get_dark(&self) -> SchemeColors {
        self.generate(&MONOCHROME_DARK_TONES)
    }

    pub fn get_light(&self) -> SchemeColors {
        self.generate(&MONOCHROME_LIGHT_TONES)
    }
}
