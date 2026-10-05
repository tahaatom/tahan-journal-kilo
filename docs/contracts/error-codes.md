# کدهای خطا

> وضعت: فاز ۱.۱ — محدوده و کدهای موتور پایه (foundation) تثبیت شده است.
> کدهای موتورهای دیگر در فازهای مربوطه تثبیت می‌شوند.

## محدوده‌های کدهای به تفکیک موتور

| موتور | محدوده | وضعیت |
|---|---|---|
| foundation | 1000–1999 | تثبیت‌شده در فاز ۱.۱ |
| storage | 2000–2999 | پیشنهادی |
| security | 3000–3999 | پیشنهادی |
| schema | 4000–4999 | پیشنهادی |
| domain | 5000–5999 | پیشنهادی |
| plugin | 6000–6999 | پیشنهادی |
| runtime | 7000–7999 | پیشنهادی |
| query | 8000–8999 | پیشنهادی |
| ui | 9000–9999 | پیشنهادی |

## کدهای موتور پایه (foundation)

منبع واحد این جدول، ثابت `ALL_ERROR_DEFINITIONS` در `crates/aria-foundation-engine/src/error.rs` است.

| کد | متغیر | کلید پیام |
|---|---|---|
| 1001 | ConfigLoadFailed | error.config.load_failed |
| 1002 | ConfigSaveFailed | error.config.save_failed |
| 1003 | ConfigInvalid | error.config.invalid |
| 1004 | ConfigPathInvalid | error.config.path_invalid |
| 1005 | TimeConversionFailed | error.time.conversion_failed |
| 1006 | JalaliDateInvalid | error.jalali.date_invalid |
| 1007 | LogInitFailed | error.logging.init_failed |
| 1008 | IdGenerationFailed | error.ids.generation_failed |
| 1009 | InvalidArgument | error.invalid_argument |
| 1010 | DirectoryCreationFailed | error.io.directory_creation_failed |
| 1011 | SerializationFailed | error.serialization_failed |

## ساختار خطا

هر خطای کرنل دارای چهار جزء پایدار است:

- کد عددی یکتا در محدوده موتور
- نام متغیر قابل‌خواندن ماشینی
- کلید پیام فارسی قابل‌خواندن برای کاربر (قالب نقطه‌ای `error.<area>.<name>`، سازگار با ساختار i18next فرانت‌اند)
- متنیان اختیاری به‌صورت JSON
- خطای منبع اختیاری (`std::error::Error`)

## قوانین

- هر کد عددی دقیقاً به یک خطا تعلق دارد؛ تکراری‌سازی کدها در تست `error_codes_are_unique_and_within_foundation_range` بررسی می‌شود.
- `anyhow` فقط در مرزهای اپلیکیشن استفاده می‌شود، نه در API عمومی موتورها.
- خطاهای موتور پایه `Send + Sync` هستند تا در مرزهای Tauri و async قابل استفاده باشند.
- کدهای تثبیت‌شده در یک نسخه اصلی (major) دست‌نخورده باقی می‌مانند؛ افزودن کد جدید مجاز است.
