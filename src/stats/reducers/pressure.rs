use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PressureReducer {
    team_zero_side_duration: f32,
    team_one_side_duration: f32,
    live_play_tracker: LivePlayTracker,
}

impl PressureReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn team_zero_side_duration(&self) -> f32 {
        self.team_zero_side_duration
    }

    pub fn team_one_side_duration(&self) -> f32 {
        self.team_one_side_duration
    }

    pub fn total_tracked_duration(&self) -> f32 {
        self.team_zero_side_duration + self.team_one_side_duration
    }

    pub fn team_zero_side_pct(&self) -> f32 {
        if self.total_tracked_duration() == 0.0 {
            0.0
        } else {
            self.team_zero_side_duration * 100.0 / self.total_tracked_duration()
        }
    }

    pub fn team_one_side_pct(&self) -> f32 {
        if self.total_tracked_duration() == 0.0 {
            0.0
        } else {
            self.team_one_side_duration * 100.0 / self.total_tracked_duration()
        }
    }
}

impl StatsReducer for PressureReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            return Ok(());
        }
        if let Some(ball) = &sample.ball {
            if ball.position().y < 0.0 {
                self.team_zero_side_duration += sample.dt;
            } else {
                self.team_one_side_duration += sample.dt;
            }
        }
        Ok(())
    }
}
