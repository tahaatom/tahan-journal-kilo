//! پایه چیدمان بسته پشتیبان.
//!
//! این ماژول فقط **چیدمان** بسته پشتیبان را تعریف و ایجاد می‌کند؛
//! فشرده‌سازی (zstd)، رمزنگاری (AES-GCM)، تولید checksums و فرایند کامل
//! پشتیبان/بازیابی در فاز ۱.۱۶ پیاده می‌شوند.
//!
//! چیدمان بسته (`docs/database/backup-format.md`):
//!
//! ```text
//! <backup_dir>/<backup_id>/
//! ├── manifest.json        متادیتای بسته (format_version، زمان، اندازه‌ها)
//! ├── checksums.json       هش blake3 فایل‌ها (در فاز ۱.۱۶ پر می‌شود)
//! ├── payload/             جای‌نگهدار اسنپ‌شات رمزنگاری‌شده دیتابیس
//! ├── attachments/         جای‌نگهدار پوشه پیوست‌ها
//! └── settings.json        اسنپ‌شات اختیاری تنظیمات
//! ```

use std::path::{Path, PathBuf};

use aria_contracts::versioning::BACKUP_FORMAT_VERSION;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};
use crate::timestamps;

/// نام فایل مانیفست بسته.
pub const BACKUP_MANIFEST_FILE: &str = "manifest.json";
/// نام فایل checksums بسته.
pub const BACKUP_CHECKSUMS_FILE: &str = "checksums.json";
/// نام پوشه جای‌نگهدار اسنپ‌شات رمزنگاری‌شده دیتابیس.
pub const BACKUP_PAYLOAD_DIR: &str = "payload";
/// نام پوشه جای‌نگهدار پیوست‌ها.
pub const BACKUP_ATTACHMENTS_DIR: &str = "attachments";
/// نام فایل اختیاری اسنپ‌شات تنظیمات.
pub const BACKUP_SETTINGS_FILE: &str = "settings.json";

/// مانیفست بسته پشتیبان (نسخه ۱).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupManifest {
    /// شناسه بسته.
    pub backup_id: Uuid,
    /// نسخه فرمت بسته (قرارداد نسخه‌بندی).
    pub format_version: u32,
    /// زمان ایجاد (UTC).
    pub created_at: DateTime<Utc>,
    /// نسخه اسکیمای دیتابیس در زمان پشتیبان.
    pub schema_version: u32,
    /// تعداد پیوست‌ها (در فاز ۱.۳ صفر).
    pub attachment_count: u64,
    /// اندازه اسنپ‌شات رمزنگاری‌شده دیتابیس به بایت (در فاز ۱.۳ صفر).
    pub encrypted_database_size_bytes: u64,
}

/// یک بسته پشتیبان ایجادشده (چیدمان).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupPackage {
    /// شناسه بسته.
    pub id: Uuid,
    /// مسیر ریشه بسته.
    pub path: PathBuf,
    /// نسخه فرمت.
    pub format_version: u32,
    /// زمان ایجاد.
    pub created_at: DateTime<Utc>,
}

impl BackupPackage {
    /// مسیر فایل مانیفست.
    pub fn manifest_path(&self) -> PathBuf {
        self.path.join(BACKUP_MANIFEST_FILE)
    }

    /// مسیر فایل checksums.
    pub fn checksums_path(&self) -> PathBuf {
        self.path.join(BACKUP_CHECKSUMS_FILE)
    }

    /// مسیر پوشه اسنپ‌شات رمزنگاری‌شده.
    pub fn payload_dir(&self) -> PathBuf {
        self.path.join(BACKUP_PAYLOAD_DIR)
    }

    /// مسیر پوشه پیوست‌ها.
    pub fn attachments_dir(&self) -> PathBuf {
        self.path.join(BACKUP_ATTACHMENTS_DIR)
    }
}

/// چیدمان بسته پشتیبان را می‌سازد و مانیفست آن را می‌نویسد.
///
/// این تابع هیچ رمزنگاری یا فشرده‌سازی‌ای انجام نمی‌دهد؛ فقط ساختار پایه را
/// ایجاد می‌کند تا فاز ۱.۱۶ آن را کامل کند.
pub fn create_layout(backup_dir: &Path, schema_version: u32) -> StorageResult<BackupPackage> {
    let id = Uuid::new_v4();
    let created_at = timestamps::now();
    let path = backup_dir.join(id.to_string());

    let package = BackupPackage {
        id,
        path,
        format_version: BACKUP_FORMAT_VERSION,
        created_at,
    };

    create_directory(&package.path)?;
    create_directory(&package.payload_dir())?;
    create_directory(&package.attachments_dir())?;

    let manifest = BackupManifest {
        backup_id: id,
        format_version: BACKUP_FORMAT_VERSION,
        created_at,
        schema_version,
        attachment_count: 0,
        encrypted_database_size_bytes: 0,
    };

    let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|source| {
        StorageError::BackupLayoutFailed {
            context: Some(serde_json::json!({ "step": "serialize_manifest" })),
            source: Some(Box::new(source)),
        }
    })?;
    std::fs::write(package.manifest_path(), manifest_json).map_err(|source| {
        StorageError::BackupLayoutFailed {
            context: Some(serde_json::json!({
                "step": "write_manifest",
                "path": package.manifest_path().display().to_string(),
            })),
            source: Some(Box::new(source)),
        }
    })?;

    // checksums در فاز ۱.۱۶ پر می‌شود؛ اینجا ساختار خالی و معتبر نوشته می‌شود.
    std::fs::write(package.checksums_path(), b"{\n  \"files\": {}\n}\n").map_err(|source| {
        StorageError::BackupLayoutFailed {
            context: Some(serde_json::json!({
                "step": "write_checksums",
                "path": package.checksums_path().display().to_string(),
            })),
            source: Some(Box::new(source)),
        }
    })?;

    Ok(package)
}

/// مانیفست یک بسته پشتیبان را می‌خواند.
pub fn read_manifest(package_dir: &Path) -> StorageResult<BackupManifest> {
    let manifest_path = package_dir.join(BACKUP_MANIFEST_FILE);
    let bytes =
        std::fs::read(&manifest_path).map_err(|source| StorageError::BackupLayoutFailed {
            context: Some(serde_json::json!({
                "step": "read_manifest",
                "path": manifest_path.display().to_string(),
            })),
            source: Some(Box::new(source)),
        })?;
    serde_json::from_slice(&bytes).map_err(|source| StorageError::BackupLayoutFailed {
        context: Some(serde_json::json!({ "step": "parse_manifest" })),
        source: Some(Box::new(source)),
    })
}

fn create_directory(path: &Path) -> StorageResult<()> {
    std::fs::create_dir_all(path).map_err(|source| StorageError::BackupLayoutFailed {
        context: Some(serde_json::json!({
            "step": "create_directory",
            "path": path.display().to_string(),
        })),
        source: Some(Box::new(source)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_constants_match_backup_format() {
        assert_eq!(BACKUP_MANIFEST_FILE, "manifest.json");
        assert_eq!(BACKUP_CHECKSUMS_FILE, "checksums.json");
        assert_eq!(BACKUP_PAYLOAD_DIR, "payload");
        assert_eq!(BACKUP_ATTACHMENTS_DIR, "attachments");
        assert_eq!(BACKUP_SETTINGS_FILE, "settings.json");
    }
}
