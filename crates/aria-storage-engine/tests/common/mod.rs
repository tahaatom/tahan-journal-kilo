//! کمک‌کننده‌های مشترک تست‌های یکپارچه موتور ذخیره‌سازی.
//!
//! هر تست یک پوشه موقت یکتا می‌سازد و پس از پایان آن را پاک می‌کند. کلید
//! رمزنگاری تست یک کلید خام ۳۲ بایتی قطعی است.

#![allow(dead_code)]

use std::path::PathBuf;

use aria_contracts::services::SecretBytes;
use aria_foundation_engine::config::KernelConfig;
use aria_storage_engine::StorageEngine;
use uuid::Uuid;

/// کلید رمزنگاری قطعی برای تست‌ها (۳۲ بایت).
pub fn test_key() -> SecretBytes {
    SecretBytes::new((0u8..32).collect())
}

/// کلید رمزنگاری متفاوت برای تست ناسازگاری کلید.
pub fn other_key() -> SecretBytes {
    SecretBytes::new((1u8..=32).collect())
}

/// پوشه موقت یکتا برای تست.
pub fn temp_data_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tahan-storage-{}-{}", label, Uuid::new_v4()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// یک `KernelConfig` برای پوشه داده موقت می‌سازد و پوشه‌ها را ایجاد می‌کند.
pub fn test_config(label: &str) -> KernelConfig {
    let base = temp_data_dir(label);
    let config = KernelConfig::from_base(&base);
    config.ensure_directories().expect("create directories");
    config
}

/// یک موتور ذخیره‌سازی رمزنگاری‌شده با مهاجرت‌های اعمال‌شده می‌سازد.
pub fn migrated_engine(label: &str) -> (StorageEngine, KernelConfig) {
    let config = test_config(label);
    let engine = StorageEngine::open(&config, Some(&test_key())).expect("open database");
    engine.run_migrations().expect("run migrations");
    (engine, config)
}

/// پاک‌سازی پوشه داده.
pub fn cleanup(config: &KernelConfig) {
    let _ = std::fs::remove_dir_all(&config.data_dir);
}

/// یک رکورد پایه (پروفایل، حساب، نماد) می‌سازد و شناسه‌هایشان را برمی‌گرداند.
///
/// صرفاً برای برآورده‌کردن کلیدهای خارجی در تست‌های ذخیره‌سازی؛ هیچ قاعده
/// کسب‌وکاری دامنه‌ای اینجا اعمال نمی‌شود.
pub fn seed_account(engine: &StorageEngine) -> (Uuid, Uuid, Uuid) {
    let profile_id = Uuid::new_v4();
    let account_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let now = aria_storage_engine::timestamps::format(aria_storage_engine::timestamps::now());

    engine
        .with_transaction(|connection| {
            connection
                .execute(
                    "INSERT INTO profiles (id, display_name, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?3)",
                    rusqlite::params![profile_id.to_string(), "پروفایل تست", now],
                )
                .map_err(aria_storage_engine::StorageError::from)?;
            connection
                .execute(
                    "INSERT INTO trading_accounts (id, profile_id, name, broker, currency, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                    rusqlite::params![
                        account_id.to_string(),
                        profile_id.to_string(),
                        "حساب تست",
                        "TestBroker",
                        "USD",
                        now,
                    ],
                )
                .map_err(aria_storage_engine::StorageError::from)?;
            connection
                .execute(
                    "INSERT INTO symbols (id, profile_id, symbol_code, name, market, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                    rusqlite::params![
                        symbol_id.to_string(),
                        profile_id.to_string(),
                        "XAUUSD",
                        "طلا",
                        "forex",
                        now,
                    ],
                )
                .map_err(aria_storage_engine::StorageError::from)?;
            Ok(())
        })
        .expect("seed account");

    (profile_id, account_id, symbol_id)
}

/// یک معامله پایه می‌سازد و شناسه آن را برمی‌گرداند.
pub fn seed_trade(engine: &StorageEngine, account_id: &Uuid, symbol_id: &Uuid) -> Uuid {
    let trade_id = Uuid::new_v4();
    let now = aria_storage_engine::timestamps::format(aria_storage_engine::timestamps::now());
    engine
        .connection()
        .execute(
            "INSERT INTO journal_trades \
             (id, account_id, symbol_id, direction, status, entry_type, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'buy', 'open', 'manual', ?4, ?4)",
            rusqlite::params![
                trade_id.to_string(),
                account_id.to_string(),
                symbol_id.to_string(),
                now,
            ],
        )
        .expect("seed trade");
    trade_id
}
