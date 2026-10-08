//! تست‌های یکپارچگی و بررسی سلامت.

mod common;

use aria_storage_engine::connection::DatabaseConnection;
use common::{cleanup, migrated_engine, seed_account, test_config, test_key};

#[test]
fn integrity_check_passes_on_fresh_database() {
    let (engine, config) = migrated_engine("integrity-fresh");
    engine.integrity_check().expect("integrity ok");
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn health_check_passes_on_consistent_database() {
    let (engine, config) = migrated_engine("integrity-health");
    seed_account(&engine);
    engine.health_check().expect("health ok");
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn foreign_key_check_reports_no_violations_when_consistent() {
    let (engine, config) = migrated_engine("integrity-fk");
    seed_account(&engine);
    let violations = engine.foreign_key_check().expect("fk check");
    assert!(violations.is_empty());
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn opening_a_non_database_file_fails() {
    let config = test_config("integrity-not-a-db");
    // یک فایل متنی معمولی به‌جای دیتابیس قرار می‌دهیم.
    std::fs::write(&config.database_path, b"this is definitely not a database")
        .expect("write junk");

    let result = DatabaseConnection::open(&config.database_path, Some(&test_key()));
    assert!(result.is_err(), "non-database file must fail to open");
    let error = result.unwrap_err();
    assert!(
        error.code() == 2004 || error.code() == 2016,
        "expected corrupted/key-mismatch error, got {}",
        error.variant()
    );

    cleanup(&config);
}

#[test]
fn integrity_check_detects_corruption_when_readable() {
    let config = test_config("integrity-corrupt");
    // ابتدا یک دیتابیس سالم رمزنگاری‌شده می‌سازیم.
    {
        let engine =
            aria_storage_engine::StorageEngine::open(&config, Some(&test_key())).expect("open");
        engine.run_migrations().expect("migrate");
        let _ = engine.close();
    }

    // بایت‌های میانی فایل را خراب می‌کنیم (بدون آسیب به سرصفحه اصلی) تا
    // SQLCipher کلید را بپذیرد اما یکپارچگی صفحه‌ها از دست برود.
    let mut bytes = std::fs::read(&config.database_path).expect("read db");
    let len = bytes.len();
    if len > 200 {
        for byte in bytes.iter_mut().skip(512).take(256) {
            *byte ^= 0xFF;
        }
    }
    std::fs::write(&config.database_path, &bytes).expect("write corrupted db");

    let result = DatabaseConnection::open(&config.database_path, Some(&test_key()));
    match result {
        Ok(connection) => {
            // اگر باز شد، بررسی یکپارچگی باید مشکل را گزارش کند.
            let integrity = connection.integrity_check();
            assert!(
                integrity.is_err(),
                "corrupted database must fail integrity check"
            );
            let _ = connection.close();
        }
        Err(error) => {
            assert!(
                error.code() == 2004 || error.code() == 2016,
                "expected corruption/key-mismatch, got {}",
                error.variant()
            );
        }
    }

    cleanup(&config);
}
