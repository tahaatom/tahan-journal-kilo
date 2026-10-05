//! تست‌های یکپارچه پاکت‌های دستور و رویداد.

use aria_contracts::command::{CommandEnvelope, CommandIssuer, CommandType, IssuerKind};
use aria_contracts::event::{EngineId, EventEnvelope, EventSource, EventType};
use aria_contracts::versioning::EVENT_ENVELOPE_VERSION;
use chrono::DateTime;
use serde_json::json;
use uuid::Uuid;

fn sample_command() -> CommandEnvelope {
    CommandEnvelope::new(
        CommandType::CreateTrade,
        json!({ "symbol": "XAUUSD", "direction": "buy" }),
        CommandIssuer::user(Uuid::new_v4()),
    )
    .with_correlation_id(Uuid::new_v4())
}

fn sample_event() -> EventEnvelope {
    EventEnvelope::new(
        EventType::TradeCreated,
        EVENT_ENVELOPE_VERSION,
        EventSource::of(EngineId::Domain).with_component("trade_service"),
        json!({ "trade_id": Uuid::new_v4() }),
    )
    .with_correlation_id(Uuid::new_v4())
}

#[test]
fn command_envelope_roundtrips_through_json() {
    let envelope = sample_command();
    let json = serde_json::to_string(&envelope).expect("serialization must succeed");
    let parsed: CommandEnvelope =
        serde_json::from_str(&json).expect("deserialization must succeed");
    assert_eq!(envelope, parsed);
}

#[test]
fn command_envelope_uses_the_contract_field_names() {
    let value = serde_json::to_value(sample_command()).unwrap();
    for field in [
        "command_id",
        "command_type",
        "payload",
        "issuer",
        "timestamp",
        "correlation_id",
    ] {
        assert!(
            value.get(field).is_some(),
            "field {} must be present",
            field
        );
    }
    assert_eq!(value["command_type"], "create_trade");
    assert_eq!(value["issuer"]["kind"], "user");
}

#[test]
fn command_envelope_omits_absent_correlation_id() {
    let envelope =
        CommandEnvelope::new(CommandType::DeleteTrade, json!({}), CommandIssuer::system());
    let value = serde_json::to_value(&envelope).unwrap();
    assert!(value.get("correlation_id").is_none());
    let parsed: CommandEnvelope = serde_json::from_value(value).unwrap();
    assert!(parsed.correlation_id.is_none());
}

#[test]
fn command_timestamp_is_utc_iso8601() {
    let value = serde_json::to_value(sample_command()).unwrap();
    let timestamp = value["timestamp"]
        .as_str()
        .expect("timestamp must be a string");
    assert!(
        timestamp.ends_with('Z'),
        "timestamp must be UTC: {}",
        timestamp
    );
    DateTime::parse_from_rfc3339(timestamp).expect("timestamp must be ISO 8601");
}

#[test]
fn unknown_command_type_is_rejected() {
    let mut value = serde_json::to_value(sample_command()).unwrap();
    value["command_type"] = json!("nuclear_launch");
    assert!(serde_json::from_value::<CommandEnvelope>(value).is_err());
}

#[test]
fn all_command_types_roundtrip() {
    let command_types = [
        CommandType::CreateTrade,
        CommandType::UpdateTrade,
        CommandType::DeleteTrade,
        CommandType::AddEntryLeg,
        CommandType::UpdateEntryLeg,
        CommandType::AddExitLeg,
        CommandType::UpdateExitLeg,
        CommandType::AssignExecutionToLeg,
        CommandType::AddManualOverride,
        CommandType::RevertManualOverride,
        CommandType::LinkAttachmentToTrade,
    ];
    for command_type in command_types {
        let envelope =
            CommandEnvelope::new(command_type, json!({}), CommandIssuer::plugin("mt-import"));
        let json = serde_json::to_string(&envelope).unwrap();
        let parsed: CommandEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.command_type, command_type);
        assert_eq!(parsed.issuer.kind, IssuerKind::Plugin);
        assert_eq!(parsed.issuer.principal_id, "mt-import");
    }
}

#[test]
fn event_envelope_roundtrips_through_json() {
    let envelope = sample_event();
    let json = serde_json::to_string(&envelope).expect("serialization must succeed");
    let parsed: EventEnvelope = serde_json::from_str(&json).expect("deserialization must succeed");
    assert_eq!(envelope, parsed);
}

#[test]
fn event_envelope_uses_the_contract_field_names() {
    let value = serde_json::to_value(sample_event()).unwrap();
    for field in [
        "event_id",
        "event_type",
        "event_version",
        "source",
        "timestamp",
        "correlation_id",
        "payload",
    ] {
        assert!(
            value.get(field).is_some(),
            "field {} must be present",
            field
        );
    }
    assert_eq!(value["event_type"], "trade_created");
    assert_eq!(value["event_version"], EVENT_ENVELOPE_VERSION);
    assert_eq!(value["source"]["engine"], "domain");
    assert_eq!(value["source"]["component"], "trade_service");
}

#[test]
fn event_envelope_omits_absent_correlation_id() {
    let envelope = EventEnvelope::new(
        EventType::StatsInvalidated,
        EVENT_ENVELOPE_VERSION,
        EventSource::of(EngineId::Query),
        json!({}),
    );
    let value = serde_json::to_value(&envelope).unwrap();
    assert!(value.get("correlation_id").is_none());
    let parsed: EventEnvelope = serde_json::from_value(value).unwrap();
    assert!(parsed.correlation_id.is_none());
}

#[test]
fn unknown_event_type_is_rejected() {
    let mut value = serde_json::to_value(sample_event()).unwrap();
    value["event_type"] = json!("market_crashed");
    assert!(serde_json::from_value::<EventEnvelope>(value).is_err());
}

#[test]
fn all_event_types_roundtrip() {
    let event_types = [
        EventType::TradeCreated,
        EventType::TradeUpdated,
        EventType::TradeDeleted,
        EventType::EntryLegAdded,
        EventType::ExitLegAdded,
        EventType::ExecutionAssigned,
        EventType::OverrideAdded,
        EventType::OverrideReverted,
        EventType::StatsInvalidated,
    ];
    for event_type in event_types {
        let envelope = EventEnvelope::new(
            event_type,
            EVENT_ENVELOPE_VERSION,
            EventSource::of(EngineId::Domain),
            json!({}),
        );
        let json = serde_json::to_string(&envelope).unwrap();
        let parsed: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, event_type);
        assert_eq!(parsed.source.engine, EngineId::Domain);
    }
}
