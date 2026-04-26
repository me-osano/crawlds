//! Faithful scheme implementation
//!
//! Prioritizes dominant colors by area coverage (what you see is what you get).

use crate::dynamic::hct::TonalPalette;
use crate::dynamic::scheme::tones::{ToneValues, DARK_TONES, LIGHT_TONES};
use crate::dynamic::scheme::{Scheme, SchemeColors};

pub struct SchemeFaithful {
    primary_palette: TonalPalette,
    secondary_palette: TonalPalette,
    tertiary_palette: TonalPalette,
    neutral_palette: TonalPalette,
    error_palette: TonalPalette,
}

impl SchemeFaithful {
    pub fn new(source_hue: f32, source_chroma: f32) -> Self {
        let chroma = if source_chroma > 48.0 {
            source_chroma
        } else {
            48.0
        };
        let sec_chroma = if source_chroma > 24.0 {
            source_chroma
        } else {
            24.0
        };

        Self {
            primary_palette: TonalPalette::new(source_hue, chroma),
            secondary_palette: TonalPalette::new((source_hue + 30.0).rem_euclid(360.0), sec_chroma),
            tertiary_palette: TonalPalette::new((source_hue + 60.0).rem_euclid(360.0), sec_chroma),
            neutral_palette: TonalPalette::new(source_hue, 8.0),
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
            surface_variant: self.neutral_palette.tone_hex(tones.surface_variant as f32),
            on_surface_variant: self
                .neutral_palette
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
            outline: self.neutral_palette.tone_hex(tones.outline as f32),
            outline_variant: self.neutral_palette.tone_hex(tones.outline_variant as f32),
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
}

impl Scheme for SchemeFaithful {
    fn get_dark(&self) -> SchemeColors {
        self.generate(&DARK_TONES)
    }

    fn get_light(&self) -> SchemeColors {
        self.generate(&LIGHT_TONES)
    }
}
