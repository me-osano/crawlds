//! Preloader module - pre-load directory contents for faster navigation

use crate::error::VfsError;
use crate::fs::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Preloader caches directory contents and pre-loads adjacent directories
#[derive(Clone)]
pub struct Preloader {
    cache: Arc<RwLock<HashMap<PathBuf, Vec<Entry>>>>,
    max_cache_entries: usize,
    preload_ahead: usize,
}

impl Preloader {
    pub fn new(max_cache_entries: usize, preload_ahead: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_entries,
            preload_ahead,
        }
    }

    /// Preload a directory and optionally its siblings
    pub async fn preload(&self, path: &Path) -> Result<Vec<Entry>, VfsError> {
        let entries = crate::fs::list_dir(&path.to_string_lossy()).await?;
        
        let mut cache = self.cache.write().await;
        
        // Evict old entries if cache is full
        if cache.len() >= self.max_cache_entries {
            // Remove oldest entries (simple LRU approximation)
            let keys_to_remove = cache.len() - self.max_cache_entries + 1;
            let keys: Vec<PathBuf> = cache.keys().cloned().take(keys_to_remove).collect();
            for key in keys {
                cache.remove(&key);
            }
        }
        
        cache.insert(path.to_path_buf(), entries.clone());
        Ok(entries)
    }

    /// Get cached entries if available, otherwise load from disk
    pub async fn get(&self, path: &Path) -> Result<Option<Vec<Entry>>, VfsError> {
        let cache = self.cache.read().await;
        Ok(cache.get(path).cloned())
    }

    /// Preload multiple paths (for navigating siblings)
    pub async fn preload_multiple(&self, paths: Vec<PathBuf>) {
        for path in paths {
            let path_clone = path.clone();
            let cache = Arc::clone(&self.cache);
            tokio::spawn(async move {
                if let Ok(entries) = crate::fs::list_dir(&path_clone.to_string_lossy()).await {
                    let mut cache = cache.write().await;
                    cache.insert(path_clone, entries);
                }
            });
        }
    }

    /// Preload adjacent directories based on current path
    pub async fn preload_adjacent(&self, current: &Path) {
        let parent = current.parent();
        
        // Preload parent
        if let Some(parent_path) = parent {
            let parent = parent_path.to_path_buf();
            let cache = Arc::clone(&self.cache);
            tokio::spawn(async move {
                if let Ok(entries) = crate::fs::list_dir(&parent.to_string_lossy()).await {
                    let mut cache = cache.write().await;
                    cache.insert(parent, entries);
                }
            });
        }

        // Preload current directory's siblings (neighbors in parent)
        if let Some(parent_path) = parent {
            if let Ok(entries) = crate::fs::list_dir(&parent_path.to_string_lossy()).await {
                // Find current entry's position and preload neighbors
                let current_name = current.file_name().map(|n| n.to_string_lossy().to_string());
                if let Some(name) = current_name {
                    let neighbor_paths: Vec<PathBuf> = entries
                        .iter()
                        .filter(|e| e.name != name)
                        .take(self.preload_ahead)
                        .map(|e| parent_path.join(&e.name))
                        .collect();
                    
                    self.preload_multiple(neighbor_paths).await;
                }
            }
        }
    }

    /// Clear cache for a specific path
    pub async fn invalidate(&self, path: &Path) {
        let mut cache = self.cache.write().await;
        cache.remove(path);
    }

    /// Clear entire cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> PreloaderStats {
        let cache = self.cache.read().await;
        PreloaderStats {
            entries: cache.len(),
            max_entries: self.max_cache_entries,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PreloaderStats {
    pub entries: usize,
    pub max_entries: usize,
}

/// Preloading strategy for different use cases
#[derive(Debug, Clone)]
pub enum PreloadStrategy {
    /// Only preload current directory
    Current,
    /// Preload current + parent
    Parent,
    /// Preload current + parent + neighbors
    Adjacent,
    /// Preload entire directory tree (be careful!)
    Deep,
}

impl Default for PreloadStrategy {
    fn default() -> Self {
        PreloadStrategy::Adjacent
    }
}

/// Preloader manager that coordinates preloading based on navigation
#[derive(Clone)]
pub struct PreloadManager {
    preloader: Preloader,
    current_path: Arc<RwLock<Option<PathBuf>>>,
}

impl PreloadManager {
    pub fn new(max_cache_entries: usize, preload_ahead: usize) -> Self {
        Self {
            preloader: Preloader::new(max_cache_entries, preload_ahead),
            current_path: Arc::new(RwLock::new(None)),
        }
    }

    /// Navigate to a path, triggering preloads based on strategy
    pub async fn navigate(&self, path: &Path, strategy: PreloadStrategy) -> Result<Vec<Entry>, VfsError> {
        // Update current path
        {
            let mut current = self.current_path.write().await;
            *current = Some(path.to_path_buf());
        }

        // Try cache first
        if let Some(cached) = self.preloader.get(path).await? {
            // Trigger background preloads
            self.trigger_preloads(path, strategy).await;
            return Ok(cached);
        }

        // Load and cache
        let entries = self.preloader.preload(path).await?;
        
        // Trigger background preloads
        self.trigger_preloads(path, strategy).await;
        
        Ok(entries)
    }

    /// Trigger background preloading based on strategy
    async fn trigger_preloads(&self, path: &Path, strategy: PreloadStrategy) {
        match strategy {
            PreloadStrategy::Current => {}
            PreloadStrategy::Parent => {
                if let Some(parent) = path.parent() {
                    let parent = parent.to_path_buf();
                    let cache = Arc::clone(&self.preloader.cache);
                    tokio::spawn(async move {
                        if let Ok(entries) = crate::fs::list_dir(&parent.to_string_lossy()).await {
                            let mut cache = cache.write().await;
                            cache.insert(parent, entries);
                        }
                    });
                }
            }
            PreloadStrategy::Adjacent => {
                self.preloader.preload_adjacent(path).await;
            }
            PreloadStrategy::Deep => {
                // Be careful with this - could preload entire tree
                // Only recommended for small directory structures
            }
        }
    }

    /// Invalidate cache when filesystem changes detected
    pub async fn handle_fs_change(&self, path: &str) {
        let path = PathBuf::from(path);
        
        // Invalidate the changed path
        self.preloader.invalidate(&path).await;
        
        // Also invalidate parent to refresh listing
        if let Some(parent) = path.parent() {
            self.preloader.invalidate(parent).await;
        }

        // Update current path to trigger reload
        {
            let mut current = self.current_path.write().await;
            if let Some(ref current_path) = *current {
                if path.starts_with(current_path) {
                    // Current directory changed, mark for reload
                    *current = None;
                }
            }
        }
    }

    /// Get cache stats
    pub async fn stats(&self) -> PreloaderStats {
        self.preloader.stats().await
    }
}