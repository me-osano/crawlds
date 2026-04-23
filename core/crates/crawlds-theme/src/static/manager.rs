use crate::error::{ThemeError, ThemeResult};
use crate::r#static::loader::load_theme_from_file;
use crate::types::ThemeData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCache {
    pub name: String,
    pub variant: String,
    pub theme: ThemeData,
}

#[derive(Debug, Clone)]
pub struct ThemeManager {
    themes_dir: PathBuf,
    cache_dir: PathBuf,
    themes: HashMap<String, ThemeData>,
    current_theme: Option<ThemeData>,
    current_name: Option<String>,
    current_variant: String,
}

impl ThemeManager {
    pub fn new(themes_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            themes_dir,
            cache_dir,
            themes: HashMap::new(),
            current_theme: None,
            current_name: None,
            current_variant: "dark".to_string(),
        }
    }

    pub fn load_all(&mut self) -> ThemeResult<Vec<String>> {
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
                                self.themes.insert(name.clone(), theme);
                                theme_names.push(name);
                            }
                            Err(e) => {
                                error!("Failed to load theme {:?}: {}", path, e);
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

    pub fn set_theme(&mut self, name: &str) -> ThemeResult<Option<ThemeData>> {
        if let Some(theme) = self.themes.get(name) {
            self.current_theme = Some(theme.clone());
            self.current_name = Some(name.to_string());
            info!("Set current theme to: {}", name);
            self.save_to_cache().ok();
            Ok(Some(theme.clone()))
        } else {
            Err(ThemeError::NotFound(name.to_string()))
        }
    }

    pub fn set_theme_with_variant(
        &mut self,
        name: &str,
        variant: &str,
    ) -> ThemeResult<Option<ThemeData>> {
        if let Some(theme) = self.themes.get(name) {
            self.current_theme = Some(theme.clone());
            self.current_name = Some(name.to_string());
            self.current_variant = variant.to_string();
            info!("Set current theme to: {} (variant: {})", name, variant);
            self.save_to_cache().ok();
            Ok(Some(theme.clone()))
        } else {
            Err(ThemeError::NotFound(name.to_string()))
        }
    }

    pub fn get_current_variant(&self) -> &str {
        &self.current_variant
    }

    pub fn set_current_variant(&mut self, variant: &str) {
        self.current_variant = variant.to_string();
        self.save_to_cache().ok();
    }

    pub fn get_current(&self) -> Option<&ThemeData> {
        self.current_theme.as_ref()
    }

    pub fn get_current_name(&self) -> Option<&str> {
        self.current_name.as_deref()
    }

    pub fn get_theme(&self, name: &str) -> Option<&ThemeData> {
        self.themes.get(name)
    }

    pub fn list_themes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn save_theme(&self, theme: &ThemeData, name: &str) -> ThemeResult<PathBuf> {
        let path = self.themes_dir.join(format!("{}.toml", name));
        let content =
            toml::to_string_pretty(theme).map_err(|e| ThemeError::Invalid(e.to_string()))?;
        std::fs::write(&path, content)?;
        info!("Saved theme to {:?}", path);
        Ok(path)
    }

    pub fn save_to_cache(&self) -> ThemeResult<()> {
        let name = self
            .current_name
            .as_ref()
            .ok_or_else(|| ThemeError::Invalid("No current theme to cache".to_string()))?;
        let theme = self
            .current_theme
            .as_ref()
            .ok_or_else(|| ThemeError::Invalid("No current theme to cache".to_string()))?;

        let cache = ThemeCache {
            name: name.clone(),
            variant: self.current_variant.clone(),
            theme: theme.clone(),
        };

        let cache_path = self.cache_dir.join("current-theme.json");

        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let json =
            serde_json::to_string_pretty(&cache).map_err(|e| ThemeError::Invalid(e.to_string()))?;
        std::fs::write(&cache_path, json)?;
        info!("Saved theme cache to {:?}", cache_path);
        Ok(())
    }

    pub fn load_from_cache(&mut self) -> ThemeResult<Option<ThemeData>> {
        let cache_path = self.cache_dir.join("current-theme.json");

        if !cache_path.exists() {
            info!("No theme cache file found");
            return Ok(None);
        }

        let content = std::fs::read_to_string(&cache_path)?;
        let cache: ThemeCache =
            serde_json::from_str(&content).map_err(|e| ThemeError::Invalid(e.to_string()))?;

        // Try to load the theme from disk
        if let Some(theme) = self.themes.get(&cache.name) {
            self.current_theme = Some(theme.clone());
            self.current_name = Some(cache.name.clone());
            self.current_variant = cache.variant.clone();
            info!(
                "Loaded theme from cache: {} (variant: {})",
                cache.name, cache.variant
            );
            Ok(Some(theme.clone()))
        } else {
            warn!(
                "Theme from cache not found in loaded themes: {}",
                cache.name
            );
            Ok(None)
        }
    }
}
