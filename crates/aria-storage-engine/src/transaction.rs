//! تراکنش‌های اتمیک نوشتن.
//!
//! خواندن‌ها نیازی به تراکنش صریح ندارند (rusqlite هر پرس‌وجو را در حالت
//! autocommit اجرا می‌کند). برای نوشتن اتمیک از [`StorageTransaction`]
//! استفاده می‌شود:
//!
//! - `commit` تغییرات را ثبت می‌کند.
//! - `rollback` آن‌ها را برمی‌گرداند.
//! - اگر تراکنش بدون `commit` رها شود، `rusqlite::Transaction` به‌صورت
//!   خودکار rollback می‌کند (حفاظت RAII در برابر رهاسازی/وحشت).
//!
//! فرض معماری: اپلیکیشن آفلاین-فرست و کاربر-محور است؛ در هر زمان یک نویسنده
//! روی دیتابیس فعال است (`rusqlite::Connection` نیز `Sync` نیست و در رشته‌های
//! مختلف به‌اشتراک گذاشته نمی‌شود).

use rusqlite::{Connection, Transaction};

use crate::error::{StorageError, StorageResult};

/// یک تراکنش نوشتن اتمیک روی دیتابیس.
pub struct StorageTransaction<'connection> {
    transaction: Transaction<'connection>,
}

impl<'connection> StorageTransaction<'connection> {
    /// یک تراکنش جدید می‌آغازد.
    pub fn begin(connection: &'connection Connection) -> StorageResult<Self> {
        let transaction = connection.unchecked_transaction().map_err(|source| {
            StorageError::TransactionFailed {
                context: Some(serde_json::json!({ "step": "begin" })),
                source: Some(Box::new(source)),
            }
        })?;
        Ok(Self { transaction })
    }

    /// دسترسی به اتصال داخل تراکنش.
    pub fn connection(&self) -> &Connection {
        &self.transaction
    }

    /// تغییرات را ثبت می‌کند.
    pub fn commit(self) -> StorageResult<()> {
        self.transaction
            .commit()
            .map_err(|source| StorageError::TransactionFailed {
                context: Some(serde_json::json!({ "step": "commit" })),
                source: Some(Box::new(source)),
            })
    }

    /// تغییرات را برمی‌گرداند.
    pub fn rollback(self) -> StorageResult<()> {
        self.transaction
            .rollback()
            .map_err(|source| StorageError::TransactionFailed {
                context: Some(serde_json::json!({ "step": "rollback" })),
                source: Some(Box::new(source)),
            })
    }

    /// اجرای یک بدنه در تراکنش: با موفقیت `commit` و با خطا `rollback`.
    pub fn run<T, F>(connection: &'connection Connection, body: F) -> StorageResult<T>
    where
        F: FnOnce(&Connection) -> StorageResult<T>,
    {
        let transaction = Self::begin(connection)?;
        match body(transaction.connection()) {
            Ok(value) => {
                transaction.commit()?;
                Ok(value)
            }
            Err(error) => {
                // rollback ممکن است خودش خطا بدهد؛ خطای اصلی حفظ می‌شود.
                let _ = transaction.rollback();
                Err(error)
            }
        }
    }
}
