//! اتصال دیتابیس رمزنگاری‌شده (سازگار با SQLCipher).
//!
//! هر اتصال با ترتیب صحیح PRAGMAها باز می‌شود:
//!
//! 1. `PRAGMA key` (در صورت وجود کلید) — باید نخستین دستور روی اتصال باشد
//! 2. `PRAGMA journal_mode = WAL`
//! 3. `PRAGMA foreign_keys = ON`
//! 4. `PRAGMA busy_timeout = 5000`
//! 5. `PRAGMA synchronous = NORMAL`
//!
//! کلید رمزنگاری یک کلید خام ۲۵۶ بیتی است (خروجی KDF موتور امنیت، فاز ۱.۴)
//! و در قالب `x'<hex>'` به SQLCipher داده می‌شود.

use std::path::{Path, PathBuf};

use aria_contracts::services::SecretBytes;
use rusqlite::Connection;

use crate::error::{StorageError, StorageResult};

/// طول لازم کلید رمزنگاری دیتابیس (۲۵۶ بیت = ۳۲ بایت).
pub const DATABASE_KEY_LENGTH_BYTES: usize = 32;

/// مهای پیش‌فرض `busy_timeout` به میلی‌ثانیه.
pub const DEFAULT_BUSY_TIMEOUT_MS: u32 = 5000;

/// یک اتصال باز‌شده به دیتابیس به‌همراه مسیر آن.
pub struct DatabaseConnection {
    connection: Connection,
    path: PathBuf,
    encrypted: bool,
}

impl std::fmt::Debug for DatabaseConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConnection")
            .field("path", &self.path)
            .field("encrypted", &self.encrypted)
            .finish_non_exhaustive()
    }
}

impl DatabaseConnection {
    /// یک اتصال را به مسیر داده‌شده باز می‌کند و PRAGMAها را اعمال می‌کند.
    ///
    /// اگر `encryption_key` داده شود، باید دقیقاً
    /// [`DATABASE_KEY_LENGTH_BYTES`] بایت باشد؛ در غیر این صورت
    /// [`StorageError::EncryptionKeyInvalid`] بازگردانده می‌شود.
    pub fn open(path: &Path, encryption_key: Option<&SecretBytes>) -> StorageResult<Self> {
        ensure_parent_directory(path)?;

        let connection =
            Connection::open(path).map_err(|source| StorageError::DatabaseOpenFailed {
                context: Some(serde_json::json!({ "path": path.display().to_string() })),
                source: Some(Box::new(source)),
            })?;

        let encrypted = encryption_key.is_some();
        if let Some(key) = encryption_key {
            apply_encryption_key(&connection, key)?;
        }

        // اعتبارسنجی خوانا بودن پیش از PRAGMAها تا ناسازگاری کلید تمیز تشخیص داده شود.
        verify_readable(&connection, path, encrypted)?;
        apply_pragmas(&connection)?;

        Ok(Self {
            connection,
            path: path.to_path_buf(),
            encrypted,
        })
    }

    /// دسترسی به اتصال زیرین.
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// اجرای چند دستور SQL (میان‌بر برای راه‌اندازی و ابزار).
    pub fn execute_batch(&self, sql: &str) -> StorageResult<()> {
        self.connection
            .execute_batch(sql)
            .map_err(|source| StorageError::QueryFailed {
                context: Some(serde_json::json!({ "operation": "execute_batch" })),
                source: Some(Box::new(source)),
            })
    }

    /// مسیر فایل دیتابیس.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// نسخه SQLCipher (برای بررسی سلامت و تست‌ها).
    pub fn cipher_version(&self) -> StorageResult<String> {
        self.connection
            .pragma_query_value(None, "cipher_version", |row| row.get(0))
            .map_err(|source| StorageError::QueryFailed {
                context: Some(serde_json::json!({ "pragma": "cipher_version" })),
                source: Some(Box::new(source)),
            })
    }

    /// آیا این اتصال با کلید رمزنگاری باز شده است؟
    ///
    /// توجه: در ساخت SQLCipher، `PRAGMA cipher_version` همیشه موجود است و
    /// برای تشخیص رمزنگاری‌بودن دیتابیس کافی نیست؛ بنابراین وضعیت کلید در
    /// زمان باز شدن اتصال نگه‌داری می‌شود.
    pub fn is_encrypted(&self) -> bool {
        self.encrypted
    }

    /// بررسی سلامت کامل با `PRAGMA integrity_check`.
    ///
    /// در صورت سالم بودن `Ok(())` و در غیر این صورت
    /// [`StorageError::IntegrityCheckFailed`] با فهرست مشکلات بازگردانده می‌شود.
    pub fn integrity_check(&self) -> StorageResult<()> {
        let mut statement =
            self.connection
                .prepare("PRAGMA integrity_check")
                .map_err(|source| StorageError::QueryFailed {
                    context: Some(serde_json::json!({ "pragma": "integrity_check" })),
                    source: Some(Box::new(source)),
                })?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|source| StorageError::QueryFailed {
                context: Some(serde_json::json!({ "pragma": "integrity_check" })),
                source: Some(Box::new(source)),
            })?;

        let mut problems = Vec::new();
        for row in rows {
            let line = row.map_err(|source| StorageError::QueryFailed {
                context: Some(serde_json::json!({ "pragma": "integrity_check" })),
                source: Some(Box::new(source)),
            })?;
            if line != "ok" {
                problems.push(line);
            }
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(StorageError::IntegrityCheckFailed {
                context: Some(serde_json::json!({ "problems": problems })),
                source: None,
            })
        }
    }

    /// بررسی یکپارچگی کلیدهای خارجی با `PRAGMA foreign_key_check`.
    ///
    /// فهرست تخلفات را برمی‌گرداند؛ فهرست خالی یعنی سالم است.
    pub fn foreign_key_check(&self) -> StorageResult<Vec<ForeignKeyViolation>> {
        let mut statement = self
            .connection
            .prepare("PRAGMA foreign_key_check")
            .map_err(|source| StorageError::QueryFailed {
                context: Some(serde_json::json!({ "pragma": "foreign_key_check" })),
                source: Some(Box::new(source)),
            })?;
        let rows = statement
            .query_map([], |row| {
                Ok(ForeignKeyViolation {
                    table: row.get::<_, String>(0)?,
                    rowid: row.get::<_, i64>(1)?,
                    parent: row.get::<_, String>(2)?,
                    constraint_index: row.get::<_, i64>(3)?,
                })
            })
            .map_err(|source| StorageError::QueryFailed {
                context: Some(serde_json::json!({ "pragma": "foreign_key_check" })),
                source: Some(Box::new(source)),
            })?;

        let mut violations = Vec::new();
        for row in rows {
            violations.push(row.map_err(|source| StorageError::QueryFailed {
                context: Some(serde_json::json!({ "pragma": "foreign_key_check" })),
                source: Some(Box::new(source)),
            })?);
        }
        Ok(violations)
    }

    /// بررسی سلامت کامل: یکپارچگی فایل و سپس یکپارچگی کلیدهای خارجی.
    ///
    /// در صورت وجود تخلف کلید خارجی، [`StorageError::ForeignKeyCheckFailed`]
    /// با فهرست تخلفات بازگردانده می‌شود.
    pub fn health_check(&self) -> StorageResult<()> {
        self.integrity_check()?;
        let violations = self.foreign_key_check()?;
        if violations.is_empty() {
            Ok(())
        } else {
            Err(StorageError::ForeignKeyCheckFailed {
                context: Some(serde_json::json!({
                    "violations": violations
                        .iter()
                        .map(|v| serde_json::json!({
                            "table": v.table,
                            "rowid": v.rowid,
                            "parent": v.parent,
                        }))
                        .collect::<Vec<_>>(),
                })),
                source: None,
            })
        }
    }

    /// اتصال را می‌بندد (با `close` صریح rusqlite).
    pub fn close(self) -> StorageResult<()> {
        let path = self.path.clone();
        self.connection
            .close()
            .map_err(|(_, source)| StorageError::DatabaseCloseFailed {
                context: Some(serde_json::json!({ "path": path.display().to_string() })),
                source: Some(Box::new(source)),
            })
    }
}

/// یک تخلف کلید خارجی گزارش‌شده توسط `PRAGMA foreign_key_check`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeyViolation {
    /// جدول فرزند.
    pub table: String,
    /// شناسه ردیف فرزند.
    pub rowid: i64,
    /// جدول والد.
    pub parent: String,
    /// شماره محدودیت نقض‌شده.
    pub constraint_index: i64,
}

fn apply_encryption_key(connection: &Connection, key: &SecretBytes) -> StorageResult<()> {
    if key.len() != DATABASE_KEY_LENGTH_BYTES {
        return Err(StorageError::EncryptionKeyInvalid {
            context: Some(serde_json::json!({
                "expected_bytes": DATABASE_KEY_LENGTH_BYTES,
                "actual_bytes": key.len(),
            })),
            source: None,
        });
    }

    let hex = to_hex(key.expose());
    // کلید خام SQLCipher: PRAGMA key = "x'<hex>'"
    connection
        .execute_batch(&format!("PRAGMA key = \"x'{}'\";", hex))
        .map_err(|source| StorageError::DatabaseOpenFailed {
            context: Some(serde_json::json!({ "step": "pragma_key" })),
            source: Some(Box::new(source)),
        })
}

/// با خواندن از `sqlite_master` بررسی می‌کند که دیتابیس با کلید داده‌شده
/// خوانا است.
///
/// - اگر دیتابیس جدید/خالی باشد، خواندن `sqlite_master` موفق است (جدول‌ها
///   هنوز ساخته نشده‌اند).
/// - اگر دیتابیس رمزنگاری‌شده باشد و کلید نادرست باشد، SQLCipher خطای
///   «file is not a database» می‌دهد که به
///   [`StorageError::EncryptionKeyMismatch`] ترجمه می‌شود.
/// - اگر دیتابیس رمزنگاری‌شده باشد و **بدون** کلید باز شود، همان خطای
///   «file is not a database» رخ می‌دهد که به
///   [`StorageError::EncryptionKeyMismatch`] ترجمه می‌شود (کلید لازم است).
fn verify_readable(connection: &Connection, path: &Path, encrypted: bool) -> StorageResult<()> {
    let result = connection.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
        row.get::<_, i64>(0)
    });
    match result {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(code, _))
            if code.code == rusqlite::ErrorCode::NotADatabase =>
        {
            Err(StorageError::EncryptionKeyMismatch {
                context: Some(serde_json::json!({
                    "path": path.display().to_string(),
                    "key_provided": encrypted,
                })),
                source: None,
            })
        }
        Err(source) => Err(StorageError::DatabaseCorrupted {
            context: Some(serde_json::json!({ "path": path.display().to_string() })),
            source: Some(Box::new(source)),
        }),
    }
}

fn apply_pragmas(connection: &Connection) -> StorageResult<()> {
    connection
        .execute_batch(&format!(
            "PRAGMA journal_mode = WAL;\
             PRAGMA foreign_keys = ON;\
             PRAGMA busy_timeout = {};\
             PRAGMA synchronous = NORMAL;",
            DEFAULT_BUSY_TIMEOUT_MS
        ))
        .map_err(|source| StorageError::DatabaseOpenFailed {
            context: Some(serde_json::json!({ "step": "pragmas" })),
            source: Some(Box::new(source)),
        })
}

fn ensure_parent_directory(path: &Path) -> StorageResult<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|source| StorageError::DatabaseOpenFailed {
                context: Some(serde_json::json!({
                    "step": "create_parent_directory",
                    "path": parent.display().to_string(),
                })),
                source: Some(Box::new(source)),
            })?;
        }
    }
    Ok(())
}

/// رمزگذاری هگز (بدون وابستگی بیرونی).
pub(crate) fn to_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_hex_encodes_lowercase() {
        assert_eq!(to_hex(&[0x00, 0x0f, 0xa5, 0xff]), "000fa5ff");
        assert_eq!(to_hex(&[]), "");
    }

    #[test]
    fn key_length_constant_is_256_bits() {
        assert_eq!(DATABASE_KEY_LENGTH_BYTES, 32);
    }
}
