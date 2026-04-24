use std::{
    collections::HashMap,
    env,
    fs,
    io,
    os::unix::process::CommandExt,
    path::Path,
    path::PathBuf,
    process::{self, Command, Stdio},
    sync::OnceLock,
    time::Duration,
};

use anyhow::Result;
use clap::Parser;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use signal_hook::{
    consts::{SIGINT, SIGTERM, SIGUSR1},
    iterator::Signals,
};

use crate::output;

fn print_ascii() {
    println!(r#"
        ____ ____      ___        ___     ____  ____  
       / ___|  _ \    / \ \      / / |   |  _ \/ ___| 
      | |   | |_) |  / _ \ \ /\ / /| |   | | | \___ \ 
      | |___|  _ <  / ___ \ V  V / | |___| |_| |___) |
       \____|_| \_\/_/   \_\_/\_/  |_____|____/|____/ 
"#);
}

type IpcTargets = HashMap<String, HashMap<String, Vec<String>>>;

static IS_SESSION_MANAGED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn set_session_managed(val: bool) {
    IS_SESSION_MANAGED.store(val, std::sync::atomic::Ordering::Relaxed);
}

fn is_session_managed() -> bool {
    IS_SESSION_MANAGED.load(std::sync::atomic::Ordering::Relaxed)
}

fn process_exit_code(status: std::process::ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return 128 + sig;
        }
    }
    1
}

fn runtime_dir() -> PathBuf {
    env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir())
}

fn has_systemd_run() -> bool {
    which::which("systemd-run").is_ok()
}

fn pid_file_path() -> PathBuf {
    runtime_dir().join(format!("crawlunix-{}.pid", process::id()))
}

fn write_pid_file(child_pid: u32) -> io::Result<()> {
    fs::write(pid_file_path(), child_pid.to_string())
}

fn remove_pid_file() {
    let _ = fs::remove_file(pid_file_path());
}

fn all_crawlds_pids() -> Vec<i32> {
    let dir = runtime_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return vec![];
    };

    let mut pids = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("crawlunix-") || !name.ends_with(".pid") {
            continue;
        }

        let pid_file = dir.join(&*name);
        let Ok(data) = fs::read_to_string(&pid_file) else {
            continue;
        };

        let Ok(child_pid) = data.trim().parse::<i32>() else {
            let _ = fs::remove_file(&pid_file);
            continue;
        };

        if process_alive(child_pid) {
            pids.push(child_pid);
        } else {
            let _ = fs::remove_file(&pid_file);
            continue;
        }

        let stem = name
            .trim_start_matches("crawlunix-")
            .trim_end_matches(".pid");
        if let Ok(parent_pid) = stem.parse::<i32>() {
            if process_alive(parent_pid) {
                pids.push(parent_pid);
            }
        }
    }

    pids
}

fn first_crawlds_pid() -> Option<i32> {
    let dir = runtime_dir();
    let entries = fs::read_dir(&dir).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("crawlunix-") || !name.ends_with(".pid") {
            continue;
        }

        let pid_file = dir.join(&*name);
        let Ok(data) = fs::read_to_string(&pid_file) else {
            continue;
        };

        let Ok(pid) = data.trim().parse::<i32>() else {
            let _ = fs::remove_file(&pid_file);
            continue;
        };

        if !process_alive(pid) {
            let _ = fs::remove_file(&pid_file);
            continue;
        }

        return Some(pid);
    }

    None
}

fn process_alive(pid: i32) -> bool {
    signal::kill(Pid::from_raw(pid), None).is_ok()
}

fn exec_detached_restart(target_pid: i32) {
    let Ok(self_path) = env::current_exe() else {
        return;
    };

    let _ = unsafe {
        Command::new(&self_path)
            .args(["restart-detached", &target_pid.to_string()])
            .pre_exec(|| {
                nix::unistd::setsid().map_err(|e| io::Error::from_raw_os_error(e as i32))?;
                Ok(())
            })
            .spawn()
    };
}

fn get_config_path() -> String {
    env::var("CRAWLDS_CONFIG_PATH").unwrap_or_else(|_| {
        if let Ok(home) = env::var("HOME") {
            format!("{}/.config/quickshell/crawldesktopshell", home)
        } else {
            String::new()
        }
    })
}

fn get_socket_path() -> String {
    env::var("CRAWLDS_SOCKET").unwrap_or_else(|_| {
        runtime_dir()
            .join("crawlds.sock")
            .to_string_lossy()
            .into_owned()
    })
}

fn write_config_state_file(config_path: &str) -> io::Result<()> {
    let path = runtime_dir().join("crawlds.path");
    fs::write(path, config_path)
}

fn remove_config_state_file() {
    let _ = fs::remove_file(runtime_dir().join("crawlds.path"));
}

fn build_qs_env(socket_path: &str, config_path: &str) -> Vec<(String, String)> {
    let mut extra: Vec<(String, String)> = Vec::new();

    extra.push(("CRAWLDS_SOCKET".into(), socket_path.into()));

    if env::var("QT_LOGGING_RULES").is_err() {
        extra.push(("QT_LOGGING_RULES".into(), "*=false\nqt.quickshell.ipc.debug=true".into()));
    }

    extra.push(("QS_APP_ID".into(), "com.crawlunix.crawlds".into()));

    if is_session_managed() && has_systemd_run() {
        extra.push((
            "CRAWLDS_DEFAULT_LAUNCH_PREFIX".into(),
            "systemd-run --user --scope".into(),
        ));
    }

    if env::var("CRAWLDS_DISABLE_HOT_RELOAD").is_err() {
        if let Ok(home) = env::var("HOME") {
            if !config_path.starts_with(&home) {
                extra.push(("CRAWLDS_DISABLE_HOT_RELOAD".into(), "1".into()));
            }
        }
    }

    if env::var("QT_QPA_PLATFORMTHEME").is_err() {
        extra.push(("QT_QPA_PLATFORMTHEME".into(), "gtk3".into()));
    }
    if env::var("QT_QPA_PLATFORMTHEME_QT6").is_err() {
        extra.push(("QT_QPA_PLATFORMTHEME_QT6".into(), "gtk3".into()));
    }
    if env::var("QT_QPA_PLATFORM").is_err() {
        extra.push(("QT_QPA_PLATFORM".into(), "wayland;xcb".into()));
    }

    extra
}

#[derive(Parser)]
pub struct RunArgs {
    #[arg(long, short = 'c', default_value = "~/.config/quickshell/crawldesktopshell")]
    pub config: String,

    #[arg(short = 'd')]
    pub daemon: bool,

    #[arg(long = "daemon-child", hide = true)]
    pub daemon_child: bool,

    #[arg(last = true)]
    pub args: Vec<String>,
}

pub async fn run(args: RunArgs) -> Result<()> {
    if args.daemon || args.daemon_child {
        run_shell_daemon(args.daemon_child);
        Ok(())
    } else {
        let config_path = if args.config.is_empty() {
            get_config_path()
        } else {
            shellexpand::tilde(&args.config).to_string()
        };

        if !Path::new(&config_path).exists() {
            anyhow::bail!(
                "Quickshell config not found at: {}\n\
                 Run the install script: curl -fsSL https://raw.githubusercontent.com/me-osano/crawlds/master/install.sh | sh --shell-only",
                config_path
            );
        }

        let mut cmd = Command::new("qs");
        cmd.arg("-p").arg(&config_path);

        for arg in &args.args {
            cmd.arg(arg);
        }

        let status = cmd.status()?;

        if !status.success() {
            anyhow::bail!("qs exited with status: {}", status);
        }

        Ok(())
    }
}

pub fn run_shell_interactive(session: bool) {
    set_session_managed(session);

    print_ascii();
    eprintln!("crawlds {}", env!("CARGO_PKG_VERSION"));

    let config_path = get_config_path();
    let socket_path = get_socket_path();

    if let Err(e) = write_config_state_file(&config_path) {
        eprintln!("WARN: Failed to write config state file: {e}");
    }

    std::thread::spawn(|| {
        if let Err(e) = server::start(false) {
            eprintln!("ERROR: Server error: {e}");
        }
    });

    eprintln!("Spawning quickshell with -p {config_path}");

    let extra_env = build_qs_env(&socket_path, &config_path);
    let mut cmd = Command::new("qs");
    cmd.args(["-p", &config_path]);
    for (k, v) in &extra_env {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Error starting quickshell: {e}");
            process::exit(1);
        }
    };

    let child_pid = child.id() as i32;

    if let Err(e) = write_pid_file(child.id()) {
        eprintln!("WARN: Failed to write PID file: {e}");
    }

    let socket_path_clone = socket_path.clone();

    std::thread::spawn(move || {
        let _ = signal::kill(Pid::from_raw(child_pid), Signal::SIGTERM);
    });

    let mut signals =
        Signals::new([SIGINT, SIGTERM, SIGUSR1]).expect("failed to install signal handlers");

    let status = child.wait().expect("failed to wait on quickshell");

    remove_pid_file();
    remove_config_state_file();
    let _ = fs::remove_file(&socket_path_clone);

    for sig in &mut signals {
        match sig {
            SIGUSR1 => {
                if is_session_managed() {
                    eprintln!("Received SIGUSR1, exiting for systemd restart...");
                    let _ = fs::remove_file(&socket_path);
                    process::exit(1);
                }
                eprintln!("Received SIGUSR1, spawning detached restart process...");
                exec_detached_restart(process::id() as i32);
                process::exit(0);
            }
            SIGINT | SIGTERM => {
                eprintln!("Received signal {sig}, shutting down...");
                let _ = fs::remove_file(&socket_path);
                process::exit(process_exit_code(status));
            }
            _ => {}
        }
    }

    process::exit(process_exit_code(status));
}

pub fn restart_shell() {
    let pids = all_crawlds_pids();

    if pids.is_empty() {
        println!("No running Crawl Desktop shell instances found. Starting daemon...");
        run_shell_daemon(false);
        return;
    }

    let current = process::id() as i32;
    let unique: std::collections::HashSet<i32> =
        pids.into_iter().filter(|&p| p != current).collect();

    for pid in unique {
        if !process_alive(pid) {
            continue;
        }
        match signal::kill(Pid::from_raw(pid), Signal::SIGUSR1) {
            Ok(_) => println!("Sent SIGUSR1 to CrawlDS process {pid}"),
            Err(e) => eprintln!("ERROR: Error sending SIGUSR1 to {pid}: {e}"),
        }
    }
}

pub fn kill_shell() {
    let pids = all_crawlds_pids();

    if pids.is_empty() {
        println!("No running CrawlDS shell instances found.");
        return;
    }

    let current = process::id() as i32;
    let unique: std::collections::HashSet<i32> =
        pids.into_iter().filter(|&p| p != current).collect();

    for pid in unique {
        if !process_alive(pid) {
            continue;
        }
        match signal::kill(Pid::from_raw(pid), Signal::SIGKILL) {
            Ok(_) => println!("Killed CrawlDS process {pid}"),
            Err(e) => eprintln!("ERROR: Error killing {pid}: {e}"),
        }
    }

    let dir = runtime_dir();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("crawlunix-") && name.ends_with(".pid") {
                let _ = fs::remove_file(dir.join(&*name));
            }
        }
    }
}

pub fn run_shell_daemon(session: bool) {
    set_session_managed(session);

    let is_daemon_child = env::args().any(|a| a == "--daemon-child");

    if !is_daemon_child {
        eprintln!("crawlds {}", env!("CARGO_PKG_VERSION"));

        let config_path = get_config_path();
        if config_path.is_empty() {
            eprintln!("WARN: No config path set, using default");
        }

        let self_path = env::current_exe().expect("cannot find current executable");
        let mut cmd = Command::new(&self_path);
        cmd.args(["run", "-d", "--daemon-child"]);
        cmd.envs(env::vars());
        if !config_path.is_empty() {
            cmd.env("CRAWLDS_CONFIG_PATH", &config_path);
        }

        unsafe {
            cmd.pre_exec(|| {
                nix::unistd::setsid()
                    .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
                Ok(())
            });
        }

        match cmd.spawn() {
            Ok(child) => println!("Crawl Desktop Shell daemon started (PID: {})", child.id()),
            Err(e) => {
                eprintln!("ERROR: Error starting daemon: {e}");
                process::exit(1);
            }
        }
        return;
    }

    eprintln!("crawlds {}", env!("CARGO_PKG_VERSION"));

    let config_path = get_config_path();
    let socket_path = get_socket_path();

    if let Err(e) = write_config_state_file(&config_path) {
        eprintln!("WARN: Failed to write config state file: {e}");
    }

    std::thread::spawn(|| {
        if let Err(e) = server::start(false) {
            eprintln!("ERROR: Server error: {e}");
        }
    });

    println!("Spawning quickshell with -p {config_path}");

    let dev_null = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/null")
        .expect("cannot open /dev/null");

    let extra_env = build_qs_env(&socket_path, &config_path);
    let mut cmd = Command::new("qs");
    cmd.args(["-p", &config_path]);
    for (k, v) in &extra_env {
        cmd.env(k, v);
    }
    cmd.stdin(dev_null.try_clone().unwrap())
        .stdout(dev_null.try_clone().unwrap())
        .stderr(dev_null);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Error starting daemon: {e}");
            process::exit(1);
        }
    };

    if let Err(e) = write_pid_file(child.id()) {
        eprintln!("WARN: Failed to write PID file: {e}");
    }

    let child_pid = child.id() as i32;
    let socket_path_clone = socket_path.clone();

    let mut signals =
        Signals::new([SIGINT, SIGTERM, SIGUSR1]).expect("failed to install signal handlers");

    std::thread::spawn(move || {
        for sig in &mut signals {
            match sig {
                SIGUSR1 => {
                    if is_session_managed() {
                        println!("Received SIGUSR1, exiting for systemd restart...");
                        let _ = signal::kill(Pid::from_raw(child_pid), Signal::SIGTERM);
                        let _ = fs::remove_file(&socket_path_clone);
                        process::exit(1);
                    }
                    println!("Received SIGUSR1, spawning detached restart process...");
                    exec_detached_restart(process::id() as i32);
                    process::exit(0);
                }
                SIGINT | SIGTERM => {
                    let _ = signal::kill(Pid::from_raw(child_pid), Signal::SIGTERM);
                    let _ = fs::remove_file(&socket_path_clone);
                    remove_pid_file();
                    remove_config_state_file();
                    process::exit(0);
                }
                _ => {}
            }
        }
    });

    let status = child.wait().expect("failed to wait on quickshell");
    remove_pid_file();
    remove_config_state_file();
    let _ = fs::remove_file(&socket_path);
    process::exit(process_exit_code(status));
}

fn qs_has_any_display() -> bool {
    static CELL: OnceLock<bool> = OnceLock::new();
    *CELL.get_or_init(|| {
        Command::new("qs")
            .args(["ipc", "--help"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).contains("--any-display"))
            .unwrap_or(false)
    })
}

pub fn parse_targets_from_ipc_show_output(output: &str) -> IpcTargets {
    let mut targets: IpcTargets = HashMap::new();
    let mut current_target = String::new();

    for line in output.lines() {
        if let Some(after) = line.strip_prefix("target ") {
            current_target = after.trim().to_owned();
            targets.entry(current_target.clone()).or_default();
            continue;
        }

        if line.starts_with("  function") && !current_target.is_empty() {
            let func_line = line.trim_start_matches("  function ");
            let mut parts = func_line.splitn(2, '(');
            let func_name = parts.next().unwrap_or("").to_owned();
            let rest = parts.next().unwrap_or(")");
            let arg_list = rest.splitn(2, ')').next().unwrap_or("");

            let args: Vec<&str> = arg_list.split(',').collect();
            let all_empty = args.iter().all(|a| a.trim().is_empty());

            let entry = targets.entry(current_target.clone()).or_default();
            if all_empty {
                entry.insert(func_name, vec![]);
            } else {
                let mut arg_names = vec![func_name.clone()];
                for arg in &args {
                    let arg_name = arg.trim().splitn(2, ':').next().unwrap_or("").trim().to_owned();
                    arg_names.push(arg_name);
                }
                entry.insert(func_name, arg_names);
            }
        }
    }

    targets
}

pub fn get_shell_ipc_completions(args: &[String]) -> Vec<String> {
    let mut cmd_args = vec!["ipc".to_owned()];
    if qs_has_any_display() {
        cmd_args.push("--any-display".into());
    }
    let config_path = get_config_path();
    cmd_args.extend(["-p".into(), config_path, "show".into()]);

    let output = Command::new("qs")
        .args(&cmd_args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let targets = parse_targets_from_ipc_show_output(&output);

    let args: &[String] = if args.first().map(|s| s.as_str()) == Some("call") {
        &args[1..]
    } else {
        args
    };

    match args.len() {
        0 => {
            let mut completions = vec!["call".to_owned()];
            completions.extend(targets.keys().cloned());
            completions
        }
        1 => targets
            .get(&args[0])
            .map(|funcs| funcs.keys().cloned().collect())
            .unwrap_or_default(),
        n => {
            let func_args = targets
                .get(&args[0])
                .and_then(|funcs| funcs.get(&args[1]))
                .cloned()
                .unwrap_or_default();
            if func_args.len() >= n {
                vec![format!("[{}]", func_args[n - 1])]
            } else {
                vec![]
            }
        }
    }
}

pub fn run_shell_ipc_command(args: &[String]) {
    if args.is_empty() {
        print_ipc_help();
        return;
    }

    let mut ipc_args: Vec<String> = if args[0] != "call" {
        let mut v = vec!["call".to_owned()];
        v.extend_from_slice(args);
        v
    } else {
        args.to_vec()
    };

    let mut cmd_args = vec!["ipc".to_owned()];

    match first_crawlds_pid() {
        Some(pid) => {
            cmd_args.push("--pid".into());
            cmd_args.push(pid.to_string());
        }
        None => {
            if qs_has_any_display() {
                cmd_args.push("--any-display".into());
            }
            let config_path = get_config_path();
            cmd_args.extend(["-p".into(), config_path]);
        }
    }

    cmd_args.append(&mut ipc_args);

    let status = Command::new("qs")
        .args(&cmd_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => process::exit(process_exit_code(s)),
        Err(e) => {
            eprintln!("ERROR: Error running IPC command: {e}");
            process::exit(1);
        }
    }
}

pub fn print_ipc_help() {
    println!("Usage: crawlds ipc <target> <function> [args...]");
    println!();

    let mut cmd_args = vec!["ipc".to_owned()];
    if qs_has_any_display() {
        cmd_args.push("--any-display".into());
    }
    let config_path = get_config_path();
    cmd_args.extend(["-p".into(), config_path, "show".into()]);

    let output = Command::new("qs")
        .args(&cmd_args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    let Some(output) = output else {
        println!("Could not retrieve available IPC targets (is CrawlDS running?)");
        return;
    };

    let targets = parse_targets_from_ipc_show_output(&output);
    if targets.is_empty() {
        println!("No IPC targets available");
        return;
    }

    println!("Targets:");

    let mut target_names: Vec<&str> = targets.keys().map(|s| s.as_str()).collect();
    target_names.sort_unstable();

    for target_name in target_names {
        let funcs = &targets[target_name];
        let mut func_names: Vec<&str> = funcs.keys().map(|s| s.as_str()).collect();
        func_names.sort_unstable();
        println!("  {:<16} {}", target_name, func_names.join(", "));
    }
}

#[derive(Parser)]
pub struct UpdateArgs {
    #[arg(last = true)]
    pass_through: Vec<String>,

    #[arg(long)]
    dry_run: bool,

    #[arg(long, short = 'j')]
    json: bool,
}

pub async fn update(args: UpdateArgs) -> Result<()> {
    let mut passthrough = args.pass_through.clone();
    if args.dry_run {
        passthrough.insert(0, "--dry-run".to_string());
    }

    let mut cmd = std::process::Command::new("bash");
    cmd.arg("-c")
        .arg("curl -fsSL https://raw.githubusercontent.com/me-osano/crawlds/main/pkg/update.sh | bash -s -- \"$@\"")
        .arg("--")
        .args(&passthrough);

    let output_res = cmd.output()?;
    let success = output_res.status.success();
    let stderr = String::from_utf8_lossy(&output_res.stderr).trim().to_string();
    if args.dry_run {
        let tag = String::from_utf8_lossy(&output_res.stdout).trim().to_string();
        let installed = get_installed_version();
        let tag_missing = tag.is_empty();
        if args.json {
            output::print_value(
                &serde_json::json!({
                    "ok": success && !tag_missing,
                    "tag": tag,
                    "installed": installed,
                    "error": if success && !tag_missing { serde_json::Value::Null } else { serde_json::json!(stderr) }
                }),
                true,
            );
        } else if success && !tag_missing {
            let installed_msg = installed.as_deref().unwrap_or("unknown");
            output::print_ok(&format!("latest release tag: {tag}"));
            output::print_ok(&format!("installed version: {installed_msg}"));
        } else {
            let err_msg = if stderr.is_empty() {
                "latest release tag not found".to_string()
            } else {
                format!("latest release tag not found: {stderr}")
            };
            output::print_err(&err_msg);
        }
    } else if args.json {
        output::print_value(&serde_json::json!({"ok": success}), true);
    } else if success {
        output::print_ok("updated crawlds to latest release");
    } else {
        output::print_err("update failed");
    }

    Ok(())
}

fn get_installed_version() -> Option<String> {
    let output = std::process::Command::new("pacman")
        .args(["-Qi", "crawlds"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("Version") {
            let version = rest.splitn(2, ':').nth(1).map(|s| s.trim())?;
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }

    None
}

#[derive(Parser)]
pub struct VersionArgs {
    #[arg(long, short = 'j')]
    json: bool,
}

pub fn version(args: VersionArgs) -> Result<()> {
    if args.json {
        output::print_value(
            &serde_json::json!({
                "name": "crawlds",
                "version": env!("CARGO_PKG_VERSION"),
                "rustc": env!("CARGO_PKG_RUST_VERSION"),
            }),
            true,
        );
    } else {
        output::print_ok(&format!("crawlds {}", env!("CARGO_PKG_VERSION")));
    }
    Ok(())
}

mod server {
    pub fn start(_verbose: bool) -> Result<(), String> {
        Ok(())
    }
}

pub async fn shell_main(command: ShellCommand) -> Result<()> {
    match command {
        ShellCommand::Run(args) => run(args).await,
        ShellCommand::Restart => {
            restart_shell();
            Ok(())
        }
        ShellCommand::Kill => {
            kill_shell();
            Ok(())
        }
        ShellCommand::Ipc(args) => {
            run_shell_ipc_command(&args.args);
            Ok(())
        }
        ShellCommand::Update(args) => update(args).await,
        ShellCommand::Version(args) => version(args),
    }
}

#[derive(Parser)]
pub enum ShellCommand {
    Run(RunArgs),
    Restart,
    Kill,
    Ipc(IpcArgs),
    Update(UpdateArgs),
    Version(VersionArgs),
}

#[derive(Parser)]
pub struct IpcArgs {
    #[arg(last = true)]
    pub args: Vec<String>,
}

#[derive(Parser)]
pub struct RestartDetachedArgs {
    pub target_pid: i32,
}

pub fn run_restart_detached(args: RestartDetachedArgs) {
    let target_pid = args.target_pid;

    std::thread::sleep(Duration::from_millis(200));

    if let Ok(()) = signal::kill(Pid::from_raw(target_pid), Signal::SIGTERM) {}

    std::thread::sleep(Duration::from_millis(500));

    kill_shell();
    run_shell_daemon(false);
}