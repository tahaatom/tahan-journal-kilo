# کدهای خطا

> وضعیت: فاز ۱.۲ — محدوده کدهای همه موتورها و ساختار خطای RPC تثبیت شده.
> کدهای اختصاصی هر موتور در فاز مربوطه تثبیت می‌شود.

## محدوده‌های کدهای به تفکیک موتور

| موتور | محدوده | وضعیت |
|---|---|---|
| foundation | 1000–1999 | تثبیت‌شده در فاز ۱.۱ |
| storage | 2000–2999 | رزروشده (فاز ۱.۳) |
| security | 3000–3999 | رزروشده (فاز ۱.۴) |
| schema | 4000–4999 | رزروشده (فاز ۱.۵) |
| domain | 5000–5999 | رزروشده (فاز ۱.۶) |
| plugin | 6000–6999 | رزروشده (فاز ۱.۷) |
| runtime | 7000–7999 | رزروشده (فاز ۱.۸) |
| query | 8000–8999 | رزروشده (فاز ۱.۹) |
| ui | 9000–9999 | رزروشده (فاز ۱.۱۰) |

منبع اجرایی محدوده‌ها: `EngineId::error_range()` در
`crates/aria-contracts/src/error.rs`. محدوده‌ها هم‌پوشانی ندارند و
کل بازه ۱۰۰۰ تا ۹۹۹۹ را پوشش می‌دهند.

## کدهای موتور پایه (foundation)

منبع واحد این جدول، ثابت `ALL_ERROR_DEFINITIONS` در
`crates/aria-foundation-engine/src/error.rs` است.

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

## ساختار خطای کرنل

هر خطای کرنل دارای چهار جزء پایدار است:

- کد عددی یکتا در محدوده موتور
- نام متغیر قابل‌خواندن ماشینی
- کلید پیام فارسی قابل‌خواندن برای کاربر (قالب نقطه‌ای `error.<area>.<name>`،
  سازگار با ساختار i18next فرانت‌اند)
- متنیان اختیاری به‌صورت JSON
- خطای منبع اختیاری (`std::error::Error`)

## ساختار خطای RPC

انتقال خطای کرنل در RPC و مرزهای سرویس از ساختار `ErrorPayload` استفاده
می‌کند (`crates/aria-contracts/src/error.rs`):

```json
{
  "code": 5001,
  "variant": "TradeNotFound",
  "message_key": "error.trade.not_found",
  "context": null
}
```

- این ساختار آینه‌ی چهار جزء پایدار خطای کرنل است (بدون خطای منبع،
  چون خطای منبع قابل سریالیزه‌سازی نیست)
- هر موتور خطای خود را در مرز خود به این ساختار تبدیل می‌کند
- در پاسخ خطای JSON-RPC، این ساختار با کد `-32000` در عضو `data`
  حمل می‌شود (جزئیات در `docs/contracts/plugin-api.md`)
- موتور مبدأ از محدوده کد قابل استنتاج است (`ErrorPayload::engine()`)

## کدهای خطای استاندارد JSON-RPC 2.0

| کد | ثابت | معنا |
|---|---|---|
| -32700 | `PARSE_ERROR` | خطای تجزیه پیام |
| -32600 | `INVALID_REQUEST` | درخواست نامعتبر |
| -32601 | `METHOD_NOT_FOUND` | متد یافت نشد |
| -32602 | `INVALID_PARAMS` | پارامترها نامعتبر |
| -32603 | `INTERNAL_ERROR` | خطای داخلی |
| -32000 | `KERNEL_ERROR` | خطای کرنل با `ErrorPayload` در `data` |

منبع اجرایی: ماژول `rpc_error_code` در `crates/aria-contracts/src/error.rs`.

## قوانین

- هر کد عددی دقیقاً به یک خطا تعلق دارد؛ تکراری‌سازی کدها در تست‌های
  هر موتور بررسی می‌شود.
- `anyhow` فقط در مرزهای اپلیکیشن استفاده می‌شود، نه در API عمومی موتورها.
- خطاهای موتور پایه `Send + Sync` هستند تا در مرزهای Tauri و async
  قابل استفاده باشند.
- کدهای تثبیت‌شده در یک نسخه اصلی (major) دست‌نخورده باقی می‌مانند؛
  افزودن کد جدید مجاز است.
