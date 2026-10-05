//! تریت‌های سرویس‌های کرنل.
//!
//! این تریت‌ها مرز رسمی بین موتورها و پلاگین‌هاست. پلاگین‌ها فقط از
//! طریق [`PluginHostServices`] و فقط به سرویس‌هایی که مجوز دارند
//! دسترسی می‌یابند. [`KeyProvider`] و [`EncryptionProvider`] صرفاً
//! داخلی هستند و هرگز به پلاگین‌ها افشا نمی‌شوند (پلاگین‌ها به
//! کلیدها دسترسی ندارند).
//!
//! سرویس‌ها در نسخه ۱ همگام (sync) هستند؛ مرزهای async با
//! `spawn_blocking` آن‌ها را فرا می‌خوانند. امضاهای حوزه‌ای تایپ‌شده
//! با موتور دامنه (فاز ۱.۶) نهایی می‌شوند؛ در حال حاضر بارهای
//! حوزه‌ای به‌صورت JSON هستند.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::error::ErrorPayload;

/// بایت‌های محرمانه که در رها شدن صفر می‌شوند.
///
/// ماده کلیدی و داده‌های رمزگشایی‌شده فقط از این نوع عبور می‌کنند تا
/// مدت زمان حضور راز در حافظه کمینه شود (آیین‌نامه امنیتی کرنل).
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    /// ماده محرمانه جدید.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// دسترسی فقط-خواندنی به ماده محرمانه.
    pub fn expose(&self) -> &[u8] {
        &self.0
    }

    /// طول ماده محرمانه.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// آیا ماده محرمانه خالی است؟
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// ماده خام را برمی‌دارد؛ فراخواننده مسئول صفرسازی آن است.
    pub fn into_inner(mut self) -> Vec<u8> {
        std::mem::take(&mut self.0)
    }
}

impl Drop for SecretBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl From<Vec<u8>> for SecretBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretBytes")
            .field("len", &self.0.len())
            .finish_non_exhaustive()
    }
}

/// مقصد استفاده از کلید.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyPurpose {
    /// رمزنگاری پایگاه داده.
    Database,
    /// رمزنگاری پشتیبان.
    Backup,
}

/// بسته رمزنگاری‌شده (قالب بسته رمزنگاری کرنل).
///
/// قالب: magic، format_version، kdf_identifier، kdf_parameters، salt،
/// cipher_identifier، nonce، authenticated_metadata، ciphertext،
/// authentication_tag. فیلدهای باینری به‌صورت رشته base64 هستند.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncryptedPackage {
    /// شناسه جادویی قالب (مثلاً `ARIA`).
    pub magic: String,
    /// نسخه قالب بسته.
    pub format_version: u32,
    /// شناسه KDF (مثلاً `argon2id`).
    pub kdf_identifier: String,
    /// پارامترهای KDF.
    pub kdf_parameters: Value,
    /// نمک KDF (base64).
    pub salt: String,
    /// شناسه رمز (مثلاً `aes-256-gcm`).
    pub cipher_identifier: String,
    /// nonce (base64).
    pub nonce: String,
    /// متادیتای احرازشده (اختیاری).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authenticated_metadata: Option<Value>,
    /// متن رمزنگاری‌شده (base64).
    pub ciphertext: String,
    /// برچسب احراز (base64).
    pub authentication_tag: String,
}

/// صفحه‌بندی استاندارد پرس‌وجوها.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pagination {
    /// حداکثر تعداد رکورد برگردانده‌شده.
    pub limit: u32,
    /// تعداد رکوردهای ردشده (offset).
    pub offset: u32,
}

impl Pagination {
    /// صفحه‌بندی با مقادیر مشخص.
    pub fn new(limit: u32, offset: u32) -> Self {
        Self { limit, offset }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

/// نتیجه عملیات حساس در لاگ حسابرسی.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure,
}

/// ورودی لاگ حسابرسی.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// عملیات حساس (مثلاً `login`، `backup.create`، `trade.delete`).
    pub operation: String,
    /// مجری عملیات.
    pub actor: String,
    /// نتیجه عملیات.
    pub outcome: AuditOutcome,
    /// زمان وقوع (UTC).
    pub timestamp: DateTime<Utc>,
    /// جزئیات اختیاری (بدون راز و بدون بار مالی کامل کاربر).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// متادیتای پیوست.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachmentInfo {
    /// شناسه پیوست.
    pub id: Uuid,
    /// نام فایل.
    pub filename: String,
    /// نوع محتوا.
    pub content_type: String,
    /// اندازه (بایت).
    pub size_bytes: u64,
    /// هش blake3 برای یکپارچگی و تشخیص تکراری.
    pub blake3_hash: String,
    /// زمان ایجاد (UTC).
    pub created_at: DateTime<Utc>,
}

/// متادیتای پشتیبان.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupInfo {
    /// شناسه پشتیبان.
    pub id: Uuid,
    /// زمان ایجاد (UTC).
    pub created_at: DateTime<Utc>,
    /// نسخه فرمت پشتیبان.
    pub format_version: u32,
    /// اندازه بسته (بایت).
    pub size_bytes: u64,
    /// دلیل ایجاد.
    pub reason: String,
}

/// سرویس مدیریت کلید (موتور امنیت).
///
/// **فقط داخلی است:** پلاگین‌ها هرگز به این سرویس دسترسی ندارند.
pub trait KeyProvider: Send + Sync {
    /// مشتق یا بازیابی کلید فعال برای پروفایل و مقصد.
    fn derive_key(
        &self,
        profile_id: Uuid,
        purpose: KeyPurpose,
        salt: &[u8],
    ) -> Result<SecretBytes, ErrorPayload>;

    /// چرخش کلید (قلاب برای نسخه‌های بعدی).
    fn rotate_key(&self, profile_id: Uuid, purpose: KeyPurpose) -> Result<(), ErrorPayload>;

    /// اثر انگشت کلید فعال برای نمایش بدون افشای کلید.
    fn key_fingerprint(
        &self,
        profile_id: Uuid,
        purpose: KeyPurpose,
    ) -> Result<String, ErrorPayload>;
}

/// سرویس رمزنگاری (موتور امنیت).
///
/// **فقط داخلی است:** پلاگین‌ها هرگز به این سرویس دسترسی ندارند.
pub trait EncryptionProvider: Send + Sync {
    /// رمزنگاری داده با متادیتای مرتبط.
    fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Result<EncryptedPackage, ErrorPayload>;

    /// رمزگشایی بسته با متادیتای مرتبط.
    fn decrypt(
        &self,
        package: &EncryptedPackage,
        associated_data: &[u8],
    ) -> Result<SecretBytes, ErrorPayload>;
}

/// سرویس اسکیمای فیلدهای سفارشی (موتور فیلدها).
pub trait SchemaProvider: Send + Sync {
    /// فهرست تعریف فیلدهای سفارشی.
    fn list_fields(&self) -> Result<Value, ErrorPayload>;

    /// تعریف یک فیلد سفارشی.
    fn get_field(&self, field_id: Uuid) -> Result<Value, ErrorPayload>;

    /// متادیتای رندر فرم (گروه‌ها، ترتیب، ویجت‌ها، اعتبارسنجی).
    fn get_form_metadata(&self) -> Result<Value, ErrorPayload>;
}

/// سرویس پرس‌وجو و تجمیع (موتور پرس‌وجو).
pub trait QueryProvider: Send + Sync {
    /// پرس‌وجوی معاملات با فیلتر و صفحه‌بندی.
    fn query_trades(&self, filter: Value, pagination: Pagination) -> Result<Value, ErrorPayload>;

    /// پرس‌وجوی تجمیعی (آمار و عملکردها).
    fn aggregate(&self, aggregation: Value) -> Result<Value, ErrorPayload>;
}

/// سرویس حسابرسی (موتور امنیت).
pub trait AuditProvider: Send + Sync {
    /// ثبت یک عملیات حساس.
    fn record(&self, entry: AuditEntry) -> Result<(), ErrorPayload>;

    /// پرس‌وجوی لاگ حسابرسی.
    fn query(&self, filter: Value, pagination: Pagination) -> Result<Value, ErrorPayload>;
}

/// سرویس پشتیبان (موتور ذخیره‌سازی).
pub trait BackupProvider: Send + Sync {
    /// ایجاد پشتیبان رمزنگاری‌شده.
    fn create_backup(&self, reason: &str) -> Result<BackupInfo, ErrorPayload>;

    /// بازگردانی پشتیبان.
    fn restore_backup(&self, backup_id: Uuid) -> Result<(), ErrorPayload>;

    /// فهرست پشتیبان‌ها.
    fn list_backups(&self) -> Result<Value, ErrorPayload>;
}

/// سرویس پیوست‌ها (موتور ذخیره‌سازی).
pub trait AttachmentProvider: Send + Sync {
    /// ذخیره پیوست و بازگرداندن متادیتای آن.
    fn store(
        &self,
        filename: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<AttachmentInfo, ErrorPayload>;

    /// دریافت محتوای خام پیوست.
    fn retrieve(&self, attachment_id: Uuid) -> Result<Vec<u8>, ErrorPayload>;

    /// پیوند پیوست به یک معامله.
    fn link_to_trade(
        &self,
        attachment_id: Uuid,
        trade_id: Uuid,
        link_type: &str,
    ) -> Result<(), ErrorPayload>;

    /// فهرست پیوست‌های یک معامله.
    fn list_for_trade(&self, trade_id: Uuid) -> Result<Value, ErrorPayload>;
}

/// سرویس حوزه معاملاتی (موتور دامنه).
pub trait TradeServiceProvider: Send + Sync {
    /// جزئیات یک معامله.
    fn get_trade(&self, trade_id: Uuid) -> Result<Value, ErrorPayload>;

    /// فهرست معاملات با فیلتر و صفحه‌بندی.
    fn list_trades(&self, filter: Value, pagination: Pagination) -> Result<Value, ErrorPayload>;

    /// ثبت معامله (دستور create_trade).
    fn create_trade(&self, command: Value) -> Result<Value, ErrorPayload>;

    /// ویرایش معامله (دستور update_trade).
    fn update_trade(&self, trade_id: Uuid, command: Value) -> Result<Value, ErrorPayload>;

    /// حذف معامله (دستور delete_trade).
    fn delete_trade(&self, trade_id: Uuid) -> Result<(), ErrorPayload>;
}

/// سرویس تعریف فیلد (موتور فیلدها).
pub trait FieldServiceProvider: Send + Sync {
    /// تعریف فیلد سفارشی جدید.
    fn define_field(&self, definition: Value) -> Result<Value, ErrorPayload>;

    /// ویرایش تعریف فیلد سفارشی.
    fn update_field(&self, field_id: Uuid, definition: Value) -> Result<Value, ErrorPayload>;

    /// غیرفعال‌سازی فیلد سفارشی (حذف نابودشونده ممنوع).
    fn deactivate_field(&self, field_id: Uuid) -> Result<(), ErrorPayload>;
}

/// مجموعه سرویس‌هایی که پلاگین‌ها از طریق پل زدن RPC می‌بینند.
///
/// **مرز امنیتی:** این واجهة عمداً [`KeyProvider`] و
/// [`EncryptionProvider`] را شامل نمی‌شود؛ پلاگین‌ها به کلیدها
/// و رمزنگاری مستقیم دسترسی ندارند.
pub trait PluginHostServices: Send + Sync {
    /// سرویس حوزه معاملاتی.
    fn trade_service(&self) -> &dyn TradeServiceProvider;

    /// سرویس تعریف فیلد.
    fn field_service(&self) -> &dyn FieldServiceProvider;

    /// سرویس اسکیمای فیلدها.
    fn schema_service(&self) -> &dyn SchemaProvider;

    /// سرویس پرس‌وجو.
    fn query_service(&self) -> &dyn QueryProvider;

    /// سرویس حسابرسی.
    fn audit_service(&self) -> &dyn AuditProvider;

    /// سرویس پشتیبان.
    fn backup_service(&self) -> &dyn BackupProvider;

    /// سرویس پیوست‌ها.
    fn attachment_service(&self) -> &dyn AttachmentProvider;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_traits_are_dyn_compatible() {
        fn check<T: ?Sized + Send + Sync>() {}
        check::<dyn KeyProvider>();
        check::<dyn EncryptionProvider>();
        check::<dyn SchemaProvider>();
        check::<dyn QueryProvider>();
        check::<dyn AuditProvider>();
        check::<dyn BackupProvider>();
        check::<dyn AttachmentProvider>();
        check::<dyn TradeServiceProvider>();
        check::<dyn FieldServiceProvider>();
        check::<dyn PluginHostServices>();
    }

    #[test]
    fn secret_bytes_zeroizes_on_drop() {
        let secret = SecretBytes::new(vec![1, 2, 3, 4]);
        assert_eq!(secret.expose(), &[1, 2, 3, 4]);
        assert_eq!(secret.len(), 4);
        assert!(!secret.is_empty());
        let raw = secret.into_inner();
        assert_eq!(raw, vec![1, 2, 3, 4]);
        // رفتار صفرسازی در Drop تضمین می‌شود؛ بافت Debug محتوا را فاش نمی‌کند.
        let secret = SecretBytes::from(vec![9, 8, 7]);
        let debug = format!("{:?}", secret);
        assert!(!debug.contains('9'));
    }

    #[test]
    fn pagination_defaults_to_limit_50() {
        assert_eq!(Pagination::default().limit, 50);
        assert_eq!(Pagination::default().offset, 0);
        assert_eq!(Pagination::new(10, 20).limit, 10);
    }
}
