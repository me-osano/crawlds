//! Wallhaven API client

use crate::config::WallhavenConfig;
use crate::http_client::HttpClient;
use crawlds_ipc::types::Wallpaper;
use serde::Deserialize;
use tracing::debug;

const API_BASE: &str = "https://wallhaven.cc/api/v1";

#[derive(Debug, Deserialize)]
struct WallhavenResponse {
    data: Vec<WallhavenWall>,
    meta: Option<WallhavenMeta>,
}

#[derive(Debug, Deserialize)]
struct WallhavenMeta {
    current_page: u32,
    last_page: u32,
    #[serde(rename = "per_page")]
    per_page: u32,
    total: u32,
}

#[derive(Debug, Deserialize)]
struct WallhavenWall {
    id: String,
    url: String,
    short_url: String,
    favorites: String,
    source: String,
    purity: String,
    category: String,
    dimension_x: String,
    dimension_y: String,
    resolution: String,
    ratio: String,
    file_size: u64,
    thumbs: WallhavenThumbs,
    tags: Vec<WallhavenTag>,
    #[serde(rename = "uploaded_at")]
    uploaded_at: String,
}

#[derive(Debug, Deserialize)]
struct WallhavenThumbs {
    tiny: String,
    small: String,
    original: String,
    large: String,
}

#[derive(Debug, Deserialize)]
struct WallhavenTag {
    id: u32,
    name: String,
    alias: Option<String>,
    category_id: u32,
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: Option<String>,
    categories: Option<String>,
    purity: Option<String>,
    sorting: Option<String>,
    order: Option<String>,
    page: Option<u32>,
    toplist: Option<String>,
    apikey: Option<String>,
}

pub struct WallhavenWorker {
    config: WallhavenConfig,
    client: HttpClient,
}

impl WallhavenWorker {
    pub fn new(config: WallhavenConfig) -> Self {
        let client = HttpClient::new("crawlds/0.1", 15);
        Self { config, client }
    }

    pub fn with_client(config: WallhavenConfig, client: HttpClient) -> Self {
        Self { config, client }
    }

    pub async fn search(
        &self,
        query: Option<String>,
        tags: Vec<String>,
        categories: Option<String>,
        purity: Option<Vec<String>>,
        page: u32,
    ) -> anyhow::Result<Vec<Wallpaper>> {
        let mut search_query = query.unwrap_or_default();
        for tag in &tags {
            if !search_query.is_empty() {
                search_query.push(' ');
            }
            search_query.push_str(tag);
        }

        let purity_str = purity
            .unwrap_or_else(|| self.config.default_purity.clone())
            .join(",");
        let categories = categories.unwrap_or_else(|| self.config.default_categories.clone());

        let params = SearchParams {
            q: if search_query.is_empty() {
                None
            } else {
                Some(search_query)
            },
            categories: Some(categories),
            purity: Some(purity_str),
            sorting: Some("random".to_string()),
            order: None,
            page: Some(page),
            toplist: None,
            apikey: self.config.api_key.clone(),
        };

        let url = build_search_url(&params);
        debug!(url = %url, "Wallhaven search");

        self.fetch_wallpapers(&url).await
    }

    pub async fn search_blocking(
        &self,
        query: &str,
        tags: Vec<String>,
        categories: Option<String>,
        purity: Option<Vec<String>>,
        page: u32,
    ) -> anyhow::Result<Vec<Wallpaper>> {
        let query = if query.is_empty() {
            None
        } else {
            Some(query.to_string())
        };
        self.search(query, tags, categories, purity, page).await
    }

    pub async fn random(&self, count: usize) -> anyhow::Result<Vec<Wallpaper>> {
        let purity_str = self.config.default_purity.join(",");
        let params = SearchParams {
            q: None,
            categories: Some(self.config.default_categories.clone()),
            purity: Some(purity_str),
            sorting: Some("random".to_string()),
            order: None,
            page: Some(1),
            toplist: None,
            apikey: self.config.api_key.clone(),
        };

        let url = build_search_url(&params);
        let mut walls = self.fetch_wallpapers(&url).await?;
        walls.truncate(count);
        Ok(walls)
    }

    pub async fn random_blocking(&self, count: usize) -> anyhow::Result<Vec<Wallpaper>> {
        self.random(count).await
    }

    async fn fetch_wallpapers(&self, url: &str) -> anyhow::Result<Vec<Wallpaper>> {
        let body = self.client.get_uncached(url).await?;

        let response: WallhavenResponse = serde_json::from_str(&body)?;

        let walls: Vec<Wallpaper> = response
            .data
            .into_iter()
            .map(|w| Wallpaper {
                id: w.id,
                url: w.url,
                thumb_url: w.thumbs.large,
                resolution: w.resolution,
                purity: w.purity,
                tags: w.tags.into_iter().map(|t| t.name).collect(),
                uploaded_at: w.uploaded_at,
                file_size: w.file_size,
            })
            .collect();

        Ok(walls)
    }
}

fn build_search_url(params: &SearchParams) -> String {
    let mut url = format!("{}/search?", API_BASE);

    if let Some(ref q) = params.q {
        url.push_str(&format!("q={}&", urlencoding::encode(q)));
    }
    if let Some(ref cats) = params.categories {
        url.push_str(&format!("categories={}&", cats));
    }
    if let Some(ref purity) = params.purity {
        url.push_str(&format!("purity={}&", purity));
    }
    if let Some(ref sorting) = params.sorting {
        url.push_str(&format!("sorting={}&", sorting));
    }
    if let Some(ref order) = params.order {
        url.push_str(&format!("order={}&", order));
    }
    if let Some(page) = params.page {
        url.push_str(&format!("page={}&", page));
    }
    if let Some(ref apikey) = params.apikey {
        url.push_str(&format!("apikey={}", apikey));
    }

    url
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                _ => {
                    for b in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", b));
                    }
                }
            }
        }
        result
    }
}
