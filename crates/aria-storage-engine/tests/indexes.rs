//! تست‌های وجود ایندکس‌های نسخه ۱.

mod common;

use aria_storage_engine::schema::{index_exists, V1_INDEXES};
use common::{cleanup, migrated_engine};

#[test]
fn all_v1_indexes_exist_after_migration() {
    let (engine, config) = migrated_engine("indexes");
    for index in V1_INDEXES {
        assert!(
            index_exists(engine.connection(), index).unwrap(),
            "index {} must exist",
            index
        );
    }
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn indexes_are_reported_by_sqlite_master() {
    let (engine, config) = migrated_engine("indexes-master");
    let count: i64 = engine
        .connection()
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_%'",
            [],
            |row| row.get(0),
        )
        .expect("count indexes");
    assert_eq!(count as usize, V1_INDEXES.len());
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn key_query_indexes_are_present() {
    let (engine, config) = migrated_engine("indexes-keys");
    for index in [
        "idx_trades_account_id",
        "idx_trades_symbol_id",
        "idx_trades_status",
        "idx_trades_entry_time",
        "idx_executions_trade_id",
        "idx_executions_leg_id",
        "idx_field_values_field_id",
        "idx_attachments_hash",
        "idx_audit_logs_time",
        "idx_source_records_batch",
        "idx_source_records_status",
    ] {
        assert!(
            index_exists(engine.connection(), index).unwrap(),
            "{}",
            index
        );
    }
    let _ = engine.close();
    cleanup(&config);
}
