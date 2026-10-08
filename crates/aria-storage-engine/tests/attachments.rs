//! تست‌های ذخیره‌سازی پیوست‌ها (متادیتا، blake3، تشخیص تکراری، یکپارچگی و پیوند).

mod common;

use common::{cleanup, migrated_engine, seed_account, seed_trade};

#[test]
fn attachment_is_stored_outside_database_with_metadata() {
    let (engine, config) = migrated_engine("attachment-store");
    let data = b"chart image bytes".to_vec();

    let info = engine
        .store_attachment("chart.png", "image/png", &data)
        .expect("store");

    assert_eq!(info.filename, "chart.png");
    assert_eq!(info.content_type, "image/png");
    assert_eq!(info.size_bytes, data.len() as u64);
    assert_eq!(info.blake3_hash, blake3::hash(&data).to_hex().to_string());

    // فایل بیرون از دیتابیس و زیر پوشه پیوست‌ها ذخیره شده است.
    let path = engine
        .attachments_dir()
        .join(&info.blake3_hash[..2])
        .join(&info.blake3_hash);
    assert!(path.is_file(), "attachment file must exist on disk");

    // متادیتا داخل دیتابیس است.
    let count: i64 = engine
        .connection()
        .query_row("SELECT count(*) FROM attachments", [], |row| row.get(0))
        .expect("count");
    assert_eq!(count, 1);

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn duplicate_content_returns_existing_metadata() {
    let (engine, config) = migrated_engine("attachment-dedup");
    let data = b"identical bytes".to_vec();

    let first = engine
        .store_attachment("a.png", "image/png", &data)
        .expect("first");
    let second = engine
        .store_attachment("b.png", "image/png", &data)
        .expect("second");

    assert_eq!(first.id, second.id, "duplicate content must reuse metadata");

    let count: i64 = engine
        .connection()
        .query_row("SELECT count(*) FROM attachments", [], |row| row.get(0))
        .expect("count");
    assert_eq!(count, 1, "only one row for duplicate content");

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn reading_attachment_returns_exact_bytes() {
    let (engine, config) = migrated_engine("attachment-read");
    let data = b"\x00\x01\x02 exact bytes".to_vec();
    let info = engine
        .store_attachment("raw.bin", "application/octet-stream", &data)
        .expect("store");

    let read_back = engine.read_attachment(&info.id).expect("read");
    assert_eq!(read_back, data);

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn integrity_verification_detects_tampering() {
    let (engine, config) = migrated_engine("attachment-integrity");
    let data = b"original content".to_vec();
    let info = engine
        .store_attachment("doc.txt", "text/plain", &data)
        .expect("store");

    assert!(engine
        .verify_attachment_integrity(&info.id)
        .expect("verify"));

    // فایل را دستکاری می‌کنیم.
    let path = engine
        .attachments_dir()
        .join(&info.blake3_hash[..2])
        .join(&info.blake3_hash);
    std::fs::write(&path, b"tampered content").expect("tamper");

    assert!(
        !engine
            .verify_attachment_integrity(&info.id)
            .expect("verify"),
        "tampered file must fail integrity check"
    );

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn missing_attachment_reports_not_found() {
    let (engine, config) = migrated_engine("attachment-missing");
    let error = engine.get_attachment(&uuid::Uuid::new_v4()).unwrap_err();
    assert_eq!(error.code(), 2010);
    assert_eq!(error.variant(), "AttachmentNotFound");
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn attachment_can_be_linked_and_listed_for_trade() {
    let (engine, config) = migrated_engine("attachment-link");
    let (_, account_id, symbol_id) = seed_account(&engine);
    let trade_id = seed_trade(&engine, &account_id, &symbol_id);

    let info = engine
        .store_attachment("before.png", "image/png", b"before-trade")
        .expect("store");
    engine
        .link_attachment_to_trade(&info.id, &trade_id, "before_trade")
        .expect("link");

    let listed = engine.list_attachments_for_trade(&trade_id).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, info.id);

    let removed = engine
        .unlink_attachment_from_trade(&info.id, &trade_id)
        .expect("unlink");
    assert!(removed);
    assert!(engine
        .list_attachments_for_trade(&trade_id)
        .expect("list")
        .is_empty());

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn invalid_link_type_is_rejected() {
    let (engine, config) = migrated_engine("attachment-bad-link");
    let (_, account_id, symbol_id) = seed_account(&engine);
    let trade_id = seed_trade(&engine, &account_id, &symbol_id);
    let info = engine
        .store_attachment("x.png", "image/png", b"data")
        .expect("store");

    let error = engine
        .link_attachment_to_trade(&info.id, &trade_id, "not_a_link_type")
        .unwrap_err();
    assert_eq!(
        error.code(),
        2008,
        "invalid link type surfaces as query failed"
    );

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn attachment_metadata_survives_reopen() {
    let config = common::test_config("attachment-reopen");
    let info_id = {
        let engine = aria_storage_engine::StorageEngine::open(&config, Some(&common::test_key()))
            .expect("open");
        engine.run_migrations().expect("migrate");
        let info = engine
            .store_attachment("persist.txt", "text/plain", b"persistent")
            .expect("store");
        let _ = engine.close();
        info.id
    };
    {
        let engine = aria_storage_engine::StorageEngine::open(&config, Some(&common::test_key()))
            .expect("reopen");
        let info = engine.get_attachment(&info_id).expect("get");
        assert_eq!(info.filename, "persist.txt");
        assert_eq!(
            engine.read_attachment(&info_id).expect("read"),
            b"persistent"
        );
        let _ = engine.close();
    }
    cleanup(&config);
}
