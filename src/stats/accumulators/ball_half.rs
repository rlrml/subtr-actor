use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BallHalfStats {
    pub tracked_time: f32,
    pub team_zero_side_time: f32,
    pub team_one_side_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

impl BallHalfStats {
    pub fn team_zero_side_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_zero_side_time * 100.0 / self.tracked_time
        }
    }

    pub fn team_one_side_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_one_side_time * 100.0 / self.tracked_time
        }
    }

    pub fn neutral_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.neutral_time * 100.0 / self.tracked_time
        }
    }

    pub fn time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_time.sum_matching(labels)
    }

    pub fn for_team(&self, is_team_zero: bool) -> BallHalfTeamStats {
        let (defensive_half_time, offensive_half_time) = if is_team_zero {
            (self.team_zero_side_time, self.team_one_side_time)
        } else {
            (self.team_one_side_time, self.team_zero_side_time)
        };

        let mut labeled_time = LabeledFloatSums::default();
        for entry in &self.labeled_time.entries {
            labeled_time.add(
                entry
                    .labels
                    .iter()
                    .map(|label| team_relative_ball_half_label(label, is_team_zero)),
                entry.value,
            );
        }

        BallHalfTeamStats {
            tracked_time: self.tracked_time,
            defensive_half_time,
            offensive_half_time,
            neutral_time: self.neutral_time,
            labeled_time,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BallHalfTeamStats {
    pub tracked_time: f32,
    pub defensive_half_time: f32,
    pub offensive_half_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BallHalfStatsAccumulator {
    stats: BallHalfStats,
}

impl BallHalfStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &BallHalfStats {
        &self.stats
    }

    pub fn apply_event(&mut self, event: &BallHalfEvent) {
        if !event.active {
            return;
        }

        self.stats.tracked_time += event.duration;
        let field_half = match event.field_half.as_str() {
            "team_zero_side" => {
                self.stats.team_zero_side_time += event.duration;
                "team_zero_side"
            }
            "team_one_side" => {
                self.stats.team_one_side_time += event.duration;
                "team_one_side"
            }
            "neutral" => {
                self.stats.neutral_time += event.duration;
                "neutral"
            }
            _ => return,
        };
        self.stats
            .labeled_time
            .add([StatLabel::new("field_half", field_half)], event.duration);
    }
}

fn team_relative_ball_half_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
    match (label.key, label.value) {
        ("field_half", "team_zero_side") => StatLabel::new(
            "field_half",
            if is_team_zero {
                "defensive_half"
            } else {
                "offensive_half"
            },
        ),
        ("field_half", "team_one_side") => StatLabel::new(
            "field_half",
            if is_team_zero {
                "offensive_half"
            } else {
                "defensive_half"
            },
        ),
        _ => label.clone(),
    }
}
