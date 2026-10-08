//! تست‌های پایه چیدمان بسته پشتیبان (فاز ۱.۳).

mod common;

use aria_contracts::versioning::BACKUP_FORMAT_VERSION;
use aria_storage_engine::backup::{
    read_manifest, BACKUP_ATTACHMENTS_DIR, BACKUP_CHECKSUMS_FILE, BACKUP_MANIFEST_FILE,
    BACKUP_PAYLOAD_DIR,
};
use common::{cleanup, migrated_engine};

#[test]
fn backup_layout_is_created_with_expected_structure() {
    let (engine, config) = migrated_engine("backup-layout");
    let package = engine.create_backup_layout().expect("create layout");

    assert!(package.path.is_dir(), "package directory must exist");
    assert!(
        package.manifest_path().is_file(),
        "manifest.json must exist"
    );
    assert!(
        package.checksums_path().is_file(),
        "checksums.json must exist"
    );
    assert!(package.payload_dir().is_dir(), "payload dir must exist");
    assert!(
        package.attachments_dir().is_dir(),
        "attachments dir must exist"
    );
    assert_eq!(package.format_version, BACKUP_FORMAT_VERSION);

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn backup_layout_paths_use_contract_names() {
    let (engine, config) = migrated_engine("backup-names");
    let package = engine.create_backup_layout().expect("create");
    assert_eq!(
        package.manifest_path().file_name().unwrap(),
        BACKUP_MANIFEST_FILE
    );
    assert_eq!(
        package.checksums_path().file_name().unwrap(),
        BACKUP_CHECKSUMS_FILE
    );
    assert_eq!(
        package.payload_dir().file_name().unwrap(),
        BACKUP_PAYLOAD_DIR
    );
    assert_eq!(
        package.attachments_dir().file_name().unwrap(),
        BACKUP_ATTACHMENTS_DIR
    );
    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn backup_manifest_is_valid_and_versioned() {
    let (engine, config) = migrated_engine("backup-manifest");
    let package = engine.create_backup_layout().expect("create");

    let manifest = read_manifest(&package.path).expect("read manifest");
    assert_eq!(manifest.backup_id, package.id);
    assert_eq!(manifest.format_version, BACKUP_FORMAT_VERSION);
    assert_eq!(
        manifest.schema_version, 1,
        "manifest records the schema version"
    );
    assert_eq!(manifest.attachment_count, 0);
    assert_eq!(manifest.encrypted_database_size_bytes, 0);

    let _ = engine.close();
    cleanup(&config);
}

#[test]
fn backup_packages_use_unique_ids() {
    let (engine, config) = migrated_engine("backup-unique");
    let first = engine.create_backup_layout().expect("first");
    let second = engine.create_backup_layout().expect("second");
    assert_ne!(first.id, second.id);
    assert_ne!(first.path, second.path);
    let _ = engine.close();
    cleanup(&config);
}
