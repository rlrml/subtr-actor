use super::*;

const FLIP_IMPULSE_EVALUATION_SECONDS: f32 = 0.18;
const FLIP_IMPULSE_MAX_CANDIDATE_SECONDS: f32 = 0.35;
const FLIP_IMPULSE_MIN_DELTA: f32 = 10.0;
const FLIP_IMPULSE_STRONG_DELTA: f32 = 280.0;
const BOOST_ACCELERATION_UU_PER_SECOND_SQUARED: f32 = 991.6667;

/// An estimated dodge impulse derived from a measurable velocity change.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeImpulse {
    pub start_position: [f32; 3],
    pub end_position: [f32; 3],
    pub start_speed: f32,
    pub end_speed: f32,
    pub raw_velocity_delta: [f32; 3],
    pub estimated_impulse_delta: [f32; 3],
    pub estimated_direction: [f32; 3],
    pub estimated_horizontal_direction: [f32; 2],
    pub estimated_impulse_magnitude: f32,
    pub estimated_horizontal_impulse_magnitude: f32,
    pub local_forward_component: f32,
    pub local_right_component: f32,
    pub local_up_component: f32,
    pub direction_label: String,
    pub boost_sample_count: u32,
    pub sample_count: u32,
    pub boost_compensation_magnitude: f32,
    pub confidence: f32,
}

/// A dodge-start event, optionally carrying an estimated dodge impulse.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeEvent {
    pub time: f32,
    pub frame: usize,
    pub resolved_time: f32,
    pub resolved_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub dodge_impulse: Option<DodgeImpulse>,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveFlipImpulseCandidate {
    is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    latest_time: f32,
    latest_frame: usize,
    start_position: glam::Vec3,
    end_position: glam::Vec3,
    start_velocity: glam::Vec3,
    end_velocity: glam::Vec3,
    local_forward: glam::Vec3,
    local_right: glam::Vec3,
    local_up: glam::Vec3,
    boost_compensation: glam::Vec3,
    sample_count: u32,
    boost_sample_count: u32,
}

impl InFlightItem for ActiveFlipImpulseCandidate {
    fn recognition(&self) -> Recognition {
        // Speculative until it survives the evaluation window: a candidate can
        // be pruned (stale) or discarded at a boundary before emitting.
        Recognition::speculative(self.start_time, self.start_frame)
    }

    fn on_boundary(&mut self, _boundary: Boundary) -> Disposition {
        // Candidates in flight at a boundary are dropped (matching the previous
        // clear-on-stoppage / drop-at-end behavior).
        Disposition::Discard
    }
}

/// Detects dodge starts / flip impulses from player frame state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlipImpulseCalculator {
    events: EventStream<DodgeEvent>,
    active_candidates: KeyedInFlightLedger<PlayerId, ActiveFlipImpulseCandidate>,
    previous_dodge_active: HashMap<PlayerId, bool>,
}

impl FlipImpulseCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[DodgeEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[DodgeEvent] {
        self.events.new_events()
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

    fn direction_label(local_forward: f32, local_right: f32, local_up: f32) -> String {
        let mut parts = Vec::new();
        if local_forward.abs() >= 0.28 {
            parts.push(if local_forward >= 0.0 {
                "forward"
            } else {
                "backward"
            });
        }
        if local_right.abs() >= 0.28 {
            parts.push(if local_right >= 0.0 { "right" } else { "left" });
        }
        if parts.is_empty() {
            parts.push("neutral");
        }
        if local_up.abs() >= 0.45 {
            parts.push(if local_up >= 0.0 { "up" } else { "down" });
        }
        parts.join("_")
    }

    fn score_confidence(
        impulse_magnitude: f32,
        boost_compensation_magnitude: f32,
        sample_count: u32,
    ) -> f32 {
        let strength_score = ((impulse_magnitude - FLIP_IMPULSE_MIN_DELTA)
            / FLIP_IMPULSE_STRONG_DELTA)
            .clamp(0.0, 1.0);
        let boost_ratio = boost_compensation_magnitude
            / (impulse_magnitude + boost_compensation_magnitude).max(1.0);
        let boost_penalty = (1.0 - boost_ratio * 0.75).clamp(0.25, 1.0);
        let sample_score = (sample_count as f32 / 3.0).clamp(0.35, 1.0);
        (0.20 + 0.80 * strength_score) * boost_penalty * sample_score
    }

    fn maybe_start_candidate(&mut self, frame: &FrameInfo, player: &PlayerSample) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        let Some(rigid_body) = player.rigid_body.as_ref() else {
            return;
        };
        let Some(position) = player.position() else {
            return;
        };
        let Some(velocity) = player.velocity() else {
            return;
        };

        let rotation = quat_to_glam(&rigid_body.rotation);
        self.active_candidates.arm(
            player.player_id.clone(),
            ActiveFlipImpulseCandidate {
                is_team_0: player.is_team_0,
                start_time: frame.time,
                start_frame: frame.frame_number,
                latest_time: frame.time,
                latest_frame: frame.frame_number,
                start_position: position,
                end_position: position,
                start_velocity: velocity,
                end_velocity: velocity,
                local_forward: rotation * glam::Vec3::X,
                local_right: rotation * glam::Vec3::Y,
                local_up: rotation * glam::Vec3::Z,
                boost_compensation: glam::Vec3::ZERO,
                sample_count: 0,
                boost_sample_count: 0,
            },
        );
    }

    fn update_candidate(
        candidate: &mut ActiveFlipImpulseCandidate,
        frame: &FrameInfo,
        player: &PlayerSample,
    ) {
        if frame.time <= candidate.start_time
            || frame.time - candidate.start_time > FLIP_IMPULSE_EVALUATION_SECONDS
        {
            return;
        }

        if let Some(position) = player.position() {
            candidate.end_position = position;
        }
        if let Some(velocity) = player.velocity() {
            candidate.end_velocity = velocity;
            candidate.sample_count += 1;
        }

        if player.boost_active {
            candidate.boost_sample_count += 1;
            candidate.boost_compensation +=
                candidate.local_forward * BOOST_ACCELERATION_UU_PER_SECOND_SQUARED * frame.dt;
        }

        candidate.latest_time = frame.time;
        candidate.latest_frame = frame.frame_number;
    }

    fn candidate_event(player_id: &PlayerId, candidate: ActiveFlipImpulseCandidate) -> DodgeEvent {
        let raw_delta = candidate.end_velocity - candidate.start_velocity;
        let estimated_delta = raw_delta - candidate.boost_compensation;
        let estimated_magnitude = estimated_delta.length();
        let dodge_impulse = (candidate.sample_count > 0
            && estimated_magnitude >= FLIP_IMPULSE_MIN_DELTA)
            .then(|| {
                let direction = estimated_delta / estimated_magnitude;
                let horizontal_delta = estimated_delta.truncate();
                let horizontal_magnitude = horizontal_delta.length();
                let horizontal_direction = if horizontal_magnitude > f32::EPSILON {
                    horizontal_delta / horizontal_magnitude
                } else {
                    glam::Vec2::ZERO
                };
                let local_forward_component = direction.dot(candidate.local_forward);
                let local_right_component = direction.dot(candidate.local_right);
                let local_up_component = direction.dot(candidate.local_up);
                let boost_compensation_magnitude = candidate.boost_compensation.length();
                let confidence = Self::score_confidence(
                    estimated_magnitude,
                    boost_compensation_magnitude,
                    candidate.sample_count,
                );

                DodgeImpulse {
                    start_position: candidate.start_position.to_array(),
                    end_position: candidate.end_position.to_array(),
                    start_speed: candidate.start_velocity.length(),
                    end_speed: candidate.end_velocity.length(),
                    raw_velocity_delta: raw_delta.to_array(),
                    estimated_impulse_delta: estimated_delta.to_array(),
                    estimated_direction: direction.to_array(),
                    estimated_horizontal_direction: horizontal_direction.to_array(),
                    estimated_impulse_magnitude: estimated_magnitude,
                    estimated_horizontal_impulse_magnitude: horizontal_magnitude,
                    local_forward_component,
                    local_right_component,
                    local_up_component,
                    direction_label: Self::direction_label(
                        local_forward_component,
                        local_right_component,
                        local_up_component,
                    ),
                    boost_sample_count: candidate.boost_sample_count,
                    sample_count: candidate.sample_count,
                    boost_compensation_magnitude,
                    confidence,
                }
            });

        DodgeEvent {
            time: candidate.start_time,
            frame: candidate.start_frame,
            resolved_time: candidate.latest_time,
            resolved_frame: candidate.latest_frame,
            player: player_id.clone(),
            is_team_0: candidate.is_team_0,
            dodge_impulse,
        }
    }

    fn finalize_candidates(&mut self, frame: &FrameInfo, force_all: bool) {
        let mut finished_candidates = Vec::new();

        for (player_id, candidate) in self.active_candidates.iter() {
            let duration = frame.time - candidate.start_time;
            if force_all || duration >= FLIP_IMPULSE_EVALUATION_SECONDS {
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
            let event = Self::candidate_event(&player_id, candidate);
            self.events.push(event);
        }
    }

    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();

        if !live_play_state.counts_toward_player_motion() {
            self.active_candidates
                .apply_boundary(Boundary::LivePlayEnded);
            return Ok(());
        }

        for player in &players.players {
            self.maybe_start_candidate(frame, player);
        }

        for (player_id, candidate) in self.active_candidates.iter_mut() {
            let Some(player) = Self::player_by_id(players, player_id) else {
                continue;
            };
            Self::update_candidate(candidate, frame, player);
        }

        self.finalize_candidates(frame, false);
        self.active_candidates.retain(|_, candidate| {
            frame.time - candidate.start_time <= FLIP_IMPULSE_MAX_CANDIDATE_SECONDS
        });
        Ok(())
    }

    pub fn finalize_parts(&mut self, frame: &FrameInfo) {
        self.finalize_candidates(frame, true);
    }
}

#[cfg(test)]
#[path = "flip_impulse_tests.rs"]
mod tests;
