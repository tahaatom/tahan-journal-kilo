//! کمک‌های بازتابی اسکیما (کشف جدول‌ها و ایندکس‌ها).
//!
//! این ماژول پرس‌وجوی تحلیلی انجام نمی‌دهد؛ فقط ساختار اسکیما را از
//! `sqlite_master` می‌خواند تا تست‌ها، بررسی سلامت و مهاجرت‌ها بتوانند
//! وجود جدول/ایندکس را بررسی کنند.

use rusqlite::Connection;

use crate::error::{StorageError, StorageResult};

/// نام جدول‌های نسخه ۱ (به‌ترتیب قرارداد اسکیمای فیزیکی).
pub const V1_TABLES: &[&str] = &[
    "profiles",
    "trading_accounts",
    "symbols",
    "journal_trades",
    "entry_legs",
    "exit_legs",
    "source_records",
    "executions",
    "manual_overrides",
    "custom_fields",
    "field_options",
    "field_values",
    "attachments",
    "attachment_trade_links",
    "audit_logs",
    "system_events",
    "plugins",
    "settings",
    "dashboard_projections",
];

/// نام ایندکس‌های نسخه ۱.
pub const V1_INDEXES: &[&str] = &[
    "idx_trades_account_id",
    "idx_trades_symbol_id",
    "idx_trades_status",
    "idx_trades_entry_time",
    "idx_entry_legs_trade_id",
    "idx_exit_legs_trade_id",
    "idx_executions_trade_id",
    "idx_executions_leg_id",
    "idx_executions_assignment",
    "idx_field_values_field_id",
    "idx_field_values_integer",
    "idx_field_values_decimal",
    "idx_field_values_text",
    "idx_field_values_datetime",
    "idx_attachments_hash",
    "idx_audit_logs_time",
    "idx_source_records_batch",
    "idx_source_records_status",
];

/// آیا جدولی با این نام وجود دارد؟
pub fn table_exists(connection: &Connection, name: &str) -> StorageResult<bool> {
    object_exists(connection, "table", name)
}

/// آیا ایندکسی با این نام وجود دارد؟
pub fn index_exists(connection: &Connection, name: &str) -> StorageResult<bool> {
    object_exists(connection, "index", name)
}

/// فهرست نام جدول‌های کاربر (به‌جز جدول‌های داخلی SQLite).
pub fn user_tables(connection: &Connection) -> StorageResult<Vec<String>> {
    let mut statement = connection
        .prepare(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
             ORDER BY name",
        )
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "object": "tables" })),
            source: Some(Box::new(source)),
        })?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "object": "tables" })),
            source: Some(Box::new(source)),
        })?;

    let mut names = Vec::new();
    for row in rows {
        names.push(row.map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "object": "tables" })),
            source: Some(Box::new(source)),
        })?);
    }
    Ok(names)
}

fn object_exists(connection: &Connection, kind: &str, name: &str) -> StorageResult<bool> {
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = ?1 AND name = ?2",
            rusqlite::params![kind, name],
            |row| row.get(0),
        )
        .map_err(|source| StorageError::QueryFailed {
            context: Some(serde_json::json!({ "object": kind, "name": name })),
            source: Some(Box::new(source)),
        })?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_table_list_matches_the_contract() {
        assert_eq!(V1_TABLES.len(), 19);
        assert!(V1_TABLES.contains(&"journal_trades"));
        assert!(V1_TABLES.contains(&"field_values"));
        assert!(V1_TABLES.contains(&"dashboard_projections"));
    }

    #[test]
    fn v1_index_list_is_complete() {
        assert_eq!(V1_INDEXES.len(), 18);
        assert!(V1_INDEXES.contains(&"idx_trades_entry_time"));
        assert!(V1_INDEXES.contains(&"idx_attachments_hash"));
    }
}
