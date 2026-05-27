use serde::{Deserialize, Deserializer, Serialize};

use super::{leak_string, StatLabel, StatUnit};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StatDescriptor {
    pub domain: &'static str,
    pub name: &'static str,
    pub variant: &'static str,
    pub unit: StatUnit,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<StatLabel>,
}

impl<'de> Deserialize<'de> for StatDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct OwnedStatDescriptor {
            domain: String,
            name: String,
            variant: String,
            unit: StatUnit,
            #[serde(default)]
            labels: Vec<StatLabel>,
        }

        let owned = OwnedStatDescriptor::deserialize(deserializer)?;
        Ok(Self {
            domain: leak_string(owned.domain),
            name: leak_string(owned.name),
            variant: leak_string(owned.variant),
            unit: owned.unit,
            labels: owned.labels,
        })
    }
}
