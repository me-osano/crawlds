//! Greeter configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub greetd_socket: String,
    pub session_ttl_secs: u64,
    pub cache_dir: String,
    pub memory_file: String,
    pub session_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            greetd_socket: "/run/greetd.sock".into(),
            session_ttl_secs: 60,
            cache_dir: "/var/cache/crawlds-greeter".into(),
            memory_file: ".local/state/memory.json".into(),
            session_file: "session.json".into(),
        }
    }
}

impl Config {
    pub fn memory_path(&self) -> String {
        format!("{}/{}", self.cache_dir, self.memory_file)
    }

    pub fn session_path(&self) -> String {
        format!("{}/{}", self.cache_dir, self.session_file)
    }
}
