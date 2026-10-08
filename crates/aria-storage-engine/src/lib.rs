//! # aria-storage-engine — موتور ذخیره‌سازی کرنل آریا
//!
//! موتور ذخیره‌سازی روی SQLite رمزنگاری‌شده (سازگار با SQLCipher، با
//! OpenSSL بسته‌بندی‌شده) بنا شده است و شامل:
//!
//! - [`connection`]: باز کردن دیتابیس، PRAGMAهای لازم، کلید رمزنگاری و بررسی سلامت
//! - [`migration`]: سیستم مهاجرت مبتنی بر `PRAGMA user_version`
//! - [`transaction`]: تراکنش‌های نوشتن اتمیک
//! - [`attachment`]: ذخیره‌سازی پیوست‌ها (blake3، محتوامحور، تشخیص تکراری)
//! - [`backup`]: پایه چیدمان بسته پشتیبان
//! - [`schema`]: کمک‌های بازتابی اسکیما
//! - [`error`]: مدل خطای موتور (محدوده ۲۰۰۰–۲۹۹۹)
//!
//! این موتور فقط ذخیره‌سازی است؛ هیچ قاعده کسب‌وکاری دامنه (فاز ۱.۶) یا
//! تحلیل پرس‌وجو (فاز ۱.۹) در آن پیاده نمی‌شود.

pub mod attachment;
pub mod backup;
pub mod connection;
pub mod error;
pub mod migration;
pub mod schema;
pub mod timestamps;
mod transaction;

pub use connection::{
    DatabaseConnection, ForeignKeyViolation, DATABASE_KEY_LENGTH_BYTES, DEFAULT_BUSY_TIMEOUT_MS,
};
pub use error::{StorageError, StorageResult, ALL_ERROR_DEFINITIONS, STORAGE_ERROR_RANGE};
pub use migration::{
    latest_schema_version, Migration, MigrationReport, SchemaMigrator, MIGRATIONS,
};
pub use transaction::StorageTransaction;

use std::path::{Path, PathBuf};

use aria_contracts::services::{AttachmentInfo, SecretBytes};
use aria_foundation_engine::config::KernelConfig;
use uuid::Uuid;

/// موتور ذخیره‌سازی: اتصال دیتابیس به‌همراه مسیرهای ذخیره‌سازی.
///
/// ساخت با [`StorageEngine::open`] انجام می‌شود؛ سپس می‌توان مهاجرت‌ها را با
/// [`StorageEngine::run_migrations`] اجرا کرد و از تراکنش‌ها و پیوست‌ها استفاده کرد.
pub struct StorageEngine {
    database: DatabaseConnection,
    attachments_dir: PathBuf,
    backup_dir: PathBuf,
}

impl StorageEngine {
    /// دیتابیس را در مسیر `config.database_path` باز می‌کند.
    ///
    /// اگر کلید داده شود، دیتابیس رمزنگاری‌شده باز می‌شود؛ کلید باید ۳۲ بایت
    /// (۲۵۶ بیت) باشد. مهاجرت‌ها به‌صورت خودکار اعمال نمی‌شوند و باید صریحاً
    /// [`StorageEngine::run_migrations`] فراخوانی شود.
    pub fn open(
        config: &KernelConfig,
        encryption_key: Option<&SecretBytes>,
    ) -> StorageResult<Self> {
        let database = DatabaseConnection::open(&config.database_path, encryption_key)?;
        Ok(Self {
            database,
            attachments_dir: config.attachments_dir.clone(),
            backup_dir: config.backup_dir.clone(),
        })
    }

    /// یک موتور را از یک اتصال موجود می‌سازد (برای ابزارها و تست‌ها).
    pub fn from_connection(
        database: DatabaseConnection,
        attachments_dir: impl Into<PathBuf>,
        backup_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            database,
            attachments_dir: attachments_dir.into(),
            backup_dir: backup_dir.into(),
        }
    }

    /// دسترسی به اتصال دیتابیس.
    pub fn database(&self) -> &DatabaseConnection {
        &self.database
    }

    /// اتصال rusqlite زیرین.
    pub fn connection(&self) -> &rusqlite::Connection {
        self.database.connection()
    }

    /// مسیر فایل دیتابیس.
    pub fn database_path(&self) -> &Path {
        self.database.path()
    }

    /// پوشه پیوست‌ها.
    pub fn attachments_dir(&self) -> &Path {
        &self.attachments_dir
    }

    /// پوشه پشتیبان‌ها.
    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    /// آیا دیتابیس رمزنگاری‌شده است؟
    pub fn is_encrypted(&self) -> bool {
        self.database.is_encrypted()
    }

    /// نسخه فعلی اسکیمای دیتابیس.
    pub fn schema_version(&self) -> StorageResult<u32> {
        migration::read_user_version(self.connection())
    }

    /// همه مهاجرت‌های معلق را بدون قلاب پشتیبان اجرا می‌کند.
    pub fn run_migrations(&self) -> StorageResult<MigrationReport> {
        SchemaMigrator::new(self.connection()).run()
    }

    /// مهاجرت‌ها را با قلاب پشتیبان‌گیری پیش از مهاجرت‌های پرریسک اجرا می‌کند.
    pub fn run_migrations_with_backup_hook<F>(
        &self,
        backup_hook: F,
    ) -> StorageResult<MigrationReport>
    where
        F: FnMut(u32) -> StorageResult<()>,
    {
        SchemaMigrator::new(self.connection()).run_with_backup_hook(backup_hook)
    }

    /// بررسی سلامت کامل دیتابیس.
    pub fn integrity_check(&self) -> StorageResult<()> {
        self.database.integrity_check()
    }

    /// بررسی یکپارچگی کلیدهای خارجی.
    pub fn foreign_key_check(&self) -> StorageResult<Vec<ForeignKeyViolation>> {
        self.database.foreign_key_check()
    }

    /// بررسی سلامت کامل (یکپارچگی فایل + یکپارچگی کلیدهای خارجی).
    pub fn health_check(&self) -> StorageResult<()> {
        self.database.health_check()
    }

    /// یک تراکنش نوشتن اتمیک می‌آغازد.
    pub fn transaction(&self) -> StorageResult<StorageTransaction<'_>> {
        StorageTransaction::begin(self.connection())
    }

    /// یک بدنه را در یک تراکنش نوشتن اتمیک اجرا می‌کند (commit در موفقیت،
    /// rollback در خطا).
    pub fn with_transaction<T, F>(&self, body: F) -> StorageResult<T>
    where
        F: FnOnce(&rusqlite::Connection) -> StorageResult<T>,
    {
        StorageTransaction::run(self.connection(), body)
    }

    // ---------------------------------------------------------------- پیوست‌ها

    /// ذخیره پیوست (فایل بیرون دیتابیس، متادیتا داخل دیتابیس).
    pub fn store_attachment(
        &self,
        filename: &str,
        content_type: &str,
        data: &[u8],
    ) -> StorageResult<AttachmentInfo> {
        attachment::store(
            self.connection(),
            &self.attachments_dir,
            filename,
            content_type,
            data,
        )
    }

    /// متادیتای یک پیوست.
    pub fn get_attachment(&self, attachment_id: &Uuid) -> StorageResult<AttachmentInfo> {
        attachment::get(self.connection(), attachment_id)
    }

    /// محتوای خام یک پیوست.
    pub fn read_attachment(&self, attachment_id: &Uuid) -> StorageResult<Vec<u8>> {
        attachment::read(self.connection(), &self.attachments_dir, attachment_id)
    }

    /// بررسی یکپارچگی فایل پیوست با هش blake3.
    pub fn verify_attachment_integrity(&self, attachment_id: &Uuid) -> StorageResult<bool> {
        attachment::verify_integrity(self.connection(), &self.attachments_dir, attachment_id)
    }

    /// پیوند پیوست به یک معامله.
    pub fn link_attachment_to_trade(
        &self,
        attachment_id: &Uuid,
        trade_id: &Uuid,
        link_type: &str,
    ) -> StorageResult<()> {
        attachment::link_to_trade(self.connection(), attachment_id, trade_id, link_type)
    }

    /// برداشتن پیوند یک پیوست از یک معامله.
    pub fn unlink_attachment_from_trade(
        &self,
        attachment_id: &Uuid,
        trade_id: &Uuid,
    ) -> StorageResult<bool> {
        attachment::unlink_from_trade(self.connection(), attachment_id, trade_id)
    }

    /// فهرست پیوست‌های یک معامله.
    pub fn list_attachments_for_trade(
        &self,
        trade_id: &Uuid,
    ) -> StorageResult<Vec<AttachmentInfo>> {
        attachment::list_for_trade(self.connection(), trade_id)
    }

    // ---------------------------------------------------------------- پشتیبان

    /// چیدمان یک بسته پشتیبان ایجاد می‌کند (فاز ۱.۳: فقط ساختار؛ فاز ۱.۱۶: کامل).
    pub fn create_backup_layout(&self) -> StorageResult<backup::BackupPackage> {
        backup::create_layout(&self.backup_dir, self.schema_version()?)
    }

    /// دیتابیس را می‌بندد.
    pub fn close(self) -> StorageResult<()> {
        self.database.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_schema_version_is_exposed() {
        assert_eq!(latest_schema_version(), 1);
    }
}
