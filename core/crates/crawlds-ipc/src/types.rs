use serde::{Deserialize, Serialize};

// ── Bluetooth ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtDevice {
    pub address: String,
    pub name: Option<String>,
    pub connected: bool,
    pub paired: bool,
    pub rssi: Option<i16>,
    pub battery: Option<u8>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtStatus {
    pub powered: bool,
    pub discovering: bool,
    pub devices: Vec<BtDevice>,
}

// ── Network ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetInterface {
    pub name: String,
    pub state: String,
    pub ip4: Option<String>,
    pub ip6: Option<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub secured: bool,
    pub connected: bool,
    pub existing: bool,
    pub cached: bool,
    pub password_required: bool,
    pub security: String,
    pub frequency_mhz: Option<u32>,
    pub bssid: Option<String>,
    pub last_seen_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWifiDetails {
    pub ifname: Option<String>,
    pub ssid: Option<String>,
    pub signal: Option<u8>,
    pub frequency_mhz: Option<u32>,
    pub band: Option<String>,
    pub channel: Option<u32>,
    pub rate_mbps: Option<u32>,
    pub ip4: Option<String>,
    pub ip6: Vec<String>,
    pub gateway4: Option<String>,
    pub gateway6: Vec<String>,
    pub dns4: Vec<String>,
    pub dns6: Vec<String>,
    pub security: Option<String>,
    pub bssid: Option<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthernetInterface {
    pub ifname: String,
    pub connected: bool,
    pub mac: Option<String>,
    pub ip4: Option<String>,
    pub ip6: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEthernetDetails {
    pub ifname: String,
    pub speed: Option<String>,
    pub ipv4: Option<String>,
    pub ipv6: Vec<String>,
    pub gateway4: Option<String>,
    pub gateway6: Vec<String>,
    pub dns4: Vec<String>,
    pub dns6: Vec<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetMode {
    Station,
    Ap,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetStatus {
    pub connectivity: String,
    pub wifi_enabled: bool,
    pub network_enabled: bool,
    pub wifi_available: bool,
    pub ethernet_available: bool,
    pub mode: NetMode,
    pub active_ssid: Option<String>,
    pub interfaces: Vec<NetInterface>,
}

// ── Hotspot ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HotspotBackend {
    NetworkManager,
    Hostapd,
}

impl Default for HotspotBackend {
    fn default() -> Self {
        Self::NetworkManager
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotConfig {
    pub ssid: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub iface: Option<String>,
    #[serde(default)]
    pub band: Option<String>,
    #[serde(default)]
    pub channel: Option<u32>,
    #[serde(default)]
    pub backend: Option<HotspotBackend>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotspotClient {
    pub mac: String,
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotspotStatus {
    pub active: bool,
    pub ssid: Option<String>,
    pub iface: Option<String>,
    pub band: Option<String>,
    pub channel: Option<u32>,
    pub clients: Vec<HotspotClient>,
    #[serde(default)]
    pub backend: HotspotBackend,
    pub supports_virtual_ap: bool,
}

// ── Notifications ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: String,
    pub urgency: Urgency,
    pub actions: Vec<NotificationAction>,
    pub expire_timeout_ms: i32,
    pub timestamp_ms: u64,
}

// ── Clipboard ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipEntry {
    pub content: String,
    pub mime: String,
    pub timestamp_ms: u64,
}

// ── Sysmon ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStatus {
    pub aggregate: f32,
    pub cores: Vec<f32>,
    pub frequency_mhz: Vec<u64>,
    pub load_avg: LoadAvg,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemStatus {
    pub total_kb: u64,
    pub used_kb: u64,
    pub available_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetTraffic {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_bps: u64,
    pub tx_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    pub name: Option<String>,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStatus {
    pub mount: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub filesystem: Option<String>,
}

// ── Brightness ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrightnessStatus {
    pub device: String,
    pub current: u64,
    pub max: u64,
    pub percent: f32,
}

// ── Processes ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub exe_path: Option<String>,
    pub cpu_percent: f32,
    pub cpu_ticks: Option<f64>,
    pub mem_rss_kb: u64,
    pub status: String,
    pub user: Option<String>,
    pub cmd: Vec<String>,
}

// ── Power (UPower) ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BatteryState {
    Charging,
    Discharging,
    FullyCharged,
    Empty,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryStatus {
    pub percent: f64,
    pub state: BatteryState,
    pub time_to_empty_secs: Option<i64>,
    pub time_to_full_secs: Option<i64>,
    pub energy_rate_w: Option<f64>,
    pub voltage_v: Option<f64>,
    pub temperature_c: Option<f64>,
    pub on_ac: bool,
}

// ── Disk (UDisks2) ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    pub device: String,
    pub label: Option<String>,
    pub size_bytes: u64,
    pub filesystem: Option<String>,
    pub mount_point: Option<String>,
    pub mounted: bool,
    pub removable: bool,
}

// ── VFS (Disk usage, file info, search) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub mount_point: String,
    pub filesystem: Option<String>,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub percent_used: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_ms: u64,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FsEvent {
    Created { path: String },
    Modified { path: String },
    Deleted { path: String },
    Renamed { from: String, to: String },
}

impl FsEvent {
    pub fn path(&self) -> Option<String> {
        match self {
            FsEvent::Created { path } => Some(path.clone()),
            FsEvent::Modified { path } => Some(path.clone()),
            FsEvent::Deleted { path } => Some(path.clone()),
            FsEvent::Renamed { from, .. } => Some(from.clone()),
        }
    }
}

// ── Greeter (greetd) ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GreeterState {
    Inactive,
    Authenticating,
    AwaitingInput,
    Ready,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GreeterMessageType {
    Visible,
    Secret,
    Info,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreeterStatus {
    pub state: GreeterState,
    pub username: String,
    pub message: Option<String>,
    pub message_type: Option<GreeterMessageType>,
    pub last_error: Option<String>,
}

// ── RSS ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssItem {
    pub feed_title: String,
    pub title: String,
    pub link: String,
    pub published: Option<String>,
    pub summary: Option<String>,
    pub thumbnail: Option<String>,
}

// ── Wallpaper ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperBackend {
    Swww,
    Unknown,
}

impl Default for WallpaperBackend {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperMonitorState {
    pub name: String,
    pub current: Option<String>,
    pub transition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperStatus {
    pub backend: WallpaperBackend,
    pub backend_available: bool,
    pub monitors: Vec<WallpaperMonitorState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperSetOptions {
    pub path: String,
    #[serde(default)]
    pub monitor: Option<String>,
    #[serde(default)]
    pub transition: Option<String>,
}

impl WallpaperSetOptions {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            monitor: None,
            transition: None,
        }
    }

    pub fn with_monitor(mut self, monitor: impl Into<String>) -> Self {
        self.monitor = Some(monitor.into());
        self
    }

    pub fn with_transition(mut self, transition: impl Into<String>) -> Self {
        self.transition = Some(transition.into());
        self
    }
}

// ── Wallhaven ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallpaper {
    pub id: String,
    pub url: String,
    pub thumb_url: String,
    pub resolution: String,
    pub purity: String,
    pub tags: Vec<String>,
    pub uploaded_at: String,
    pub file_size: u64,
}

// ── Theme ────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSchemeType {
    Vibrant,
    Tonalspot,
    Excited,
    Rainbow,
    Dark,
    Light,
    Amoled,
}

impl Default for ThemeSchemeType {
    fn default() -> Self {
        Self::Tonalspot
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMetadata {
    pub name: String,   // theme name for generic, wallpaper name for dynamic
    #[serde(rename = "source", default)]
    pub source: String, // "generic" | "dynamic"
    #[serde(default)]
    pub scheme: String, // "static" for generic, "tonal-spot" etc for dynamic
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub on_primary: String,
    pub secondary: String,
    pub on_secondary: String,
    pub tertiary: String,
    pub on_tertiary: String,
    pub error: String,
    pub on_error: String,
    pub surface: String,
    pub on_surface: String,
    pub surface_variant: String,
    pub on_surface_variant: String,
    pub outline: String,
    pub shadow: String,
    pub hover: String,
    pub on_hover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColors {
    pub normal: TerminalColorSet,
    pub bright: TerminalColorSet,
    pub foreground: String,
    pub background: String,
    pub selection_fg: String,
    pub selection_bg: String,
    pub cursor_text: String,
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColorSet {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMode {
    #[serde(flatten)]
    pub colors: ThemeColors,
    pub terminal: TerminalColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeData {
    pub metadata: ThemeMetadata,
    pub dark: ThemeMode,
    pub light: ThemeMode,
}
