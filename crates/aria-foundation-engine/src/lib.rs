//! # aria-foundation-engine — موتور پایه کرنل آریا
//!
//! موتور پایه شامل:
//! - [`error`]: مدل خطای ساختاریافته کرنل (کد عددی یکتا، متغیر، کلید پیام فارسی)
//! - [`ids`]: شناسه‌های تایپ‌شده مبتنی بر UUID v4
//! - [`jalali`]: کمک‌های تقویم جلالی برای نمایش فارسی
//! - [`time`]: سرویس زمان با ذخیره UTC/ISO 8601 و نشست‌های معاملاتی
//! - [`config`]: پیکربندی کرنل
//! - [`logging`]: لاگ‌گذاری محلی
//!
//! قراردادهای عمومی این موتور پایدار و نسخه‌بندی‌شده هستند؛
//! هرگونه تغییر شکسته باید طبق سیاست نسخه‌بندی قراردادها اعمال شود.

pub mod config;
pub mod error;
pub mod ids;
pub mod jalali;
pub mod logging;
pub mod time;

pub use config::{
    KernelConfig, APP_DATA_DIR_NAME, CONFIG_FILE_NAME, DATABASE_FILE_NAME,
    DEFAULT_AUTO_LOCK_TIMEOUT_SECS, DEFAULT_LOCALE, DEFAULT_THEME,
};
pub use error::{
    ErrorDefinition, KernelError, KernelResult, ALL_ERROR_DEFINITIONS, FOUNDATION_ERROR_RANGE,
};
pub use ids::{
    AccountId, AttachmentId, CommandId, EntryLegId, EventId, ExecutionId, ExitLegId, FieldId,
    PluginId, ProfileId, SourceRecordId, SymbolId, TradeId,
};
pub use jalali::{
    days_in_jalali_month, format_jalali_date, format_jalali_date_long, gregorian_to_jalali,
    is_jalali_leap_year, jalali_date_from_utc, jalali_month_name, jalali_to_gregorian,
    jalali_weekday, to_persian_digits, JALALI_MONTH_NAMES, JALALI_WEEKDAY_NAMES,
};
pub use logging::{init_logging, LogGuard, LOG_FILE_PREFIX};
pub use time::{
    classify_trading_session, is_market_weekend, TimeService, TradingSession,
    DEFAULT_TRADING_SESSIONS,
};
