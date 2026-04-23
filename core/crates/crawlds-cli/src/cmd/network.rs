// cmd/network.rs
use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::{client::CrawlClient, output};

#[derive(Args)]
pub struct NetArgs {
    #[arg(long)]
    pub status: bool,
    #[arg(long)]
    pub wifi: bool,
    #[arg(long)]
    pub eth: bool,
    #[arg(long)]
    pub hotspot: bool,
    #[arg(long)]
    pub scan: bool,
    #[arg(long)]
    pub list: bool,
    #[arg(long)]
    pub details: bool,
    #[arg(long)]
    pub connect: bool,
    #[arg(long)]
    pub disconnect: bool,
    #[arg(long)]
    pub forget: bool,
    #[arg(long)]
    pub ssid: Option<String>,
    #[arg(long)]
    pub password: Option<String>,
    #[arg(long)]
    pub iface: Option<String>,
    #[arg(long)]
    pub band: Option<String>,
    #[arg(long)]
    pub channel: Option<u32>,
    #[arg(long, value_name = "networkmanager|hostapd")]
    pub backend: Option<String>,
    #[arg(long, value_name = "on|off")]
    pub power: Option<String>,
}

pub async fn run(client: CrawlClient, args: NetArgs, json: bool) -> Result<()> {
    if let Some(power) = args.power.as_deref() {
        let enabled = matches!(power, "on" | "true" | "1");
        let res = client.post("/network/power", json!({ "on": enabled })).await?;
        if json {
            output::print_value(&res, true);
        } else {
            output::print_ok(if enabled { "Network enabled" } else { "Network disabled" });
        }
    } else if args.hotspot {
        if args.connect {
            let ssid = args.ssid.clone().unwrap_or_else(|| {
                "CrawlDS-Hotspot".to_string()
            });
            let mut payload = json!({ "ssid": ssid });
            if let Some(ref pwd) = args.password {
                payload["password"] = json!(pwd);
            }
            if let Some(ref band) = args.band {
                payload["band"] = json!(band);
            }
            if let Some(ch) = args.channel {
                payload["channel"] = json!(ch);
            }
            if let Some(ref be) = args.backend {
                payload["backend"] = json!(be);
            }
            let res = client.post("/network/hotspot/start", payload).await?;
            if json {
                output::print_value(&res, true);
            } else {
                let ssid_out = res["ssid"].as_str().unwrap_or(&ssid);
                let iface_out = res["iface"].as_str().unwrap_or("?");
                output::print_ok(&format!("Hotspot started: '{ssid_out}' on {iface_out}"));
            }
        } else if args.disconnect {
            let res = client.post("/network/hotspot/stop", json!({})).await?;
            if json {
                output::print_value(&res, true);
            } else {
                output::print_ok("Hotspot stopped");
            }
        } else {
            let res = client.get("/network/hotspot/status").await?;
            output::print_value(&res, json);
        }
    } else if args.wifi {
        if args.forget {
            let ssid = args.ssid.clone().unwrap_or_default();
            let res = client.post("/network/wifi/forget", json!({ "ssid": ssid })).await?;
            if json {
                output::print_value(&res, true);
            } else {
                output::print_ok(&format!("Network '{ssid}' forgotten"));
            }
        } else if args.scan {
            let res = client.post("/network/wifi/scan", json!({})).await?;
            if json {
                output::print_value(&res, true);
            } else {
                output::print_ok("WiFi scan requested");
            }
        } else if args.details {
            let res = client.get("/network/wifi/details").await?;
            output::print_value(&res, json);
        } else if args.connect {
            let ssid = args.ssid.clone().unwrap_or_default();
            let res = client
                .post(
                    "/network/wifi/connect",
                    json!({ "ssid": ssid, "password": args.password }),
                )
                .await?;
            if json {
                output::print_value(&res, true);
            } else {
                output::print_ok("WiFi connect requested");
            }
        } else if args.disconnect {
            let res = client.post("/network/wifi/disconnect", json!({})).await?;
            if json {
                output::print_value(&res, true);
            } else {
                output::print_ok("WiFi disconnected");
            }
        } else if args.list {
            let res = client.get("/network/wifi").await?;
            output::print_value(&res, json);
        } else {
            let res = client.get("/network/wifi").await?;
            output::print_value(&res, json);
        }
    } else if args.eth {
        if args.connect {
            let res = client
                .post(
                    "/network/eth/connect",
                    json!({ "interface": args.iface }),
                )
                .await?;
            if json {
                output::print_value(&res, true);
            } else {
                let iface_out = res["interface"].as_str().unwrap_or("?");
                output::print_ok(&format!("Ethernet connected on {iface_out}"));
            }
        } else if args.disconnect {
            let res = client
                .post(
                    "/network/eth/disconnect",
                    json!({ "interface": args.iface }),
                )
                .await?;
            if json {
                output::print_value(&res, true);
            } else {
                let iface_out = res["interface"].as_str().unwrap_or("?");
                output::print_ok(&format!("Ethernet disconnected on {iface_out}"));
            }
        } else if args.details {
            let query = if let Some(iface) = args.iface.as_deref() {
                format!("?interface={}", urlencoding::encode(iface))
            } else {
                String::new()
            };
            let res = client.get(&format!("/network/eth/details{query}")).await?;
            output::print_value(&res, json);
        } else if args.list {
            let res = client.get("/network/eth").await?;
            output::print_value(&res, json);
        } else {
            let res = client.get("/network/eth").await?;
            output::print_value(&res, json);
        }
    } else if args.status {
        let res = client.get("/network/status").await?;
        if json {
            output::print_value(&res, true);
        } else {
            output::print_table(&[
                ("connectivity", res["connectivity"].as_str().unwrap_or("?").to_string()),
                ("wifi", res["wifi_enabled"].as_bool().unwrap_or(false).to_string()),
                ("ssid", res["active_ssid"].as_str().unwrap_or("—").to_string()),
            ]);
        }
    } else {
        let res = client.get("/network/status").await?;
        if json {
            output::print_value(&res, true);
        } else if let Some(interfaces) = res["interfaces"].as_array() {
            println!(
                "  {:<12}  {:<12}  {:<15}  {}",
                "IFACE", "STATE", "IP", "MAC"
            );
            for iface in interfaces {
                let name = iface["name"].as_str().unwrap_or("?");
                let state = iface["state"].as_str().unwrap_or("?");
                let ip4 = iface["ip4"].as_str().unwrap_or("—");
                let mac = iface["mac"].as_str().unwrap_or("—");
                println!("  {name:<12}  {state:<12}  {ip4:<15}  {mac}");
            }
        }
    }
    Ok(())
}
