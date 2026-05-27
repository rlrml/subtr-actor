use serde::{Deserialize, Serialize};

use super::StatLabel;

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
