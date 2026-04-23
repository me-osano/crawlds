use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level daemon config — mirrors core.toml structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub config_path: PathBuf,
    #[serde(default)]
    pub assets_dir: PathBuf,
    #[serde(default)]
    pub cache_dir: PathBuf,
    pub theme: ThemeConfig,
    pub daemon: DaemonConfig,
    pub greeter: GreeterConfig,
    pub bluetooth: crawlds_bluetooth::Config,
    pub network: crawlds_network::Config,
    pub notifications: crawlds_notify::Config,
    pub clipboard: crawlds_clipboard::Config,
    pub sysmon: crawlds_sysmon::Config,
    pub brightness: crawlds_display::Config,
    pub processes: crawlds_proc::Config,
    pub power: crawlds_power::Config,
    pub idle: IdleConfig,
    pub vfs: crawlds_vfs::Config,
    pub webservice: crawlds_webservice::Config,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeConfig {
    #[serde(default)]
    pub current: Option<String>,
    #[serde(default)]
    pub variant: Option<String>,
    #[serde(default)]
    pub default_scheme: String,
    #[serde(default)]
    pub default_mode: String,
    #[serde(default)]
    pub auto_generate: bool,
    #[serde(default)]
    pub cache_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Path to the Unix socket. Defaults to $XDG_RUNTIME_DIR/crawlds.sock
    pub socket_path: String,
    /// Optional TCP bind address for HTTP bridge (e.g. 127.0.0.1:9280). Empty disables.
    pub tcp_addr: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreeterConfig {
    /// Path to the greetd IPC socket (default: /run/greetd.sock).
    pub greetd_socket: String,
    /// How long to keep auth sessions alive in seconds after inactivity.
    pub session_ttl_secs: u64,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            greetd_socket: "/run/greetd.sock".into(),
            session_ttl_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleConfig {
    pub idle_timeout_secs: u64,
    pub dim_timeout_secs: u64,
    pub sleep_timeout_secs: u64,
    pub screen_off_timeout_secs: u64,
    pub lock_timeout_secs: u64,
    pub suspend_timeout_secs: u64,
    pub fade_duration_secs: u64,
    pub screen_off_command: Option<String>,
    pub lock_command: Option<String>,
    pub suspend_command: Option<String>,
}

impl Default for IdleConfig {
    fn default() -> Self {
        Self {
            idle_timeout_secs: 300,
            dim_timeout_secs: 60,
            sleep_timeout_secs: 600,
            screen_off_timeout_secs: 600,
            lock_timeout_secs: 660,
            suspend_timeout_secs: 1800,
            fade_duration_secs: 5,
            screen_off_command: None,
            lock_command: None,
            suspend_command: None,
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
        Self {
            socket_path: format!("{runtime_dir}/crawlds.sock"),
            tcp_addr: String::new(),
            log_level: "info".into(),
        }
    }
}

pub fn load() -> anyhow::Result<Config> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap_or_default()));
    let config_path = PathBuf::from(&config_home)
        .join("crawlds")
        .join("core.toml");

    let mut config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file(&config_path))
        .merge(Env::prefixed("CRAWLDS_").split("__"))
        .extract()?;

    config.config_path = config_path;

    // Set default assets_dir if not configured
    if config.assets_dir.as_os_str().is_empty() {
        config.assets_dir = PathBuf::from("/usr/share/local/crawlds/assets");
    }

    // Set default cache_dir if not configured
    if config.cache_dir.as_os_str().is_empty() {
        let cache_home = std::env::var("XDG_CACHE_HOME")
            .unwrap_or_else(|_| format!("{}/.cache", std::env::var("HOME").unwrap_or_default()));
        config.cache_dir = PathBuf::from(cache_home).join("crawlds");
    }

    Ok(config)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: PathBuf::new(),
            assets_dir: PathBuf::new(),
            cache_dir: PathBuf::new(),
            theme: ThemeConfig::default(),
            daemon: DaemonConfig::default(),
            greeter: GreeterConfig::default(),
            bluetooth: crawlds_bluetooth::Config::default(),
            network: crawlds_network::Config::default(),
            notifications: crawlds_notify::Config::default(),
            clipboard: crawlds_clipboard::Config::default(),
            sysmon: crawlds_sysmon::Config::default(),
            brightness: crawlds_display::Config::default(),
            processes: crawlds_proc::Config::default(),
            power: crawlds_power::Config::default(),
            idle: IdleConfig::default(),
            vfs: crawlds_vfs::Config::default(),
            webservice: crawlds_webservice::Config::default(),
        }
    }
}
