//! قرارداد پاکت رویداد (Event Envelope).
//!
//! رویدادها پس از کامیت منتشر می‌شوند، مصرف‌کننده‌ها باید ایدمپوتنت باشند،
//! رویدادهای حیاتی از طریق outbox ارسال می‌شوند و رویدادهای فقط-رابط‌کاربری
//! در صورت نیاز ذخیره نمی‌شوند. سطح رویدادهای نسخه ۱ بسته است.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub use crate::error::EngineId;

/// پاکت رویداد — ساختار پایدار رویدادهای کرنل.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// شناسه یکتای رویداد (UUID v4).
    pub event_id: Uuid,
    /// نوع رویداد.
    pub event_type: EventType,
    /// نسخه شکل بار رویداد (نسخه اصلی قرارداد رویداد).
    pub event_version: u32,
    /// منبع تولید رویداد.
    pub source: EventSource,
    /// زمان وقوع (UTC، ISO 8601).
    pub timestamp: DateTime<Utc>,
    /// شناسه همبستگی با دستور یا رویداد مبدأ (اختیاری).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
    /// بار رویداد به‌صورت JSON.
    pub payload: Value,
}

impl EventEnvelope {
    /// یک پاکت رویداد جدید با شناسه و زمان تولیدشده می‌سازد.
    pub fn new(
        event_type: EventType,
        event_version: u32,
        source: EventSource,
        payload: Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type,
            event_version,
            source,
            timestamp: Utc::now(),
            correlation_id: None,
            payload,
        }
    }

    /// شناسه همبستگی را تعیین می‌کند.
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

/// انواع رویدادهای نسخه ۱.
///
/// این سطح بسته با رویدادهای قرارداد دامنه (فاز ۱.۶) یکی است. نوع
/// ناشناخته در مرز ورودی رد می‌شود.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    #[serde(rename = "trade_created")]
    TradeCreated,
    #[serde(rename = "trade_updated")]
    TradeUpdated,
    #[serde(rename = "trade_deleted")]
    TradeDeleted,
    #[serde(rename = "entry_leg_added")]
    EntryLegAdded,
    #[serde(rename = "exit_leg_added")]
    ExitLegAdded,
    #[serde(rename = "execution_assigned")]
    ExecutionAssigned,
    #[serde(rename = "override_added")]
    OverrideAdded,
    #[serde(rename = "override_reverted")]
    OverrideReverted,
    #[serde(rename = "stats_invalidated")]
    StatsInvalidated,
}

impl EventType {
    /// نام پایدار نوع رویداد.
    pub fn as_str(self) -> &'static str {
        match self {
            EventType::TradeCreated => "trade_created",
            EventType::TradeUpdated => "trade_updated",
            EventType::TradeDeleted => "trade_deleted",
            EventType::EntryLegAdded => "entry_leg_added",
            EventType::ExitLegAdded => "exit_leg_added",
            EventType::ExecutionAssigned => "execution_assigned",
            EventType::OverrideAdded => "override_added",
            EventType::OverrideReverted => "override_reverted",
            EventType::StatsInvalidated => "stats_invalidated",
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// منبع تولید رویداد.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventSource {
    /// موتور تولیدکننده رویداد.
    pub engine: EngineId,
    /// جزء اختیاری داخلی موتور.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
}

impl EventSource {
    /// منبع رویداد برای یک موتور.
    pub fn of(engine: EngineId) -> Self {
        Self {
            engine,
            component: None,
        }
    }

    /// منبع رویداد با جزء داخلی موتور.
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_names_are_stable() {
        assert_eq!(EventType::TradeCreated.as_str(), "trade_created");
        assert_eq!(EventType::StatsInvalidated.as_str(), "stats_invalidated");
        assert_eq!(EventType::OverrideReverted.to_string(), "override_reverted");
    }

    #[test]
    fn event_source_builders_work() {
        let source = EventSource::of(EngineId::Domain).with_component("trade_service");
        assert_eq!(source.engine, EngineId::Domain);
        assert_eq!(source.component.as_deref(), Some("trade_service"));
        assert!(EventSource::of(EngineId::Storage).component.is_none());
    }

    #[test]
    fn new_envelope_generates_unique_ids() {
        let first = EventEnvelope::new(
            EventType::TradeCreated,
            1,
            EventSource::of(EngineId::Domain),
            Value::Null,
        );
        let second = EventEnvelope::new(
            EventType::TradeCreated,
            1,
            EventSource::of(EngineId::Domain),
            Value::Null,
        );
        assert_ne!(first.event_id, second.event_id);
        assert!(first.correlation_id.is_none());
    }
}
