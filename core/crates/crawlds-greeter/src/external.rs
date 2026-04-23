//! External authentication detection (fprintd, U2F)
//!
//! This module provides D-Bus based detection of external authentication
//! methods like fingerprint (fprintd) and U2F tokens.

use crate::types::ExternalAuthStatus;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExternalAuthError {
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),

    #[error("probe failed: {0}")]
    ProbeFailed(String),

    #[error("not available")]
    NotAvailable,
}

pub struct ExternalAuthDetector;

impl ExternalAuthDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn probe_fprintd(&self) -> Result<bool, ExternalAuthError> {
        // Check if gdbus is available
        let gdbus_check = Command::new("sh")
            .args(["-c", "command -v gdbus >/dev/null 2>&1 || echo PROBE_UNAVAILABLE"])
            .output();

        if let Ok(output) = gdbus_check {
            if String::from_utf8_lossy(&output.stdout).contains("PROBE_UNAVAILABLE") {
                return Ok(false);
            }
        }

        // Try to get fprintd devices via D-Bus
        let output = Command::new("sh")
            .args([
                "-c",
                "gdbus call --system \
                 --dest net.reactivated.Fprint \
                 --object-path /net/reactivated/Fprint/Manager \
                 --method net.reactivated.Fprint.Manager.GetDevices 2>/dev/null \
                 || echo PROBE_UNAVAILABLE",
            ])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains("PROBE_UNAVAILABLE") {
                    Ok(false)
                } else {
                    // Check if any devices are present
                    Ok(stdout.contains("objectpath") || stdout.contains("Device"))
                }
            }
            Err(_) => Ok(false),
        }
    }

    pub fn check_u2f_available() -> bool {
        // Check common U2F PAM module locations
        let u2f_paths = [
            "/etc/pam.d/greetd",
            "/etc/pam.d/system-auth",
            "/etc/pam.d/common-auth",
        ];

        for path in &u2f_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if content.contains("pam_u2f.so") {
                    return true;
                }
            }
        }

        false
    }

    pub async fn detect_external_auth(&self) -> ExternalAuthStatus {
        let has_u2f = Self::check_u2f_available();

        // Try to probe fprintd
        let fprintd_available = match self.probe_fprintd() {
            Ok(available) => available,
            Err(e) => {
                tracing::debug!("fprintd probe failed: {}", e);
                false
            }
        };

        ExternalAuthStatus::with_detection(
            fprintd_available,
            has_u2f,
            true, // probe_complete - we tried
            fprintd_available,
        )
    }

    pub fn detect_external_auth_sync() -> ExternalAuthStatus {
        let detector = Self::new();
        let has_u2f = Self::check_u2f_available();

        // Probe fprintd synchronously
        let fprintd_available = detector.probe_fprintd().unwrap_or(false);

        ExternalAuthStatus::with_detection(
            fprintd_available,
            has_u2f,
            true,
            fprintd_available,
        )
    }
}

impl Default for ExternalAuthDetector {
    fn default() -> Self {
        Self::new()
    }
}
