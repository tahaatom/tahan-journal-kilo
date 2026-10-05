//! مدل خطای ساختاریافته کرنل آریا.
//!
//! هر خطای کرنل دارای چهار جزء پایدار است:
//! - کد عددی یکتا در محدوده موتور پایه (`1000..=1999`)
//! - نام متغیر قابل‌خواندن ماشینی
//! - کلید پیام فارسی قابل‌خواندن برای کاربر (قالب نقطه‌ای، سازگار با i18next)
//! - متنیان اختیاری به‌صورت JSON و خطای منبع اختیاری
//!
//! `anyhow` فقط در مرزهای اپلیکیشن مجاز است، نه در API عمومی موتورها.

use std::error::Error as StdError;
use std::ops::RangeInclusive;

use serde::Serialize;
use serde_json::Value;

/// محدوده کدهای خطای موتور پایه.
pub const FOUNDATION_ERROR_RANGE: RangeInclusive<u32> = 1000..=1999;

/// تعریف پایدار یک خطای کرنل.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorDefinition {
    /// کد عددی یکتای خطا.
    pub code: u32,
    /// نام متغیر قابل‌خواندن ماشینی.
    pub variant: &'static str,
    /// کلید پیام فارسی قابل‌خواندن برای کاربر.
    pub message_key: &'static str,
}

/// خطای ساختاریافته کرنل آریا.
#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("failed to load kernel configuration")]
    ConfigLoadFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("failed to save kernel configuration")]
    ConfigSaveFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("kernel configuration is invalid")]
    ConfigInvalid {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("kernel configuration path is invalid or unavailable")]
    ConfigPathInvalid {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("time conversion failed")]
    TimeConversionFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("jalali date is invalid")]
    JalaliDateInvalid {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("failed to initialize logging")]
    LogInitFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("failed to generate identifier")]
    IdGenerationFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("invalid argument")]
    InvalidArgument {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("failed to create directory")]
    DirectoryCreationFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    #[error("serialization failed")]
    SerializationFailed {
        context: Option<Value>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

/// جدول تعریف‌های پایدار همه خطاهای موتور پایه.
/// این جدول منبع واحد مستندات کدهای خطا و قرارداد RPC خطاست.
pub const ALL_ERROR_DEFINITIONS: &[ErrorDefinition] = &[
    ErrorDefinition {
        code: 1001,
        variant: "ConfigLoadFailed",
        message_key: "error.config.load_failed",
    },
    ErrorDefinition {
        code: 1002,
        variant: "ConfigSaveFailed",
        message_key: "error.config.save_failed",
    },
    ErrorDefinition {
        code: 1003,
        variant: "ConfigInvalid",
        message_key: "error.config.invalid",
    },
    ErrorDefinition {
        code: 1004,
        variant: "ConfigPathInvalid",
        message_key: "error.config.path_invalid",
    },
    ErrorDefinition {
        code: 1005,
        variant: "TimeConversionFailed",
        message_key: "error.time.conversion_failed",
    },
    ErrorDefinition {
        code: 1006,
        variant: "JalaliDateInvalid",
        message_key: "error.jalali.date_invalid",
    },
    ErrorDefinition {
        code: 1007,
        variant: "LogInitFailed",
        message_key: "error.logging.init_failed",
    },
    ErrorDefinition {
        code: 1008,
        variant: "IdGenerationFailed",
        message_key: "error.ids.generation_failed",
    },
    ErrorDefinition {
        code: 1009,
        variant: "InvalidArgument",
        message_key: "error.invalid_argument",
    },
    ErrorDefinition {
        code: 1010,
        variant: "DirectoryCreationFailed",
        message_key: "error.io.directory_creation_failed",
    },
    ErrorDefinition {
        code: 1011,
        variant: "SerializationFailed",
        message_key: "error.serialization_failed",
    },
];

impl KernelError {
    /// تعریف پایدار این خطا (کد، متغیر، کلید پیام).
    pub fn definition(&self) -> ErrorDefinition {
        match self {
            KernelError::ConfigLoadFailed { .. } => ALL_ERROR_DEFINITIONS[0],
            KernelError::ConfigSaveFailed { .. } => ALL_ERROR_DEFINITIONS[1],
            KernelError::ConfigInvalid { .. } => ALL_ERROR_DEFINITIONS[2],
            KernelError::ConfigPathInvalid { .. } => ALL_ERROR_DEFINITIONS[3],
            KernelError::TimeConversionFailed { .. } => ALL_ERROR_DEFINITIONS[4],
            KernelError::JalaliDateInvalid { .. } => ALL_ERROR_DEFINITIONS[5],
            KernelError::LogInitFailed { .. } => ALL_ERROR_DEFINITIONS[6],
            KernelError::IdGenerationFailed { .. } => ALL_ERROR_DEFINITIONS[7],
            KernelError::InvalidArgument { .. } => ALL_ERROR_DEFINITIONS[8],
            KernelError::DirectoryCreationFailed { .. } => ALL_ERROR_DEFINITIONS[9],
            KernelError::SerializationFailed { .. } => ALL_ERROR_DEFINITIONS[10],
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
            KernelError::ConfigLoadFailed { context, .. }
            | KernelError::ConfigSaveFailed { context, .. }
            | KernelError::ConfigInvalid { context, .. }
            | KernelError::ConfigPathInvalid { context, .. }
            | KernelError::TimeConversionFailed { context, .. }
            | KernelError::JalaliDateInvalid { context, .. }
            | KernelError::LogInitFailed { context, .. }
            | KernelError::IdGenerationFailed { context, .. }
            | KernelError::InvalidArgument { context, .. }
            | KernelError::DirectoryCreationFailed { context, .. }
            | KernelError::SerializationFailed { context, .. } => context.as_ref(),
        }
    }

    /// متنیان JSON را به خطا اضافه می‌کند (جایگزین متنیان قبلی).
    pub fn with_context(mut self, context: impl Serialize) -> Self {
        let value = serde_json::to_value(context).ok();
        match &mut self {
            KernelError::ConfigLoadFailed { context, .. }
            | KernelError::ConfigSaveFailed { context, .. }
            | KernelError::ConfigInvalid { context, .. }
            | KernelError::ConfigPathInvalid { context, .. }
            | KernelError::TimeConversionFailed { context, .. }
            | KernelError::JalaliDateInvalid { context, .. }
            | KernelError::LogInitFailed { context, .. }
            | KernelError::IdGenerationFailed { context, .. }
            | KernelError::InvalidArgument { context, .. }
            | KernelError::DirectoryCreationFailed { context, .. }
            | KernelError::SerializationFailed { context, .. } => *context = value,
        }
        self
    }

    /// خطای منبع را به خطا اضافه می‌کند.
    pub fn with_source(mut self, source_error: impl StdError + Send + Sync + 'static) -> Self {
        match &mut self {
            KernelError::ConfigLoadFailed { source, .. }
            | KernelError::ConfigSaveFailed { source, .. }
            | KernelError::ConfigInvalid { source, .. }
            | KernelError::ConfigPathInvalid { source, .. }
            | KernelError::TimeConversionFailed { source, .. }
            | KernelError::JalaliDateInvalid { source, .. }
            | KernelError::LogInitFailed { source, .. }
            | KernelError::IdGenerationFailed { source, .. }
            | KernelError::InvalidArgument { source, .. }
            | KernelError::DirectoryCreationFailed { source, .. }
            | KernelError::SerializationFailed { source, .. } => {
                *source = Some(Box::new(source_error))
            }
        }
        self
    }
}

/// نتیجه استاندارد موتورهای کرنل آریا.
pub type KernelResult<T> = Result<T, KernelError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn all_error_samples() -> Vec<KernelError> {
        vec![
            KernelError::ConfigLoadFailed {
                context: None,
                source: None,
            },
            KernelError::ConfigSaveFailed {
                context: None,
                source: None,
            },
            KernelError::ConfigInvalid {
                context: None,
                source: None,
            },
            KernelError::ConfigPathInvalid {
                context: None,
                source: None,
            },
            KernelError::TimeConversionFailed {
                context: None,
                source: None,
            },
            KernelError::JalaliDateInvalid {
                context: None,
                source: None,
            },
            KernelError::LogInitFailed {
                context: None,
                source: None,
            },
            KernelError::IdGenerationFailed {
                context: None,
                source: None,
            },
            KernelError::InvalidArgument {
                context: None,
                source: None,
            },
            KernelError::DirectoryCreationFailed {
                context: None,
                source: None,
            },
            KernelError::SerializationFailed {
                context: None,
                source: None,
            },
        ]
    }

    #[test]
    fn error_codes_are_unique_and_within_foundation_range() {
        let mut codes = HashSet::new();
        for definition in ALL_ERROR_DEFINITIONS {
            assert!(
                FOUNDATION_ERROR_RANGE.contains(&definition.code),
                "code {} is outside the foundation range",
                definition.code
            );
            assert!(
                codes.insert(definition.code),
                "duplicate error code {}",
                definition.code
            );
            assert!(!definition.variant.is_empty());
            assert!(definition.message_key.starts_with("error."));
        }
        assert_eq!(ALL_ERROR_DEFINITIONS.len(), 11);
    }

    #[test]
    fn variant_definitions_match_the_table() {
        for error in all_error_samples() {
            let definition = error.definition();
            assert!(
                ALL_ERROR_DEFINITIONS.contains(&definition),
                "definition {:?} is missing from ALL_ERROR_DEFINITIONS",
                definition
            );
            assert_eq!(error.code(), definition.code);
            assert_eq!(error.variant(), definition.variant);
            assert_eq!(error.message_key(), definition.message_key);
        }
    }

    #[test]
    fn error_carries_context_and_source() {
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let error = KernelError::ConfigLoadFailed {
            context: Some(serde_json::json!({ "path": "/tmp/kernel.json" })),
            source: Some(Box::new(source)),
        };
        assert_eq!(error.code(), 1001);
        assert_eq!(error.variant(), "ConfigLoadFailed");
        assert_eq!(error.message_key(), "error.config.load_failed");
        let context = error.context().expect("context must be present");
        assert_eq!(
            context.get("path").and_then(Value::as_str),
            Some("/tmp/kernel.json")
        );
        let source = StdError::source(&error).expect("source must be present");
        assert_eq!(source.to_string(), "file missing");
    }

    #[test]
    fn with_context_and_with_source_builders_work() {
        let error = KernelError::InvalidArgument {
            context: None,
            source: None,
        }
        .with_context(serde_json::json!({ "field": "direction" }))
        .with_source(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "bad input",
        ));
        assert_eq!(error.code(), 1009);
        assert_eq!(
            error
                .context()
                .and_then(|c| c.get("field"))
                .and_then(Value::as_str),
            Some("direction")
        );
        assert!(StdError::source(&error).is_some());
    }

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<KernelError>();
    }
}
