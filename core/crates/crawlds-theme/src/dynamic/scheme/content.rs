//! Content scheme implementation
//!
//! Preserves source color's chroma for the primary palette.

use crate::dynamic::hct::{Hct, TonalPalette};
use crate::dynamic::scheme::tones::{DARK_TONES, LIGHT_TONES};
use crate::dynamic::scheme::SchemeColors;

pub struct SchemeContent {
    primary_palette: TonalPalette,
    secondary_palette: TonalPalette,
    tertiary_palette: TonalPalette,
    neutral_palette: TonalPalette,
    neutral_variant_palette: TonalPalette,
    error_palette: TonalPalette,
}

impl SchemeContent {
    pub fn new(source_hct: &Hct) -> Self {
        let hue = source_hct.get_hue();
        let chroma = source_hct.get_chroma();

        let secondary_chroma = (chroma - 32.0).max(chroma * 0.5);

        let tertiary_hue = (hue + 60.0) % 360.0;

        Self {
            primary_palette: TonalPalette::new(hue, chroma),
            secondary_palette: TonalPalette::new(hue, secondary_chroma.max(0.0)),
            tertiary_palette: TonalPalette::new(tertiary_hue, chroma * 0.5),
            neutral_palette: TonalPalette::new(hue, chroma / 8.0),
            neutral_variant_palette: TonalPalette::new(hue, chroma / 8.0 + 4.0),
            error_palette: TonalPalette::new(25.0, 84.0),
        }
    }

    fn generate(&self, tones: &super::tones::ToneValues) -> SchemeColors {
        SchemeColors {
            primary: self.primary_palette.tone_hex(tones.primary as f32),
            on_primary: self.primary_palette.tone_hex(tones.on_primary as f32),
            primary_container: self
                .primary_palette
                .tone_hex(tones.primary_container as f32),
            on_primary_container: self
                .primary_palette
                .tone_hex(tones.on_primary_container as f32),
            secondary: self.secondary_palette.tone_hex(tones.secondary as f32),
            on_secondary: self.secondary_palette.tone_hex(tones.on_secondary as f32),
            secondary_container: self
                .secondary_palette
                .tone_hex(tones.secondary_container as f32),
            on_secondary_container: self
                .secondary_palette
                .tone_hex(tones.on_secondary_container as f32),
            tertiary: self.tertiary_palette.tone_hex(tones.tertiary as f32),
            on_tertiary: self.tertiary_palette.tone_hex(tones.on_tertiary as f32),
            tertiary_container: self
                .tertiary_palette
                .tone_hex(tones.tertiary_container as f32),
            on_tertiary_container: self
                .tertiary_palette
                .tone_hex(tones.on_tertiary_container as f32),
            error: self.error_palette.tone_hex(tones.error as f32),
            on_error: self.error_palette.tone_hex(tones.on_error as f32),
            error_container: self.error_palette.tone_hex(tones.error_container as f32),
            on_error_container: self.error_palette.tone_hex(tones.on_error_container as f32),
            surface: self.neutral_palette.tone_hex(tones.surface as f32),
            on_surface: self.neutral_palette.tone_hex(tones.on_surface as f32),
            surface_variant: self
                .neutral_variant_palette
                .tone_hex(tones.surface_variant as f32),
            on_surface_variant: self
                .neutral_variant_palette
                .tone_hex(tones.on_surface_variant as f32),
            surface_container_lowest: self
                .neutral_palette
                .tone_hex(tones.surface_container_lowest as f32),
            surface_container_low: self
                .neutral_palette
                .tone_hex(tones.surface_container_low as f32),
            surface_container: self
                .neutral_palette
                .tone_hex(tones.surface_container as f32),
            surface_container_high: self
                .neutral_palette
                .tone_hex(tones.surface_container_high as f32),
            surface_container_highest: self
                .neutral_palette
                .tone_hex(tones.surface_container_highest as f32),
            outline: self.neutral_variant_palette.tone_hex(tones.outline as f32),
            outline_variant: self
                .neutral_variant_palette
                .tone_hex(tones.outline_variant as f32),
            shadow: self.neutral_palette.tone_hex(tones.shadow as f32),
            scrim: self.neutral_palette.tone_hex(tones.scrim as f32),
            inverse_surface: self.neutral_palette.tone_hex(tones.inverse_surface as f32),
            inverse_on_surface: self
                .neutral_palette
                .tone_hex(tones.inverse_on_surface as f32),
            inverse_primary: self.primary_palette.tone_hex(tones.inverse_primary as f32),
            surface_tint: self.primary_palette.tone_hex(tones.primary as f32),
            background: self.neutral_palette.tone_hex(tones.background as f32),
            on_background: self.neutral_palette.tone_hex(tones.on_background as f32),
            surface_dim: self.neutral_palette.tone_hex(tones.surface_dim as f32),
            surface_bright: self.neutral_palette.tone_hex(tones.surface_bright as f32),
        }
    }

    pub fn get_dark(&self) -> SchemeColors {
        self.generate(&DARK_TONES)
    }

    pub fn get_light(&self) -> SchemeColors {
        self.generate(&LIGHT_TONES)
    }
}
