use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PossessionStats {
    pub tracked_time: f32,
    pub team_zero_time: f32,
    pub team_one_time: f32,
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
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PossessionReducer {
    stats: PossessionStats,
    current_team_is_team_0: Option<bool>,
    live_play_tracker: LivePlayTracker,
}

impl PossessionReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &PossessionStats {
        &self.stats
    }
}

impl StatsReducer for PossessionReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        let active_team_before_sample = if sample.touch_events.is_empty() {
            self.current_team_is_team_0
                .or(sample.possession_team_is_team_0)
        } else {
            self.current_team_is_team_0
        };

        if live_play {
            if let Some(possession_team_is_team_0) = active_team_before_sample {
                self.stats.tracked_time += sample.dt;
                if possession_team_is_team_0 {
                    self.stats.team_zero_time += sample.dt;
                } else {
                    self.stats.team_one_time += sample.dt;
                }
            }
        }

        if let Some(last_touch) = sample.touch_events.last() {
            self.current_team_is_team_0 = Some(last_touch.team_is_team_0);
        } else {
            self.current_team_is_team_0 = sample
                .possession_team_is_team_0
                .or(self.current_team_is_team_0);
        }
        Ok(())
    }
}
