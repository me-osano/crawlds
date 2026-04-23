use crate::error::{ThemeError, ThemeResult};
use crate::types::{
    TerminalColorSet, TerminalColors, ThemeColors, ThemeData, ThemeMetadata, ThemeSchemeType,
    ThemeVariant,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawThemeMetadata {
    pub name: String,
    pub author: String,
    #[serde(default, rename = "isDark")]
    pub is_dark: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawThemeColors {
    #[serde(rename = "primary")]
    pub primary: String,
    #[serde(rename = "onPrimary", default)]
    pub on_primary: String,
    #[serde(rename = "secondary")]
    pub secondary: String,
    #[serde(rename = "onSecondary", default)]
    pub on_secondary: String,
    #[serde(rename = "tertiary")]
    pub tertiary: String,
    #[serde(rename = "onTertiary", default)]
    pub on_tertiary: String,
    #[serde(rename = "error")]
    pub error: String,
    #[serde(rename = "onError", default)]
    pub on_error: String,
    #[serde(rename = "surface")]
    pub surface: String,
    #[serde(rename = "onSurface", default)]
    pub on_surface: String,
    #[serde(rename = "surfaceVariant", default)]
    pub surface_variant: String,
    #[serde(rename = "onSurfaceVariant", default)]
    pub on_surface_variant: String,
    #[serde(rename = "outline", default)]
    pub outline: String,
    #[serde(rename = "shadow", default)]
    pub shadow: String,
    #[serde(rename = "hover", default)]
    pub hover: String,
    #[serde(rename = "onHover", default)]
    pub on_hover: String,
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
pub struct RawTerminalColors {
    #[serde(rename = "normal")]
    pub normal: RawTerminalColorSet,
    #[serde(rename = "bright")]
    pub bright: RawTerminalColorSet,
    #[serde(rename = "foreground")]
    pub foreground: String,
    #[serde(rename = "background")]
    pub background: String,
    #[serde(rename = "selectionFg", default)]
    pub selection_fg: String,
    #[serde(rename = "selectionBg", default)]
    pub selection_bg: String,
    #[serde(rename = "cursorText", default)]
    pub cursor_text: String,
    #[serde(rename = "cursor", default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawThemeVariant {
    #[serde(flatten)]
    pub colors: RawThemeColors,
    #[serde(rename = "terminal")]
    pub terminal: RawTerminalColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTheme {
    #[serde(rename = "metadata")]
    pub metadata: RawThemeMetadata,
    #[serde(rename = "dark")]
    pub dark: RawThemeVariant,
    #[serde(rename = "light")]
    pub light: RawThemeVariant,
}

fn is_valid_hex(color: &str) -> bool {
    color.starts_with('#') && (color.len() == 7 || color.len() == 9)
}

fn validate_color(color: &str, field: &str) -> ThemeResult<()> {
    if !is_valid_hex(color) {
        return Err(ThemeError::InvalidColor(format!("{}: {}", field, color)));
    }
    Ok(())
}

fn validate_theme_colors(colors: &RawThemeColors, variant: &str) -> ThemeResult<()> {
    validate_color(&colors.primary, &format!("{}.primary", variant))?;
    validate_color(&colors.on_primary, &format!("{}.onPrimary", variant))?;
    validate_color(&colors.secondary, &format!("{}.secondary", variant))?;
    validate_color(&colors.on_secondary, &format!("{}.onSecondary", variant))?;
    validate_color(&colors.tertiary, &format!("{}.tertiary", variant))?;
    validate_color(&colors.on_tertiary, &format!("{}.onTertiary", variant))?;
    validate_color(&colors.error, &format!("{}.error", variant))?;
    validate_color(&colors.on_error, &format!("{}.onError", variant))?;
    validate_color(&colors.surface, &format!("{}.surface", variant))?;
    validate_color(&colors.on_surface, &format!("{}.onSurface", variant))?;
    validate_color(
        &colors.surface_variant,
        &format!("{}.surfaceVariant", variant),
    )?;
    validate_color(
        &colors.on_surface_variant,
        &format!("{}.onSurfaceVariant", variant),
    )?;
    validate_color(&colors.outline, &format!("{}.outline", variant))?;
    validate_color(&colors.shadow, &format!("{}.shadow", variant))?;
    validate_color(&colors.hover, &format!("{}.hover", variant))?;
    validate_color(&colors.on_hover, &format!("{}.onHover", variant))?;
    Ok(())
}

fn validate_terminal_colors(terminal: &RawTerminalColors, variant: &str) -> ThemeResult<()> {
    let sets = [("normal", &terminal.normal), ("bright", &terminal.bright)];
    for (name, set) in sets {
        validate_color(&set.black, &format!("{}.terminal.{}.black", variant, name))?;
        validate_color(&set.red, &format!("{}.terminal.{}.red", variant, name))?;
        validate_color(&set.green, &format!("{}.terminal.{}.green", variant, name))?;
        validate_color(
            &set.yellow,
            &format!("{}.terminal.{}.yellow", variant, name),
        )?;
        validate_color(&set.blue, &format!("{}.terminal.{}.blue", variant, name))?;
        validate_color(
            &set.magenta,
            &format!("{}.terminal.{}.magenta", variant, name),
        )?;
        validate_color(&set.cyan, &format!("{}.terminal.{}.cyan", variant, name))?;
        validate_color(&set.white, &format!("{}.terminal.{}.white", variant, name))?;
    }
    validate_color(
        &terminal.foreground,
        &format!("{}.terminal.foreground", variant),
    )?;
    validate_color(
        &terminal.background,
        &format!("{}.terminal.background", variant),
    )?;
    validate_color(
        &terminal.selection_fg,
        &format!("{}.terminal.selectionFg", variant),
    )?;
    validate_color(
        &terminal.selection_bg,
        &format!("{}.terminal.selectionBg", variant),
    )?;
    validate_color(
        &terminal.cursor_text,
        &format!("{}.terminal.cursorText", variant),
    )?;
    validate_color(&terminal.cursor, &format!("{}.terminal.cursor", variant))?;
    Ok(())
}

fn convert_colors(raw: RawThemeColors) -> ThemeColors {
    ThemeColors {
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

fn convert_terminal_color_set(raw: RawTerminalColorSet) -> TerminalColorSet {
    TerminalColorSet {
        black: raw.black,
        red: raw.red,
        green: raw.green,
        yellow: raw.yellow,
        blue: raw.blue,
        magenta: raw.magenta,
        cyan: raw.cyan,
        white: raw.white,
    }
}

fn convert_terminal(raw: RawTerminalColors) -> TerminalColors {
    TerminalColors {
        normal: convert_terminal_color_set(raw.normal),
        bright: convert_terminal_color_set(raw.bright),
        foreground: raw.foreground,
        background: raw.background,
        selection_fg: raw.selection_fg,
        selection_bg: raw.selection_bg,
        cursor_text: raw.cursor_text,
        cursor: raw.cursor,
    }
}

fn convert_variant(raw: RawThemeVariant, scheme_type: ThemeSchemeType) -> ThemeVariant {
    ThemeVariant {
        colors: convert_colors(raw.colors),
        terminal: convert_terminal(raw.terminal),
        scheme_type,
    }
}

pub fn load_theme_from_file(path: &PathBuf) -> ThemeResult<ThemeData> {
    info!("Loading theme from {:?}", path);
    let content = std::fs::read_to_string(path)?;
    let raw: RawTheme = toml::from_str(&content)?;

    validate_theme_colors(&raw.dark.colors, "dark")?;
    validate_theme_colors(&raw.light.colors, "light")?;
    validate_terminal_colors(&raw.dark.terminal, "dark")?;
    validate_terminal_colors(&raw.light.terminal, "light")?;

    let metadata = ThemeMetadata {
        name: raw.metadata.name,
        author: raw.metadata.author,
        origin: "static".to_string(),
        scheme: "".to_string(),
        is_dark: raw.metadata.is_dark,
    };

    let theme = ThemeData {
        metadata,
        dark: convert_variant(raw.dark, ThemeSchemeType::Dark),
        light: convert_variant(raw.light, ThemeSchemeType::Light),
        schema_version: 1,
    };

    debug!("Theme loaded successfully: {}", theme.metadata.name);
    Ok(theme)
}
