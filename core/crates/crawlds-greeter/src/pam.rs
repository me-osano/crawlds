//! PAM stack detection and configuration parsing
//!
//! This module provides functionality to:
//! - Detect available PAM modules (fprintd, U2F)
//! - Parse lockout policies (faillock, tally)
//! - Generate user-friendly auth feedback

use crate::types::PamInfo;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PamError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),
}

pub struct PamStack {
    pub content: String,
    pub path: String,
}

impl PamStack {
    pub fn load(path: &str) -> Result<Option<Self>, PamError> {
        let path = Path::new(path);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        Ok(Some(Self {
            content,
            path: path.to_string_lossy().to_string(),
        }))
    }

    pub fn has_module(&self, module_name: &str) -> bool {
        self.lines()
            .filter(|l| !l.trim().starts_with('#'))
            .any(|l| l.contains(module_name))
    }

    pub fn uses_lockout(&self) -> bool {
        self.has_module("pam_faillock.so")
            || self.has_module("pam_tally2.so")
            || self.has_module("pam_tally.so")
    }

    pub fn parse_deny_value(&self) -> Option<i32> {
        for line in self.lines() {
            let line = line.trim();
            if !line.starts_with('#') {
                if let Some(deny) = Self::extract_deny(line) {
                    return Some(deny);
                }
            }
        }
        None
    }

    fn extract_deny(line: &str) -> Option<i32> {
        if !line.contains("pam_faillock.so")
            && !line.contains("pam_tally2.so")
            && !line.contains("pam_tally.so")
        {
            return None;
        }

        // Match deny=<number>
        if let Some(idx) = line.find("deny") {
            let rest = &line[idx..];
            if let Some(eq_idx) = rest.find('=') {
                let value_str = &rest[eq_idx + 1..];
                if let Some(space_idx) =
                    value_str.find(|c: char| c.is_whitespace() || c == '#' || c == '\0')
                {
                    let value = &value_str[..space_idx];
                    return value.parse().ok();
                } else if !value_str.is_empty() {
                    return value_str.parse().ok();
                }
            }
        }
        None
    }

    fn lines(&self) -> impl Iterator<Item = &str> {
        self.content.lines()
    }
}

pub struct PamDetector;

impl PamDetector {
    pub fn detect_pam_info() -> PamInfo {
        let mut info = PamInfo::default();

        // Check main PAM config files
        let config_files = [
            "/etc/pam.d/greetd",
            "/etc/pam.d/system-auth",
            "/etc/pam.d/common-auth",
            "/etc/pam.d/password-auth",
            "/etc/pam.d/system-login",
            "/etc/pam.d/system-local-login",
            "/etc/pam.d/common-auth-pc",
            "/etc/pam.d/login",
        ];

        let mut lockout_found = false;
        let mut min_deny = None;

        for path in &config_files {
            if let Ok(Some(stack)) = PamStack::load(path) {
                // Check for fprintd
                if stack.has_module("pam_fprintd.so") {
                    info.has_fprintd = true;
                }

                // Check for U2F
                if stack.has_module("pam_u2f.so") {
                    info.has_u2f = true;
                }

                // Check for lockout
                if stack.uses_lockout() {
                    lockout_found = true;
                }

                // Get deny value
                if let Some(deny) = stack.parse_deny_value() {
                    min_deny = Some(min_deny.map_or(deny, |m: i32| m.min(deny)));
                }
            }
        }

        // Check faillock config
        if let Ok(Some(deny)) = Self::parse_faillock_config() {
            min_deny = Some(min_deny.map_or(deny, |m: i32| m.min(deny)));
        }

        info.lockout_configured = lockout_found;
        info.faillock_deny = min_deny.unwrap_or(-1);

        // Default to 3 if lockout is configured but no explicit deny value
        if lockout_found && info.faillock_deny < 0 {
            info.faillock_deny = 3;
        }

        info
    }

    fn parse_faillock_config() -> Result<Option<i32>, std::io::Error> {
        let path = Path::new("/etc/security/faillock.conf");
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if let Some(deny) = Self::extract_faillock_deny(line) {
                return Ok(Some(deny));
            }
        }

        Ok(None)
    }

    fn extract_faillock_deny(line: &str) -> Option<i32> {
        if !line.to_lowercase().starts_with("deny") {
            return None;
        }

        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return None;
        }

        let value = parts[1].trim();
        value.parse().ok()
    }

    pub fn generate_auth_feedback(
        pam_state: &str,
        failure_count: i32,
        info: &PamInfo,
    ) -> crate::types::AuthFeedback {
        let mut message = String::new();
        let mut is_lockout = false;
        let mut attempts_remaining: Option<i32> = None;

        match pam_state {
            "error" => {
                message = "Authentication error - try again".to_string();
            }
            "max" => {
                message = "Too many failed attempts - account may be locked".to_string();
                is_lockout = true;
            }
            "fail" => {
                if info.lockout_configured && info.faillock_deny >= 0 {
                    let remaining = (info.faillock_deny - failure_count).max(0);
                    attempts_remaining = Some(remaining);

                    if remaining > 0 {
                        message = format!(
                            "Incorrect password - attempt {} of {} (lockout may follow)",
                            failure_count + 1,
                            info.faillock_deny
                        );
                    } else {
                        message = "Incorrect password - next failures may trigger account lockout"
                            .to_string();
                    }
                } else {
                    message = "Incorrect password".to_string();
                }
            }
            _ => {}
        }

        crate::types::AuthFeedback {
            message,
            pam_state: pam_state.to_string(),
            is_lockout,
            attempts_remaining,
        }
    }

    pub fn is_lockout_message(message: &str) -> bool {
        let lower = message.to_lowercase();
        lower.contains("account is locked")
            || lower.contains("too many")
            || lower.contains("maximum number of")
            || lower.contains("auth_err")
    }
}
