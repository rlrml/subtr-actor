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

    /// Fold one live-play frame of possession into the cumulative stats.
    ///
    /// `possession_state` is the current possession label (`team_zero` /
    /// `team_one` / `neutral`); `field_third` / `field_half` are the ball's
    /// current zone labels sourced from the canonical `ball_third` / `ball_half`
    /// streams. The cross-tab is the per-frame join of possession with those
    /// zones — possession events themselves carry no zone, so this is the only
    /// place the dimensions meet.
    pub fn apply_frame(
        &mut self,
        possession_state: &str,
        field_third: Option<&str>,
        field_half: Option<&str>,
        dt: f32,
    ) {
        self.stats.tracked_time += dt;
        let possession_value = match possession_state {
            "team_zero" => {
                self.stats.team_zero_time += dt;
                "team_zero"
            }
            "team_one" => {
                self.stats.team_one_time += dt;
                "team_one"
            }
            "neutral" => {
                self.stats.neutral_time += dt;
                "neutral"
            }
            _ => return,
        };

        let mut labels = vec![StatLabel::new("possession_state", possession_value)];
        if let Some(field_third) = field_third.and_then(static_field_third_label_value) {
            labels.push(StatLabel::new("field_third", field_third));
        }
        if let Some(field_half) = field_half.and_then(static_field_half_label_value) {
            labels.push(StatLabel::new("field_half", field_half));
        }
        self.stats.labeled_time.add(labels, dt);
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

fn static_field_half_label_value(value: &str) -> Option<&'static str> {
    match value {
        "team_zero_side" => Some("team_zero_side"),
        "team_one_side" => Some("team_one_side"),
        "neutral" => Some("neutral"),
        _ => None,
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

    #[test]
    fn possession_labeled_time_includes_field_half() {
        let mut accumulator = PossessionStatsAccumulator::default();
        accumulator.apply_frame(
            "team_zero",
            Some("team_zero_third"),
            Some("team_zero_side"),
            2.0,
        );

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
    fn possession_half_is_independent_of_third() {
        // The neutral third straddles the midfield line: a ball in the neutral
        // third can be in either half. The half comes from the ball_half stream,
        // not from the third bucket, so it is not collapsed to a neutral half.
        let mut accumulator = PossessionStatsAccumulator::default();
        accumulator.apply_frame(
            "team_zero",
            Some("neutral_third"),
            Some("team_one_side"),
            1.0,
        );

        assert_eq!(
            accumulator.stats().labeled_time.entries[0].labels,
            vec![
                StatLabel::new("field_half", "team_one_side"),
                StatLabel::new("field_third", "neutral_third"),
                StatLabel::new("possession_state", "team_zero"),
            ]
        );
    }

    #[test]
    fn team_relative_possession_stats_translate_field_half() {
        let mut accumulator = PossessionStatsAccumulator::default();
        accumulator.apply_frame(
            "team_zero",
            Some("team_one_third"),
            Some("team_one_side"),
            3.0,
        );

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
