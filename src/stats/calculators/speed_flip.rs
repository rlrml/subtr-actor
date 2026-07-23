use super::*;

// A speed flip, from first principles, is a forward diagonal dodge whose flip
// is cancelled so the car keeps pointing where it is going: it rolls about its
// nose-to-tail axis to recover instead of tumbling end-over-end. We therefore
// detect it purely from the observed body orientation over the airborne arc of a
// ground dodge -- from the dodge until the car lands back on the ground -- with
// no impulse estimation (the raw `DodgeImpulse` is not even replicated in every
// replay; `DodgeTorque` is, but its frame is ambiguous, so we rely on the motion
// the dodge produces):
//
//   (a) a dodge fires while the car is on the ground,
//   (b) the maneuver leaves the car moving fast (the forward impulse landed),
//   (c) the nose stays aligned with the direction of travel throughout, and
//   (d) there is no end-over-end: the nose barely sweeps while the car rolls a
//       lot about its nose-to-tail axis.
//
// The maneuver is the airborne arc, so a candidate is evaluated the moment the
// car touches the ground again -- and a speed flip is a quick ground-recovery
// move, so the car must come back down within a bounded time. A dodge that stays
// airborne longer than that is an aerial, not a speed flip, and is discarded.

/// The dodge must land back on the ground within this long, otherwise the car is
/// flying (an aerial), not speed flipping. Measured dodge-to-landing arcs top out
/// around ~1.5s across real games (the dodge fires early in the jump, so the full
/// arc is ~1s); this sits above that distribution so real flips always land
/// first while genuine aerials never do.
const SPEED_FLIP_MAX_AIRBORNE_SECONDS: f32 = 1.8;
/// A speed flip is a ground-initiated dodge: the dodge fires out of the jump,
/// within this long of the car last touching the ground. Real speed flips dodge
/// ~0.15-0.25s into the jump; genuine aerial dodges only happen a second or more
/// after leaving the ground, so this cleanly separates them.
const SPEED_FLIP_MAX_GROUND_LEAVE_SECONDS: f32 = 0.30;
/// Ceiling on the dodge-start height. The dodge fires mid-jump (~60-90uu up), so
/// this is generous enough to cover the jump arc while excluding true aerials.
const SPEED_FLIP_MAX_START_Z: f32 = 130.0;
/// (b) The maneuver must leave the car genuinely fast. This both encodes "the
/// forward impulse landed" and rejects slow wavedashes/stalls.
const SPEED_FLIP_MIN_MAX_SPEED: f32 = 1600.0;
/// (c) Minimum cosine alignment the nose keeps with the horizontal travel
/// direction across the window (~53 degrees of slack).
const SPEED_FLIP_MIN_TRAVEL_ALIGNMENT: f32 = 0.60;
/// (d) The nose may not sweep more than this far from its dodge-start heading.
/// A genuine front/back flip swings the nose ~180 degrees; a speed flip keeps
/// it near its heading because the flip is cancelled.
const SPEED_FLIP_MAX_FORWARD_DEVIATION_DEGREES: f32 = 70.0;
/// (d) The car must actually roll about its nose-to-tail axis. A flat wavedash
/// barely rolls; a speed flip rolls through ~half-to-full revolution to recover.
const SPEED_FLIP_MIN_ROLL_SWEEP_DEGREES: f32 = 95.0;
/// Minimum forward-diagonal score when replicated dodge torque is available.
/// Inputs without dodge torque fall back to the observed roll-to-recover arc.
const SPEED_FLIP_MIN_DIAGONAL_SCORE: f32 = 0.35;
/// During kickoff approach, only dodges this soon after the kickoff start are
/// annotated with a meaningful `time_since_kickoff_start`.
const SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS: f32 = 1.1;
/// Players reaching at least this speed are treated as moving for kickoff-start
/// timing.
const SPEED_FLIP_KICKOFF_MOTION_SPEED: f32 = 100.0;
/// Below this horizontal speed the travel direction is too noisy to compare the
/// nose against, so those frames are skipped for the alignment measurement.
const SPEED_FLIP_MIN_TRAVEL_SPEED: f32 = 200.0;

/// A forward diagonal dodge whose flip is cancelled into a roll-to-recover,
/// keeping the car pointed along its travel direction (a speed flip).
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
    /// Seconds between the kickoff start and this dodge, or 0 when the dodge did
    /// not happen during a kickoff approach.
    pub time_since_kickoff_start: f32,
    pub start_position: [f32; 3],
    pub end_position: [f32; 3],
    pub start_speed: f32,
    pub max_speed: f32,
    /// (b) Peak gain in speed measured along the car's dodge-start heading
    /// (uu/s). Tops out near zero when the car was already supersonic.
    pub forward_speed_gain: f32,
    /// (c) Minimum cosine alignment between the car's horizontal forward and its
    /// horizontal travel direction across the window. Higher means the nose
    /// stayed pointed where the car was going.
    pub min_travel_alignment: f32,
    /// (d) Largest angle (degrees) the nose swept from its dodge-start heading.
    /// Small means there was no end-over-end tumble.
    pub max_forward_deviation_degrees: f32,
    /// (d) Largest angle (degrees) the car's up vector swept from dodge start:
    /// the roll about the nose-to-tail axis that recovers the flip.
    pub roll_sweep_degrees: f32,
    /// Car-local side component of the replicated dodge torque. Its sign is
    /// used to retain left/right kickoff approach direction.
    #[serde(default)]
    pub dodge_side_component: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveSpeedFlipCandidate {
    is_team_0: bool,
    kickoff_start_time: Option<f32>,
    start_time: f32,
    start_frame: usize,
    start_position: [f32; 3],
    end_position: [f32; 3],
    start_speed: f32,
    start_forward: glam::Vec3,
    start_up: glam::Vec3,
    start_heading_xy: glam::Vec2,
    start_forward_speed: f32,
    max_speed: f32,
    max_forward_speed: f32,
    min_travel_alignment: f32,
    max_forward_deviation_degrees: f32,
    roll_sweep_degrees: f32,
    dodge_torque: Option<glam::Vec3>,
    has_landed: bool,
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

/// Detects speed flips from the body orientation a ground dodge produces.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpeedFlipCalculator {
    events: EventStream<SpeedFlipEvent>,
    active_candidates: KeyedInFlightLedger<PlayerId, ActiveSpeedFlipCandidate>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    last_ground_contacts: HashMap<PlayerId, f32>,
    kickoff_approach_active_last_frame: bool,
    kickoff_window_open: bool,
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

    /// Whether we are inside a kickoff approach window.
    ///
    /// The opening kickoff reports `ball_has_been_hit == None` between the
    /// countdown ending and the first touch. Keep the countdown-opened window
    /// alive through that interval instead of requiring `Some(false)`.
    fn update_kickoff_window(&mut self, gameplay: &GameplayState) -> bool {
        self.kickoff_window_open =
            if gameplay.kickoff_countdown_active() || gameplay.ball_has_been_hit == Some(false) {
                true
            } else if gameplay.ball_has_been_hit == Some(true) {
                false
            } else {
                self.kickoff_window_open
            };
        self.kickoff_window_open
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

    /// Forward-diagonal score from replicated car-relative dodge torque.
    ///
    /// `y` is the forward/back component and `x` is the side component. The
    /// score peaks at a 45-degree forward diagonal and falls to zero for pure
    /// forward, pure side, and backward dodges.
    fn diagonal_score_from_torque(torque: glam::Vec3) -> f32 {
        let forward = torque.y;
        let side = torque.x.abs();
        if forward <= 0.0 {
            return 0.0;
        }

        let magnitude = glam::Vec2::new(side, forward).length();
        if magnitude <= f32::EPSILON {
            return 0.0;
        }

        // forward * side / magnitude^2 peaks at 0.5 for a perfect 45-degree
        // diagonal, so scale it to a maximum score of 1.0.
        (2.0 * forward * side / (magnitude * magnitude)).clamp(0.0, 1.0)
    }

    fn orientation(player: &PlayerSample) -> Option<(glam::Vec3, glam::Vec3)> {
        let rigid_body = player.rigid_body.as_ref()?;
        let rotation = quat_to_glam(&rigid_body.rotation);
        Some((rotation * glam::Vec3::X, rotation * glam::Vec3::Z))
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

    fn reset_kickoff_state(&mut self) {
        self.active_candidates.clear();
        self.current_kickoff_start_time = None;
    }

    fn maybe_start_candidate(
        &mut self,
        frame: &FrameInfo,
        kickoff_approach_active: bool,
        player: &PlayerSample,
    ) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        // (a) The dodge must fire from the ground.
        let Some(player_position) = player.position() else {
            return;
        };
        if player_position.z > SPEED_FLIP_MAX_START_Z {
            return;
        }
        let recently_grounded = self
            .last_ground_contacts
            .get(&player.player_id)
            .is_some_and(|contact| frame.time - *contact <= SPEED_FLIP_MAX_GROUND_LEAVE_SECONDS);
        if !recently_grounded {
            return;
        }

        let Some((start_forward, start_up)) = Self::orientation(player) else {
            return;
        };
        let start_heading_xy = start_forward.truncate().normalize_or_zero();
        if start_heading_xy.length_squared() <= f32::EPSILON {
            return;
        }
        let start_velocity = player.velocity().unwrap_or(glam::Vec3::ZERO);
        let start_speed = start_velocity.length();
        let start_forward_speed = start_velocity.truncate().dot(start_heading_xy);

        let kickoff_start_time = if kickoff_approach_active {
            self.current_kickoff_start_time
                .filter(|kickoff_start_time| {
                    frame.time - kickoff_start_time <= SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS
                })
        } else {
            None
        };

        self.active_candidates.arm(
            player.player_id.clone(),
            ActiveSpeedFlipCandidate {
                is_team_0: player.is_team_0,
                kickoff_start_time,
                start_time: frame.time,
                start_frame: frame.frame_number,
                start_position: player_position.to_array(),
                end_position: player_position.to_array(),
                start_speed,
                start_forward,
                start_up,
                start_heading_xy,
                start_forward_speed,
                max_speed: start_speed,
                max_forward_speed: start_forward_speed,
                min_travel_alignment: 1.0,
                max_forward_deviation_degrees: 0.0,
                roll_sweep_degrees: 0.0,
                dodge_torque: player.dodge_torque,
                has_landed: false,
                latest_time: frame.time,
                latest_frame: frame.frame_number,
            },
        );
    }

    fn update_candidate(
        candidate: &mut ActiveSpeedFlipCandidate,
        frame: &FrameInfo,
        player: &PlayerSample,
    ) {
        if let Some(player_position) = player.position() {
            candidate.end_position = player_position.to_array();
            // The dodge fires airborne (mid-jump); the maneuver ends when the
            // car comes back down and touches the ground.
            if frame.time > candidate.start_time && player_position.z <= PLAYER_GROUND_Z_THRESHOLD {
                candidate.has_landed = true;
            }
        }

        if let Some(velocity) = player.velocity() {
            candidate.max_speed = candidate.max_speed.max(velocity.length());
            let velocity_xy = velocity.truncate();
            candidate.max_forward_speed = candidate
                .max_forward_speed
                .max(velocity_xy.dot(candidate.start_heading_xy));

            // (c) How well the nose stays pointed along the direction of travel.
            if velocity_xy.length() >= SPEED_FLIP_MIN_TRAVEL_SPEED {
                if let Some((forward, _)) = Self::orientation(player) {
                    let forward_xy = forward.truncate().normalize_or_zero();
                    let travel_xy = velocity_xy.normalize_or_zero();
                    if forward_xy.length_squared() > f32::EPSILON
                        && travel_xy.length_squared() > f32::EPSILON
                    {
                        candidate.min_travel_alignment = candidate
                            .min_travel_alignment
                            .min(forward_xy.dot(travel_xy));
                    }
                }
            }
        }

        // (d) Separate nose sweep (end-over-end) from roll about the nose axis.
        if let Some((forward, up)) = Self::orientation(player) {
            candidate.max_forward_deviation_degrees = candidate
                .max_forward_deviation_degrees
                .max(forward.angle_between(candidate.start_forward).to_degrees());
            candidate.roll_sweep_degrees = candidate
                .roll_sweep_degrees
                .max(up.angle_between(candidate.start_up).to_degrees());
        }

        candidate.latest_time = frame.time;
        candidate.latest_frame = frame.frame_number;
    }

    fn candidate_event(
        player_id: &PlayerId,
        candidate: ActiveSpeedFlipCandidate,
    ) -> Option<SpeedFlipEvent> {
        // The car must have come back down within the airborne budget. A
        // candidate finalized at the cap (or at replay end) without landing was
        // an aerial, not a speed flip.
        if !candidate.has_landed {
            return None;
        }
        // (b) The maneuver left the car fast.
        if candidate.max_speed < SPEED_FLIP_MIN_MAX_SPEED {
            return None;
        }
        // (c) The nose stayed pointed along travel.
        if candidate.min_travel_alignment < SPEED_FLIP_MIN_TRAVEL_ALIGNMENT {
            return None;
        }
        // (d) No end-over-end, and a real roll about the nose-to-tail axis.
        if candidate.max_forward_deviation_degrees > SPEED_FLIP_MAX_FORWARD_DEVIATION_DEGREES {
            return None;
        }
        if candidate.roll_sweep_degrees < SPEED_FLIP_MIN_ROLL_SWEEP_DEGREES {
            return None;
        }
        let diagonal_score = candidate.dodge_torque.map(Self::diagonal_score_from_torque);
        if diagonal_score.is_some_and(|score| score < SPEED_FLIP_MIN_DIAGONAL_SCORE) {
            return None;
        }
        // A near-complete inversion is only a clean cancelled recovery when
        // the car remains tightly aligned with its travel. This rejects full
        // diagonal flips without dropping the high-alignment speed flips that
        // briefly approach 180 degrees during their recovery.
        if candidate.roll_sweep_degrees > 170.0
            && candidate.min_travel_alignment < 0.89
            && diagonal_score.is_none_or(|score| score < 0.90)
        {
            return None;
        }
        if candidate.roll_sweep_degrees >= 165.0 && candidate.min_travel_alignment < 0.75 {
            return None;
        }

        let forward_speed_gain =
            (candidate.max_forward_speed - candidate.start_forward_speed).max(0.0);
        let time_since_kickoff_start = candidate
            .kickoff_start_time
            .map(|kickoff_start_time| (candidate.start_time - kickoff_start_time).max(0.0))
            .unwrap_or(0.0);

        // Confidence is a quality readout for the overlay, not a gate: the hard
        // criteria above decide acceptance. It rewards staying level and aligned,
        // a full roll, and high speed.
        let level_term = 1.0
            - Self::normalize_score(
                candidate.max_forward_deviation_degrees,
                10.0,
                SPEED_FLIP_MAX_FORWARD_DEVIATION_DEGREES,
            );
        let alignment_term = Self::normalize_score(candidate.min_travel_alignment, 0.60, 0.95);
        let roll_term = Self::normalize_score(
            candidate.roll_sweep_degrees,
            SPEED_FLIP_MIN_ROLL_SWEEP_DEGREES,
            180.0,
        );
        let speed_term =
            Self::normalize_score(candidate.max_speed, SPEED_FLIP_MIN_MAX_SPEED, 2300.0);
        let confidence =
            (0.35 * level_term + 0.30 * alignment_term + 0.20 * roll_term + 0.15 * speed_term)
                .clamp(0.0, 1.0);

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
            forward_speed_gain,
            min_travel_alignment: candidate.min_travel_alignment,
            max_forward_deviation_degrees: candidate.max_forward_deviation_degrees,
            roll_sweep_degrees: candidate.roll_sweep_degrees,
            dodge_side_component: candidate.dodge_torque.map_or(0.0, |torque| torque.x),
            confidence,
        })
    }

    fn finalize_candidates(&mut self, frame: &FrameInfo, force_all: bool) {
        let mut finished_candidates = Vec::new();

        for (player_id, candidate) in self.active_candidates.iter() {
            let duration = frame.time - candidate.start_time;
            if force_all || candidate.has_landed || duration >= SPEED_FLIP_MAX_AIRBORNE_SECONDS {
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
                self.events.push(event);
            }
        }
    }

    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        _ball: &BallFrameState,
        players: &PlayerFrameState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        let kickoff_approach_active = self.update_kickoff_window(gameplay);
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
            self.maybe_start_candidate(frame, kickoff_approach_active, player);
        }

        for (player_id, candidate) in self.active_candidates.iter_mut() {
            let Some(player) = Self::player_by_id(players, player_id) else {
                continue;
            };
            Self::update_candidate(candidate, frame, player);
        }

        self.finalize_candidates(frame, false);

        self.active_candidates.retain(|_, candidate| {
            frame.time - candidate.start_time <= SPEED_FLIP_MAX_AIRBORNE_SECONDS
        });

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
