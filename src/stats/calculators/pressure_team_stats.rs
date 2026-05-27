use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PressureTeamStats {
    pub tracked_time: f32,
    pub defensive_half_time: f32,
    pub offensive_half_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

impl PressureTeamStats {
    pub(super) fn from_pressure_stats(stats: &PressureStats, is_team_zero: bool) -> Self {
        let (defensive_half_time, offensive_half_time) = if is_team_zero {
            (stats.team_zero_side_time, stats.team_one_side_time)
        } else {
            (stats.team_one_side_time, stats.team_zero_side_time)
        };

        Self {
            tracked_time: stats.tracked_time,
            defensive_half_time,
            offensive_half_time,
            neutral_time: stats.neutral_time,
            labeled_time: team_relative_labeled_time(stats, is_team_zero),
        }
    }
}

fn team_relative_labeled_time(stats: &PressureStats, is_team_zero: bool) -> LabeledFloatSums {
    let mut labeled_time = LabeledFloatSums::default();
    for entry in &stats.labeled_time.entries {
        labeled_time.add(
            entry
                .labels
                .iter()
                .map(|label| team_relative_pressure_label(label, is_team_zero)),
            entry.value,
        );
    }
    labeled_time
}
