use super::rush_labels::{RUSH_ATTACKER_LABELS, RUSH_DEFENDER_LABELS, RUSH_TEAM_LABELS};
use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RushStats {
    pub team_zero_count: u32,
    pub team_zero_two_v_one_count: u32,
    pub team_zero_two_v_two_count: u32,
    pub team_zero_two_v_three_count: u32,
    pub team_zero_three_v_one_count: u32,
    pub team_zero_three_v_two_count: u32,
    pub team_zero_three_v_three_count: u32,
    pub team_one_count: u32,
    pub team_one_two_v_one_count: u32,
    pub team_one_two_v_two_count: u32,
    pub team_one_two_v_three_count: u32,
    pub team_one_three_v_one_count: u32,
    pub team_one_three_v_two_count: u32,
    pub team_one_three_v_three_count: u32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_rush_counts: LabeledCounts,
}

impl RushStats {
    pub(super) fn record(&mut self, event: &RushEvent) {
        self.labeled_rush_counts.increment(event.labels());
        self.sync_legacy_counts();
    }

    pub fn rush_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_rush_counts.count_matching(labels)
    }

    pub fn complete_labeled_rush_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &RUSH_TEAM_LABELS,
                &RUSH_ATTACKER_LABELS,
                &RUSH_DEFENDER_LABELS,
            ],
            &self.labeled_rush_counts,
        )
    }

    pub fn with_complete_labeled_rush_counts(mut self) -> Self {
        self.labeled_rush_counts = self.complete_labeled_rush_counts();
        self
    }
}
