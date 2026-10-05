//! قرارداد RPC پلاگین (JSON-RPC 2.0).
//!
//! پلاگین‌ها از طریق JSON-RPC 2.0 روی stdio یا IPC محلی با کرنل ارتباط
//! می‌برند:
//!
//! - پیام‌ها خط‌محور (newline-delimited JSON) هستند
//! - حداکثر اندازه پیام: ۱۰ مگابایت
//! - تایم‌اوت پیش‌فرض درخواست: ۳۰ ثانیه
//! - احراز هویت با نشانه نشست
//! - متدهای ناشناخته رد می‌شوند
//! - مجوزها به‌ازای هر متد اجرا می‌شوند

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use uuid::Uuid;

use crate::error::{rpc_error_code, ErrorPayload};
use crate::manifest::Capability;

/// حداکثر اندازه پیام RPC (۱۰ مگابایت).
pub const MAX_MESSAGE_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// تایم‌اوت پیش‌فرض درخواست RPC (۳۰ ثانیه).
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// نسخه پروتکل JSON-RPC؛ فقط مقدار «2.0» پذیرفته است.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonRpcVersion;

impl JsonRpcVersion {
    /// رشته نسخه پروتکل.
    pub const VERSION: &'static str = "2.0";
}

impl Serialize for JsonRpcVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(Self::VERSION)
    }
}

impl<'de> Deserialize<'de> for JsonRpcVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == Self::VERSION {
            Ok(JsonRpcVersion)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported JSON-RPC version {}, only {} is accepted",
                value,
                Self::VERSION
            )))
        }
    }
}

/// شناسه پیام JSON-RPC (عدد یا رشته).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcId {
    Number(i64),
    Text(String),
}

/// درخواست JSON-RPC 2.0.
///
/// در صورت نبود یا null بودن `id`، پیام یک اطلاع‌رسانی (notification) است
/// و کرنل پاسخی برای آن ارسال نمی‌کند.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcRequest {
    /// نسخه پروتکل (باید «2.0» باشد).
    pub jsonrpc: JsonRpcVersion,
    /// نام متد.
    pub method: String,
    /// پارامترها (اختیاری).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// شناسه پیام (اختیاری؛ نبودن آن اطلاع‌رسانی است).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<RpcId>,
}

impl RpcRequest {
    /// یک درخواست با شناسه مشخص می‌سازد.
    pub fn new(id: Option<RpcId>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            method: method.into(),
            params,
            id,
        }
    }

    /// یک اطلاع‌رسانی (درخواست بدون شناسه) می‌سازد.
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self::new(None, method, params)
    }

    /// آیا این پیام یک اطلاع‌رسانی است؟
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// پاسخ موفق JSON-RPC 2.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcResponse {
    /// نسخه پروتکل.
    pub jsonrpc: JsonRpcVersion,
    /// شناسه پیام مرتبط.
    pub id: Option<RpcId>,
    /// نتیجه.
    pub result: Value,
}

impl RpcResponse {
    /// یک پاسخ موفق می‌سازد.
    pub fn new(id: Option<RpcId>, result: Value) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            id,
            result,
        }
    }
}

/// خطای JSON-RPC 2.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcError {
    /// کد خطا.
    pub code: i32,
    /// پیام خطا.
    pub message: String,
    /// داده‌های اختیاری خطا (برای خطاهای کرنل، ساختار `ErrorPayload` است).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// پاسخ خطای JSON-RPC 2.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcErrorResponse {
    /// نسخه پروتکل.
    pub jsonrpc: JsonRpcVersion,
    /// شناسه پیام مرتبط.
    pub id: Option<RpcId>,
    /// خطا.
    pub error: RpcError,
}

impl RpcErrorResponse {
    /// یک پاسخ خطای ساخته‌شده دستی می‌سازد.
    pub fn new(id: Option<RpcId>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion,
            id,
            error: RpcError {
                code,
                message: message.into(),
                data: None,
            },
        }
    }

    /// داده خطا را تعیین می‌کند.
    pub fn with_data(mut self, data: Value) -> Self {
        self.error.data = Some(data);
        self
    }

    /// خطای تجزیه پیام.
    pub fn parse_error(id: Option<RpcId>) -> Self {
        Self::new(id, rpc_error_code::PARSE_ERROR, "parse error")
    }

    /// درخواست نامعتبر.
    pub fn invalid_request(id: Option<RpcId>) -> Self {
        Self::new(id, rpc_error_code::INVALID_REQUEST, "invalid request")
    }

    /// متد یافت نشد.
    pub fn method_not_found(id: Option<RpcId>) -> Self {
        Self::new(id, rpc_error_code::METHOD_NOT_FOUND, "method not found")
    }

    /// پارامترها نامعتبر هستند.
    pub fn invalid_params(id: Option<RpcId>, message: impl Into<String>) -> Self {
        Self::new(id, rpc_error_code::INVALID_PARAMS, message)
    }

    /// خطای داخلی کرنل.
    pub fn internal_error(id: Option<RpcId>) -> Self {
        Self::new(id, rpc_error_code::INTERNAL_ERROR, "internal error")
    }

    /// خطای منشأ‌گرفته از کرنل: کد `-32000` و ساختار `ErrorPayload` در `data`.
    pub fn kernel_error(id: Option<RpcId>, payload: &ErrorPayload) -> Self {
        let data = serde_json::to_value(payload).unwrap_or(Value::Null);
        Self::new(id, rpc_error_code::KERNEL_ERROR, "kernel error").with_data(data)
    }
}

/// انواع پیام‌های JSON-RPC 2.0.
#[derive(Debug, Clone, PartialEq)]
pub enum RpcMessage {
    Request(RpcRequest),
    Response(RpcResponse),
    ErrorResponse(RpcErrorResponse),
}

impl RpcMessage {
    /// یک پیام JSON را به‌عنوان پیام JSON-RPC 2.0 تحلیل می‌کند.
    ///
    /// پیام‌های بزرگ‌تر از [`MAX_MESSAGE_SIZE_BYTES`]، پیام‌های JSON نامعتبر
    /// و پیام‌هایی با نسخه پروتکل یا ساختار نامعتبر رد می‌شوند.
    pub fn parse(text: &str) -> Result<Self, RpcMessageError> {
        if text.len() > MAX_MESSAGE_SIZE_BYTES {
            return Err(RpcMessageError::MessageTooLarge);
        }
        let value: Value = serde_json::from_str(text).map_err(|_| RpcMessageError::ParseError)?;
        Self::from_json(&value)
    }

    /// یک مقدار JSON را به‌عنوان پیام JSON-RPC 2.0 تفسیر می‌کند.
    pub fn from_json(value: &Value) -> Result<Self, RpcMessageError> {
        let object = value.as_object().ok_or(RpcMessageError::InvalidRequest)?;
        let version = object
            .get("jsonrpc")
            .and_then(Value::as_str)
            .ok_or(RpcMessageError::InvalidRequest)?;
        if version != JsonRpcVersion::VERSION {
            return Err(RpcMessageError::InvalidRequest);
        }
        if object.contains_key("method") {
            let request: RpcRequest = serde_json::from_value(value.clone())
                .map_err(|_| RpcMessageError::InvalidRequest)?;
            Ok(RpcMessage::Request(request))
        } else if object.contains_key("result") {
            let response: RpcResponse = serde_json::from_value(value.clone())
                .map_err(|_| RpcMessageError::InvalidRequest)?;
            Ok(RpcMessage::Response(response))
        } else if object.contains_key("error") {
            let response: RpcErrorResponse = serde_json::from_value(value.clone())
                .map_err(|_| RpcMessageError::InvalidRequest)?;
            Ok(RpcMessage::ErrorResponse(response))
        } else {
            Err(RpcMessageError::InvalidRequest)
        }
    }
}

/// خطاهای تحلیل پیام RPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcMessageError {
    /// پیام از حداکثر اندازه مجاوزت دارد.
    MessageTooLarge,
    /// پیام JSON نامعتبر است.
    ParseError,
    /// ساختار پیام JSON-RPC نامعتبر است.
    InvalidRequest,
}

impl fmt::Display for RpcMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcMessageError::MessageTooLarge => {
                write!(f, "RPC message exceeds {} bytes", MAX_MESSAGE_SIZE_BYTES)
            }
            RpcMessageError::ParseError => f.write_str("RPC message is not valid JSON"),
            RpcMessageError::InvalidRequest => f.write_str("invalid JSON-RPC 2.0 message"),
        }
    }
}

impl std::error::Error for RpcMessageError {}

/// نشانه نشست پلاگین — احراز هویت پلاگین در RPC.
///
/// نشانه‌ها UUID v4 هستند و برای هر نشست اجرای پلاگین تولید می‌شوند.
/// مقایسه ثابت‌زمانی نشانه‌ها در موتور اجرا (فاز ۱.۸) اعمال می‌شود.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionToken(String);

impl SessionToken {
    /// نشانه نشست جدید (UUID v4).
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// مقدار رشته‌ای نشانه.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionToken {
    fn default() -> Self {
        Self::new()
    }
}

/// توصیف یک متد کرنل در دفترچه روش‌ها (method registry).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodDescriptor {
    /// نام پایدار متد با قالب `domain.action`.
    pub method: &'static str,
    /// قابلیت مورد نیاز برای اجرای متد؛ `None` یعنی متد عمومی است.
    pub required_capability: Option<Capability>,
    /// توضیح کوتاه متد.
    pub description: &'static str,
}

/// دفترچه روش‌های کرنل در نسخه ۱.
///
/// قرارداد دفترچه روش‌ها:
///
/// - نام متدها باید با قالب `domain.action` باشد
/// - هر متد یا دقیقاً یک قابلیت مورد نیاز دارد یا عمومی است
/// - متدهای ناشناخته باید رد شوند
/// - مجوزها به‌ازای هر متد اجرا می‌شوند
///
/// قابلیت‌های منتقل‌شده (`mt.live`، `insights.write`، `network.access`،
/// `ui.widget` و `ui.page`) تا فازهای مربوطه متدی در این دفترچه ندارند.
pub const KERNEL_METHODS: &[MethodDescriptor] = &[
    MethodDescriptor {
        method: "kernel.ping",
        required_capability: None,
        description: "بررسی سلامت کرنل",
    },
    MethodDescriptor {
        method: "kernel.version",
        required_capability: None,
        description: "اطلاعات نسخه کرنل و قراردادها",
    },
    MethodDescriptor {
        method: "trades.list",
        required_capability: Some(Capability::TradesRead),
        description: "فهرست معاملات با فیلتر و صفحه‌بندی",
    },
    MethodDescriptor {
        method: "trades.get",
        required_capability: Some(Capability::TradesRead),
        description: "جزئیات یک معامله",
    },
    MethodDescriptor {
        method: "trades.create",
        required_capability: Some(Capability::TradesWrite),
        description: "ثبت معامله",
    },
    MethodDescriptor {
        method: "trades.update",
        required_capability: Some(Capability::TradesWrite),
        description: "ویرایش معامله",
    },
    MethodDescriptor {
        method: "trades.delete",
        required_capability: Some(Capability::TradesDelete),
        description: "حذف معامله",
    },
    MethodDescriptor {
        method: "fields.list",
        required_capability: Some(Capability::FieldsRead),
        description: "فهرست فیلدهای سفارشی",
    },
    MethodDescriptor {
        method: "fields.get",
        required_capability: Some(Capability::FieldsRead),
        description: "جزئیات یک فیلد سفارشی",
    },
    MethodDescriptor {
        method: "fields.define",
        required_capability: Some(Capability::FieldsDefine),
        description: "تعریف فیلد سفارشی جدید",
    },
    MethodDescriptor {
        method: "fields.update",
        required_capability: Some(Capability::FieldsDefine),
        description: "ویرایش تعریف فیلد سفارشی",
    },
    MethodDescriptor {
        method: "fields.deactivate",
        required_capability: Some(Capability::FieldsDefine),
        description: "غیرفعال‌سازی فیلد سفارشی",
    },
    MethodDescriptor {
        method: "attachments.list",
        required_capability: Some(Capability::AttachmentsRead),
        description: "فهرست پیوست‌های یک معامله",
    },
    MethodDescriptor {
        method: "attachments.get",
        required_capability: Some(Capability::AttachmentsRead),
        description: "دریافت متادیتا یا محتوای پیوست",
    },
    MethodDescriptor {
        method: "attachments.upload",
        required_capability: Some(Capability::AttachmentsWrite),
        description: "بارگذاری پیوست",
    },
    MethodDescriptor {
        method: "stats.summary",
        required_capability: Some(Capability::StatsRead),
        description: "خلاصه آمار داشبورد",
    },
    MethodDescriptor {
        method: "stats.query",
        required_capability: Some(Capability::StatsRead),
        description: "پرس‌وجوی تجمیعی",
    },
    MethodDescriptor {
        method: "backup.create",
        required_capability: Some(Capability::BackupCreate),
        description: "ایجاد پشتیبان",
    },
    MethodDescriptor {
        method: "backup.restore",
        required_capability: Some(Capability::BackupRestore),
        description: "بازگردانی پشتیبان",
    },
    MethodDescriptor {
        method: "mt.import",
        required_capability: Some(Capability::MtImport),
        description: "واردات داده متاتریدر",
    },
    MethodDescriptor {
        method: "notifications.show",
        required_capability: Some(Capability::NotificationsShow),
        description: "نمایش اطلاع‌رسانی به کاربر",
    },
];

/// یافتن توصیف متد بر اساس نام.
pub fn find_method(method: &str) -> Option<&'static MethodDescriptor> {
    KERNEL_METHODS
        .iter()
        .find(|descriptor| descriptor.method == method)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_roundtrips_through_json() {
        let request = RpcRequest::new(
            Some(RpcId::Number(42)),
            "trades.list",
            Some(serde_json::json!({
                "filter": {},
                "pagination": { "limit": 50, "offset": 0 }
            })),
        );
        let json = serde_json::to_string(&request).unwrap();
        let parsed: RpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, parsed);
        assert!(!request.is_notification());
    }

    #[test]
    fn notification_has_no_id_and_roundtrips() {
        let notification = RpcRequest::notification("kernel.ping", None);
        assert!(notification.is_notification());
        let json = serde_json::to_string(&notification).unwrap();
        assert!(!json.contains("\"id\""));
        let parsed: RpcRequest = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_notification());
    }

    #[test]
    fn response_and_error_roundtrip() {
        let response = RpcResponse::new(
            Some(RpcId::Text("abc".into())),
            serde_json::json!({ "ok": true }),
        );
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(
            serde_json::from_str::<RpcResponse>(&json).unwrap(),
            response
        );

        let error = RpcErrorResponse::method_not_found(Some(RpcId::Number(7)));
        let json = serde_json::to_string(&error).unwrap();
        let parsed: RpcErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.error.code, rpc_error_code::METHOD_NOT_FOUND);
    }

    #[test]
    fn kernel_error_response_carries_error_payload() {
        let payload = ErrorPayload::new(5001, "TradeNotFound", "error.trade.not_found");
        let response = RpcErrorResponse::kernel_error(Some(RpcId::Number(1)), &payload);
        assert_eq!(response.error.code, rpc_error_code::KERNEL_ERROR);
        let data = response.error.data.expect("data must be present");
        assert_eq!(data["variant"], "TradeNotFound");
        assert_eq!(data["message_key"], "error.trade.not_found");
        assert_eq!(data["code"], 5001);
    }

    #[test]
    fn message_parsing_classifies_all_kinds() {
        let request =
            RpcMessage::parse(r#"{"jsonrpc":"2.0","id":1,"method":"kernel.ping"}"#).unwrap();
        assert!(matches!(request, RpcMessage::Request(_)));

        let response = RpcMessage::parse(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#).unwrap();
        assert!(matches!(response, RpcMessage::Response(_)));

        let error = RpcMessage::parse(
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"method not found"}}"#,
        )
        .unwrap();
        assert!(matches!(error, RpcMessage::ErrorResponse(_)));
    }

    #[test]
    fn invalid_messages_are_rejected() {
        assert_eq!(
            RpcMessage::parse("not json"),
            Err(RpcMessageError::ParseError)
        );
        assert_eq!(
            RpcMessage::parse(r#"{"jsonrpc":"1.0","id":1,"method":"x"}"#),
            Err(RpcMessageError::InvalidRequest)
        );
        assert_eq!(
            RpcMessage::parse(r#"{"id":1}"#),
            Err(RpcMessageError::InvalidRequest)
        );
        assert_eq!(
            RpcMessage::parse(r#"[1,2,3]"#),
            Err(RpcMessageError::InvalidRequest)
        );
        let huge = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"x\",\"params\":\"{}\"}}",
            "a".repeat(MAX_MESSAGE_SIZE_BYTES)
        );
        assert_eq!(
            RpcMessage::parse(&huge),
            Err(RpcMessageError::MessageTooLarge)
        );
    }

    #[test]
    fn session_tokens_are_unique() {
        let first = SessionToken::new();
        let second = SessionToken::new();
        assert_ne!(first, second);
        assert_eq!(first.as_str().len(), 36);
    }

    #[test]
    fn method_registry_contract_holds() {
        assert!(find_method("kernel.ping").is_some());
        assert!(find_method("trades.create").is_some());
        assert!(find_method("no.such_method").is_none());
        for descriptor in KERNEL_METHODS {
            let parts: Vec<&str> = descriptor.method.split('.').collect();
            assert_eq!(
                parts.len(),
                2,
                "method {} must be namespaced as domain.action",
                descriptor.method
            );
        }
        let deferred = [
            Capability::MtLive,
            Capability::InsightsWrite,
            Capability::NetworkAccess,
            Capability::UiWidget,
            Capability::UiPage,
        ];
        for capability in Capability::ALL {
            let has_method = KERNEL_METHODS
                .iter()
                .any(|descriptor| descriptor.required_capability == Some(*capability));
            if deferred.contains(capability) {
                assert!(
                    !has_method,
                    "deferred capability {} must not have a v1 method",
                    capability
                );
            } else {
                assert!(
                    has_method,
                    "capability {} must have at least one v1 method",
                    capability
                );
            }
        }
    }
}
