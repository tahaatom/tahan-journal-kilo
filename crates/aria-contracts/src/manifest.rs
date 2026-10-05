//! قرارداد مانیفست پلاگین.
//!
//! اسکیمای JSON مانیفست در `docs/contracts/plugin-manifest.schema.json`
//! تعریف شده و در اینجا با `include_str!` به‌صورت زمان کامپایل تعبیه می‌شود
//! تا منبع واحد حقیقت باقی بماند. اعتبارسنجی کامل اسکیما در موتور پلاگین
//! (فاز ۱.۷) انجام می‌شود؛ این ماژول فقط انواع و کمک‌کننده‌های پایه را
//! فراهم می‌کند.

use std::fmt;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

/// اسکیمای JSON مانیفست پلاگین
/// (منبع واحد حقیقت: `docs/contracts/plugin-manifest.schema.json`).
pub const PLUGIN_MANIFEST_SCHEMA: &str =
    include_str!("../../../docs/contracts/plugin-manifest.schema.json");

/// مانیفست پلاگین — قرارداد هویت، قابلیت و محدوده‌های پلاگین.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    /// شناسه یکتای پلاگین (حروف کوچک لاتین، اعداد و خط تیره).
    pub id: String,
    /// نام نمایشی پلاگین.
    pub name: String,
    /// نسخه معنایی پلاگین.
    pub version: Version,
    /// نسخه API پلاگین.
    pub api_version: Version,
    /// مسیر نقطه ورود پلاگین.
    pub entrypoint: String,
    /// حالت اجرا.
    pub runtime_mode: RuntimeMode,
    /// محدوده نسخه کرنل سازگار.
    pub kernel_version_range: VersionReq,
    /// محدوده نسخه API سازگار.
    pub api_version_range: VersionReq,
    /// قابلیت‌های مورد نیاز پلاگین.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    /// مجوزهای اعطایی (در نسخه ۱ همان واژگان قابلیت‌ها).
    #[serde(default)]
    pub permissions: Vec<String>,
    /// محدوده منابع پلاگین.
    #[serde(default)]
    pub resources: Option<ResourceLimits>,
    /// شناسه پلاگین‌های وابسته.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// زبان‌های پشتیبانی‌شده.
    #[serde(default)]
    pub locales: Vec<String>,
    /// نقاط افزونه UI اعلانی.
    #[serde(default)]
    pub ui_extension_points: Vec<UiExtensionPoint>,
    /// توضیحات.
    #[serde(default)]
    pub description: Option<String>,
    /// نویسنده.
    #[serde(default)]
    pub author: Option<String>,
    /// مجوز.
    #[serde(default)]
    pub license: Option<String>,
    /// سطح اعتماد پلاگین.
    #[serde(default)]
    pub trust_level: Option<TrustLevel>,
    /// سیاست سلامت پلاگین.
    #[serde(default)]
    pub health_policy: Option<HealthPolicy>,
}

impl PluginManifest {
    /// بررسی الگوی شناسه پلاگین (`^[a-z0-9][a-z0-9-]*$`).
    pub fn has_valid_id(&self) -> bool {
        is_valid_plugin_id(&self.id)
    }
}

/// اعتبارسنجی الگوی شناسه پلاگین.
pub fn is_valid_plugin_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() || first.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// حالت اجرای پلاگین.
///
/// پلاگین‌های قابل نصب پیش‌فرض `out_of_process` هستند؛ اجرای درون‌فرایندی
/// فقط برای ماژول‌های داخلی قابل اعتماد مجاز است.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeMode {
    #[serde(rename = "out_of_process")]
    OutOfProcess,
    #[serde(rename = "in_process_trusted")]
    InProcessTrusted,
}

/// قابلیت‌هایی که یک پلاگین می‌تواند درخواست کند.
///
/// فهرست نسخه ۱ بسته است؛ افزودن قابلیت جدید فقط از طریق ارتقاء
/// نسخه‌بندی‌شده قرارداد مجاز است.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    #[serde(rename = "trades.read")]
    TradesRead,
    #[serde(rename = "trades.write")]
    TradesWrite,
    #[serde(rename = "trades.delete")]
    TradesDelete,
    #[serde(rename = "fields.read")]
    FieldsRead,
    #[serde(rename = "fields.define")]
    FieldsDefine,
    #[serde(rename = "attachments.read")]
    AttachmentsRead,
    #[serde(rename = "attachments.write")]
    AttachmentsWrite,
    #[serde(rename = "stats.read")]
    StatsRead,
    #[serde(rename = "ui.widget")]
    UiWidget,
    #[serde(rename = "ui.page")]
    UiPage,
    #[serde(rename = "notifications.show")]
    NotificationsShow,
    #[serde(rename = "backup.create")]
    BackupCreate,
    #[serde(rename = "backup.restore")]
    BackupRestore,
    #[serde(rename = "mt.import")]
    MtImport,
    #[serde(rename = "mt.live")]
    MtLive,
    #[serde(rename = "insights.write")]
    InsightsWrite,
    #[serde(rename = "network.access")]
    NetworkAccess,
}

impl Capability {
    /// همه قابلیت‌های نسخه ۱.
    pub const ALL: &'static [Capability] = &[
        Capability::TradesRead,
        Capability::TradesWrite,
        Capability::TradesDelete,
        Capability::FieldsRead,
        Capability::FieldsDefine,
        Capability::AttachmentsRead,
        Capability::AttachmentsWrite,
        Capability::StatsRead,
        Capability::UiWidget,
        Capability::UiPage,
        Capability::NotificationsShow,
        Capability::BackupCreate,
        Capability::BackupRestore,
        Capability::MtImport,
        Capability::MtLive,
        Capability::InsightsWrite,
        Capability::NetworkAccess,
    ];

    /// نام پایدار قابلیت.
    pub fn as_str(self) -> &'static str {
        match self {
            Capability::TradesRead => "trades.read",
            Capability::TradesWrite => "trades.write",
            Capability::TradesDelete => "trades.delete",
            Capability::FieldsRead => "fields.read",
            Capability::FieldsDefine => "fields.define",
            Capability::AttachmentsRead => "attachments.read",
            Capability::AttachmentsWrite => "attachments.write",
            Capability::StatsRead => "stats.read",
            Capability::UiWidget => "ui.widget",
            Capability::UiPage => "ui.page",
            Capability::NotificationsShow => "notifications.show",
            Capability::BackupCreate => "backup.create",
            Capability::BackupRestore => "backup.restore",
            Capability::MtImport => "mt.import",
            Capability::MtLive => "mt.live",
            Capability::InsightsWrite => "insights.write",
            Capability::NetworkAccess => "network.access",
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// سطح اعتماد پلاگین.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Official,
    Certified,
    Community,
}

/// نقطه افزونه UI اعلانی (نسخه ۱).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UiExtensionPoint {
    #[serde(rename = "dashboard_widget")]
    DashboardWidget,
    #[serde(rename = "report_page")]
    ReportPage,
    #[serde(rename = "command_menu")]
    CommandMenu,
    #[serde(rename = "form_field")]
    FormField,
    #[serde(rename = "plugin_settings")]
    PluginSettings,
}

/// محدوده منابع پلاگین.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// حداکثر حافظه (مگابایت).
    pub max_memory_mb: u64,
    /// حداکثر مصرف CPU (درصد).
    pub max_cpu_percent: f64,
    /// حداکثر مدت اجرا (ثانیه).
    pub timeout_seconds: u64,
}

/// سیاست سلامت پلاگین.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthPolicy {
    /// تعداد دفعات خرابی پیش از قرنطینه.
    #[serde(default = "default_max_crashes_before_quarantine")]
    pub max_crashes_before_quarantine: u32,
}

fn default_max_crashes_before_quarantine() -> u32 {
    3
}

impl Default for HealthPolicy {
    fn default() -> Self {
        Self {
            max_crashes_before_quarantine: default_max_crashes_before_quarantine(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_names_use_dotted_vocabulary() {
        assert_eq!(Capability::TradesRead.as_str(), "trades.read");
        assert_eq!(Capability::NetworkAccess.as_str(), "network.access");
        assert_eq!(Capability::ALL.len(), 17);
        assert_eq!(
            Capability::NotificationsShow.to_string(),
            "notifications.show"
        );
    }

    #[test]
    fn plugin_id_pattern_is_enforced() {
        assert!(is_valid_plugin_id("mt-import"));
        assert!(is_valid_plugin_id("a"));
        assert!(is_valid_plugin_id("sample-analytics"));
        assert!(!is_valid_plugin_id(""));
        assert!(!is_valid_plugin_id("-leading-hyphen"));
        assert!(!is_valid_plugin_id("UPPERCASE"));
        assert!(!is_valid_plugin_id("with_underscore"));
        assert!(!is_valid_plugin_id("with space"));
    }

    #[test]
    fn health_policy_defaults_to_three_crashes() {
        assert_eq!(HealthPolicy::default().max_crashes_before_quarantine, 3);
    }

    #[test]
    fn manifest_deserializes_from_minimal_json() {
        let json = serde_json::json!({
            "id": "basic-reports",
            "name": "گزارش‌های پایه",
            "version": "1.0.0",
            "api_version": "1.0.0",
            "entrypoint": "main.py",
            "runtime_mode": "out_of_process",
            "kernel_version_range": ">=1.0.0, <2.0.0",
            "api_version_range": "^1.0.0"
        });
        let manifest: PluginManifest = serde_json::from_value(json).unwrap();
        assert_eq!(manifest.id, "basic-reports");
        assert!(manifest.capabilities.is_empty());
        assert!(manifest.resources.is_none());
        assert!(manifest.has_valid_id());
    }

    #[test]
    fn manifest_rejects_invalid_semver() {
        let json = serde_json::json!({
            "id": "broken",
            "name": "Broken",
            "version": "1.2",
            "api_version": "1.0.0",
            "entrypoint": "main.py",
            "runtime_mode": "out_of_process",
            "kernel_version_range": "*",
            "api_version_range": "*"
        });
        assert!(serde_json::from_value::<PluginManifest>(json).is_err());
    }
}
