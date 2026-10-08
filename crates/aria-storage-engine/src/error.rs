//! مدل خطای موتور ذخیره‌سازی.
//!
//! کدهای این موتور در محدوده پایدار `2000..=2999` قرار دارند (قرارداد خطا:
//! `docs/contracts/error-codes.md`). هر خطا دارای کد عددی یکتا، نام متغیر
//! ماشین‌خوان، کلید پیام فارسی و متنیان اختیاری است.
//!
//! تبدیل خطا به ساختار قابل‌انتقال در مرزهای سرویس با [`StorageError::to_payload`]
//! انجام می‌شود (ساختار `ErrorPayload` قراردادها).

use std::error::Error as StdError;
use std::ops::RangeInclusive;

use aria_contracts::error::ErrorPayload;
use aria_foundation_engine::ErrorDefinition;
use serde::Serialize;
use serde_json::Value;

/// محدوده کدهای خطای موتور ذخیره‌سازی.
pub const STORAGE_ERROR_RANGE: RangeInclusive<u32> = 2000..=2999;

const E_DATABASE_OPEN_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2001,
    variant: "DatabaseOpenFailed",
    message_key: "error.storage.database_open_failed",
};
const E_DATABASE_CLOSE_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2002,
    variant: "DatabaseCloseFailed",
    message_key: "error.storage.database_close_failed",
};
const E_ENCRYPTION_KEY_INVALID: ErrorDefinition = ErrorDefinition {
    code: 2003,
    variant: "EncryptionKeyInvalid",
    message_key: "error.storage.encryption_key_invalid",
};
const E_DATABASE_CORRUPTED: ErrorDefinition = ErrorDefinition {
    code: 2004,
    variant: "DatabaseCorrupted",
    message_key: "error.storage.database_corrupted",
};
const E_MIGRATION_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2005,
    variant: "MigrationFailed",
    message_key: "error.storage.migration_failed",
};
const E_MIGRATION_VERSION_UNKNOWN: ErrorDefinition = ErrorDefinition {
    code: 2006,
    variant: "MigrationVersionUnknown",
    message_key: "error.storage.migration_version_unknown",
};
const E_TRANSACTION_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2007,
    variant: "TransactionFailed",
    message_key: "error.storage.transaction_failed",
};
const E_QUERY_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2008,
    variant: "QueryFailed",
    message_key: "error.storage.query_failed",
};
const E_ATTACHMENT_WRITE_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2009,
    variant: "AttachmentWriteFailed",
    message_key: "error.storage.attachment_write_failed",
};
const E_ATTACHMENT_NOT_FOUND: ErrorDefinition = ErrorDefinition {
    code: 2010,
    variant: "AttachmentNotFound",
    message_key: "error.storage.attachment_not_found",
};
const E_ATTACHMENT_INTEGRITY_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2011,
    variant: "AttachmentIntegrityFailed",
    message_key: "error.storage.attachment_integrity_failed",
};
const E_BACKUP_LAYOUT_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2012,
    variant: "BackupLayoutFailed",
    message_key: "error.storage.backup_layout_failed",
};
const E_INTEGRITY_CHECK_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2013,
    variant: "IntegrityCheckFailed",
    message_key: "error.storage.integrity_check_failed",
};
const E_FOREIGN_KEY_CHECK_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2014,
    variant: "ForeignKeyCheckFailed",
    message_key: "error.storage.foreign_key_check_failed",
};
const E_MIGRATION_BACKUP_FAILED: ErrorDefinition = ErrorDefinition {
    code: 2015,
    variant: "MigrationBackupFailed",
    message_key: "error.storage.migration_backup_failed",
};
const E_ENCRYPTION_KEY_MISMATCH: ErrorDefinition = ErrorDefinition {
    code: 2016,
    variant: "EncryptionKeyMismatch",
    message_key: "error.storage.encryption_key_mismatch",
};

/// جدول تعریف‌های پایدار همه خطاهای موتور ذخیره‌سازی.
pub const ALL_ERROR_DEFINITIONS: &[ErrorDefinition] = &[
    E_DATABASE_OPEN_FAILED,
    E_DATABASE_CLOSE_FAILED,
    E_ENCRYPTION_KEY_INVALID,
    E_DATABASE_CORRUPTED,
    E_MIGRATION_FAILED,
    E_MIGRATION_VERSION_UNKNOWN,
    E_TRANSACTION_FAILED,
    E_QUERY_FAILED,
    E_ATTACHMENT_WRITE_FAILED,
    E_ATTACHMENT_NOT_FOUND,
    E_ATTACHMENT_INTEGRITY_FAILED,
    E_BACKUP_LAYOUT_FAILED,
    E_INTEGRITY_CHECK_FAILED,
    E_FOREIGN_KEY_CHECK_FAILED,
    E_MIGRATION_BACKUP_FAILED,
    E_ENCRYPTION_KEY_MISMATCH,
];

/// خطای ساختاریافته موتور ذخیره‌سازی.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("failed to open the database")]
    DatabaseOpenFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("failed to close the database")]
    DatabaseCloseFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("encryption key is invalid")]
    EncryptionKeyInvalid {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("database is corrupted or not readable")]
    DatabaseCorrupted {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("migration failed")]
    MigrationFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("database schema version is newer than supported")]
    MigrationVersionUnknown {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("transaction operation failed")]
    TransactionFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("storage query failed")]
    QueryFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("failed to store attachment")]
    AttachmentWriteFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("attachment not found")]
    AttachmentNotFound {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("attachment integrity check failed")]
    AttachmentIntegrityFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("failed to create backup package layout")]
    BackupLayoutFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("database integrity check reported problems")]
    IntegrityCheckFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("foreign key check reported violations")]
    ForeignKeyCheckFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("pre-migration backup failed")]
    MigrationBackupFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("encryption key does not match the database")]
    EncryptionKeyMismatch {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

impl StorageError {
    /// تعریف پایدار این خطا (کد، متغیر، کلید پیام).
    pub fn definition(&self) -> &'static ErrorDefinition {
        match self {
            StorageError::DatabaseOpenFailed { .. } => &E_DATABASE_OPEN_FAILED,
            StorageError::DatabaseCloseFailed { .. } => &E_DATABASE_CLOSE_FAILED,
            StorageError::EncryptionKeyInvalid { .. } => &E_ENCRYPTION_KEY_INVALID,
            StorageError::DatabaseCorrupted { .. } => &E_DATABASE_CORRUPTED,
            StorageError::MigrationFailed { .. } => &E_MIGRATION_FAILED,
            StorageError::MigrationVersionUnknown { .. } => &E_MIGRATION_VERSION_UNKNOWN,
            StorageError::TransactionFailed { .. } => &E_TRANSACTION_FAILED,
            StorageError::QueryFailed { .. } => &E_QUERY_FAILED,
            StorageError::AttachmentWriteFailed { .. } => &E_ATTACHMENT_WRITE_FAILED,
            StorageError::AttachmentNotFound { .. } => &E_ATTACHMENT_NOT_FOUND,
            StorageError::AttachmentIntegrityFailed { .. } => &E_ATTACHMENT_INTEGRITY_FAILED,
            StorageError::BackupLayoutFailed { .. } => &E_BACKUP_LAYOUT_FAILED,
            StorageError::IntegrityCheckFailed { .. } => &E_INTEGRITY_CHECK_FAILED,
            StorageError::ForeignKeyCheckFailed { .. } => &E_FOREIGN_KEY_CHECK_FAILED,
            StorageError::MigrationBackupFailed { .. } => &E_MIGRATION_BACKUP_FAILED,
            StorageError::EncryptionKeyMismatch { .. } => &E_ENCRYPTION_KEY_MISMATCH,
        }
    }

    /// کد عددی یکتای خطا.
    pub fn code(&self) -> u32 {
        self.definition().code
    }

    /// نام متغیر قابل‌خواندن ماشینی.
    pub fn variant(&self) -> &'static str {
        self.definition().variant
    }

    /// کلید پیام فارسی قابل‌خواندن برای کاربر.
    pub fn message_key(&self) -> &'static str {
        self.definition().message_key
    }

    /// متنیان اختیاری خطا.
    pub fn context(&self) -> Option<&Value> {
        match self {
            StorageError::DatabaseOpenFailed { context, .. }
            | StorageError::DatabaseCloseFailed { context, .. }
            | StorageError::EncryptionKeyInvalid { context, .. }
            | StorageError::DatabaseCorrupted { context, .. }
            | StorageError::MigrationFailed { context, .. }
            | StorageError::MigrationVersionUnknown { context, .. }
            | StorageError::TransactionFailed { context, .. }
            | StorageError::QueryFailed { context, .. }
            | StorageError::AttachmentWriteFailed { context, .. }
            | StorageError::AttachmentNotFound { context, .. }
            | StorageError::AttachmentIntegrityFailed { context, .. }
            | StorageError::BackupLayoutFailed { context, .. }
            | StorageError::IntegrityCheckFailed { context, .. }
            | StorageError::ForeignKeyCheckFailed { context, .. }
            | StorageError::MigrationBackupFailed { context, .. }
            | StorageError::EncryptionKeyMismatch { context, .. } => context.as_ref(),
        }
    }

    /// متنیان JSON را به خطا اضافه می‌کند (جایگزین متنیان قبلی).
    pub fn with_context(mut self, context: impl Serialize) -> Self {
        let value = serde_json::to_value(context).ok();
        match &mut self {
            StorageError::DatabaseOpenFailed { context, .. }
            | StorageError::DatabaseCloseFailed { context, .. }
            | StorageError::EncryptionKeyInvalid { context, .. }
            | StorageError::DatabaseCorrupted { context, .. }
            | StorageError::MigrationFailed { context, .. }
            | StorageError::MigrationVersionUnknown { context, .. }
            | StorageError::TransactionFailed { context, .. }
            | StorageError::QueryFailed { context, .. }
            | StorageError::AttachmentWriteFailed { context, .. }
            | StorageError::AttachmentNotFound { context, .. }
            | StorageError::AttachmentIntegrityFailed { context, .. }
            | StorageError::BackupLayoutFailed { context, .. }
            | StorageError::IntegrityCheckFailed { context, .. }
            | StorageError::ForeignKeyCheckFailed { context, .. }
            | StorageError::MigrationBackupFailed { context, .. }
            | StorageError::EncryptionKeyMismatch { context, .. } => *context = value,
        }
        self
    }

    /// خطای منبع را به خطا اضافه می‌کند.
    pub fn with_source(mut self, source_error: impl StdError + Send + Sync + 'static) -> Self {
        match &mut self {
            StorageError::DatabaseOpenFailed { source, .. }
            | StorageError::DatabaseCloseFailed { source, .. }
            | StorageError::EncryptionKeyInvalid { source, .. }
            | StorageError::DatabaseCorrupted { source, .. }
            | StorageError::MigrationFailed { source, .. }
            | StorageError::MigrationVersionUnknown { source, .. }
            | StorageError::TransactionFailed { source, .. }
            | StorageError::QueryFailed { source, .. }
            | StorageError::AttachmentWriteFailed { source, .. }
            | StorageError::AttachmentNotFound { source, .. }
            | StorageError::AttachmentIntegrityFailed { source, .. }
            | StorageError::BackupLayoutFailed { source, .. }
            | StorageError::IntegrityCheckFailed { source, .. }
            | StorageError::ForeignKeyCheckFailed { source, .. }
            | StorageError::MigrationBackupFailed { source, .. }
            | StorageError::EncryptionKeyMismatch { source, .. } => {
                *source = Some(Box::new(source_error))
            }
        }
        self
    }

    /// تبدیل به ساختار خطای قابل‌انتقال در مرزهای سرویس و RPC.
    pub fn to_payload(&self) -> ErrorPayload {
        ErrorPayload {
            code: self.code(),
            variant: self.variant().to_string(),
            message_key: self.message_key().to_string(),
            context: self.context().cloned(),
        }
    }
}

/// تبدیل عمومی خطای rusqlite به خطای پرس‌وجوی ذخیره‌سازی.
impl From<rusqlite::Error> for StorageError {
    fn from(source: rusqlite::Error) -> Self {
        StorageError::QueryFailed {
            context: None,
            source: Some(Box::new(source)),
        }
    }
}

/// نتیجه استاندارد موتور ذخیره‌سازی.
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)]
mod tests {
    use super::*;
    use aria_contracts::error::EngineId;
    use std::collections::HashSet;

    fn all_error_samples() -> Vec<StorageError> {
        vec![
            StorageError::DatabaseOpenFailed {
                context: None,
                source: None,
            },
            StorageError::DatabaseCloseFailed {
                context: None,
                source: None,
            },
            StorageError::EncryptionKeyInvalid {
                context: None,
                source: None,
            },
            StorageError::DatabaseCorrupted {
                context: None,
                source: None,
            },
            StorageError::MigrationFailed {
                context: None,
                source: None,
            },
            StorageError::MigrationVersionUnknown {
                context: None,
                source: None,
            },
            StorageError::TransactionFailed {
                context: None,
                source: None,
            },
            StorageError::QueryFailed {
                context: None,
                source: None,
            },
            StorageError::AttachmentWriteFailed {
                context: None,
                source: None,
            },
            StorageError::AttachmentNotFound {
                context: None,
                source: None,
            },
            StorageError::AttachmentIntegrityFailed {
                context: None,
                source: None,
            },
            StorageError::BackupLayoutFailed {
                context: None,
                source: None,
            },
            StorageError::IntegrityCheckFailed {
                context: None,
                source: None,
            },
            StorageError::ForeignKeyCheckFailed {
                context: None,
                source: None,
            },
            StorageError::MigrationBackupFailed {
                context: None,
                source: None,
            },
            StorageError::EncryptionKeyMismatch {
                context: None,
                source: None,
            },
        ]
    }

    #[test]
    fn error_codes_are_unique_and_within_storage_range() {
        let mut codes = HashSet::new();
        for definition in ALL_ERROR_DEFINITIONS {
            assert!(
                STORAGE_ERROR_RANGE.contains(&definition.code),
                "code {} is outside the storage range",
                definition.code
            );
            assert!(
                codes.insert(definition.code),
                "duplicate error code {}",
                definition.code
            );
            assert!(!definition.variant.is_empty());
            assert!(definition.message_key.starts_with("error.storage."));
        }
        assert_eq!(ALL_ERROR_DEFINITIONS.len(), 16);
    }

    #[test]
    fn variant_definitions_match_the_table() {
        for error in all_error_samples() {
            let definition = error.definition();
            assert!(
                ALL_ERROR_DEFINITIONS.contains(definition),
                "definition {:?} is missing from ALL_ERROR_DEFINITIONS",
                definition
            );
            assert_eq!(error.code(), definition.code);
            assert_eq!(error.variant(), definition.variant);
            assert_eq!(error.message_key(), definition.message_key);
        }
    }

    #[test]
    fn payload_carries_storage_engine_and_details() {
        let error = StorageError::AttachmentNotFound {
            context: Some(serde_json::json!({ "attachment_id": "abc" })),
            source: None,
        };
        let payload = error.to_payload();
        assert_eq!(payload.code, 2010);
        assert_eq!(payload.variant, "AttachmentNotFound");
        assert_eq!(payload.engine(), Some(EngineId::Storage));
        assert_eq!(
            payload
                .context
                .and_then(|c| c["attachment_id"].as_str().map(str::to_string)),
            Some("abc".to_string())
        );
    }

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StorageError>();
    }
}
