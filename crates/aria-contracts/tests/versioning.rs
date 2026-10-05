//! تست‌های کمک‌کننده‌های سازگاری نسخه‌بندی.

use aria_contracts::manifest::{PluginManifest, RuntimeMode};
use aria_contracts::versioning::{
    check_manifest_compatibility, is_api_compatible, is_event_version_supported,
    is_kernel_compatible, CompatibilityStatus, ContractVersions, DATABASE_SCHEMA_VERSION,
    KERNEL_CONTRACT_VERSION, PLUGIN_API_VERSION,
};
use semver::{Version, VersionReq};

fn compatible_manifest() -> PluginManifest {
    PluginManifest {
        id: "sample-analytics".to_string(),
        name: "نمونه تحلیل".to_string(),
        version: Version::new(1, 2, 0),
        api_version: Version::new(1, 0, 0),
        entrypoint: "main.py".to_string(),
        runtime_mode: RuntimeMode::OutOfProcess,
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
    }
}

#[test]
fn version_constants_are_consistent() {
    assert_eq!(
        aria_contracts::versioning::kernel_version().to_string(),
        KERNEL_CONTRACT_VERSION
    );
    assert_eq!(
        aria_contracts::versioning::plugin_api_version().to_string(),
        PLUGIN_API_VERSION
    );
    assert_eq!(KERNEL_CONTRACT_VERSION, "1.0.0");
    assert_eq!(PLUGIN_API_VERSION, "1.0.0");
    assert_eq!(DATABASE_SCHEMA_VERSION, 1);
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
    assert_eq!(
        check_manifest_compatibility(&compatible_manifest()),
        CompatibilityStatus::Compatible
    );

    let mut kernel_mismatch = compatible_manifest();
    kernel_mismatch.kernel_version_range = VersionReq::parse(">=2.0.0").unwrap();
    assert_eq!(
        check_manifest_compatibility(&kernel_mismatch),
        CompatibilityStatus::KernelMismatch
    );

    let mut api_mismatch = compatible_manifest();
    api_mismatch.api_version_range = VersionReq::parse("^2.0.0").unwrap();
    assert_eq!(
        check_manifest_compatibility(&api_mismatch),
        CompatibilityStatus::ApiMismatch
    );
}

#[test]
fn event_version_policy_allows_same_and_older_major_during_deprecation() {
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
    assert_eq!(
        versions.backup_format_version,
        aria_contracts::versioning::BACKUP_FORMAT_VERSION
    );
    assert_eq!(
        versions.event_envelope_version,
        aria_contracts::versioning::EVENT_ENVELOPE_VERSION
    );
}
