//! Theme Generator Module
//!
//! High-level API for theme generation from images or colors.

use crate::dynamic::hct::Hct;
use crate::dynamic::quantizer::extract_source_color;
use crate::dynamic::scheme::SchemeColors;
use crate::dynamic::scheme::{
    Scheme, SchemeContent, SchemeFaithful, SchemeFruitSalad, SchemeMonochrome, SchemeMuted,
    SchemeRainbow, SchemeTonalSpot, SchemeVibrant,
};
use crate::dynamic::terminal::TerminalTheme;
use crate::dynamic::terminal::{alacritty, foot, ghostty, kitty, wezterm};
use crawlds_ipc::{
    TerminalColorSet, TerminalColors, ThemeColors, ThemeData, ThemeMetadata, ThemeMode,
    ThemeSchemeType,
};

#[derive(Clone, Debug)]
pub struct GeneratorConfig {
    pub scheme_type: SchemeType,
    pub color_index: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            scheme_type: SchemeType::TonalSpot,
            color_index: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SchemeType {
    TonalSpot,
    Rainbow,
    Content,
    Monochrome,
    FruitSalad,
    Vibrant,
    Faithful,
    Muted,
}

impl SchemeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchemeType::TonalSpot => "tonal-spot",
            SchemeType::Rainbow => "rainbow",
            SchemeType::Content => "content",
            SchemeType::Monochrome => "monochrome",
            SchemeType::FruitSalad => "fruit-salad",
            SchemeType::Vibrant => "vibrant",
            SchemeType::Faithful => "faithful",
            SchemeType::Muted => "muted",
        }
    }

    pub fn to_theme_scheme_type(&self) -> ThemeSchemeType {
        match self {
            SchemeType::TonalSpot => ThemeSchemeType::Tonalspot,
            SchemeType::Rainbow => ThemeSchemeType::Rainbow,
            SchemeType::Content => ThemeSchemeType::Excited,
            SchemeType::Monochrome => ThemeSchemeType::Dark,
            SchemeType::FruitSalad => ThemeSchemeType::Vibrant,
            SchemeType::Vibrant => ThemeSchemeType::Vibrant,
            SchemeType::Faithful => ThemeSchemeType::Vibrant,
            SchemeType::Muted => ThemeSchemeType::Dark,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThemeGenerator {
    config: GeneratorConfig,
}

impl ThemeGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self::new(GeneratorConfig::default())
    }

    pub fn generate_from_color(&self, hex_color: &str) -> GeneratedTheme {
        let hex = hex_color.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        let hct = Hct::from_rgb(r, g, b);
        self.generate_from_hct(&hct)
    }

    pub fn generate_from_rgb(&self, r: u8, g: u8, b: u8) -> GeneratedTheme {
        let hct = Hct::from_rgb(r, g, b);
        self.generate_from_hct(&hct)
    }

    pub fn generate_from_hct(&self, source: &Hct) -> GeneratedTheme {
        let hue = source.get_hue();
        let chroma = source.get_chroma();
        let tone = source.get_tone();

        let dark: SchemeColors;
        let light: SchemeColors;

        match self.config.scheme_type {
            SchemeType::TonalSpot => {
                let scheme = SchemeTonalSpot::new(hue);
                dark = scheme.get_dark();
                light = scheme.get_light();
            }
            SchemeType::Rainbow => {
                let scheme = SchemeRainbow::new(hue);
                dark = scheme.get_dark();
                light = scheme.get_light();
            }
            SchemeType::Content => {
                let scheme = SchemeContent::new(source);
                dark = scheme.get_dark();
                light = scheme.get_light();
            }
            SchemeType::Monochrome => {
                let scheme = SchemeMonochrome::new(hue);
                dark = scheme.get_dark();
                light = scheme.get_light();
            }
            SchemeType::FruitSalad => {
                let scheme = SchemeFruitSalad::new(hue);
                dark = scheme.get_dark();
                light = scheme.get_light();
            }
            SchemeType::Vibrant => {
                let scheme = SchemeVibrant::new(hue, chroma);
                dark = scheme.get_dark();
                light = scheme.get_light();
            }
            SchemeType::Faithful => {
                let scheme = SchemeFaithful::new(hue, chroma);
                dark = scheme.get_dark();
                light = scheme.get_light();
            }
            SchemeType::Muted => {
                let scheme = SchemeMuted::new(hue);
                dark = scheme.get_dark();
                light = scheme.get_light();
            }
        }

        GeneratedTheme {
            source_hue: hue,
            source_chroma: chroma,
            source_tone: tone,
            dark,
            light,
            scheme_type: self.config.scheme_type.clone(),
        }
    }

    pub fn generate_from_image_pixels(&self, pixels: &[(u8, u8, u8)]) -> GeneratedTheme {
        let argb = extract_source_color(pixels, 0xFF4285F4);
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        self.generate_from_rgb(r, g, b)
    }
}

pub struct GeneratedTheme {
    pub source_hue: f32,
    pub source_chroma: f32,
    pub source_tone: f32,
    pub dark: SchemeColors,
    pub light: SchemeColors,
    pub scheme_type: SchemeType,
}

impl GeneratedTheme {
    pub fn dark_terminal(&self) -> TerminalTheme {
        TerminalTheme::from_scheme(&self.dark)
    }

    pub fn light_terminal(&self) -> TerminalTheme {
        TerminalTheme::from_scheme(&self.light)
    }

    pub fn dark_terminal_kitty(&self) -> String {
        kitty::generate(&self.dark_terminal())
    }

    pub fn light_terminal_kitty(&self) -> String {
        kitty::generate(&self.light_terminal())
    }

    pub fn dark_terminal_foot(&self) -> String {
        foot::generate(&self.dark_terminal())
    }

    pub fn light_terminal_foot(&self) -> String {
        foot::generate(&self.light_terminal())
    }

    pub fn dark_terminal_alacritty(&self) -> String {
        alacritty::generate(&self.dark_terminal())
    }

    pub fn light_terminal_alacritty(&self) -> String {
        alacritty::generate(&self.light_terminal())
    }

    pub fn dark_terminal_wezterm(&self) -> String {
        wezterm::generate(&self.dark_terminal())
    }

    pub fn light_terminal_wezterm(&self) -> String {
        wezterm::generate(&self.light_terminal())
    }

    pub fn dark_terminal_ghostty(&self) -> String {
        ghostty::generate(&self.dark_terminal())
    }

    pub fn light_terminal_ghostty(&self) -> String {
        ghostty::generate(&self.light_terminal())
    }

    pub fn to_theme_data(&self, name: &str, _source_image: &str) -> ThemeData {
        let scheme_name = self.scheme_type.as_str();

        let metadata = ThemeMetadata {
            name: name.to_string(),
            source: "dynamic".to_string(),
            scheme: scheme_name.to_string(),
        };

        let dark_terminal = self.dark_terminal();
        let light_terminal = self.light_terminal();

        let dark_mode = ThemeMode {
            colors: scheme_colors_to_theme_colors(&self.dark),
            terminal: terminal_theme_to_terminal_colors(&dark_terminal),
        };

        let light_mode = ThemeMode {
            colors: scheme_colors_to_theme_colors(&self.light),
            terminal: terminal_theme_to_terminal_colors(&light_terminal),
        };

        ThemeData {
            metadata,
            dark: dark_mode,
            light: light_mode,
        }
    }
}

fn scheme_colors_to_theme_colors(scheme: &SchemeColors) -> ThemeColors {
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

fn terminal_theme_to_terminal_colors(theme: &TerminalTheme) -> TerminalColors {
    let c = &theme.colors;
    TerminalColors {
        normal: TerminalColorSet {
            black: c.normal.black.clone(),
            red: c.normal.red.clone(),
            green: c.normal.green.clone(),
            yellow: c.normal.yellow.clone(),
            blue: c.normal.blue.clone(),
            magenta: c.normal.magenta.clone(),
            cyan: c.normal.cyan.clone(),
            white: c.normal.white.clone(),
        },
        bright: TerminalColorSet {
            black: c.bright.black.clone(),
            red: c.bright.red.clone(),
            green: c.bright.green.clone(),
            yellow: c.bright.yellow.clone(),
            blue: c.bright.blue.clone(),
            magenta: c.bright.magenta.clone(),
            cyan: c.bright.cyan.clone(),
            white: c.bright.white.clone(),
        },
        foreground: c.foreground.clone(),
        background: c.background.clone(),
        selection_fg: c.selection_fg.clone(),
        selection_bg: c.selection_bg.clone(),
        cursor: c.cursor.clone(),
        cursor_text: c.cursor_text.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_from_color() {
        let generator = ThemeGenerator::with_default();
        let theme = generator.generate_from_color("#4285f4");

        assert!(!theme.dark.primary.is_empty());
        assert!(!theme.light.primary.is_empty());
    }

    #[test]
    fn test_terminal_output() {
        let generator = ThemeGenerator::with_default();
        let theme = generator.generate_from_color("#4285f4");

        let kitty = theme.dark_terminal_kitty();
        assert!(kitty.contains("color0"));
    }
}
