use super::*;

const HALF_FLIP_EVALUATION_SECONDS: f32 = 0.65;
const HALF_FLIP_MAX_CANDIDATE_SECONDS: f32 = 1.0;
const HALF_FLIP_MAX_START_Z: f32 = PLAYER_GROUND_Z_THRESHOLD + 45.0;
const HALF_FLIP_MIN_START_SPEED: f32 = 250.0;
const HALF_FLIP_MIN_START_BACKWARD_ALIGNMENT: f32 = 0.55;
const HALF_FLIP_MIN_REORIENTATION_ALIGNMENT: f32 = 0.60;
const HALF_FLIP_MIN_FORWARD_REVERSAL: f32 = 0.55;
const HALF_FLIP_MIN_FORWARD_VERTICAL: f32 = 0.22;
const HALF_FLIP_MIN_CONFIDENCE: f32 = 0.55;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfFlipEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub start_position: [f32; 3],
    pub end_position: [f32; 3],
    pub start_speed: f32,
    pub end_speed: f32,
    pub start_backward_alignment: f32,
    pub best_reorientation_alignment: f32,
    pub best_forward_reversal: f32,
    pub max_forward_vertical: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveHalfFlipCandidate {
    is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    latest_time: f32,
    latest_frame: usize,
    start_position: [f32; 3],
    end_position: [f32; 3],
    start_speed: f32,
    end_speed: f32,
    start_forward_xy: glam::Vec2,
    start_backward_alignment: f32,
    best_reorientation_alignment: f32,
    best_forward_reversal: f32,
    max_forward_vertical: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HalfFlipCalculator {
    events: EventStream<HalfFlipEvent>,
    active_candidates: HashMap<PlayerId, ActiveHalfFlipCandidate>,
    previous_dodge_active: HashMap<PlayerId, bool>,
}

impl HalfFlipCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[HalfFlipEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[HalfFlipEvent] {
        self.events.new_events()
    }

    fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }

        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    fn horizontal_velocity(player: &PlayerSample) -> Option<glam::Vec2> {
        let velocity = player.velocity()?.truncate();
        if velocity.length_squared() <= f32::EPSILON {
            return None;
        }
        Some(velocity)
    }

    fn forward_vector(player: &PlayerSample) -> Option<glam::Vec3> {
        let rigid_body = player.rigid_body.as_ref()?;
        Some(quat_to_glam(&rigid_body.rotation) * glam::Vec3::X)
    }

    fn forward_xy(player: &PlayerSample) -> Option<glam::Vec2> {
        let forward_xy = Self::forward_vector(player)?.truncate().normalize_or_zero();
        if forward_xy.length_squared() <= f32::EPSILON {
            return None;
        }
        Some(forward_xy)
    }

    fn maybe_start_candidate(&mut self, frame: &FrameInfo, player: &PlayerSample) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        let Some(position) = player.position() else {
            return;
        };
        if position.z > HALF_FLIP_MAX_START_Z {
            return;
        }

        let velocity_xy = Self::horizontal_velocity(player).unwrap_or(glam::Vec2::ZERO);
        let start_speed = velocity_xy.length();
        if start_speed < HALF_FLIP_MIN_START_SPEED {
            return;
        }

        let Some(start_forward_xy) = Self::forward_xy(player) else {
            return;
        };
        let velocity_direction = velocity_xy.normalize_or_zero();
        let start_backward_alignment = -start_forward_xy.dot(velocity_direction);
        if start_backward_alignment < HALF_FLIP_MIN_START_BACKWARD_ALIGNMENT {
            return;
        }

        let max_forward_vertical =
            Self::forward_vector(player).map_or(0.0, |forward| forward.z.abs());

        self.active_candidates.insert(
            player.player_id.clone(),
            ActiveHalfFlipCandidate {
                is_team_0: player.is_team_0,
                start_time: frame.time,
                start_frame: frame.frame_number,
                latest_time: frame.time,
                latest_frame: frame.frame_number,
                start_position: position.to_array(),
                end_position: position.to_array(),
                start_speed,
                end_speed: start_speed,
                start_forward_xy,
                start_backward_alignment,
                best_reorientation_alignment: 0.0,
                best_forward_reversal: 0.0,
                max_forward_vertical,
            },
        );
    }

    fn update_candidate(
        candidate: &mut ActiveHalfFlipCandidate,
        frame: &FrameInfo,
        player: &PlayerSample,
    ) {
        if let Some(position) = player.position() {
            candidate.end_position = position.to_array();
        }

        let velocity_xy = Self::horizontal_velocity(player).unwrap_or(glam::Vec2::ZERO);
        candidate.end_speed = velocity_xy.length();
        let velocity_direction = velocity_xy.normalize_or_zero();

        if let Some(forward) = Self::forward_vector(player) {
            candidate.max_forward_vertical = candidate.max_forward_vertical.max(forward.z.abs());
            let forward_xy = forward.truncate().normalize_or_zero();
            if forward_xy.length_squared() > f32::EPSILON {
                candidate.best_forward_reversal = candidate
                    .best_forward_reversal
                    .max((-candidate.start_forward_xy.dot(forward_xy)).clamp(-1.0, 1.0));
                if velocity_direction.length_squared() > f32::EPSILON {
                    candidate.best_reorientation_alignment = candidate
                        .best_reorientation_alignment
                        .max(forward_xy.dot(velocity_direction));
                }
            }
        }

        candidate.latest_time = frame.time;
        candidate.latest_frame = frame.frame_number;
    }

    fn candidate_event(
        player_id: &PlayerId,
        candidate: ActiveHalfFlipCandidate,
    ) -> Option<HalfFlipEvent> {
        if candidate.best_reorientation_alignment < HALF_FLIP_MIN_REORIENTATION_ALIGNMENT
            || candidate.best_forward_reversal < HALF_FLIP_MIN_FORWARD_REVERSAL
            || candidate.max_forward_vertical < HALF_FLIP_MIN_FORWARD_VERTICAL
        {
            return None;
        }

        let backward_score = Self::normalize_score(
            candidate.start_backward_alignment,
            HALF_FLIP_MIN_START_BACKWARD_ALIGNMENT,
            0.95,
        );
        let reorientation_score = Self::normalize_score(
            candidate.best_reorientation_alignment,
            HALF_FLIP_MIN_REORIENTATION_ALIGNMENT,
            0.98,
        );
        let reversal_score = Self::normalize_score(
            candidate.best_forward_reversal,
            HALF_FLIP_MIN_FORWARD_REVERSAL,
            0.98,
        );
        let flip_score = Self::normalize_score(
            candidate.max_forward_vertical,
            HALF_FLIP_MIN_FORWARD_VERTICAL,
            0.85,
        );
        let speed_score = Self::normalize_score(candidate.end_speed, 900.0, 1800.0).max(
            Self::normalize_score(candidate.end_speed - candidate.start_speed, 100.0, 700.0) * 0.7,
        );
        let confidence = 0.25 * backward_score
            + 0.30 * reorientation_score
            + 0.25 * reversal_score
            + 0.10 * flip_score
            + 0.10 * speed_score;

        if confidence < HALF_FLIP_MIN_CONFIDENCE {
            return None;
        }

        Some(HalfFlipEvent {
            time: candidate.latest_time,
            frame: candidate.latest_frame,
            player: player_id.clone(),
            is_team_0: candidate.is_team_0,
            start_position: candidate.start_position,
            end_position: candidate.end_position,
            start_speed: candidate.start_speed,
            end_speed: candidate.end_speed,
            start_backward_alignment: candidate.start_backward_alignment,
            best_reorientation_alignment: candidate.best_reorientation_alignment,
            best_forward_reversal: candidate.best_forward_reversal,
            max_forward_vertical: candidate.max_forward_vertical,
            confidence,
        })
    }

    fn apply_event(&mut self, event: HalfFlipEvent) {
        self.events.push(event);
    }

    fn finalize_candidates(&mut self, frame: &FrameInfo, force_all: bool) {
        let mut finished_candidates = Vec::new();

        for (player_id, candidate) in &self.active_candidates {
            let duration = frame.time - candidate.start_time;
            if force_all || duration >= HALF_FLIP_EVALUATION_SECONDS {
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
            let Some(candidate) = self.active_candidates.remove(&player_id) else {
                continue;
            };
            if let Some(event) = Self::candidate_event(&player_id, candidate) {
                self.apply_event(event);
            }
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play {
            self.active_candidates.clear();
            return Ok(());
        }

        for player in &players.players {
            self.maybe_start_candidate(frame, player);
        }

        let mut visible_players = HashSet::new();
        for player in &players.players {
            visible_players.insert(player.player_id.clone());
            if let Some(candidate) = self.active_candidates.get_mut(&player.player_id) {
                Self::update_candidate(candidate, frame, player);
            }
        }

        self.finalize_candidates(frame, false);
        self.active_candidates.retain(|player_id, candidate| {
            visible_players.contains(player_id)
                && frame.time - candidate.start_time <= HALF_FLIP_MAX_CANDIDATE_SECONDS
        });

        Ok(())
    }

    pub fn finalize(&mut self, frame: &FrameInfo) {
        self.finalize_candidates(frame, true);
    }
}

#[cfg(test)]
#[path = "half_flip_tests.rs"]
mod tests;
