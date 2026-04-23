use crate::dynamic::matugen::Matugen;
use crate::error::{ThemeError, ThemeResult};
use crate::types::{
    TerminalColorSet, TerminalColors, ThemeColors, ThemeData, ThemeMetadata, ThemeSchemeType,
    ThemeVariant,
};
use serde::Deserialize;
use tracing::info;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
struct MatugenOutput {
    #[serde(rename = "colors")]
    color_schemes: Option<MatugenColorSchemes>,
    #[serde(default)]
    dark: Option<MatugenScheme>,
    #[serde(default)]
    light: Option<MatugenScheme>,
    #[serde(default)]
    amoled: Option<MatugenScheme>,
}

#[derive(Debug, Deserialize)]
struct MatugenColorSchemes {
    #[serde(default)]
    vibrant: Option<MatugenScheme>,
    #[serde(default)]
    tonalspot: Option<MatugenScheme>,
    #[serde(default)]
    excited: Option<MatugenScheme>,
    #[serde(default)]
    rainbow: Option<MatugenScheme>,
    #[serde(default)]
    dark: Option<MatugenScheme>,
    #[serde(default)]
    light: Option<MatugenScheme>,
    #[serde(default)]
    amoled: Option<MatugenScheme>,
}

#[derive(Debug, Deserialize)]
struct MatugenScheme {
    #[serde(rename = "background")]
    pub background: Option<String>,
    #[serde(rename = "onBackground")]
    pub on_background: Option<String>,
    #[serde(rename = "surface")]
    pub surface: Option<String>,
    #[serde(rename = "onSurface")]
    pub on_surface: Option<String>,
    #[serde(rename = "surfaceContainerHighest")]
    pub surface_container_highest: Option<String>,
    #[serde(rename = "onSurfaceVariant")]
    pub on_surface_variant: Option<String>,
    #[serde(rename = "outline")]
    pub outline: Option<String>,
    #[serde(rename = "outlineVariant")]
    pub outline_variant: Option<String>,
    #[serde(rename = "primary")]
    pub primary: Option<String>,
    #[serde(rename = "onPrimary")]
    pub on_primary: Option<String>,
    #[serde(rename = "primaryContainer")]
    pub primary_container: Option<String>,
    #[serde(rename = "onPrimaryContainer")]
    pub on_primary_container: Option<String>,
    #[serde(rename = "secondary")]
    pub secondary: Option<String>,
    #[serde(rename = "onSecondary")]
    pub on_secondary: Option<String>,
    #[serde(rename = "secondaryContainer")]
    pub secondary_container: Option<String>,
    #[serde(rename = "onSecondaryContainer")]
    pub on_secondary_container: Option<String>,
    #[serde(rename = "tertiary")]
    pub tertiary: Option<String>,
    #[serde(rename = "onTertiary")]
    pub on_tertiary: Option<String>,
    #[serde(rename = "tertiaryContainer")]
    pub tertiary_container: Option<String>,
    #[serde(rename = "onTertiaryContainer")]
    pub on_tertiary_container: Option<String>,
    #[serde(rename = "error")]
    pub error: Option<String>,
    #[serde(rename = "onError")]
    pub on_error: Option<String>,
    #[serde(rename = "errorContainer")]
    pub error_container: Option<String>,
    #[serde(rename = "onErrorContainer")]
    pub on_error_container: Option<String>,
    #[serde(rename = "inverseSurface")]
    pub inverse_surface: Option<String>,
    #[serde(rename = "inverseOnSurface")]
    pub inverse_on_surface: Option<String>,
    #[serde(rename = "inversePrimary")]
    pub inverse_primary: Option<String>,
    #[serde(rename = "surfaceTint")]
    pub surface_tint: Option<String>,
    #[serde(rename = "scrim")]
    pub scrim: Option<String>,
}

pub fn generate_terminal_colors(base: &str) -> ThemeVariant {
    let base_lower = base.trim_start_matches('#').to_lowercase();
    let r = u8::from_str_radix(&base_lower[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&base_lower[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&base_lower[4..6], 16).unwrap_or(0);

    fn shift(c: u8, amount: i16) -> String {
        let s = (c as i16 + amount).clamp(0, 255) as u8;
        format!("#{:02x}{:02x}{:02x}", s, s, s)
    }

    ThemeVariant {
        colors: ThemeColors {
            primary: base.to_string(),
            on_primary: shift(r, 128),
            secondary: shift(g, 128),
            on_secondary: shift(g, 128),
            tertiary: shift(b, 128),
            on_tertiary: shift(b, 128),
            error: "#ff0000".to_string(),
            on_error: "#ffffff".to_string(),
            surface: base.to_string(),
            on_surface: shift(r, 128),
            surface_variant: shift(r, 64),
            on_surface_variant: shift(r, 96),
            outline: shift(r, 48),
            shadow: "#000000".to_string(),
            hover: shift(r, 32),
            on_hover: shift(r, 128),
        },
        terminal: TerminalColors {
            normal: TerminalColorSet {
                black: "#000000".to_string(),
                red: format!(
                    "#{:02x}{:02x}{:02x}",
                    r.saturating_sub(60),
                    g.saturating_sub(60),
                    b.saturating_sub(60)
                ),
                green: format!(
                    "#{:02x}{:02x}{:02x}",
                    r,
                    g.saturating_sub(60),
                    b.saturating_sub(60)
                ),
                yellow: format!(
                    "#{:02x}{:02x}{:02x}",
                    r.saturating_sub(30),
                    g,
                    b.saturating_sub(60)
                ),
                blue: format!(
                    "#{:02x}{:02x}{:02x}",
                    r.saturating_sub(60),
                    g.saturating_sub(60),
                    b
                ),
                magenta: format!(
                    "#{:02x}{:02x}{:02x}",
                    r.saturating_sub(30),
                    g.saturating_sub(60),
                    b.saturating_sub(30)
                ),
                cyan: format!(
                    "#{:02x}{:02x}{:02x}",
                    r.saturating_sub(60),
                    g,
                    b.saturating_sub(30)
                ),
                white: "#ffffff".to_string(),
            },
            bright: TerminalColorSet {
                black: "#686868".to_string(),
                red: format!(
                    "#{:02x}{:02x}{:02x}",
                    r.saturating_add(30),
                    g.saturating_add(30),
                    b.saturating_add(30)
                ),
                green: format!(
                    "#{:02x}{:02x}{:02x}",
                    r,
                    g.saturating_add(30),
                    b.saturating_add(30)
                ),
                yellow: format!(
                    "#{:02x}{:02x}{:02x}",
                    r.saturating_add(30),
                    g.saturating_add(30),
                    b
                ),
                blue: format!(
                    "#{:02x}{:02x}{:02x}",
                    r,
                    g.saturating_add(30),
                    b.saturating_add(30)
                ),
                magenta: format!(
                    "#{:02x}{:02x}{:02x}",
                    r.saturating_add(30),
                    g,
                    b.saturating_add(30)
                ),
                cyan: format!(
                    "#{:02x}{:02x}{:02x}",
                    r,
                    g.saturating_add(30),
                    b.saturating_add(30)
                ),
                white: "#ffffff".to_string(),
            },
            foreground: shift(r, 128),
            background: base.to_string(),
            selection_fg: shift(r, 128),
            selection_bg: shift(r, 64),
            cursor_text: base.to_string(),
            cursor: shift(r, 128),
        },
        scheme_type: ThemeSchemeType::Tonalspot,
    }
}

fn derive_shadow(surface: &str) -> String {
    let hex = surface.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        let factor = if hex.len() >= 8 { 0.7 } else { 0.8 };
        let r = (r as f32 * factor) as u8;
        let g = (g as f32 * factor) as u8;
        let b = (b as f32 * factor) as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    } else {
        "#000000".to_string()
    }
}

fn convert_scheme(
    scheme: &MatugenScheme,
    is_dark: bool,
    scheme_type: ThemeSchemeType,
) -> ThemeVariant {
    // Map matugen fields to theme fields:
    // background -> surface (matugen's background is the main background)
    // surface -> surface_variant (matugen's surface is a lighter surface)
    let primary = scheme
        .primary
        .clone()
        .unwrap_or_else(|| "#6200ee".to_string());

    // background becomes surface
    let surface = scheme
        .background
        .clone()
        .or_else(|| scheme.surface.clone())
        .unwrap_or_else(|| {
            if is_dark {
                "#1c1b1f".to_string()
            } else {
                "#fffbfe".to_string()
            }
        });

    // surface becomes surface_variant
    let surface_variant = scheme
        .surface
        .clone()
        .or_else(|| scheme.surface_container_highest.clone())
        .unwrap_or_else(|| "#49454f".to_string());

    let shadow = derive_shadow(&surface);

    let generated_terminal = generate_terminal_colors(&primary);

    ThemeVariant {
        colors: ThemeColors {
            primary: primary.clone(),
            on_primary: scheme
                .on_primary
                .clone()
                .unwrap_or_else(|| "#ffffff".to_string()),
            secondary: scheme
                .secondary
                .clone()
                .unwrap_or_else(|| "#03dac6".to_string()),
            on_secondary: scheme
                .on_secondary
                .clone()
                .unwrap_or_else(|| "#000000".to_string()),
            tertiary: scheme
                .tertiary
                .clone()
                .unwrap_or_else(|| "#3700b3".to_string()),
            on_tertiary: scheme
                .on_tertiary
                .clone()
                .unwrap_or_else(|| "#ffffff".to_string()),
            error: scheme
                .error
                .clone()
                .unwrap_or_else(|| "#f2b8b5".to_string()),
            on_error: scheme
                .on_error
                .clone()
                .unwrap_or_else(|| "#601410".to_string()),
            surface,
            on_surface: scheme
                .on_surface
                .clone()
                .unwrap_or_else(|| "#1c1b1f".to_string()),
            surface_variant,
            on_surface_variant: scheme
                .on_surface_variant
                .clone()
                .unwrap_or_else(|| "#cac4d0".to_string()),
            outline: scheme
                .outline
                .clone()
                .unwrap_or_else(|| "#79747e".to_string()),
            shadow,
            hover: scheme
                .surface_tint
                .clone()
                .unwrap_or_else(|| primary.clone()),
            on_hover: scheme
                .on_surface_variant
                .clone()
                .unwrap_or_else(|| "#cac4d0".to_string()),
        },
        terminal: generated_terminal.terminal,
        scheme_type,
    }
}

fn get_scheme_for_type(
    color_schemes: &MatugenColorSchemes,
    scheme_type: ThemeSchemeType,
) -> Option<&MatugenScheme> {
    match scheme_type {
        ThemeSchemeType::Vibrant => color_schemes.vibrant.as_ref(),
        ThemeSchemeType::Tonalspot => color_schemes.tonalspot.as_ref(),
        ThemeSchemeType::Excited => color_schemes.excited.as_ref(),
        ThemeSchemeType::Rainbow => color_schemes.rainbow.as_ref(),
        ThemeSchemeType::Dark => color_schemes.dark.as_ref(),
        ThemeSchemeType::Light => color_schemes.light.as_ref(),
        ThemeSchemeType::Amoled => color_schemes.amoled.as_ref(),
    }
}

fn scheme_type_to_string(scheme_type: &ThemeSchemeType) -> String {
    match scheme_type {
        ThemeSchemeType::Vibrant => "vibrant".to_string(),
        ThemeSchemeType::Tonalspot => "tonal-spot".to_string(),
        ThemeSchemeType::Excited => "excited".to_string(),
        ThemeSchemeType::Rainbow => "rainbow".to_string(),
        ThemeSchemeType::Dark => "dark".to_string(),
        ThemeSchemeType::Light => "light".to_string(),
        ThemeSchemeType::Amoled => "amoled".to_string(),
    }
}

pub struct DynamicThemeGenerator {
    matugen: Matugen,
}

impl DynamicThemeGenerator {
    pub fn new() -> Self {
        Self {
            matugen: Matugen::new(),
        }
    }

    pub fn is_available(&self) -> bool {
        self.matugen.is_available()
    }

    pub fn generate_from_wallpaper(
        &self,
        wallpaper_path: &str,
        mode: &str,
        color_index: usize,
        theme_name: &str,
        scheme_type: ThemeSchemeType,
    ) -> ThemeResult<ThemeData> {
        info!(
            "Generating dynamic theme from wallpaper: {}",
            wallpaper_path
        );

        let json_output = self
            .matugen
            .generate_from_image(wallpaper_path, mode, color_index)?;
        let output: MatugenOutput = serde_json::from_str(&json_output)?;

        let variant = if mode == "dark" { "dark" } else { "light" };

        let scheme = output
            .color_schemes
            .as_ref()
            .and_then(|cs| get_scheme_for_type(cs, scheme_type))
            .or_else(|| match variant {
                "dark" => output.dark.as_ref(),
                "light" => output.light.as_ref(),
                _ => output.dark.as_ref(),
            })
            .or_else(|| {
                output
                    .color_schemes
                    .as_ref()
                    .and_then(|cs| cs.tonalspot.as_ref())
            })
            .ok_or_else(|| ThemeError::Invalid("No color scheme in matugen output".to_string()))?;

        let is_dark = mode == "dark";
        let theme_variant = convert_scheme(scheme, is_dark, scheme_type);

        let dark_scheme = output
            .color_schemes
            .as_ref()
            .and_then(|cs| cs.dark.as_ref())
            .or_else(|| output.dark.as_ref());
        let light_scheme = output
            .color_schemes
            .as_ref()
            .and_then(|cs| cs.light.as_ref())
            .or_else(|| output.light.as_ref());

        let dark = dark_scheme
            .map(|s| convert_scheme(s, true, ThemeSchemeType::Dark))
            .unwrap_or_else(|| theme_variant.clone());
        let light = light_scheme
            .map(|s| convert_scheme(s, false, ThemeSchemeType::Light))
            .unwrap_or_else(|| generate_terminal_colors(&theme_variant.colors.primary));

        let theme = ThemeData {
            metadata: ThemeMetadata {
                name: "".to_string(),
                author: "".to_string(),
                origin: "dynamic".to_string(),
                scheme: scheme_type_to_string(&scheme_type),
                is_dark,
            },
            dark,
            light,
            schema_version: SCHEMA_VERSION,
        };

        info!("Generated dynamic theme: {}", theme_name);
        Ok(theme)
    }

    pub fn generate_from_color(
        &self,
        color: &str,
        mode: &str,
        theme_name: &str,
        scheme_type: ThemeSchemeType,
    ) -> ThemeResult<ThemeData> {
        info!("Generating dynamic theme from color: {}", color);

        let json_output = self.matugen.generate_from_color(color, mode)?;
        let output: MatugenOutput = serde_json::from_str(&json_output)?;

        let variant = if mode == "dark" { "dark" } else { "light" };

        let scheme = output
            .color_schemes
            .as_ref()
            .and_then(|cs| get_scheme_for_type(cs, scheme_type))
            .or_else(|| match variant {
                "dark" => output.dark.as_ref(),
                "light" => output.light.as_ref(),
                _ => output.dark.as_ref(),
            })
            .or_else(|| {
                output
                    .color_schemes
                    .as_ref()
                    .and_then(|cs| cs.tonalspot.as_ref())
            })
            .ok_or_else(|| ThemeError::Invalid("No color scheme in matugen output".to_string()))?;

        let is_dark = mode == "dark";
        let theme_variant = convert_scheme(scheme, is_dark, scheme_type);

        let dark_scheme = output
            .color_schemes
            .as_ref()
            .and_then(|cs| cs.dark.as_ref())
            .or_else(|| output.dark.as_ref());
        let light_scheme = output
            .color_schemes
            .as_ref()
            .and_then(|cs| cs.light.as_ref())
            .or_else(|| output.light.as_ref());

        let dark = dark_scheme
            .map(|s| convert_scheme(s, true, ThemeSchemeType::Dark))
            .unwrap_or_else(|| theme_variant.clone());
        let light = light_scheme
            .map(|s| convert_scheme(s, false, ThemeSchemeType::Light))
            .unwrap_or_else(|| generate_terminal_colors(&theme_variant.colors.primary));

        let theme = ThemeData {
            metadata: ThemeMetadata {
                name: "".to_string(),
                author: "".to_string(),
                origin: "dynamic".to_string(),
                scheme: scheme_type_to_string(&scheme_type),
                is_dark,
            },
            dark,
            light,
            schema_version: SCHEMA_VERSION,
        };

        info!("Generated dynamic theme: {}", theme_name);
        Ok(theme)
    }
}

impl Default for DynamicThemeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
