use super::possession_stats_labels::team_relative_labeled_time;
use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PossessionStats {
    pub tracked_time: f32,
    pub team_zero_time: f32,
    pub team_one_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

impl PossessionStats {
    pub fn team_zero_pct(&self) -> f32 {
        percentage(self.team_zero_time, self.tracked_time)
    }

    pub fn team_one_pct(&self) -> f32 {
        percentage(self.team_one_time, self.tracked_time)
    }

    pub fn neutral_pct(&self) -> f32 {
        percentage(self.neutral_time, self.tracked_time)
    }

    pub fn time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_time.sum_matching(labels)
    }

    pub fn for_team(&self, is_team_zero: bool) -> PossessionTeamStats {
        let (possession_time, opponent_possession_time) = if is_team_zero {
            (self.team_zero_time, self.team_one_time)
        } else {
            (self.team_one_time, self.team_zero_time)
        };
        PossessionTeamStats {
            tracked_time: self.tracked_time,
            possession_time,
            opponent_possession_time,
            neutral_time: self.neutral_time,
            labeled_time: team_relative_labeled_time(&self.labeled_time, is_team_zero),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PossessionTeamStats {
    pub tracked_time: f32,
    pub possession_time: f32,
    pub opponent_possession_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PossessionEvent {
    pub time: f32,
    pub frame: usize,
    pub active: bool,
    pub possession_state: String,
    pub field_third: Option<String>,
}

fn percentage(value: f32, total: f32) -> f32 {
    if total == 0.0 {
        0.0
    } else {
        value * 100.0 / total
    }
}
