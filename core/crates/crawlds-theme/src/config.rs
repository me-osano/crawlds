use serde::{Deserialize, Serialize};

use crate::dynamic::generator::SchemeType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub source: String,
    pub name: String,
    pub scheme: String,
    pub mode: String,
    pub default_scheme: String,
    pub default_mode: String,
    pub cache_dir: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TemplateConfig {
    pub alacritty: bool,
    pub btop: bool,
    pub code: bool,
    pub crawlds: bool,
    pub emacs: bool,
    pub foot: bool,
    pub ghostty: bool,
    pub gtk: bool,
    pub helix: bool,
    pub kitty: bool,
    pub walker: bool,
    pub wezterm: bool,
    pub yazi: bool,
    pub zed_editor: bool,
    pub zen_browser: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            source: "generic".to_string(),
            name: "Nord".to_string(),
            scheme: "static".to_string(),
            mode: "dark".to_string(),
            default_scheme: "tonal-spot".to_string(),
            default_mode: "dark".to_string(),
            cache_dir: String::new(),
        }
    }
}

impl ThemeConfig {
    pub fn scheme_type(&self) -> SchemeType {
        match self.default_scheme.to_lowercase().as_str() {
            "rainbow" => SchemeType::Rainbow,
            "content" => SchemeType::Content,
            "monochrome" => SchemeType::Monochrome,
            _ => SchemeType::TonalSpot,
        }
    }
}
