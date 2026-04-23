//! crawlds-greeter binary
//!
//! This binary can be used as a greetd greeter, providing a simpler
//! alternative to running the full crawlds-daemon.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;

mod config;
mod external;
mod greetd;
mod memory;
mod pam;
mod types;

use config::Config;
use memory::Memory;

#[derive(Parser)]
#[command(name = "crawlds-greeter")]
#[command(about = "CrawlDS Greeter - greetd integration")]
struct Cli {
    #[arg(long, default_value = "/run/greetd.sock")]
    socket: String,

    #[arg(long)]
    cache_dir: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync settings from user config to greeter cache
    Sync,
    /// Probe PAM and external auth
    Probe,
    /// Show greeter status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load or create config
    let mut config = Config::default();
    config.greetd_socket = cli.socket;

    if let Some(cache_dir) = cli.cache_dir {
        config.cache_dir = cache_dir;
    }

    match cli.command {
        Some(Commands::Sync) => {
            sync_settings(&config).await?;
        }
        Some(Commands::Probe) => {
            probe_pam().await?;
        }
        Some(Commands::Status) => {
            show_status(&config).await?;
        }
        None => {
            // Run as greeter
            run_greeter(config).await?;
        }
    }

    Ok(())
}

async fn sync_settings(config: &Config) -> Result<()> {
    println!("Syncing greeter settings to {}", config.cache_dir);

    // Create cache directory if needed
    tokio::fs::create_dir_all(&config.cache_dir).await?;

    // Copy settings from user config if available
    let user_settings = dirs::config_dir()
        .map(|p| p.join("crawlds").join("settings.json"));

    if let Some(src) = user_settings {
        if src.exists() {
            let dst = format!("{}/settings.json", config.cache_dir);
            tokio::fs::copy(&src, &dst).await?;
            println!("Copied settings to {}", dst);
        }
    }

    Ok(())
}

async fn probe_pam() -> Result<()> {
    println!("Probing PAM configuration...");

    let info = pam::PamDetector::detect_pam_info();
    println!("PAM Info:");
    println!("  fprintd available: {}", info.has_fprintd);
    println!("  U2F available: {}", info.has_u2f);
    println!("  Lockout configured: {}", info.lockout_configured);
    println!("  Faillock deny: {}", info.faillock_deny);

    let external = external::ExternalAuthDetector::detect_external_auth_sync();
    println!("\nExternal Auth:");
    println!("  Available: {}", external.available);
    println!("  fprintd has device: {}", external.fprintd_has_device);
    println!("  U2F available: {}", external.has_u2f);

    Ok(())
}

async fn show_status(config: &Config) -> Result<()> {
    println!("Greeter Status");
    println!("=============");
    println!("Socket: {}", config.greetd_socket);
    println!("Cache dir: {}", config.cache_dir);

    let memory = Memory::load(config.clone()).await;
    match memory {
        Ok(m) => {
            println!("\nSession Memory:");
            println!("  Last user: {:?}", m.last_successful_user());
            println!("  Last session: {:?}", m.last_session_id());
        }
        Err(e) => {
            println!("\nFailed to load memory: {}", e);
        }
    }

    Ok(())
}

async fn run_greeter(config: Config) -> Result<()> {
    println!("Starting CrawlDS Greeter on {}", config.greetd_socket);
    println!("This binary is designed to be used with greetd.");
    println!("For full greeter functionality, use Quickshell with the greeter module.");

    // For now, just wait for signals
    tokio::signal::ctrl_c().await?;
    Ok(())
}
