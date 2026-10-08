//! قالب زمان بومی ذخیره‌سازی.
//!
//! قرارداد (فاز ۱.۱/۱.۳): زمان همیشه UTC و ISO 8601 ذخیره می‌شود. برای
//! پایدارسازی **ترتیب واژگانی** در فیلترهای بازه‌ای و ایندکس‌های زمانی،
//! لایه ذخیره‌سازی از قالب ثابت با دقت میلی‌ثانیه و پسوند `Z` استفاده می‌کند:
//!
//! ```text
//! YYYY-MM-DDTHH:MM:SS.mmmZ
//! ```
//!
//! این قالب هم‌عرض است و مقایسه رشته‌ای آن دقیقاً با مقایسه زمانی یکی است.
//! تبدیل و قالب‌بندی نمایشی (جلالی) مسئولیت موتور پایه (`TimeService`) است.

use chrono::{DateTime, SecondsFormat, Utc};

/// زمان فعلی UTC.
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// زمان را به قالب بومی ذخیره‌سازی تبدیل می‌کند.
pub fn format(datetime: DateTime<Utc>) -> String {
    datetime.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// یک رشته زمان ذخیره‌شده را به `DateTime<Utc>` تبدیل می‌کند.
///
/// هم قالب بومی (`…Z`) و هم دیگر شکل‌های معتبر RFC 3339 (مثل `+00:00`)
/// پذیرفته می‌شوند تا داده‌های نوشته‌شده توسط ابزارهای دیگر نیز خوانده شوند.
pub fn parse(value: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(value).map(|datetime| datetime.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_is_fixed_width_utc_with_z() {
        let datetime = DateTime::parse_from_rfc3339("2026-10-05T03:04:05.678Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(format(datetime), "2026-10-05T03:04:05.678Z");
        assert_eq!(format(datetime).len(), 24);
    }

    #[test]
    fn format_is_lexicographically_sortable() {
        let earlier = DateTime::parse_from_rfc3339("2026-10-05T03:04:05.000Z")
            .unwrap()
            .with_timezone(&Utc);
        let later = DateTime::parse_from_rfc3339("2026-10-05T03:04:05.001Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(format(earlier) < format(later));
    }

    #[test]
    fn parse_roundtrips_both_utc_forms() {
        let value = parse("2026-10-05T03:04:05.678Z").unwrap();
        assert_eq!(format(value), "2026-10-05T03:04:05.678Z");
        let offset_form = parse("2026-10-05T03:04:05.678+00:00").unwrap();
        assert_eq!(offset_form, value);
    }
}
