# قرارداد رویداد (Event Envelope)

> وضعیت: فاز ۱.۲ — تثبیت‌شده.
> منبع اجرایی: `crates/aria-contracts/src/event.rs`

## ساختار پاکت رویداد

| عضو | نوع | الزامی | توضیح |
|---|---|---|---|
| `event_id` | UUID v4 | بله | شناسه یکتای رویداد |
| `event_type` | `EventType` | بله | نوع رویداد (سطح نسخه ۱ بسته است) |
| `event_version` | عدد صحیح | بله | نسخه اصلی شکل بار رویداد |
| `source` | `EventSource` | بله | منبع تولید رویداد |
| `timestamp` | `DateTime<Utc>` | بله | زمان وقوع (ISO 8601، UTC) |
| `correlation_id` | UUID v4 | خیر | شناسه همبستگی با دستور یا رویداد مبدأ |
| `payload` | JSON | بله | بار رویداد |

## نمونه JSON

```json
{
  "event_id": "1e2d3f4a-5b6c-4d2e-9f1c-2a3e4b5c6d7e",
  "event_type": "trade_created",
  "event_version": 1,
  "source": { "engine": "domain", "component": "trade_service" },
  "timestamp": "2026-10-05T03:00:01Z",
  "correlation_id": "6f1c2a3e-9b1d-4f2e-8a3c-1e2d3f4a5b6c",
  "payload": { "trade_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6" }
}
```

## انواع رویداد در نسخه ۱

| نوع | رشته پایدار |
|---|---|
| `TradeCreated` | `trade_created` |
| `TradeUpdated` | `trade_updated` |
| `TradeDeleted` | `trade_deleted` |
| `EntryLegAdded` | `entry_leg_added` |
| `ExitLegAdded` | `exit_leg_added` |
| `ExecutionAssigned` | `execution_assigned` |
| `OverrideAdded` | `override_added` |
| `OverrideReverted` | `override_reverted` |
| `StatsInvalidated` | `stats_invalidated` |

سطح رویدادهای نسخه ۱ بسته است و دقیقاً با رویدادهای قرارداد دامنه
(پیاده‌سازی در فاز ۱.۶) یکی است. نوع ناشناخته در مرز ورودی رد می‌شود.

## منبع رویداد

- `engine`: یکی از موتورهای کرنل — `foundation`، `storage`، `security`،
  `schema`، `domain`، `plugin`، `runtime`، `query`، `ui`
- `component`: جزء اختیاری داخلی موتور (مثلاً `trade_service`)

## قوانین

- رویدادها **پس از کامیت** منتشر می‌شوند.
- مصرف‌کننده‌ها باید **ایدمپوتنت** باشند.
- رویدادهای حیاتی از طریق **outbox** ارسال می‌شوند.
- رویدادهای فقط-رابط‌کاربری در صورت نیاز ذخیره نمی‌شوند.
- نسخه‌بندی رویدادها طبق `docs/contracts/versioning-policy.md` است:
  تغییرات افزایشی نسخه را تغییر نمی‌دهد، تغییر شکست‌زا نسخه اصلی
  را می‌افزاید و نسخه قبلی تا پایان دوره منقضی‌سازی پشتیبانی می‌شود.
