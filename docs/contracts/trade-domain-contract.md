# قرارداد دامنه معاملاتی

> وضعیت: فاز ۱.۲ — تثبیت‌شده.
> پیاده‌سازی: فاز ۱.۶ (موتور دامنه، `aria-domain-engine`)

## موجودیت‌ها

- **Profile** — پروفایل کاربری
- **TradingAccount** — حساب معاملاتی
- **Symbol** — نماد
- **JournalTrade** — معامله ژورنال
- **EntryLeg** — لگ ورود
- **ExitLeg** — لگ خروج
- **Execution** — اجرا (بُرد واقعی بروکر)
- **SourceRecord** — رکورد مبدا (داده خام وارداتی)
- **ManualOverride** — دست‌نویس دستی
- **PositionGroup** — رزرو شده در نسخه ۱

## معامله ژورنال (Journal Trade) در نسخه ۱

- دقیقاً به **یک** حساب معاملاتی تعلق دارد
- دقیقاً به **یک** نماد تعلق دارد
- دقیقاً **یک** جهت دارد: خرید یا فروش
- می‌تواند **چندین Entry Leg** داشته باشد
- می‌تواند **چندین Exit Leg** داشته باشد
- می‌تواند **چندین Execution** داشته باشد
- دقیقاً **یک Position Group ضمنی** در نسخه ۱ دارد
- تصمیمات معاملاتی مستقل باید **Journal Trade جداگانه** باشند

## Position Group

- در نسخه ۱ **رزرو شده** (هر معامله یک گروه ضمنی دارد)
- در نسخه ۲ صریحاً پیاده می‌شود

## دستورهای نسخه ۱

قرارداد پاکت دستور و انواع آن در
`docs/contracts/command-contract.md` تثبیت شده است:

| دستور | نوع |
|---|---|
| CreateTradeCommand | `create_trade` |
| UpdateTradeCommand | `update_trade` |
| DeleteTradeCommand | `delete_trade` |
| AddEntryLegCommand | `add_entry_leg` |
| UpdateEntryLegCommand | `update_entry_leg` |
| AddExitLegCommand | `add_exit_leg` |
| UpdateExitLegCommand | `update_exit_leg` |
| AssignExecutionToLegCommand | `assign_execution_to_leg` |
| AddManualOverrideCommand | `add_manual_override` |
| RevertManualOverrideCommand | `revert_manual_override` |
| LinkAttachmentToTradeCommand | `link_attachment_to_trade` |

## رویدادهای نسخه ۱

قرارداد پاکت رویداد و انواع آن در
`docs/contracts/event-contract.md` تثبیت شده است:

| رویداد | نوع |
|---|---|
| TradeCreated | `trade_created` |
| TradeUpdated | `trade_updated` |
| TradeDeleted | `trade_deleted` |
| EntryLegAdded | `entry_leg_added` |
| ExitLegAdded | `exit_leg_added` |
| ExecutionAssigned | `execution_assigned` |
| OverrideAdded | `override_added` |
| OverrideReverted | `override_reverted` |
| StatsInvalidated | `stats_invalidated` |

## قوانین رویدادها

- رویدادها پس از کامیت منتشر می‌شوند.
- رویدادها برای مصرف‌کننده‌ها ایدمپوتنت هستند.
- رویدادهای حیاتی از طریق outbox ارسال می‌شوند.
- رویدادهای فقط-رابط‌کاربری در صورت نیاز ذخیره نمی‌شوند.
