//! crawlds-theme: Material You theme generation with HCT color space
//!
//! This crate provides complete Material Design 3 theme generation capabilities:
//! - HCT (Hue-Chroma-Tone) color space implementation
//! - Color quantization (Wu + WSMeans algorithms)
//! - M3 scheme generation (tonal-spot, fruit-salad, rainbow, etc.)
//! - Terminal color configuration generation
//! - Generic/static theme loading
//! - Template rendering for theme application

pub mod config;
pub mod dynamic;
pub mod generic;
pub mod manager;
pub mod template;

pub use generic::load_theme_from_file;
pub use manager::{TemplateSettings, ThemeManager, ThemeSettings};

pub use config::{TemplateConfig, ThemeConfig};
pub use dynamic::generator::{GeneratorConfig, ThemeGenerator};
pub use dynamic::hct::{Hct, TonalPalette};
pub use dynamic::quantizer::{extract_source_color, quantize_wsmeans, quantize_wu, score_colors};
pub use dynamic::scheme::{
    SchemeContent, SchemeFaithful, SchemeFruitSalad, SchemeMonochrome, SchemeMuted, SchemeRainbow,
    SchemeTonalSpot, SchemeVibrant,
};

pub use template::{
    apps, apply, load_template, render_template, render_template_dual_mode,
    DualModeTemplateRenderer, TemplateRenderer,
};

pub use crawlds_ipc::{
    TerminalColorSet, TerminalColors, ThemeColors, ThemeData, ThemeMetadata, ThemeMode,
};

pub use dynamic::scheme::SchemeColors as SchemeColorsInner;
pub use dynamic::terminal::TerminalTheme;

pub fn scheme_colors_to_theme_colors(scheme: &SchemeColorsInner) -> ThemeColors {
    ThemeColors {
        primary: scheme.primary.clone(),
        on_primary: scheme.on_primary.clone(),
        secondary: scheme.secondary.clone(),
        on_secondary: scheme.on_secondary.clone(),
        tertiary: scheme.tertiary.clone(),
        on_tertiary: scheme.on_tertiary.clone(),
        error: scheme.error.clone(),
        on_error: scheme.on_error.clone(),
        surface: scheme.surface.clone(),
        on_surface: scheme.on_surface.clone(),
        surface_variant: scheme.surface_variant.clone(),
        on_surface_variant: scheme.on_surface_variant.clone(),
        outline: scheme.outline.clone(),
        shadow: scheme.shadow.clone(),
        hover: scheme.surface_tint.clone(),
        on_hover: scheme.on_surface.clone(),
    }
}

pub fn theme_colors_to_scheme_colors(colors: &ThemeColors) -> SchemeColorsInner {
    SchemeColorsInner {
        primary: colors.primary.clone(),
        on_primary: colors.on_primary.clone(),
        primary_container: colors.primary.clone(),
        on_primary_container: colors.on_primary.clone(),
        secondary: colors.secondary.clone(),
        on_secondary: colors.on_secondary.clone(),
        secondary_container: colors.secondary.clone(),
        on_secondary_container: colors.on_secondary.clone(),
        tertiary: colors.tertiary.clone(),
        on_tertiary: colors.on_tertiary.clone(),
        tertiary_container: colors.tertiary.clone(),
        on_tertiary_container: colors.on_tertiary.clone(),
        error: colors.error.clone(),
        on_error: colors.on_error.clone(),
        error_container: colors.error.clone(),
        on_error_container: colors.on_error.clone(),
        surface: colors.surface.clone(),
        on_surface: colors.on_surface.clone(),
        surface_variant: colors.surface_variant.clone(),
        on_surface_variant: colors.on_surface_variant.clone(),
        surface_container_lowest: colors.surface.clone(),
        surface_container_low: colors.surface.clone(),
        surface_container: colors.surface.clone(),
        surface_container_high: colors.surface.clone(),
        surface_container_highest: colors.surface.clone(),
        outline: colors.outline.clone(),
        outline_variant: colors.surface_variant.clone(),
        shadow: colors.shadow.clone(),
        scrim: colors.shadow.clone(),
        inverse_surface: colors.on_surface.clone(),
        inverse_on_surface: colors.surface.clone(),
        inverse_primary: colors.primary.clone(),
        surface_tint: colors.hover.clone(),
        background: colors.surface.clone(),
        on_background: colors.on_surface.clone(),
        surface_dim: colors.surface.clone(),
        surface_bright: colors.surface.clone(),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeCache {
    pub name: String,
    pub mode: String,
    pub theme: ThemeData,
}

pub mod prelude {
    pub use crate::dynamic::generator::{GeneratorConfig, ThemeGenerator};
    pub use crate::dynamic::hct::{Hct, TonalPalette};
}
