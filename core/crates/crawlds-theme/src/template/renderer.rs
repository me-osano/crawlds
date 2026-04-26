//! Template Renderer
//!
//! Core rendering with filters.

use std::collections::HashMap;

pub use crate::dynamic::scheme::SchemeColors;

pub const TEMPLATE_DELIMITER: &str = "{{";
pub const TEMPLATE_END: &str = "}}";

#[derive(Debug, Clone)]
pub enum TemplateNode {
    Text(String),
    Variable(String),
    Filter(String, Vec<(String, Vec<String>)>),
    For {
        variable: String,
        iterable: String,
        body: Vec<TemplateNode>,
    },
    If {
        condition: String,
        negated: bool,
        then_body: Vec<TemplateNode>,
        else_body: Vec<TemplateNode>,
    },
}

#[derive(Clone)]
pub struct ColorFilters;

impl ColorFilters {
    pub fn apply(filter_name: &str, value: &str, args: &[&str]) -> String {
        match filter_name {
            "set_alpha" => Self::set_alpha(value, args),
            "grayscale" => Self::grayscale(value, args),
            "lighten" => Self::lighten(value, args),
            "darken" => Self::darken(value, args),
            "saturate" => Self::saturate(value, args),
            "desaturate" => Self::desaturate(value, args),
            "invert" => Self::invert(value, args),
            "blend" => Self::blend(value, args),
            _ => value.to_string(),
        }
    }

    pub fn set_alpha(value: &str, args: &[&str]) -> String {
        if args.is_empty() {
            return value.to_string();
        }
        if let Ok(alpha) = args[0].parse::<f32>() {
            let alpha = (alpha * 255.0) as u8;
            let hex = value.trim_start_matches('#');
            if hex.len() == 6 {
                return format!("#{:02x}{}", alpha, hex);
            }
        }
        value.to_string()
    }

    pub fn grayscale(value: &str, _args: &[&str]) -> String {
        let hex = value.trim_start_matches('#');
        if hex.len() != 6 {
            return value.to_string();
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;
        format!("#{:02x}{:02x}{:02x}", gray, gray, gray)
    }

    pub fn lighten(value: &str, _args: &[&str]) -> String {
        Self::adjust_lightness(value, 1.2)
    }

    pub fn darken(value: &str, _args: &[&str]) -> String {
        Self::adjust_lightness(value, 0.8)
    }

    pub fn saturate(value: &str, _args: &[&str]) -> String {
        Self::adjust_saturation(value, 1.2)
    }

    pub fn desaturate(value: &str, _args: &[&str]) -> String {
        Self::adjust_saturation(value, 0.8)
    }

    fn adjust_lightness(value: &str, factor: f32) -> String {
        let hex = value.trim_start_matches('#');
        if hex.len() != 6 {
            return value.to_string();
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        let (h, s, l) = rgb_to_hsl(r, g, b);
        let new_l = (l * factor).clamp(0.0, 1.0);
        let (nr, ng, nb) = hsl_to_rgb(h, s, new_l);
        format!(
            "#{:02x}{:02x}{:02x}",
            (nr * 255.0) as u8,
            (ng * 255.0) as u8,
            (nb * 255.0) as u8
        )
    }

    fn adjust_saturation(value: &str, factor: f32) -> String {
        let hex = value.trim_start_matches('#');
        if hex.len() != 6 {
            return value.to_string();
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        let (h, s, l) = rgb_to_hsl(r, g, b);
        let new_s = (s * factor).clamp(0.0, 1.0);
        let (nr, ng, nb) = hsl_to_rgb(h, new_s, l);
        format!(
            "#{:02x}{:02x}{:02x}",
            (nr * 255.0) as u8,
            (ng * 255.0) as u8,
            (nb * 255.0) as u8
        )
    }

    pub fn invert(value: &str, _args: &[&str]) -> String {
        let hex = value.trim_start_matches('#');
        if hex.len() != 6 {
            return value.to_string();
        }
        let r = 255 - u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = 255 - u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = 255 - u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn blend(value: &str, args: &[&str]) -> String {
        if args.len() < 2 {
            return value.to_string();
        }
        let hex1 = value.trim_start_matches('#');
        let hex2 = args[0].trim_start_matches('#');
        if hex1.len() != 6 || hex2.len() != 6 {
            return value.to_string();
        }
        let ratio = args
            .get(1)
            .and_then(|r| r.parse::<f32>().ok())
            .unwrap_or(0.5);
        let r1 = u8::from_str_radix(&hex1[0..2], 16).unwrap_or(0) as f32;
        let g1 = u8::from_str_radix(&hex1[2..4], 16).unwrap_or(0) as f32;
        let b1 = u8::from_str_radix(&hex1[4..6], 16).unwrap_or(0) as f32;
        let r2 = u8::from_str_radix(&hex2[0..2], 16).unwrap_or(0) as f32;
        let g2 = u8::from_str_radix(&hex2[2..4], 16).unwrap_or(0) as f32;
        let b2 = u8::from_str_radix(&hex2[4..6], 16).unwrap_or(0) as f32;
        let r = (r1 * (1.0 - ratio) + r2 * ratio) as u8;
        let g = (g1 * (1.0 - ratio) + g2 * ratio) as u8;
        let b = (b1 * (1.0 - ratio) + b2 * ratio) as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < f32::EPSILON {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h / 6.0, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < f32::EPSILON {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 0.5 {
            return q;
        }
        if t < 2.0 / 6.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    };
    let h = h * 6.0;
    (
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 2.0 / 3.0),
        hue_to_rgb(p, q, h - 4.0 / 3.0),
    )
}

pub struct TemplateRenderer {
    color_map: HashMap<String, String>,
}

impl TemplateRenderer {
    pub fn new(colors: &SchemeColors) -> Self {
        let color_map = colors.to_map();
        Self { color_map }
    }

    pub fn render(&self, template: &str) -> String {
        let mut result = template.to_string();

        for (key, value) in &self.color_map {
            let patterns = [
                format!("{{{}}}", key),
                format!("{{{}.hex}}", key),
                format!("{{{}.hex_stripped}}", key),
                format!("{{{}.default.hex}}", key),
                format!("{{{}.default.hex_stripped}}", key),
            ];
            for pattern in patterns {
                result = result.replace(&pattern, value);
            }
            if let Some(stripped) = value.strip_prefix('#') {
                let stripped_pattern = format!("{{{}.hex_stripped}}", key);
                result = result.replace(&stripped_pattern, stripped);
                let default_stripped = format!("{{{}.default.hex_stripped}}", key);
                result = result.replace(&default_stripped, stripped);
            }
        }

        result = self.render_filters(&result);
        result
    }

    fn render_filters(&self, template: &str) -> String {
        let filter_re =
            regex::Regex::new(r"\{\{\s*(\w+)\.(\w+)\.hex\s*\|(\w+)(?:\s*([^}]+))?\s*\}\}").unwrap();

        let mut replacements: Vec<(String, String)> = Vec::new();

        for cap in filter_re.captures_iter(template) {
            let var = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let _color = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let filter_name = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let args_str = cap.get(4).map(|m| m.as_str()).unwrap_or("");
            let args: Vec<&str> = args_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            let key = format!("{}.{}", var, "primary");
            if let Some(value) = self
                .color_map
                .get(&key)
                .or_else(|| self.color_map.get(&format!("{}.{}", var, var)))
            {
                let filtered = ColorFilters::apply(filter_name, value, &args);
                let pattern_str = cap
                    .get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !pattern_str.is_empty() {
                    replacements.push((pattern_str, filtered));
                }
            }
        }

        let mut result = template.to_string();
        for (from, to) in replacements {
            result = result.replace(&from, &to);
        }
        result
    }

    pub fn render_with_keys(&self, template: &str, keys: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, value) in keys {
            let pattern = format!("{{{}}}", key);
            result = result.replace(&pattern, value);
        }
        result
    }
}

pub struct DualModeTemplateRenderer {
    dark_color_map: HashMap<String, String>,
    light_color_map: HashMap<String, String>,
}

impl DualModeTemplateRenderer {
    pub fn from_colors(dark: &SchemeColors, light: &SchemeColors) -> Self {
        Self {
            dark_color_map: dark.to_map(),
            light_color_map: light.to_map(),
        }
    }

    pub fn render(&self, template: &str) -> String {
        let mut result = template.to_string();

        let maps = [
            ("dark", &self.dark_color_map),
            ("light", &self.light_color_map),
        ];
        for (mode, color_map) in maps {
            for (key, value) in color_map {
                let patterns = [
                    format!("{{{}.{}.hex}}", key, mode),
                    format!("{{{}.{}.hex_stripped}}", key, mode),
                    format!("{{{}}}", key),
                ];
                for pattern in patterns {
                    result = result.replace(&pattern, value);
                }
                let default_pattern = format!("{{{}.default.hex}}", key);
                result = result.replace(&default_pattern, value);
                if let Some(stripped) = value.strip_prefix('#') {
                    let stripped_pattern = format!("{{{}.{}.hex_stripped}}", key, mode);
                    result = result.replace(&stripped_pattern, stripped);
                }
            }
        }

        result = self.render_filters(&result);
        result
    }

    fn render_filters(&self, template: &str) -> String {
        let filter_re =
            regex::Regex::new(r"\{\{\s*(\w+)\.(\w+)\.hex\s*\|(\w+)(?:\s*([^}]+))?\s*\}\}").unwrap();

        let mut replacements: Vec<(String, String)> = Vec::new();

        for cap in filter_re.captures_iter(template) {
            let var = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let mode = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let filter_name = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let args_str = cap.get(4).map(|m| m.as_str()).unwrap_or("");
            let args: Vec<&str> = args_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            let key = format!("{}.{}", var, "primary");
            let color_map = if mode == "dark" {
                &self.dark_color_map
            } else {
                &self.light_color_map
            };
            if let Some(value) = color_map
                .get(&key)
                .or_else(|| color_map.get(&format!("{}.{}", var, var)))
            {
                let filtered = ColorFilters::apply(filter_name, value, &args);
                let pattern_str = cap
                    .get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !pattern_str.is_empty() {
                    replacements.push((pattern_str, filtered));
                }
            }
        }

        let mut result = template.to_string();
        for (from, to) in replacements {
            result = result.replace(&from, &to);
        }
        result
    }

    pub fn load_and_render(&self, template_path: &std::path::Path) -> Result<String, String> {
        let content = std::fs::read_to_string(template_path)
            .map_err(|e| format!("Failed to read template: {}", e))?;
        Ok(self.render(&content))
    }
}

pub fn load_template(template_path: &std::path::Path) -> Result<String, String> {
    std::fs::read_to_string(template_path).map_err(|e| format!("Failed to read template: {}", e))
}

pub fn render_template(template: &str, colors: &SchemeColors) -> String {
    TemplateRenderer::new(colors).render(template)
}

pub fn render_template_dual_mode(
    template: &str,
    dark: &SchemeColors,
    light: &SchemeColors,
) -> String {
    DualModeTemplateRenderer::from_colors(dark, light).render(template)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::scheme::{Scheme, SchemeTonalSpot};

    #[test]
    fn test_render_simple() {
        let scheme = SchemeTonalSpot::new(220.0);
        let colors = scheme.get_light();
        let renderer = TemplateRenderer::new(&colors);
        let result = renderer.render("background = {background}");
        assert!(result.contains('#'));
    }

    #[test]
    fn test_filter_grayscale() {
        let result = ColorFilters::grayscale("#ff0000", &[]);
        assert_eq!(result, "#555555");
    }

    #[test]
    fn test_filter_invert() {
        let result = ColorFilters::invert("#000000", &[]);
        assert_eq!(result, "#ffffff");
    }

    #[test]
    fn test_filter_set_alpha() {
        let result = ColorFilters::set_alpha("#ff0000", &["0.5"]);
        let hex = result.trim_start_matches('#');
        assert_eq!(
            hex.len(),
            8,
            "Expected 8-char hex with alpha prefix, got: {}",
            result
        );
    }
}