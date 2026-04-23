//! File system watcher using notify crate

use crate::error::VfsError;
use notify::event::Event;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::broadcast;

pub type WatcherEvent = Event;

pub struct FsWatcher {
    watcher: RecommendedWatcher,
    tx: broadcast::Sender<WatcherEvent>,
}

impl FsWatcher {
    pub fn new() -> Result<Self, VfsError> {
        let (tx, _rx) = broadcast::channel(100);
        let tx_clone = tx.clone();

        let watcher = RecommendedWatcher::new(
            move |res: Result<WatcherEvent, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx_clone.send(event);
                }
            },
            Config::default(),
        )?;

        Ok(Self { watcher, tx })
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), VfsError> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| VfsError::WatchError(e.to_string()))
    }

    pub fn unwatch(&mut self, path: &Path) -> Result<(), VfsError> {
        self.watcher
            .unwatch(path)
            .map_err(|e| VfsError::WatchError(e.to_string()))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WatcherEvent> {
        self.tx.subscribe()
    }

    pub fn tx(&self) -> broadcast::Sender<WatcherEvent> {
        self.tx.clone()
    }
}

pub fn watcher_to_fs_event(event: WatcherEvent) -> crawlds_ipc::types::FsEvent {
    use notify::event::EventKind;
    let paths: Vec<String> = event
        .paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    match event.kind {
        EventKind::Create(_) => {
            if let Some(path) = paths.first() {
                crawlds_ipc::types::FsEvent::Created { path: path.clone() }
            } else {
                crawlds_ipc::types::FsEvent::Modified {
                    path: String::new(),
                }
            }
        }
        EventKind::Modify(_) => {
            if let Some(path) = paths.first() {
                crawlds_ipc::types::FsEvent::Modified { path: path.clone() }
            } else {
                crawlds_ipc::types::FsEvent::Modified {
                    path: String::new(),
                }
            }
        }
        EventKind::Remove(_) => {
            if let Some(path) = paths.first() {
                crawlds_ipc::types::FsEvent::Deleted { path: path.clone() }
            } else {
                crawlds_ipc::types::FsEvent::Deleted {
                    path: String::new(),
                }
            }
        }
        _ => crawlds_ipc::types::FsEvent::Modified {
            path: String::new(),
        },
    }
}
