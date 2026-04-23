//! Webservice configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rss: RssConfig,
    pub wallhaven: WallhavenConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rss: RssConfig::default(),
            wallhaven: WallhavenConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssConfig {
    pub enabled: bool,
    pub feeds: Vec<String>,
    pub poll_interval_secs: u64,
    pub user_agent: String,
    pub timeout_secs: u64,
}

impl Default for RssConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            feeds: Vec::new(),
            poll_interval_secs: 300, // 5 minutes
            user_agent: "crawlds/0.1".to_string(),
            timeout_secs: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallhavenConfig {
    pub api_key: Option<String>,
    pub default_purity: Vec<String>,
    pub default_categories: String,
}

impl Default for WallhavenConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            default_purity: vec!["sfw".to_string()],
            default_categories: "111".to_string(), // anime, people, general
        }
    }
}
