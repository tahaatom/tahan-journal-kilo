//! تست‌های مهاجرت: اجرای رو به بالا، اتمیک بودن، نسخه ناشناخته و قلاب پشتیبان.

mod common;

use aria_storage_engine::migration::{
    latest_schema_version, Migration, SchemaMigrator, MIGRATIONS,
};
use aria_storage_engine::schema::{table_exists, V1_TABLES};
use aria_storage_engine::{StorageEngine, StorageError};
use common::{cleanup, test_config, test_key};

#[test]
fn migrations_apply_cleanly_and_set_user_version() {
    let config = test_config("migrate-up");
    let engine = StorageEngine::open(&config, Some(&test_key())).expect("open");
    assert_eq!(engine.schema_version().unwrap(), 0);

    let report = engine.run_migrations().expect("migrate");
    assert_eq!(report.from_version, 0);
    assert_eq!(report.to_version, 1);
    assert_eq!(report.applied, vec![1]);
    assert!(report.changed());
    assert_eq!(engine.schema_version().unwrap(), latest_schema_version());

    for table in V1_TABLES {
        assert!(
            table_exists(engine.connection(), table).unwrap(),
            "table {} must exist after migration",
            table
        );
    }

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn migrations_are_idempotent() {
    let config = test_config("migrate-idempotent");
    let engine = StorageEngine::open(&config, Some(&test_key())).expect("open");
    engine.run_migrations().expect("first migrate");

    let second = engine.run_migrations().expect("second migrate");
    assert_eq!(second.from_version, 1);
    assert_eq!(second.to_version, 1);
    assert!(second.applied.is_empty());
    assert!(!second.changed());

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn schema_persists_across_reopen() {
    let config = test_config("migrate-persist");
    {
        let engine = StorageEngine::open(&config, Some(&test_key())).expect("open");
        engine.run_migrations().expect("migrate");
        let _ = engine.close();
    }
    {
        let engine = StorageEngine::open(&config, Some(&test_key())).expect("reopen");
        assert_eq!(engine.schema_version().unwrap(), 1);
        assert!(table_exists(engine.connection(), "journal_trades").unwrap());
        let _ = engine.close();
    }
    cleanup(&config);
}

#[test]
fn unknown_newer_schema_version_is_rejected() {
    let config = test_config("migrate-unknown");
    let engine = StorageEngine::open(&config, Some(&test_key())).expect("open");
    // شبیه‌سازی دیتابیسی که با نسخه جدیدتر نوشته شده است.
    engine
        .connection()
        .execute_batch("PRAGMA user_version = 99")
        .expect("set user_version");

    let error = engine.run_migrations().unwrap_err();
    assert_eq!(error.code(), 2006);
    assert_eq!(error.variant(), "MigrationVersionUnknown");

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn failing_migration_is_atomic_and_does_not_bump_version() {
    let config = test_config("migrate-atomic");
    let engine = StorageEngine::open(&config, Some(&test_key())).expect("open");

    // مهاجرت ساختگی که در میانه شکست می‌خورد (جدول دوم تکراری است).
    const BROKEN: &[Migration] = &[Migration {
        version: 1,
        name: "broken_v1",
        sql: "CREATE TABLE good_table (id TEXT PRIMARY KEY); \
              CREATE TABLE good_table (id TEXT PRIMARY KEY);",
        requires_backup: false,
    }];

    let result = SchemaMigrator::new(engine.connection()).run_migrations_from(BROKEN, |_| Ok(()));
    assert!(result.is_err(), "broken migration must fail");

    // نه نسخه تغییر کرده و نه جدول نیمه‌ساخته باقی مانده است.
    assert_eq!(engine.schema_version().unwrap(), 0);
    assert!(!table_exists(engine.connection(), "good_table").unwrap());

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn backup_hook_is_invoked_before_risky_migration() {
    let config = test_config("migrate-backup-hook");
    let engine = StorageEngine::open(&config, Some(&test_key())).expect("open");

    const RISKY: &[Migration] = &[Migration {
        version: 1,
        name: "risky_v1",
        sql: "CREATE TABLE risky_table (id TEXT PRIMARY KEY);",
        requires_backup: true,
    }];

    let mut hook_calls = Vec::new();
    let report = SchemaMigrator::new(engine.connection())
        .run_migrations_from(RISKY, |version| {
            hook_calls.push(version);
            Ok(())
        })
        .expect("migrate with hook");

    assert_eq!(hook_calls, vec![1]);
    assert_eq!(report.applied, vec![1]);

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn backup_hook_failure_blocks_migration() {
    let config = test_config("migrate-backup-fail");
    let engine = StorageEngine::open(&config, Some(&test_key())).expect("open");

    const RISKY: &[Migration] = &[Migration {
        version: 1,
        name: "risky_v1",
        sql: "CREATE TABLE risky_table (id TEXT PRIMARY KEY);",
        requires_backup: true,
    }];

    let error = SchemaMigrator::new(engine.connection())
        .run_migrations_from(RISKY, |_| {
            Err(StorageError::AttachmentWriteFailed {
                context: None,
                source: None,
            })
        })
        .unwrap_err();

    assert_eq!(error.code(), 2015);
    assert_eq!(error.variant(), "MigrationBackupFailed");
    assert_eq!(
        engine.schema_version().unwrap(),
        0,
        "migration must not run"
    );
    assert!(!table_exists(engine.connection(), "risky_table").unwrap());

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn real_migration_list_is_valid() {
    assert_eq!(MIGRATIONS.len(), 1);
    assert_eq!(latest_schema_version(), 1);
}
