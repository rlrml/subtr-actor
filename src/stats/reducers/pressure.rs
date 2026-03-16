use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PressureStats {
    pub tracked_time: f32,
    pub team_zero_side_time: f32,
    pub team_one_side_time: f32,
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
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PressureReducer {
    stats: PressureStats,
    live_play_tracker: LivePlayTracker,
}

impl PressureReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &PressureStats {
        &self.stats
    }

    pub fn team_zero_side_duration(&self) -> f32 {
        self.stats.team_zero_side_time
    }

    pub fn team_one_side_duration(&self) -> f32 {
        self.stats.team_one_side_time
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
}

impl StatsReducer for PressureReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            return Ok(());
        }
        if let Some(ball) = &sample.ball {
            self.stats.tracked_time += sample.dt;
            if ball.position().y < 0.0 {
                self.stats.team_zero_side_time += sample.dt;
            } else {
                self.stats.team_one_side_time += sample.dt;
            }
        }
        Ok(())
    }
}
