//! JSON-RPC 2.0 Server for CrawlDS
//! Protocol: {"jsonrpc": "2.0", "method": "CmdName", "params": {...}, "id": 1}
//!          -> {"jsonrpc": "2.0", "result": {...}, "id": 1}
//! Events (NDJSON): {"jsonrpc": "2.0", "method": "event", "params": {...}}

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::RwLock as TokioRwLock;
use tracing::{debug, error, info};

use crate::state::AppState;
use crawlds_ipc::CrawlEvent;
use crawlds_webservice::WallhavenWorker;

const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: Option<serde_json::Value>, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
            id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum Command {
    Hello { client: Option<String>, version: Option<String> },
    Ping,
    Get { key: Option<String> },
    Set { key: String, value: serde_json::Value },
    ThemeList,
    ThemeCurrent,
    ThemeSet { name: String },
    ThemeGet { name: Option<String> },
    SysmonCpu,
    SysmonMem,
    SysmonDisk,
    SysmonNet,
    SysmonGpu,
    PowerBattery,
    PowerProfileGet,
    PowerProfileSet { profile: String },
    NetStatus,
    NetWifiList,
    NetWifiDetails,
    NetWifiConnect { ssid: String, password: Option<String> },
    NetWifiDisconnect,
    NetWifiScan,
    NetWifiForget { ssid: String },
    NetHotspotStart,
    NetHotspotStop,
    NetHotspotStatus,
    NetPower { enabled: bool },
    NetEthList,
    NetEthConnect { interface: String },
    NetEthDetails { interface: String },
    NetEthDisconnect,
    BtStatus,
    BtDevices,
    BtScan,
    BtConnect { address: String },
    BtDisconnect { address: String },
    BtPower { enabled: bool },
    BtPair { address: String },
    BtRemove { address: String },
    BtDiscoverable { enabled: bool },
    BtTrust { address: String, trusted: bool },
    BtAlias { address: String, alias: String },
    BtPairable { enabled: bool },
    NotifyDismiss { id: u32 },
    DiskEject { path: String },
    BrightnessGet,
    BrightnessSet { value: i32 },
    BrightnessInc { value: i32 },
    BrightnessDec { value: i32 },
    NotifyList,
    NotifySend { title: String, body: String },
    ClipGet,
    ClipHistory { limit: Option<usize> },
    ClipDelete { id: String },
    ClipClear,
    ClipPin { id: String },
    ClipUnpin { id: String },
    ClipCopy { id: String },
    DiskList,
    DiskMount { path: String },
    DiskUnmount { path: String },
    VfsDiskUsage,
    VfsList { path: Option<String> },
    VfsSearch { query: String },
    VfsMkdir { path: String },
    VfsDelete { path: String },
    VfsCopy { from: String, to: String },
    VfsMove { from: String, to: String },
    VfsRename { path: String, name: String },
    VfsTrash { path: String },
    IdleStatus,
    IdleActivity,
    IdleInhibit { why: String },
    IdleUninhibit { id: String },
    ClipSet { text: String },
    ClipPinnedCount,
    RssFeeds,
    RssRefresh,
    RssAdd { url: String },
    RssRemove { url: String },
    WallhavenSearch { query: String },
    WallhavenRandom,
    GreeterStatus,
    GreeterSession { username: Option<String> },
    GreeterLaunch,
    GreeterRespond { response: String },
    GreeterCancel,
    GreeterCreate,
    GreeterPamInfo,
    GreeterExternalAuth { user: String, auth: String },
    Health,
    Subscribe,
    ProcList { sort: Option<String>, top: Option<usize> },
    ProcTop { limit: Option<usize> },
    ProcFind { name: String },
    ProcKill { pid: u32, force: bool },
    ProcWatch { pid: u32 },
}

pub struct JsonServer {
    config_path: PathBuf,
    state: Arc<TokioRwLock<Option<Arc<AppState>>>>,
    event_rx: Arc<TokioRwLock<Option<tokio::sync::broadcast::Receiver<CrawlEvent>>>>,
}

impl JsonServer {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            state: Arc::new(TokioRwLock::new(None)),
            event_rx: Arc::new(TokioRwLock::new(None)),
        }
    }

    pub async fn set_state(&self, state: Arc<AppState>, event_rx: tokio::sync::broadcast::Receiver<CrawlEvent>) {
        let mut lock = self.state.write().await;
        *lock = Some(state);
        let mut rx_lock = self.event_rx.write().await;
        *rx_lock = Some(event_rx);
    }

    async fn get_state(&self) -> Option<Arc<AppState>> {
        self.state.read().await.clone()
    }

    pub async fn run(&self, socket_path: PathBuf) -> anyhow::Result<()> {
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }
        let listener = UnixListener::bind(&socket_path)?;
        info!("JSON server listening on {:?}", socket_path);
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, &server).await {
                            error!("connection error: {}", e);
                        }
                    });
                }
                Err(e) => error!("accept error: {}", e),
            }
        }
    }
}

impl Clone for JsonServer {
    fn clone(&self) -> Self {
        Self {
            config_path: self.config_path.clone(),
            state: self.state.clone(),
            event_rx: self.event_rx.clone(),
        }
    }
}

async fn handle_connection(mut stream: tokio::net::UnixStream, server: &JsonServer) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut subscribed = false;
    let mut event_rx = {
        let rx_lock = server.event_rx.read().await;
        rx_lock.as_ref().map(|rx| rx.resubscribe())
    };
    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                let n = match result { Ok(n) => n, Err(_) => break };
                if n == 0 { break; }
                let trimmed = line.trim();
                if trimmed.is_empty() { line.clear(); continue; }

                // Parse as JSON-RPC request
                let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
                    Ok(r) => r,
                    Err(_) => {
                        let resp = JsonRpcResponse::error(None, -32600, "Invalid JSON-RPC request");
                        let mut response = serde_json::to_string(&resp).unwrap();
                        response.push('\n');
                        writer.write_all(response.as_bytes()).await?;
                        writer.flush().await?;
                        line.clear();
                        continue;
                    }
                };

                // Handle Subscribe specially (event subscription mode)
                if req.method == "Subscribe" {
                    subscribed = true;
                    let resp = JsonRpcResponse::success(
                        req.id,
                        serde_json::json!({"subscribed": true, "time_ms": now_ms()}),
                    );
                    let mut response = serde_json::to_string(&resp).unwrap();
                    response.push('\n');
                    writer.write_all(response.as_bytes()).await?;
                    writer.flush().await?;
                    line.clear();
                    continue;
                }

                // Execute command
                let resp = server.execute(req.method, req.params, req.id).await;
                let mut response = serde_json::to_string(&resp).unwrap();
                response.push('\n');
                writer.write_all(response.as_bytes()).await?;
                writer.flush().await?;
                line.clear();
            }
            _ = async { if let Some(ref mut rx) = event_rx { rx.recv().await.ok() } else { None } } => {}
        }
        if subscribed {
            if let Some(ref mut rx) = event_rx {
                while let Ok(evt) = rx.try_recv() {
                    // NDJSON event format
                    let event_json = serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "event",
                        "params": evt,
                    });
                    let mut response = serde_json::to_string(&event_json).unwrap();
                    response.push('\n');
                    writer.write_all(response.as_bytes()).await?;
                    writer.flush().await?;
                }
            }
        }
    }
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

impl JsonServer {
    pub async fn execute(&self, method: String, params: serde_json::Value, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let cmd: Command = match serde_json::from_value(serde_json::json!({ "method": method, "params": params })) {
            Ok(c) => c,
            Err(e) => return JsonRpcResponse::error(id, -32602, &format!("Invalid params: {}", e)),
        };
        debug!("JSON-RPC method: {:?}", cmd);
        match cmd {
            Command::Hello { client, version } => {
                JsonRpcResponse::success(id, serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "time_ms": now_ms(),
                    "client": client,
                    "client_version": version,
                }))
            }
            Command::Ping => {
                JsonRpcResponse::success(id, serde_json::json!({"time_ms": now_ms()}))
            }
            Command::Get { key } => self.handle_get(key, id),
            Command::Set { key, value } => self.handle_set(key, value, id),
            Command::ThemeList => self.theme_list().await,
            Command::ThemeCurrent => self.theme_current().await,
            Command::ThemeSet { name } => self.theme_set(name).await,
            Command::ThemeGet { name } => self.theme_get(name).await,
            Command::SysmonCpu => self.sysmon_cpu().await,
            Command::SysmonMem => self.sysmon_mem().await,
            Command::SysmonDisk => self.sysmon_disk().await,
            Command::SysmonNet => self.sysmon_net().await,
            Command::SysmonGpu => self.sysmon_gpu().await,
            Command::PowerBattery => self.power_battery().await,
            Command::PowerProfileGet => self.power_profile_get().await,
            Command::PowerProfileSet { profile } => self.power_profile_set(profile).await,
            Command::NetStatus => self.net_status().await,
            Command::NetWifiList => self.net_wifi_list().await,
            Command::NetWifiDetails => self.net_wifi_details().await,
            Command::NetWifiConnect { ssid, password } => self.net_wifi_connect(ssid, password).await,
            Command::NetWifiDisconnect => self.net_wifi_disconnect().await,
            Command::NetWifiScan => self.net_wifi_scan().await,
            Command::NetWifiForget { ssid } => self.net_wifi_forget(ssid).await,
            Command::NetHotspotStart => self.net_hotspot_start().await,
            Command::NetHotspotStop => self.net_hotspot_stop().await,
            Command::NetHotspotStatus => self.net_hotspot_status().await,
            Command::NetPower { enabled } => self.net_power(enabled).await,
            Command::NetEthList => self.net_eth_list().await,
            Command::NetEthConnect { interface } => self.net_eth_connect(interface).await,
            Command::NetEthDisconnect => self.net_eth_disconnect().await,
            Command::NetEthDetails { interface } => self.net_eth_details(interface).await,
            Command::BtStatus => self.bt_status().await,
            Command::BtDevices => self.bt_devices().await,
            Command::BtScan => self.bt_scan().await,
            Command::BtConnect { address } => self.bt_connect(address).await,
            Command::BtDisconnect { address } => self.bt_disconnect(address).await,
            Command::BtPower { enabled } => self.bt_power(enabled).await,
            Command::BtPair { address } => self.bt_pair(address).await,
            Command::BtRemove { address } => self.bt_remove(address).await,
            Command::BtDiscoverable { enabled } => self.bt_discoverable(enabled).await,
            Command::BtTrust { address, trusted } => self.bt_trust(address, trusted).await,
            Command::BtAlias { address, alias } => self.bt_alias(address, alias).await,
            Command::BtPairable { enabled } => self.bt_pairable(enabled).await,
            Command::NotifyList => self.notify_list().await,
            Command::NotifyDismiss { id } => self.notify_dismiss(id).await,
            Command::NotifySend { title, body } => self.notify_send(title, body).await,
            Command::ClipGet => self.clip_get().await,
            Command::ClipHistory { limit } => self.clip_history(limit).await,
            Command::ClipDelete { id } => self.clip_delete(id).await,
            Command::ClipClear => self.clip_clear().await,
            Command::ClipPin { id } => self.clip_pin(id).await,
            Command::ClipUnpin { id } => self.clip_unpin(id).await,
            Command::ClipCopy { id } => self.clip_copy(id).await,
            Command::DiskList => self.disk_list().await,
            Command::DiskMount { path } => self.disk_mount(path).await,
            Command::DiskUnmount { path } => self.disk_unmount(path).await,
            Command::DiskEject { path } => self.disk_eject(path).await,
            Command::VfsDiskUsage => self.vfs_disk_usage().await,
            Command::VfsList { path } => self.vfs_list(path).await,
            Command::VfsSearch { query } => self.vfs_search(query).await,
            Command::VfsMkdir { path } => self.vfs_mkdir(path).await,
            Command::VfsDelete { path } => self.vfs_delete(path).await,
            Command::VfsCopy { from, to } => self.vfs_copy(from, to).await,
            Command::VfsMove { from, to } => self.vfs_move(from, to).await,
            Command::VfsRename { path, name } => self.vfs_rename(path, name).await,
            Command::VfsTrash { path } => self.vfs_trash(path).await,
            Command::IdleStatus => self.idle_status().await,
            Command::IdleActivity => self.idle_activity().await,
            Command::IdleInhibit { why } => self.idle_inhibit(why).await,
            Command::IdleUninhibit { id } => self.idle_uninhibit(id).await,
            Command::ClipSet { text } => self.clip_set(text).await,
            Command::ClipPinnedCount => self.clip_pinned_count().await,
            Command::RssFeeds => self.rss_feeds().await,
            Command::RssRefresh => self.rss_refresh().await,
            Command::RssAdd { url } => self.rss_add(url).await,
            Command::RssRemove { url } => self.rss_remove(url).await,
            Command::WallhavenSearch { query } => self.wallhaven_search(query).await,
            Command::WallhavenRandom => self.wallhaven_random().await,
            Command::GreeterStatus => self.greeter_status().await,
            Command::GreeterSession { username } => self.greeter_session(username).await,
            Command::GreeterLaunch => self.greeter_launch().await,
            Command::GreeterRespond { response } => self.greeter_respond(response).await,
            Command::GreeterCancel => self.greeter_cancel().await,
            Command::GreeterCreate => self.greeter_create().await,
            Command::GreeterPamInfo => self.greeter_pam_info().await,
            Command::GreeterExternalAuth { user, auth } => self.greeter_external_auth(user, auth).await,
            Command::BrightnessGet => self.brightness_get().await,
            Command::BrightnessSet { value } => self.brightness_set(value).await,
            Command::BrightnessInc { value } => self.brightness_inc(value).await,
            Command::BrightnessDec { value } => self.brightness_dec(value).await,
            Command::ProcList { sort, top } => self.proc_list(sort, top).await,
            Command::ProcTop { limit } => self.proc_top(limit).await,
            Command::ProcFind { name } => self.proc_find(name).await,
            Command::ProcKill { pid, force } => self.proc_kill(pid, force).await,
            Command::ProcWatch { pid } => self.proc_watch(pid).await,
            Command::Health => self.health().await,
            Command::Subscribe => JsonRpcResponse::success(id, serde_json::json!({ "subscribed": true })),
        }
    }

    fn handle_get(&self, key: Option<String>, id: Option<serde_json::Value>) -> JsonRpcResponse {
        if key.is_none() {
            let content = std::fs::read_to_string(&self.config_path).unwrap_or_default();
            return JsonRpcResponse::success(id, serde_json::json!({ "config": content }));
        }
        let k = key.unwrap();
        let content = std::fs::read_to_string(&self.config_path).unwrap_or_default();
        let section = k.split('.').next().unwrap_or(&k);
        let target_key = k.split('.').nth(1).unwrap_or(&k);
        let mut in_section = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == format!("[{}]", section) { in_section = true; continue; }
            if in_section && trimmed.starts_with('[') && trimmed.ends_with(']') { break; }
            if in_section && trimmed.starts_with(target_key) && trimmed.contains('=') {
                if let Some(val) = trimmed.splitn(2, '=').nth(1) {
                    return JsonRpcResponse::success(id, serde_json::json!({ "key": k, "value": val.trim() }));
                }
            }
        }
        JsonRpcResponse::error(id, -32601, "key not found")
    }

    fn handle_set(&self, key: String, value: serde_json::Value, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let content = match std::fs::read_to_string(&self.config_path) {
            Ok(c) => c,
            Err(e) => return JsonRpcResponse::error(id, -32000, &e.to_string()),
        };
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let section = key.split('.').next().unwrap_or(&key).to_string();
        let target_key = key.split('.').nth(1).unwrap_or(&key);
        let value_str = value.to_string();
        let new_line = format!("{} = {}", target_key, value_str);
        let mut in_section = false;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == format!("[{}]", section) { in_section = true; continue; }
            if in_section && trimmed.starts_with('[') && trimmed.ends_with(']') { in_section = false; continue; }
            if in_section && trimmed.starts_with(target_key) && trimmed.contains('=') {
                lines[i] = new_line.clone();
                let new_content = lines.join("\n");
                if let Err(e) = std::fs::write(&self.config_path, new_content) {
                    return JsonRpcResponse::error(id, -32000, &e.to_string());
                }
                return JsonRpcResponse::success(id, serde_json::json!({ "ok": true }));
            }
        }
        JsonRpcResponse::error(id, -32601, "section not found")
    }

    async fn theme_list(&self) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let guard = s.theme_manager.lock().await;
            let themes = guard.list_themes();
            JsonRpcResponse::success(None, serde_json::json!({ "themes": themes }))
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn theme_current(&self) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let guard = s.theme_manager.lock().await;
            let name = guard.get_current_name();
            JsonRpcResponse::success(None, serde_json::json!({ "theme": name }))
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn theme_set(&self, name: String) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let mut guard = s.theme_manager.lock().await;
            match guard.set_theme(&name) {
                Ok(_) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
                Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn theme_get(&self, name: Option<String>) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let guard = s.theme_manager.lock().await;
            if let Some(n) = name {
                let theme = guard.get_theme(&n);
                JsonRpcResponse::success(None, serde_json::json!({ "theme": theme }))
            } else {
                let theme = guard.get_current();
                JsonRpcResponse::success(None, serde_json::json!({ "theme": theme }))
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn sysmon_cpu(&self) -> JsonRpcResponse {
        let cpu = crawlds_sysmon::get_cpu();
        JsonRpcResponse::success(None, serde_json::to_value(cpu).unwrap_or_default())
    }

    async fn sysmon_mem(&self) -> JsonRpcResponse {
        let mem = crawlds_sysmon::get_mem();
        JsonRpcResponse::success(None, serde_json::to_value(mem).unwrap_or_default())
    }

    async fn sysmon_disk(&self) -> JsonRpcResponse {
        let disks = crawlds_sysmon::get_disks();
        JsonRpcResponse::success(None, serde_json::to_value(disks).unwrap_or_default())
    }

    async fn sysmon_net(&self) -> JsonRpcResponse {
        let net = crawlds_sysmon::get_net();
        JsonRpcResponse::success(None, serde_json::to_value(net).unwrap_or_default())
    }

    async fn power_battery(&self) -> JsonRpcResponse {
        match crawlds_power::get_battery().await {
            Ok(battery) => JsonRpcResponse::success(None, serde_json::to_value(battery).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn power_profile_get(&self) -> JsonRpcResponse {
        match crawlds_power::get_profile().await {
            Ok(profile) => JsonRpcResponse::success(None, serde_json::to_value(profile).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn power_profile_set(&self, profile: String) -> JsonRpcResponse {
        let profile_num = match profile.parse::<u32>() {
            Ok(n) => n,
            Err(_) => return JsonRpcResponse::error(None, -32602, "invalid profile number"),
        };
        match crawlds_power::set_profile(profile_num).await {
            Ok(profile) => JsonRpcResponse::success(None, serde_json::to_value(profile).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_status(&self) -> JsonRpcResponse {
        match crawlds_network::get_status().await {
            Ok(status) => JsonRpcResponse::success(None, serde_json::to_value(status).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_wifi_list(&self) -> JsonRpcResponse {
        match crawlds_network::list_wifi().await {
            Ok(list) => JsonRpcResponse::success(None, serde_json::to_value(list).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_wifi_connect(&self, ssid: String, password: Option<String>) -> JsonRpcResponse {
        match crawlds_network::connect_wifi(&ssid, password.as_deref()).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_wifi_disconnect(&self) -> JsonRpcResponse {
        match crawlds_network::disconnect_wifi().await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_wifi_scan(&self) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn net_wifi_forget(&self, _ssid: String) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn net_hotspot_start(&self) -> JsonRpcResponse {
        JsonRpcResponse::error(None, -32000, "hotspot not supported")
    }

    async fn net_hotspot_stop(&self) -> JsonRpcResponse {
        match crawlds_network::stop_hotspot().await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_hotspot_status(&self) -> JsonRpcResponse {
        match crawlds_network::hotspot_status().await {
            Ok(status) => JsonRpcResponse::success(None, serde_json::to_value(status).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_power(&self, _enabled: bool) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn net_eth_list(&self) -> JsonRpcResponse {
        match crawlds_network::list_ethernet().await {
            Ok(list) => JsonRpcResponse::success(None, serde_json::to_value(list).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_eth_connect(&self, interface: String) -> JsonRpcResponse {
        match crawlds_network::connect_ethernet(Some(&interface)).await {
            Ok(iface) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true, "interface": iface })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_eth_disconnect(&self) -> JsonRpcResponse {
        match crawlds_network::disconnect_ethernet(None).await {
            Ok(iface) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true, "interface": iface })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn net_eth_details(&self, interface: String) -> JsonRpcResponse {
        match crawlds_network::get_ethernet_details(Some(&interface)).await {
            Ok(details) => JsonRpcResponse::success(None, serde_json::to_value(details).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn brightness_get(&self) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let backlight = match crawlds_display::Backlight::open(&s.config.brightness) {
                Ok(b) => b,
                Err(e) => return JsonRpcResponse::error(None, -32000, &e.to_string()),
            };
            match backlight.status() {
                Ok(status) => JsonRpcResponse::success(None, serde_json::to_value(status).unwrap_or_default()),
                Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn brightness_set(&self, value: i32) -> JsonRpcResponse {
        let value = value as f32;
        if let Some(s) = self.get_state().await {
            let backlight = match crawlds_display::Backlight::open(&s.config.brightness) {
                Ok(b) => b,
                Err(e) => return JsonRpcResponse::error(None, -32000, &e.to_string()),
            };
            match backlight.set_percent(value, &s.config.brightness) {
                Ok(status) => {
                    let _ = s.event_tx.send(CrawlEvent::Brightness(crawlds_ipc::events::BrightnessEvent::Changed { status: status.clone() }));
                    JsonRpcResponse::success(None, serde_json::to_value(status).unwrap_or_default())
                }
                Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn notify_list(&self) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let notifications = s.notify_store.list();
            let list: Vec<serde_json::Value> = notifications.iter().map(|n| {
                serde_json::json!({ "id": n.id, "summary": n.summary, "body": n.body, "urgency": n.urgency })
            }).collect();
            JsonRpcResponse::success(None, serde_json::json!({ "notifications": list }))
        } else { 
            JsonRpcResponse::success(None, serde_json::json!({ "notifications": [] })) 
        }
    }

    async fn clip_get(&self) -> JsonRpcResponse {
        if let Ok(output) = std::process::Command::new("sh")
            .args(["-c", "wl-paste --type text 2>/dev/null || echo ''"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return JsonRpcResponse::success(None, serde_json::json!({ "entry": { "content": text, "mime": "text/plain" } }));
        }
        JsonRpcResponse::success(None, serde_json::json!({ "entry": serde_json::Value::Null }))
    }

    async fn clip_history(&self, limit: Option<usize>) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let limit = limit.unwrap_or(50);
            let entries = s.clipboard_store.get_history(limit).await;
            let history: Vec<serde_json::Value> = entries.into_iter().map(|e| {
                serde_json::json!({ "id": e.id, "content": e.content, "pinned": e.pinned })
            }).collect();
            JsonRpcResponse::success(None, serde_json::json!({ "history": history }))
        } else { 
            JsonRpcResponse::success(None, serde_json::json!({ "history": [] })) 
        }
    }

    async fn clip_clear(&self) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            s.clipboard_store.clear_history().await;
            JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn disk_list(&self) -> JsonRpcResponse {
        match crawlds_vfs::list_devices().await {
            Ok(d) => JsonRpcResponse::success(None, serde_json::to_value(d).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn vfs_disk_usage(&self) -> JsonRpcResponse {
        match crawlds_vfs::get_disk_usage().await {
            Ok(u) => JsonRpcResponse::success(None, serde_json::to_value(u).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn vfs_list(&self, path: Option<String>) -> JsonRpcResponse {
        match crawlds_vfs::list_dir(path.as_deref().unwrap_or("/")).await {
            Ok(entries) => JsonRpcResponse::success(None, serde_json::to_value(entries).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn idle_status(&self) -> JsonRpcResponse {
        match crawlds_power::idle::get_idle_status().await {
            Ok(s) => JsonRpcResponse::success(None, serde_json::to_value(s).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn idle_activity(&self) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "active": true }))
    }

    async fn idle_inhibit(&self, _why: String) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true, "id": "1" }))
    }

    async fn idle_uninhibit(&self, _id: String) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn clip_set(&self, text: String) -> JsonRpcResponse {
        use std::process::Command;
        use tokio::task;
        let _ = task::spawn_blocking(move || {
            let _ = Command::new("sh").args(["-c", &format!("echo -n '{}' | wl-copy", text)]).output();
        });
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn clip_pinned_count(&self) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "count": 0 }))
    }

    async fn rss_feeds(&self) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let feeds = s.webservice_store.feeds.lock().await;
            JsonRpcResponse::success(None, serde_json::to_value((*feeds).clone()).unwrap_or_default())
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn wallhaven_search(&self, query: String) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let worker = WallhavenWorker::new(s.config.webservice.wallhaven.clone());
            match worker.search(Some(query), vec![], None, None, 1).await {
                Ok(w) => JsonRpcResponse::success(None, serde_json::to_value(w).unwrap_or_default()),
                Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn greeter_status(&self) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let guard = s.greeter.lock().await;
            let status = guard.status();
            JsonRpcResponse::success(None, serde_json::to_value(status).unwrap_or_default())
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn greeter_session(&self, _username: Option<String>) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let mut guard = s.greeter.lock().await;
            guard.clear_session();
            JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn health(&self) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "status": "ok", "timestamp_ms": now_ms() }))
    }

    async fn sysmon_gpu(&self) -> JsonRpcResponse {
        let gpu = crawlds_sysmon::get_gpu();
        JsonRpcResponse::success(None, serde_json::to_value(gpu).unwrap_or_default())
    }

    async fn net_wifi_details(&self) -> JsonRpcResponse {
        match crawlds_network::get_wifi_details().await {
            Ok(d) => JsonRpcResponse::success(None, serde_json::to_value(d).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_status(&self) -> JsonRpcResponse {
        match crawlds_bluetooth::get_status().await {
            Ok(s) => JsonRpcResponse::success(None, serde_json::to_value(s).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_devices(&self) -> JsonRpcResponse {
        match crawlds_bluetooth::get_devices().await {
            Ok(d) => JsonRpcResponse::success(None, serde_json::to_value(d).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_scan(&self) -> JsonRpcResponse {
        match crawlds_bluetooth::scan().await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_connect(&self, address: String) -> JsonRpcResponse {
        match crawlds_bluetooth::connect(&address).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_disconnect(&self, address: String) -> JsonRpcResponse {
        match crawlds_bluetooth::disconnect(&address).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_power(&self, enabled: bool) -> JsonRpcResponse {
        match crawlds_bluetooth::set_powered(enabled).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn clip_delete(&self, id: String) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let deleted = s.clipboard_store.delete_entry(&id).await;
            JsonRpcResponse::success(None, serde_json::json!({ "ok": deleted }))
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn clip_pin(&self, id: String) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            match s.clipboard_store.pin_entry(&id).await {
                Ok(_) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
                Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn clip_unpin(&self, id: String) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            s.clipboard_store.unpin_entry(&id).await;
            JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn vfs_search(&self, query: String) -> JsonRpcResponse {
        match crawlds_vfs::search_home(&query, 50).await {
            Ok(results) => JsonRpcResponse::success(None, serde_json::to_value(results).unwrap_or_default()),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn vfs_mkdir(&self, path: String) -> JsonRpcResponse {
        match crawlds_vfs::create_directory(std::path::Path::new(&path)).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn vfs_delete(&self, path: String) -> JsonRpcResponse {
        match crawlds_vfs::delete_file(&path).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn rss_add(&self, url: String) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            s.webservice_store.add_feed(&url).await;
            JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn rss_remove(&self, url: String) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            s.webservice_store.remove_feed(&url).await;
            JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn rss_refresh(&self) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn wallhaven_random(&self) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let worker = WallhavenWorker::new(s.config.webservice.wallhaven.clone());
            match worker.random(1).await {
                Ok(w) => JsonRpcResponse::success(None, serde_json::to_value(w).unwrap_or_default()),
                Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn bt_pair(&self, address: String) -> JsonRpcResponse {
        match crawlds_bluetooth::pair(&address).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_remove(&self, address: String) -> JsonRpcResponse {
        match crawlds_bluetooth::remove_device(&address).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_discoverable(&self, enabled: bool) -> JsonRpcResponse {
        match crawlds_bluetooth::set_discoverable(enabled).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_trust(&self, address: String, trusted: bool) -> JsonRpcResponse {
        match crawlds_bluetooth::set_trusted(&address, trusted).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_alias(&self, address: String, alias: String) -> JsonRpcResponse {
        match crawlds_bluetooth::set_alias(&address, &alias).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn bt_pairable(&self, enabled: bool) -> JsonRpcResponse {
        match crawlds_bluetooth::set_pairable(enabled).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn notify_send(&self, _title: String, _body: String) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn notify_dismiss(&self, id: u32) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            if s.notify_store.remove(id).is_some() {
                let _ = s.event_tx.send(CrawlEvent::Notify(crawlds_ipc::events::NotifyEvent::Closed { id, reason: 3 }));
                JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
            } else {
                JsonRpcResponse::error(None, -32601, "notification not found")
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn clip_copy(&self, _id: String) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn disk_mount(&self, path: String) -> JsonRpcResponse {
        match crawlds_vfs::mount(path.as_str()).await {
            Ok(mp) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true, "mount_point": mp })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn disk_unmount(&self, path: String) -> JsonRpcResponse {
        match crawlds_vfs::unmount(path.as_str()).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn disk_eject(&self, path: String) -> JsonRpcResponse {
        match crawlds_vfs::eject(&path).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn vfs_copy(&self, from: String, to: String) -> JsonRpcResponse {
        match crawlds_vfs::copy_files(&from, &to).await {
            Ok(n) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true, "copied": n })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn vfs_move(&self, from: String, to: String) -> JsonRpcResponse {
        match crawlds_vfs::move_files(&from, &to).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn vfs_rename(&self, path: String, name: String) -> JsonRpcResponse {
        match crawlds_vfs::rename_file(&path, &name).await {
            Ok(new_path) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true, "path": new_path })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn vfs_trash(&self, path: String) -> JsonRpcResponse {
        match crawlds_vfs::move_to_trash(vec![std::path::PathBuf::from(&path)]).await {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn greeter_launch(&self) -> JsonRpcResponse {
        JsonRpcResponse::error(None, -32000, "use session command")
    }

    async fn greeter_respond(&self, _response: String) -> JsonRpcResponse {
        JsonRpcResponse::error(None, -32000, "greeter not active")
    }

    async fn greeter_cancel(&self) -> JsonRpcResponse {
        JsonRpcResponse::error(None, -32000, "greeter not active")
    }

    async fn greeter_create(&self) -> JsonRpcResponse {
        JsonRpcResponse::error(None, -32000, "greeter not active")
    }

    async fn greeter_pam_info(&self) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "authok": false, "error": "" }))
    }

    async fn greeter_external_auth(&self, _user: String, _auth: String) -> JsonRpcResponse {
        JsonRpcResponse::success(None, serde_json::json!({ "ok": true }))
    }

    async fn brightness_inc(&self, value: i32) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let backlight = match crawlds_display::Backlight::open(&s.config.brightness) {
                Ok(b) => b,
                Err(e) => return JsonRpcResponse::error(None, -32000, &e.to_string()),
            };
            match backlight.adjust_percent(value as f32, &s.config.brightness) {
                Ok(status) => {
                    let _ = s.event_tx.send(CrawlEvent::Brightness(crawlds_ipc::events::BrightnessEvent::Changed { status: status.clone() }));
                    JsonRpcResponse::success(None, serde_json::json!({ "status": status }))
                }
                Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn brightness_dec(&self, value: i32) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let backlight = match crawlds_display::Backlight::open(&s.config.brightness) {
                Ok(b) => b,
                Err(e) => return JsonRpcResponse::error(None, -32000, &e.to_string()),
            };
            match backlight.adjust_percent(-value as f32, &s.config.brightness) {
                Ok(status) => {
                    let _ = s.event_tx.send(CrawlEvent::Brightness(crawlds_ipc::events::BrightnessEvent::Changed { status: status.clone() }));
                    JsonRpcResponse::success(None, serde_json::json!({ "status": status }))
                }
                Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
            }
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn proc_list(&self, sort: Option<String>, top: Option<usize>) -> JsonRpcResponse {
        if let Some(s) = self.get_state().await {
            let sort = sort.unwrap_or_else(|| s.config.processes.default_sort.clone());
            let top = top.unwrap_or(s.config.processes.default_top);
            JsonRpcResponse::success(None, serde_json::to_value(crawlds_proc::list_processes(&sort, top)).unwrap_or_default())
        } else { JsonRpcResponse::error(None, -32000, "no state") }
    }

    async fn proc_top(&self, limit: Option<usize>) -> JsonRpcResponse {
        let limit = limit.unwrap_or(10);
        let top_cpu = crawlds_proc::list_processes("cpu", limit);
        let top_mem = crawlds_proc::list_processes("mem", limit);
        JsonRpcResponse::success(None, serde_json::json!({
            "top_by_cpu": top_cpu,
            "top_by_mem": top_mem
        }))
    }

    async fn proc_find(&self, name: String) -> JsonRpcResponse {
        if name.is_empty() {
            return JsonRpcResponse::error(None, -32602, "name is required");
        }
        JsonRpcResponse::success(None, serde_json::to_value(crawlds_proc::find_processes(&name)).unwrap_or_default())
    }

    async fn proc_kill(&self, pid: u32, force: bool) -> JsonRpcResponse {
        match crawlds_proc::kill_process(pid, force) {
            Ok(()) => JsonRpcResponse::success(None, serde_json::json!({ "ok": true })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }

    async fn proc_watch(&self, pid: u32) -> JsonRpcResponse {
        match crawlds_proc::watch_pid(pid).await {
            Ok(name) => JsonRpcResponse::success(None, serde_json::json!({ "pid": pid, "name": name, "exit_code": serde_json::Value::Null })),
            Err(e) => JsonRpcResponse::error(None, -32000, &e.to_string()),
        }
    }
}
