use super::*;

const SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 320.0;
const HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD: f32 = 900.0;
const AERIAL_TOUCH_MIN_PLAYER_Z: f32 = AIR_DRIBBLE_MIN_PLAYER_Z;

/// How long after a touch we keep watching the toucher's dodge component before
/// giving up on associating a flip with it. The `CarComponent_Dodge`
/// `ReplicatedActive` byte routinely replicates *after* the ball-hit it produced
/// (the hit and the dodge-activation land on adjacent frames), so a flip-into-ball
/// contact can be sampled on the one frame where the dodge flag has not yet
/// flipped on. Re-checking for a brief window lets such touches be recognized as
/// dodge contacts after the fact. Shared with the flick detector and flip-reset
/// confirmation via [`DODGE_ACTIVE_BYTE_LAG_TOLERANCE_SECONDS`].
const DODGE_LAG_TOLERANCE_SECONDS: f32 = DODGE_ACTIVE_BYTE_LAG_TOLERANCE_SECONDS;

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

/// A classified ball touch with strength kind, surface/height context, and an inferred intention.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchClassificationEvent {
    /// Identity of the source [`TouchEvent`](crate::TouchEvent) this
    /// classification was derived from. Join on this instead of player + frame.
    /// `None` only for data serialized before touch ids existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number")]
    pub touch_id: Option<u64>,
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    // Ball position (uu) at the touch's sample frame: the actual point of contact
    // on the ball's trajectory, unlike `player_position` (the car centre, up to a
    // hitbox+ball-radius away). Diagrams placing a touch on the ball's path prefer
    // this. Non-doc comment so ts-rs keeps the binding in sync with `player_position`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub kind: String,
    pub height_band: String,
    pub surface: String,
    pub dodge_state: String,
    pub intention: String,
    #[serde(default)]
    pub first_touch: bool,
    #[serde(default)]
    pub contested: bool,
    #[serde(default)]
    pub role: RoleState,
    #[serde(default)]
    pub play_depth: PlayDepthState,
    pub ball_speed_change: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball_movement: Option<TouchBallMovement>,
}

/// Ball movement produced by a touch.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchBallMovement {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    pub travel_distance: f32,
    pub advance_distance: f32,
    pub retreat_distance: f32,
    pub finalized: bool,
}

impl TouchBallMovement {
    fn absorb_delta(&mut self, event: Self) {
        self.end_time = event.end_time;
        self.end_frame = event.end_frame;
        self.duration += event.duration;
        self.travel_distance += event.travel_distance;
        self.advance_distance += event.advance_distance;
        self.retreat_distance += event.retreat_distance;
    }

    fn finalized(mut self) -> Self {
        self.finalized = true;
        self
    }
}

/// A `no_dodge` touch still within [`DODGE_LAG_TOLERANCE_SECONDS`] of being
/// retroactively upgraded to a dodge contact, should the toucher's dodge
/// component go active in the frames immediately following the hit.
#[derive(Debug, Clone, PartialEq)]
struct PendingDodgeUpgrade {
    touch_index: usize,
    player_id: PlayerId,
    touch_time: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PendingFiftyFiftyMovement {
    start_frame: usize,
    travel_distance: f32,
    y_delta: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct PendingTouchBallMovementCredit {
    touch_index: usize,
    movement: TouchBallMovement,
}

impl InFlightItem for PendingTouchBallMovementCredit {
    fn recognition(&self) -> Recognition {
        // A touch credit always corresponds to a real touch, so it is committed
        // from the moment it is armed.
        Recognition::committed(self.movement.start_time, self.movement.start_frame)
    }

    fn on_boundary(&mut self, boundary: Boundary) -> Disposition {
        // The credit window always closes at a boundary: the accumulated travel
        // up to the goal / stoppage / end of replay is exactly what we keep.
        Disposition::Finalize(FinalizeReason::Boundary(boundary))
    }
}

/// Classifies ball touches into typed touch events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchCalculator {
    events: EventStream<TouchClassificationEvent>,
    ball_movement: InFlightLedger<PendingTouchBallMovementCredit>,
    active_touch_index_by_player: HashMap<PlayerId, usize>,
    previous_ball_velocity: Option<glam::Vec3>,
    previous_ball_position: Option<glam::Vec3>,
    pending_fifty_fifty_movement: Option<PendingFiftyFiftyMovement>,
    pending_dodge_upgrades: Vec<PendingDodgeUpgrade>,
    intention_classifier: TouchIntentionClassifier,
    control_follow: ControlFollowTracker,
}

impl TouchCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[TouchClassificationEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[TouchClassificationEvent] {
        self.events.new_events()
    }

    pub fn flush_pending_ball_movement_credit(&mut self) {
        let finalized = self.ball_movement.finalize_all(FinalizeReason::Completed);
        self.write_finalized_ball_movement(finalized);
    }

    /// Finalize any pending ball-movement credit at end of stream. Routed
    /// through the ledger so the boundary is handled uniformly and can't be
    /// forgotten.
    pub fn finish(&mut self) {
        let finalized = self.ball_movement.finish();
        self.write_finalized_ball_movement(finalized);
        let resolution = self.control_follow.flush();
        self.apply_control_resolution(resolution);
    }

    fn finalize_ball_movement_at_boundary(&mut self, boundary: Boundary) {
        let finalized = self.ball_movement.apply_boundary(boundary);
        self.write_finalized_ball_movement(finalized);
    }

    fn write_finalized_ball_movement(
        &mut self,
        finalized: Vec<(PendingTouchBallMovementCredit, FinalizeReason)>,
    ) {
        for (pending, _reason) in finalized {
            if let Some(event) = self.events.get_mut(pending.touch_index) {
                event.ball_movement = Some(pending.movement.finalized());
            }
        }
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

    fn teammate_positions(
        players: &PlayerFrameState,
        player_id: &PlayerId,
        is_team_0: bool,
    ) -> Vec<glam::Vec3> {
        players
            .players
            .iter()
            .filter(|player| player.is_team_0 == is_team_0 && &player.player_id != player_id)
            .filter_map(PlayerSample::position)
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        rotation: &RotationCalculator,
        touch_events: &[TouchEvent],
        fifty_fifty_state: &FiftyFiftyState,
    ) {
        let ball_speed_change = Self::ball_speed_change(frame, ball, self.previous_ball_velocity);
        let contested_frame = touch_events.iter().any(|touch| touch.team_is_team_0)
            && touch_events.iter().any(|touch| !touch.team_is_team_0);

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
            let (role, play_depth) = rotation.current_role_and_depth(player_id);
            let classification = Self::classify_touch(
                height_band,
                surface,
                dodge_state,
                ball_speed_change,
                controlled_touch_kind,
            );
            let contested = contested_frame
                || fifty_fifty_state
                    .active_event
                    .as_ref()
                    .is_some_and(|active| {
                        fifty_fifty_involves_player(active, player_id, touch_event.team_is_team_0)
                    });
            let teammate_positions =
                Self::teammate_positions(players, player_id, touch_event.team_is_team_0);
            let control_resolution = self
                .control_follow
                .observe_touch(player_id, touch_event.time);
            self.apply_control_resolution(control_resolution);
            let resolution = self.intention_classifier.classify(
                touch_event,
                player_id,
                &TouchIntentionFrameContext {
                    ball_position: ball.position(),
                    ball_velocity: ball.velocity(),
                    previous_ball_position: self.previous_ball_position,
                    previous_ball_velocity: self.previous_ball_velocity,
                    teammate_positions: &teammate_positions,
                    contested,
                },
            );
            let event = TouchClassificationEvent {
                touch_id: touch_event.touch_id,
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
                ball_position: ball.position().map(|position| position.to_array()),
                is_team_0: touch_event.team_is_team_0,
                kind: classification.kind.as_label_value().to_owned(),
                height_band: classification.height_band.as_label().value.to_owned(),
                surface: classification.surface.as_label_value().to_owned(),
                dodge_state: classification.dodge_state.as_label_value().to_owned(),
                intention: resolution.intention.as_label_value().to_owned(),
                first_touch: resolution.first_touch,
                contested: resolution.contested,
                role,
                play_depth,
                ball_speed_change,
                ball_movement: None,
            };
            let touch_index = self.events.len();
            self.events.push(event);
            self.active_touch_index_by_player
                .insert(player_id.clone(), touch_index);
            // The dodge byte often lags the hit by a frame or two; if this touch
            // looked dodge-less, keep watching the toucher's dodge component so a
            // flip-into-ball contact still gets recognized once it activates.
            if matches!(classification.dodge_state, TouchDodgeState::NoDodge) {
                self.pending_dodge_upgrades.push(PendingDodgeUpgrade {
                    touch_index,
                    player_id: player_id.clone(),
                    touch_time: touch_event.time,
                });
            }
            if matches!(
                resolution.intention,
                TouchIntention::Pass | TouchIntention::Neutral
            ) {
                self.control_follow
                    .open(touch_index, player_id, touch_event.time);
            }
        }
    }

    /// Apply a closed control-follow window: upgrade the touch's intention to
    /// control when the toucher stayed with the ball or earned the follow-up.
    fn apply_control_resolution(&mut self, resolution: Option<ControlResolution>) {
        let Some(resolution) = resolution else {
            return;
        };
        if !resolution.control {
            return;
        }
        if let Some(event) = self.events.get_mut(resolution.touch_index) {
            event.intention = TouchIntention::Control.as_label_value().to_owned();
        }
    }

    /// Re-examine recent `no_dodge` touches against this frame's dodge state.
    /// The dodge component's active byte frequently replicates a frame or two
    /// after the ball-hit it caused, so a flip-into-ball contact can be sampled
    /// on the one frame where the flag has not yet flipped on. Any pending touch
    /// whose toucher is now dodging within [`DODGE_LAG_TOLERANCE_SECONDS`] is
    /// upgraded to a dodge contact; entries that age out are dropped.
    fn advance_dodge_upgrades(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        if self.pending_dodge_upgrades.is_empty() {
            return;
        }
        let mut resolved = Vec::new();
        self.pending_dodge_upgrades.retain(|pending| {
            if frame.time - pending.touch_time > DODGE_LAG_TOLERANCE_SECONDS {
                return false;
            }
            if Self::player_dodge_active(players, &pending.player_id) {
                resolved.push(pending.touch_index);
                return false;
            }
            true
        });
        for touch_index in resolved {
            if let Some(event) = self.events.get_mut(touch_index) {
                event.dodge_state = TouchDodgeState::Dodge.as_label_value().to_owned();
            }
        }
    }

    /// Feed this frame's ball and toucher samples to any open control-follow
    /// window and apply its resolution once it ages out.
    fn advance_control_follow(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
    ) {
        let Some(window_player) = self.control_follow.window_player().cloned() else {
            return;
        };
        let player_sample = players
            .players
            .iter()
            .find(|player| player.player_id == window_player);
        let resolution = self.control_follow.advance(
            frame,
            ball.position(),
            ball.velocity(),
            player_sample.and_then(PlayerSample::position),
            player_sample.and_then(PlayerSample::velocity),
        );
        self.apply_control_resolution(resolution);
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
        let movement = TouchBallMovement {
            start_time: time,
            start_frame: frame,
            end_time: time,
            end_frame: frame,
            duration,
            travel_distance,
            advance_distance,
            retreat_distance,
            finalized: false,
        };
        self.record_ball_movement_credit(player_id, player_position, movement);
    }

    fn record_ball_movement_credit(
        &mut self,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        movement: TouchBallMovement,
    ) {
        let Some(&touch_index) = self.active_touch_index_by_player.get(player_id) else {
            return;
        };
        let pending_index = self
            .ball_movement
            .in_flight()
            .first()
            .map(|pending| pending.touch_index);

        if pending_index == Some(touch_index) {
            // Same touch still in flight: fold this frame's travel into it.
            let merged = {
                let pending = self
                    .ball_movement
                    .in_flight_mut()
                    .first_mut()
                    .expect("pending credit present");
                pending.movement.absorb_delta(movement);
                pending.movement.clone()
            };
            if let Some(event) = self.events.get_mut(touch_index) {
                event.player_position = player_position.or(event.player_position);
                event.ball_movement = Some(merged);
            }
            return;
        }

        // A different touch (or none) is in flight: supersede the old credit and
        // arm a fresh one for this touch.
        if pending_index.is_some() {
            let finalized = self.ball_movement.finalize_all(FinalizeReason::Superseded);
            self.write_finalized_ball_movement(finalized);
        }
        if let Some(event) = self.events.get_mut(touch_index) {
            event.player_position = player_position.or(event.player_position);
            event.ball_movement = Some(movement.clone());
        }
        self.ball_movement.arm(PendingTouchBallMovementCredit {
            touch_index,
            movement,
        });
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
        let movement = TouchBallMovement {
            start_time: event.resolve_time,
            start_frame: event.resolve_frame,
            end_time: event.resolve_time,
            end_frame: event.resolve_frame,
            duration: 0.0,
            travel_distance: pending.travel_distance,
            advance_distance,
            retreat_distance,
            finalized: false,
        };
        self.flush_pending_ball_movement_credit();
        self.record_ball_movement_credit(
            player_id,
            if team_is_team_0 {
                Some(event.team_zero_position)
            } else {
                Some(event.team_one_position)
            },
            movement,
        );
        self.flush_pending_ball_movement_credit();
    }

    fn credit_ball_movement(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        possession_state: &PossessionState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play_state: &LivePlayState,
    ) {
        let current_ball_position = ball.position();
        if !live_play_state.is_live_play {
            self.finalize_ball_movement_at_boundary(Boundary::LivePlayEnded);
            self.previous_ball_position = current_ball_position;
            self.pending_fifty_fifty_movement = None;
            return;
        }

        let Some(current_ball_position) = current_ball_position else {
            self.flush_pending_ball_movement_credit();
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
            self.flush_pending_ball_movement_credit();
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
            self.flush_pending_ball_movement_credit();
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
        rotation: &RotationCalculator,
        touch_state: &TouchState,
        possession_state: &PossessionState,
        fifty_fifty_state: &FiftyFiftyState,
        events_state: &FrameEventsState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.finalize_ball_movement_at_boundary(Boundary::LivePlayEnded);
            self.previous_ball_velocity = ball.velocity();
            self.previous_ball_position = ball.position();
            self.pending_fifty_fifty_movement = None;
            self.pending_dodge_upgrades.clear();
            self.intention_classifier.reset();
            let resolution = self.control_follow.flush();
            self.apply_control_resolution(resolution);
            return Ok(());
        }
        self.intention_classifier
            .begin_frame(frame, &events_state.player_stat_events);
        self.apply_touch_events(
            frame,
            ball,
            players,
            vertical_state,
            rotation,
            &touch_state.touch_events,
            fifty_fifty_state,
        );
        self.advance_dodge_upgrades(frame, players);
        self.advance_control_follow(frame, ball, players);
        self.credit_ball_movement(
            frame,
            ball,
            players,
            possession_state,
            fifty_fifty_state,
            live_play_state,
        );
        self.previous_ball_velocity = ball.velocity();

        Ok(())
    }
}

#[cfg(test)]
#[path = "touch_tests.rs"]
mod tests;
