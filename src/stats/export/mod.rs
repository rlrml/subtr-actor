use serde::{Deserialize, Deserializer, Serialize};

mod backboard;
mod ball_carry;
mod boost;
mod ceiling_shot;
mod core;
mod demo;
mod dodge_reset;
mod double_tap;
mod fifty_fifty;
mod movement;
mod musty_flick;
mod positioning;
mod possession;
mod powerslide;
mod pressure;
mod rush;
mod speed_flip;
mod touch;

pub const LEGACY_STAT_VARIANT: &str = "legacy";
pub const LABELED_STAT_VARIANT: &str = "labeled";

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "value_type", content = "value", rename_all = "snake_case")]
pub enum StatValue {
    Float(f32),
    Unsigned(u32),
    Signed(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LabeledCountEntry {
    pub labels: Vec<StatLabel>,
    pub count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LabeledCounts {
    pub entries: Vec<LabeledCountEntry>,
}

impl LabeledCounts {
    pub fn increment<I>(&mut self, labels: I)
    where
        I: IntoIterator<Item = StatLabel>,
    {
        let mut labels: Vec<_> = labels.into_iter().collect();
        labels.sort();

        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.labels == labels) {
            entry.count += 1;
            return;
        }

        self.entries.push(LabeledCountEntry { labels, count: 1 });
        self.entries
            .sort_by(|left, right| left.labels.cmp(&right.labels));
    }

    pub fn count_matching(&self, required_labels: &[StatLabel]) -> u32 {
        self.entries
            .iter()
            .filter(|entry| {
                required_labels
                    .iter()
                    .all(|required_label| entry.labels.contains(required_label))
            })
            .map(|entry| entry.count)
            .sum()
    }

    pub fn count_exact(&self, labels: &[StatLabel]) -> u32 {
        let mut normalized_labels = labels.to_vec();
        normalized_labels.sort();

        self.entries
            .iter()
            .find(|entry| entry.labels == normalized_labels)
            .map(|entry| entry.count)
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LabeledFloatSumEntry {
    pub labels: Vec<StatLabel>,
    pub value: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LabeledFloatSums {
    pub entries: Vec<LabeledFloatSumEntry>,
}

impl LabeledFloatSums {
    pub fn add<I>(&mut self, labels: I, value: f32)
    where
        I: IntoIterator<Item = StatLabel>,
    {
        let mut labels: Vec<_> = labels.into_iter().collect();
        labels.sort();

        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.labels == labels) {
            entry.value += value;
            return;
        }

        self.entries.push(LabeledFloatSumEntry { labels, value });
        self.entries
            .sort_by(|left, right| left.labels.cmp(&right.labels));
    }

    pub fn sum_matching(&self, required_labels: &[StatLabel]) -> f32 {
        self.entries
            .iter()
            .filter(|entry| {
                required_labels
                    .iter()
                    .all(|required_label| entry.labels.contains(required_label))
            })
            .map(|entry| entry.value)
            .sum()
    }

    pub fn sum_exact(&self, labels: &[StatLabel]) -> f32 {
        let mut normalized_labels = labels.to_vec();
        normalized_labels.sort();

        self.entries
            .iter()
            .find(|entry| entry.labels == normalized_labels)
            .map(|entry| entry.value)
            .unwrap_or(0.0)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportedStat {
    #[serde(flatten)]
    pub descriptor: StatDescriptor,
    pub value: StatValue,
}

impl ExportedStat {
    pub fn float(domain: &'static str, name: &'static str, unit: StatUnit, value: f32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
                labels: Vec::new(),
            },
            value: StatValue::Float(value),
        }
    }

    pub fn unsigned(domain: &'static str, name: &'static str, unit: StatUnit, value: u32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
                labels: Vec::new(),
            },
            value: StatValue::Unsigned(value),
        }
    }

    pub fn signed(domain: &'static str, name: &'static str, unit: StatUnit, value: i32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
                labels: Vec::new(),
            },
            value: StatValue::Signed(value),
        }
    }

    pub fn unsigned_labeled(
        domain: &'static str,
        name: &'static str,
        unit: StatUnit,
        labels: Vec<StatLabel>,
        value: u32,
    ) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LABELED_STAT_VARIANT,
                unit,
                labels,
            },
            value: StatValue::Unsigned(value),
        }
    }

    pub fn float_labeled(
        domain: &'static str,
        name: &'static str,
        unit: StatUnit,
        labels: Vec<StatLabel>,
        value: f32,
    ) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LABELED_STAT_VARIANT,
                unit,
                labels,
            },
            value: StatValue::Float(value),
        }
    }
}

pub trait StatFieldProvider {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat));

    fn stat_fields(&self) -> Vec<ExportedStat> {
        let mut fields = Vec::new();
        self.visit_stat_fields(&mut |field| fields.push(field));
        fields
    }
}

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
