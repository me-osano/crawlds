mod config;
mod json_server;
mod state;

use anyhow::Context;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use state::AppState;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(log_filter)
        .init();

    info!("crawlds-daemon starting");

    let cfg = config::load().context("failed to load crawlds config")?;
    info!("config loaded from {:?}", cfg.config_path);

    let (event_tx, _) = broadcast::channel(100);
    let notify_store = Arc::new(crawlds_notify::NotifyStore::new(cfg.notifications.max_store));
    let state = Arc::new(AppState::new(cfg.clone(), event_tx.clone(), notify_store));

    let socket_path = PathBuf::from(&cfg.daemon.socket_path);
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .with_context(|| format!("failed to remove stale socket {:?}", socket_path))?;
    }

    let json_server = json_server::JsonServer::new(cfg.config_path.clone());
    json_server.set_state(state.clone(), event_tx.subscribe()).await;

    tokio::spawn(async move {
        if let Err(e) = json_server.run(socket_path).await {
            error!("JSON server error: {}", e);
        }
    });

    spawn_domains(&state).await;

    info!("crawlds-daemon running on {}", cfg.daemon.socket_path);

    tokio::signal::ctrl_c().await?;
    info!("crawlds-daemon shutting down");
    Ok(())
}

async fn spawn_domains(state: &Arc<AppState>) {
    let tx = state.event_tx.clone();
    let cfg = state.config.clone();

    tokio::spawn(crawlds_bluetooth::run(cfg.bluetooth.clone(), tx.clone()));
    tokio::spawn(crawlds_network::run(cfg.network.clone(), tx.clone()));

    let ns = state.notify_store.clone();
    let nc = cfg.notifications.clone();
    tokio::spawn(crawlds_notify::run_with_store(nc, tx.clone(), (*ns).clone()));

    tokio::spawn(crawlds_clipboard::run(cfg.clipboard.clone(), tx.clone()));
    tokio::spawn(crawlds_sysmon::run(cfg.sysmon.clone(), tx.clone()));
    tokio::spawn(crawlds_display::run_brightness(cfg.brightness.clone(), tx.clone()));
    tokio::spawn(crawlds_proc::run(cfg.processes.clone(), tx.clone()));
    tokio::spawn(crawlds_power::run(cfg.power.clone(), tx.clone()));

    let idle_cfg = crawlds_power::idle::Config {
        idle_timeout_secs: cfg.idle.idle_timeout_secs,
        dim_timeout_secs: cfg.idle.dim_timeout_secs,
        sleep_timeout_secs: cfg.idle.sleep_timeout_secs,
        screen_off_timeout_secs: cfg.idle.screen_off_timeout_secs,
        lock_timeout_secs: cfg.idle.lock_timeout_secs,
        suspend_timeout_secs: cfg.idle.suspend_timeout_secs,
        fade_duration_secs: 5,
        screen_off_command: cfg.idle.screen_off_command.clone(),
        lock_command: cfg.idle.lock_command.clone(),
        suspend_command: cfg.idle.suspend_command.clone(),
        resume_screen_off_command: None,
        resume_lock_command: None,
        resume_suspend_command: None,
    };
    tokio::spawn(crawlds_power::idle::run(idle_cfg, tx.clone()));

    let ws_store = (*state.webservice_store).clone();
    let wcfg = cfg.webservice.clone();
    tokio::spawn(crawlds_webservice::run_with_state(wcfg, tx.clone(), ws_store));
}