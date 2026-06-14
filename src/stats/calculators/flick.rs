use super::*;

const FLICK_MAX_DODGE_TO_TOUCH_SECONDS: f32 = 0.32;
const FLICK_MAX_CONTROL_TO_DODGE_SECONDS: f32 = 0.08;
const FLICK_MAX_SETUP_STALE_SECONDS: f32 = 0.35;
const FLICK_DODGE_LAG_TOLERANCE_SECONDS: f32 = 0.12;
const FLICK_MIN_PENDING_DODGE_SETUP_SECONDS: f32 = 0.10;
const FLICK_MIN_SETUP_SECONDS: f32 = 0.20;
const FLICK_MIN_BALL_SPEED_CHANGE: f32 = 325.0;
const FLICK_MIN_CONFIDENCE: f32 = 0.55;
const FLICK_MAX_CONTROL_BALL_Z: f32 = 700.0;
const FLICK_MAX_CONTROL_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 1.7;
const FLICK_MIN_CONTROL_VERTICAL_GAP: f32 = 35.0;
const FLICK_MAX_CONTROL_VERTICAL_GAP: f32 = 280.0;
const FLICK_MIN_LOCAL_Z: f32 = 20.0;
const FLICK_MAX_LOCAL_X_BEHIND: f32 = 95.0;
const FLICK_MAX_LOCAL_X_FRONT: f32 = 210.0;
const FLICK_MAX_LOCAL_Y: f32 = 170.0;
const FLICK_MIN_IMPULSE_AWAY_ALIGNMENT: f32 = 0.15;
const REVERSE_FLICK_MIN_BACKFLIP_PITCH_RATE: f32 = 2.5;
const REVERSE_FLICK_MIN_FORWARD_IMPULSE: f32 = 450.0;
const REVERSE_FLICK_MIN_FORWARD_IMPULSE_ALIGNMENT: f32 = 0.55;
const REVERSE_FLICK_MIN_ROTATION_UNDER_BALL_DEGREES: f32 = 15.0;

/// The kind of flick detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlickKind {
    Other,
    Reverse,
}

/// Rotation direction of the car during the flick setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlickSetupRotationDirection {
    Unknown,
    Left,
    Right,
}

pub(crate) const FLICK_KIND_LABELS: [StatLabel; 2] = [
    StatLabel::new("kind", "other"),
    StatLabel::new("kind", "reverse"),
];

pub(crate) const FLICK_SETUP_ROTATION_DIRECTION_LABELS: [StatLabel; 3] = [
    StatLabel::new("setup_rotation_direction", "unknown"),
    StatLabel::new("setup_rotation_direction", "left"),
    StatLabel::new("setup_rotation_direction", "right"),
];

impl FlickKind {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Other => "other",
            Self::Reverse => "reverse",
        }
    }

    pub fn as_label(self) -> StatLabel {
        flick_kind_label(self.as_label_value())
    }
}

impl FlickSetupRotationDirection {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

pub(crate) fn flick_kind_label(value: &str) -> StatLabel {
    match value {
        "reverse" => StatLabel::new("kind", "reverse"),
        _ => StatLabel::new("kind", "other"),
    }
}

pub(crate) fn flick_setup_rotation_direction_label(value: &str) -> StatLabel {
    match value {
        "left" => StatLabel::new("setup_rotation_direction", "left"),
        "right" => StatLabel::new("setup_rotation_direction", "right"),
        _ => StatLabel::new("setup_rotation_direction", "unknown"),
    }
}

/// A dodge-powered touch following a short controlled carry setup.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FlickEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub dodge_time: f32,
    pub dodge_frame: usize,
    pub time_since_dodge: f32,
    pub setup_start_time: f32,
    pub setup_start_frame: usize,
    pub setup_duration: f32,
    pub setup_touch_count: u32,
    pub average_horizontal_gap: f32,
    pub average_vertical_gap: f32,
    pub ball_speed_change: f32,
    pub ball_impulse: [f32; 3],
    pub impulse_away_alignment: f32,
    pub vertical_impulse: f32,
    pub kind: String,
    pub local_ball_position: [f32; 3],
    pub local_ball_impulse: [f32; 3],
    pub backflip_pitch_rate: f32,
    pub rotation_under_ball_degrees: f32,
    pub setup_rotation_degrees: f32,
    pub setup_rotation_direction: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct FlickControlObservation {
    horizontal_gap: f32,
    vertical_gap: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveFlickSetup {
    is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    last_time: f32,
    last_frame: usize,
    duration: f32,
    horizontal_gap_integral: f32,
    vertical_gap_integral: f32,
    touch_count: u32,
    start_forward: Option<glam::Vec3>,
    max_horizontal_rotation_degrees: f32,
    signed_horizontal_rotation_degrees: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct FlickSetupSummary {
    is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    last_time: f32,
    last_frame: usize,
    duration: f32,
    average_horizontal_gap: f32,
    average_vertical_gap: f32,
    touch_count: u32,
    rotation_under_ball_degrees: f32,
    setup_rotation_degrees: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct RecentDodgeStart {
    time: f32,
    frame: usize,
    setup: FlickSetupSummary,
    rotation_at_dodge: Option<glam::Quat>,
}

#[derive(Debug, Clone)]
struct PendingFlickTouch {
    touch_event: TouchEvent,
    ball: BallFrameState,
    player: PlayerSample,
    dodge_start: RecentDodgeStart,
    ball_impulse: glam::Vec3,
}

impl PartialEq for PendingFlickTouch {
    fn eq(&self, other: &Self) -> bool {
        self.touch_event.touch_id == other.touch_event.touch_id
            && self.touch_event.player == other.touch_event.player
            && self.touch_event.frame == other.touch_event.frame
    }
}

/// Detects flicks from ball/player state and touches.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlickCalculator {
    events: EventStream<FlickEvent>,
    active_setups: HashMap<PlayerId, ActiveFlickSetup>,
    recent_setups: HashMap<PlayerId, FlickSetupSummary>,
    recent_dodge_starts: HashMap<PlayerId, RecentDodgeStart>,
    pending_dodge_touches: Vec<PendingFlickTouch>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    previous_ball_velocity: Option<glam::Vec3>,
}

impl FlickCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[FlickEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[FlickEvent] {
        self.events.new_events()
    }

    fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }

        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    fn ball_impulse(
        frame: &FrameInfo,
        ball: &BallFrameState,
        previous_ball_velocity: Option<glam::Vec3>,
    ) -> glam::Vec3 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = ball.sample() else {
            return glam::Vec3::ZERO;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return glam::Vec3::ZERO;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * frame.dt.max(0.0));
        ball.velocity() - previous_ball_velocity - expected_linear_delta
    }

    fn control_observation(
        ball: &BallSample,
        player: &PlayerSample,
        controlling_player: Option<&PlayerId>,
    ) -> Option<FlickControlObservation> {
        if controlling_player != Some(&player.player_id) {
            return None;
        }

        let player_rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let ball_position = ball.position();
        if !(BALL_CARRY_MIN_BALL_Z..=FLICK_MAX_CONTROL_BALL_Z).contains(&ball_position.z) {
            return None;
        }

        let horizontal_gap = player_position
            .truncate()
            .distance(ball_position.truncate());
        if horizontal_gap > FLICK_MAX_CONTROL_HORIZONTAL_GAP {
            return None;
        }

        let vertical_gap = ball_position.z - player_position.z;
        if !(FLICK_MIN_CONTROL_VERTICAL_GAP..=FLICK_MAX_CONTROL_VERTICAL_GAP)
            .contains(&vertical_gap)
        {
            return None;
        }

        let local_ball_position =
            quat_to_glam(&player_rigid_body.rotation).inverse() * (ball_position - player_position);
        if local_ball_position.x < -FLICK_MAX_LOCAL_X_BEHIND
            || local_ball_position.x > FLICK_MAX_LOCAL_X_FRONT
            || local_ball_position.y.abs() > FLICK_MAX_LOCAL_Y
            || local_ball_position.z < FLICK_MIN_LOCAL_Z
        {
            return None;
        }

        Some(FlickControlObservation {
            horizontal_gap,
            vertical_gap,
        })
    }

    fn setup_summary(setup: &ActiveFlickSetup) -> FlickSetupSummary {
        FlickSetupSummary {
            is_team_0: setup.is_team_0,
            start_time: setup.start_time,
            start_frame: setup.start_frame,
            last_time: setup.last_time,
            last_frame: setup.last_frame,
            duration: setup.duration,
            average_horizontal_gap: setup.horizontal_gap_integral
                / setup.duration.max(f32::EPSILON),
            average_vertical_gap: setup.vertical_gap_integral / setup.duration.max(f32::EPSILON),
            touch_count: setup.touch_count,
            rotation_under_ball_degrees: setup.max_horizontal_rotation_degrees,
            setup_rotation_degrees: setup.signed_horizontal_rotation_degrees,
        }
    }

    fn setup_qualifies(setup: &FlickSetupSummary) -> bool {
        setup.duration >= FLICK_MIN_SETUP_SECONDS
    }

    fn classify_kind(
        player_rotation: glam::Quat,
        player_angular_velocity: glam::Vec3,
        rotation_at_dodge: Option<glam::Quat>,
        rotation_under_ball_degrees: f32,
        relative_ball_position: glam::Vec3,
        ball_impulse: glam::Vec3,
    ) -> (FlickKind, glam::Vec3, glam::Vec3, f32) {
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        let impulse_reference_rotation = rotation_at_dodge.unwrap_or(player_rotation);
        let local_ball_impulse = impulse_reference_rotation.inverse() * ball_impulse;
        let local_angular_velocity = player_rotation.inverse() * player_angular_velocity;
        let backflip_pitch_rate = (-local_angular_velocity.y).max(0.0);
        let forward_impulse_alignment = ball_impulse
            .normalize_or_zero()
            .dot(impulse_reference_rotation * glam::Vec3::X);
        let kind = if backflip_pitch_rate >= REVERSE_FLICK_MIN_BACKFLIP_PITCH_RATE
            && local_ball_impulse.x >= REVERSE_FLICK_MIN_FORWARD_IMPULSE
            && forward_impulse_alignment >= REVERSE_FLICK_MIN_FORWARD_IMPULSE_ALIGNMENT
            && rotation_under_ball_degrees >= REVERSE_FLICK_MIN_ROTATION_UNDER_BALL_DEGREES
        {
            FlickKind::Reverse
        } else {
            FlickKind::Other
        };

        (
            kind,
            local_ball_position,
            local_ball_impulse,
            backflip_pitch_rate,
        )
    }

    fn signed_horizontal_rotation_degrees(
        start_forward: Option<glam::Vec3>,
        current_forward: Option<glam::Vec3>,
    ) -> Option<f32> {
        let start = start_forward?.truncate().normalize_or_zero();
        let current = current_forward?.truncate().normalize_or_zero();
        if start.length_squared() <= f32::EPSILON || current.length_squared() <= f32::EPSILON {
            return None;
        }

        let cross_z = start.x * current.y - start.y * current.x;
        Some(cross_z.atan2(start.dot(current)).to_degrees())
    }

    fn setup_rotation_direction(signed_degrees: f32) -> FlickSetupRotationDirection {
        if signed_degrees.abs() < REVERSE_FLICK_MIN_ROTATION_UNDER_BALL_DEGREES {
            FlickSetupRotationDirection::Unknown
        } else if signed_degrees > 0.0 {
            FlickSetupRotationDirection::Right
        } else {
            FlickSetupRotationDirection::Left
        }
    }

    fn store_recent_setup(&mut self, player_id: PlayerId, setup: FlickSetupSummary) {
        if Self::setup_qualifies(&setup) {
            self.recent_setups.insert(player_id, setup);
        }
    }

    fn finish_setup(&mut self, player_id: &PlayerId) {
        let Some(setup) = self.active_setups.remove(player_id) else {
            return;
        };
        self.store_recent_setup(player_id.clone(), Self::setup_summary(&setup));
    }

    fn recent_setup_for_player(
        &self,
        player_id: &PlayerId,
        current_time: f32,
    ) -> Option<FlickSetupSummary> {
        if let Some(active) = self.active_setups.get(player_id) {
            return Some(Self::setup_summary(active));
        }

        self.recent_setups
            .get(player_id)
            .filter(|setup| current_time - setup.last_time <= FLICK_MAX_SETUP_STALE_SECONDS)
            .cloned()
    }

    fn update_control_setups(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
        controlling_player: Option<&PlayerId>,
    ) {
        let Some(ball) = ball.sample() else {
            let player_ids: Vec<_> = self.active_setups.keys().cloned().collect();
            for player_id in player_ids {
                self.finish_setup(&player_id);
            }
            return;
        };

        let mut observed_players = HashSet::new();
        for player in &players.players {
            let Some(observation) = Self::control_observation(ball, player, controlling_player)
            else {
                continue;
            };
            observed_players.insert(player.player_id.clone());
            let current_forward = player
                .rigid_body
                .as_ref()
                .map(|rigid_body| quat_to_glam(&rigid_body.rotation) * glam::Vec3::X);
            let setup = self
                .active_setups
                .entry(player.player_id.clone())
                .or_insert_with(|| ActiveFlickSetup {
                    is_team_0: player.is_team_0,
                    start_time: (frame.time - frame.dt).max(0.0),
                    start_frame: frame.frame_number.saturating_sub(1),
                    last_time: frame.time,
                    last_frame: frame.frame_number,
                    duration: frame.dt.max(0.0),
                    horizontal_gap_integral: observation.horizontal_gap * frame.dt.max(0.0),
                    vertical_gap_integral: observation.vertical_gap * frame.dt.max(0.0),
                    touch_count: 0,
                    start_forward: current_forward,
                    max_horizontal_rotation_degrees: 0.0,
                    signed_horizontal_rotation_degrees: 0.0,
                });

            if setup.last_frame != frame.frame_number {
                setup.last_time = frame.time;
                setup.last_frame = frame.frame_number;
                setup.duration += frame.dt.max(0.0);
                setup.horizontal_gap_integral += observation.horizontal_gap * frame.dt.max(0.0);
                setup.vertical_gap_integral += observation.vertical_gap * frame.dt.max(0.0);
                if let Some(signed_degrees) =
                    Self::signed_horizontal_rotation_degrees(setup.start_forward, current_forward)
                {
                    let degrees = signed_degrees.abs();
                    if degrees > setup.max_horizontal_rotation_degrees {
                        setup.max_horizontal_rotation_degrees = degrees;
                        setup.signed_horizontal_rotation_degrees = signed_degrees;
                    }
                }
            }
        }

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            if let Some(setup) = self.active_setups.get_mut(player_id) {
                setup.touch_count += 1;
            }
        }

        let active_ids: Vec<_> = self.active_setups.keys().cloned().collect();
        for player_id in active_ids {
            if !observed_players.contains(&player_id) {
                self.finish_setup(&player_id);
            }
        }
    }

    fn track_dodge_starts(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if !player.dodge_active || was_dodge_active {
                continue;
            }

            let Some(setup) = self.recent_setup_for_player(&player.player_id, frame.time) else {
                continue;
            };
            if !Self::setup_qualifies(&setup) {
                continue;
            }
            if frame.time - setup.last_time > FLICK_MAX_CONTROL_TO_DODGE_SECONDS {
                continue;
            }

            self.recent_dodge_starts.insert(
                player.player_id.clone(),
                RecentDodgeStart {
                    time: frame.time,
                    frame: frame.frame_number,
                    setup,
                    rotation_at_dodge: player
                        .rigid_body
                        .as_ref()
                        .map(|rigid_body| quat_to_glam(&rigid_body.rotation)),
                },
            );
        }
    }

    fn prune_recent_state(&mut self, current_time: f32) {
        self.recent_setups
            .retain(|_, setup| current_time - setup.last_time <= FLICK_MAX_SETUP_STALE_SECONDS);
        self.recent_dodge_starts
            .retain(|_, dodge| current_time - dodge.time <= FLICK_MAX_DODGE_TO_TOUCH_SECONDS);
    }

    fn candidate_event(
        &self,
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        dodge_start: &RecentDodgeStart,
        ball_impulse: glam::Vec3,
    ) -> Option<FlickEvent> {
        let ball = ball.sample()?;
        let player_rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let time_since_dodge = touch_event.time - dodge_start.time;
        if !(0.0..=FLICK_MAX_DODGE_TO_TOUCH_SECONDS).contains(&time_since_dodge) {
            return None;
        }

        let ball_speed_change = ball_impulse.length();
        if ball_speed_change < FLICK_MIN_BALL_SPEED_CHANGE {
            return None;
        }

        let to_ball = (ball.position() - player_position).normalize_or_zero();
        let impulse_direction = ball_impulse.normalize_or_zero();
        if to_ball.length_squared() <= f32::EPSILON
            || impulse_direction.length_squared() <= f32::EPSILON
        {
            return None;
        }

        let impulse_away_alignment = impulse_direction.dot(to_ball);
        if impulse_away_alignment < FLICK_MIN_IMPULSE_AWAY_ALIGNMENT {
            return None;
        }

        let vertical_impulse = ball_impulse.z.max(0.0);
        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let player_angular_velocity = player_rigid_body
            .angular_velocity
            .as_ref()
            .map(vec_to_glam)
            .unwrap_or(glam::Vec3::ZERO);
        let (kind, local_ball_position, local_ball_impulse, backflip_pitch_rate) =
            Self::classify_kind(
                player_rotation,
                player_angular_velocity,
                dodge_start.rotation_at_dodge,
                dodge_start.setup.rotation_under_ball_degrees,
                ball.position() - player_position,
                ball_impulse,
            );
        let setup = &dodge_start.setup;
        let setup_rotation_direction = Self::setup_rotation_direction(setup.setup_rotation_degrees);
        let timing_score =
            1.0 - (time_since_dodge / FLICK_MAX_DODGE_TO_TOUCH_SECONDS).clamp(0.0, 1.0);
        let setup_duration_score =
            Self::normalize_score(setup.duration, FLICK_MIN_SETUP_SECONDS, 0.75);
        let horizontal_control_score =
            1.0 - (setup.average_horizontal_gap / FLICK_MAX_CONTROL_HORIZONTAL_GAP).clamp(0.0, 1.0);
        let vertical_control_score = 1.0
            - ((setup.average_vertical_gap - 110.0).abs() / FLICK_MAX_CONTROL_VERTICAL_GAP)
                .clamp(0.0, 1.0);
        let impulse_score =
            Self::normalize_score(ball_speed_change, FLICK_MIN_BALL_SPEED_CHANGE, 1450.0);
        let away_score = Self::normalize_score(
            impulse_away_alignment,
            FLICK_MIN_IMPULSE_AWAY_ALIGNMENT,
            0.85,
        );
        let vertical_score = Self::normalize_score(vertical_impulse, 100.0, 750.0);

        let confidence = 0.16 * timing_score
            + 0.19 * setup_duration_score
            + 0.12 * horizontal_control_score
            + 0.10 * vertical_control_score
            + 0.22 * impulse_score
            + 0.15 * away_score
            + 0.06 * vertical_score;
        if confidence < FLICK_MIN_CONFIDENCE {
            return None;
        }

        Some(FlickEvent {
            time: touch_event.time,
            frame: touch_event.frame,
            sample_time: touch_event.time,
            sample_frame: touch_event.frame,
            player: player.player_id.clone(),
            player_position: Some(player_position.to_array()),
            is_team_0: player.is_team_0,
            dodge_time: dodge_start.time,
            dodge_frame: dodge_start.frame,
            time_since_dodge,
            setup_start_time: setup.start_time,
            setup_start_frame: setup.start_frame,
            setup_duration: setup.duration,
            setup_touch_count: setup.touch_count,
            average_horizontal_gap: setup.average_horizontal_gap,
            average_vertical_gap: setup.average_vertical_gap,
            ball_speed_change,
            ball_impulse: ball_impulse.to_array(),
            impulse_away_alignment,
            vertical_impulse,
            kind: kind.as_label_value().to_owned(),
            local_ball_position: local_ball_position.to_array(),
            local_ball_impulse: local_ball_impulse.to_array(),
            backflip_pitch_rate,
            rotation_under_ball_degrees: setup.rotation_under_ball_degrees,
            setup_rotation_degrees: setup.setup_rotation_degrees,
            setup_rotation_direction: setup_rotation_direction.as_label_value().to_owned(),
            confidence,
        })
    }

    fn apply_event(&mut self, frame: &FrameInfo, mut event: FlickEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        self.events.push(event);
    }

    fn dodge_start_for_touch(&self, player: &PlayerSample) -> Option<RecentDodgeStart> {
        if let Some(dodge_start) = self.recent_dodge_starts.get(&player.player_id) {
            return Some(dodge_start.clone());
        }
        None
    }

    fn classified_as_dodge_touch(
        touch_event: &TouchEvent,
        touch_classification_events: &[TouchClassificationEvent],
    ) -> bool {
        let Some(touch_player) = touch_event.player.as_ref() else {
            return false;
        };
        touch_classification_events.iter().any(|event| {
            let same_touch = match (event.touch_id, touch_event.touch_id) {
                (Some(event_id), Some(touch_id)) => event_id == touch_id,
                _ => event.player == *touch_player && event.frame == touch_event.frame,
            };
            same_touch && event.dodge_state == "dodge"
        })
    }

    fn pending_dodge_start_for_touch(
        &self,
        player: &PlayerSample,
        touch_event: &TouchEvent,
    ) -> Option<RecentDodgeStart> {
        let setup = self.recent_setup_for_player(&player.player_id, touch_event.time)?;
        if setup.duration < FLICK_MIN_PENDING_DODGE_SETUP_SECONDS {
            return None;
        }
        Some(RecentDodgeStart {
            time: touch_event.time,
            frame: touch_event.frame,
            setup,
            rotation_at_dodge: player
                .rigid_body
                .as_ref()
                .map(|rigid_body| quat_to_glam(&rigid_body.rotation)),
        })
    }

    fn store_pending_dodge_touch(
        &mut self,
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        ball_impulse: glam::Vec3,
    ) {
        if self.pending_dodge_touches.iter().any(|pending| {
            pending.touch_event.touch_id == touch_event.touch_id
                && pending.touch_event.player == touch_event.player
                && pending.touch_event.frame == touch_event.frame
        }) {
            return;
        }
        let Some(dodge_start) = self.pending_dodge_start_for_touch(player, touch_event) else {
            return;
        };
        self.pending_dodge_touches.push(PendingFlickTouch {
            touch_event: touch_event.clone(),
            ball: ball.clone(),
            player: player.clone(),
            dodge_start,
            ball_impulse,
        });
    }

    fn resolve_pending_dodge_touches(
        &mut self,
        frame: &FrameInfo,
        touch_classification_events: &[TouchClassificationEvent],
    ) {
        let mut resolved = Vec::new();
        self.pending_dodge_touches.retain(|pending| {
            if frame.time - pending.touch_event.time > FLICK_DODGE_LAG_TOLERANCE_SECONDS {
                return false;
            }
            if Self::classified_as_dodge_touch(&pending.touch_event, touch_classification_events) {
                resolved.push(pending.clone());
                return false;
            }
            true
        });
        for pending in resolved {
            if let Some(event) = self.candidate_event(
                &pending.ball,
                &pending.player,
                &pending.touch_event,
                &pending.dodge_start,
                pending.ball_impulse,
            ) {
                self.apply_event(frame, event);
            }
        }
    }

    fn apply_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
    ) {
        let ball_impulse = Self::ball_impulse(frame, ball, self.previous_ball_velocity);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let Some(player) = players
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
            else {
                continue;
            };
            let Some(dodge_start) = self.dodge_start_for_touch(player) else {
                self.store_pending_dodge_touch(ball, player, touch_event, ball_impulse);
                continue;
            };
            let Some(event) =
                self.candidate_event(ball, player, touch_event, &dodge_start, ball_impulse)
            else {
                continue;
            };

            self.apply_event(frame, event);
        }
    }

    fn reset_live_play_state(&mut self, ball: &BallFrameState) {
        self.active_setups.clear();
        self.recent_setups.clear();
        self.recent_dodge_starts.clear();
        self.pending_dodge_touches.clear();
        self.previous_dodge_active.clear();
        self.previous_ball_velocity = ball.velocity();
    }

    fn update_with_touch_classification_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        touch_classification_events: &[TouchClassificationEvent],
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.reset_live_play_state(ball);
            return Ok(());
        }
        self.prune_recent_state(frame.time);
        self.resolve_pending_dodge_touches(frame, touch_classification_events);
        self.update_control_setups(
            frame,
            ball,
            players,
            &touch_state.touch_events,
            touch_state.last_touch_player.as_ref(),
        );
        self.track_dodge_starts(frame, players);
        self.apply_touch_events(frame, ball, players, &touch_state.touch_events);
        self.previous_ball_velocity = ball.velocity();
        Ok(())
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        touch: &TouchCalculator,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.update_with_touch_classification_events(
            frame,
            ball,
            players,
            touch_state,
            touch.events(),
            live_play_state,
        )
    }
}

#[cfg(test)]
#[path = "flick_tests.rs"]
mod tests;
