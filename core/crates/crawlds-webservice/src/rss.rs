//! RSS feed fetching and parsing

use crate::{config::RssConfig, http_client::HttpClient, WebserviceState};
use crawlds_ipc::{events::WebserviceEvent, types::RssItem};
use feed_rs::parser;
use std::time::Duration;
use thiserror::Error;
use tokio::time;
use tracing::{debug, info, warn};

#[derive(Debug, Error)]
pub enum RssError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Feed error: {0}")]
    Feed(String),
    #[error("Not modified")]
    NotModified,
}

pub struct RssWorker {
    config: RssConfig,
    state: WebserviceState,
    client: HttpClient,
}

impl RssWorker {
    pub fn new(config: RssConfig, state: WebserviceState) -> Self {
        let client = HttpClient::new(&config.user_agent, config.timeout_secs);
        Self { config, state, client }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!(
            feeds = %self.config.feeds.len(),
            interval_secs = %self.config.poll_interval_secs,
            "RSS worker starting"
        );

        let mut ticker = time::interval(Duration::from_secs(self.config.poll_interval_secs));

        self.poll_all().await;

        loop {
            ticker.tick().await;
            self.poll_all().await;
        }
    }

    async fn poll_all(&self) {
        let feeds = self.state.feeds.lock().await;
        let feed_urls = feeds.clone();
        drop(feeds);

        for url in feed_urls {
            match self.fetch_feed(&url).await {
                Ok(items) => {
                    debug!(url = %url, count = items.len(), "Fetched RSS feed");
                    let _ = self.state.tx.send(crawlds_ipc::CrawlEvent::Webservice(
                        WebserviceEvent::RssFeedUpdated {
                            feed_url: url,
                            items,
                        },
                    ));
                }
                Err(RssError::NotModified) => {
                    debug!(url = %url, "Feed not modified, skipping");
                }
                Err(e) => {
                    warn!(url = %url, error = %e, "Failed to fetch RSS feed");
                    let _ = self.state.tx.send(crawlds_ipc::CrawlEvent::Webservice(
                        WebserviceEvent::RssFeedError {
                            feed_url: url,
                            message: e.to_string(),
                        },
                    ));
                }
            }
        }

        let _ = self.state.tx.send(crawlds_ipc::CrawlEvent::Webservice(
            WebserviceEvent::RssFeedsRefreshed,
        ));
    }

    async fn fetch_feed(&self, url: &str) -> Result<Vec<RssItem>, RssError> {
        let body = match self.client.get(url).await {
            Ok(body) => body,
            Err(crate::http_client::HttpError::NotModified) => {
                return Err(RssError::NotModified);
            }
            Err(e) => return Err(RssError::Http(e.to_string())),
        };

        let feed = parser::parse(body.as_bytes()).map_err(|e| RssError::Parse(e.to_string()))?;

        let feed_title = feed
            .title
            .map(|t| t.content)
            .unwrap_or_default();

        let items: Vec<RssItem> = feed
            .entries
            .into_iter()
            .map(|e| {
                let title = e.title.map(|t| t.content).unwrap_or_default();
                let link = e
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .unwrap_or_default();
                let published = e.published.map(|d| d.to_rfc3339());
                let summary = e.summary.map(|s| s.content);

                // Try to extract thumbnail from media extensions
                let mut thumbnail: Option<String> = None;

                // Try content URLs first
                if thumbnail.is_none() {
                    if let Some(m) = e.media.iter().next() {
                        if let Some(c) = m.content.iter().next() {
                            if let Some(url) = &c.url {
                                thumbnail = Some(url.to_string());
                            }
                        }
                    }
                }

                // Try thumbnail URLs if no content URL found
                if thumbnail.is_none() {
                    if let Some(m) = e.media.iter().next() {
                        if let Some(t) = m.thumbnails.iter().next() {
                            thumbnail = Some(t.image.uri.clone());
                        }
                    }
                }

                RssItem {
                    feed_title: feed_title.clone(),
                    title,
                    link,
                    published,
                    summary,
                    thumbnail,
                }
            })
            .collect();

        Ok(items)
    }

    /// Manually add a feed URL
    pub async fn add_feed(&self, url: &str) {
        let mut feeds = self.state.feeds.lock().await;
        if !feeds.contains(&url.to_string()) {
            info!(url = %url, "Adding RSS feed");
            feeds.push(url.to_string());
        }
    }

    /// Manually remove a feed URL
    pub async fn remove_feed(&self, url: &str) {
        let mut feeds = self.state.feeds.lock().await;
        feeds.retain(|f| f != url);
    }

    /// Force refresh all feeds
    pub async fn refresh(&self) {
        self.poll_all().await;
    }
}
