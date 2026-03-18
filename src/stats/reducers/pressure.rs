use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PressureHalfLabel {
    TeamZeroSide,
    TeamOneSide,
}

impl PressureHalfLabel {
    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::TeamZeroSide => "team_zero_side",
            Self::TeamOneSide => "team_one_side",
        };
        StatLabel::new("field_half", value)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PressureStats {
    pub tracked_time: f32,
    pub team_zero_side_time: f32,
    pub team_one_side_time: f32,
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

    pub fn time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_time.sum_matching(labels)
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

    fn apply_pressure_time(stats: &mut PressureStats, half: PressureHalfLabel, dt: f32) {
        match half {
            PressureHalfLabel::TeamZeroSide => stats.team_zero_side_time += dt,
            PressureHalfLabel::TeamOneSide => stats.team_one_side_time += dt,
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
            if ball.position().y < 0.0 {
                Self::apply_pressure_time(
                    &mut self.stats,
                    PressureHalfLabel::TeamZeroSide,
                    sample.dt,
                );
            } else {
                Self::apply_pressure_time(
                    &mut self.stats,
                    PressureHalfLabel::TeamOneSide,
                    sample.dt,
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use boxcars::{Quaternion, RigidBody, Vector3f};

    use super::*;

    fn ball(y: f32) -> BallSample {
        BallSample {
            rigid_body: RigidBody {
                sleeping: false,
                location: Vector3f { x: 0.0, y, z: 100.0 },
                rotation: Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                linear_velocity: Some(Vector3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                angular_velocity: Some(Vector3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            },
        }
    }

    fn sample(frame_number: usize, time: f32, ball_y: f32) -> StatsSample {
        StatsSample {
            frame_number,
            time,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([1, 1]),
            ball: Some(ball(ball_y)),
            players: Vec::new(),
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    #[test]
    fn pressure_reducer_tracks_labeled_half_time() {
        let mut reducer = PressureReducer::new();

        reducer.on_sample(&sample(0, 0.0, -100.0)).unwrap();
        reducer.on_sample(&sample(1, 1.0, 200.0)).unwrap();

        let stats = reducer.stats();
        assert_eq!(stats.tracked_time, 2.0);
        assert_eq!(stats.team_zero_side_time, 1.0);
        assert_eq!(stats.team_one_side_time, 1.0);
        assert_eq!(
            stats.time_with_labels(&[StatLabel::new("field_half", "team_zero_side")]),
            1.0
        );
        assert_eq!(
            stats.time_with_labels(&[StatLabel::new("field_half", "team_one_side")]),
            1.0
        );
    }
}
