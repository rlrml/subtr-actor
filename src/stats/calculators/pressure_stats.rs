use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PressureStats {
    pub tracked_time: f32,
    pub team_zero_side_time: f32,
    pub team_one_side_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

impl PressureStats {
    pub fn team_zero_side_pct(&self) -> f32 {
        pct(self.team_zero_side_time, self.tracked_time)
    }

    pub fn team_one_side_pct(&self) -> f32 {
        pct(self.team_one_side_time, self.tracked_time)
    }

    pub fn neutral_pct(&self) -> f32 {
        pct(self.neutral_time, self.tracked_time)
    }

    pub fn time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_time.sum_matching(labels)
    }

    pub fn for_team(&self, is_team_zero: bool) -> PressureTeamStats {
        PressureTeamStats::from_pressure_stats(self, is_team_zero)
    }
}

fn pct(value: f32, tracked_time: f32) -> f32 {
    if tracked_time == 0.0 {
        0.0
    } else {
        value * 100.0 / tracked_time
    }
}
