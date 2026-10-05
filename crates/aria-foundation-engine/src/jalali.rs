//! کمک‌های تقویم جلالی برای نمایش فارسی.
//!
//! ذخیره‌سازی زمان همیشه UTC/ISO 8601 است؛ این ماژول فقط برای تبدیل و
//! قالب‌بندی نمایش فارسی/جلالی استفاده می‌شود.

use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use parsidate::ParsiDate;

use crate::error::KernelError;

/// نام ماه‌های جلالی به ترتیب از فروردین تا اسفند.
pub const JALALI_MONTH_NAMES: [&str; 12] = [
    "فروردین",
    "اردیبهشت",
    "خرداد",
    "تیر",
    "مرداد",
    "شهریور",
    "مهر",
    "آبان",
    "آذر",
    "دی",
    "بهمن",
    "اسفند",
];

/// نام روزهای هفته به فارسی، از شنبه (شاخص ۰) تا جمعه (شاخص ۶).
pub const JALALI_WEEKDAY_NAMES: [&str; 7] = [
    "شنبه",
    "یکشنبه",
    "دوشنبه",
    "سه‌شنبه",
    "چهارشنبه",
    "پنجشنبه",
    "جمعه",
];

/// یک تاریخ میلادی را به تاریخ جلالی تبدیل می‌کند.
pub fn gregorian_to_jalali(date: NaiveDate) -> Result<ParsiDate, KernelError> {
    ParsiDate::from_gregorian(date).map_err(|source| KernelError::TimeConversionFailed {
        context: Some(serde_json::json!({ "gregorian": date.to_string() })),
        source: Some(Box::new(source)),
    })
}

/// یک تاریخ جلالی را به تاریخ میلادی تبدیل می‌کند.
pub fn jalali_to_gregorian(date: ParsiDate) -> Result<NaiveDate, KernelError> {
    date.to_gregorian()
        .map_err(|source| KernelError::JalaliDateInvalid {
            context: Some(serde_json::json!({ "jalali": date.to_string() })),
            source: Some(Box::new(source)),
        })
}

/// تاریخ جلالی متناظر با یک زمان UTC را برمی‌گرداند.
pub fn jalali_date_from_utc(datetime: DateTime<Utc>) -> Result<ParsiDate, KernelError> {
    gregorian_to_jalali(datetime.date_naive())
}

/// آیا سال جلالی داده‌شده کبیسه است؟ (چرخه ۳۳ ساله)
pub fn is_jalali_leap_year(year: i32) -> bool {
    ParsiDate::is_persian_leap_year(year)
}

/// تعداد روزهای ماه جلالی داده‌شده.
pub fn days_in_jalali_month(year: i32, month: u32) -> u32 {
    ParsiDate::days_in_month(year, month)
}

/// نام ماه جلالی را بر اساس شماره ماه (۱ تا ۱۲) برمی‌گرداند.
pub fn jalali_month_name(month: u32) -> Option<&'static str> {
    if (1..=12).contains(&month) {
        Some(JALALI_MONTH_NAMES[(month - 1) as usize])
    } else {
        None
    }
}

/// نام روز هفته به فارسی را برای یک تاریخ جلالی برمی‌گرداند.
pub fn jalali_weekday(date: ParsiDate) -> Result<&'static str, KernelError> {
    let gregorian = jalali_to_gregorian(date)?;
    let index = match gregorian.weekday() {
        Weekday::Sat => 0,
        Weekday::Sun => 1,
        Weekday::Mon => 2,
        Weekday::Tue => 3,
        Weekday::Wed => 4,
        Weekday::Thu => 5,
        Weekday::Fri => 6,
    };
    Ok(JALALI_WEEKDAY_NAMES[index])
}

/// ارقام لاتین را در یک رشته به ارقام فارسی تبدیل می‌کند.
pub fn to_persian_digits(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '0' => '۰',
            '1' => '۱',
            '2' => '۲',
            '3' => '۳',
            '4' => '۴',
            '5' => '۵',
            '6' => '۶',
            '7' => '۷',
            '8' => '۸',
            '9' => '۹',
            _ => c,
        })
        .collect()
}

/// تاریخ جلالی را به‌صورت «۱۴۰۳/۰۵/۰۲» با ارقام فارسی قالب‌بندی می‌کند.
pub fn format_jalali_date(date: ParsiDate) -> String {
    to_persian_digits(&format!(
        "{:04}/{:02}/{:02}",
        date.year(),
        date.month(),
        date.day()
    ))
}

/// تاریخ جلالی را به‌صورت «۲ مرداد ۱۴۰۳» قالب‌بندی می‌کند.
pub fn format_jalali_date_long(date: ParsiDate) -> String {
    let month = jalali_month_name(date.month()).unwrap_or_default();
    to_persian_digits(&format!("{} {} {}", date.day(), month, date.year()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use parsidate::DateError;

    #[test]
    fn known_gregorian_to_jalali_conversions() {
        // ۲۰۲۴-۰۳-۲۰ = ۱۴۰۳/۰۱/۰۱ (عین فلكی ۱۴۰۳ در تهران)
        let nowruz = gregorian_to_jalali(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap()).unwrap();
        assert_eq!((nowruz.year(), nowruz.month(), nowruz.day()), (1403, 1, 1));
        // ۲۰۲۴-۰۳-۲۱ = ۱۴۰۳/۰۱/۰۲
        let day_after = gregorian_to_jalali(NaiveDate::from_ymd_opt(2024, 3, 21).unwrap()).unwrap();
        assert_eq!(
            (day_after.year(), day_after.month(), day_after.day()),
            (1403, 1, 2)
        );

        // ۲۰۲۴-۰۷-۲۳ = ۱۴۰۳/۰۵/۰۲
        let mordad = gregorian_to_jalali(NaiveDate::from_ymd_opt(2024, 7, 23).unwrap()).unwrap();
        assert_eq!((mordad.year(), mordad.month(), mordad.day()), (1403, 5, 2));

        // ۲۰۲۳-۰۳-۲۰ = ۱۴۰۱/۱۲/۲۹ (آخر سال غیرکبیسه)
        let esfand = gregorian_to_jalali(NaiveDate::from_ymd_opt(2023, 3, 20).unwrap()).unwrap();
        assert_eq!(
            (esfand.year(), esfand.month(), esfand.day()),
            (1401, 12, 29)
        );
    }

    #[test]
    fn jalali_to_gregorian_roundtrip() {
        let date = gregorian_to_jalali(NaiveDate::from_ymd_opt(2024, 7, 23).unwrap()).unwrap();
        let back = jalali_to_gregorian(date).unwrap();
        assert_eq!(back, NaiveDate::from_ymd_opt(2024, 7, 23).unwrap());

        // ۱۴۰۳/۱۲/۳۰ (روز کبیسه) = ۲۰۲۵-۰۳-۲۰
        let leap = ParsiDate::new(1403, 12, 30).unwrap();
        assert_eq!(
            jalali_to_gregorian(leap).unwrap(),
            NaiveDate::from_ymd_opt(2025, 3, 20).unwrap()
        );
    }

    #[test]
    fn invalid_jalali_date_is_rejected() {
        let invalid = ParsiDate::new(1404, 12, 30).unwrap_err();
        assert!(matches!(invalid, DateError::InvalidDate));
        let unsafe_invalid = unsafe { ParsiDate::new_unchecked(1404, 12, 30) };
        assert!(jalali_to_gregorian(unsafe_invalid).is_err());
    }

    #[test]
    fn leap_year_and_month_length() {
        assert!(is_jalali_leap_year(1403));
        assert!(!is_jalali_leap_year(1404));
        assert!(is_jalali_leap_year(1399));
        assert_eq!(days_in_jalali_month(1403, 1), 31);
        assert_eq!(days_in_jalali_month(1403, 7), 30);
        assert_eq!(days_in_jalali_month(1403, 12), 30);
        assert_eq!(days_in_jalali_month(1404, 12), 29);
        assert_eq!(days_in_jalali_month(1403, 13), 0);
    }

    #[test]
    fn month_names_lookup() {
        assert_eq!(jalali_month_name(1), Some("فروردین"));
        assert_eq!(jalali_month_name(5), Some("مرداد"));
        assert_eq!(jalali_month_name(12), Some("اسفند"));
        assert_eq!(jalali_month_name(0), None);
        assert_eq!(jalali_month_name(13), None);
        assert_eq!(JALALI_MONTH_NAMES.len(), 12);
    }

    #[test]
    fn weekday_names() {
        // ۱۴۰۳/۰۵/۰۲ = سه‌شنبه
        let date = ParsiDate::new(1403, 5, 2).unwrap();
        assert_eq!(jalali_weekday(date).unwrap(), "سه‌شنبه");
        // ۱۴۰۳/۰۱/۰۴ = شنبه
        let saturday = ParsiDate::new(1403, 1, 4).unwrap();
        assert_eq!(jalali_weekday(saturday).unwrap(), "شنبه");
        // ۱۴۰۳/۰۱/۱۰ = جمعه
        let friday = ParsiDate::new(1403, 1, 10).unwrap();
        assert_eq!(jalali_weekday(friday).unwrap(), "جمعه");
    }

    #[test]
    fn persian_digit_conversion() {
        assert_eq!(to_persian_digits("2024/03/21"), "۲۰۲۴/۰۳/۲۱");
        assert_eq!(to_persian_digits("R 1,234.5"), "R ۱,۲۳۴.۵");
        assert_eq!(to_persian_digits(""), "");
    }

    #[test]
    fn jalali_date_formatting() {
        let date = ParsiDate::new(1403, 5, 2).unwrap();
        assert_eq!(format_jalali_date(date), "۱۴۰۳/۰۵/۰۲");
        assert_eq!(format_jalali_date_long(date), "۲ مرداد ۱۴۰۳");
        let first = ParsiDate::new(1403, 1, 1).unwrap();
        assert_eq!(format_jalali_date(first), "۱۴۰۳/۰۱/۰۱");
        assert_eq!(format_jalali_date_long(first), "۱ فروردین ۱۴۰۳");
    }
}
