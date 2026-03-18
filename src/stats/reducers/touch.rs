use super::*;

const SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 320.0;
const HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 900.0;
const AERIAL_TOUCH_Z_THRESHOLD: f32 = 180.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TouchKind {
    Dribble,
    Control,
    MediumHit,
    HardHit,
}

impl TouchKind {
    fn as_label(self) -> StatLabel {
        let value = match self {
            Self::Dribble => "dribble",
            Self::Control => "control",
            Self::MediumHit => "medium_hit",
            Self::HardHit => "hard_hit",
        };
        StatLabel::new("kind", value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TouchClassification {
    kind: TouchKind,
    is_aerial: bool,
    is_high_aerial: bool,
}

impl TouchClassification {
    fn labels(self) -> [StatLabel; 3] {
        [
            self.kind.as_label(),
            StatLabel::new("aerial", if self.is_aerial { "true" } else { "false" }),
            StatLabel::new(
                "high_aerial",
                if self.is_high_aerial { "true" } else { "false" },
            ),
        ]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TouchStats {
    pub touch_count: u32,
    pub dribble_touch_count: u32,
    pub control_touch_count: u32,
    pub medium_hit_count: u32,
    pub hard_hit_count: u32,
    pub aerial_touch_count: u32,
    pub high_aerial_touch_count: u32,
    pub is_last_touch: bool,
    pub last_touch_time: Option<f32>,
    pub last_touch_frame: Option<usize>,
    pub time_since_last_touch: Option<f32>,
    pub frames_since_last_touch: Option<usize>,
    pub last_ball_speed_change: Option<f32>,
    pub max_ball_speed_change: f32,
    pub cumulative_ball_speed_change: f32,
    #[serde(skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_touch_counts: LabeledCounts,
}

impl TouchStats {
    pub fn average_ball_speed_change(&self) -> f32 {
        if self.touch_count == 0 {
            0.0
        } else {
            self.cumulative_ball_speed_change / self.touch_count as f32
        }
    }

    pub fn touch_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_touch_counts.count_matching(labels)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchReducer {
    player_stats: HashMap<PlayerId, TouchStats>,
    current_last_touch_player: Option<PlayerId>,
    previous_ball_velocity: Option<glam::Vec3>,
    live_play_tracker: LivePlayTracker,
}

impl TouchReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, TouchStats> {
        &self.player_stats
    }

    fn ball_speed_change(sample: &StatsSample, previous_ball_velocity: Option<glam::Vec3>) -> f32 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = sample.ball.as_ref() else {
            return 0.0;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return 0.0;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * sample.dt.max(0.0));
        let residual_linear_impulse =
            ball.velocity() - previous_ball_velocity - expected_linear_delta;
        residual_linear_impulse.length()
    }

    fn classify_touch(player_height: Option<f32>, ball_speed_change: f32) -> TouchClassification {
        let player_height = player_height.unwrap_or(0.0);
        let is_aerial_touch = player_height >= AERIAL_TOUCH_Z_THRESHOLD;
        let is_high_aerial_touch = player_height >= HIGH_AIR_Z_THRESHOLD;

        let kind = if ball_speed_change <= SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD {
            if is_aerial_touch {
                TouchKind::Control
            } else {
                TouchKind::Dribble
            }
        } else if ball_speed_change < HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD {
            TouchKind::MediumHit
        } else {
            TouchKind::HardHit
        };

        TouchClassification {
            kind,
            is_aerial: is_aerial_touch,
            is_high_aerial: is_high_aerial_touch,
        }
    }

    fn apply_touch_classification(stats: &mut TouchStats, classification: TouchClassification) {
        if classification.is_aerial {
            stats.aerial_touch_count += 1;
        }
        if classification.is_high_aerial {
            stats.high_aerial_touch_count += 1;
        }

        match classification.kind {
            TouchKind::Dribble => stats.dribble_touch_count += 1,
            TouchKind::Control => stats.control_touch_count += 1,
            TouchKind::MediumHit => stats.medium_hit_count += 1,
            TouchKind::HardHit => stats.hard_hit_count += 1,
        }

        stats
            .labeled_touch_counts
            .increment(classification.labels());
    }

    fn begin_sample(&mut self, sample: &StatsSample) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_touch = false;
            stats.time_since_last_touch = stats
                .last_touch_time
                .map(|time| (sample.time - time).max(0.0));
            stats.frames_since_last_touch = stats
                .last_touch_frame
                .map(|frame| sample.frame_number.saturating_sub(frame));
        }
    }

    fn apply_touch_events(&mut self, sample: &StatsSample, touch_events: &[TouchEvent]) {
        let ball_speed_change = Self::ball_speed_change(sample, self.previous_ball_velocity);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let player_height = sample
                .players
                .iter()
                .find(|player| player.player_id == *player_id)
                .and_then(PlayerSample::position)
                .map(|position| position.z);
            let classification = Self::classify_touch(player_height, ball_speed_change);
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.touch_count += 1;
            Self::apply_touch_classification(stats, classification);
            stats.last_touch_time = Some(touch_event.time);
            stats.last_touch_frame = Some(touch_event.frame);
            stats.time_since_last_touch = Some((sample.time - touch_event.time).max(0.0));
            stats.frames_since_last_touch =
                Some(sample.frame_number.saturating_sub(touch_event.frame));
            stats.last_ball_speed_change = Some(ball_speed_change);
            stats.max_ball_speed_change = stats.max_ball_speed_change.max(ball_speed_change);
            stats.cumulative_ball_speed_change += ball_speed_change;
        }

        if let Some(last_touch) = touch_events.last() {
            self.current_last_touch_player = last_touch.player.clone();
        }

        if let Some(player_id) = self.current_last_touch_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_touch = true;
            }
        }
    }
}

impl StatsReducer for TouchReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            return Ok(());
        }

        self.begin_sample(sample);
        self.apply_touch_events(sample, &sample.touch_events);
        self.previous_ball_velocity = sample.ball.as_ref().map(BallSample::velocity);

        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            return Ok(());
        }

        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();

        self.begin_sample(sample);
        self.apply_touch_events(sample, &touch_state.touch_events);
        self.previous_ball_velocity = sample.ball.as_ref().map(BallSample::velocity);

        if let Some(player_id) = touch_state.last_touch_player.as_ref() {
            self.current_last_touch_player = Some(player_id.clone());
        }

        if let Some(player_id) = self.current_last_touch_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_touch = true;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use boxcars::RemoteId;

    use super::*;

    fn rigid_body(x: f32, y: f32, z: f32, vx: f32, vy: f32, vz: f32) -> boxcars::RigidBody {
        boxcars::RigidBody {
            sleeping: false,
            location: boxcars::Vector3f { x, y, z },
            rotation: boxcars::Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(boxcars::Vector3f {
                x: vx,
                y: vy,
                z: vz,
            }),
            angular_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }
    }

    fn sample(
        frame_number: usize,
        time: f32,
        player_z: f32,
        ball_velocity_x: f32,
        touch: bool,
    ) -> StatsSample {
        StatsSample {
            frame_number,
            time,
            dt: 1.0 / 120.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([1, 1]),
            ball: Some(BallSample {
                rigid_body: rigid_body(0.0, 0.0, 120.0, ball_velocity_x, 0.0, 0.0),
            }),
            players: vec![
                PlayerSample {
                    player_id: RemoteId::Steam(1),
                    is_team_0: true,
                    rigid_body: Some(rigid_body(0.0, 0.0, player_z, 0.0, 0.0, 0.0)),
                    boost_amount: None,
                    last_boost_amount: None,
                    boost_active: false,
                    powerslide_active: false,
                    match_goals: None,
                    match_assists: None,
                    match_saves: None,
                    match_shots: None,
                    match_score: None,
                },
                PlayerSample {
                    player_id: RemoteId::Steam(2),
                    is_team_0: false,
                    rigid_body: Some(rigid_body(4000.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
                    boost_amount: None,
                    last_boost_amount: None,
                    boost_active: false,
                    powerslide_active: false,
                    match_goals: None,
                    match_assists: None,
                    match_saves: None,
                    match_shots: None,
                    match_score: None,
                },
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: if touch {
                vec![TouchEvent {
                    time,
                    frame: frame_number,
                    team_is_team_0: true,
                    player: Some(RemoteId::Steam(1)),
                    closest_approach_distance: Some(0.0),
                }]
            } else {
                Vec::new()
            },
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    #[test]
    fn touch_reducer_classifies_touch_strength_and_aerials() {
        let mut reducer = TouchReducer::new();

        let baseline = sample(0, 0.0, 0.0, 0.0, false);
        reducer.on_sample(&baseline).unwrap();

        let dribble = sample(1, 1.0 / 120.0, 0.0, 120.0, true);
        reducer.on_sample(&dribble).unwrap();

        let control = sample(2, 2.0 / 120.0, 240.0, 220.0, true);
        reducer.on_sample(&control).unwrap();

        let medium = sample(3, 3.0 / 120.0, 0.0, 720.0, true);
        reducer.on_sample(&medium).unwrap();

        let hard_high_aerial = sample(4, 4.0 / 120.0, 900.0, 1900.0, true);
        reducer.on_sample(&hard_high_aerial).unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.touch_count, 4);
        assert_eq!(stats.dribble_touch_count, 1);
        assert_eq!(stats.control_touch_count, 1);
        assert_eq!(stats.medium_hit_count, 1);
        assert_eq!(stats.hard_hit_count, 1);
        assert_eq!(stats.aerial_touch_count, 2);
        assert_eq!(stats.high_aerial_touch_count, 1);
        assert_eq!(
            stats.touch_count_with_labels(&[StatLabel::new("kind", "dribble")]),
            1
        );
        assert_eq!(
            stats.touch_count_with_labels(&[StatLabel::new("aerial", "true")]),
            2
        );
        assert_eq!(
            stats.touch_count_with_labels(&[
                StatLabel::new("kind", "hard_hit"),
                StatLabel::new("aerial", "true"),
            ]),
            1
        );
        assert_eq!(
            stats.touch_count_with_labels(&[
                StatLabel::new("kind", "hard_hit"),
                StatLabel::new("high_aerial", "true"),
            ]),
            1
        );
        assert!(stats.last_ball_speed_change.is_some());
        assert!(stats.max_ball_speed_change >= stats.average_ball_speed_change());
    }
}
