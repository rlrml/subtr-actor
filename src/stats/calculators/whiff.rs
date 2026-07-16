use super::*;

// These thresholds are intentionally permissive: a candidate starts from
// domain evidence of a committed move at a nearby ball, then becomes a whiff
// only once the player clearly separates from the ball without any touch.
// Downstream consumers can use the emitted attempt span and closest-approach
// evidence to evaluate or refine that decision without redefining the event's
// lifecycle.
//
// The non-dodge approach path and the shared distance gate carry the loosening;
// the dodge-specific gates below are left at their stricter values so a clear
// side-dodge past the ball is still not treated as an attempt.
const WHIFF_ENTER_DISTANCE: f32 = 220.0;
const WHIFF_EXIT_DISTANCE: f32 = 360.0;
const WHIFF_MAX_CANDIDATE_SECONDS: f32 = 1.0;
const WHIFF_MIN_APPROACH_SPEED: f32 = 350.0;
const WHIFF_MIN_CLOSING_SPEED: f32 = 250.0;
const WHIFF_MIN_FORWARD_ALIGNMENT: f32 = 0.3;
const WHIFF_MIN_VELOCITY_ALIGNMENT: f32 = 0.45;
const WHIFF_MIN_DODGE_APPROACH_SPEED: f32 = 450.0;
const WHIFF_MIN_DODGE_CLOSING_SPEED: f32 = 300.0;
const WHIFF_MIN_DODGE_FORWARD_ALIGNMENT: f32 = 0.25;
const WHIFF_MAX_LATERAL_OFFSET: f32 = 200.0;
const WHIFF_MAX_DODGE_LATERAL_OFFSET: f32 = 150.0;
const WHIFF_MIN_LOCAL_FORWARD_OFFSET: f32 = 0.0;
const WHIFF_MIN_DODGE_LOCAL_FORWARD_OFFSET: f32 = -20.0;

/// Legacy outcome carried by serialized whiff events.
///
/// New detection only emits the whiff variant. The beaten-to-ball variant
/// remains readable so existing timelines and accumulated stats stay
/// compatible; new beaten-to-ball detection has its own event stream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum WhiffEventKind {
    #[default]
    Whiff,
    BeatenToBall,
}

/// Why a whiff candidate became a confirmed whiff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum WhiffResolutionReason {
    /// The player moved back outside the candidate window without any touch.
    SeparatedFromBall,
    /// Compatibility value for serialized events that predate resolution
    /// reasons (including the legacy beaten-to-ball subtype).
    LegacyUnknown,
}

/// A committed attempt near the ball that resolves as a clear miss.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WhiffEvent {
    #[serde(default)]
    pub kind: WhiffEventKind,
    /// First frame where the player satisfied the committed-attempt gates.
    pub start_time: f32,
    pub start_frame: usize,
    /// Closest-approach anchor for the event.
    pub time: f32,
    pub frame: usize,
    /// Frame where the miss became known.
    pub resolved_time: f32,
    pub resolved_frame: usize,
    pub resolution_reason: WhiffResolutionReason,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    /// Player position at closest approach.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub closest_approach_distance: f32,
    /// Car-forward alignment with the ball direction at closest approach.
    pub forward_alignment: f32,
    /// Player velocity projected toward the ball at closest approach.
    pub approach_speed: f32,
    /// Player-minus-ball velocity projected toward the ball at closest
    /// approach. Optional only for compatibility with older serialized events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closing_speed_at_closest: Option<f32>,
    /// Player velocity alignment with the ball direction at closest approach.
    /// Optional only for compatibility with older serialized events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub velocity_alignment_at_closest: Option<f32>,
    /// Ball position in car-local coordinates at closest approach. Optional
    /// only for compatibility with older serialized events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_ball_position_at_closest: Option<[f32; 3]>,
    /// Hitbox distance on the frame that confirmed separation. Optional only
    /// for compatibility with older serialized events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_distance: Option<f32>,
    /// Whether a dodge was observed during the attempt.
    pub dodge_active: bool,
    /// Whether the player was airborne during the attempt.
    pub aerial: bool,
}

pub(crate) const WHIFF_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("dodge_state", "no_dodge"),
    StatLabel::new("dodge_state", "dodge"),
];

impl WhiffEvent {
    pub(crate) fn labels(&self) -> [StatLabel; 2] {
        [
            vertical_state_label(self.aerial),
            whiff_dodge_state_label(self.dodge_active),
        ]
    }
}

pub(crate) fn whiff_dodge_state_label(dodge_active: bool) -> StatLabel {
    if dodge_active {
        StatLabel::new("dodge_state", "dodge")
    } else {
        StatLabel::new("dodge_state", "no_dodge")
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveWhiffCandidate {
    player: PlayerId,
    is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    closest_time: f32,
    closest_frame: usize,
    closest_position: [f32; 3],
    closest_approach_distance: f32,
    forward_alignment: f32,
    approach_speed: f32,
    closing_speed_at_closest: f32,
    velocity_alignment_at_closest: f32,
    local_ball_position_at_closest: [f32; 3],
    dodge_active: bool,
    aerial: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct WhiffEvidence {
    distance: f32,
    player_position: [f32; 3],
    local_ball_position: [f32; 3],
    forward_alignment: f32,
    approach_speed: f32,
    closing_speed: f32,
    velocity_alignment: f32,
    dodge_active: bool,
    aerial: bool,
}

impl InFlightItem for ActiveWhiffCandidate {
    fn recognition(&self) -> Recognition {
        // A whiff candidate is speculative: it only becomes an event once the
        // player separates from the ball without any touch. Touches, expiry,
        // missing players, and game-flow boundaries discard it.
        Recognition::speculative(self.start_time, self.start_frame)
    }

    fn on_boundary(&mut self, _boundary: Boundary) -> Disposition {
        // An in-flight candidate at a boundary never resolved into a whiff.
        Disposition::Discard
    }
}

/// Detects committed attempts that resolve as whiffs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WhiffCalculator {
    active_candidates: KeyedInFlightLedger<PlayerId, ActiveWhiffCandidate>,
    expired_candidates: HashSet<PlayerId>,
    events: EventStream<WhiffEvent>,
}

impl WhiffCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[WhiffEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[WhiffEvent] {
        self.events.new_events()
    }

    fn hitbox_distance(ball_position: glam::Vec3, player: &PlayerSample) -> Option<f32> {
        let rigid_body = player.rigid_body.as_ref()?;
        car_hitbox_distance(ball_position, rigid_body, player.hitbox)
    }

    fn evidence(
        ball_position: glam::Vec3,
        ball_velocity: glam::Vec3,
        player: &PlayerSample,
    ) -> Option<WhiffEvidence> {
        let distance = Self::hitbox_distance(ball_position, player)?;
        let rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let rotation = quat_to_glam(&rigid_body.rotation);
        let local_ball_position = rotation.inverse() * (ball_position - player_position);
        let to_ball = (ball_position - player_position).normalize_or_zero();
        if to_ball.length_squared() <= f32::EPSILON {
            return None;
        }

        let forward_alignment = (rotation * glam::Vec3::X).dot(to_ball);
        let player_velocity = player.velocity().unwrap_or(glam::Vec3::ZERO);
        let player_speed = player_velocity.length();
        let velocity_alignment = if player_speed <= f32::EPSILON {
            0.0
        } else {
            player_velocity.normalize_or_zero().dot(to_ball)
        };
        let approach_speed = player_velocity.dot(to_ball);
        let closing_speed = (player_velocity - ball_velocity).dot(to_ball);

        Some(WhiffEvidence {
            distance,
            player_position: player_position.to_array(),
            local_ball_position: local_ball_position.to_array(),
            forward_alignment,
            approach_speed,
            closing_speed,
            velocity_alignment,
            dodge_active: player.dodge_active,
            aerial: player_position.z > POWERSLIDE_MAX_Z_THRESHOLD,
        })
    }

    fn whiff_candidate(
        frame: &FrameInfo,
        player: &PlayerSample,
        evidence: WhiffEvidence,
    ) -> Option<ActiveWhiffCandidate> {
        if evidence.distance > WHIFF_ENTER_DISTANCE {
            return None;
        }

        let local_ball_position = glam::Vec3::from_array(evidence.local_ball_position);
        let ball_in_front = local_ball_position.x >= WHIFF_MIN_LOCAL_FORWARD_OFFSET
            && local_ball_position.y.abs() <= WHIFF_MAX_LATERAL_OFFSET;
        let dodge_ball_in_front = local_ball_position.x >= WHIFF_MIN_DODGE_LOCAL_FORWARD_OFFSET
            && local_ball_position.y.abs() <= WHIFF_MAX_DODGE_LATERAL_OFFSET;
        let committed_approach = evidence.approach_speed >= WHIFF_MIN_APPROACH_SPEED
            && evidence.closing_speed >= WHIFF_MIN_CLOSING_SPEED
            && evidence.forward_alignment >= WHIFF_MIN_FORWARD_ALIGNMENT;
        let directed_motion = evidence.velocity_alignment >= WHIFF_MIN_VELOCITY_ALIGNMENT;
        let committed_dodge = evidence.dodge_active
            && evidence.approach_speed >= WHIFF_MIN_DODGE_APPROACH_SPEED
            && evidence.closing_speed >= WHIFF_MIN_DODGE_CLOSING_SPEED
            && evidence.forward_alignment >= WHIFF_MIN_DODGE_FORWARD_ALIGNMENT
            && dodge_ball_in_front;
        if !(committed_dodge || committed_approach && directed_motion && ball_in_front) {
            return None;
        }

        Some(ActiveWhiffCandidate {
            player: player.player_id.clone(),
            is_team_0: player.is_team_0,
            start_time: frame.time,
            start_frame: frame.frame_number,
            closest_time: frame.time,
            closest_frame: frame.frame_number,
            closest_position: evidence.player_position,
            closest_approach_distance: evidence.distance,
            forward_alignment: evidence.forward_alignment,
            approach_speed: evidence.approach_speed,
            closing_speed_at_closest: evidence.closing_speed,
            velocity_alignment_at_closest: evidence.velocity_alignment,
            local_ball_position_at_closest: evidence.local_ball_position,
            dodge_active: evidence.dodge_active,
            aerial: evidence.aerial,
        })
    }

    fn emit_candidate(
        &mut self,
        candidate: ActiveWhiffCandidate,
        frame: &FrameInfo,
        resolved_distance: f32,
    ) {
        let event = WhiffEvent {
            kind: WhiffEventKind::Whiff,
            start_time: candidate.start_time,
            start_frame: candidate.start_frame,
            time: candidate.closest_time,
            frame: candidate.closest_frame,
            resolved_time: frame.time,
            resolved_frame: frame.frame_number,
            resolution_reason: WhiffResolutionReason::SeparatedFromBall,
            player: candidate.player.clone(),
            player_position: Some(candidate.closest_position),
            is_team_0: candidate.is_team_0,
            closest_approach_distance: candidate.closest_approach_distance,
            forward_alignment: candidate.forward_alignment,
            approach_speed: candidate.approach_speed,
            closing_speed_at_closest: Some(candidate.closing_speed_at_closest),
            velocity_alignment_at_closest: Some(candidate.velocity_alignment_at_closest),
            local_ball_position_at_closest: Some(candidate.local_ball_position_at_closest),
            resolved_distance: Some(resolved_distance),
            dodge_active: candidate.dodge_active,
            aerial: candidate.aerial,
        };
        self.events.push(event);
    }

    fn update_active_candidates(
        &mut self,
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        ball_velocity: glam::Vec3,
        players: &PlayerFrameState,
    ) {
        let mut visible_players = HashSet::new();

        for player in &players.players {
            let player_id = player.player_id.clone();
            visible_players.insert(player_id.clone());
            let evidence = Self::evidence(ball_position, ball_velocity, player);

            if self.expired_candidates.contains(&player_id) {
                if evidence
                    .as_ref()
                    .is_some_and(|evidence| evidence.distance > WHIFF_ENTER_DISTANCE)
                {
                    self.expired_candidates.remove(&player_id);
                }
                continue;
            }

            if self.active_candidates.contains(&player_id) {
                let expired = self
                    .active_candidates
                    .get(&player_id)
                    .is_some_and(|candidate| {
                        frame.time - candidate.start_time > WHIFF_MAX_CANDIDATE_SECONDS
                    });
                if expired {
                    // Candidate age is an implementation bound, not evidence of
                    // a miss. Suppress the same continuous interaction until
                    // the player leaves the entry window or a touch resets it.
                    self.active_candidates.discard(&player_id);
                    self.expired_candidates.insert(player_id);
                    continue;
                }

                if let Some(evidence) = evidence.as_ref() {
                    if let Some(candidate) = self.active_candidates.get_mut(&player_id) {
                        candidate.dodge_active |= evidence.dodge_active;
                        candidate.aerial |= evidence.aerial;
                        if evidence.distance < candidate.closest_approach_distance {
                            candidate.closest_approach_distance = evidence.distance;
                            candidate.closest_time = frame.time;
                            candidate.closest_frame = frame.frame_number;
                            candidate.closest_position = evidence.player_position;
                            candidate.forward_alignment = evidence.forward_alignment;
                            candidate.approach_speed = evidence.approach_speed;
                            candidate.closing_speed_at_closest = evidence.closing_speed;
                            candidate.velocity_alignment_at_closest = evidence.velocity_alignment;
                            candidate.local_ball_position_at_closest = evidence.local_ball_position;
                        }
                    }

                    if evidence.distance > WHIFF_EXIT_DISTANCE {
                        if let Some(candidate) = self
                            .active_candidates
                            .finalize(&player_id, FinalizeReason::Completed)
                        {
                            self.emit_candidate(candidate, frame, evidence.distance);
                        }
                        continue;
                    }
                }
                continue;
            }

            if let Some(candidate) =
                evidence.and_then(|evidence| Self::whiff_candidate(frame, player, evidence))
            {
                self.active_candidates.arm(player_id, candidate);
            }
        }

        let missing_players = self
            .active_candidates
            .keys()
            .filter(|player_id| !visible_players.contains(*player_id))
            .cloned()
            .collect::<Vec<_>>();
        for player_id in missing_players {
            self.active_candidates.discard(&player_id);
        }
        self.expired_candidates
            .retain(|player_id| visible_players.contains(player_id));
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.active_candidates
                .apply_boundary(Boundary::LivePlayEnded);
            self.expired_candidates.clear();
            return Ok(());
        }
        if !touch_state.touch_events.is_empty() {
            // Any touch resolves the current ball interaction before a whiff is
            // known: the candidate either succeeded or was interrupted. The
            // dedicated beaten-to-ball detector evaluates opponent touches.
            self.active_candidates.clear();
            self.expired_candidates.clear();
        }
        if touch_state.touch_events.is_empty() {
            if let Some(ball_position) = ball.position() {
                self.update_active_candidates(
                    frame,
                    ball_position,
                    ball.velocity().unwrap_or(glam::Vec3::ZERO),
                    players,
                );
            }
        }
        Ok(())
    }

    /// Resolve any in-flight candidates at end of stream. An unresolved
    /// candidate never became a whiff, so it is discarded (handled uniformly via
    /// the ledger rather than left to drop implicitly).
    pub fn finish(&mut self) {
        self.active_candidates.finish();
        self.expired_candidates.clear();
    }
}

#[cfg(test)]
#[path = "whiff_tests.rs"]
mod tests;
