//! قرارداد پاکت دستور (Command Envelope).
//!
//! دستورات از طریق IPC امن Tauri صادر می‌شوند، اتمیک هستند و پس از اجرای
//! موفق رویدادهای دامنه‌ای منتشر می‌کنند. سطح دستورهای نسخه ۱ بسته است و
//! دقیقاً همان دستورهایی است که در قرارداد دامنه تعریف شده‌اند؛ افزودن
//! دستور جدید فقط از طریق ارتقاء نسخه‌بندی‌شده قرارداد مجاز است.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// پاکت دستور — ساختار پایدار صدور دستور به کرنل.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEnvelope {
    /// شناسه یکتای دستور (UUID v4).
    pub command_id: Uuid,
    /// نوع دستور.
    pub command_type: CommandType,
    /// بار دستور به‌صورت JSON.
    pub payload: Value,
    /// صادرکننده دستور.
    pub issuer: CommandIssuer,
    /// زمان صدور (UTC، ISO 8601).
    pub timestamp: DateTime<Utc>,
    /// شناسه همبستگی برای ردیابی زنجیره دستور-رویداد (اختیاری).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
}

impl CommandEnvelope {
    /// یک پاکت دستور جدید با شناسه و زمان تولیدشده می‌سازد.
    pub fn new(command_type: CommandType, payload: Value, issuer: CommandIssuer) -> Self {
        Self {
            command_id: Uuid::new_v4(),
            command_type,
            payload,
            issuer,
            timestamp: Utc::now(),
            correlation_id: None,
        }
    }

    /// شناسه همبستگی را تعیین می‌کند.
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

/// انواع دستورهای نسخه ۱.
///
/// این سطح بسته با دستورهای قرارداد دامنه (فاز ۱.۶) یکی است. نوع
/// ناشناخته در مرز ورودی رد می‌شود.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandType {
    #[serde(rename = "create_trade")]
    CreateTrade,
    #[serde(rename = "update_trade")]
    UpdateTrade,
    #[serde(rename = "delete_trade")]
    DeleteTrade,
    #[serde(rename = "add_entry_leg")]
    AddEntryLeg,
    #[serde(rename = "update_entry_leg")]
    UpdateEntryLeg,
    #[serde(rename = "add_exit_leg")]
    AddExitLeg,
    #[serde(rename = "update_exit_leg")]
    UpdateExitLeg,
    #[serde(rename = "assign_execution_to_leg")]
    AssignExecutionToLeg,
    #[serde(rename = "add_manual_override")]
    AddManualOverride,
    #[serde(rename = "revert_manual_override")]
    RevertManualOverride,
    #[serde(rename = "link_attachment_to_trade")]
    LinkAttachmentToTrade,
}

impl CommandType {
    /// نام پایدار نوع دستور.
    pub fn as_str(self) -> &'static str {
        match self {
            CommandType::CreateTrade => "create_trade",
            CommandType::UpdateTrade => "update_trade",
            CommandType::DeleteTrade => "delete_trade",
            CommandType::AddEntryLeg => "add_entry_leg",
            CommandType::UpdateEntryLeg => "update_entry_leg",
            CommandType::AddExitLeg => "add_exit_leg",
            CommandType::UpdateExitLeg => "update_exit_leg",
            CommandType::AssignExecutionToLeg => "assign_execution_to_leg",
            CommandType::AddManualOverride => "add_manual_override",
            CommandType::RevertManualOverride => "revert_manual_override",
            CommandType::LinkAttachmentToTrade => "link_attachment_to_trade",
        }
    }
}

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// نوع صادرکننده دستور.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuerKind {
    /// کاربر انسانی از طریق رابط کاربری.
    User,
    /// خود کرنل.
    System,
    /// پلاگین از طریق RPC.
    Plugin,
}

/// صادرکننده دستور.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandIssuer {
    /// نوع صادرکننده.
    pub kind: IssuerKind,
    /// شناسه اصلی صادرکننده: شناسه پروفایل برای کاربر، رشته `kernel` برای
    /// سیستم یا شناسه پلاگین برای پلاگین.
    pub principal_id: String,
}

impl CommandIssuer {
    /// صادرکننده کاربری.
    pub fn user(profile_id: Uuid) -> Self {
        Self {
            kind: IssuerKind::User,
            principal_id: profile_id.to_string(),
        }
    }

    /// صادرکننده سیستمی (خود کرنل).
    pub fn system() -> Self {
        Self {
            kind: IssuerKind::System,
            principal_id: "kernel".to_string(),
        }
    }

    /// صادرکننده پلاگینی.
    pub fn plugin(plugin_id: &str) -> Self {
        Self {
            kind: IssuerKind::Plugin,
            principal_id: plugin_id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_type_names_are_stable() {
        assert_eq!(CommandType::CreateTrade.as_str(), "create_trade");
        assert_eq!(
            CommandType::AssignExecutionToLeg.as_str(),
            "assign_execution_to_leg"
        );
        assert_eq!(
            CommandType::LinkAttachmentToTrade.to_string(),
            "link_attachment_to_trade"
        );
    }

    #[test]
    fn issuer_constructors_set_principals() {
        let profile_id = Uuid::new_v4();
        assert_eq!(
            CommandIssuer::user(profile_id).principal_id,
            profile_id.to_string()
        );
        assert_eq!(CommandIssuer::system().principal_id, "kernel");
        assert_eq!(CommandIssuer::plugin("mt-import").principal_id, "mt-import");
    }

    #[test]
    fn new_envelope_generates_unique_ids() {
        let first = CommandEnvelope::new(
            CommandType::CreateTrade,
            Value::Null,
            CommandIssuer::system(),
        );
        let second = CommandEnvelope::new(
            CommandType::CreateTrade,
            Value::Null,
            CommandIssuer::system(),
        );
        assert_ne!(first.command_id, second.command_id);
        assert!(first.correlation_id.is_none());
    }
}
