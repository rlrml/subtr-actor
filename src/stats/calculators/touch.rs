use super::*;

const SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 320.0;
const HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 900.0;
const AERIAL_TOUCH_MIN_PLAYER_Z: f32 = AIR_DRIBBLE_MIN_PLAYER_Z;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TouchKind {
    Control,
    MediumHit,
    HardHit,
}

const ALL_TOUCH_KINDS: [TouchKind; 3] =
    [TouchKind::Control, TouchKind::MediumHit, TouchKind::HardHit];

impl TouchKind {
    fn as_label(self) -> StatLabel {
        let value = match self {
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
    #[serde(default)]
    pub total_ball_travel_distance: f32,
    #[serde(default)]
    pub total_ball_advance_distance: f32,
    #[serde(default)]
    pub total_ball_retreat_distance: f32,
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
struct PendingFiftyFiftyMovement {
    start_frame: usize,
    travel_distance: f32,
    y_delta: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchCalculator {
    player_stats: HashMap<PlayerId, TouchStats>,
    current_last_touch_player: Option<PlayerId>,
    previous_ball_velocity: Option<glam::Vec3>,
    previous_ball_position: Option<glam::Vec3>,
    pending_fifty_fifty_movement: Option<PendingFiftyFiftyMovement>,
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
        controlled_touch_kind: Option<BallCarryKind>,
    ) -> TouchClassification {
        let kind = if controlled_touch_kind.is_some()
            || ball_speed_change <= SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD
        {
            TouchKind::Control
        } else if ball_speed_change < HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD {
            TouchKind::MediumHit
        } else {
            TouchKind::HardHit
        };

        TouchClassification { kind, height_band }
    }

    fn height_band_for_touch(sample: Option<&PlayerVerticalSample>) -> PlayerVerticalBand {
        let Some(sample) = sample else {
            return PlayerVerticalBand::Ground;
        };

        if sample.height < AERIAL_TOUCH_MIN_PLAYER_Z {
            PlayerVerticalBand::Ground
        } else {
            sample.band
        }
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

    fn controlled_touch_kind(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<BallCarryKind> {
        let ball = ball.sample()?;
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(|player| {
                BallCarryCalculator::carry_frame_sample(player, ball).map(|sample| sample.kind)
            })
    }

    fn apply_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        touch_events: &[TouchEvent],
    ) {
        let ball_speed_change = Self::ball_speed_change(frame, ball, self.previous_ball_velocity);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let height_band = Self::height_band_for_touch(vertical_state.sample(player_id));
            let controlled_touch_kind = Self::controlled_touch_kind(ball, players, player_id);
            let classification =
                Self::classify_touch(height_band, ball_speed_change, controlled_touch_kind);
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

    fn apply_ball_movement_credit(
        &mut self,
        player_id: &PlayerId,
        team_is_team_0: bool,
        delta: glam::Vec3,
        travel_distance: f32,
    ) {
        let team_forward_sign = if team_is_team_0 { 1.0 } else { -1.0 };
        let advance_distance = delta.y * team_forward_sign;
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        stats.total_ball_travel_distance += travel_distance;
        if advance_distance >= 0.0 {
            stats.total_ball_advance_distance += advance_distance;
        } else {
            stats.total_ball_retreat_distance += -advance_distance;
        }
    }

    fn resolved_fifty_fifty_winner(event: &FiftyFiftyEvent) -> Option<(&PlayerId, bool)> {
        let winning_team_is_team_0 = event.winning_team_is_team_0?;
        let player = if winning_team_is_team_0 {
            event.team_zero_player.as_ref()
        } else {
            event.team_one_player.as_ref()
        }?;
        Some((player, winning_team_is_team_0))
    }

    fn buffer_fifty_fifty_movement(
        &mut self,
        start_frame: usize,
        delta: glam::Vec3,
        travel_distance: f32,
    ) {
        let pending = self
            .pending_fifty_fifty_movement
            .get_or_insert(PendingFiftyFiftyMovement {
                start_frame,
                travel_distance: 0.0,
                y_delta: 0.0,
            });
        if pending.start_frame != start_frame {
            *pending = PendingFiftyFiftyMovement {
                start_frame,
                travel_distance: 0.0,
                y_delta: 0.0,
            };
        }
        pending.travel_distance += travel_distance;
        pending.y_delta += delta.y;
    }

    fn flush_fifty_fifty_movement(&mut self, event: &FiftyFiftyEvent) {
        let Some(pending) = self.pending_fifty_fifty_movement.take() else {
            return;
        };
        if pending.start_frame != event.start_frame {
            return;
        }
        let Some((player_id, team_is_team_0)) = Self::resolved_fifty_fifty_winner(event) else {
            return;
        };

        let team_forward_sign = if team_is_team_0 { 1.0 } else { -1.0 };
        let advance_distance = pending.y_delta * team_forward_sign;
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        stats.total_ball_travel_distance += pending.travel_distance;
        if advance_distance >= 0.0 {
            stats.total_ball_advance_distance += advance_distance;
        } else {
            stats.total_ball_retreat_distance += -advance_distance;
        }
    }

    fn credit_ball_movement(
        &mut self,
        ball: &BallFrameState,
        possession_state: &PossessionState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play: bool,
    ) {
        let current_ball_position = ball.position();
        if !live_play {
            self.previous_ball_position = current_ball_position;
            self.pending_fifty_fifty_movement = None;
            return;
        }

        let Some(current_ball_position) = current_ball_position else {
            self.previous_ball_position = None;
            self.pending_fifty_fifty_movement = None;
            return;
        };
        let Some(previous_ball_position) = self.previous_ball_position else {
            self.previous_ball_position = Some(current_ball_position);
            return;
        };
        self.previous_ball_position = Some(current_ball_position);

        let delta = current_ball_position - previous_ball_position;
        let travel_distance = delta.length();
        if travel_distance <= f32::EPSILON {
            return;
        }

        if let Some(active_event) = fifty_fifty_state.active_event.as_ref() {
            self.buffer_fifty_fifty_movement(active_event.start_frame, delta, travel_distance);
            return;
        }

        if let Some(event) = fifty_fifty_state.resolved_events.last() {
            self.buffer_fifty_fifty_movement(event.start_frame, delta, travel_distance);
            self.flush_fifty_fifty_movement(event);
            return;
        }

        self.pending_fifty_fifty_movement = None;

        let (Some(player_id), Some(team_is_team_0)) = (
            possession_state.active_player_before_sample.as_ref(),
            possession_state.active_team_before_sample,
        ) else {
            return;
        };

        self.apply_ball_movement_credit(player_id, team_is_team_0, delta, travel_distance);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        touch_state: &TouchState,
        possession_state: &PossessionState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if !live_play {
            self.current_last_touch_player = None;
            self.previous_ball_velocity = ball.velocity();
            self.previous_ball_position = ball.position();
            self.pending_fifty_fifty_movement = None;
            return Ok(());
        }

        self.begin_sample(frame);
        self.apply_touch_events(
            frame,
            ball,
            players,
            vertical_state,
            &touch_state.touch_events,
        );
        self.credit_ball_movement(ball, possession_state, fifty_fifty_state, live_play);
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

#[cfg(test)]
#[path = "touch_tests.rs"]
mod tests;
