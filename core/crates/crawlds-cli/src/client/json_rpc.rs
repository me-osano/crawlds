use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst)
}

pub struct JsonRpcClient {
    socket_path: PathBuf,
}

impl JsonRpcClient {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    pub async fn cmd(&self, method: &str, params: Value) -> Result<Value> {
        let id = next_id();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        let stream = timeout(
            Duration::from_secs(5),
            UnixStream::connect(&self.socket_path)
        )
        .await
        .with_context(|| format!(
            "failed to connect to crawlds daemon at {:?}\n\
             Is crawlds-daemon running? Try: systemctl --user start crawlds",
            self.socket_path
        ))??;

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        let req_str = serde_json::to_string(&request)?;
        writer.write_all(req_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        let mut line = String::new();
        timeout(
            Duration::from_secs(5),
            reader.read_line(&mut line)
        )
        .await
        .with_context(|| "request timed out")??;

        let response: Value = serde_json::from_str(&line)
            .context("failed to parse JSON-RPC response")?;

        if let Some(error) = response.get("error") {
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            anyhow::bail!("daemon error: {}", message);
        }

        Ok(response)
    }
}