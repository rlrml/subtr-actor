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

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PressureStats {
    pub tracked_time: f32,
    pub team_zero_side_time: f32,
    pub team_one_side_time: f32,
    pub neutral_time: f32,
    #[serde(skip_serializing_if = "LabeledFloatSums::is_empty")]
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct PressureReducerConfig {
    pub neutral_zone_half_width_y: f32,
}

impl Default for PressureReducerConfig {
    fn default() -> Self {
        Self {
            neutral_zone_half_width_y: DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PressureReducer {
    config: PressureReducerConfig,
    stats: PressureStats,
    live_play_tracker: LivePlayTracker,
}

impl PressureReducer {
    pub fn new() -> Self {
        Self::with_config(PressureReducerConfig::default())
    }

    pub fn with_config(config: PressureReducerConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn stats(&self) -> &PressureStats {
        &self.stats
    }

    pub fn config(&self) -> &PressureReducerConfig {
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
}

impl StatsReducer for PressureReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            return Ok(());
        }
        if let Some(ball) = &sample.ball {
            self.stats.tracked_time += sample.dt;
            let ball_y = ball.position().y;
            let half = if ball_y.abs() <= self.config.neutral_zone_half_width_y {
                PressureHalfLabel::Neutral
            } else if ball_y < 0.0 {
                PressureHalfLabel::TeamZeroSide
            } else {
                PressureHalfLabel::TeamOneSide
            };
            Self::apply_pressure_time(&mut self.stats, half, sample.dt);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "pressure_test.rs"]
mod tests;
