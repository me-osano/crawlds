use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use crate::output;

const GREETD_CONFIG_PATH: &str = "/etc/greetd/config.toml";
const DEFAULT_CACHE_DIR: &str = "/var/cache/crawlds-greeter";

#[derive(Args)]
pub struct GreeterArgs {
    #[command(subcommand)]
    command: GreeterCommand,
}

#[derive(Subcommand)]
enum GreeterCommand {
    /// Install and configure the greeter (greetd)
    Install(GreeterInstallArgs),
    /// Enable the greeter in greetd config
    Enable(GreeterEnableArgs),
    /// Sync CrawlDS settings/theme into greeter cache
    Sync(GreeterSyncArgs),
    /// Report greeter config and sync status
    Status,
    /// Uninstall greeter config and restore previous DM
    Uninstall(GreeterUninstallArgs),
}

#[derive(Args)]
pub struct GreeterInstallArgs {
    /// Non-interactive mode: skip prompts
    #[arg(long, short = 'y')]
    yes: bool,
    /// Override compositor (niri, hyprland, sway, etc.)
    #[arg(long)]
    compositor: Option<String>,
    /// Override CrawlDS quickshell path (-p)
    #[arg(long)]
    path: Option<String>,
    /// Override greeter cache directory
    #[arg(long)]
    cache_dir: Option<String>,
}

#[derive(Args)]
pub struct GreeterEnableArgs {
    /// Non-interactive mode: skip prompts
    #[arg(long, short = 'y')]
    yes: bool,
    /// Override compositor (niri, hyprland, sway, etc.)
    #[arg(long)]
    compositor: Option<String>,
    /// Override CrawlDS quickshell path (-p)
    #[arg(long)]
    path: Option<String>,
    /// Override greeter cache directory
    #[arg(long)]
    cache_dir: Option<String>,
}

#[derive(Args)]
pub struct GreeterSyncArgs {
    /// Non-interactive mode: skip prompts
    #[arg(long, short = 'y')]
    yes: bool,
    /// Override greeter cache directory
    #[arg(long)]
    cache_dir: Option<String>,
}

#[derive(Args)]
pub struct GreeterUninstallArgs {
    /// Non-interactive mode: skip prompts
    #[arg(long, short = 'y')]
    yes: bool,
}

pub async fn run(args: GreeterArgs) -> Result<()> {
    match args.command {
        GreeterCommand::Install(args) => install(args),
        GreeterCommand::Enable(args) => enable(args),
        GreeterCommand::Sync(args) => sync(args),
        GreeterCommand::Status => status(),
        GreeterCommand::Uninstall(args) => uninstall(args),
    }
}

fn install(args: GreeterInstallArgs) -> Result<()> {
    if !args.yes {
        confirm("This will install/configure greetd and enable the CrawlDS greeter. Continue? [Y/n]: ")?;
    }

    ensure_greetd_installed()?;
    enable(GreeterEnableArgs {
        yes: true,
        compositor: args.compositor,
        path: args.path,
        cache_dir: args.cache_dir.clone(),
    })?;
    sync(GreeterSyncArgs { yes: true, cache_dir: args.cache_dir })?;
    Ok(())
}

fn enable(args: GreeterEnableArgs) -> Result<()> {
    let cache_dir = args.cache_dir.unwrap_or_else(|| DEFAULT_CACHE_DIR.to_string());
    let wrapper = resolve_greeter_wrapper()?;
    let compositor = resolve_compositor(args.compositor, args.yes)?;

    if !args.yes {
        confirm("This will update /etc/greetd/config.toml and may disable other display managers. Continue? [Y/n]: ")?;
    }

    backup_greetd_config()?;
    let greeter_user = detect_greeter_user();
    let command_line = build_greeter_command(&wrapper, &compositor, &cache_dir, args.path.as_deref());
    upsert_greetd_config(&greeter_user, &command_line)?;

    ensure_graphical_target();
    disable_conflicting_display_managers();
    ensure_greetd_enabled();

    output::print_ok("greetd configured for crawlds-greeter");
    Ok(())
}

fn sync(args: GreeterSyncArgs) -> Result<()> {
    let cache_dir = args.cache_dir.unwrap_or_else(|| DEFAULT_CACHE_DIR.to_string());
    if !args.yes {
        confirm("This will sync CrawlDS settings into the greeter cache. Continue? [Y/n]: ")?;
    }

    ensure_cache_dir(&cache_dir)?;
    ensure_symlinks(&cache_dir)?;
    output::print_ok("greeter cache synced");
    Ok(())
}

fn status() -> Result<()> {
    let config = fs::read_to_string(GREETD_CONFIG_PATH).unwrap_or_default();
    let command = read_default_session_command(&config);
    let enabled = command.contains("crawlds-greeter");

    output::print_table(&[
        ("enabled", enabled.to_string()),
        ("command", if command.is_empty() { "<none>".into() } else { command.clone() }),
    ]);

    let cache_dir = extract_cache_dir_from_command(&command).unwrap_or_else(|| DEFAULT_CACHE_DIR.to_string());
    let cache_ok = Path::new(&cache_dir).is_dir();
    output::print_table(&[
        ("cache_dir", cache_dir.clone()),
        ("cache_dir_exists", cache_ok.to_string()),
    ]);

    let (settings_ok, session_ok, colors_ok) = check_symlinks(&cache_dir);
    output::print_table(&[
        ("settings_link", settings_ok.to_string()),
        ("session_link", session_ok.to_string()),
        ("colors_link", colors_ok.to_string()),
    ]);

    Ok(())
}

fn uninstall(args: GreeterUninstallArgs) -> Result<()> {
    if !args.yes {
        confirm("This will disable greetd and restore previous greeter config. Continue? [y/N]: ")?;
    }

    disable_greetd();
    restore_backup_greetd_config().unwrap_or_else(|e| {
        output::print_err(&format!("failed to restore greetd config: {e}"));
    });
    output::print_ok("greeter uninstall complete (reboot recommended)");
    Ok(())
}

fn confirm(prompt: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(prompt.as_bytes())?;
    stdout.flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let response = input.trim().to_lowercase();
    if response == "n" || response == "no" {
        anyhow::bail!("aborted by user");
    }
    Ok(())
}

fn resolve_greeter_wrapper() -> Result<String> {
    let candidates = ["/usr/bin/crawlds-greeter", "~/.local/bin/crawlds-greeter"];
    for candidate in candidates {
        if Path::new(candidate).exists() {
            return Ok(candidate.to_string());
        }
    }
    anyhow::bail!("crawlds-greeter wrapper not found; install it or set up the package")
}

fn resolve_compositor(override_name: Option<String>, non_interactive: bool) -> Result<String> {
    if let Some(name) = override_name {
        return Ok(name.to_lowercase());
    }

    let mut found = Vec::new();
    for comp in ["niri", "hyprland", "sway", "labwc", "mango", "miracle-wm", "scroll"] {
        if command_exists(comp) {
            found.push(comp.to_string());
        }
    }

    if found.is_empty() {
        anyhow::bail!("no supported compositor found (niri, hyprland, sway, labwc, mango, miracle-wm, scroll)");
    }

    if found.len() == 1 || non_interactive {
        return Ok(found[0].clone());
    }

    println!("Multiple compositors detected:");
    for (idx, comp) in found.iter().enumerate() {
        println!("  {}) {}", idx + 1, comp);
    }
    let mut input = String::new();
    print!("Choose compositor: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let choice: usize = input.trim().parse().unwrap_or(0);
    if choice == 0 || choice > found.len() {
        anyhow::bail!("invalid choice");
    }
    Ok(found[choice - 1].clone())
}

fn build_greeter_command(wrapper: &str, compositor: &str, cache_dir: &str, path: Option<&str>) -> String {
    let mut cmd = format!("{wrapper} --command {compositor} --cache-dir {cache_dir}");
    if let Some(path) = path {
        cmd.push_str(&format!(" -p {path}"));
    }
    cmd
}

fn backup_greetd_config() -> Result<()> {
    let path = Path::new(GREETD_CONFIG_PATH);
    if !path.exists() {
        return Ok(());
    }
    let ts = current_timestamp();
    let backup = format!("{GREETD_CONFIG_PATH}.backup-{ts}");
    run_sudo(&["cp", GREETD_CONFIG_PATH, &backup])
        .context("failed to backup greetd config")?;
    run_sudo(&["chmod", "644", &backup]).ok();
    Ok(())
}

fn restore_backup_greetd_config() -> Result<()> {
    let mut backups = list_backups()?;
    backups.sort_by(|a, b| b.cmp(a));
    for candidate in backups {
        let content = fs::read_to_string(&candidate).unwrap_or_default();
        if content.contains("crawlds-greeter") {
            continue;
        }
        run_sudo(&["cp", candidate.to_string_lossy().as_ref(), GREETD_CONFIG_PATH])?;
        run_sudo(&["chmod", "644", GREETD_CONFIG_PATH]).ok();
        return Ok(());
    }

    let fallback = r#"[terminal]
vt = 1

[default_session]
user = "greeter"
command = "agreety --cmd /bin/bash"
"#;
    let tmp_path = write_temp("greetd-fallback-", fallback)?;
    run_sudo(&["cp", &tmp_path, GREETD_CONFIG_PATH])?;
    run_sudo(&["chmod", "644", GREETD_CONFIG_PATH]).ok();
    Ok(())
}

fn upsert_greetd_config(user: &str, command: &str) -> Result<()> {
    let content = fs::read_to_string(GREETD_CONFIG_PATH).unwrap_or_else(|_| "[terminal]\nvt = 1\n\n[default_session]\n".to_string());
    let updated = upsert_default_session(&content, user, command);
    let tmp_path = write_temp("greetd-config-", &updated)?;
    run_sudo(&["mkdir", "-p", "/etc/greetd"]).ok();
    run_sudo(&["install", "-o", "root", "-g", "root", "-m", "0644", &tmp_path, GREETD_CONFIG_PATH])?;
    Ok(())
}

fn upsert_default_session(content: &str, user: &str, command: &str) -> String {
    let mut out = Vec::new();
    let mut in_default = false;
    let mut found = false;
    let mut user_set = false;
    let mut cmd_set = false;

    for line in content.lines() {
        if let Some(section) = parse_toml_section(line) {
            if in_default {
                if !user_set {
                    out.push(format!("user = \"{user}\""));
                }
                if !cmd_set {
                    out.push(format!("command = \"{command}\""));
                }
            }
            in_default = section == "default_session";
            if in_default {
                found = true;
                user_set = false;
                cmd_set = false;
            }
            out.push(line.to_string());
            continue;
        }

        if in_default {
            let trimmed = strip_toml_comment(line);
            if trimmed.starts_with("user =") || trimmed.starts_with("user=") {
                out.push(format!("user = \"{user}\""));
                user_set = true;
                continue;
            }
            if trimmed.starts_with("command =") || trimmed.starts_with("command=") {
                out.push(format!("command = \"{command}\""));
                cmd_set = true;
                continue;
            }
        }

        out.push(line.to_string());
    }

    if in_default {
        if !user_set {
            out.push(format!("user = \"{user}\""));
        }
        if !cmd_set {
            out.push(format!("command = \"{command}\""));
        }
    }

    if !found {
        out.push(String::new());
        out.push("[default_session]".into());
        out.push(format!("user = \"{user}\""));
        out.push(format!("command = \"{command}\""));
    }

    out.join("\n")
}

fn strip_toml_comment(line: &str) -> String {
    let trimmed = line.trim();
    if let Some(idx) = trimmed.find('#') {
        return trimmed[..idx].trim().to_string();
    }
    trimmed.to_string()
}

fn parse_toml_section(line: &str) -> Option<String> {
    let trimmed = strip_toml_comment(line);
    if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() > 2 {
        return Some(trimmed[1..trimmed.len() - 1].trim().to_string());
    }
    None
}

fn read_default_session_command(content: &str) -> String {
    let mut in_default = false;
    for line in content.lines() {
        if let Some(section) = parse_toml_section(line) {
            in_default = section == "default_session";
            continue;
        }
        if !in_default {
            continue;
        }
        let trimmed = strip_toml_comment(line);
        if trimmed.starts_with("command") {
            if let Some((_, val)) = trimmed.split_once('=') {
                return val.trim().trim_matches('"').to_string();
            }
        }
    }
    String::new()
}

fn extract_cache_dir_from_command(command: &str) -> Option<String> {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    for (idx, token) in tokens.iter().enumerate() {
        if *token == "--cache-dir" {
            return tokens.get(idx + 1).map(|v| v.trim_matches('"').to_string());
        }
        if let Some(rest) = token.strip_prefix("--cache-dir=") {
            return Some(rest.trim_matches('"').to_string());
        }
    }
    None
}

fn ensure_cache_dir(cache_dir: &str) -> Result<()> {
    run_sudo(&["mkdir", "-p", cache_dir])?;
    let group = detect_greeter_group();
    let user = detect_greeter_user();
    let owner = format!("{user}:{group}");
    if run_sudo(&["chown", &owner, cache_dir]).is_err() {
        let fallback = format!("root:{group}");
        run_sudo(&["chown", &fallback, cache_dir])?;
    }
    run_sudo(&["chmod", "2770", cache_dir])?;

    for subdir in [".local", ".local/state", ".local/share", ".cache"] {
        let path = format!("{cache_dir}/{subdir}");
        run_sudo(&["mkdir", "-p", &path])?;
        run_sudo(&["chown", &owner, &path]).ok();
        run_sudo(&["chmod", "2770", &path]).ok();
    }

    Ok(())
}

fn ensure_symlinks(cache_dir: &str) -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    let links = [
        (
            format!("{home}/.config/crawlds/settings.json"),
            format!("{cache_dir}/settings.json"),
        ),
        (
            format!("{home}/.local/state/crawlds/session.json"),
            format!("{cache_dir}/session.json"),
        ),
        (
            format!("{home}/.cache/crawlds/crawlds-colors.json"),
            format!("{cache_dir}/colors.json"),
        ),
    ];

    for (source, target) in links {
        run_sudo(&["rm", "-f", &target]).ok();
        run_sudo(&["ln", "-sf", &source, &target])?;
    }
    Ok(())
}

fn check_symlinks(cache_dir: &str) -> (bool, bool, bool) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    let settings = check_symlink(
        &format!("{cache_dir}/settings.json"),
        &format!("{home}/.config/crawlds/settings.json"),
    );
    let session = check_symlink(
        &format!("{cache_dir}/session.json"),
        &format!("{home}/.local/state/crawlds/session.json"),
    );
    let colors = check_symlink(
        &format!("{cache_dir}/colors.json"),
        &format!("{home}/.cache/crawlds/crawlds-colors.json"),
    );
    (settings, session, colors)
}

fn check_symlink(target: &str, source: &str) -> bool {
    let link = fs::read_link(target).ok();
    match link {
        Some(dest) => dest.to_string_lossy() == source,
        None => false,
    }
}

fn detect_greeter_group() -> String {
    let data = fs::read_to_string("/etc/group").unwrap_or_default();
    for name in ["greeter", "greetd", "_greeter"] {
        if data.lines().any(|line| line.starts_with(&format!("{name}:"))) {
            return name.to_string();
        }
    }
    "greeter".to_string()
}

fn detect_greeter_user() -> String {
    let passwd = fs::read_to_string("/etc/passwd").unwrap_or_default();
    for name in ["greeter", "greetd", "_greeter"] {
        if passwd.lines().any(|line| line.starts_with(&format!("{name}:"))) {
            return name.to_string();
        }
    }
    "greeter".to_string()
}

fn ensure_greetd_installed() -> Result<()> {
    if command_exists("greetd") || Path::new("/usr/sbin/greetd").exists() || Path::new("/sbin/greetd").exists() {
        return Ok(());
    }
    anyhow::bail!("greetd not found; install greetd first")
}

fn ensure_greetd_enabled() {
    let _ = run_sudo(&["systemctl", "enable", "--force", "greetd"]);
}

fn disable_greetd() {
    let _ = run_sudo(&["systemctl", "disable", "greetd"]);
}

fn disable_conflicting_display_managers() {
    for dm in ["gdm", "gdm3", "lightdm", "sddm", "lxdm", "xdm", "cosmic-greeter"] {
        let _ = run_sudo(&["systemctl", "disable", dm]);
    }
}

fn ensure_graphical_target() {
    let _ = run_sudo(&["systemctl", "set-default", "graphical.target"]);
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
}

fn run_sudo(args: &[&str]) -> Result<()> {
    let status = Command::new("sudo").args(args).status()?;
    if !status.success() {
        anyhow::bail!("sudo command failed: sudo {}", args.join(" "));
    }
    Ok(())
}

fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}", now.as_secs())
}

fn write_temp(prefix: &str, contents: &str) -> Result<String> {
    let mut path = std::env::temp_dir();
    path.push(format!("{}{}", prefix, current_timestamp()));
    fs::write(&path, contents)?;
    Ok(path.to_string_lossy().to_string())
}

fn list_backups() -> Result<Vec<std::path::PathBuf>> {
    let dir = Path::new("/etc/greetd");
    let mut backups = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("config.toml.backup-") {
                    backups.push(path);
                }
            }
        }
    }
    backups.sort_by(|a, b| b.cmp(a));
    Ok(backups)
}
