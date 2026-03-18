use serde::Serialize;

mod ball_carry;
mod boost;
mod core;
mod demo;
mod dodge_reset;
mod movement;
mod positioning;
mod possession;
mod powerslide;
mod pressure;
mod touch;

pub const LEGACY_STAT_VARIANT: &str = "legacy";
pub const LABELED_STAT_VARIANT: &str = "labeled";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct StatLabel {
    pub key: &'static str,
    pub value: &'static str,
}

impl StatLabel {
    pub const fn new(key: &'static str, value: &'static str) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StatDescriptor {
    pub domain: &'static str,
    pub name: &'static str,
    pub variant: &'static str,
    pub unit: StatUnit,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<StatLabel>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "value_type", content = "value", rename_all = "snake_case")]
pub enum StatValue {
    Float(f32),
    Unsigned(u32),
    Signed(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LabeledCountEntry {
    pub labels: Vec<StatLabel>,
    pub count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
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
        self.entries.sort_by(|left, right| left.labels.cmp(&right.labels));
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

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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
}

pub trait StatFieldProvider {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat));

    fn stat_fields(&self) -> Vec<ExportedStat> {
        let mut fields = Vec::new();
        self.visit_stat_fields(&mut |field| fields.push(field));
        fields
    }
}
