use super::*;

const SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS: f32 = 1.1;
const SPEED_FLIP_EVALUATION_SECONDS: f32 = 0.32;
const SPEED_FLIP_MAX_CANDIDATE_SECONDS: f32 = 0.55;
const SPEED_FLIP_MAX_GROUND_Z: f32 = 80.0;
const SPEED_FLIP_KICKOFF_MOTION_SPEED: f32 = 100.0;
const SPEED_FLIP_MIN_ALIGNMENT: f32 = 0.72;
const SPEED_FLIP_MAX_DODGE_DELAY_AFTER_GROUND_LEAVE_SECONDS: f32 = 0.20;
const SPEED_FLIP_DODGE_ACCELERATION_SAMPLE_SECONDS: f32 = 0.18;
const SPEED_FLIP_MIN_FORWARD_DODGE_DELTA: f32 = 80.0;
const SPEED_FLIP_MIN_FORWARD_DODGE_DELTA_ALIGNMENT: f32 = 0.35;
const SPEED_FLIP_MIN_ESTIMATED_DODGE_IMPULSE_MAGNITUDE: f32 = 90.0;
const SPEED_FLIP_MIN_DIRECTIONAL_WEAK_IMPULSE_MAGNITUDE: f32 = 20.0;
const SPEED_FLIP_MIN_ESTIMATED_DODGE_FORWARD_COMPONENT: f32 = 0.35;
const SPEED_FLIP_MIN_WEAK_IMPULSE_SIDE_COMPONENT: f32 = 0.10;
const SPEED_FLIP_MIN_ESTIMATED_DODGE_SIDE_COMPONENT: f32 = 0.88;
const SPEED_FLIP_MAX_ESTIMATED_DODGE_SIDE_COMPONENT: f32 = 0.95;
const SPEED_FLIP_MAX_ESTIMATED_DODGE_UP_COMPONENT: f32 = 0.82;
const SPEED_FLIP_MIN_DIAGONAL_SCORE: f32 = 0.35;
const SPEED_FLIP_MIN_STRONG_DIAGONAL_SCORE: f32 = 0.75;
const SPEED_FLIP_MIN_UP_ROTATION_DEGREES: f32 = 90.0;
const SPEED_FLIP_MAX_CANCELLED_FORWARD_ROTATION_DEGREES: f32 = 45.0;
const SPEED_FLIP_MIN_BOOST_ALIGNMENT: f32 = 0.80;
const SPEED_FLIP_MIN_CONFIDENCE: f32 = 0.45;
const BOOST_ACCELERATION_UU_PER_SECOND_SQUARED: f32 = 991.6667;

/// A ground-started diagonal dodge/cancel acceleration pattern, primarily for kickoff speed flips.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SpeedFlipEvent {
    pub time: f32,
    pub frame: usize,
    pub resolved_time: f32,
    pub resolved_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub time_since_kickoff_start: f32,
    pub start_position: [f32; 3],
    pub end_position: [f32; 3],
    pub start_speed: f32,
    pub max_speed: f32,
    pub best_alignment: f32,
    #[serde(default)]
    pub initial_boost_alignment: f32,
    #[serde(default)]
    pub best_boost_alignment: f32,
    #[serde(default)]
    pub boost_alignment_sample_count: u32,
    #[serde(default)]
    pub dodge_delay_after_ground_leave_seconds: f32,
    pub diagonal_score: f32,
    #[serde(default)]
    pub estimated_dodge_impulse_magnitude: f32,
    #[serde(default)]
    pub estimated_dodge_impulse_forward_component: f32,
    #[serde(default)]
    pub estimated_dodge_impulse_side_component: f32,
    #[serde(default)]
    pub estimated_dodge_impulse_up_component: f32,
    pub cancel_score: f32,
    pub speed_score: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveSpeedFlipCandidate {
    is_team_0: bool,
    is_kickoff: bool,
    kickoff_start_time: Option<f32>,
    start_time: f32,
    start_frame: usize,
    start_position: [f32; 3],
    end_position: [f32; 3],
    start_velocity: glam::Vec3,
    start_velocity_xy: glam::Vec2,
    start_forward_xy: glam::Vec2,
    local_forward: glam::Vec3,
    local_right: glam::Vec3,
    local_up: glam::Vec3,
    start_speed: f32,
    max_speed: f32,
    best_alignment: f32,
    initial_boost_alignment: Option<f32>,
    best_boost_alignment: f32,
    boost_alignment_sample_count: u32,
    dodge_delay_after_ground_leave_seconds: f32,
    dodge_boost_compensation: glam::Vec3,
    best_dodge_forward_delta: f32,
    best_dodge_delta_alignment: f32,
    best_estimated_dodge_impulse_magnitude: f32,
    best_estimated_dodge_impulse_forward_component: f32,
    best_estimated_dodge_impulse_side_component: f32,
    best_estimated_dodge_impulse_up_component: f32,
    dodge_acceleration_sample_count: u32,
    best_diagonal_score: f32,
    max_forward_rotation_degrees: f32,
    max_up_rotation_degrees: f32,
    min_forward_z: f32,
    latest_forward_z: f32,
    latest_time: f32,
    latest_frame: usize,
}

impl InFlightItem for ActiveSpeedFlipCandidate {
    fn recognition(&self) -> Recognition {
        // Speculative: a candidate can be pruned (stale), discarded at a
        // boundary, or evaluate to no speed flip before emitting.
        Recognition::speculative(self.start_time, self.start_frame)
    }

    fn on_boundary(&mut self, _boundary: Boundary) -> Disposition {
        Disposition::Discard
    }
}

/// Detects speed flips from gameplay/ball/player state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpeedFlipCalculator {
    events: EventStream<SpeedFlipEvent>,
    active_candidates: KeyedInFlightLedger<PlayerId, ActiveSpeedFlipCandidate>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    last_ground_contacts: HashMap<PlayerId, f32>,
    kickoff_approach_active_last_frame: bool,
    current_kickoff_start_time: Option<f32>,
}

impl SpeedFlipCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[SpeedFlipEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[SpeedFlipEvent] {
        self.events.new_events()
    }

    fn kickoff_approach_active(gameplay: &GameplayState) -> bool {
        gameplay.ball_has_been_hit == Some(false)
    }

    fn player_by_id<'a>(
        players: &'a PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<&'a PlayerSample> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
    }

    fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }

        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    fn diagonal_score(local_angular_velocity: glam::Vec3) -> f32 {
        let pitch_rate = local_angular_velocity.y.abs();
        let side_spin = local_angular_velocity
            .x
            .abs()
            .max(local_angular_velocity.z.abs());
        if pitch_rate <= f32::EPSILON || side_spin <= f32::EPSILON {
            return 0.0;
        }

        let pitch_score = Self::normalize_score(pitch_rate, 35.0, 180.0);
        let side_score = Self::normalize_score(side_spin, 60.0, 260.0);
        let balance = pitch_rate.min(side_spin) / pitch_rate.max(side_spin);
        let balance_score = Self::normalize_score(balance, 0.18, 0.65);

        (pitch_score * side_score).sqrt() * (0.75 + 0.25 * balance_score)
    }

    fn forward_speed_alignment(player: &PlayerSample) -> Option<f32> {
        let velocity = player.velocity()?;
        let rigid_body = player.rigid_body.as_ref()?;
        let velocity_xy = velocity.truncate().normalize_or_zero();
        if velocity_xy.length_squared() <= f32::EPSILON {
            return None;
        }

        let forward_xy = (quat_to_glam(&rigid_body.rotation) * glam::Vec3::X)
            .truncate()
            .normalize_or_zero();
        if forward_xy.length_squared() <= f32::EPSILON {
            return None;
        }

        Some(forward_xy.dot(velocity_xy))
    }

    fn forward_xy(player: &PlayerSample) -> Option<glam::Vec2> {
        let rigid_body = player.rigid_body.as_ref()?;
        let forward_xy = (quat_to_glam(&rigid_body.rotation) * glam::Vec3::X)
            .truncate()
            .normalize_or_zero();
        (forward_xy.length_squared() > f32::EPSILON).then_some(forward_xy)
    }

    fn boost_alignment(player: &PlayerSample) -> Option<f32> {
        player
            .boost_active
            .then(|| Self::forward_speed_alignment(player))
            .flatten()
    }

    fn update_ground_contacts(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            if player
                .position()
                .is_some_and(|position| position.z <= PLAYER_GROUND_Z_THRESHOLD)
            {
                self.last_ground_contacts
                    .insert(player.player_id.clone(), frame.time);
            }
        }

        self.last_ground_contacts
            .retain(|_, ground_contact_time| frame.time - *ground_contact_time <= 2.0);
    }

    fn candidate_alignment(
        _ball: &BallFrameState,
        player: &PlayerSample,
        _is_kickoff: bool,
    ) -> Option<f32> {
        Self::forward_speed_alignment(player)
    }

    fn apply_event(&mut self, event: SpeedFlipEvent) {
        self.events.push(event);
    }

    fn reset_kickoff_state(&mut self) {
        self.active_candidates.clear();
        self.current_kickoff_start_time = None;
    }

    fn kickoff_motion_started(players: &PlayerFrameState) -> bool {
        players.players.iter().any(|player| {
            player.dodge_active
                || player
                    .speed()
                    .is_some_and(|speed| speed >= SPEED_FLIP_KICKOFF_MOTION_SPEED)
        })
    }

    fn update_kickoff_start_time(
        &mut self,
        frame: &FrameInfo,
        kickoff_approach_active: bool,
        players: &PlayerFrameState,
    ) {
        if !kickoff_approach_active {
            self.current_kickoff_start_time = None;
            return;
        }

        if self.current_kickoff_start_time.is_none() && Self::kickoff_motion_started(players) {
            self.current_kickoff_start_time = Some(frame.time);
        }
    }

    fn maybe_start_candidate(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        player: &PlayerSample,
        _live_play_state: &LivePlayState,
    ) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        let is_kickoff = Self::kickoff_approach_active(gameplay);
        let kickoff_start_time = if is_kickoff {
            let Some(kickoff_start_time) = self.current_kickoff_start_time else {
                return;
            };
            if frame.time - kickoff_start_time > SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS {
                return;
            }
            Some(kickoff_start_time)
        } else {
            None
        };

        let Some(rigid_body) = player.rigid_body.as_ref() else {
            return;
        };
        let Some(player_position) = player.position() else {
            return;
        };
        if player_position.z > SPEED_FLIP_MAX_GROUND_Z {
            return;
        }

        let start_speed = player.speed().unwrap_or(0.0);
        let Some(best_alignment) = Self::candidate_alignment(ball, player, is_kickoff) else {
            return;
        };
        if best_alignment < SPEED_FLIP_MIN_ALIGNMENT {
            return;
        }
        let Some(start_velocity) = player.velocity() else {
            return;
        };
        let start_velocity_xy = start_velocity.truncate();
        let Some(start_forward_xy) = Self::forward_xy(player) else {
            return;
        };
        let dodge_delay_after_ground_leave_seconds = self
            .last_ground_contacts
            .get(&player.player_id)
            .map(|ground_contact_time| (frame.time - *ground_contact_time).max(0.0))
            .unwrap_or(0.0);

        let rotation = quat_to_glam(&rigid_body.rotation);
        let local_angular_velocity = rigid_body
            .angular_velocity
            .as_ref()
            .map(vec_to_glam)
            .map(|angular_velocity| rotation.inverse() * angular_velocity)
            .unwrap_or(glam::Vec3::ZERO);
        let best_diagonal_score = Self::diagonal_score(local_angular_velocity);
        let forward_z = (rotation * glam::Vec3::X).z;
        let initial_boost_alignment = Self::boost_alignment(player);

        self.active_candidates.arm(
            player.player_id.clone(),
            ActiveSpeedFlipCandidate {
                is_team_0: player.is_team_0,
                is_kickoff,
                kickoff_start_time,
                start_time: frame.time,
                start_frame: frame.frame_number,
                start_position: player_position.to_array(),
                end_position: player_position.to_array(),
                start_velocity,
                start_velocity_xy,
                start_forward_xy,
                local_forward: rotation * glam::Vec3::X,
                local_right: rotation * glam::Vec3::Y,
                local_up: rotation * glam::Vec3::Z,
                start_speed,
                max_speed: start_speed,
                best_alignment,
                initial_boost_alignment,
                best_boost_alignment: initial_boost_alignment.unwrap_or(best_alignment),
                boost_alignment_sample_count: u32::from(initial_boost_alignment.is_some()),
                dodge_delay_after_ground_leave_seconds,
                dodge_boost_compensation: glam::Vec3::ZERO,
                best_dodge_forward_delta: 0.0,
                best_dodge_delta_alignment: -1.0,
                best_estimated_dodge_impulse_magnitude: 0.0,
                best_estimated_dodge_impulse_forward_component: -1.0,
                best_estimated_dodge_impulse_side_component: 0.0,
                best_estimated_dodge_impulse_up_component: 1.0,
                dodge_acceleration_sample_count: 0,
                best_diagonal_score,
                max_forward_rotation_degrees: 0.0,
                max_up_rotation_degrees: 0.0,
                min_forward_z: forward_z,
                latest_forward_z: forward_z,
                latest_time: frame.time,
                latest_frame: frame.frame_number,
            },
        );
    }

    fn update_candidate(
        candidate: &mut ActiveSpeedFlipCandidate,
        frame: &FrameInfo,
        ball: &BallFrameState,
        player: &PlayerSample,
    ) {
        let Some(rigid_body) = player.rigid_body.as_ref() else {
            return;
        };

        if let Some(player_position) = player.position() {
            candidate.end_position = player_position.to_array();
        }
        candidate.max_speed = candidate.max_speed.max(player.speed().unwrap_or(0.0));
        if let Some(alignment) = Self::candidate_alignment(ball, player, candidate.is_kickoff) {
            candidate.best_alignment = candidate.best_alignment.max(alignment);
        }
        if let Some(boost_alignment) = Self::boost_alignment(player) {
            if candidate.initial_boost_alignment.is_none() {
                candidate.initial_boost_alignment = Some(boost_alignment);
            }
            candidate.best_boost_alignment = candidate.best_boost_alignment.max(boost_alignment);
            candidate.boost_alignment_sample_count += 1;
        }
        if frame.time > candidate.start_time
            && frame.time - candidate.start_time <= SPEED_FLIP_DODGE_ACCELERATION_SAMPLE_SECONDS
        {
            if let Some(velocity) = player.velocity() {
                if player.boost_active {
                    candidate.dodge_boost_compensation += candidate.local_forward
                        * BOOST_ACCELERATION_UU_PER_SECOND_SQUARED
                        * frame.dt;
                }
                let velocity_delta = velocity.truncate() - candidate.start_velocity_xy;
                let delta_length = velocity_delta.length();
                if delta_length > f32::EPSILON {
                    let forward_delta = velocity_delta.dot(candidate.start_forward_xy);
                    candidate.best_dodge_forward_delta =
                        candidate.best_dodge_forward_delta.max(forward_delta);
                    candidate.best_dodge_delta_alignment = candidate
                        .best_dodge_delta_alignment
                        .max(forward_delta / delta_length);
                    candidate.dodge_acceleration_sample_count += 1;
                }

                let estimated_delta =
                    velocity - candidate.start_velocity - candidate.dodge_boost_compensation;
                let estimated_horizontal_magnitude = estimated_delta.truncate().length();
                if estimated_horizontal_magnitude > f32::EPSILON {
                    let estimated_magnitude = estimated_delta.length();
                    let estimated_direction = estimated_delta / estimated_magnitude;
                    let forward_component = estimated_direction.dot(candidate.local_forward);
                    if estimated_horizontal_magnitude
                        > candidate.best_estimated_dodge_impulse_magnitude
                    {
                        candidate.best_estimated_dodge_impulse_magnitude =
                            estimated_horizontal_magnitude;
                        candidate.best_estimated_dodge_impulse_forward_component =
                            forward_component;
                        candidate.best_estimated_dodge_impulse_side_component =
                            estimated_direction.dot(candidate.local_right);
                        candidate.best_estimated_dodge_impulse_up_component =
                            estimated_direction.dot(candidate.local_up);
                    }
                }
            }
        }

        let rotation = quat_to_glam(&rigid_body.rotation);
        let local_angular_velocity = rigid_body
            .angular_velocity
            .as_ref()
            .map(vec_to_glam)
            .map(|angular_velocity| rotation.inverse() * angular_velocity)
            .unwrap_or(glam::Vec3::ZERO);
        candidate.best_diagonal_score = candidate
            .best_diagonal_score
            .max(Self::diagonal_score(local_angular_velocity));

        let current_forward = rotation * glam::Vec3::X;
        let current_up = rotation * glam::Vec3::Z;
        candidate.max_forward_rotation_degrees = candidate.max_forward_rotation_degrees.max(
            candidate
                .local_forward
                .angle_between(current_forward)
                .to_degrees(),
        );
        candidate.max_up_rotation_degrees = candidate
            .max_up_rotation_degrees
            .max(candidate.local_up.angle_between(current_up).to_degrees());

        let forward_z = (rotation * glam::Vec3::X).z;
        candidate.min_forward_z = candidate.min_forward_z.min(forward_z);
        candidate.latest_forward_z = forward_z;
        candidate.latest_time = frame.time;
        candidate.latest_frame = frame.frame_number;
    }

    fn candidate_event(
        player_id: &PlayerId,
        candidate: ActiveSpeedFlipCandidate,
    ) -> Option<SpeedFlipEvent> {
        let time_since_kickoff_start = candidate
            .kickoff_start_time
            .map(|kickoff_start_time| (candidate.start_time - kickoff_start_time).max(0.0))
            .unwrap_or(0.0);
        let timeliness_score = if candidate.is_kickoff {
            1.0 - Self::normalize_score(time_since_kickoff_start, 0.55, 1.1)
        } else {
            1.0
        };
        let cancel_recovery = candidate.latest_forward_z - candidate.min_forward_z;
        let level_recovery_score =
            1.0 - Self::normalize_score(candidate.latest_forward_z.abs(), 0.05, 0.55);
        let cancel_score = 0.25 * Self::normalize_score(-candidate.min_forward_z, 0.05, 0.35)
            + 0.35 * Self::normalize_score(cancel_recovery, 0.08, 0.5)
            + 0.40 * level_recovery_score;
        let speed_score = 0.55 * Self::normalize_score(candidate.max_speed, 1450.0, 1900.0)
            + 0.45
                * Self::normalize_score(candidate.max_speed - candidate.start_speed, 180.0, 650.0);
        let alignment_score = Self::normalize_score(candidate.best_alignment, 0.78, 0.98);
        if candidate.boost_alignment_sample_count == 0 {
            return None;
        }
        if candidate.dodge_delay_after_ground_leave_seconds
            > SPEED_FLIP_MAX_DODGE_DELAY_AFTER_GROUND_LEAVE_SECONDS
        {
            return None;
        }
        if candidate.dodge_acceleration_sample_count == 0
            || candidate.best_dodge_forward_delta < SPEED_FLIP_MIN_FORWARD_DODGE_DELTA
            || candidate.best_dodge_delta_alignment < SPEED_FLIP_MIN_FORWARD_DODGE_DELTA_ALIGNMENT
        {
            return None;
        }
        if candidate.max_up_rotation_degrees < SPEED_FLIP_MIN_UP_ROTATION_DEGREES {
            return None;
        }
        let estimated_dodge_side_component =
            candidate.best_estimated_dodge_impulse_side_component.abs();
        let estimated_dodge_up_component =
            candidate.best_estimated_dodge_impulse_up_component.abs();
        let has_meaningful_impulse = candidate.best_estimated_dodge_impulse_magnitude
            >= SPEED_FLIP_MIN_ESTIMATED_DODGE_IMPULSE_MAGNITUDE;
        let has_incompatible_meaningful_impulse = has_meaningful_impulse
            && (candidate.best_estimated_dodge_impulse_forward_component
                < SPEED_FLIP_MIN_ESTIMATED_DODGE_FORWARD_COMPONENT
                || estimated_dodge_side_component > SPEED_FLIP_MAX_ESTIMATED_DODGE_SIDE_COMPONENT
                || estimated_dodge_up_component > SPEED_FLIP_MAX_ESTIMATED_DODGE_UP_COMPONENT);
        if has_incompatible_meaningful_impulse {
            return None;
        }
        let weak_impulse_direction_is_usable = candidate.best_estimated_dodge_impulse_magnitude
            < SPEED_FLIP_MIN_DIRECTIONAL_WEAK_IMPULSE_MAGNITUDE
            || (candidate.best_estimated_dodge_impulse_forward_component >= 0.0
                && estimated_dodge_side_component >= SPEED_FLIP_MIN_WEAK_IMPULSE_SIDE_COMPONENT);
        if !weak_impulse_direction_is_usable {
            return None;
        }
        let has_strong_diagonal_rotation =
            candidate.best_diagonal_score >= SPEED_FLIP_MIN_STRONG_DIAGONAL_SCORE;
        let has_diagonal_impulse = has_meaningful_impulse
            && candidate.best_estimated_dodge_impulse_forward_component
                >= SPEED_FLIP_MIN_ESTIMATED_DODGE_FORWARD_COMPONENT
            && (SPEED_FLIP_MIN_ESTIMATED_DODGE_SIDE_COMPONENT
                ..=SPEED_FLIP_MAX_ESTIMATED_DODGE_SIDE_COMPONENT)
                .contains(&estimated_dodge_side_component)
            && estimated_dodge_up_component <= SPEED_FLIP_MAX_ESTIMATED_DODGE_UP_COMPONENT
            && candidate.best_diagonal_score >= SPEED_FLIP_MIN_DIAGONAL_SCORE;
        // Boost-through speed flips suppress the estimated dodge impulse (the
        // boost compensation swallows it) and sloppier flips land below the
        // strong-rotation score, but the flip cancel itself leaves a crisp
        // signature: the car's up vector sweeps through the dodge rotation
        // while the nose barely pitches because the cancel levels it out.
        let has_cancelled_diagonal_dodge = candidate.max_forward_rotation_degrees
            <= SPEED_FLIP_MAX_CANCELLED_FORWARD_ROTATION_DEGREES
            && candidate.best_diagonal_score >= SPEED_FLIP_MIN_DIAGONAL_SCORE;
        if !(has_strong_diagonal_rotation || has_diagonal_impulse || has_cancelled_diagonal_dodge) {
            return None;
        }
        let boost_alignment_score =
            Self::normalize_score(candidate.best_boost_alignment, 0.82, 0.99);
        let confidence = 0.30 * candidate.best_diagonal_score
            + 0.30 * cancel_score
            + 0.15 * speed_score
            + 0.15 * alignment_score
            + 0.05 * boost_alignment_score
            + 0.05 * timeliness_score;

        // Mid-flip the car is yawed off its velocity axis, so raw alignment of
        // a boost-through speed flip tops out around ~0.82 inside the short
        // evaluation window; gate on the raw alignment rather than the score.
        if candidate.best_boost_alignment < SPEED_FLIP_MIN_BOOST_ALIGNMENT {
            return None;
        }
        if cancel_score < 0.35 || confidence < SPEED_FLIP_MIN_CONFIDENCE {
            return None;
        }

        Some(SpeedFlipEvent {
            time: candidate.start_time,
            frame: candidate.start_frame,
            resolved_time: candidate.latest_time,
            resolved_frame: candidate.latest_frame,
            player: player_id.clone(),
            is_team_0: candidate.is_team_0,
            time_since_kickoff_start,
            start_position: candidate.start_position,
            end_position: candidate.end_position,
            start_speed: candidate.start_speed,
            max_speed: candidate.max_speed,
            best_alignment: candidate.best_alignment,
            initial_boost_alignment: candidate
                .initial_boost_alignment
                .unwrap_or(candidate.best_boost_alignment),
            best_boost_alignment: candidate.best_boost_alignment,
            boost_alignment_sample_count: candidate.boost_alignment_sample_count,
            dodge_delay_after_ground_leave_seconds: candidate
                .dodge_delay_after_ground_leave_seconds,
            diagonal_score: candidate.best_diagonal_score,
            estimated_dodge_impulse_magnitude: candidate.best_estimated_dodge_impulse_magnitude,
            estimated_dodge_impulse_forward_component: candidate
                .best_estimated_dodge_impulse_forward_component,
            estimated_dodge_impulse_side_component: candidate
                .best_estimated_dodge_impulse_side_component,
            estimated_dodge_impulse_up_component: candidate
                .best_estimated_dodge_impulse_up_component,
            cancel_score,
            speed_score,
            confidence,
        })
    }

    fn finalize_candidates(&mut self, frame: &FrameInfo, force_all: bool) {
        let mut finished_candidates = Vec::new();

        for (player_id, candidate) in self.active_candidates.iter() {
            let duration = frame.time - candidate.start_time;
            if force_all || duration >= SPEED_FLIP_EVALUATION_SECONDS {
                finished_candidates.push((
                    candidate.start_time,
                    candidate.start_frame,
                    format!("{player_id:?}"),
                    player_id.clone(),
                ));
            }
        }

        finished_candidates.sort_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });

        for (_, _, _, player_id) in finished_candidates {
            let Some(candidate) = self
                .active_candidates
                .finalize(&player_id, FinalizeReason::Completed)
            else {
                continue;
            };
            if let Some(event) = Self::candidate_event(&player_id, candidate) {
                self.apply_event(event);
            }
        }
    }

    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        let kickoff_approach_active = Self::kickoff_approach_active(gameplay);
        if !live_play_state.is_live_play && !kickoff_approach_active {
            self.active_candidates
                .apply_boundary(Boundary::LivePlayEnded);
            self.current_kickoff_start_time = None;
            self.kickoff_approach_active_last_frame = false;
            self.last_ground_contacts.clear();
            return Ok(());
        }

        if kickoff_approach_active && !self.kickoff_approach_active_last_frame {
            self.reset_kickoff_state();
        }

        self.update_kickoff_start_time(frame, kickoff_approach_active, players);
        self.update_ground_contacts(frame, players);

        for player in &players.players {
            self.maybe_start_candidate(frame, gameplay, ball, player, live_play_state);
        }

        for (player_id, candidate) in self.active_candidates.iter_mut() {
            let Some(player) = Self::player_by_id(players, player_id) else {
                continue;
            };
            Self::update_candidate(candidate, frame, ball, player);
        }

        self.finalize_candidates(frame, false);

        self.active_candidates.retain(|_, candidate| {
            frame.time - candidate.start_time <= SPEED_FLIP_MAX_CANDIDATE_SECONDS
        });

        if !kickoff_approach_active {
            self.current_kickoff_start_time = None;
        }

        self.kickoff_approach_active_last_frame = kickoff_approach_active;
        Ok(())
    }

    pub fn finalize_parts(&mut self, frame: &FrameInfo) {
        self.finalize_candidates(frame, true);
    }
}

#[cfg(test)]
#[path = "speed_flip_tests.rs"]
mod tests;
