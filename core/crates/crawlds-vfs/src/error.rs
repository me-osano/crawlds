//! VFS Error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VfsError {
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),

    #[error("device not found: {0}")]
    NotFound(String),

    #[error("mount failed: {0}")]
    MountFailed(String),

    #[error("unmount failed: {0}")]
    UnmountFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("search failed: {0}")]
    SearchFailed(String),

    #[error("file watcher error: {0}")]
    WatchError(String),

    #[error("file operation failed: {0}")]
    OperationFailed(String),

    #[error("trash operation failed: {0}")]
    TrashFailed(String),

    #[error("index error: {0}")]
    IndexError(String),
}

impl From<notify::Error> for VfsError {
    fn from(e: notify::Error) -> Self {
        VfsError::WatchError(e.to_string())
    }
}
