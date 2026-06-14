use super::*;

/// Accumulated possession time split by team and neutral.
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
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_zero_time * 100.0 / self.tracked_time
        }
    }

    pub fn team_one_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_one_time * 100.0 / self.tracked_time
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

    pub fn for_team(&self, is_team_zero: bool) -> PossessionTeamStats {
        let (possession_time, opponent_possession_time) = if is_team_zero {
            (self.team_zero_time, self.team_one_time)
        } else {
            (self.team_one_time, self.team_zero_time)
        };

        let mut labeled_time = LabeledFloatSums::default();
        for entry in &self.labeled_time.entries {
            labeled_time.add(
                entry
                    .labels
                    .iter()
                    .map(|label| team_relative_possession_label(label, is_team_zero)),
                entry.value,
            );
        }

        PossessionTeamStats {
            tracked_time: self.tracked_time,
            possession_time,
            opponent_possession_time,
            neutral_time: self.neutral_time,
            labeled_time,
        }
    }
}

/// Per-team accumulated possession stats.
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

/// Accumulates possession stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PossessionStatsAccumulator {
    stats: PossessionStats,
}

impl PossessionStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &PossessionStats {
        &self.stats
    }

    pub fn apply_event(&mut self, event: &PossessionEvent) {
        if !event.active {
            return;
        }

        self.stats.tracked_time += event.duration;
        let possession_value = match event.possession_state.as_str() {
            "team_zero" => {
                self.stats.team_zero_time += event.duration;
                "team_zero"
            }
            "team_one" => {
                self.stats.team_one_time += event.duration;
                "team_one"
            }
            "neutral" => {
                self.stats.neutral_time += event.duration;
                "neutral"
            }
            _ => return,
        };

        let possession_label = StatLabel::new("possession_state", possession_value);
        if let Some(field_third) = event
            .field_third
            .as_deref()
            .and_then(static_field_third_label_value)
        {
            let field_half = field_half_for_field_third(field_third);
            self.stats.labeled_time.add(
                [
                    possession_label,
                    StatLabel::new("field_third", field_third),
                    StatLabel::new("field_half", field_half),
                ],
                event.duration,
            );
        } else {
            self.stats
                .labeled_time
                .add([possession_label], event.duration);
        }
    }
}

fn static_field_third_label_value(value: &str) -> Option<&'static str> {
    match value {
        "team_zero_third" => Some("team_zero_third"),
        "neutral_third" => Some("neutral_third"),
        "team_one_third" => Some("team_one_third"),
        _ => None,
    }
}

fn field_half_for_field_third(value: &str) -> &'static str {
    match value {
        "team_zero_third" => "team_zero_side",
        "team_one_third" => "team_one_side",
        _ => "neutral",
    }
}

fn team_relative_possession_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
    match (label.key, label.value) {
        ("possession_state", "team_zero") => StatLabel::new(
            "possession_state",
            if is_team_zero { "own" } else { "opponent" },
        ),
        ("possession_state", "team_one") => StatLabel::new(
            "possession_state",
            if is_team_zero { "opponent" } else { "own" },
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn event(possession_state: &str, field_third: &str, duration: f32) -> PossessionEvent {
        PossessionEvent {
            time: 0.0,
            frame: 0,
            end_time: duration,
            end_frame: 1,
            active: true,
            duration,
            possession_state: possession_state.to_owned(),
            player_id: None,
            field_third: Some(field_third.to_owned()),
        }
    }

    #[test]
    fn possession_labeled_time_includes_field_half() {
        let mut accumulator = PossessionStatsAccumulator::default();
        accumulator.apply_event(&event("team_zero", "team_zero_third", 2.0));

        assert_eq!(
            accumulator.stats().labeled_time.entries[0].labels,
            vec![
                StatLabel::new("field_half", "team_zero_side"),
                StatLabel::new("field_third", "team_zero_third"),
                StatLabel::new("possession_state", "team_zero"),
            ]
        );
        assert_eq!(accumulator.stats().labeled_time.entries[0].value, 2.0);
    }

    #[test]
    fn team_relative_possession_stats_translate_field_half() {
        let mut accumulator = PossessionStatsAccumulator::default();
        accumulator.apply_event(&event("team_zero", "team_one_third", 3.0));

        let team_zero = accumulator.stats().for_team(true);
        assert_eq!(
            team_zero.labeled_time.entries[0].labels,
            vec![
                StatLabel::new("field_half", "offensive_half"),
                StatLabel::new("field_third", "offensive_third"),
                StatLabel::new("possession_state", "own"),
            ]
        );
    }
}
