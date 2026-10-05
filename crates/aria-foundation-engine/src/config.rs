//! پیکربندی کرنل آریا.
//!
//! پیکربندی به‌صورت JSON در مسیر استاندارد داده‌های اپلیکیشن
//! (`data_dir/kernel.json`) ذخیره می‌شود. مسیرهای پیش‌فرض از
//! کریت `dirs` (مسیر استاندارد سیستم‌عامل) به‌دست می‌آیند.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::KernelError;

/// نام پوشه اپلیکیشن در مسیر داده‌های استاندارد سیستم.
pub const APP_DATA_DIR_NAME: &str = "Tahan Journal";

/// نام فایل پیکربندی کرنل.
pub const CONFIG_FILE_NAME: &str = "kernel.json";

/// نام پایه پایگاه داده.
pub const DATABASE_FILE_NAME: &str = "tahan.db";

/// مقدار پیش‌فرض زبان.
pub const DEFAULT_LOCALE: &str = "fa";

/// مقدار پیش‌فرض تم.
pub const DEFAULT_THEME: &str = "light";

/// مهای پیش‌فرض قفل خودکار به ثانیه (۱۵ دقیقه).
pub const DEFAULT_AUTO_LOCK_TIMEOUT_SECS: u64 = 900;

/// پیکربندی کرنل آریا.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelConfig {
    /// پوشه ریشه داده‌های اپلیکیشن.
    pub data_dir: PathBuf,
    /// مسیر فایل پایگاه داده.
    pub database_path: PathBuf,
    /// پوشه پیوست‌ها.
    pub attachments_dir: PathBuf,
    /// پوشه پشتیبان‌ها.
    pub backup_dir: PathBuf,
    /// پوشه لاگ‌ها.
    pub log_dir: PathBuf,
    /// پوشه پلاگین‌ها.
    pub plugin_dir: PathBuf,
    /// زبان رابط کاربری.
    pub locale: String,
    /// تم رابط کاربری.
    pub theme: String,
    /// مهای قفل خودکار به ثانیه.
    pub auto_lock_timeout_secs: u64,
}

impl KernelConfig {
    /// مسیر پیش‌فرض پوشه داده‌ها بر اساس مسیر استاندارد سیستم.
    pub fn default_data_dir() -> Result<PathBuf, KernelError> {
        dirs::data_dir()
            .map(|base| base.join(APP_DATA_DIR_NAME))
            .ok_or_else(|| KernelError::ConfigPathInvalid {
                context: Some(serde_json::json!({
                    "reason": "system data directory is unavailable"
                })),
                source: None,
            })
    }

    /// پیکربندی پیش‌فرض بر اساس مسیر استاندارد داده‌های سیستم.
    pub fn default_paths() -> Result<Self, KernelError> {
        Ok(Self::from_base(&Self::default_data_dir()?))
    }

    /// پیکربندی پیش‌فرض برای یک پوشه ریشه داده دلخواه.
    pub fn from_base(base: &Path) -> Self {
        let data_dir = base.to_path_buf();
        Self {
            database_path: data_dir.join(DATABASE_FILE_NAME),
            attachments_dir: data_dir.join("attachments"),
            backup_dir: data_dir.join("backup"),
            log_dir: data_dir.join("logs"),
            plugin_dir: data_dir.join("plugins"),
            data_dir,
            locale: DEFAULT_LOCALE.to_string(),
            theme: DEFAULT_THEME.to_string(),
            auto_lock_timeout_secs: DEFAULT_AUTO_LOCK_TIMEOUT_SECS,
        }
    }

    /// مسیر فایل پیکربندی.
    pub fn config_path(&self) -> PathBuf {
        self.data_dir.join(CONFIG_FILE_NAME)
    }

    /// پیکربندی را از مسیر پیش‌فرض بارگذاری می‌کند؛
    /// در صورت نبود فایل، پیکربندی پیش‌فرض ساخته، ذخیره و برمی‌گرداند.
    pub fn load() -> Result<Self, KernelError> {
        let base = Self::default_data_dir()?;
        Self::load_from_base(&base)
    }

    /// پیکربندی را نسبت به یک پوشه ریشه داده دلخواه بارگذاری می‌کند؛
    /// در صورت نبود فایل، پیکربندی پیش‌فرض ساخته، ذخیره و برمی‌گرداند.
    pub fn load_from_base(base: &Path) -> Result<Self, KernelError> {
        let config = Self::from_base(base);
        let path = config.config_path();
        if path.is_file() {
            Self::load_from(&path)
        } else {
            config.ensure_directories()?;
            config.save()?;
            Ok(config)
        }
    }

    /// پیکربندی را از یک مسیر فایل دلخواه بارگذاری می‌کند.
    pub fn load_from(path: &Path) -> Result<Self, KernelError> {
        let file = fs::File::open(path).map_err(|source| KernelError::ConfigLoadFailed {
            context: Some(serde_json::json!({ "path": path.display().to_string() })),
            source: Some(Box::new(source)),
        })?;
        serde_json::from_reader(file).map_err(|source| KernelError::ConfigInvalid {
            context: Some(serde_json::json!({ "path": path.display().to_string() })),
            source: Some(Box::new(source)),
        })
    }

    /// پیکربندی را در مسیر پیش‌فرض ذخیره می‌کند.
    pub fn save(&self) -> Result<(), KernelError> {
        self.save_to(&self.config_path())
    }

    /// پیکربندی را در یک مسیر فایل دلخواه ذخیره می‌کند.
    pub fn save_to(&self, path: &Path) -> Result<(), KernelError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|source| {
                    KernelError::DirectoryCreationFailed {
                        context: Some(serde_json::json!({ "path": parent.display().to_string() })),
                        source: Some(Box::new(source)),
                    }
                })?;
            }
        }
        let json =
            serde_json::to_vec_pretty(self).map_err(|source| KernelError::SerializationFailed {
                context: None,
                source: Some(Box::new(source)),
            })?;
        fs::write(path, json).map_err(|source| KernelError::ConfigSaveFailed {
            context: Some(serde_json::json!({ "path": path.display().to_string() })),
            source: Some(Box::new(source)),
        })
    }

    /// همه پوشه‌های مورد نیاز کرنل را می‌سازد.
    pub fn ensure_directories(&self) -> Result<(), KernelError> {
        let directories = [
            &self.data_dir,
            &self.attachments_dir,
            &self.backup_dir,
            &self.log_dir,
            &self.plugin_dir,
        ];
        for directory in directories {
            fs::create_dir_all(directory).map_err(|source| {
                KernelError::DirectoryCreationFailed {
                    context: Some(serde_json::json!({ "path": directory.display().to_string() })),
                    source: Some(Box::new(source)),
                }
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_dir(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "tahan-foundation-config-{}-{}",
            name,
            Uuid::new_v4()
        ));
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn default_paths_are_consistent() {
        let config = KernelConfig::default_paths().unwrap();
        assert!(config.database_path.starts_with(&config.data_dir));
        assert!(config.attachments_dir.starts_with(&config.data_dir));
        assert!(config.backup_dir.starts_with(&config.data_dir));
        assert!(config.log_dir.starts_with(&config.data_dir));
        assert!(config.plugin_dir.starts_with(&config.data_dir));
        assert_eq!(config.locale, DEFAULT_LOCALE);
        assert_eq!(config.theme, DEFAULT_THEME);
        assert_eq!(
            config.auto_lock_timeout_secs,
            DEFAULT_AUTO_LOCK_TIMEOUT_SECS
        );
        assert_eq!(config.config_path(), config.data_dir.join(CONFIG_FILE_NAME));
        assert_eq!(
            config.database_path.file_name().and_then(|n| n.to_str()),
            Some(DATABASE_FILE_NAME)
        );
    }

    #[test]
    fn from_base_builds_expected_layout() {
        let base = Path::new("/tmp/tahan-test-base");
        let config = KernelConfig::from_base(base);
        assert_eq!(config.data_dir, base);
        assert_eq!(config.database_path, base.join("tahan.db"));
        assert_eq!(config.attachments_dir, base.join("attachments"));
        assert_eq!(config.backup_dir, base.join("backup"));
        assert_eq!(config.log_dir, base.join("logs"));
        assert_eq!(config.plugin_dir, base.join("plugins"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let directory = temp_dir("roundtrip");
        let config = KernelConfig::from_base(&directory);
        let path = directory.join(CONFIG_FILE_NAME);
        config.save_to(&path).unwrap();
        let loaded = KernelConfig::load_from(&path).unwrap();
        assert_eq!(loaded, config);
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn load_from_base_creates_default_when_missing() {
        let directory = temp_dir("autocreate");
        let config = KernelConfig::load_from_base(&directory).unwrap();
        assert!(config.config_path().is_file());
        assert!(config.attachments_dir.is_dir());
        assert!(config.backup_dir.is_dir());
        assert!(config.log_dir.is_dir());
        assert!(config.plugin_dir.is_dir());
        let reloaded = KernelConfig::load_from_base(&directory).unwrap();
        assert_eq!(reloaded, config);
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn load_from_missing_file_fails_with_load_error() {
        let directory = temp_dir("missing");
        let _ = fs::create_dir_all(&directory);
        let path = directory.join("does-not-exist.json");
        let error = KernelConfig::load_from(&path).unwrap_err();
        assert_eq!(error.code(), 1001);
        assert_eq!(error.variant(), "ConfigLoadFailed");
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn load_from_invalid_json_fails_with_invalid_error() {
        let directory = temp_dir("invalid");
        let _ = fs::create_dir_all(&directory);
        let path = directory.join(CONFIG_FILE_NAME);
        fs::write(&path, "{ not valid json").unwrap();
        let error = KernelConfig::load_from(&path).unwrap_err();
        assert_eq!(error.code(), 1003);
        assert_eq!(error.variant(), "ConfigInvalid");
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn save_to_blocked_parent_fails_with_directory_error() {
        let directory = temp_dir("blocked");
        let _ = fs::create_dir_all(&directory);
        // یک فایل به‌عنوان والد مسیر: ساخت پوشه روی آن ناممکن است
        let blocker = directory.join("blocker");
        fs::write(&blocker, "occupied").unwrap();
        let path = blocker.join(CONFIG_FILE_NAME);
        let error = KernelConfig::from_base(&directory)
            .save_to(&path)
            .unwrap_err();
        assert_eq!(error.code(), 1010);
        assert_eq!(error.variant(), "DirectoryCreationFailed");
        let _ = fs::remove_dir_all(&directory);
    }
}
