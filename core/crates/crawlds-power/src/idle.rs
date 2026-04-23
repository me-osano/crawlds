//! Idle detection via D-Bus (org.freedesktop.ScreenSaver or logind).
//!
//! Polls session idle time and emits events for idle/active transitions.
//! Executes configured commands at idle timeout thresholds.

use crawlds_ipc::events::{CrawlEvent, IdleEvent};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tracing::info;
use zbus::{proxy, Connection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub idle_timeout_secs: u64,
    pub sleep_timeout_secs: u64,
    pub dim_timeout_secs: u64,
    pub screen_off_timeout_secs: u64,
    pub lock_timeout_secs: u64,
    pub suspend_timeout_secs: u64,
    pub fade_duration_secs: u64,
    pub screen_off_command: Option<String>,
    pub lock_command: Option<String>,
    pub suspend_command: Option<String>,
    pub resume_screen_off_command: Option<String>,
    pub resume_lock_command: Option<String>,
    pub resume_suspend_command: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            idle_timeout_secs: 300,
            sleep_timeout_secs: 600,
            dim_timeout_secs: 60,
            screen_off_timeout_secs: 600,
            lock_timeout_secs: 660,
            suspend_timeout_secs: 1800,
            fade_duration_secs: 5,
            screen_off_command: None,
            lock_command: None,
            suspend_command: None,
            resume_screen_off_command: None,
            resume_lock_command: None,
            resume_suspend_command: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IdleState {
    Active,
    Idle,
    Dimming,
    Sleeping,
    ScreenOff,
    Locked,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleStatus {
    pub idle_time_secs: u64,
    pub state: IdleState,
    pub inhibited: bool,
    pub pending_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InhibitEntry {
    pub id: u32,
    pub reason: String,
}

#[derive(Debug, Error)]
pub enum IdleError {
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),
    #[error("no idle backend available")]
    NoBackend,
    #[error("inhibit not found: {0}")]
    InhibitNotFound(u32),
    #[error("command execution failed: {0}")]
    CommandError(String),
}

#[proxy(
    interface = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/org/freedesktop/ScreenSaver"
)]
trait ScreenSaver {
    async fn get_session_idle_time(&self) -> zbus::Result<u32>;
    async fn inhibit(&self, app_id: &str, reason: &str) -> zbus::Result<u32>;
    async fn uninhibit(&self, inhibit_id: u32) -> zbus::Result<()>;
}

static INHIBITS: std::sync::OnceLock<tokio::sync::Mutex<Vec<InhibitEntry>>> =
    std::sync::OnceLock::new();

fn inhibits() -> &'static tokio::sync::Mutex<Vec<InhibitEntry>> {
    INHIBITS.get_or_init(|| tokio::sync::Mutex::new(Vec::new()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IdleAction {
    ScreenOff,
    Lock,
    Suspend,
}

struct IdleTaskState {
    cfg: Config,
    prev_state: IdleState,
    action_timers: std::collections::HashMap<IdleAction, u64>,
    executed_actions: std::collections::HashSet<IdleAction>,
}

pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) -> anyhow::Result<()> {
    info!("idle detection starting");

    let conn = Connection::session().await?;

    let ss = match ScreenSaverProxy::new(&conn).await {
        Ok(ss) => ss,
        Err(e) => {
            tracing::warn!("ScreenSaver not available: {}", e);
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        }
    };

    let state = Arc::new(Mutex::new(IdleTaskState {
        cfg: cfg.clone(),
        prev_state: IdleState::Active,
        action_timers: std::collections::HashMap::new(),
        executed_actions: std::collections::HashSet::new(),
    }));

    loop {
        let idle_time = ss.get_session_idle_time().await.unwrap_or(0) as u64;

        let is_inhibited = inhibits().lock().await.len() > 0;

        let (current_state, pending_action) = {
            let mut s = state.lock().await;
            s.cfg = cfg.clone();
            let timers = s.action_timers.clone();
            let executed = s.executed_actions.clone();
            let result = calculate_state(idle_time, &s.cfg, timers, executed);
            s.action_timers = result.2;
            s.executed_actions = result.3;
            (result.0, result.1)
        };

        let event_name = match current_state {
            IdleState::Active => {
                let mut s = state.lock().await;
                s.executed_actions.clear();
                s.action_timers.clear();
                "resumed"
            }
            IdleState::Idle => "idle_detected",
            IdleState::Dimming => "dimming",
            IdleState::Sleeping => "sleeping",
            IdleState::ScreenOff => "screen_off",
            IdleState::Locked => "locked",
            IdleState::Suspended => "suspended",
        };

        if !is_inhibited || current_state == IdleState::Active {
            let _ = tx.send(CrawlEvent::Idle(IdleEvent {
                event: event_name.to_string(),
                idle_time_secs: idle_time,
                pending_action: pending_action.clone(),
            }));
        }

        if let Some(action) = pending_action {
            let _ = tx.send(CrawlEvent::Idle(IdleEvent {
                event: "action_pending".to_string(),
                idle_time_secs: idle_time,
                pending_action: Some(action),
            }));
        }

        {
            let mut s = state.lock().await;
            s.prev_state = current_state;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn calculate_state(
    idle_time: u64,
    cfg: &Config,
    action_timers: std::collections::HashMap<IdleAction, u64>,
    executed_actions: std::collections::HashSet<IdleAction>,
) -> (IdleState, Option<String>, std::collections::HashMap<IdleAction, u64>, std::collections::HashSet<IdleAction>) {
    let base_state = if cfg.sleep_timeout_secs > 0 && idle_time >= cfg.sleep_timeout_secs {
        IdleState::Sleeping
    } else if cfg.idle_timeout_secs > 0 && idle_time >= cfg.idle_timeout_secs {
        IdleState::Idle
    } else if cfg.dim_timeout_secs > 0 && idle_time >= cfg.dim_timeout_secs {
        IdleState::Dimming
    } else {
        IdleState::Active
    };

    if executed_actions.contains(&IdleAction::Suspend) {
        return (IdleState::Suspended, None, action_timers, executed_actions);
    }
    if executed_actions.contains(&IdleAction::Lock) {
        return (IdleState::Locked, None, action_timers, executed_actions);
    }
    if executed_actions.contains(&IdleAction::ScreenOff) {
        return (IdleState::ScreenOff, None, action_timers, executed_actions);
    }

    if cfg.suspend_timeout_secs > 0 && idle_time >= cfg.suspend_timeout_secs {
        if cfg.suspend_command.is_some() {
            return (IdleState::Sleeping, Some("suspend".to_string()), action_timers, executed_actions);
        }
    }
    if cfg.lock_timeout_secs > 0 && idle_time >= cfg.lock_timeout_secs {
        if cfg.lock_command.is_some() {
            return (IdleState::Sleeping, Some("lock".to_string()), action_timers, executed_actions);
        }
    }
    if cfg.screen_off_timeout_secs > 0 && idle_time >= cfg.screen_off_timeout_secs {
        if cfg.screen_off_command.is_some() {
            return (IdleState::Sleeping, Some("screen_off".to_string()), action_timers, executed_actions);
        }
    }

    (base_state, None, action_timers, executed_actions)
}

pub async fn get_idle_status() -> Result<IdleStatus, IdleError> {
    let conn = Connection::session().await?;

    let ss = ScreenSaverProxy::new(&conn).await?;
    let idle_time = ss.get_session_idle_time().await.unwrap_or(0) as u64;

    let cfg = Config::default();
    let state = calculate_state(idle_time, &cfg, std::collections::HashMap::new(), std::collections::HashSet::new()).0;
    let inhibits_list = inhibits().lock().await;

    Ok(IdleStatus {
        idle_time_secs: idle_time,
        state,
        inhibited: !inhibits_list.is_empty(),
        pending_action: None,
    })
}

pub async fn inhibit(reason: &str) -> Result<u32, IdleError> {
    let conn = Connection::session().await?;

    let ss = ScreenSaverProxy::new(&conn).await?;
    let id = ss.inhibit("crawlds", reason).await?;

    let mut inhibits = inhibits().lock().await;
    let entry = InhibitEntry {
        id,
        reason: reason.to_string(),
    };
    inhibits.push(entry);

    Ok(id)
}

pub async fn uninhibit(id: u32) -> Result<(), IdleError> {
    let conn = Connection::session().await?;

    let mut inhibits = inhibits().lock().await;
    let pos = inhibits.iter().position(|e| e.id == id);

    match pos {
        Some(idx) => {
            let ss = ScreenSaverProxy::new(&conn).await?;
            ss.uninhibit(id).await?;
            inhibits.remove(idx);
            Ok(())
        }
        None => Err(IdleError::InhibitNotFound(id)),
    }
}

pub async fn simulate_activity() -> Result<(), IdleError> {
    inhibit("user_activity").await?;
    uninhibit(0).await?;
    Ok(())
}

pub fn execute_command(command: &str) -> Result<(), IdleError> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| IdleError::CommandError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("Idle command failed: {}", stderr);
    }

    Ok(())
}