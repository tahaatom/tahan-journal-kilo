# وابستگی‌های موتورها

> وضعیت: فاز ۰ — تثبیت‌شده طبق ماستر پرامپت.

## سطوح

| سطح | موتورها |
|---|---|
| ۰ | aria-contracts، aria-foundation-engine |
| ۱ | aria-storage-engine، aria-security-engine |
| ۲ | aria-schema-engine، aria-domain-engine |
| ۳ | aria-plugin-engine، aria-query-engine |
| ۴ | aria-runtime-engine، aria-ui-engine |

## قوانین

- موتورها فقط به موتورهای سطح پایین‌تر و قراردادهای مشترک وابسته می‌شوند.
- هیچ موتوری به داده یا جداول خصوصی موتور دیگر دسترسی ندارد.
- ارتباط فقط از طریق تریت‌های عمومی و قراردادهای نسخه‌بندی‌شده انجام می‌شود.
- هر موتور Cargo.toml، تست، README و مستندات خود را دارد.
- افزودن موتور جدید یا لایه انتزاعی جدید بدون تأیید صریح ممنوع است.
