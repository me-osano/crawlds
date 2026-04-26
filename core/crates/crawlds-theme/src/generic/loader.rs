use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crawlds_ipc::ThemeData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawThemeMetadata {
    pub name: String,
    #[serde(rename = "source", default)]
    pub source: String,
    #[serde(default)]
    pub scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawThemeColors {
    pub primary: String,
    pub on_primary: String,
    pub secondary: String,
    pub on_secondary: String,
    pub tertiary: String,
    pub on_tertiary: String,
    pub error: String,
    pub on_error: String,
    pub surface: String,
    pub on_surface: String,
    pub surface_variant: String,
    pub on_surface_variant: String,
    pub outline: String,
    pub shadow: String,
    pub hover: String,
    pub on_hover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTerminalColors {
    pub normal: RawTerminalColorSet,
    pub bright: RawTerminalColorSet,
    pub foreground: String,
    pub background: String,
    pub selection_fg: String,
    pub selection_bg: String,
    pub cursor_text: String,
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTerminalColorSet {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawThemeMode {
    pub colors: RawThemeColors,
    pub terminal: RawTerminalColors,
    #[serde(default)]
    pub scheme_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTheme {
    pub metadata: RawThemeMetadata,
    pub dark: RawThemeMode,
    pub light: RawThemeMode,
}

pub fn load_theme_from_file(path: &PathBuf) -> Result<ThemeData, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read theme file: {}", e))?;

    let raw: RawTheme =
        toml::from_str(&content).map_err(|e| format!("Failed to parse theme file: {}", e))?;

    let metadata = crate::ThemeMetadata {
        name: raw.metadata.name,
        source: "generic".to_string(),
        scheme: "static".to_string(),
    };

    let dark = crate::ThemeMode {
        colors: convert_colors(raw.dark.colors),
        terminal: convert_terminal(raw.dark.terminal),
    };

    let light = crate::ThemeMode {
        colors: convert_colors(raw.light.colors),
        terminal: convert_terminal(raw.light.terminal),
    };

    Ok(ThemeData {
        metadata,
        dark,
        light,
    })
}

fn convert_colors(raw: RawThemeColors) -> crate::ThemeColors {
    crate::ThemeColors {
        primary: raw.primary,
        on_primary: raw.on_primary,
        secondary: raw.secondary,
        on_secondary: raw.on_secondary,
        tertiary: raw.tertiary,
        on_tertiary: raw.on_tertiary,
        error: raw.error,
        on_error: raw.on_error,
        surface: raw.surface,
        on_surface: raw.on_surface,
        surface_variant: raw.surface_variant,
        on_surface_variant: raw.on_surface_variant,
        outline: raw.outline,
        shadow: raw.shadow,
        hover: raw.hover,
        on_hover: raw.on_hover,
    }
}

fn convert_terminal(raw: RawTerminalColors) -> crate::TerminalColors {
    crate::TerminalColors {
        normal: crate::TerminalColorSet {
            black: raw.normal.black,
            red: raw.normal.red,
            green: raw.normal.green,
            yellow: raw.normal.yellow,
            blue: raw.normal.blue,
            magenta: raw.normal.magenta,
            cyan: raw.normal.cyan,
            white: raw.normal.white,
        },
        bright: crate::TerminalColorSet {
            black: raw.bright.black,
            red: raw.bright.red,
            green: raw.bright.green,
            yellow: raw.bright.yellow,
            blue: raw.bright.blue,
            magenta: raw.bright.magenta,
            cyan: raw.bright.cyan,
            white: raw.bright.white,
        },
        foreground: raw.foreground,
        background: raw.background,
        selection_fg: raw.selection_fg,
        selection_bg: raw.selection_bg,
        cursor_text: raw.cursor_text,
        cursor: raw.cursor,
    }
}
