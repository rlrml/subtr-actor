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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TouchSurface {
    Ground,
    Air,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TouchDodgeState {
    NoDodge,
    Dodge,
}

impl TouchKind {
    fn as_label_value(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::MediumHit => "medium_hit",
            Self::HardHit => "hard_hit",
        }
    }
}

impl TouchSurface {
    fn as_label_value(self) -> &'static str {
        match self {
            Self::Ground => "ground",
            Self::Air => "air",
            Self::Wall => "wall",
        }
    }
}

impl TouchDodgeState {
    fn from_dodge_active(dodge_active: bool) -> Self {
        if dodge_active {
            Self::Dodge
        } else {
            Self::NoDodge
        }
    }

    fn as_label_value(self) -> &'static str {
        match self {
            Self::NoDodge => "no_dodge",
            Self::Dodge => "dodge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TouchClassification {
    kind: TouchKind,
    height_band: PlayerVerticalBand,
    surface: TouchSurface,
    dodge_state: TouchDodgeState,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchStatsEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub kind: String,
    pub height_band: String,
    pub surface: String,
    pub dodge_state: String,
    pub ball_speed_change: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchBallMovementEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub travel_distance: f32,
    pub advance_distance: f32,
    pub retreat_distance: f32,
}

impl TouchBallMovementEvent {
    fn absorb_delta(&mut self, event: Self) {
        self.end_time = event.time;
        self.end_frame = event.frame;
        self.duration += event.duration;
        self.player_position = event.player_position;
        self.travel_distance += event.travel_distance;
        self.advance_distance += event.advance_distance;
        self.retreat_distance += event.retreat_distance;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchLastTouchEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    pub is_team_0: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PendingFiftyFiftyMovement {
    start_frame: usize,
    travel_distance: f32,
    y_delta: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct PendingTouchBallMovementEvent {
    player: PlayerId,
    is_team_0: bool,
    event: TouchBallMovementEvent,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchCalculator {
    events: EventStream<TouchStatsEvent>,
    ball_movement_events: EventStream<TouchBallMovementEvent>,
    last_touch_events: EventStream<TouchLastTouchEvent>,
    pending_ball_movement_event: Option<PendingTouchBallMovementEvent>,
    previous_ball_velocity: Option<glam::Vec3>,
    previous_ball_position: Option<glam::Vec3>,
    pending_fifty_fifty_movement: Option<PendingFiftyFiftyMovement>,
}

impl TouchCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[TouchStatsEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[TouchStatsEvent] {
        self.events.new_events()
    }

    pub fn ball_movement_events(&self) -> &[TouchBallMovementEvent] {
        self.ball_movement_events.all()
    }

    pub fn new_ball_movement_events(&self) -> &[TouchBallMovementEvent] {
        self.ball_movement_events.new_events()
    }

    pub fn projected_ball_movement_events(&self) -> Vec<TouchBallMovementEvent> {
        let mut events = self.ball_movement_events.all().to_vec();
        if let Some(pending) = &self.pending_ball_movement_event {
            events.push(pending.event.clone());
        }
        events
    }

    pub fn flush_pending_ball_movement_event(&mut self) {
        let Some(pending) = self.pending_ball_movement_event.take() else {
            return;
        };
        self.ball_movement_events.push(pending.event);
    }

    pub fn last_touch_events(&self) -> &[TouchLastTouchEvent] {
        self.last_touch_events.all()
    }

    pub fn new_last_touch_events(&self) -> &[TouchLastTouchEvent] {
        self.last_touch_events.new_events()
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
        surface: TouchSurface,
        dodge_state: TouchDodgeState,
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

        TouchClassification {
            kind,
            height_band,
            surface,
            dodge_state,
        }
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

    fn surface_for_touch(
        player_position: Option<glam::Vec3>,
        height_band: PlayerVerticalBand,
    ) -> TouchSurface {
        if player_position.is_some_and(player_is_on_wall) {
            TouchSurface::Wall
        } else if height_band.is_grounded() {
            TouchSurface::Ground
        } else {
            TouchSurface::Air
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

    fn player_position(players: &PlayerFrameState, player_id: &PlayerId) -> Option<glam::Vec3> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(PlayerSample::position)
    }

    fn player_dodge_active(players: &PlayerFrameState, player_id: &PlayerId) -> bool {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .is_some_and(|player| player.dodge_active)
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
            let surface =
                Self::surface_for_touch(Self::player_position(players, player_id), height_band);
            let dodge_state = TouchDodgeState::from_dodge_active(
                touch_event.dodge_contact || Self::player_dodge_active(players, player_id),
            );
            let controlled_touch_kind = Self::controlled_touch_kind(ball, players, player_id);
            let classification = Self::classify_touch(
                height_band,
                surface,
                dodge_state,
                ball_speed_change,
                controlled_touch_kind,
            );
            let event = TouchStatsEvent {
                time: touch_event.time,
                frame: touch_event.frame,
                sample_time: frame.time,
                sample_frame: frame.frame_number,
                player: player_id.clone(),
                player_position: touch_event
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array())
                    .or_else(|| {
                        Self::player_position(players, player_id)
                            .map(|position| position.to_array())
                    }),
                is_team_0: touch_event.team_is_team_0,
                kind: classification.kind.as_label_value().to_owned(),
                height_band: classification.height_band.as_label().value.to_owned(),
                surface: classification.surface.as_label_value().to_owned(),
                dodge_state: classification.dodge_state.as_label_value().to_owned(),
                ball_speed_change,
            };
            self.events.push(event);
        }

        if let Some(last_touch) = touch_events.last() {
            self.last_touch_events.push(TouchLastTouchEvent {
                time: last_touch.time,
                frame: last_touch.frame,
                sample_time: frame.time,
                sample_frame: frame.frame_number,
                is_team_0: last_touch.team_is_team_0,
                player: last_touch.player.clone(),
                player_position: last_touch
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array())
                    .or_else(|| {
                        last_touch
                            .player
                            .as_ref()
                            .and_then(|player_id| Self::player_position(players, player_id))
                            .map(|position| position.to_array())
                    }),
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_ball_movement_credit(
        &mut self,
        frame: usize,
        time: f32,
        duration: f32,
        player_id: &PlayerId,
        team_is_team_0: bool,
        player_position: Option<[f32; 3]>,
        delta: glam::Vec3,
        travel_distance: f32,
    ) {
        let team_forward_sign = if team_is_team_0 { 1.0 } else { -1.0 };
        let advance_distance = delta.y * team_forward_sign;
        let (advance_distance, retreat_distance) = if advance_distance >= 0.0 {
            (advance_distance, 0.0)
        } else {
            (0.0, -advance_distance)
        };
        let event = TouchBallMovementEvent {
            time,
            frame,
            end_time: time,
            end_frame: frame,
            duration,
            player: player_id.clone(),
            player_position,
            is_team_0: team_is_team_0,
            travel_distance,
            advance_distance,
            retreat_distance,
        };
        self.record_ball_movement_event(event);
    }

    fn record_ball_movement_event(&mut self, event: TouchBallMovementEvent) {
        let Some(pending) = self.pending_ball_movement_event.as_mut() else {
            self.pending_ball_movement_event = Some(PendingTouchBallMovementEvent {
                player: event.player.clone(),
                is_team_0: event.is_team_0,
                event,
            });
            return;
        };

        if pending.player == event.player && pending.is_team_0 == event.is_team_0 {
            pending.event.absorb_delta(event);
        } else {
            let previous =
                self.pending_ball_movement_event
                    .replace(PendingTouchBallMovementEvent {
                        player: event.player.clone(),
                        is_team_0: event.is_team_0,
                        event,
                    });
            let Some(previous) = previous else {
                return;
            };
            self.ball_movement_events.push(previous.event);
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
        let (advance_distance, retreat_distance) = if advance_distance >= 0.0 {
            (advance_distance, 0.0)
        } else {
            (0.0, -advance_distance)
        };
        let event = TouchBallMovementEvent {
            time: event.resolve_time,
            frame: event.resolve_frame,
            end_time: event.resolve_time,
            end_frame: event.resolve_frame,
            duration: 0.0,
            player: player_id.clone(),
            player_position: if team_is_team_0 {
                Some(event.team_zero_position)
            } else {
                Some(event.team_one_position)
            },
            is_team_0: team_is_team_0,
            travel_distance: pending.travel_distance,
            advance_distance,
            retreat_distance,
        };
        self.flush_pending_ball_movement_event();
        self.ball_movement_events.push(event);
    }

    fn credit_ball_movement(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        possession_state: &PossessionState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play: bool,
    ) {
        let current_ball_position = ball.position();
        if !live_play {
            self.flush_pending_ball_movement_event();
            self.previous_ball_position = current_ball_position;
            self.pending_fifty_fifty_movement = None;
            return;
        }

        let Some(current_ball_position) = current_ball_position else {
            self.flush_pending_ball_movement_event();
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
            self.flush_pending_ball_movement_event();
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
            self.flush_pending_ball_movement_event();
            return;
        };

        self.apply_ball_movement_credit(
            frame.frame_number,
            frame.time,
            frame.dt,
            player_id,
            team_is_team_0,
            players.player_position(player_id),
            delta,
            travel_distance,
        );
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
        self.events.begin_update();
        self.ball_movement_events.begin_update();
        self.last_touch_events.begin_update();
        if !live_play {
            self.flush_pending_ball_movement_event();
            self.previous_ball_velocity = ball.velocity();
            self.previous_ball_position = ball.position();
            self.pending_fifty_fifty_movement = None;
            return Ok(());
        }
        self.apply_touch_events(
            frame,
            ball,
            players,
            vertical_state,
            &touch_state.touch_events,
        );
        self.credit_ball_movement(
            frame,
            ball,
            players,
            possession_state,
            fifty_fifty_state,
            live_play,
        );
        self.previous_ball_velocity = ball.velocity();

        Ok(())
    }
}

#[cfg(test)]
#[path = "touch_tests.rs"]
mod tests;
