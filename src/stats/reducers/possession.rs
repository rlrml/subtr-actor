use super::*;

const PENDING_TURNOVER_CONFIRMATION_WINDOW_SECONDS: f32 = 1.25;
const LOOSE_BALL_TIMEOUT_SECONDS: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PossessionStateLabel {
    TeamZero,
    TeamOne,
    Neutral,
}

impl PossessionStateLabel {
    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::TeamZero => "team_zero",
            Self::TeamOne => "team_one",
            Self::Neutral => "neutral",
        };
        StatLabel::new("possession_state", value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
enum FieldThirdLabel {
    TeamZeroThird,
    NeutralThird,
    TeamOneThird,
}

impl FieldThirdLabel {
    fn from_ball(ball: &BallSample) -> Self {
        let ball_y = ball.position().y;
        if ball_y < -FIELD_ZONE_BOUNDARY_Y {
            Self::TeamZeroThird
        } else if ball_y > FIELD_ZONE_BOUNDARY_Y {
            Self::TeamOneThird
        } else {
            Self::NeutralThird
        }
    }

    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::TeamZeroThird => "team_zero_third",
            Self::NeutralThird => "neutral_third",
            Self::TeamOneThird => "team_one_third",
        };
        StatLabel::new("field_third", value)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PossessionStats {
    pub tracked_time: f32,
    pub team_zero_time: f32,
    pub team_one_time: f32,
    pub neutral_time: f32,
    #[serde(skip_serializing_if = "LabeledFloatSums::is_empty")]
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
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PossessionTracker {
    current_team_is_team_0: Option<bool>,
    current_player: Option<PlayerId>,
    last_possession_touch_time: Option<f32>,
    pending_turnover_team_is_team_0: Option<bool>,
    pending_turnover_touch_time: Option<f32>,
}

impl PossessionTracker {
    fn clear_pending_turnover(&mut self) {
        self.pending_turnover_team_is_team_0 = None;
        self.pending_turnover_touch_time = None;
    }

    pub(crate) fn reset(&mut self) {
        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
        self.clear_pending_turnover();
    }

    fn expire_pending_turnover(&mut self, time: f32) {
        let Some(pending_time) = self.pending_turnover_touch_time else {
            return;
        };
        if time - pending_time < PENDING_TURNOVER_CONFIRMATION_WINDOW_SECONDS {
            return;
        }

        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
        self.clear_pending_turnover();
    }

    fn expire_loose_ball(&mut self, time: f32) {
        if self.pending_turnover_team_is_team_0.is_some() {
            return;
        }
        let Some(last_touch_time) = self.last_possession_touch_time else {
            return;
        };
        if time - last_touch_time < LOOSE_BALL_TIMEOUT_SECONDS {
            return;
        }

        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
    }

    fn register_single_team_touch(&mut self, team_is_team_0: bool, time: f32) {
        if self.current_team_is_team_0 == Some(team_is_team_0) {
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        if self.current_team_is_team_0.is_none() {
            self.current_team_is_team_0 = Some(team_is_team_0);
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        if self.pending_turnover_team_is_team_0 == Some(team_is_team_0) {
            self.current_team_is_team_0 = Some(team_is_team_0);
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        self.pending_turnover_team_is_team_0 = Some(team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn register_contested_touch(&mut self, time: f32) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.clear_pending_turnover();
            return;
        };

        self.last_possession_touch_time = Some(time);
        self.pending_turnover_team_is_team_0 = Some(!current_team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn update_player_control(
        &mut self,
        active_team_before_sample: Option<bool>,
        touched_team_zero_player: Option<&PlayerId>,
        touched_team_one_player: Option<&PlayerId>,
    ) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.current_player = None;
            return;
        };

        if self.pending_turnover_team_is_team_0.is_some() {
            self.current_player = None;
            return;
        }

        let controlling_touch_player = if current_team_is_team_0 {
            touched_team_zero_player
        } else {
            touched_team_one_player
        };
        if let Some(player) = controlling_touch_player {
            self.current_player = Some(player.clone());
            return;
        }

        if active_team_before_sample != self.current_team_is_team_0 {
            self.current_player = None;
        }
    }

    pub(crate) fn update(
        &mut self,
        sample: &StatsSample,
        touch_events: &[TouchEvent],
    ) -> PossessionState {
        self.expire_pending_turnover(sample.time);
        self.expire_loose_ball(sample.time);

        let active_team_before_sample = self.current_team_is_team_0;
        let active_player_before_sample = self.current_player.clone();
        let touched_team_zero = touch_events.iter().any(|touch| touch.team_is_team_0);
        let touched_team_one = touch_events.iter().any(|touch| !touch.team_is_team_0);
        let touched_team_zero_player = touch_events
            .iter()
            .rev()
            .find(|touch| touch.team_is_team_0)
            .and_then(|touch| touch.player.clone());
        let touched_team_one_player = touch_events
            .iter()
            .rev()
            .find(|touch| !touch.team_is_team_0)
            .and_then(|touch| touch.player.clone());

        match (touched_team_zero, touched_team_one) {
            (true, true) => self.register_contested_touch(sample.time),
            (true, false) => self.register_single_team_touch(true, sample.time),
            (false, true) => self.register_single_team_touch(false, sample.time),
            (false, false) => {}
        }
        self.update_player_control(
            active_team_before_sample,
            touched_team_zero_player.as_ref(),
            touched_team_one_player.as_ref(),
        );

        PossessionState {
            active_team_before_sample,
            current_team_is_team_0: self.current_team_is_team_0,
            active_player_before_sample,
            current_player: self.current_player.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PossessionReducer {
    stats: PossessionStats,
    tracker: PossessionTracker,
    live_play_tracker: LivePlayTracker,
}

impl PossessionReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &PossessionStats {
        &self.stats
    }

    fn apply_possession_time(
        stats: &mut PossessionStats,
        state: PossessionStateLabel,
        field_third: Option<FieldThirdLabel>,
        dt: f32,
    ) {
        match state {
            PossessionStateLabel::TeamZero => stats.team_zero_time += dt,
            PossessionStateLabel::TeamOne => stats.team_one_time += dt,
            PossessionStateLabel::Neutral => stats.neutral_time += dt,
        }
        if let Some(field_third) = field_third {
            stats
                .labeled_time
                .add([state.as_label(), field_third.as_label()], dt);
        } else {
            stats.labeled_time.add([state.as_label()], dt);
        }
    }
}

impl StatsReducer for PossessionReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        let active_team_before_sample = if live_play {
            self.tracker
                .update(sample, &sample.touch_events)
                .active_team_before_sample
        } else {
            self.tracker.reset();
            None
        };

        if live_play {
            self.stats.tracked_time += sample.dt;
            let field_third = sample.ball.as_ref().map(FieldThirdLabel::from_ball);
            if let Some(possession_team_is_team_0) = active_team_before_sample {
                let state = if possession_team_is_team_0 {
                    PossessionStateLabel::TeamZero
                } else {
                    PossessionStateLabel::TeamOne
                };
                Self::apply_possession_time(&mut self.stats, state, field_third, sample.dt);
            } else {
                Self::apply_possession_time(
                    &mut self.stats,
                    PossessionStateLabel::Neutral,
                    field_third,
                    sample.dt,
                );
            }
        }
        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        let active_team_before_sample = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .and_then(|state| state.active_team_before_sample);

        if live_play {
            self.stats.tracked_time += sample.dt;
            let field_third = sample.ball.as_ref().map(FieldThirdLabel::from_ball);
            if let Some(possession_team_is_team_0) = active_team_before_sample {
                let state = if possession_team_is_team_0 {
                    PossessionStateLabel::TeamZero
                } else {
                    PossessionStateLabel::TeamOne
                };
                Self::apply_possession_time(&mut self.stats, state, field_third, sample.dt);
            } else {
                Self::apply_possession_time(
                    &mut self.stats,
                    PossessionStateLabel::Neutral,
                    field_third,
                    sample.dt,
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

    use super::*;

    fn ball(ball_y: f32) -> BallSample {
        BallSample {
            rigid_body: RigidBody {
                sleeping: false,
                location: Vector3f {
                    x: 0.0,
                    y: ball_y,
                    z: 100.0,
                },
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

    fn sample(frame_number: usize, time: f32, ball_y: f32, touch_teams: &[bool]) -> StatsSample {
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
            touch_events: touch_teams
                .iter()
                .enumerate()
                .map(|(index, &team_is_team_0)| TouchEvent {
                    time,
                    frame: frame_number,
                    player: Some(RemoteId::Steam(index as u64 + 1)),
                    team_is_team_0,
                    closest_approach_distance: None,
                })
                .collect(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    #[test]
    fn possession_reducer_tracks_labeled_possession_time() {
        let mut reducer = PossessionReducer::new();

        reducer.on_sample(&sample(0, 0.0, 0.0, &[])).unwrap();
        reducer.on_sample(&sample(1, 1.0, 0.0, &[true])).unwrap();
        reducer.on_sample(&sample(2, 2.0, 0.0, &[])).unwrap();
        reducer.on_sample(&sample(3, 3.0, 0.0, &[false])).unwrap();
        reducer.on_sample(&sample(4, 4.0, 0.0, &[false])).unwrap();
        reducer.on_sample(&sample(5, 5.0, 0.0, &[])).unwrap();

        let stats = reducer.stats();
        assert_eq!(stats.tracked_time, 6.0);
        assert_eq!(stats.neutral_time, 2.0);
        assert_eq!(stats.team_zero_time, 3.0);
        assert_eq!(stats.team_one_time, 1.0);
        assert_eq!(
            stats.time_with_labels(&[StatLabel::new("possession_state", "neutral")]),
            2.0
        );
        assert_eq!(
            stats.time_with_labels(&[StatLabel::new("possession_state", "team_zero")]),
            3.0
        );
        assert_eq!(
            stats.time_with_labels(&[StatLabel::new("possession_state", "team_one")]),
            1.0
        );
    }

    #[test]
    fn possession_reducer_tracks_possession_time_by_field_third() {
        let mut reducer = PossessionReducer::new();

        reducer.on_sample(&sample(0, 0.0, -3000.0, &[])).unwrap();
        reducer.on_sample(&sample(1, 1.0, 0.0, &[true])).unwrap();
        reducer
            .on_sample(&sample(2, 2.0, -3000.0, &[true]))
            .unwrap();
        reducer.on_sample(&sample(3, 3.0, 0.0, &[])).unwrap();
        reducer.on_sample(&sample(4, 4.0, 3000.0, &[])).unwrap();
        reducer
            .on_sample(&sample(5, 5.0, 3000.0, &[false]))
            .unwrap();
        reducer
            .on_sample(&sample(6, 6.0, -3000.0, &[false]))
            .unwrap();

        let stats = reducer.stats();
        assert_eq!(stats.tracked_time, 7.0);
        assert_eq!(
            stats.time_with_labels(&[
                StatLabel::new("possession_state", "neutral"),
                StatLabel::new("field_third", "team_zero_third"),
            ]),
            1.0
        );
        assert_eq!(
            stats.time_with_labels(&[
                StatLabel::new("possession_state", "neutral"),
                StatLabel::new("field_third", "neutral_third"),
            ]),
            1.0
        );
        assert_eq!(
            stats.time_with_labels(&[
                StatLabel::new("possession_state", "neutral"),
                StatLabel::new("field_third", "team_one_third"),
            ]),
            1.0
        );
        assert_eq!(
            stats.time_with_labels(&[
                StatLabel::new("possession_state", "team_zero"),
                StatLabel::new("field_third", "team_zero_third"),
            ]),
            1.0
        );
        assert_eq!(
            stats.time_with_labels(&[
                StatLabel::new("possession_state", "team_zero"),
                StatLabel::new("field_third", "neutral_third"),
            ]),
            1.0
        );
        assert_eq!(
            stats.time_with_labels(&[
                StatLabel::new("possession_state", "team_zero"),
                StatLabel::new("field_third", "team_one_third"),
            ]),
            1.0
        );
        assert_eq!(
            stats.time_with_labels(&[
                StatLabel::new("possession_state", "team_one"),
                StatLabel::new("field_third", "team_zero_third"),
            ]),
            1.0
        );
    }
}
