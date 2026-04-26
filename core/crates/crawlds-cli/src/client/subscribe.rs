use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

pub struct EventSubscription {
    socket_path: PathBuf,
}

impl EventSubscription {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    pub async fn subscribe<F, T>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(T),
        T: DeserializeOwned,
    {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .with_context(|| format!("failed to connect to crawlds at {:?}", self.socket_path))?;

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        let subscribe = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "Subscribe",
            "params": {},
            "id": 0
        });
        let req_str = serde_json::to_string(&subscribe)?;
        writer.write_all(req_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let _initial: Value = serde_json::from_str(&line)
            .context("failed to parse subscription confirmation")?;

        line.clear();
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(&line) {
                Ok(value) => {
                    if let Some(params) = value.get("params") {
                        match serde_json::from_value(params.clone()) {
                            Ok(event) => handler(event),
                            Err(e) => {
                                eprintln!("WARN: failed to parse event: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WARN: failed to parse NDJSON line: {}", e);
                }
            }
        }

        Ok(())
    }
}