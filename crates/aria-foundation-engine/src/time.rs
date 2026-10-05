//! سرویس زمان کرنل آریا.
//!
//! قاعده کلی: زمان همیشه به‌صورت UTC/ISO 8601 ذخیره می‌شود.
//! تبدیل‌ها و قالب‌بندی‌ها فقط برای نمایش فارسی/جلالی استفاده می‌شوند.
//!
//! نشست‌های معاملاتی نسخه ۱ به‌عنوان یک قلاب ساده بر اساس ساعت
//! ثابت تهران (UTC+3:30) تعریف شده‌اند.

use chrono::{DateTime, Datelike, FixedOffset, NaiveTime, Timelike, Utc, Weekday};
use parsidate::ParsiDate;

use crate::error::KernelError;
use crate::jalali;

/// سرویس زمان کرنل.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeService;

impl TimeService {
    /// زمان فعلی را به‌صورت UTC برمی‌گرداند.
    pub fn now_utc() -> DateTime<Utc> {
        Utc::now()
    }

    /// یک زمان UTC را به رشته ISO 8601 (RFC 3339) تبدیل می‌کند.
    pub fn to_iso8601(datetime: DateTime<Utc>) -> String {
        datetime.to_rfc3339()
    }

    /// یک رشته ISO 8601 (RFC 3339) را به زمان UTC تبدیل می‌کند.
    pub fn from_iso8601(value: &str) -> Result<DateTime<Utc>, KernelError> {
        DateTime::parse_from_rfc3339(value)
            .map(|datetime| datetime.with_timezone(&Utc))
            .map_err(|source| KernelError::TimeConversionFailed {
                context: Some(serde_json::json!({ "value": value })),
                source: Some(Box::new(source)),
            })
    }

    /// تاریخ جلالی متناظر با یک زمان UTC را برمی‌گرداند.
    pub fn to_jalali_date(datetime: DateTime<Utc>) -> Result<ParsiDate, KernelError> {
        jalali::jalali_date_from_utc(datetime)
    }

    /// تاریخ جلالی قابل‌خواندن فارسی («۱۴۰۳/۰۵/۰۲») برای یک زمان UTC برمی‌گرداند.
    pub fn to_jalali_display(datetime: DateTime<Utc>) -> Result<String, KernelError> {
        let date = jalali::jalali_date_from_utc(datetime)?;
        Ok(jalali::format_jalali_date(date))
    }

    /// تاریخ و زمان جلالی قابل‌خواندن فارسی («۱۴۰۳/۰۵/۰۲ ۱۵:۳۰:۴۵») برمی‌گرداند.
    /// زمان نمایشی بر اساس ساعت تهران (UTC+3:30) است.
    pub fn jalali_datetime_display(datetime: DateTime<Utc>) -> Result<String, KernelError> {
        let date = jalali::jalali_date_from_utc(datetime)?;
        let tehran_time = datetime.with_timezone(&tehran_offset()).time();
        let time = jalali::to_persian_digits(&format!(
            "{:02}:{:02}:{:02}",
            tehran_time.hour(),
            tehran_time.minute(),
            tehran_time.second()
        ));
        Ok(format!("{} {}", jalali::format_jalali_date(date), time))
    }
}

/// یک نشست معاملاتی با بازه زمانی در ساعت تهران (UTC+3:30).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradingSession {
    /// کلید ماشین‌خوان نشست.
    pub key: &'static str,
    /// ساعت شروع.
    pub start_hour: u32,
    /// دقیقه شروع.
    pub start_minute: u32,
    /// ساعت پایان.
    pub end_hour: u32,
    /// دقیقه پایان.
    pub end_minute: u32,
}

impl TradingSession {
    /// آیا زمان داده‌شده در این نشست قرار دارد؟
    pub fn contains(&self, time: NaiveTime) -> bool {
        let start =
            NaiveTime::from_hms_opt(self.start_hour, self.start_minute, 0).unwrap_or_default();
        let end = NaiveTime::from_hms_opt(self.end_hour, self.end_minute, 0).unwrap_or_default();
        time >= start && time < end
    }
}

/// نشست‌های پیش‌فرض بازار در نسخه ۱ (ساعت تهران):
/// - `pre_market`: ۰۵:۳۰ تا ۰۹:۰۰
/// - `regular`: ۰۹:۰۰ تا ۱۲:۳۰
/// - `after_hours`: ۱۲:۳۰ تا ۱۵:۰۰
pub const DEFAULT_TRADING_SESSIONS: &[TradingSession] = &[
    TradingSession {
        key: "pre_market",
        start_hour: 5,
        start_minute: 30,
        end_hour: 9,
        end_minute: 0,
    },
    TradingSession {
        key: "regular",
        start_hour: 9,
        start_minute: 0,
        end_hour: 12,
        end_minute: 30,
    },
    TradingSession {
        key: "after_hours",
        start_hour: 12,
        start_minute: 30,
        end_hour: 15,
        end_minute: 0,
    },
];

/// انحراف ثابت ساعت تهران (UTC+3:30). ایران از سال ۱۴۰۱ ساعت تابستانی ندارد.
const TEHRAN_OFFSET_SECS: i32 = 3 * 3600 + 30 * 60;

fn tehran_offset() -> FixedOffset {
    FixedOffset::east_opt(TEHRAN_OFFSET_SECS).expect("Tehran UTC offset is valid")
}

/// نشست معاملاتی متناظر با یک زمان UTC را برمی‌گرداند.
pub fn classify_trading_session(datetime: DateTime<Utc>) -> Option<&'static TradingSession> {
    let tehran_time = datetime.with_timezone(&tehran_offset()).time();
    DEFAULT_TRADING_SESSIONS
        .iter()
        .find(|session| session.contains(tehran_time))
}

/// آیا زمان داده‌شده در تعطیلات آخر هفته بازار (جمعه) قرار دارد؟
pub fn is_market_weekend(datetime: DateTime<Utc>) -> bool {
    datetime.with_timezone(&tehran_offset()).weekday() == Weekday::Fri
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::error::Error as StdError;

    fn datetime(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .unwrap()
    }

    #[test]
    fn iso8601_roundtrip() {
        let now = TimeService::now_utc();
        let text = TimeService::to_iso8601(now);
        let parsed = TimeService::from_iso8601(&text).unwrap();
        assert_eq!(now.timestamp(), parsed.timestamp());
        assert_eq!(now.timestamp_nanos_opt(), parsed.timestamp_nanos_opt());
    }

    #[test]
    fn invalid_iso8601_is_rejected() {
        let error = TimeService::from_iso8601("not-a-date").unwrap_err();
        assert_eq!(error.code(), 1005);
        assert_eq!(error.variant(), "TimeConversionFailed");
        assert!(error.source().is_some() || StdError::source(&error).is_some());
    }

    #[test]
    fn jalali_conversion_of_known_dates() {
        // ۲۰۲۴-۰۳-۲۰ = ۱۴۰۳/۰۱/۰۱ (عین فلكی ۱۴۰۳ در تهران)
        let nowruz = datetime(2024, 3, 20, 12, 0);
        let jalali = TimeService::to_jalali_date(nowruz).unwrap();
        assert_eq!((jalali.year(), jalali.month(), jalali.day()), (1403, 1, 1));

        // ۲۰۲۴-۰۷-۲۳ = ۱۴۰۳/۰۵/۰۲
        let mordad = datetime(2024, 7, 23, 12, 0);
        assert_eq!(
            TimeService::to_jalali_display(mordad).unwrap(),
            "۱۴۰۳/۰۵/۰۲"
        );
    }

    #[test]
    fn jalali_datetime_display_uses_tehran_time() {
        // UTC ۲۰۲۴-۰۷-۲۳ ۱۲:۰۰ = تهران ۱۵:۳۰
        let datetime = datetime(2024, 7, 23, 12, 0);
        assert_eq!(
            TimeService::jalali_datetime_display(datetime).unwrap(),
            "۱۴۰۳/۰۵/۰۲ ۱۵:۳۰:۰۰"
        );
    }

    #[test]
    fn trading_session_classification() {
        // UTC ۰۳:۰۰ = تهران ۰۶:۳۰ → pre_market
        let pre = datetime(2024, 7, 23, 3, 0);
        assert_eq!(
            classify_trading_session(pre).map(|s| s.key),
            Some("pre_market")
        );

        // UTC ۰۵:۳۰ = تهران ۰۹:۰۰ → regular (مرز شامل شروع)
        let regular_start = datetime(2024, 7, 23, 5, 30);
        assert_eq!(
            classify_trading_session(regular_start).map(|s| s.key),
            Some("regular")
        );

        // UTC ۰۶:۳۰ = تهران ۱۰:۰۰ → regular
        let regular = datetime(2024, 7, 23, 6, 30);
        assert_eq!(
            classify_trading_session(regular).map(|s| s.key),
            Some("regular")
        );

        // UTC ۰۹:۰۰ = تهران ۱۲:۳۰ → after_hours (مرز شامل شروع)
        let after_start = datetime(2024, 7, 23, 9, 0);
        assert_eq!(
            classify_trading_session(after_start).map(|s| s.key),
            Some("after_hours")
        );

        // UTC ۱۰:۰۰ = تهران ۱۳:۳۰ → after_hours
        let after = datetime(2024, 7, 23, 10, 0);
        assert_eq!(
            classify_trading_session(after).map(|s| s.key),
            Some("after_hours")
        );

        // UTC ۱۲:۰۰ = تهران ۱۵:۳۰ → خارج از نشست‌ها
        let none = datetime(2024, 7, 23, 12, 0);
        assert!(classify_trading_session(none).is_none());

        // UTC ۰۲:۰۰ = تهران ۰۵:۳۰ → pre_market (مرز شامل شروع)
        let pre_start = datetime(2024, 7, 23, 2, 0);
        assert_eq!(
            classify_trading_session(pre_start).map(|s| s.key),
            Some("pre_market")
        );

        // UTC ۰۱:۵۹ = تهران ۰۵:۲۹ → خارج از نشست‌ها
        let before = datetime(2024, 7, 23, 1, 59);
        assert!(classify_trading_session(before).is_none());
    }

    #[test]
    fn market_weekend_is_friday() {
        // ۲۰۲۴-۰۷-۲۶ جمعه است
        assert!(is_market_weekend(datetime(2024, 7, 26, 12, 0)));
        // ۲۰۲۴-۰۷-۲۵ پنجشنبه است
        assert!(!is_market_weekend(datetime(2024, 7, 25, 12, 0)));
        // ۲۰۲۴-۰۷-۲۷ شنبه است
        assert!(!is_market_weekend(datetime(2024, 7, 27, 12, 0)));
    }

    #[test]
    fn session_contains_bounds() {
        let session = TradingSession {
            key: "test",
            start_hour: 9,
            start_minute: 0,
            end_hour: 12,
            end_minute: 30,
        };
        assert!(session.contains(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert!(session.contains(NaiveTime::from_hms_opt(12, 29, 59).unwrap()));
        assert!(!session.contains(NaiveTime::from_hms_opt(12, 30, 0).unwrap()));
        assert!(!session.contains(NaiveTime::from_hms_opt(8, 59, 59).unwrap()));
    }
}
