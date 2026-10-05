//! قرارداد خطا.
//!
//! - محدوده کدهای خطای پایدار به‌ازای هر موتور
//! - ساختار خطای RPC (`ErrorPayload`) برای انتقال خطای کرنل در مرزها
//! - کدهای خطای استاندارد JSON-RPC 2.0

use std::fmt;
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// شناسه موتورهای کرنل آریا.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineId {
    Foundation,
    Storage,
    Security,
    Schema,
    Domain,
    Plugin,
    Runtime,
    Query,
    Ui,
}

impl EngineId {
    /// همه موتورهای کرنل.
    pub const ALL: &'static [EngineId] = &[
        EngineId::Foundation,
        EngineId::Storage,
        EngineId::Security,
        EngineId::Schema,
        EngineId::Domain,
        EngineId::Plugin,
        EngineId::Runtime,
        EngineId::Query,
        EngineId::Ui,
    ];

    /// نام پایدار موتور.
    pub fn as_str(self) -> &'static str {
        match self {
            EngineId::Foundation => "foundation",
            EngineId::Storage => "storage",
            EngineId::Security => "security",
            EngineId::Schema => "schema",
            EngineId::Domain => "domain",
            EngineId::Plugin => "plugin",
            EngineId::Runtime => "runtime",
            EngineId::Query => "query",
            EngineId::Ui => "ui",
        }
    }

    /// محدوده پایدار کدهای خطای این موتور.
    pub fn error_range(self) -> RangeInclusive<u32> {
        match self {
            EngineId::Foundation => 1000..=1999,
            EngineId::Storage => 2000..=2999,
            EngineId::Security => 3000..=3999,
            EngineId::Schema => 4000..=4999,
            EngineId::Domain => 5000..=5999,
            EngineId::Plugin => 6000..=6999,
            EngineId::Runtime => 7000..=7999,
            EngineId::Query => 8000..=8999,
            EngineId::Ui => 9000..=9999,
        }
    }

    /// تحلیل شناسه موتور از نام پایدار آن.
    pub fn parse(value: &str) -> Option<EngineId> {
        Self::ALL
            .iter()
            .copied()
            .find(|engine| engine.as_str() == value)
    }
}

impl fmt::Display for EngineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// ساختار خطای پایدار برای انتقال خطای کرنل در RPC و مرزهای سرویس.
///
/// این ساختار آینه‌ی چهار جزء پایدار خطای کرنل (کد، متغیر، کلید پیام،
/// متنیان) است. هر موتور خطای خود را در مرز خود به این ساختار تبدیل می‌کند
/// تا مصرف‌کننده‌ها (از جمله پلاگین‌ها) بدون وابستگی به موتور مبدأ، خطا
/// را بخوانند.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// کد عددی یکتای خطا در محدوده موتور مبدأ.
    pub code: u32,
    /// نام متغیر قابل‌خواندن ماشینی.
    pub variant: String,
    /// کلید پیام فارسی قابل‌خواندن برای کاربر.
    pub message_key: String,
    /// متنیان اختیاری به‌صورت JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
}

impl ErrorPayload {
    /// ساختار خطای جدید.
    pub fn new(code: u32, variant: impl Into<String>, message_key: impl Into<String>) -> Self {
        Self {
            code,
            variant: variant.into(),
            message_key: message_key.into(),
            context: None,
        }
    }

    /// متنیان JSON را به خطا اضافه می‌کند.
    pub fn with_context(mut self, context: impl Serialize) -> Self {
        self.context = serde_json::to_value(context).ok();
        self
    }

    /// موتور مبدأ خطا بر اساس محدوده کد.
    pub fn engine(&self) -> Option<EngineId> {
        EngineId::ALL
            .iter()
            .copied()
            .find(|engine| engine.error_range().contains(&self.code))
    }
}

/// کدهای خطای استاندارد JSON-RPC 2.0 و کد خطای کرنل.
pub mod rpc_error_code {
    /// خطای تجزیه پیام.
    pub const PARSE_ERROR: i32 = -32700;
    /// درخواست نامعتبر.
    pub const INVALID_REQUEST: i32 = -32600;
    /// متد یافت نشد.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// پارامترها نامعتبر هستند.
    pub const INVALID_PARAMS: i32 = -32602;
    /// خطای داخلی.
    pub const INTERNAL_ERROR: i32 = -32603;
    /// خطای منشأ‌گرفته از کرنل؛ ساختار `ErrorPayload` در `data` حمل می‌شود.
    pub const KERNEL_ERROR: i32 = -32000;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn engine_ranges_are_disjoint_and_cover_1000_to_9999() {
        let mut seen = HashSet::new();
        for engine in EngineId::ALL {
            for code in engine.error_range() {
                assert!(seen.insert(code), "code {} overlaps between engines", code);
            }
        }
        assert_eq!(seen.len(), 9000);
        assert_eq!(*seen.iter().min().expect("non-empty"), 1000);
        assert_eq!(*seen.iter().max().expect("non-empty"), 9999);
    }

    #[test]
    fn engine_ids_roundtrip() {
        for engine in EngineId::ALL {
            assert_eq!(EngineId::parse(engine.as_str()), Some(*engine));
        }
        assert_eq!(EngineId::parse("unknown"), None);
        assert_eq!(EngineId::Foundation.to_string(), "foundation");
    }

    #[test]
    fn error_payload_derives_engine_from_code() {
        let payload = ErrorPayload::new(1001, "ConfigLoadFailed", "error.config.load_failed")
            .with_context(serde_json::json!({ "path": "kernel.json" }));
        assert_eq!(payload.engine(), Some(EngineId::Foundation));
        let path = payload
            .context
            .as_ref()
            .and_then(|context| context.get("path"))
            .and_then(serde_json::Value::as_str);
        assert_eq!(path, Some("kernel.json"));

        let domain = ErrorPayload::new(5001, "TradeNotFound", "error.trade.not_found");
        assert_eq!(domain.engine(), Some(EngineId::Domain));

        let unknown = ErrorPayload::new(99_999, "Mystery", "error.mystery");
        assert_eq!(unknown.engine(), None);
    }
}
