//! تست‌های رمزنگاری: باز/بسته‌کردن دیتابیس با SQLCipher و رفتار کلید.

mod common;

use aria_storage_engine::connection::{DatabaseConnection, DATABASE_KEY_LENGTH_BYTES};
use aria_storage_engine::StorageError;
use common::{cleanup, other_key, test_config, test_key};

#[test]
fn sqlcipher_is_available() {
    let config = test_config("enc-cipher-version");
    let connection =
        DatabaseConnection::open(&config.database_path, Some(&test_key())).expect("open");
    let version = connection.cipher_version().expect("cipher_version");
    assert!(
        version.to_lowercase().contains("cipher") || version.starts_with('4'),
        "unexpected cipher_version: {}",
        version
    );
    assert!(connection.is_encrypted());
    let _ = connection.close();
    cleanup(&config);
}

#[test]
fn encrypted_database_rejects_plaintext_on_disk() {
    let config = test_config("enc-plaintext");
    let secret_payload = "supersecretfinancialdata";
    {
        let connection =
            DatabaseConnection::open(&config.database_path, Some(&test_key())).expect("open");
        connection
            .execute_batch("CREATE TABLE secrets (id INTEGER PRIMARY KEY, value TEXT);")
            .expect("create");
        connection
            .connection()
            .execute(
                "INSERT INTO secrets (value) VALUES (?1)",
                rusqlite::params![secret_payload],
            )
            .expect("insert");
        let _ = connection.close();
    }

    let bytes = std::fs::read(&config.database_path).expect("read db");
    let as_text = String::from_utf8_lossy(&bytes);
    assert!(
        !as_text.contains(secret_payload),
        "plaintext must not appear in the encrypted database file"
    );

    cleanup(&config);
}

#[test]
fn database_reopens_with_correct_key() {
    let config = test_config("enc-reopen");
    {
        let connection =
            DatabaseConnection::open(&config.database_path, Some(&test_key())).expect("open");
        connection
            .execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, v TEXT); INSERT INTO t (v) VALUES ('hello');")
            .expect("init");
        let _ = connection.close();
    }
    {
        let connection =
            DatabaseConnection::open(&config.database_path, Some(&test_key())).expect("reopen");
        let value: String = connection
            .connection()
            .query_row("SELECT v FROM t WHERE id = 1", [], |row| row.get(0))
            .expect("read");
        assert_eq!(value, "hello");
        let _ = connection.close();
    }
    cleanup(&config);
}

#[test]
fn wrong_key_is_rejected() {
    let config = test_config("enc-wrong-key");
    {
        let connection =
            DatabaseConnection::open(&config.database_path, Some(&test_key())).expect("open");
        connection
            .execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY);")
            .expect("create");
        let _ = connection.close();
    }

    let result = DatabaseConnection::open(&config.database_path, Some(&other_key()));
    assert!(result.is_err(), "wrong key must be rejected");
    assert_eq!(result.unwrap_err().variant(), "EncryptionKeyMismatch");

    cleanup(&config);
}

#[test]
fn opening_encrypted_database_without_key_is_rejected() {
    let config = test_config("enc-no-key");
    {
        let connection =
            DatabaseConnection::open(&config.database_path, Some(&test_key())).expect("open");
        connection
            .execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY);")
            .expect("create");
        let _ = connection.close();
    }

    let result = DatabaseConnection::open(&config.database_path, None);
    assert!(result.is_err(), "encrypted database must require a key");
    assert_eq!(result.unwrap_err().variant(), "EncryptionKeyMismatch");

    cleanup(&config);
}

#[test]
fn invalid_key_length_is_rejected() {
    let config = test_config("enc-bad-length");
    let short_key = aria_contracts::services::SecretBytes::new(vec![0u8; 16]);
    let error = DatabaseConnection::open(&config.database_path, Some(&short_key)).unwrap_err();
    assert_eq!(error.code(), 2003);
    assert_eq!(error.variant(), "EncryptionKeyInvalid");

    let context = error.context().expect("context");
    assert_eq!(
        context.get("expected_bytes").and_then(|v| v.as_u64()),
        Some(DATABASE_KEY_LENGTH_BYTES as u64)
    );

    cleanup(&config);
}

#[test]
fn unencrypted_database_is_supported_for_development() {
    let config = test_config("enc-plain");
    let connection = DatabaseConnection::open(&config.database_path, None).expect("open");
    assert!(!connection.is_encrypted());
    connection
        .execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY);")
        .expect("create");
    let _ = connection.close();
    cleanup(&config);
}

#[test]
fn encrypted_storage_engine_full_roundtrip() {
    let config = test_config("enc-engine");
    let trade_count = {
        let engine =
            aria_storage_engine::StorageEngine::open(&config, Some(&test_key())).expect("open");
        assert!(engine.is_encrypted());
        engine.run_migrations().expect("migrate");
        common::seed_account(&engine);
        let count: i64 = engine
            .connection()
            .query_row("SELECT count(*) FROM trading_accounts", [], |row| {
                row.get(0)
            })
            .expect("count");
        let _ = engine.close();
        count
    };
    assert_eq!(trade_count, 1);

    let engine =
        aria_storage_engine::StorageEngine::open(&config, Some(&test_key())).expect("reopen");
    let count: i64 = engine
        .connection()
        .query_row("SELECT count(*) FROM trading_accounts", [], |row| {
            row.get(0)
        })
        .expect("count");
    assert_eq!(count, 1);
    let _ = engine.close();

    let _ = StorageError::DatabaseOpenFailed {
        context: None,
        source: None,
    };
    cleanup(&config);
}
