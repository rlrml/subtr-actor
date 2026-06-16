use serde::{Deserialize, Deserializer, Serialize};

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

    pub fn total(&self) -> u32 {
        self.entries.iter().map(|entry| entry.count).sum()
    }

    pub fn complete_from_label_sets(label_sets: &[&[StatLabel]], counts: &Self) -> Self {
        fn append_entries(
            label_sets: &[&[StatLabel]],
            index: usize,
            labels: &mut Vec<StatLabel>,
            counts: &LabeledCounts,
            entries: &mut Vec<LabeledCountEntry>,
        ) {
            if index == label_sets.len() {
                let mut normalized_labels = labels.clone();
                normalized_labels.sort();
                entries.push(LabeledCountEntry {
                    count: counts.count_matching(&normalized_labels),
                    labels: normalized_labels,
                });
                return;
            }

            for label in label_sets[index] {
                labels.push(label.clone());
                append_entries(label_sets, index + 1, labels, counts, entries);
                labels.pop();
            }
        }

        let mut entries = Vec::new();
        append_entries(label_sets, 0, &mut Vec::new(), counts, &mut entries);
        entries.sort_by(|left, right| left.labels.cmp(&right.labels));
        Self { entries }
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

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
