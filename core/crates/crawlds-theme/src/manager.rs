use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::dynamic::generator::{GeneratorConfig, SchemeType, ThemeGenerator};
use crate::generic::loader::load_theme_from_file;
use crawlds_ipc::ThemeData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub source: String,
    pub name: String,
    pub scheme: String,
    pub mode: String,
    pub default_scheme: String,
    pub default_mode: String,
    pub cache_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateSettings {
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

#[derive(Debug, Clone)]
pub struct ThemeManager {
    pub settings: ThemeSettings,
    pub templates: TemplateSettings,
    pub themes_dir: PathBuf,
    pub cache_dir: PathBuf,
    generic_themes: HashMap<String, ThemeData>,
    current_theme: Option<ThemeData>,
    dynamic_generator: ThemeGenerator,
}

impl Default for ThemeSettings {
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

impl ThemeManager {
    pub fn new(themes_dir: PathBuf, cache_dir: PathBuf) -> Self {
        let default_scheme = SchemeType::TonalSpot;
        let generator = ThemeGenerator::new(GeneratorConfig {
            scheme_type: default_scheme,
            color_index: 0,
        });

        Self {
            settings: ThemeSettings::default(),
            templates: TemplateSettings::default(),
            themes_dir,
            cache_dir,
            generic_themes: HashMap::new(),
            current_theme: None,
            dynamic_generator: generator,
        }
    }

    pub fn with_settings(
        themes_dir: PathBuf,
        cache_dir: PathBuf,
        settings: ThemeSettings,
        templates: TemplateSettings,
    ) -> Self {
        let scheme = match settings.default_scheme.as_str() {
            "rainbow" => SchemeType::Rainbow,
            "content" => SchemeType::Content,
            "monochrome" => SchemeType::Monochrome,
            _ => SchemeType::TonalSpot,
        };
        let generator = ThemeGenerator::new(GeneratorConfig {
            scheme_type: scheme,
            color_index: 0,
        });

        Self {
            settings,
            templates,
            themes_dir,
            cache_dir,
            generic_themes: HashMap::new(),
            current_theme: None,
            dynamic_generator: generator,
        }
    }

    pub fn load_all(&mut self) -> Result<Vec<String>, String> {
        info!("Loading all themes from {:?}", self.themes_dir);
        let mut theme_names = Vec::new();

        if !self.themes_dir.exists() {
            return Ok(theme_names);
        }

        for entry in walkdir::WalkDir::new(&self.themes_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "toml" {
                        match load_theme_from_file(&path.to_path_buf()) {
                            Ok(theme) => {
                                let name = theme.metadata.name.clone();
                                debug!("Loaded theme: {}", name);
                                self.generic_themes.insert(name.clone(), theme);
                                theme_names.push(name);
                            }
                            Err(e) => {
                                warn!("Failed to load theme {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        theme_names.sort();
        info!("Loaded {} themes", theme_names.len());
        Ok(theme_names)
    }

    pub fn set_theme(&mut self, name: &str) -> Result<ThemeData, String> {
        if let Some(theme) = self.generic_themes.get(name) {
            self.current_theme = Some(theme.clone());
            self.settings.name = name.to_string();
            self.settings.source = "generic".to_string();
            self.settings.scheme = "static".to_string();
            info!("Set theme to: {} (generic)", name);
            Ok(theme.clone())
        } else {
            Err(format!("Theme not found: {}", name))
        }
    }

    pub fn set_dynamic(&mut self, hex_color: &str, name: &str) -> ThemeData {
        let theme = self.dynamic_generator.generate_from_color(hex_color);
        let theme_data = theme.to_theme_data(name, "dynamic");

        self.current_theme = Some(theme_data.clone());
        self.settings.name = name.to_string();
        self.settings.source = "dynamic".to_string();
        self.settings.scheme = self.settings.default_scheme.clone();
        info!(
            "Generated dynamic theme: {} ({})",
            name, self.settings.default_scheme
        );

        theme_data
    }

    pub fn set_mode(&mut self, mode: &str) {
        self.settings.mode = mode.to_string();
    }

    pub fn get_current(&self) -> Option<&ThemeData> {
        self.current_theme.as_ref()
    }

    pub fn get_current_name(&self) -> Option<&str> {
        if self.settings.name.is_empty() {
            None
        } else {
            Some(&self.settings.name)
        }
    }

    pub fn get_theme(&self, name: &str) -> Option<&ThemeData> {
        self.generic_themes.get(name)
    }

    pub fn get_settings(&self) -> &ThemeSettings {
        &self.settings
    }

    pub fn get_templates(&self) -> &TemplateSettings {
        &self.templates
    }

    pub fn update_settings(&mut self, settings: ThemeSettings) {
        self.settings = settings;

        self.dynamic_generator = ThemeGenerator::new(GeneratorConfig {
            scheme_type: match self.settings.default_scheme.as_str() {
                "rainbow" => SchemeType::Rainbow,
                "content" => SchemeType::Content,
                "monochrome" => SchemeType::Monochrome,
                _ => SchemeType::TonalSpot,
            },
            color_index: 0,
        });
    }

    pub fn update_templates(&mut self, templates: TemplateSettings) {
        self.templates = templates;
    }

    pub fn list_themes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.generic_themes.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn save_theme_json(&self, config_dir: &PathBuf) -> Result<PathBuf, String> {
        let theme = self
            .current_theme
            .as_ref()
            .ok_or_else(|| "No current theme to save".to_string())?;

        let json_path = config_dir.join("theme.json");

        if let Some(parent) = json_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let json = serde_json::to_string_pretty(&theme)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        std::fs::write(&json_path, json).map_err(|e| format!("Failed to write: {}", e))?;
        info!("Saved theme.json to {:?}", json_path);
        Ok(json_path)
    }

    pub fn load_theme_json(config_dir: &PathBuf) -> Result<ThemeData, String> {
        let json_path = config_dir.join("theme.json");

        if !json_path.exists() {
            return Err("theme.json not found".to_string());
        }

        let content =
            std::fs::read_to_string(&json_path).map_err(|e| format!("Failed to read: {}", e))?;
        let theme: ThemeData =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse: {}", e))?;

        info!("Loaded theme.json from {:?}", json_path);
        Ok(theme)
    }
}
