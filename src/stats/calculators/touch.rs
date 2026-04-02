use super::*;

const SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 320.0;
const HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 900.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TouchKind {
    Dribble,
    Control,
    MediumHit,
    HardHit,
}

const ALL_TOUCH_KINDS: [TouchKind; 4] = [
    TouchKind::Dribble,
    TouchKind::Control,
    TouchKind::MediumHit,
    TouchKind::HardHit,
];

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
    height_band: PlayerVerticalBand,
}

impl TouchClassification {
    fn labels(self) -> [StatLabel; 2] {
        [self.kind.as_label(), self.height_band.as_label()]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
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
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
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

    pub fn complete_labeled_touch_counts(&self) -> LabeledCounts {
        let mut entries: Vec<_> = ALL_PLAYER_VERTICAL_BANDS
            .into_iter()
            .flat_map(|height_band| {
                ALL_TOUCH_KINDS.into_iter().map(move |kind| {
                    let mut labels = vec![kind.as_label(), height_band.as_label()];
                    labels.sort();
                    LabeledCountEntry {
                        count: self.labeled_touch_counts.count_exact(&labels),
                        labels,
                    }
                })
            })
            .collect();

        entries.sort_by(|left, right| left.labels.cmp(&right.labels));

        LabeledCounts { entries }
    }

    pub fn with_complete_labeled_touch_counts(mut self) -> Self {
        self.labeled_touch_counts = self.complete_labeled_touch_counts();
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchCalculator {
    player_stats: HashMap<PlayerId, TouchStats>,
    current_last_touch_player: Option<PlayerId>,
    previous_ball_velocity: Option<glam::Vec3>,
}

impl TouchCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, TouchStats> {
        &self.player_stats
    }

    fn ball_speed_change(
        frame: &FrameInfo,
        ball: &BallFrameState,
        previous_ball_velocity: Option<glam::Vec3>,
    ) -> f32 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = ball.sample() else {
            return 0.0;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return 0.0;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * frame.dt.max(0.0));
        let residual_linear_impulse =
            ball.velocity() - previous_ball_velocity - expected_linear_delta;
        residual_linear_impulse.length()
    }

    fn classify_touch(
        height_band: PlayerVerticalBand,
        ball_speed_change: f32,
    ) -> TouchClassification {
        let kind = if ball_speed_change <= SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD {
            if height_band.is_airborne() {
                TouchKind::Control
            } else {
                TouchKind::Dribble
            }
        } else if ball_speed_change < HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD {
            TouchKind::MediumHit
        } else {
            TouchKind::HardHit
        };

        TouchClassification { kind, height_band }
    }

    fn apply_touch_classification(stats: &mut TouchStats, classification: TouchClassification) {
        match classification.height_band {
            PlayerVerticalBand::Ground => {}
            PlayerVerticalBand::LowAir => stats.aerial_touch_count += 1,
            PlayerVerticalBand::HighAir => {
                stats.aerial_touch_count += 1;
                stats.high_aerial_touch_count += 1;
            }
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

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_touch = false;
            stats.time_since_last_touch = stats
                .last_touch_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_touch = stats
                .last_touch_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    fn apply_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        vertical_state: &PlayerVerticalState,
        touch_events: &[TouchEvent],
    ) {
        let ball_speed_change = Self::ball_speed_change(frame, ball, self.previous_ball_velocity);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let height_band = vertical_state
                .band_for_player(player_id)
                .unwrap_or(PlayerVerticalBand::Ground);
            let classification = Self::classify_touch(height_band, ball_speed_change);
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.touch_count += 1;
            Self::apply_touch_classification(stats, classification);
            stats.last_touch_time = Some(touch_event.time);
            stats.last_touch_frame = Some(touch_event.frame);
            stats.time_since_last_touch = Some((frame.time - touch_event.time).max(0.0));
            stats.frames_since_last_touch =
                Some(frame.frame_number.saturating_sub(touch_event.frame));
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

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        vertical_state: &PlayerVerticalState,
        touch_state: &TouchState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if !live_play {
            self.current_last_touch_player = None;
            self.previous_ball_velocity = ball.velocity();
            return Ok(());
        }

        self.begin_sample(frame);
        self.apply_touch_events(frame, ball, vertical_state, &touch_state.touch_events);
        self.previous_ball_velocity = ball.velocity();

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
