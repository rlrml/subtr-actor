use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BallThirdStats {
    pub tracked_time: f32,
    pub team_zero_third_time: f32,
    pub neutral_third_time: f32,
    pub team_one_third_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

impl BallThirdStats {
    pub fn team_zero_third_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_zero_third_time * 100.0 / self.tracked_time
        }
    }

    pub fn team_one_third_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_one_third_time * 100.0 / self.tracked_time
        }
    }

    pub fn neutral_third_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.neutral_third_time * 100.0 / self.tracked_time
        }
    }

    pub fn time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_time.sum_matching(labels)
    }

    pub fn for_team(&self, is_team_zero: bool) -> BallThirdTeamStats {
        let (defensive_third_time, offensive_third_time) = if is_team_zero {
            (self.team_zero_third_time, self.team_one_third_time)
        } else {
            (self.team_one_third_time, self.team_zero_third_time)
        };

        let mut labeled_time = LabeledFloatSums::default();
        for entry in &self.labeled_time.entries {
            labeled_time.add(
                entry
                    .labels
                    .iter()
                    .map(|label| team_relative_ball_third_label(label, is_team_zero)),
                entry.value,
            );
        }

        BallThirdTeamStats {
            tracked_time: self.tracked_time,
            defensive_third_time,
            neutral_third_time: self.neutral_third_time,
            offensive_third_time,
            labeled_time,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BallThirdTeamStats {
    pub tracked_time: f32,
    pub defensive_third_time: f32,
    pub neutral_third_time: f32,
    pub offensive_third_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BallThirdStatsAccumulator {
    stats: BallThirdStats,
}

impl BallThirdStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &BallThirdStats {
        &self.stats
    }

    pub fn apply_event(&mut self, event: &BallThirdEvent) {
        if !event.active {
            return;
        }

        self.stats.tracked_time += event.duration;
        let field_third = match event.field_third.as_str() {
            "team_zero_third" => {
                self.stats.team_zero_third_time += event.duration;
                "team_zero_third"
            }
            "team_one_third" => {
                self.stats.team_one_third_time += event.duration;
                "team_one_third"
            }
            "neutral_third" => {
                self.stats.neutral_third_time += event.duration;
                "neutral_third"
            }
            _ => return,
        };
        self.stats
            .labeled_time
            .add([StatLabel::new("field_third", field_third)], event.duration);
    }
}

fn team_relative_ball_third_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
    match (label.key, label.value) {
        ("field_third", "team_zero_third") => StatLabel::new(
            "field_third",
            if is_team_zero {
                "defensive_third"
            } else {
                "offensive_third"
            },
        ),
        ("field_third", "team_one_third") => StatLabel::new(
            "field_third",
            if is_team_zero {
                "offensive_third"
            } else {
                "defensive_third"
            },
        ),
        _ => label.clone(),
    }
}
