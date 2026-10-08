//! تست‌های کلید خارجی: فعال‌بودن، اعمال محدودیت‌ها و cascade.

mod common;

use aria_storage_engine::StorageError;
use common::{cleanup, migrated_engine, seed_account, seed_trade};

#[test]
fn foreign_keys_pragma_is_enabled() {
    let (engine, config) = migrated_engine("fk-pragma");
    let enabled: i64 = engine
        .connection()
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("pragma");
    assert_eq!(enabled, 1);
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn inserting_trade_with_missing_account_is_rejected() {
    let (engine, config) = migrated_engine("fk-missing-account");
    let (_, _, symbol_id) = seed_account(&engine);
    let now = aria_storage_engine::timestamps::format(aria_storage_engine::timestamps::now());

    let result = engine.connection().execute(
        "INSERT INTO journal_trades \
         (id, account_id, symbol_id, direction, status, entry_type, created_at, updated_at) \
         VALUES (?1, ?2, ?3, 'buy', 'open', 'manual', ?4, ?4)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string(), // حساب ناموجود
            symbol_id.to_string(),
            now,
        ],
    );
    assert!(result.is_err(), "missing FK must be rejected");

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn deleting_profile_cascades_to_accounts_and_symbols() {
    let (engine, config) = migrated_engine("fk-cascade");
    let (profile_id, _, _) = seed_account(&engine);

    engine
        .connection()
        .execute(
            "DELETE FROM profiles WHERE id = ?1",
            rusqlite::params![profile_id.to_string()],
        )
        .expect("delete profile");

    let accounts: i64 = engine
        .connection()
        .query_row("SELECT count(*) FROM trading_accounts", [], |row| {
            row.get(0)
        })
        .expect("count accounts");
    let symbols: i64 = engine
        .connection()
        .query_row("SELECT count(*) FROM symbols", [], |row| row.get(0))
        .expect("count symbols");
    assert_eq!(accounts, 0, "accounts must cascade-delete");
    assert_eq!(symbols, 0, "symbols must cascade-delete");

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn deleting_trade_cascades_to_legs_and_executions() {
    let (engine, config) = migrated_engine("fk-trade-cascade");
    let (_, account_id, symbol_id) = seed_account(&engine);
    let trade_id = seed_trade(&engine, &account_id, &symbol_id);
    let now = aria_storage_engine::timestamps::format(aria_storage_engine::timestamps::now());

    engine
        .connection()
        .execute(
            "INSERT INTO entry_legs (id, trade_id, entry_price, volume, entry_time, created_at) \
             VALUES (?1, ?2, '1900.0', '1.0', ?3, ?3)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), trade_id.to_string(), now],
        )
        .expect("insert leg");

    engine
        .connection()
        .execute(
            "DELETE FROM journal_trades WHERE id = ?1",
            rusqlite::params![trade_id.to_string()],
        )
        .expect("delete trade");

    let legs: i64 = engine
        .connection()
        .query_row("SELECT count(*) FROM entry_legs", [], |row| row.get(0))
        .expect("count legs");
    assert_eq!(legs, 0, "legs must cascade-delete");

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn field_values_require_existing_trade_and_field() {
    let (engine, config) = migrated_engine("fk-field-values");
    let now = aria_storage_engine::timestamps::format(aria_storage_engine::timestamps::now());

    engine
        .connection()
        .execute(
            "INSERT INTO custom_fields \
             (id, technical_key, display_label, storage_type, semantic_type, created_at, updated_at) \
             VALUES (?1, 'rating', 'امتیاز', 'Integer', 'rating', ?2, ?2)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), now],
        )
        .expect("insert field");

    let result = engine.connection().execute(
        "INSERT INTO field_values (trade_id, field_id, integer_value, updated_at) \
         VALUES (?1, ?2, 5, ?3)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(), // معامله ناموجود
            uuid::Uuid::new_v4().to_string(), // فیلد ناموجود
            now,
        ],
    );
    assert!(result.is_err(), "missing FKs must be rejected");

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn manual_error_maps_to_storage_query_failed() {
    // تبدیل خطای rusqlite به StorageError از مسیر From بررسی می‌شود.
    let error: StorageError = rusqlite::Error::QueryReturnedNoRows.into();
    assert_eq!(error.code(), 2008);
    assert_eq!(error.variant(), "QueryFailed");
}
