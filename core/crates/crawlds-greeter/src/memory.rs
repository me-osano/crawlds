//! Session memory persistence
//!
//! Handles saving and loading of greeter session memory including:
//! - Last successful user
//! - Last selected session

use crate::config::Config;
use crate::types::SessionMemory;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("memory not found")]
    NotFound,
}

pub struct Memory {
    config: Config,
    memory: SessionMemory,
    dirty: bool,
}

impl Memory {
    pub async fn load(config: Config) -> Result<Self, MemoryError> {
        let path = config.memory_path();
        let memory = if Path::new(&path).exists() {
            let content = fs::read_to_string(&path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            SessionMemory::default()
        };

        Ok(Self {
            config,
            memory,
            dirty: false,
        })
    }

    pub fn load_sync(config: &Config) -> Result<Self, MemoryError> {
        let path = config.memory_path();
        let memory = if Path::new(&path).exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            SessionMemory::default()
        };

        Ok(Self {
            config: config.clone(),
            memory,
            dirty: false,
        })
    }

    pub async fn save(&mut self) -> Result<(), MemoryError> {
        if !self.dirty {
            return Ok(());
        }

        let parent = Path::new(&self.config.cache_dir);
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }

        let path = &self.config.memory_path();
        let content = serde_json::to_string_pretty(&self.memory)?;
        fs::write(path, content).await?;

        self.dirty = false;
        Ok(())
    }

    pub fn save_sync(&mut self) -> Result<(), MemoryError> {
        if !self.dirty {
            return Ok(());
        }

        let parent = Path::new(&self.config.cache_dir);
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }

        let path = &self.config.memory_path();
        let content = serde_json::to_string_pretty(&self.memory)?;
        std::fs::write(path, content)?;

        self.dirty = false;
        Ok(())
    }

    pub fn last_session_id(&self) -> Option<&str> {
        self.memory.last_session_id.as_deref()
    }

    pub fn last_successful_user(&self) -> Option<&str> {
        self.memory.last_successful_user.as_deref()
    }

    pub fn set_last_session_id(&mut self, id: Option<String>) {
        if self.memory.last_session_id != id {
            self.memory.last_session_id = id;
            self.dirty = true;
        }
    }

    pub fn set_last_successful_user(&mut self, user: Option<String>) {
        if self.memory.last_successful_user != user {
            self.memory.last_successful_user = user;
            self.dirty = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn memory(&self) -> &SessionMemory {
        &self.memory
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            config: Config::default(),
            memory: SessionMemory::default(),
            dirty: false,
        }
    }
}
