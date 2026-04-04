use super::*;

const DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y: f32 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PressureHalfLabel {
    TeamZeroSide,
    TeamOneSide,
    Neutral,
}

impl PressureHalfLabel {
    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::TeamZeroSide => "team_zero_side",
            Self::TeamOneSide => "team_one_side",
            Self::Neutral => "neutral",
        };
        StatLabel::new("field_half", value)
    }
}

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

    pub fn for_team(&self, is_team_zero: bool) -> PressureTeamStats {
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
                    .map(|label| team_relative_pressure_label(label, is_team_zero)),
                entry.value,
            );
        }

        PressureTeamStats {
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
pub struct PressureTeamStats {
    pub tracked_time: f32,
    pub defensive_half_time: f32,
    pub offensive_half_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

fn team_relative_pressure_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
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

#[derive(Debug, Clone, PartialEq)]
pub struct PressureCalculatorConfig {
    pub neutral_zone_half_width_y: f32,
}

impl Default for PressureCalculatorConfig {
    fn default() -> Self {
        Self {
            neutral_zone_half_width_y: DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PressureCalculator {
    config: PressureCalculatorConfig,
    stats: PressureStats,
}

impl PressureCalculator {
    pub fn new() -> Self {
        Self::with_config(PressureCalculatorConfig::default())
    }

    pub fn with_config(config: PressureCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn stats(&self) -> &PressureStats {
        &self.stats
    }

    pub fn config(&self) -> &PressureCalculatorConfig {
        &self.config
    }

    pub fn team_zero_side_duration(&self) -> f32 {
        self.stats.team_zero_side_time
    }

    pub fn team_one_side_duration(&self) -> f32 {
        self.stats.team_one_side_time
    }

    pub fn neutral_duration(&self) -> f32 {
        self.stats.neutral_time
    }

    pub fn total_tracked_duration(&self) -> f32 {
        self.stats.tracked_time
    }

    pub fn team_zero_side_pct(&self) -> f32 {
        self.stats.team_zero_side_pct()
    }

    pub fn team_one_side_pct(&self) -> f32 {
        self.stats.team_one_side_pct()
    }

    pub fn neutral_pct(&self) -> f32 {
        self.stats.neutral_pct()
    }

    fn apply_pressure_time(stats: &mut PressureStats, half: PressureHalfLabel, dt: f32) {
        match half {
            PressureHalfLabel::TeamZeroSide => stats.team_zero_side_time += dt,
            PressureHalfLabel::TeamOneSide => stats.team_one_side_time += dt,
            PressureHalfLabel::Neutral => stats.neutral_time += dt,
        }
        stats.labeled_time.add([half.as_label()], dt);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        if !live_play_state.is_live_play {
            return Ok(());
        }
        if let Some(ball) = ball.sample() {
            self.stats.tracked_time += frame.dt;
            let ball_y = ball.position().y;
            let half = if ball_y.abs() <= self.config.neutral_zone_half_width_y {
                PressureHalfLabel::Neutral
            } else if ball_y < 0.0 {
                PressureHalfLabel::TeamZeroSide
            } else {
                PressureHalfLabel::TeamOneSide
            };
            Self::apply_pressure_time(&mut self.stats, half, frame.dt);
        }
        Ok(())
    }
}
