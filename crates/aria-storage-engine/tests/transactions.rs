//! تست‌های تراکنش: ثبت، بازگشت و رفتار در خطا.

mod common;

use aria_storage_engine::{StorageError, StorageTransaction};
use common::{cleanup, migrated_engine};

fn count_accounts(connection: &rusqlite::Connection) -> i64 {
    connection
        .query_row("SELECT count(*) FROM trading_accounts", [], |row| {
            row.get(0)
        })
        .expect("count")
}

#[test]
fn committed_transaction_persists_changes() {
    let (engine, config) = migrated_engine("txn-commit");
    common::seed_account(&engine);
    assert_eq!(count_accounts(engine.connection()), 1);

    engine
        .with_transaction(|connection| {
            connection
                .execute(
                    "INSERT INTO trading_accounts \
                     (id, profile_id, name, broker, currency, created_at, updated_at) \
                     SELECT ?1, profile_id, 'حساب دوم', 'B', 'USD', created_at, updated_at \
                     FROM trading_accounts LIMIT 1",
                    rusqlite::params![uuid::Uuid::new_v4().to_string()],
                )
                .map_err(StorageError::from)?;
            Ok(())
        })
        .expect("transaction");

    assert_eq!(count_accounts(engine.connection()), 2);
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn explicit_rollback_discards_changes() {
    let (engine, config) = migrated_engine("txn-rollback");
    common::seed_account(&engine);

    let transaction = StorageTransaction::begin(engine.connection()).expect("begin");
    transaction
        .connection()
        .execute(
            "INSERT INTO trading_accounts \
             (id, profile_id, name, broker, currency, created_at, updated_at) \
             SELECT ?1, profile_id, 'حساب موقت', 'B', 'USD', created_at, updated_at \
             FROM trading_accounts LIMIT 1",
            rusqlite::params![uuid::Uuid::new_v4().to_string()],
        )
        .expect("insert");
    transaction.rollback().expect("rollback");

    assert_eq!(count_accounts(engine.connection()), 1);
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn dropped_transaction_rolls_back_automatically() {
    let (engine, config) = migrated_engine("txn-drop");
    common::seed_account(&engine);

    {
        let transaction = StorageTransaction::begin(engine.connection()).expect("begin");
        transaction
            .connection()
            .execute(
                "INSERT INTO trading_accounts \
                 (id, profile_id, name, broker, currency, created_at, updated_at) \
                 SELECT ?1, profile_id, 'حساب رهاشده', 'B', 'USD', created_at, updated_at \
                 FROM trading_accounts LIMIT 1",
                rusqlite::params![uuid::Uuid::new_v4().to_string()],
            )
            .expect("insert");
        // بدون commit رها می‌شود؛ باید خودکار rollback شود.
    }

    assert_eq!(count_accounts(engine.connection()), 1);
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn transaction_run_rolls_back_on_error() {
    let (engine, config) = migrated_engine("txn-run-error");
    common::seed_account(&engine);

    let result: Result<(), StorageError> = engine.with_transaction(|connection| {
        connection
            .execute(
                "INSERT INTO trading_accounts \
                 (id, profile_id, name, broker, currency, created_at, updated_at) \
                 SELECT ?1, profile_id, 'ناتمام', 'B', 'USD', created_at, updated_at \
                 FROM trading_accounts LIMIT 1",
                rusqlite::params![uuid::Uuid::new_v4().to_string()],
            )
            .map_err(StorageError::from)?;
        // خطای عمدی پس از نوشتن.
        Err(StorageError::TransactionFailed {
            context: None,
            source: None,
        })
    });

    assert!(result.is_err());
    assert_eq!(
        count_accounts(engine.connection()),
        1,
        "write must be rolled back"
    );
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn reads_do_not_require_a_transaction() {
    let (engine, config) = migrated_engine("txn-read");
    common::seed_account(&engine);

    // خواندن مستقیم روی اتصال (بدون تراکنش صریح) کار می‌کند.
    let count: i64 = engine
        .connection()
        .query_row("SELECT count(*) FROM profiles", [], |row| row.get(0))
        .expect("read");
    assert_eq!(count, 1);

    let _ = engine.close();
    cleanup(&config);
}
