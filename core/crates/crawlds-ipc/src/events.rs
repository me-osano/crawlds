use crate::types::{
    ActiveEthernetDetails, ActiveWifiDetails, BatteryStatus, BlockDevice, BrightnessStatus,
    BtDevice, ClipEntry, CpuStatus, DiskUsage, EthernetInterface, FsEvent, GpuStatus,
    GreeterMessageType, GreeterStatus, HotspotClient, HotspotStatus, MemStatus, NetMode,
    NetTraffic, Notification, ProcessInfo, RssItem, ThemeData, Wallpaper, WifiNetwork,
};
use serde::{Deserialize, Serialize};

/// All events broadcast over the SSE `/events` stream.
/// Quickshell and CLI --watch consumers filter by domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "domain", content = "data", rename_all = "snake_case")]
pub enum CrawlEvent {
    // Bluetooth
    Bluetooth(BtEvent),
    // Network
    Network(NetEvent),
    // Notifications
    Notify(NotifyEvent),
    // Clipboard
    Clipboard(ClipboardEvent),
    // Sysmon
    Sysmon(SysmonEvent),
    // Brightness
    Brightness(BrightnessEvent),
    // Processes
    Proc(ProcEvent),
    // Power
    Power(PowerEvent),
    // Idle
    Idle(IdleEvent),
    // Disk
    Disk(DiskEvent),
    // Greeter
    Greeter(GreeterEvent),
    // Webservice (RSS, Wallhaven)
    Webservice(WebserviceEvent),
    // Theme
    Theme(ThemeEvent),
    // Daemon lifecycle
    Daemon(DaemonEvent),
}

// ── Per-domain event types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BtEvent {
    DeviceDiscovered { device: BtDevice },
    DeviceConnected { device: BtDevice },
    DeviceDisconnected { address: String },
    DeviceRemoved { address: String },
    AdapterPowered { on: bool },
    ScanStarted,
    ScanStopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum NetEvent {
    Connected { ssid: Option<String>, iface: String },
    Disconnected { iface: String },
    IpChanged { iface: String, ip: String },
    WifiEnabled,
    WifiDisabled,
    WifiScanStarted,
    WifiScanFinished,
    WifiListUpdated { networks: Vec<WifiNetwork> },
    ActiveWifiDetailsChanged { details: ActiveWifiDetails },
    EthernetInterfacesChanged { interfaces: Vec<EthernetInterface> },
    ActiveEthernetDetailsChanged { details: ActiveEthernetDetails },
    ModeChanged { mode: NetMode },
    ConnectivityChanged { state: String },
    HotspotStarted { status: HotspotStatus },
    HotspotStopped,
    HotspotStatusChanged { status: HotspotStatus },
    HotspotClientJoined { client: HotspotClient },
    HotspotClientLeft { mac: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum NotifyEvent {
    New { notification: Notification },
    Closed { id: u32, reason: u32 },
    ActionInvoked { id: u32, action_key: String },
    Replaced { notification: Notification },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ClipboardEvent {
    Changed { entry: ClipEntry },
    PrimaryChanged { entry: ClipEntry },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SysmonEvent {
    CpuUpdate { cpu: CpuStatus },
    MemUpdate { mem: MemStatus },
    NetUpdate { traffic: NetTraffic },
    GpuUpdate { gpu: GpuStatus },
    CpuSpike { usage: f32, threshold: f32 },
    MemPressure { used_percent: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BrightnessEvent {
    Changed { status: BrightnessStatus },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ProcEvent {
    Spawned {
        pid: u32,
        name: String,
    },
    Exited {
        pid: u32,
        name: String,
        exit_code: Option<i32>,
    },
    TopUpdate {
        top_by_cpu: Vec<ProcessInfo>,
        top_by_mem: Vec<ProcessInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum PowerEvent {
    BatteryUpdate { status: BatteryStatus },
    AcConnected,
    AcDisconnected,
    LowBattery { percent: f64 },
    Critical { percent: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleEvent {
    pub event: String,
    pub idle_time_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum DiskEvent {
    DeviceMounted {
        device: BlockDevice,
    },
    DeviceUnmounted {
        device_path: String,
    },
    DeviceAdded {
        device: BlockDevice,
    },
    DeviceRemoved {
        device_path: String,
    },
    DiskUsageUpdated {
        usage: Vec<DiskUsage>,
    },
    #[serde(rename = "fs_changed")]
    FsChanged {
        fs_event: FsEvent,
    },
    OperationProgress {
        operation_id: String,
        operation_kind: String,
        current_file: String,
        processed_bytes: u64,
        total_bytes: u64,
        files_processed: u32,
        total_files: u32,
        percent: f64,
        status: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GreeterEvent {
    StateChanged {
        status: GreeterStatus,
    },
    AuthMessage {
        message: String,
        message_type: GreeterMessageType,
    },
    AuthSuccess,
    AuthFailure {
        message: String,
    },
    SessionStarted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum DaemonEvent {
    Started,
    Stopping,
    DomainError { domain: String, message: String },
}

// ── Webservice (RSS, Wallhaven) ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WebserviceEvent {
    RssFeedUpdated {
        feed_url: String,
        items: Vec<RssItem>,
    },
    RssFeedError {
        feed_url: String,
        message: String,
    },
    RssFeedsRefreshed,
    WallhavenResults {
        walls: Vec<Wallpaper>,
    },
    WallhavenError {
        message: String,
    },
}

// ── Theme ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ThemeEvent {
    Loaded { theme: ThemeData },
    Changed { theme: ThemeData, variant: String },
    ListUpdated { themes: Vec<String> },
    Error { message: String },
    DynamicGenerated { theme: ThemeData, variant: String },
    DynamicApplied { theme: ThemeData, variant: String },
    DynamicError { message: String },
}
