//! Template application module.
//!
//! Handles writing rendered templates to files and reloading applications.

use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, warn};

pub struct TemplateApplicator {
    pub dry_run: bool,
}

impl TemplateApplicator {
    pub fn new() -> Self {
        Self { dry_run: false }
    }

    pub fn with_dry_run(dry_run: bool) -> Self {
        Self { dry_run }
    }

    pub fn apply(&self, content: &str, path: &PathBuf) -> Result<(), String> {
        if self.dry_run {
            info!(
                "[DRY RUN] Would write {} bytes to {:?}",
                content.len(),
                path
            );
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        std::fs::write(path, content).map_err(|e| format!("Failed to write template: {}", e))?;

        info!("Applied template to {:?}", path);
        Ok(())
    }

    pub fn reload(&self, app: &str) -> Result<(), String> {
        if self.dry_run {
            info!("[DRY RUN] Would reload {}", app);
            return Ok(());
        }

        let result = match app {
            "foot" => self.reload_foot(),
            "kitty" => self.reload_kitty(),
            "gtk" | "gtk3" | "gtk4" => self.reload_gtk(),
            "hyprland" => self.reload_hyprland(),
            "sway" => self.reload_sway(),
            "wofi" => self.reload_wofi(),
            _ => {
                warn!("No reload handler for {}", app);
                Ok(())
            }
        };

        if result.is_ok() {
            info!("Reloaded {}", app);
        }
        result
    }

    fn reload_foot(&self) -> Result<(), String> {
        Command::new("foot")
            .args(["--app-id", "crawlds", "-e", "ls"])
            .spawn()
            .map_err(|e| format!("Failed to signal foot: {}", e))?;
        Ok(())
    }

    fn reload_kitty(&self) -> Result<(), String> {
        Command::new("kitty")
            .args(["@", "set-spacing", "0"])
            .spawn()
            .map_err(|e| format!("Failed to signal kitty: {}", e))?;
        Ok(())
    }

    fn reload_gtk(&self) -> Result<(), String> {
        if let Ok(output) = Command::new("gsettings")
            .args(["set", "org.gnome.desktop.interface", "gtk-theme", "CrawlDS"])
            .output()
        {
            if !output.status.success() {
                error!("GTK settings: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Ok(())
    }

    fn reload_hyprland(&self) -> Result<(), String> {
        Command::new("hyprctl")
            .args(["reload"])
            .output()
            .map_err(|e| format!("Failed to reload Hyprland: {}", e))?;
        Ok(())
    }

    fn reload_sway(&self) -> Result<(), String> {
        Command::new("swaymsg")
            .args(["reload"])
            .output()
            .map_err(|e| format!("Failed to reload Sway: {}", e))?;
        Ok(())
    }

    fn reload_wofi(&self) -> Result<(), String> {
        Command::new("pkill")
            .args(["-HUP", "wofi"])
            .output()
            .map_err(|e| format!("Failed to signal wofi: {}", e))?;
        Ok(())
    }
}

impl Default for TemplateApplicator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn apply_template(
    template_content: &str,
    output_path: &PathBuf,
    reload_app: Option<&str>,
) -> Result<(), String> {
    let applicator = TemplateApplicator::new();
    applicator.apply(template_content, output_path)?;

    if let Some(app) = reload_app {
        applicator.reload(app)?;
    }

    Ok(())
}
