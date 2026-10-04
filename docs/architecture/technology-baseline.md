# خط پایه فناوری

> نسخه: فاز ۰ — تاریخ: ۲۰۲۶-۱۰-۰۴

## نسخه‌های دقیق و مجاز

| فناوری | نسخه پایه | نسخه تست‌شده در محیط |
|---|---|---|
| Rust | stable (MSRV 1.75) | 1.97.1 |
| Node.js | 22 LTS یا 24 LTS | 24.18.0 |
| npm | 10+ | 12.0.2 |
| Python (سایدکار) | 3.11+ | 3.14.6 |
| Tauri | 2.x | 2.x |
| React | 18+ | 18.3.x |
| TypeScript | 5.4+ (strict) | 5.9.x |
| Vite | 5+ | 8.x |
| Tailwind CSS | 4.x | 4.3.x |
| SQLite | رمزنگاری‌شده، سازگار با SQLCipher | — |

## کتابخانه‌های تأییدشده

### بک‌اند و کرنل (Rust)

tokio، serde، serde_json، thiserror، anyhow، tracing، tracing-subscriber، tracing-appender، rusqlite، refinery، uuid، chrono، کتابخانه جلالی سازگار با chrono، semver، jsonschema، sysinfo، interprocess یا IPC مبتنی بر stdio، argon2، aes-gcm، zeroize، secrecy، blake3، zstd، image، csv، encoding_rs، directories یا dirs

### فرانت‌اند

React 18+، TypeScript strict، Vite، Tailwind CSS، Radix UI یا Headless UI، React Hook Form، Zod، @hookform/resolvers، TanStack Table/Virtual/Query، Zustand یا Jotai، Apache ECharts، i18next، react-i18next، date-fns-jalali یا افزونه جلالی dayjs، فونت Vazirmatn، Vitest، React Testing Library، Playwright یا tauri-driver

### سایدکار پایتون

polars، pandas، numpy، scipy، matplotlib؛ در نسخه ۲ برای گزارش‌های پیشرفته: fpdf2، arabic_reshaper، python-bidi

## کتابخانه‌های ممنوعه

- هر کتابخانه‌ای که در فهرست تأییدشده نیست، بدون تأیید صریح ممنوع است.
- موتورهای فرمول اجرای کد دلخواه (eval) ممنوع.
- کلاینت‌های شبکه پیش‌فرض ممنوع (آفلاین-فرست).
- دسترسی مستقیم پلاگین‌ها به SQLite ممنوع.
- وابستگی‌های سنگین بدون تأیید صریح ممنوع.

## سیاست ارتقا

- نسخه‌های patch و minor: در CI تست می‌شوند و با بازبینی معمولی اعمال می‌شوند.
- نسخه‌های major: یک‌باره و پس از بازبینی قراردادها؛ هرگز چند ارتقای major هم‌زمان.
- هر ارتقای major باید سازگاری بازگشتی قراردادهای عمومی را حفظ کند.
- تغییر فناوری پایه نیازمند ثبت ADR در docs/architecture و به‌روزرسانی این سند است.

## سیاست مهاجرت برای تغییرات فناوری آینده

1. ثبت دلیل، جایگزین‌ها و تأثیر بر قراردادها.
2. مهاجرت تدریجی و فازی؛ بدون بازنویسی هم‌زمان چند موتور.
3. حفظ نسخه‌بندی قراردادها و سیاست منقضی‌سازی صریح.
4. تست بازگشتی کامل پیش از ادغام.
