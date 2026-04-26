//! Terminal color configuration generators
//!
//! Generates color configurations for: Foot, Kitty, Alacritty, Wezterm, Ghostty

pub mod alacritty;
pub mod foot;
pub mod ghostty;
pub mod kitty;
pub mod wezterm;

use serde::{Deserialize, Serialize};

use crate::dynamic::scheme::SchemeColors;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalColorSet {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalColors {
    pub normal: TerminalColorSet,
    pub bright: TerminalColorSet,
    pub foreground: String,
    pub background: String,
    pub selection_fg: String,
    pub selection_bg: String,
    pub cursor: String,
    pub cursor_text: String,
}

#[derive(Clone, Debug)]
pub struct TerminalTheme {
    pub colors: TerminalColors,
}

impl TerminalTheme {
    pub fn from_scheme(scheme: &SchemeColors) -> Self {
        let bg = &scheme.background;
        let fg = &scheme.on_background;
        let primary = &scheme.primary;
        let secondary = &scheme.secondary;
        let tertiary = &scheme.tertiary;
        let error = &scheme.error;
        let surface_variant = &scheme.surface_variant;
        let on_surface_variant = &scheme.on_surface_variant;

        Self {
            colors: TerminalColors {
                normal: TerminalColorSet {
                    black: surface_variant.clone(),
                    red: error.clone(),
                    green: primary.clone(),
                    yellow: secondary.clone(),
                    blue: tertiary.clone(),
                    magenta: primary.clone(),
                    cyan: secondary.clone(),
                    white: on_surface_variant.clone(),
                },
                bright: TerminalColorSet {
                    black: on_surface_variant.clone(),
                    red: error.clone(),
                    green: primary.clone(),
                    yellow: secondary.clone(),
                    blue: tertiary.clone(),
                    magenta: primary.clone(),
                    cyan: secondary.clone(),
                    white: fg.clone(),
                },
                foreground: fg.clone(),
                background: bg.clone(),
                selection_fg: on_surface_variant.clone(),
                selection_bg: surface_variant.clone(),
                cursor: fg.clone(),
                cursor_text: bg.clone(),
            },
        }
    }
}

pub struct TerminalConfig {
    pub terminal: String,
    pub output_path: String,
}

impl TerminalConfig {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                terminal: "kitty".to_string(),
                output_path: "~/.config/kitty/themes/crawlds.conf".to_string(),
            },
            Self {
                terminal: "foot".to_string(),
                output_path: "~/.config/foot/themes/crawlds".to_string(),
            },
            Self {
                terminal: "alacritty".to_string(),
                output_path: "~/.config/alacritty/themes/crawlds.toml".to_string(),
            },
            Self {
                terminal: "wezterm".to_string(),
                output_path: "~/.config/wezterm/colors/CrawlDS.lua".to_string(),
            },
            Self {
                terminal: "ghostty".to_string(),
                output_path: "~/.config/ghostty/themes/crawlds".to_string(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::scheme::{Scheme, SchemeTonalSpot};

    #[test]
    fn test_kitty_output() {
        let scheme = SchemeTonalSpot::new(220.0);
        let colors = scheme.get_light();
        let theme = TerminalTheme::from_scheme(&colors);
        let output = kitty::generate(&theme);
        assert!(output.contains("color0"));
        assert!(output.contains("background"));
    }

    #[test]
    fn test_foot_output() {
        let scheme = SchemeTonalSpot::new(220.0);
        let colors = scheme.get_light();
        let theme = TerminalTheme::from_scheme(&colors);
        let output = foot::generate(&theme);
        assert!(output.contains("[colors]"));
        assert!(output.contains("background"));
    }
}
