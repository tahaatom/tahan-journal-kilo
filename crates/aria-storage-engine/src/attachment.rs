//! ذخیره‌سازی پیوست‌ها.
//!
//! قرارداد (فاز ۱.۳ و ۱.۱۳):
//!
//! - فایل پیوست **بیرون** از دیتابیس ذخیره می‌شود؛ فقط متادیتا داخل دیتابیس است.
//! - هش **blake3** برای یکپارچگی و تشخیص تکراری‌بودن محتوا استفاده می‌شود.
//! - فایل‌ها **محتوامحور** ذخیره می‌شوند: `attachments/<xx>/<hash>` که `<xx>`
//!   دو نویسه نخست هش است (توزیع پوشه‌ها برای جلوگیری از پوشه‌های بسیار بزرگ).
//! - `store` برای محتوای تکراری، متادیتای موجود را برمی‌گرداند (بدون نوشتن دوباره).
//! - متادیتای مسیر بندانگشتی (`thumbnail_path`) در نسخه ۱ رزرو است و در فاز ۱.۱۳
//!   پر می‌شود.

use std::path::{Path, PathBuf};

use aria_contracts::services::AttachmentInfo;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};
use crate::timestamps;

/// انواع مجاز پیوند پیوست به معامله (قرارداد فاز ۱.۱۳).
pub const LINK_TYPES: &[&str] = &["before_trade", "after_trade", "chart", "news", "other"];

/// ذخیره پیوست و بازگرداندن متادیتای آن.
///
/// اگر محتوای یکسان (بر اساس هش blake3) قبلاً ذخیره شده باشد، همان متادیتا
/// بازگردانده می‌شود و فایل دوباره نوشته نمی‌شود.
pub fn store(
    connection: &Connection,
    attachments_dir: &Path,
    filename: &str,
    content_type: &str,
    data: &[u8],
) -> StorageResult<AttachmentInfo> {
    let hash = blake3::hash(data).to_hex().to_string();

    if let Some(existing) = find_by_hash(connection, &hash)? {
        return Ok(existing);
    }

    let id = Uuid::new_v4();
    let relative_path = relative_storage_path(&hash);
    let absolute_path = attachments_dir.join(&relative_path);
    write_file(&absolute_path, data)?;

    let created_at = timestamps::now();
    let info = AttachmentInfo {
        id,
        filename: filename.to_string(),
        content_type: content_type.to_string(),
        size_bytes: data.len() as u64,
        blake3_hash: hash,
        created_at,
    };

    connection
        .execute(
            "INSERT INTO attachments \
             (id, filename, content_type, size_bytes, blake3_hash, storage_path, thumbnail_path, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7)",
            rusqlite::params![
                id.to_string(),
                info.filename,
                info.content_type,
                info.size_bytes,
                info.blake3_hash,
                relative_path,
                timestamps::format(created_at),
            ],
        )
        .map_err(|source| StorageError::AttachmentWriteFailed {
            context: Some(serde_json::json!({ "filename": filename })),
            source: Some(Box::new(source)),
        })?;

    Ok(info)
}

/// متادیتای یک پیوست را برمی‌گرداند.
pub fn get(connection: &Connection, attachment_id: &Uuid) -> StorageResult<AttachmentInfo> {
    load_attachment(
        connection,
        "SELECT id, filename, content_type, size_bytes, blake3_hash, created_at \
         FROM attachments WHERE id = ?1",
        attachment_id,
    )
}

/// محتوای خام یک پیوست را می‌خواند.
pub fn read(
    connection: &Connection,
    attachments_dir: &Path,
    attachment_id: &Uuid,
) -> StorageResult<Vec<u8>> {
    let path = absolute_path_for(connection, attachments_dir, attachment_id)?;
    std::fs::read(&path).map_err(|source| StorageError::AttachmentNotFound {
        context: Some(serde_json::json!({
            "attachment_id": attachment_id.to_string(),
            "path": path.display().to_string(),
        })),
        source: Some(Box::new(source)),
    })
}

/// یکپارچگی فایل پیوست را با بازمحاسبه هش blake3 بررسی می‌کند.
///
/// `true` یعنی فایل دست‌نخورده است؛ `false` یعنی فایل با هش ثبت‌شده تفاوت دارد.
pub fn verify_integrity(
    connection: &Connection,
    attachments_dir: &Path,
    attachment_id: &Uuid,
) -> StorageResult<bool> {
    let info = get(connection, attachment_id)?;
    let path = absolute_path_for(connection, attachments_dir, attachment_id)?;
    let data = std::fs::read(&path).map_err(|source| StorageError::AttachmentNotFound {
        context: Some(serde_json::json!({
            "attachment_id": attachment_id.to_string(),
            "path": path.display().to_string(),
        })),
        source: Some(Box::new(source)),
    })?;
    Ok(blake3::hash(&data).to_hex().to_string() == info.blake3_hash)
}

/// یک پیوست را به یک معامله پیوند می‌زند.
pub fn link_to_trade(
    connection: &Connection,
    attachment_id: &Uuid,
    trade_id: &Uuid,
    link_type: &str,
) -> StorageResult<()> {
    if !LINK_TYPES.contains(&link_type) {
        return Err(StorageError::QueryFailed {
            context: Some(serde_json::json!({
                "reason": "invalid_link_type",
                "link_type": link_type,
                "allowed": LINK_TYPES,
            })),
            source: None,
        });
    }

    connection
        .execute(
            "INSERT INTO attachment_trade_links (attachment_id, trade_id, link_type, created_at) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT (attachment_id, trade_id) DO UPDATE SET link_type = excluded.link_type",
            rusqlite::params![
                attachment_id.to_string(),
                trade_id.to_string(),
                link_type,
                timestamps::format(timestamps::now()),
            ],
        )
        .map_err(|source| StorageError::AttachmentWriteFailed {
            context: Some(serde_json::json!({
                "attachment_id": attachment_id.to_string(),
                "trade_id": trade_id.to_string(),
            })),
            source: Some(Box::new(source)),
        })?;
    Ok(())
}

/// پیوند یک پیوست از یک معامله را برمی‌دارد. اگر پیوندی وجود نداشته باشد،
/// `false` برمی‌گرداند.
pub fn unlink_from_trade(
    connection: &Connection,
    attachment_id: &Uuid,
    trade_id: &Uuid,
) -> StorageResult<bool> {
    let affected = connection
        .execute(
            "DELETE FROM attachment_trade_links WHERE attachment_id = ?1 AND trade_id = ?2",
            rusqlite::params![attachment_id.to_string(), trade_id.to_string()],
        )
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "operation": "unlink_attachment" })),
            source: Some(Box::new(source)),
        })?;
    Ok(affected > 0)
}

/// فهرست پیوست‌های پیوند‌خورده به یک معامله.
pub fn list_for_trade(
    connection: &Connection,
    trade_id: &Uuid,
) -> StorageResult<Vec<AttachmentInfo>> {
    let mut statement = connection
        .prepare(
            "SELECT a.id, a.filename, a.content_type, a.size_bytes, a.blake3_hash, a.created_at \
             FROM attachments a \
             JOIN attachment_trade_links l ON l.attachment_id = a.id \
             WHERE l.trade_id = ?1 \
             ORDER BY a.created_at",
        )
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "operation": "list_attachments_for_trade" })),
            source: Some(Box::new(source)),
        })?;

    let rows = statement
        .query_map(rusqlite::params![trade_id.to_string()], map_attachment_row)
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "operation": "list_attachments_for_trade" })),
            source: Some(Box::new(source)),
        })?;

    let mut attachments = Vec::new();
    for row in rows {
        attachments.push(row.map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "operation": "list_attachments_for_trade" })),
            source: Some(Box::new(source)),
        })?);
    }
    Ok(attachments)
}

fn find_by_hash(connection: &Connection, hash: &str) -> StorageResult<Option<AttachmentInfo>> {
    connection
        .query_row(
            "SELECT id, filename, content_type, size_bytes, blake3_hash, created_at \
             FROM attachments WHERE blake3_hash = ?1 LIMIT 1",
            rusqlite::params![hash],
            map_attachment_row,
        )
        .optional()
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "operation": "find_attachment_by_hash" })),
            source: Some(Box::new(source)),
        })
}

fn load_attachment(
    connection: &Connection,
    sql: &str,
    attachment_id: &Uuid,
) -> StorageResult<AttachmentInfo> {
    connection
        .query_row(
            sql,
            rusqlite::params![attachment_id.to_string()],
            map_attachment_row,
        )
        .optional()
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "operation": "get_attachment" })),
            source: Some(Box::new(source)),
        })?
        .ok_or_else(|| StorageError::AttachmentNotFound {
            context: Some(serde_json::json!({ "attachment_id": attachment_id.to_string() })),
            source: None,
        })
}

fn absolute_path_for(
    connection: &Connection,
    attachments_dir: &Path,
    attachment_id: &Uuid,
) -> StorageResult<PathBuf> {
    let relative: String = connection
        .query_row(
            "SELECT storage_path FROM attachments WHERE id = ?1",
            rusqlite::params![attachment_id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "operation": "attachment_storage_path" })),
            source: Some(Box::new(source)),
        })?
        .ok_or_else(|| StorageError::AttachmentNotFound {
            context: Some(serde_json::json!({ "attachment_id": attachment_id.to_string() })),
            source: None,
        })?;
    Ok(attachments_dir.join(relative))
}

fn relative_storage_path(hash: &str) -> String {
    let prefix = &hash[..2];
    format!("{prefix}/{hash}")
}

fn write_file(path: &Path, data: &[u8]) -> StorageResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| StorageError::AttachmentWriteFailed {
            context: Some(serde_json::json!({ "path": parent.display().to_string() })),
            source: Some(Box::new(source)),
        })?;
    }
    std::fs::write(path, data).map_err(|source| StorageError::AttachmentWriteFailed {
        context: Some(serde_json::json!({ "path": path.display().to_string() })),
        source: Some(Box::new(source)),
    })
}

fn map_attachment_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AttachmentInfo> {
    let id: String = row.get(0)?;
    let created_at: String = row.get(5)?;
    let id = Uuid::parse_str(&id).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
    Ok(AttachmentInfo {
        id,
        filename: row.get(1)?,
        content_type: row.get(2)?,
        size_bytes: row.get(3)?,
        blake3_hash: row.get(4)?,
        created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_are_sharded_by_hash_prefix() {
        let hash = "abcdef0123456789";
        assert_eq!(relative_storage_path(hash), "ab/abcdef0123456789");
    }

    #[test]
    fn link_types_match_the_contract() {
        assert_eq!(
            LINK_TYPES,
            &["before_trade", "after_trade", "chart", "news", "other"]
        );
    }
}
