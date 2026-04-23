//! Filesystem module - disk usage, file operations, directory listing

use crate::error::VfsError;
use crate::types::DiskUsage;
use crate::ops as ops_mod;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub use ops_mod::{copy_file, create_directory, create_file, list_trash as get_trash_list, trash_paths as move_to_trash};

// Re-export disk types
use crate::disk::{UDisks2ManagerProxy, UDisks2FilesystemProxy, UDisks2BlockProxy};

// ── Entry type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
    pub kind: EntryKind,
    pub size: u64,
    pub mtime: SystemTime,
    pub mime: String,
    pub is_hidden: bool,
    pub is_symlink: bool,
    pub permissions: u16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EntryKind {
    File,
    Directory,
    Symlink { target: PathBuf },
}

impl Entry {
    pub fn from_path(path: &Path) -> Result<Self, VfsError> {
        let metadata = std::fs::metadata(path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        let is_hidden = name.starts_with('.');

        let mime = infer::get_from_path(path)
            .ok()
            .flatten()
            .map(|m| m.mime_type().to_string())
            .unwrap_or_else(|| {
                if metadata.is_dir() {
                    "inode/directory".to_string()
                } else {
                    "application/octet-stream".to_string()
                }
            });

        let kind = if metadata.is_symlink() {
            let target = std::fs::read_link(path).unwrap_or_default();
            EntryKind::Symlink { target }
        } else if metadata.is_dir() {
            EntryKind::Directory
        } else {
            EntryKind::File
        };

        let permissions = metadata.permissions().readonly() as u16;

        Ok(Entry {
            path: path.to_path_buf(),
            name,
            extension,
            kind,
            size: metadata.len(),
            mtime: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            mime,
            is_hidden,
            is_symlink: metadata.is_symlink(),
            permissions,
        })
    }
}

// ── Disk Usage ───────────────────────────────────────────────────────────────

pub async fn get_disk_usage() -> Result<Vec<DiskUsage>, VfsError> {
    let conn = zbus::Connection::system().await?;
    let manager = UDisks2ManagerProxy::new(&conn).await?;
    let paths = manager.get_block_devices(Default::default()).await?;

    let mut usage_list = Vec::new();

    for path in &paths {
        let fs = match UDisks2FilesystemProxy::builder(&conn)
            .path(path.as_str())?
            .build()
            .await
        {
            Ok(f) => f,
            Err(_) => continue,
        };

        let mount_points: Vec<Vec<u8>> = fs.mount_points().await.unwrap_or_default();

        for mp in mount_points {
            let mount_point = String::from_utf8_lossy(&mp).trim_end_matches('\0').to_string();
            if mount_point.is_empty() || mount_point == "/" 
                || mount_point.starts_with("/sys") || mount_point.starts_with("/proc") 
                || mount_point.starts_with("/dev")
            {
                continue;
            }

            let mount_path = Path::new(&mount_point);
            if let Ok(stat) = nix::sys::statvfs::statvfs(mount_path) {
                let total = stat.blocks() * stat.fragment_size();
                let available = stat.blocks_available() * stat.fragment_size();
                let used = total.saturating_sub(available);
                let percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };

                let block: Option<UDisks2BlockProxy<'_>> = UDisks2BlockProxy::builder(&conn)
                    .path(path.as_str())?
                    .build()
                    .await
                    .ok();
                let fs_type = match block {
                    Some(b) => b.id_type().await.unwrap_or_default(),
                    None => String::new(),
                };

                usage_list.push(DiskUsage {
                    mount_point,
                    filesystem: if fs_type.is_empty() { None } else { Some(fs_type) },
                    total_bytes: total,
                    used_bytes: used,
                    available_bytes: available,
                    percent_used: (percent * 100.0).round() / 100.0,
                });
            }
        }
    }

    Ok(usage_list)
}

// ── File Operations ─────────────────────────────────────────────────────────

pub async fn list_dir(path: &str) -> Result<Vec<Entry>, VfsError> {
    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(path)?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        match Entry::from_path(&path) {
            Ok(e) => entries.push(e),
            Err(_) => continue,
        }
    }

    entries.sort_by(|a, b| {
        match (&a.kind, &b.kind) {
            (EntryKind::Directory, EntryKind::File) => std::cmp::Ordering::Less,
            (EntryKind::File, EntryKind::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

pub async fn file_info(path: &str) -> Result<Entry, VfsError> {
    Entry::from_path(Path::new(path))
}

pub async fn copy_files(src: &str, dst: &str) -> Result<u64, VfsError> {
    ops_mod::copy(Path::new(src), Path::new(dst)).await
}

pub async fn move_files(src: &str, dst: &str) -> Result<(), VfsError> {
    ops_mod::move_entry(Path::new(src), Path::new(dst)).await
}

pub async fn delete_file(path: &str) -> Result<(), VfsError> {
    ops_mod::delete(Path::new(path)).await
}

pub async fn rename_file(path: &str, new_name: &str) -> Result<String, VfsError> {
    let new_path = ops_mod::rename(Path::new(path), new_name).await?;
    Ok(new_path.to_string_lossy().to_string())
}

pub async fn trash_files(paths: Vec<String>) -> Result<(), VfsError> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    ops_mod::trash_paths(paths).await
}

// ── Path utilities ─────────────────────────────────────────────────────────

pub fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

pub fn config_dir() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|h| h.join(".config")))
}

pub fn cache_dir() -> Option<PathBuf> {
    std::env::var("XDG_CACHE_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|h| h.join(".cache")))
}