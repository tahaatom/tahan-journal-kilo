//! سیاست نسخه‌بندی کرنل آریا.
//!
//! چهار نسخه پایدار وجود دارد:
//!
//! - نسخه قرارداد کرنل (kernel contract version)
//! - نسخه API پلاگین (plugin API version)
//! - نسخه اسکیمای دیتابیس (database schema version)
//! - نسخه فرمت پشتیبان (backup format version)
//!
//! علاوه بر این‌ها، نسخه پاکت رویداد (event envelope version) شکل
//! بار رویدادها را نسخه‌بندی می‌کند.

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::manifest::PluginManifest;

/// نسخه قرارداد کرنل آریا.
pub const KERNEL_CONTRACT_VERSION: &str = "1.0.0";

/// نسخه API پلاگین.
pub const PLUGIN_API_VERSION: &str = "1.0.0";

/// نسخه اسکیمای پایگاه داده (PRAGMA user_version / refinery).
pub const DATABASE_SCHEMA_VERSION: u32 = 1;

/// نسخه فرمت پشتیبان.
pub const BACKUP_FORMAT_VERSION: u32 = 1;

/// نسخه پاکت رویداد (شکل بار رویداد).
pub const EVENT_ENVELOPE_VERSION: u32 = 1;

/// نسخه قرارداد کرنل به‌عنوان یک نسخه معنایی.
pub fn kernel_version() -> Version {
    Version::parse(KERNEL_CONTRACT_VERSION).expect("kernel contract version must be valid semver")
}

/// نسخه API پلاگین به‌عنوان یک نسخه معنایی.
pub fn plugin_api_version() -> Version {
    Version::parse(PLUGIN_API_VERSION).expect("plugin API version must be valid semver")
}

/// وضعیت سازگاری یک پلاگین با کرنل.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityStatus {
    /// هر دو محدوده سازگار هستند.
    Compatible,
    /// محدوده نسخه کرنل سازگار نیست.
    KernelMismatch,
    /// محدوده نسخه API سازگار نیست.
    ApiMismatch,
}

/// سازگاری محدوده نسخه با نسخه کرنل را بررسی می‌کند.
pub fn is_kernel_compatible(required: &VersionReq) -> bool {
    required.matches(&kernel_version())
}

/// سازگاری محدوده نسخه API با نسخه API کرنل را بررسی می‌کند.
pub fn is_api_compatible(required: &VersionReq) -> bool {
    required.matches(&plugin_api_version())
}

/// سازگاری مانیفست پلاگین با کرنل فعلی را بررسی می‌کند.
pub fn check_manifest_compatibility(manifest: &PluginManifest) -> CompatibilityStatus {
    if !is_kernel_compatible(&manifest.kernel_version_range) {
        CompatibilityStatus::KernelMismatch
    } else if !is_api_compatible(&manifest.api_version_range) {
        CompatibilityStatus::ApiMismatch
    } else {
        CompatibilityStatus::Compatible
    }
}

/// بررسی اینکه نسخه رویدادی توسط مصرف‌کننده پشتیبانی می‌شود.
///
/// قرارداد رویدادها: مصرف‌کننده باید رویدادهای همین نسخه اصلی و
/// نسخه‌های اصلی قدیمی‌تر را تا پایان دوره منقضی‌سازی پشتیبانی کند.
pub fn is_event_version_supported(event_version: u32, supported_major: u32) -> bool {
    event_version <= supported_major
}

/// اطلاعات نسخه‌های کرنل برای روش `kernel.version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractVersions {
    /// نسخه قرارداد کرنل.
    pub kernel_contract_version: &'static str,
    /// نسخه API پلاگین.
    pub plugin_api_version: &'static str,
    /// نسخه اسکیمای پایگاه داده.
    pub database_schema_version: u32,
    /// نسخه فرمت پشتیبان.
    pub backup_format_version: u32,
    /// نسخه پاکت رویداد.
    pub event_envelope_version: u32,
}

impl ContractVersions {
    /// اسنپ‌شات نسخه‌های فعلی کرنل.
    pub fn current() -> Self {
        Self {
            kernel_contract_version: KERNEL_CONTRACT_VERSION,
            plugin_api_version: PLUGIN_API_VERSION,
            database_schema_version: DATABASE_SCHEMA_VERSION,
            backup_format_version: BACKUP_FORMAT_VERSION,
            event_envelope_version: EVENT_ENVELOPE_VERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constants_are_consistent() {
        assert_eq!(kernel_version().to_string(), KERNEL_CONTRACT_VERSION);
        assert_eq!(plugin_api_version().to_string(), PLUGIN_API_VERSION);
        assert_eq!(KERNEL_CONTRACT_VERSION, "1.0.0");
        assert_eq!(PLUGIN_API_VERSION, "1.0.0");
        assert_eq!(DATABASE_SCHEMA_VERSION, 1);
        assert_eq!(BACKUP_FORMAT_VERSION, 1);
        assert_eq!(EVENT_ENVELOPE_VERSION, 1);
    }

    #[test]
    fn compatible_ranges_are_accepted() {
        assert!(is_kernel_compatible(
            &VersionReq::parse(">=1.0.0, <2.0.0").unwrap()
        ));
        assert!(is_kernel_compatible(&VersionReq::parse("^1.0.0").unwrap()));
        assert!(is_kernel_compatible(&VersionReq::parse("~1.0").unwrap()));
        assert!(is_kernel_compatible(&VersionReq::parse("*").unwrap()));
        assert!(is_api_compatible(&VersionReq::parse("^1.0.0").unwrap()));
    }

    #[test]
    fn incompatible_ranges_are_rejected() {
        assert!(!is_kernel_compatible(
            &VersionReq::parse(">=2.0.0").unwrap()
        ));
        assert!(!is_kernel_compatible(&VersionReq::parse("<1.0.0").unwrap()));
        assert!(!is_api_compatible(&VersionReq::parse("^2.0.0").unwrap()));
    }

    #[test]
    fn manifest_compatibility_is_reported() {
        let manifest = PluginManifest {
            id: "sample-analytics".to_string(),
            name: "نمونه تحلیل".to_string(),
            version: Version::new(1, 2, 0),
            api_version: Version::new(1, 0, 0),
            entrypoint: "main.py".to_string(),
            runtime_mode: crate::manifest::RuntimeMode::OutOfProcess,
            kernel_version_range: VersionReq::parse(">=1.0.0, <2.0.0").unwrap(),
            api_version_range: VersionReq::parse("^1.0.0").unwrap(),
            capabilities: Vec::new(),
            permissions: Vec::new(),
            resources: None,
            dependencies: Vec::new(),
            locales: Vec::new(),
            ui_extension_points: Vec::new(),
            description: None,
            author: None,
            license: None,
            trust_level: None,
            health_policy: None,
        };
        assert_eq!(
            check_manifest_compatibility(&manifest),
            CompatibilityStatus::Compatible
        );

        let mut kernel_mismatch = manifest.clone();
        kernel_mismatch.kernel_version_range = VersionReq::parse(">=2.0.0").unwrap();
        assert_eq!(
            check_manifest_compatibility(&kernel_mismatch),
            CompatibilityStatus::KernelMismatch
        );

        let mut api_mismatch = manifest;
        api_mismatch.api_version_range = VersionReq::parse("^2.0.0").unwrap();
        assert_eq!(
            check_manifest_compatibility(&api_mismatch),
            CompatibilityStatus::ApiMismatch
        );
    }

    #[test]
    fn event_version_policy_allows_same_and_older_major() {
        assert!(is_event_version_supported(1, 1));
        assert!(is_event_version_supported(1, 2));
        assert!(!is_event_version_supported(2, 1));
    }

    #[test]
    fn contract_versions_snapshot_is_complete() {
        let versions = ContractVersions::current();
        assert_eq!(versions.kernel_contract_version, KERNEL_CONTRACT_VERSION);
        assert_eq!(versions.plugin_api_version, PLUGIN_API_VERSION);
        assert_eq!(versions.database_schema_version, DATABASE_SCHEMA_VERSION);
        assert_eq!(versions.backup_format_version, BACKUP_FORMAT_VERSION);
        assert_eq!(versions.event_envelope_version, EVENT_ENVELOPE_VERSION);
    }
}
