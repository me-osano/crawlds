mod client;
mod cmd;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "crawlds",
    version,
    about = "System services CLI — Bluetooth, network, audio, brightness and more",
    long_about = None,
)]
struct Cli {
    /// Override the daemon socket path
    #[arg(long, env = "CRAWLDS_SOCKET", global = true)]
    socket: Option<String>,

    /// Output raw JSON instead of formatted output
    #[arg(long, short = 'j', global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the Quickshell desktop shell
    Run(cmd::shell::RunArgs),
    /// Restart the running shell
    Restart,
    /// Kill the running shell
    Kill,
    #[doc(hidden)]
    RestartDetached(cmd::shell::RestartDetachedArgs),
    /// IPC commands to the shell
    Ipc(cmd::shell::IpcArgs),
    /// Update crawlds to the latest release
    Update(cmd::shell::UpdateArgs),
    /// Show version information
    Version(cmd::shell::VersionArgs),
    /// Bluetooth management
    Bluetooth(cmd::bluetooth::BtArgs),
    /// Network management
    Network(cmd::network::NetArgs),
    /// Notification control
    Notify(cmd::notify::NotifyArgs),
    /// Clipboard access
    Clipboard(cmd::clipboard::ClipArgs),
    /// System monitoring (CPU, memory, disk)
    Sysmon(cmd::sysmon::SysmonArgs),
    /// Display brightness control
    Brightness(cmd::brightness::BrightnessArgs),
    /// Process management
    Proc(cmd::proc_::ProcArgs),
    /// Battery and power status
    Power(cmd::power::PowerArgs),
    /// Disk and removable media management
    Disk(cmd::disk::DiskArgs),
    /// Daemon control
    Daemon(cmd::daemon::DaemonArgs),
    /// Greeter (greetd) management
    Greeter(cmd::greeter::GreeterArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = color_eyre::install();

    let cli = Cli::parse();

    let socket_path = cli.socket.unwrap_or_else(|| {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
        format!("{runtime_dir}/crawlds.sock")
    });

    let client = client::CrawlClient::new(socket_path);
    let json_mode = cli.json;

    match cli.command {
        Commands::Run(args)            => cmd::shell::run(args).await?,
        Commands::Restart              => { cmd::shell::restart_shell(); },
        Commands::Kill                 => { cmd::shell::kill_shell(); },
        Commands::RestartDetached(args) => { cmd::shell::run_restart_detached(args); },
        Commands::Ipc(args)            => { cmd::shell::run_shell_ipc_command(&args.args); },
        Commands::Update(args)         => cmd::shell::update(args).await?,
        Commands::Version(args)        => cmd::shell::version(args)?,
        Commands::Bluetooth(args)         => cmd::bluetooth::run(client, args, json_mode).await?,
        Commands::Network(args)           => cmd::network::run(client, args, json_mode).await?,
        Commands::Notify(args)            => cmd::notify::run(client, args, json_mode).await?,
        Commands::Clipboard(args)             => cmd::clipboard::run(client, args, json_mode).await?,
        Commands::Sysmon(args)           => cmd::sysmon::run(client, args, json_mode).await?,
        Commands::Brightness(args)       => cmd::brightness::run(client, args, json_mode).await?,
        Commands::Proc(args)             => cmd::proc_::run(client, args, json_mode).await?,
        Commands::Power(args)            => cmd::power::run(client, args, json_mode).await?,
        Commands::Disk(args)             => cmd::disk::run(client, args, json_mode).await?,
        Commands::Daemon(args)           => cmd::daemon::run(client, args, json_mode).await?,
        Commands::Greeter(args)          => cmd::greeter::run(args).await?,
    }

    Ok(())
}
