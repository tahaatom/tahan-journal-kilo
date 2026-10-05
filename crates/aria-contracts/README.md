# aria-contracts

قراردادهای عمومی کرنل آریا — انواع، تریت‌ها، اسکیماها و نسخه‌بندی.

## وضعیت

فاز ۱.۲ — قراردادها تثبیت شده. این کریت فقط قرارداد تعریف می‌کند
و هیچ منطق کسب‌وکاری ندارد.

## ماژول‌ها

| ماژول | محتوا |
|---|---|
| `command` | پاکت دستور (Command Envelope) و انواع دستورهای نسخه ۱ |
| `event` | پاکت رویداد (Event Envelope) و انواع رویدادهای نسخه ۱ |
| `manifest` | قرارداد مانیفست پلاگین و اسکیمای JSON آن (تعبیه‌شده) |
| `rpc` | قرارداد RPC پلاگین (JSON-RPC 2.0، دفترچه روش‌ها، نشانه نشست) |
| `services` | تریت‌های سرویس‌های کرنل (۱۰ سرویس) |
| `error` | محدوده کدهای خطا به‌ازای هر موتور و ساختار خطای RPC |
| `versioning` | سیاست نسخه‌بندی و کمک‌کننده‌های سازگاری |

## وابستگی

سطح ۰ — فقط وابستگی‌های خط پایه (serde، serde_json، uuid، chrono،
semver، zeroize)؛ بدون وابستگی به سایر موتورها.

## تست‌ها

- تست‌های یکپارچه اسکیمای مانیفست (`tests/manifest_schema.rs`)
- تست‌های یکپارچه پاکت‌های دستور و رویداد (`tests/envelopes.rs`)
- تست‌های سازگاری نسخه‌بندی (`tests/versioning.rs`)

## مستندات قراردادی

- `docs/contracts/command-contract.md`
- `docs/contracts/event-contract.md`
- `docs/contracts/plugin-manifest.schema.json`
- `docs/contracts/plugin-api.md`
- `docs/contracts/error-codes.md`
- `docs/contracts/versioning-policy.md`
- `docs/contracts/trade-domain-contract.md`
- `docs/contracts/r-calculation-contract.md`
- `docs/contracts/execution-leg-contract.md`
