//! crawlds-webservice: RSS feeds and Wallhaven API integration.
//!
//! Provides:
//! - RSS/Atom/JSON Feed polling with feed-rs
//! - Wallhaven wallpaper search and random fetching

pub mod config;
pub mod rss;
pub mod wallhaven;
pub mod http_client;

use crawlds_ipc::{events::CrawlEvent, types::{RssItem, Wallpaper}};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

pub use config::{Config, WallhavenConfig, RssConfig};
pub use rss::RssWorker;
pub use wallhaven::WallhavenWorker;

// ── Events ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WebserviceEvent {
    RssFeedUpdated { feed_url: String, items: Vec<RssItem> },
    RssFeedError { feed_url: String, message: String },
    RssFeedsRefreshed,
    WallhavenResults { walls: Vec<Wallpaper> },
    WallhavenError { message: String },
}

// ── Shared state ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct WebserviceState {
    pub feeds: Arc<Mutex<Vec<String>>>,
    pub wallhaven_api_key: Option<String>,
    pub wallhaven_config: WallhavenConfig,
    pub tx: broadcast::Sender<CrawlEvent>,
}

impl WebserviceState {
    pub fn new(tx: broadcast::Sender<CrawlEvent>, api_key: Option<String>) -> Self {
        Self {
            feeds: Arc::new(Mutex::new(Vec::new())),
            wallhaven_api_key: api_key,
            wallhaven_config: WallhavenConfig::default(),
            tx,
        }
    }

    pub fn new_with_config(
        tx: broadcast::Sender<CrawlEvent>,
        api_key: Option<String>,
        config: WallhavenConfig,
    ) -> Self {
        Self {
            feeds: Arc::new(Mutex::new(Vec::new())),
            wallhaven_api_key: api_key,
            wallhaven_config: config,
            tx,
        }
    }

    pub async fn add_feed(&self, url: &str) {
        let mut feeds = self.feeds.lock().await;
        if !feeds.contains(&url.to_string()) {
            feeds.push(url.to_string());
        }
    }

    pub async fn remove_feed(&self, url: &str) {
        let mut feeds = self.feeds.lock().await;
        feeds.retain(|f| f != url);
    }

    pub fn wallhaven_search(
        &self,
        query: Option<String>,
        tags: Vec<String>,
        page: u32,
    ) -> tokio::task::JoinHandle<anyhow::Result<Vec<Wallpaper>>> {
        let config = self.wallhaven_config.clone();
        let q = query.unwrap_or_default();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let worker = wallhaven::WallhavenWorker::new(config);
                let result = worker.search_blocking(&q, tags, None, None, page).await;
                result
            })
        })
    }

    pub fn wallhaven_random(
        &self,
        count: usize,
    ) -> tokio::task::JoinHandle<anyhow::Result<Vec<Wallpaper>>> {
        let config = self.wallhaven_config.clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let worker = wallhaven::WallhavenWorker::new(config);
                let result = worker.random_blocking(count).await;
                result
            })
        })
    }
}

// ── Run all webservice domains ────────────────────────────────────────────────

pub async fn run_with_state(
    cfg: Config,
    _tx: broadcast::Sender<CrawlEvent>,
    state: WebserviceState,
) -> anyhow::Result<()> {
    info!("crawlds-webservice starting with shared state");

    // Add configured feeds
    {
        let mut feeds = state.feeds.lock().await;
        for feed_url in &cfg.rss.feeds {
            if !feeds.contains(feed_url) {
                feeds.push(feed_url.clone());
            }
        }
    }

    // Spawn RSS worker
    if cfg.rss.enabled {
        let rss_cfg = cfg.rss.clone();
        let rss_state = state.clone();
        tokio::spawn(async move {
            let worker = RssWorker::new(rss_cfg, rss_state);
            if let Err(e) = worker.run().await {
                warn!(domain = "rss", error = %e, "RSS worker failed");
            }
        });
    }

    // Keep running until Ctrl+C
    tokio::signal::ctrl_c().await?;
    Ok(())
}

pub async fn run(cfg: Config, tx: broadcast::Sender<CrawlEvent>) -> anyhow::Result<()> {
    info!("crawlds-webservice starting");

    let state = WebserviceState::new_with_config(
        tx.clone(),
        cfg.wallhaven.api_key.clone(),
        cfg.wallhaven.clone(),
    );

    run_with_state(cfg, tx, state).await
}
