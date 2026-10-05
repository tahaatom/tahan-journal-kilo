# قرارداد دستور (Command Envelope)

> وضعیت: فاز ۱.۲ — تثبیت‌شده.
> منبع اجرایی: `crates/aria-contracts/src/command.rs`

## ساختار پاکت دستور

| عضو | نوع | الزامی | توضیح |
|---|---|---|---|
| `command_id` | UUID v4 | بله | شناسه یکتای دستور |
| `command_type` | `CommandType` | بله | نوع دستور (سطح نسخه ۱ بسته است) |
| `payload` | JSON | بله | بار دستور |
| `issuer` | `CommandIssuer` | بله | صادرکننده دستور |
| `timestamp` | `DateTime<Utc>` | بله | زمان صدور (ISO 8601، UTC) |
| `correlation_id` | UUID v4 | خیر | شناسه همبستگی برای ردیابی زنجیره دستور-رویداد |

## نمونه JSON

```json
{
  "command_id": "6f1c2a3e-9b1d-4f2e-8a3c-1e2d3f4a5b6c",
  "command_type": "create_trade",
  "payload": { "symbol": "XAUUSD", "direction": "buy" },
  "issuer": { "kind": "user", "principal_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6" },
  "timestamp": "2026-10-05T03:00:00Z",
  "correlation_id": "9b1d4f2e-8a3c-6f1c-2a3e-1e2d3f4a5b6c"
}
```

در صورت نبود `correlation_id`، عضو در JSON حذف می‌شود (نه null).

## انواع دستور در نسخه ۱

| نوع | رشته پایدار |
|---|---|
| `CreateTrade` | `create_trade` |
| `UpdateTrade` | `update_trade` |
| `DeleteTrade` | `delete_trade` |
| `AddEntryLeg` | `add_entry_leg` |
| `UpdateEntryLeg` | `update_entry_leg` |
| `AddExitLeg` | `add_exit_leg` |
| `UpdateExitLeg` | `update_exit_leg` |
| `AssignExecutionToLeg` | `assign_execution_to_leg` |
| `AddManualOverride` | `add_manual_override` |
| `RevertManualOverride` | `revert_manual_override` |
| `LinkAttachmentToTrade` | `link_attachment_to_trade` |

سطح دستورهای نسخه ۱ بسته است و دقیقاً با دستورهای قرارداد دامنه
(`docs/contracts/trade-domain-contract.md`، پیاده‌سازی در فاز ۱.۶) یکی
است. افزودن دستور جدید فقط از طریق ارتقاء نسخه‌بندی‌شده قرارداد مجاز
است؛ نوع ناشناخته در مرز ورودی رد می‌شود.

## صادرکننده

- `kind`:
  - `user` — کاربر انسانی از طریق رابط کاربری
  - `system` — خود کرنل
  - `plugin` — پلاگین از طریق RPC
- `principal_id`:
  - برای `user`: شناسه پروفایل (UUID)
  - برای `system`: رشته ثابت `kernel`
  - برای `plugin`: شناسه پلاگین

## قوانین

- دستورات از طریق IPC امن Tauri صادر می‌شوند.
- دستورات اتمیک هستند: یا کامل اجرا می‌شوند یا هیچ اثری ندارند.
- هر دستور پس از اجرای موفق، رویدادهای دامنه‌ای منتشر می‌کند.
- `correlation_id` زنجیره دستور ← رویدادهای ناشی را قابل ردیابی می‌کند.
- زمان‌ها همیشه UTC و ISO 8601 ذخیره می‌شوند.
- انواع دستور ناشناخته باید در مرز ورودی رد شوند.
