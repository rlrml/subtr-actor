use serde::{Deserialize, Serialize};

use super::StatLabel;

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
                    count: counts.count_exact(&normalized_labels),
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
