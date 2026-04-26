//! Tonal Spot scheme implementation
//!
//! The default Android 12-13 Material You scheme.

use std::collections::HashMap;

use crate::dynamic::hct::{Hct, TonalPalette};
use crate::dynamic::scheme::tones::{ToneValues, DARK_TONES, LIGHT_TONES};

#[derive(Clone, Debug)]
pub struct SchemeColors {
    pub primary: String,
    pub on_primary: String,
    pub primary_container: String,
    pub on_primary_container: String,
    pub secondary: String,
    pub on_secondary: String,
    pub secondary_container: String,
    pub on_secondary_container: String,
    pub tertiary: String,
    pub on_tertiary: String,
    pub tertiary_container: String,
    pub on_tertiary_container: String,
    pub error: String,
    pub on_error: String,
    pub error_container: String,
    pub on_error_container: String,
    pub surface: String,
    pub on_surface: String,
    pub surface_variant: String,
    pub on_surface_variant: String,
    pub surface_container_lowest: String,
    pub surface_container_low: String,
    pub surface_container: String,
    pub surface_container_high: String,
    pub surface_container_highest: String,
    pub outline: String,
    pub outline_variant: String,
    pub shadow: String,
    pub scrim: String,
    pub inverse_surface: String,
    pub inverse_on_surface: String,
    pub inverse_primary: String,
    pub surface_tint: String,
    pub background: String,
    pub on_background: String,
    pub surface_dim: String,
    pub surface_bright: String,
}

impl SchemeColors {
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        macro_rules! add {
            ($field:ident) => {
                map.insert(stringify!($field).to_string(), self.$field.clone());
            };
        }
        add!(primary);
        add!(on_primary);
        add!(primary_container);
        add!(on_primary_container);
        add!(secondary);
        add!(on_secondary);
        add!(secondary_container);
        add!(on_secondary_container);
        add!(tertiary);
        add!(on_tertiary);
        add!(tertiary_container);
        add!(on_tertiary_container);
        add!(error);
        add!(on_error);
        add!(error_container);
        add!(on_error_container);
        add!(surface);
        add!(on_surface);
        add!(surface_variant);
        add!(on_surface_variant);
        add!(surface_container_lowest);
        add!(surface_container_low);
        add!(surface_container);
        add!(surface_container_high);
        add!(surface_container_highest);
        add!(outline);
        add!(outline_variant);
        add!(shadow);
        add!(scrim);
        add!(inverse_surface);
        add!(inverse_on_surface);
        add!(inverse_primary);
        add!(surface_tint);
        add!(background);
        add!(on_background);
        add!(surface_dim);
        add!(surface_bright);
        map
    }
}

pub trait Scheme {
    fn get_dark(&self) -> SchemeColors;
    fn get_light(&self) -> SchemeColors;
}

pub struct SchemeTonalSpot {
    primary_palette: TonalPalette,
    secondary_palette: TonalPalette,
    tertiary_palette: TonalPalette,
    neutral_palette: TonalPalette,
    neutral_variant_palette: TonalPalette,
    error_palette: TonalPalette,
}

impl SchemeTonalSpot {
    pub fn new(source_hue: f32) -> Self {
        Self {
            primary_palette: TonalPalette::new(source_hue, 48.0),
            secondary_palette: TonalPalette::new(source_hue, 16.0),
            tertiary_palette: TonalPalette::new((source_hue + 60.0) % 360.0, 24.0),
            neutral_palette: TonalPalette::new(source_hue, 4.0),
            neutral_variant_palette: TonalPalette::new(source_hue, 8.0),
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
}

impl Scheme for SchemeTonalSpot {
    fn get_dark(&self) -> SchemeColors {
        self.generate(&DARK_TONES)
    }

    fn get_light(&self) -> SchemeColors {
        self.generate(&LIGHT_TONES)
    }
}

pub struct SchemeRainbow {
    tonal_spot: SchemeTonalSpot,
}

impl SchemeRainbow {
    pub fn new(source_hue: f32) -> Self {
        Self {
            tonal_spot: SchemeTonalSpot::new(source_hue),
        }
    }
}

impl Scheme for SchemeRainbow {
    fn get_dark(&self) -> SchemeColors {
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

    fn get_light(&self) -> SchemeColors {
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
}

impl Scheme for SchemeContent {
    fn get_dark(&self) -> SchemeColors {
        self.generate(&DARK_TONES)
    }

    fn get_light(&self) -> SchemeColors {
        self.generate(&LIGHT_TONES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::scheme::{Scheme, SchemeContent, SchemeTonalSpot};

    #[test]
    fn test_tonal_spot() {
        let scheme = SchemeTonalSpot::new(220.0);
        let dark = scheme.get_dark();
        let light = scheme.get_light();
        assert!(!dark.primary.is_empty());
        assert!(!light.primary.is_empty());
    }

    #[test]
    fn test_content() {
        let hct = Hct::from(220.0, 50.0, 50.0);
        let scheme = SchemeContent::new(&hct);
        let dark = scheme.get_dark();
        assert!(!dark.primary.is_empty());
    }
}
