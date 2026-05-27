use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatUnit {
    Seconds,
    Percent,
    UnrealUnits,
    UnrealUnitsPerSecond,
    Boost,
    BoostPerMinute,
    Count,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatLabel {
    pub key: &'static str,
    pub value: &'static str,
}

impl StatLabel {
    pub const fn new(key: &'static str, value: &'static str) -> Self {
        Self { key, value }
    }
}

impl<'de> Deserialize<'de> for StatLabel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct OwnedStatLabel {
            key: String,
            value: String,
        }

        let owned = OwnedStatLabel::deserialize(deserializer)?;
        Ok(Self {
            key: leak_string(owned.key),
            value: leak_string(owned.value),
        })
    }
}

pub(super) fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
