use super::fifty_fifty_labels::{
    FIFTY_FIFTY_PHASE_LABELS, FIFTY_FIFTY_POSSESSION_LABELS,
    FIFTY_FIFTY_TEAM_ONE_DODGE_STATE_LABELS, FIFTY_FIFTY_TEAM_OUTCOME_LABELS,
    FIFTY_FIFTY_TEAM_ZERO_DODGE_STATE_LABELS,
};
use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyStats {
    pub count: u32,
    pub team_zero_wins: u32,
    pub team_one_wins: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_team_zero_wins: u32,
    pub kickoff_team_one_wins: u32,
    pub kickoff_neutral_outcomes: u32,
    pub team_zero_possession_after_count: u32,
    pub team_one_possession_after_count: u32,
    pub neutral_possession_after_count: u32,
    pub kickoff_team_zero_possession_after_count: u32,
    pub kickoff_team_one_possession_after_count: u32,
    pub kickoff_neutral_possession_after_count: u32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl FiftyFiftyStats {
    pub(super) fn record_event(&mut self, event: &FiftyFiftyEvent) {
        self.labeled_event_counts.increment(event.labels());
        self.sync_legacy_counts();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &FIFTY_FIFTY_PHASE_LABELS,
                &FIFTY_FIFTY_TEAM_OUTCOME_LABELS,
                &FIFTY_FIFTY_POSSESSION_LABELS,
                &FIFTY_FIFTY_TEAM_ZERO_DODGE_STATE_LABELS,
                &FIFTY_FIFTY_TEAM_ONE_DODGE_STATE_LABELS,
            ],
            &self.labeled_event_counts,
        )
    }

    pub fn team_zero_win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.team_zero_wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn team_one_win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.team_one_wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn kickoff_team_zero_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_team_zero_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }

    pub fn kickoff_team_one_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_team_one_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }
}
