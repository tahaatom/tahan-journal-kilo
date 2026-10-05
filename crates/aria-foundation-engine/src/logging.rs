//! لاگ‌گذاری کرنل آریا.
//!
//! لاگ‌ها کاملاً محلی هستند: رویدادها هم به خروجی استاندارد و هم به
//! فایل‌های چرخشی روزانه در پوشه لاگ نوشته می‌شوند.
//!
//! قوانین لاگ:
//! - لاگ‌ها هیچ‌گاه نباید حاوی رازها (رمز عبور، کلیدها) باشند.
//! - لاگ‌ها نباید حاوی بار مالی کامل کاربر باشند؛ فقط شناسه‌ها و خلاصه‌ها.
//! - سطح لاگ از طریق `RUST_LOG` قابل تنظیم است.

use std::fs;
use std::path::Path;

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::error::KernelError;

/// پیشوند نام فایل لاگ (تاریخ چرخش به آن اضافه می‌شود، مثلاً `tahan-journal.log.2026-10-05`).
pub const LOG_FILE_PREFIX: &str = "tahan-journal.log";

/// محافظ نویسنده غیربلوکه لاگ.
///
/// باید تا پایان عمر اپلیکیشن زنده نگه داشته شود تا رویدادهای لاگ
/// به‌صورت کامل در فایل نوشته شوند.
pub struct LogGuard {
    _guard: WorkerGuard,
}

/// لاگ‌گذاری کرنل را می‌آماده می‌کند:
/// فیلتر سطح، خروجی استاندارد و فایل چرخشی روزانه در `log_dir`.
///
/// محافظ برگردانده‌شده را باید زنده نگه داشت.
pub fn init_logging(log_dir: &Path, level: Level) -> Result<LogGuard, KernelError> {
    fs::create_dir_all(log_dir).map_err(|source| KernelError::DirectoryCreationFailed {
        context: Some(serde_json::json!({ "path": log_dir.display().to_string() })),
        source: Some(Box::new(source)),
    })?;

    let file_appender = tracing_appender::rolling::daily(log_dir, LOG_FILE_PREFIX);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();

    let stdout_layer = layer().with_filter(filter.clone());
    let file_layer = layer().with_writer(non_blocking).with_filter(filter);

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer);

    subscriber
        .try_init()
        .map_err(|source| KernelError::LogInitFailed {
            context: None,
            source: Some(Box::new(source)),
        })?;

    Ok(LogGuard { _guard: guard })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::TradeId;
    use uuid::Uuid;

    #[test]
    fn log_initialization_writes_structured_local_file() {
        let directory =
            std::env::temp_dir().join(format!("tahan-foundation-logging-{}", Uuid::new_v4()));
        let guard = init_logging(&directory, Level::INFO).unwrap();

        let trade_id = TradeId::new();
        tracing::info!(trade_id = %trade_id, "foundation log initialization test");
        drop(guard);

        let entries = fs::read_dir(&directory).unwrap();
        let log_files: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with(LOG_FILE_PREFIX))
                    .unwrap_or(false)
            })
            .collect();
        assert!(
            !log_files.is_empty(),
            "no log file created in {:?}",
            directory
        );

        let content = fs::read_to_string(&log_files[0]).unwrap();
        assert!(content.contains("foundation log initialization test"));
        assert!(content.contains(&trade_id.to_string()));

        let _ = fs::remove_dir_all(&directory);
    }
}
