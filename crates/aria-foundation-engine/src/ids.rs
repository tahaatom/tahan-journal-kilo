//! شناسه‌های تایپ‌شده کرنل آریا.
//!
//! همه شناسه‌ها از UUID نسخه ۴ ساخته می‌شوند و به‌صورت newtype تایپ‌شده ارائه
//! می‌شوند تا اختلاط شناسه‌های موجودیت‌های مختلف (مثلاً TradeId با AccountId)
//! در زمان کامپایل جلوگیری شود.
//!
//! سریالیزه‌سازی به‌صورت شفاف (transparent) روی رشته UUID انجام می‌شود.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_typed_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// یک شناسه جدید تصادفی (UUID v4) می‌سازد.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// از یک `Uuid` موجود می‌سازد.
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// `Uuid` زیرین را برمی‌گرداند.
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }

            /// شناسه صفر (همه بیت‌ها صفر) را برمی‌گرداند.
            pub const fn nil() -> Self {
                Self(Uuid::nil())
            }

            /// آیا این شناسه صفر است؟
            pub const fn is_nil(&self) -> bool {
                self.0.is_nil()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::nil()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Uuid {
                id.0
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map(Self)
            }
        }
    };
}

define_typed_id!(ProfileId);
define_typed_id!(AccountId);
define_typed_id!(SymbolId);
define_typed_id!(TradeId);
define_typed_id!(EntryLegId);
define_typed_id!(ExitLegId);
define_typed_id!(ExecutionId);
define_typed_id!(SourceRecordId);
define_typed_id!(FieldId);
define_typed_id!(AttachmentId);
define_typed_id!(PluginId);
define_typed_id!(EventId);
define_typed_id!(CommandId);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    macro_rules! uniqueness_test {
        ($($name:ident),* $(,)?) => {
            #[test]
            fn all_typed_ids_generate_unique_values() {
                let mut seen = HashSet::new();
                $(
                    for _ in 0..1_000 {
                        let id = $name::new();
                        assert!(!id.is_nil());
                        assert!(seen.insert(id.as_uuid()), "duplicate UUID generated for {}", stringify!($name));
                    }
                )*
                assert_eq!(seen.len(), 13_000);
            }
        };
    }

    uniqueness_test!(
        ProfileId,
        AccountId,
        SymbolId,
        TradeId,
        EntryLegId,
        ExitLegId,
        ExecutionId,
        SourceRecordId,
        FieldId,
        AttachmentId,
        PluginId,
        EventId,
        CommandId,
    );

    #[test]
    fn single_type_ids_are_unique() {
        let mut seen = HashSet::new();
        for _ in 0..10_000 {
            let id = TradeId::new();
            assert!(seen.insert(id), "duplicate TradeId generated");
        }
        assert_eq!(seen.len(), 10_000);
    }

    #[test]
    fn id_string_roundtrip() {
        let id = ExecutionId::new();
        let text = id.to_string();
        let parsed: ExecutionId = text.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn id_serde_roundtrip_is_transparent() {
        let id = SourceRecordId::new();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{}\"", id));
        let parsed: SourceRecordId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn nil_id_behaviour() {
        let id = PluginId::nil();
        assert!(id.is_nil());
        assert_eq!(id, PluginId::default());
        let parsed: PluginId = id.to_string().parse().unwrap();
        assert!(parsed.is_nil());
        assert_eq!(id.as_uuid(), Uuid::nil());
    }

    #[test]
    fn uuid_conversions() {
        let uuid = Uuid::new_v4();
        let id: FieldId = uuid.into();
        assert_eq!(id.as_uuid(), uuid);
        let back: Uuid = id.into();
        assert_eq!(back, uuid);
        assert_eq!(FieldId::from_uuid(uuid), id);
    }

    #[test]
    fn invalid_uuid_string_is_rejected() {
        let result: Result<CommandId, _> = "not-a-uuid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn distinct_types_with_same_uuid_are_distinct_at_compile_time() {
        let uuid = Uuid::new_v4();
        let trade: TradeId = uuid.into();
        let account: AccountId = uuid.into();
        assert_eq!(trade.as_uuid(), account.as_uuid());
    }
}
