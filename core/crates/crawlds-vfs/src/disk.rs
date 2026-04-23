//! Disk management module - UDisks2 block devices, mount/unmount/eject

use crate::error::VfsError;
use crate::types::BlockDevice;
use zbus::{proxy, Connection};

// ── D-Bus proxies ─────────────────────────────────────────────────────────────

#[proxy(
    interface = "org.freedesktop.UDisks2.Manager",
    default_service = "org.freedesktop.UDisks2",
    default_path = "/org/freedesktop/UDisks2/Manager"
)]
pub trait UDisks2Manager {
    fn get_block_devices(
        &self,
        options: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.UDisks2.Block",
    default_service = "org.freedesktop.UDisks2"
)]
pub trait UDisks2Block {
    #[zbus(property)]
    fn device(&self) -> zbus::Result<Vec<u8>>;

    #[zbus(property)]
    fn id_label(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn id_type(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn size(&self) -> zbus::Result<u64>;

    #[zbus(property)]
    fn drive(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.UDisks2.Filesystem",
    default_service = "org.freedesktop.UDisks2"
)]
pub trait UDisks2Filesystem {
    fn mount(
        &self,
        options: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<String>;

    fn unmount(
        &self,
        options: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<()>;

    #[zbus(property)]
    fn mount_points(&self) -> zbus::Result<Vec<Vec<u8>>>;
}

#[proxy(
    interface = "org.freedesktop.UDisks2.Drive",
    default_service = "org.freedesktop.UDisks2"
)]
pub trait UDisks2Drive {
    fn eject(
        &self,
        options: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<()>;

    #[zbus(property)]
    fn removable(&self) -> zbus::Result<bool>;
}

// ── Public API ───────────────────────────────────────────────────────────────

pub async fn list_devices() -> Result<Vec<BlockDevice>, VfsError> {
    let conn = Connection::system().await?;
    let manager = UDisks2ManagerProxy::new(&conn).await?;
    let paths = manager.get_block_devices(Default::default()).await?;
    let mut devices = Vec::new();
    for path in &paths {
        let dev_result = build_block_device(&conn, path.as_str()).await;
        if let Ok(dev) = dev_result {
            if dev.size_bytes > 1_048_576 && !dev.device.ends_with("loop") {
                devices.push(dev);
            }
        }
    }
    Ok(devices)
}

pub async fn mount(device_path: &str) -> Result<String, VfsError> {
    let conn = Connection::system().await?;
    let block_path = resolve_block_path(&conn, device_path).await?;
    let fs = UDisks2FilesystemProxy::builder(&conn).path(block_path.as_str())?.build().await?;
    fs.mount(Default::default())
        .await
        .map_err(|e| VfsError::MountFailed(e.to_string()))
}

pub async fn unmount(device_path: &str) -> Result<(), VfsError> {
    let conn = Connection::system().await?;
    let block_path = resolve_block_path(&conn, device_path).await?;
    let fs = UDisks2FilesystemProxy::builder(&conn).path(block_path.as_str())?.build().await?;
    fs.unmount(Default::default())
        .await
        .map_err(|e| VfsError::UnmountFailed(e.to_string()))
}

pub async fn eject(drive_path: &str) -> Result<(), VfsError> {
    let conn = Connection::system().await?;
    let block_path = resolve_block_path(&conn, drive_path).await?;
    let block = UDisks2BlockProxy::builder(&conn).path(block_path.as_str())?.build().await?;
    let drive_path = block.drive().await?;
    let drive = UDisks2DriveProxy::builder(&conn).path(drive_path.as_str())?.build().await?;
    drive.eject(Default::default()).await?;
    Ok(())
}

// ── Internal helpers ─────────────────────────────────────────────────────────

pub async fn build_block_device(conn: &Connection, path: &str) -> Result<BlockDevice, VfsError> {
    let block = UDisks2BlockProxy::builder(conn).path(path)?.build().await?;
    let fs = UDisks2FilesystemProxy::builder(conn).path(path)?.build().await;

    let device_bytes = block.device().await.unwrap_or_default();
    let device = String::from_utf8_lossy(&device_bytes)
        .trim_end_matches('\0').to_string();

    let mount_points: Vec<Vec<u8>> = match fs {
        Ok(ref fs_proxy) => fs_proxy.mount_points().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    let mount_point = mount_points.first()
        .map(|mp| String::from_utf8_lossy(mp).trim_end_matches('\0').to_string());

    let label = block.id_label().await.unwrap_or_default();
    let size_bytes = block.size().await.unwrap_or(0);
    let fs_type = block.id_type().await.unwrap_or_default();

    let removable = match block.drive().await {
        Ok(drive_path) => {
            let drive = UDisks2DriveProxy::builder(conn).path(drive_path.as_str())?.build().await;
            match drive {
                Ok(proxy) => proxy.removable().await.unwrap_or(false),
                Err(_) => false,
            }
        }
        Err(_) => false,
    };

    Ok(BlockDevice {
        device,
        label: if label.is_empty() { None } else { Some(label) },
        size_bytes,
        filesystem: if fs_type.is_empty() { None } else { Some(fs_type) },
        mount_point,
        mounted: !mount_points.is_empty(),
        removable,
    })
}

async fn resolve_block_path(conn: &Connection, device: &str) -> Result<String, VfsError> {
    if device.starts_with("/org/freedesktop/UDisks2/") {
        return Ok(device.to_string());
    }

    let manager = UDisks2ManagerProxy::new(conn).await?;
    let paths = manager.get_block_devices(Default::default()).await?;
    for path in paths {
        let block = UDisks2BlockProxy::builder(conn).path(path.as_str())?.build().await?;
        let device_bytes = block.device().await.unwrap_or_default();
        let dev = String::from_utf8_lossy(&device_bytes)
            .trim_end_matches('\0')
            .to_string();
        if dev == device {
            return Ok(path.to_string());
        }
    }

    Err(VfsError::NotFound(device.to_string()))
}