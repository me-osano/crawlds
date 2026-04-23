//! Greeter types shared between modules
//!
//! Re-exports types from crawlds_ipc for use in greeter functionality.

pub use crawlds_ipc::types::{GreeterMessageType, GreeterState, GreeterStatus};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct SessionMemory {
    #[serde(default)]
    pub last_session_id: Option<String>,
    #[serde(default)]
    pub last_successful_user: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PamInfo {
    pub has_fprintd: bool,
    pub has_u2f: bool,
    pub lockout_configured: bool,
    pub faillock_deny: i32,
    pub pam_config_valid: bool,
}

impl Default for PamInfo {
    fn default() -> Self {
        Self {
            has_fprintd: false,
            has_u2f: false,
            lockout_configured: false,
            faillock_deny: -1,
            pam_config_valid: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthFeedback {
    pub message: String,
    pub pam_state: String,
    pub is_lockout: bool,
    pub attempts_remaining: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExternalAuthStatus {
    pub available: bool,
    pub has_fprintd: bool,
    pub has_u2f: bool,
    pub fprintd_probe_complete: bool,
    pub fprintd_has_device: bool,
}

impl Default for ExternalAuthStatus {
    fn default() -> Self {
        Self {
            available: false,
            has_fprintd: false,
            has_u2f: false,
            fprintd_probe_complete: false,
            fprintd_has_device: false,
        }
    }
}

impl ExternalAuthStatus {
    pub fn with_detection(
        has_fprintd: bool,
        has_u2f: bool,
        fprintd_probe_complete: bool,
        fprintd_has_device: bool,
    ) -> Self {
        Self {
            has_fprintd,
            has_u2f,
            fprintd_probe_complete,
            fprintd_has_device,
            available: (has_fprintd && fprintd_probe_complete && fprintd_has_device) || has_u2f,
        }
    }
}
