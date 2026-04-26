use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::{client::CrawlClient, output};

#[derive(Args)]
pub struct BtArgs {
    /// List paired/known devices
    #[arg(long)]
    pub list: bool,

    /// Start device discovery scan
    #[arg(long)]
    pub scan: bool,

    /// Connect to a device by address
    #[arg(long, value_name = "ADDRESS")]
    pub connect: Option<String>,

    /// Disconnect a device by address
    #[arg(long, value_name = "ADDRESS")]
    pub disconnect: Option<String>,

    /// Power the adapter on or off
    #[arg(long, value_name = "on|off")]
    pub power: Option<String>,

    /// Show adapter status
    #[arg(long)]
    pub status: bool,

    /// Set adapter discoverable on/off
    #[arg(long, value_name = "on|off")]
    pub discoverable: Option<String>,

    /// Set adapter pairable on/off
    #[arg(long, value_name = "on|off")]
    pub pairable: Option<String>,

    /// Set device alias (name)
    #[arg(long, value_name = "ADDRESS", number_of_values = 2)]
    pub alias: Option<Vec<String>>,

    /// Pair with a device by address
    #[arg(long, value_name = "ADDRESS")]
    pub pair: Option<String>,

    /// Trust or untrust a device by address
    #[arg(long, value_name = "on|off", number_of_values = 2)]
    pub trust: Option<Vec<String>>,

    /// Remove/forget a device by address
    #[arg(long, value_name = "ADDRESS")]
    pub remove: Option<String>,
}

pub async fn run(client: CrawlClient, args: BtArgs, json: bool) -> Result<()> {
    if let Some(addr) = &args.connect {
        let res = client.cmd("BtConnect", json!({ "address": addr })).await?;
        if json { output::print_value(&res, true); } else { output::print_ok(&format!("Connected to {addr}")); }
    } else if let Some(addr) = &args.disconnect {
        let res = client.cmd("BtDisconnect", json!({ "address": addr })).await?;
        if json { output::print_value(&res, true); } else { output::print_ok(&format!("Disconnected {addr}")); }
    } else if let Some(ref state) = args.power {
        let on = state == "on";
        let res = client.cmd("BtPower", json!({ "on": on })).await?;
        if json { output::print_value(&res, true); } else { output::print_ok(&format!("Adapter powered {state}")); }
    } else if args.scan {
        let res = client.cmd("BtScan", json!({})).await?;
        if json { output::print_value(&res, true); } else { output::print_ok("Scan started"); }
    } else if let Some(ref state) = args.discoverable {
        let on = state == "on";
        let res = client.cmd("BtDiscoverable", json!({ "on": on })).await?;
        if json { output::print_value(&res, true); } else { output::print_ok(&format!("Discoverable {state}")); }
    } else if let Some(ref state) = args.pairable {
        let on = state == "on";
        let res = client.cmd("BtPairable", json!({ "on": on })).await?;
        if json { output::print_value(&res, true); } else { output::print_ok(&format!("Pairable {state}")); }
    } else if let Some(ref vals) = args.alias {
        if vals.len() == 2 {
            let addr = &vals[0];
            let alias = &vals[1];
            let res = client.cmd("BtAlias", json!({ "address": addr, "alias": alias })).await?;
            if json { output::print_value(&res, true); } else { output::print_ok(&format!("Set {addr} alias to '{alias}'")); }
        } else {
            output::print_ok("Usage: --alias <ADDRESS> <ALIAS>");
        }
    } else if let Some(addr) = &args.pair {
        let res = client.cmd("BtPair", json!({ "address": addr })).await?;
        if json { output::print_value(&res, true); } else { output::print_ok(&format!("Pairing with {addr}")); }
    } else if let Some(ref vals) = args.trust {
        if vals.len() == 2 {
            let addr = &vals[0];
            let trusted = vals[1] == "on";
            let res = client.cmd("BtTrust", json!({ "address": addr, "trusted": trusted })).await?;
            if json { output::print_value(&res, true); } else { output::print_ok(&format!("{addr} trust: {}", if trusted { "enabled" } else { "disabled" })); }
        } else {
            output::print_ok("Usage: --trust <ADDRESS> <on|off>");
        }
    } else if let Some(addr) = &args.remove {
        let res = client.cmd("BtRemove", json!({ "address": addr })).await?;
        if json { output::print_value(&res, true); } else { output::print_ok(&format!("Removed {addr}")); }
    } else {
        let res = client.cmd("BtStatus", serde_json::json!({})).await?;
        if json {
            output::print_value(&res, true);
        } else {
            let powered    = res["powered"].as_bool().unwrap_or(false);
            let discovering = res["discovering"].as_bool().unwrap_or(false);
            println!("Bluetooth");
            output::print_table(&[
                ("powered",     powered.to_string()),
                ("discovering", discovering.to_string()),
            ]);
            if let Some(devices) = res["devices"].as_array() {
                if devices.is_empty() {
                    println!("  no devices");
                } else {
                    println!("  {:<20}  {:<24}  {:<8}  {:<8}  battery", "address", "name", "connected", "paired");
                    for d in devices {
                        let addr   = d["address"].as_str().unwrap_or("?");
                        let name   = d["name"].as_str().unwrap_or("(unknown)");
                        let conn   = d["connected"].as_bool().unwrap_or(false);
                        let paired = d["paired"].as_bool().unwrap_or(false);
                        let bat    = d["battery"].as_u64().map(|b| format!("{b}%")).unwrap_or_else(|| "—".into());
                        println!("  {addr:<20}  {name:<24}  {:<8}  {:<8}  {bat}",
                                 if conn   { "yes" } else { "no"  },
                                 if paired { "yes" } else { "no"  });
                    }
                }
            }
        }
    }
    Ok(())
}