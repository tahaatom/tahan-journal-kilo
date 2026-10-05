//! تست یکپارچه قراردادهای عمومی موتور پایه (فاز ۱.۱).
//!
//! این تست‌ها سطح عمومی موتور را به‌عنوان یک کل بررسی می‌کنند:
//! قرارداد خطا، شناسه‌ها، سرویس زمان، پیکربندی و لاگ.

use std::collections::HashSet;
use std::fs;

use aria_foundation_engine::{
    classify_trading_session, gregorian_to_jalali, init_logging, is_market_weekend,
    to_persian_digits, KernelConfig, KernelError, TimeService, TradeId, LOG_FILE_PREFIX,
};
use chrono::{DateTime, TimeZone, Utc};
use tracing::Level;

fn datetime(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .unwrap()
}

#[test]
fn error_contract_is_stable() {
    let error = KernelError::InvalidArgument {
        context: Some(serde_json::json!({ "field": "direction" })),
        source: None,
    };
    assert_eq!(error.code(), 1009);
    assert_eq!(error.variant(), "InvalidArgument");
    assert_eq!(error.message_key(), "error.invalid_argument");
    assert_eq!(
        error
            .context()
            .and_then(|context| context.get("field"))
            .and_then(|value| value.as_str()),
        Some("direction")
    );
    assert!(aria_foundation_engine::ALL_ERROR_DEFINITIONS
        .iter()
        .any(|definition| definition.code == error.code()));
}

#[test]
fn typed_ids_are_unique_and_parseable() {
    let mut seen = HashSet::new();
    for _ in 0..1_000 {
        let id = TradeId::new();
        let parsed: TradeId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
        assert!(seen.insert(id));
    }
    assert_eq!(seen.len(), 1_000);
}

#[test]
fn time_service_stores_utc_and_displays_jalali() {
    let now = TimeService::now_utc();
    let iso = TimeService::to_iso8601(now);
    let parsed = TimeService::from_iso8601(&iso).unwrap();
    assert_eq!(now.timestamp(), parsed.timestamp());

    let display = TimeService::to_jalali_display(datetime(2024, 7, 23, 12, 0)).unwrap();
    assert_eq!(display, "۱۴۰۳/۰۵/۰۲");
}

#[test]
fn jalali_helpers_convert_known_dates() {
    use chrono::NaiveDate;
    // ۲۰۲۴-۰۳-۲۰ = ۱۴۰۳/۰۱/۰۱ (عین فلكی ۱۴۰۳ در تهران)
    let nowruz = gregorian_to_jalali(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap()).unwrap();
    assert_eq!((nowruz.year(), nowruz.month(), nowruz.day()), (1403, 1, 1));
    assert_eq!(to_persian_digits("2024/03/20"), "۲۰۲۴/۰۳/۲۰");
}

#[test]
fn trading_sessions_classify_tehran_time() {
    // UTC ۰۶:۳۰ = تهران ۱۰:۰۰ → regular
    assert_eq!(
        classify_trading_session(datetime(2024, 7, 23, 6, 30)).map(|session| session.key),
        Some("regular")
    );
    // UTC ۰۳:۰۰ = تهران ۰۶:۳۰ → pre_market
    assert_eq!(
        classify_trading_session(datetime(2024, 7, 23, 3, 0)).map(|session| session.key),
        Some("pre_market")
    );
    // UTC ۱۰:۰۰ = تهران ۱۳:۳۰ → after_hours
    assert_eq!(
        classify_trading_session(datetime(2024, 7, 23, 10, 0)).map(|session| session.key),
        Some("after_hours")
    );
    // UTC ۱۲:۰۰ = تهران ۱۵:۳۰ → خارج از نشست‌ها
    assert!(classify_trading_session(datetime(2024, 7, 23, 12, 0)).is_none());
    // ۲۰۲۴-۰۷-۲۶ جمعه است
    assert!(is_market_weekend(datetime(2024, 7, 26, 12, 0)));
    assert!(!is_market_weekend(datetime(2024, 7, 25, 12, 0)));
}

#[test]
fn config_roundtrips_through_json() {
    let directory = std::env::temp_dir().join(format!("tahan-integration-{}", TradeId::new()));
    let mut config = KernelConfig::default_paths().unwrap();
    config.data_dir = directory.clone();
    config.database_path = directory.join("tahan.db");
    config.attachments_dir = directory.join("attachments");
    config.backup_dir = directory.join("backup");
    config.log_dir = directory.join("logs");
    config.plugin_dir = directory.join("plugins");

    let path = directory.join("kernel.json");
    config.save_to(&path).unwrap();
    let loaded = KernelConfig::load_from(&path).unwrap();
    assert_eq!(loaded, config);

    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn logging_writes_structured_local_file() {
    let directory = std::env::temp_dir().join(format!("tahan-integration-log-{}", TradeId::new()));
    let guard = init_logging(&directory, Level::INFO).unwrap();

    let trade_id = TradeId::new();
    tracing::info!(trade_id = %trade_id, "integration log test");
    drop(guard);

    let entries: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with(LOG_FILE_PREFIX))
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !entries.is_empty(),
        "no log file created in {:?}",
        directory
    );

    let content = fs::read_to_string(&entries[0]).unwrap();
    assert!(content.contains("integration log test"));
    assert!(content.contains(&trade_id.to_string()));

    let _ = fs::remove_dir_all(&directory);
}
