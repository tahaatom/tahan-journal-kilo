# قرارداد API پلاگین (Plugin RPC)

> وضعیت: فاز ۱.۲ — تثبیت‌شده.
> منبع اجرایی: `crates/aria-contracts/src/rpc.rs`

## پروتکل

- JSON-RPC 2.0 روی stdio یا IPC محلی
- پیام‌ها **خط‌محور** (newline-delimited JSON) هستند
- حداکثر اندازه پیام: **۱۰ مگابایت** (`MAX_MESSAGE_SIZE_BYTES`)
- تایم‌اوت پیش‌فرض درخواست: **۳۰ ثانیه** (`DEFAULT_REQUEST_TIMEOUT`)
- احراز هویت با **نشانه نشست** (`SessionToken`، UUID v4)؛ نشانه در لایه
  نشست به هر جریان اجرای پلاگین پیوند می‌خورد و مقایسه آن ثابت‌زمانی است
- پلاگین‌ها هرگز به پایگاه داده، حافظه کرنل، سیستم فایل یا پلاگین‌های
  دیگر دسترسی مستقیم ندارند

## ساختار پیام

### درخواست

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "trades.list",
  "params": { "filter": {}, "pagination": { "limit": 50, "offset": 0 } }
}
```

- `jsonrpc` باید دقیقاً `2.0` باشد؛ هر مقدار دیگر رد می‌شود
- `id` عدد یا رشته است؛ در صورت نبود یا null، پیام **اطلاع‌رسانی**
  (notification) است و پاسخی داده نمی‌شود
- `params` اختیاری است

### پاسخ موفق

```json
{ "jsonrpc": "2.0", "id": 1, "result": { "trades": [] } }
```

### خطا

```json
{ "jsonrpc": "2.0", "id": 1, "error": { "code": -32601, "message": "method not found" } }
```

## کدهای خطا

| کد | معنا |
|---|---|
| `-32700` | خطای تجزیه پیام |
| `-32600` | درخواست نامعتبر (از جمله نسخه پروتکل یا ساختار نامعتبر) |
| `-32601` | متد یافت نشد |
| `-32602` | پارامترها نامعتبر هستند |
| `-32603` | خطای داخلی |
| `-32000` | خطای منشأ‌گرفته از کرنل؛ ساختار `ErrorPayload` در `data` حمل می‌شود |

نمونه خطای کرنل:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "kernel error",
    "data": {
      "code": 5001,
      "variant": "TradeNotFound",
      "message_key": "error.trade.not_found",
      "context": null
    }
  }
}
```

ساختار `data` دقیقاً آینه‌ی خطای کرنل است (کد موتور، متغیر، کلید
پیام فارسی، متنیان)؛ برای جزئیات محدوده کدها به
`docs/contracts/error-codes.md` مراجعه کنید.

## دفترچه روش‌ها (Method Registry)

قرارداد دفترچه روش‌ها:

- نام متدها باید با قالب `domain.action` باشد
- هر متد یا **دقیقاً یک قابلیت مورد نیاز** دارد یا عمومی است
- **متدهای ناشناخته رد می‌شوند**
- **مجوزها به‌ازای هر متد اجرا می‌شوند**

فهرست متدهای نسخه ۱:

| متد | قابلیت مورد نیاز | توضیح |
|---|---|---|
| `kernel.ping` | — (عمومی) | بررسی سلامت کرنل |
| `kernel.version` | — (عمومی) | اطلاعات نسخه کرنل و قراردادها |
| `trades.list` | `trades.read` | فهرست معاملات با فیلتر و صفحه‌بندی |
| `trades.get` | `trades.read` | جزئیات یک معامله |
| `trades.create` | `trades.write` | ثبت معامله |
| `trades.update` | `trades.write` | ویرایش معامله |
| `trades.delete` | `trades.delete` | حذف معامله |
| `fields.list` | `fields.read` | فهرست فیلدهای سفارشی |
| `fields.get` | `fields.read` | جزئیات یک فیلد سفارشی |
| `fields.define` | `fields.define` | تعریف فیلد سفارشی جدید |
| `fields.update` | `fields.define` | ویرایش تعریف فیلد سفارشی |
| `fields.deactivate` | `fields.define` | غیرفعال‌سازی فیلد سفارشی |
| `attachments.list` | `attachments.read` | فهرست پیوست‌های یک معامله |
| `attachments.get` | `attachments.read` | دریافت متادیتا یا محتوای پیوست |
| `attachments.upload` | `attachments.write` | بارگذاری پیوست |
| `stats.summary` | `stats.read` | خلاصه آمار داشبورد |
| `stats.query` | `stats.read` | پرس‌وجوی تجمیعی |
| `backup.create` | `backup.create` | ایجاد پشتیبان |
| `backup.restore` | `backup.restore` | بازگردانی پشتیبان |
| `mt.import` | `mt.import` | واردات داده متاتریدر |
| `notifications.show` | `notifications.show` | نمایش اطلاع‌رسانی به کاربر |

قابلیت‌های منتقل‌شده (`mt.live`، `insights.write`، `network.access`،
`ui.widget` و `ui.page`) تا فازهای مربوطه متدی در این دفترچه ندارند.
متدهای مربوط به واردات متاتریدر با پلاگین `mt-import` در فاز ۱.۱۵
تکمیل می‌شوند.

## قوانین

- پلاگین فقط از طریق سرویس‌های رسمی کرنل (`PluginHostServices`) به
  داده دسترسی می‌یابد.
- `KeyProvider` و `EncryptionProvider` جزو سرویس‌های پلاگین **نیستند**؛
  پلاگین‌ها به کلیدها دسترسی ندارند.
- پیام‌های بزرگ‌تر از ۱۰ مگابایت **پیش از تجزیه** رد می‌شوند.
- پیام‌هایی با `jsonrpc` غیر از «2.0» یا ساختار نامعتبر رد می‌شوند.
- پاسخ به اطلاع‌رسانی‌ها ارسال نمی‌شود.
