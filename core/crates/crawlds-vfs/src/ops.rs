//! File operations module - copy, move, delete, rename, trash with progress

use crate::error::VfsError;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Progress {
    pub operation_id: String,
    pub kind: ProgressKind,
    pub current_file: String,
    pub processed_bytes: u64,
    pub total_bytes: u64,
    pub files_processed: u32,
    pub total_files: u32,
    pub percent: f64,
    pub status: ProgressStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProgressKind {
    Copy,
    Move,
    Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProgressStatus {
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

pub type ProgressSender = mpsc::Sender<Progress>;

pub async fn copy_with_progress(
    src: &Path,
    dst: &Path,
    operation_id: String,
    tx: Option<ProgressSender>,
) -> Result<u64, VfsError> {
    let kind = if src.is_dir() { ProgressKind::Copy } else { ProgressKind::Copy };
    let total_bytes = calculate_total_size(src)?;
    let total_files = count_files(src)?;

    if src.is_dir() {
        copy_dir_with_progress(src, dst, operation_id, tx, kind, total_bytes, total_files).await
    } else {
        copy_file_with_progress(src, dst, operation_id, tx, kind, total_bytes, 1).await
    }
}

fn calculate_total_size(path: &Path) -> Result<u64, VfsError> {
    let mut total = 0u64;
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    Ok(total)
}

fn count_files(path: &Path) -> Result<u32, VfsError> {
    let mut count = 0u32;
    if path.is_file() {
        return Ok(1);
    }
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

async fn copy_file_with_progress(
    src: &Path,
    dst: &Path,
    operation_id: String,
    tx: Option<ProgressSender>,
    kind: ProgressKind,
    total_bytes: u64,
    total_files: u32,
) -> Result<u64, VfsError> {
    tokio::fs::create_dir_all(dst.parent().unwrap())
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    let mut src_file = tokio::fs::File::open(src)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    let mut dst_file = tokio::fs::File::create(dst)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    let dst_path = dst.to_string_lossy().to_string();

    let mut total_copied = 0u64;
    let mut buffer = vec![0u8; 65536];

    loop {
        let n = src_file.read(&mut buffer).await.map_err(|e| VfsError::OperationFailed(e.to_string()))?;
        if n == 0 {
            break;
        }
        dst_file.write_all(&buffer[..n]).await.map_err(|e| VfsError::OperationFailed(e.to_string()))?;
        total_copied += n as u64;

        if let Some(sender) = &tx {
            let percent = if total_bytes > 0 { (total_copied as f64 / total_bytes as f64) * 100.0 } else { 0.0 };
            let progress = Progress {
                operation_id: operation_id.clone(),
                kind: kind.clone(),
                current_file: dst_path.clone(),
                processed_bytes: total_copied,
                total_bytes,
                files_processed: 1,
                total_files,
                percent: (percent * 100.0).round() / 100.0,
                status: ProgressStatus::InProgress,
            };
            let _ = sender.send(progress).await;
        }
    }

    dst_file.flush().await.map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    if let Some(sender) = &tx {
        let progress = Progress {
            operation_id,
            kind,
            current_file: dst_path,
            processed_bytes: total_copied,
            total_bytes,
            files_processed: 1,
            total_files,
            percent: 100.0,
            status: ProgressStatus::Completed,
        };
        let _ = sender.send(progress).await;
    }

    Ok(total_copied)
}

async fn copy_dir_with_progress(
    src: &Path,
    dst: &Path,
    operation_id: String,
    tx: Option<ProgressSender>,
    kind: ProgressKind,
    total_bytes: u64,
    total_files: u32,
) -> Result<u64, VfsError> {
    let dst = dst.join(src.file_name().unwrap_or_default());
    tokio::fs::create_dir_all(&dst)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    let mut total_copied = 0u64;
    let mut files_copied = 0u32;

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(src).unwrap();
        let dst_path = dst.join(rel_path);

        if src_path.is_dir() {
            tokio::fs::create_dir_all(&dst_path)
                .await
                .map_err(|e| VfsError::OperationFailed(e.to_string()))?;
        } else {
            let bytes = copy_file_with_progress(
                src_path,
                &dst_path,
                operation_id.clone(),
                tx.clone(),
                kind.clone(),
                total_bytes,
                total_files,
            ).await?;
            total_copied += bytes;
            files_copied += 1;

            if let Some(sender) = &tx {
                let percent = if total_bytes > 0 { (total_copied as f64 / total_bytes as f64) * 100.0 } else { 0.0 };
                let progress = Progress {
                    operation_id: operation_id.clone(),
                    kind: kind.clone(),
                    current_file: dst_path.to_string_lossy().to_string(),
                    processed_bytes: total_copied,
                    total_bytes,
                    files_processed: files_copied,
                    total_files,
                    percent: (percent * 100.0).round() / 100.0,
                    status: ProgressStatus::InProgress,
                };
                let _ = sender.send(progress).await;
            }
        }
    }

    if let Some(sender) = &tx {
        let progress = Progress {
            operation_id,
            kind,
            current_file: dst.to_string_lossy().to_string(),
            processed_bytes: total_copied,
            total_bytes,
            files_processed: files_copied,
            total_files,
            percent: 100.0,
            status: ProgressStatus::Completed,
        };
        let _ = sender.send(progress).await;
    }

    Ok(total_copied)
}

pub async fn copy(src: &Path, dst: &Path) -> Result<u64, VfsError> {
    if src.is_dir() {
        let dst = dst.join(src.file_name().unwrap_or_default());
        copy_dir(src, &dst).await
    } else {
        let dst = dst.join(src.file_name().unwrap_or_default());
        copy_file(src, &dst).await
    }
}

pub async fn copy_file(src: &Path, dst: &Path) -> Result<u64, VfsError> {
    tokio::fs::create_dir_all(dst.parent().unwrap())
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    let mut src_file = tokio::fs::File::open(src)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    let mut dst_file = tokio::fs::File::create(dst)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    let bytes_copied = tokio::io::copy(&mut src_file, &mut dst_file)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    Ok(bytes_copied)
}

pub async fn copy_dir(src: &Path, dst: &Path) -> Result<u64, VfsError> {
    let mut total_bytes = 0u64;

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(src).unwrap();
        let dst_path = dst.join(rel_path);

        if src_path.is_dir() {
            tokio::fs::create_dir_all(&dst_path)
                .await
                .map_err(|e| VfsError::OperationFailed(e.to_string()))?;
        } else {
            total_bytes += copy_file(src_path, &dst_path).await?;
        }
    }

    Ok(total_bytes)
}

pub async fn move_file(src: &Path, dst: &Path) -> Result<(), VfsError> {
    if dst.exists() {
        return Err(VfsError::OperationFailed(format!(
            "destination already exists: {}",
            dst.display()
        )));
    }

    tokio::fs::create_dir_all(dst.parent().unwrap())
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    tokio::fs::rename(src, dst)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))
}

pub async fn move_entry(src: &Path, dst: &Path) -> Result<(), VfsError> {
    if src.is_dir() {
        let dst = dst.join(src.file_name().unwrap_or_default());
        tokio::fs::rename(src, &dst)
            .await
            .map_err(|e| VfsError::OperationFailed(e.to_string()))
    } else {
        move_file(src, dst).await
    }
}

pub async fn delete(path: &Path) -> Result<(), VfsError> {
    if path.is_dir() {
        tokio::fs::remove_dir_all(path)
            .await
            .map_err(|e| VfsError::OperationFailed(e.to_string()))
    } else {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| VfsError::OperationFailed(e.to_string()))
    }
}

pub async fn rename(path: &Path, new_name: &str) -> Result<PathBuf, VfsError> {
    let parent = path
        .parent()
        .ok_or_else(|| VfsError::OperationFailed("no parent directory".to_string()))?;

    let new_path = parent.join(new_name);

    if new_path.exists() {
        return Err(VfsError::OperationFailed(format!(
            "name already exists: {}",
            new_path.display()
        )));
    }

    tokio::fs::rename(path, &new_path)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))?;

    Ok(new_path)
}

pub async fn trash_paths(paths: Vec<PathBuf>) -> Result<(), VfsError> {
    for path in paths {
        trash::delete(&path).map_err(|e| VfsError::TrashFailed(e.to_string()))?;
    }
    Ok(())
}

pub fn list_trash() -> Result<Vec<PathBuf>, VfsError> {
    let mut files = Vec::new();
    
    if let Ok(home) = std::env::var("HOME") {
        let trash_home = PathBuf::from(&home).join(".local/share/Trash");
        if trash_home.exists() {
            let files_dir = trash_home.join("files");
            if let Ok(entries) = std::fs::read_dir(&files_dir) {
                for entry in entries.flatten() {
                    files.push(entry.path());
                }
            }
        }
    }
    
    Ok(files)
}

pub async fn create_directory(path: &Path) -> Result<(), VfsError> {
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))
}

pub async fn create_file(path: &Path) -> Result<(), VfsError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| VfsError::OperationFailed(e.to_string()))?;
    }
    tokio::fs::write(path, "")
        .await
        .map_err(|e| VfsError::OperationFailed(e.to_string()))
}