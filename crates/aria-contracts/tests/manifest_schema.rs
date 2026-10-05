//! تست‌های یکپارچه اسکیمای مانیفست پلاگین.

use aria_contracts::manifest::{
    is_valid_plugin_id, Capability, PluginManifest, ResourceLimits, RuntimeMode, TrustLevel,
    UiExtensionPoint, PLUGIN_MANIFEST_SCHEMA,
};
use jsonschema::JSONSchema;
use serde_json::{json, Value};

fn validator() -> JSONSchema {
    let schema: Value =
        serde_json::from_str(PLUGIN_MANIFEST_SCHEMA).expect("embedded schema must be valid JSON");
    JSONSchema::compile(&schema).expect("embedded schema must compile")
}

fn sample_manifest() -> Value {
    json!({
        "id": "sample-analytics",
        "name": "نمونه تحلیل",
        "version": "1.2.0",
        "api_version": "1.0.0",
        "entrypoint": "main.py",
        "runtime_mode": "out_of_process",
        "kernel_version_range": ">=1.0.0, <2.0.0",
        "api_version_range": "^1.0.0",
        "capabilities": ["stats.read"],
        "permissions": ["stats.read"],
        "resources": { "max_memory_mb": 256, "max_cpu_percent": 50.0, "timeout_seconds": 60 },
        "dependencies": [],
        "locales": ["fa", "en"],
        "ui_extension_points": ["dashboard_widget"],
        "description": "پلاگین نمونه",
        "author": "Tahan",
        "license": "MIT",
        "trust_level": "official",
        "health_policy": { "max_crashes_before_quarantine": 3 }
    })
}

fn validation_errors(validator: &JSONSchema, instance: &Value) -> Vec<String> {
    match validator.validate(instance) {
        Ok(()) => Vec::new(),
        Err(errors) => errors.map(|error| error.to_string()).collect(),
    }
}

#[test]
fn sample_manifest_validates_against_the_schema() {
    let validator = validator();
    let errors = validation_errors(&validator, &sample_manifest());
    assert!(
        errors.is_empty(),
        "sample manifest must be valid: {:?}",
        errors
    );
}

#[test]
fn minimal_manifest_validates() {
    let validator = validator();
    let minimal = json!({
        "id": "basic-reports",
        "name": "گزارش‌های پایه",
        "version": "1.0.0",
        "api_version": "1.0.0",
        "entrypoint": "main.py",
        "runtime_mode": "out_of_process",
        "kernel_version_range": "*",
        "api_version_range": "*"
    });
    let errors = validation_errors(&validator, &minimal);
    assert!(
        errors.is_empty(),
        "minimal manifest must be valid: {:?}",
        errors
    );
}

#[test]
fn missing_required_field_is_rejected() {
    let validator = validator();
    for field in [
        "id",
        "name",
        "version",
        "api_version",
        "entrypoint",
        "runtime_mode",
        "kernel_version_range",
        "api_version_range",
    ] {
        let mut manifest = sample_manifest();
        manifest.as_object_mut().expect("object").remove(field);
        let errors = validation_errors(&validator, &manifest);
        assert!(!errors.is_empty(), "missing {} must be rejected", field);
    }
}

#[test]
fn invalid_plugin_id_is_rejected() {
    let validator = validator();
    for id in ["Bad-ID", "-leading", "with_underscore", ""] {
        let mut manifest = sample_manifest();
        manifest["id"] = json!(id);
        let errors = validation_errors(&validator, &manifest);
        assert!(!errors.is_empty(), "id {:?} must be rejected", id);
    }
}

#[test]
fn invalid_semver_is_rejected() {
    let validator = validator();
    for version in ["1.2", "v1.2.0", "1.2.0.0"] {
        let mut manifest = sample_manifest();
        manifest["version"] = json!(version);
        let errors = validation_errors(&validator, &manifest);
        assert!(!errors.is_empty(), "version {} must be rejected", version);
    }
}

#[test]
fn unknown_capability_is_rejected() {
    let validator = validator();
    let mut manifest = sample_manifest();
    manifest["capabilities"] = json!(["trades.nuke"]);
    assert!(!validation_errors(&validator, &manifest).is_empty());
}

#[test]
fn duplicate_capability_is_rejected() {
    let validator = validator();
    let mut manifest = sample_manifest();
    manifest["capabilities"] = json!(["stats.read", "stats.read"]);
    assert!(!validation_errors(&validator, &manifest).is_empty());
}

#[test]
fn unknown_runtime_mode_is_rejected() {
    let validator = validator();
    let mut manifest = sample_manifest();
    manifest["runtime_mode"] = json!("in_process");
    assert!(!validation_errors(&validator, &manifest).is_empty());
}

#[test]
fn unknown_ui_extension_point_is_rejected() {
    let validator = validator();
    let mut manifest = sample_manifest();
    manifest["ui_extension_points"] = json!(["sidebar_panel"]);
    assert!(!validation_errors(&validator, &manifest).is_empty());
}

#[test]
fn unknown_health_policy_property_is_rejected() {
    let validator = validator();
    let mut manifest = sample_manifest();
    manifest["health_policy"] =
        json!({ "max_crashes_before_quarantine": 3, "restart_policy": "always" });
    assert!(!validation_errors(&validator, &manifest).is_empty());
}

#[test]
fn unknown_top_level_property_is_rejected() {
    let validator = validator();
    let mut manifest = sample_manifest();
    manifest["future_field"] = json!(true);
    assert!(!validation_errors(&validator, &manifest).is_empty());
}

#[test]
fn rust_type_matches_schema_sample() {
    let manifest: PluginManifest = serde_json::from_value(sample_manifest())
        .expect("sample must deserialize into the Rust type");
    assert_eq!(manifest.id, "sample-analytics");
    assert_eq!(manifest.version.to_string(), "1.2.0");
    assert_eq!(manifest.api_version.to_string(), "1.0.0");
    assert_eq!(manifest.runtime_mode, RuntimeMode::OutOfProcess);
    assert_eq!(manifest.capabilities, vec![Capability::StatsRead]);
    assert_eq!(
        manifest.resources,
        Some(ResourceLimits {
            max_memory_mb: 256,
            max_cpu_percent: 50.0,
            timeout_seconds: 60,
        })
    );
    assert_eq!(
        manifest.ui_extension_points,
        vec![UiExtensionPoint::DashboardWidget]
    );
    assert_eq!(manifest.trust_level, Some(TrustLevel::Official));
    assert_eq!(
        manifest
            .health_policy
            .unwrap()
            .max_crashes_before_quarantine,
        3
    );
    assert!(manifest.has_valid_id());
}

#[test]
fn schema_is_embedded_and_current() {
    let schema: Value = serde_json::from_str(PLUGIN_MANIFEST_SCHEMA).unwrap();
    assert_eq!(
        schema["$id"],
        "https://tahan.local/schemas/plugin-manifest.schema.json"
    );
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), 8);
}

#[test]
fn plugin_id_helper_matches_the_schema_pattern() {
    assert!(is_valid_plugin_id("mt-import"));
    assert!(!is_valid_plugin_id("Bad-ID"));
}
