use crate::error::{ThemeError, ThemeResult};
use std::process::Command;
use tracing::{debug, info, warn};

pub struct Matugen {
    binary_path: Option<String>,
}

impl Matugen {
    pub fn new() -> Self {
        let binary_path = Self::find_matugen();
        Self { binary_path }
    }

    fn find_matugen() -> Option<String> {
        let candidates = [
            "matugen",
            "/usr/bin/matugen",
            "/usr/local/bin/matugen",
            ".cargo/bin/matugen",
            ".local/bin/matugen",
        ];

        for candidate in candidates {
            if Command::new(candidate)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                debug!("Found matugen at: {}", candidate);
                return Some(candidate.to_string());
            }
        }

        warn!("matugen not found in PATH");
        None
    }

    pub fn is_available(&self) -> bool {
        self.binary_path.is_some()
    }

    pub fn generate_from_image(
        &self,
        image_path: &str,
        mode: &str,
        color_index: usize,
    ) -> ThemeResult<String> {
        let binary = self
            .binary_path
            .as_ref()
            .ok_or(ThemeError::MatugenNotFound)?;

        info!(
            "Generating theme from image: {} (mode: {})",
            image_path, mode
        );

        let output = Command::new(binary)
            .args([
                "image",
                image_path,
                "--json",
                "hex",
                "--mode",
                mode,
                "--source-color-index",
                &color_index.to_string(),
            ])
            .output()?;

        if !output.status.success() {
            let exit_code = output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ThemeError::Matugen(format!(
                "matugen failed with exit code {}: {}",
                exit_code,
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Matugen output: {}", stdout);
        Ok(stdout.to_string())
    }

    pub fn generate_from_color(&self, color: &str, mode: &str) -> ThemeResult<String> {
        let binary = self
            .binary_path
            .as_ref()
            .ok_or(ThemeError::MatugenNotFound)?;

        info!("Generating theme from color: {} (mode: {})", color, mode);

        let output = Command::new(binary)
            .args(["color", color, "--json", "hex", "--mode", mode])
            .output()?;

        if !output.status.success() {
            let exit_code = output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ThemeError::Matugen(format!(
                "matugen failed with exit code {}: {}",
                exit_code,
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}

impl Default for Matugen {
    fn default() -> Self {
        Self::new()
    }
}
