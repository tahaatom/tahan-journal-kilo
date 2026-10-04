# اسکیمای دیتابیس

> وضعیت: فاز ۰ — ساختار اولیه. پیاده‌سازی در فاز ۱.۳.

## اصول

- SQLite رمزنگاری‌شده (سازگار با SQLCipher)
- WAL فعال، foreign_keys فعال، busy_timeout تنظیم‌شده
- مهاجرت‌ها نسخه‌بندی‌شده، مرتب و قابل تست
- داده خام غیرقابل تغییر

## جداول نسخه ۱ (پیش‌نمایش)

profiles، trading_accounts، symbols، journal_trades، entry_legs، exit_legs، executions، source_records، manual_overrides، custom_fields، field_options، field_values، attachments، attachment_trade_links، audit_logs، system_events، plugins، settings و جداول خلاصه داشبورد
