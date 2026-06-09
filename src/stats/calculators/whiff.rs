use super::*;

const WHIFF_ENTER_DISTANCE: f32 = 150.0;
const WHIFF_EXIT_DISTANCE: f32 = 285.0;
const WHIFF_MAX_CANDIDATE_SECONDS: f32 = 0.65;
const WHIFF_MIN_APPROACH_SPEED: f32 = 700.0;
const WHIFF_MIN_CLOSING_SPEED: f32 = 450.0;
const WHIFF_MIN_FORWARD_ALIGNMENT: f32 = 0.55;
const WHIFF_MIN_VELOCITY_ALIGNMENT: f32 = 0.7;
const WHIFF_MIN_DODGE_APPROACH_SPEED: f32 = 450.0;
const WHIFF_MIN_DODGE_CLOSING_SPEED: f32 = 300.0;
const WHIFF_MIN_DODGE_FORWARD_ALIGNMENT: f32 = 0.25;
const WHIFF_MAX_LATERAL_OFFSET: f32 = 120.0;
const WHIFF_MAX_DODGE_LATERAL_OFFSET: f32 = 150.0;
const WHIFF_MIN_LOCAL_FORWARD_OFFSET: f32 = 0.0;
const WHIFF_MIN_DODGE_LOCAL_FORWARD_OFFSET: f32 = -20.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum WhiffEventKind {
    #[default]
    Whiff,
    BeatenToBall,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WhiffEvent {
    #[serde(default)]
    pub kind: WhiffEventKind,
    pub time: f32,
    pub frame: usize,
    pub resolved_time: f32,
    pub resolved_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub closest_approach_distance: f32,
    pub forward_alignment: f32,
    pub approach_speed: f32,
    pub dodge_active: bool,
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
    closest_time: f32,
    closest_frame: usize,
    closest_position: [f32; 3],
    closest_approach_distance: f32,
    forward_alignment: f32,
    approach_speed: f32,
    dodge_active: bool,
    aerial: bool,
}

impl InFlightItem for ActiveWhiffCandidate {
    fn recognition(&self) -> Recognition {
        // A whiff candidate is speculative: it only becomes an event if the
        // player misses (exit/timeout) or is beaten to the ball. A touch by the
        // candidate's own player, or a boundary, discards it.
        Recognition::speculative(self.start_time, self.closest_frame)
    }

    fn on_boundary(&mut self, _boundary: Boundary) -> Disposition {
        // An in-flight candidate at a boundary never resolved into a whiff.
        Disposition::Discard
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WhiffCalculator {
    active_candidates: KeyedInFlightLedger<PlayerId, ActiveWhiffCandidate>,
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

    fn local_ball_position(ball_position: glam::Vec3, player: &PlayerSample) -> Option<glam::Vec3> {
        let rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        Some(quat_to_glam(&rigid_body.rotation).inverse() * (ball_position - player_position))
    }

    fn whiff_candidate(
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        ball_velocity: glam::Vec3,
        player: &PlayerSample,
    ) -> Option<ActiveWhiffCandidate> {
        let distance = Self::hitbox_distance(ball_position, player)?;
        if distance > WHIFF_ENTER_DISTANCE {
            return None;
        }

        let rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let local_ball_position = Self::local_ball_position(ball_position, player)?;
        let to_ball = (ball_position - player_position).normalize_or_zero();
        if to_ball.length_squared() <= f32::EPSILON {
            return None;
        }

        let rotation = quat_to_glam(&rigid_body.rotation);
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
        let ball_in_front = local_ball_position.x >= WHIFF_MIN_LOCAL_FORWARD_OFFSET
            && local_ball_position.y.abs() <= WHIFF_MAX_LATERAL_OFFSET;
        let dodge_ball_in_front = local_ball_position.x >= WHIFF_MIN_DODGE_LOCAL_FORWARD_OFFSET
            && local_ball_position.y.abs() <= WHIFF_MAX_DODGE_LATERAL_OFFSET;
        let committed_approach = approach_speed >= WHIFF_MIN_APPROACH_SPEED
            && closing_speed >= WHIFF_MIN_CLOSING_SPEED
            && forward_alignment >= WHIFF_MIN_FORWARD_ALIGNMENT;
        let directed_motion = velocity_alignment >= WHIFF_MIN_VELOCITY_ALIGNMENT;
        let committed_dodge = player.dodge_active
            && approach_speed >= WHIFF_MIN_DODGE_APPROACH_SPEED
            && closing_speed >= WHIFF_MIN_DODGE_CLOSING_SPEED
            && forward_alignment >= WHIFF_MIN_DODGE_FORWARD_ALIGNMENT
            && dodge_ball_in_front;
        if !(committed_dodge || committed_approach && directed_motion && ball_in_front) {
            return None;
        }

        Some(ActiveWhiffCandidate {
            player: player.player_id.clone(),
            is_team_0: player.is_team_0,
            start_time: frame.time,
            closest_time: frame.time,
            closest_frame: frame.frame_number,
            closest_position: player_position.to_array(),
            closest_approach_distance: distance,
            forward_alignment,
            approach_speed,
            dodge_active: player.dodge_active,
            aerial: player_position.z > POWERSLIDE_MAX_Z_THRESHOLD,
        })
    }

    fn finish_touched_candidates(&mut self, frame: &FrameInfo, touch_state: &TouchState) {
        let touched_players = touch_state
            .touch_events
            .iter()
            .filter_map(|touch| touch.player.as_ref())
            .collect::<HashSet<_>>();
        let touched_teams = touch_state
            .touch_events
            .iter()
            .map(|touch| touch.team_is_team_0)
            .collect::<HashSet<_>>();
        if touched_players.is_empty() && touched_teams.is_empty() {
            return;
        }

        let candidate_players = self.active_candidates.keys().cloned().collect::<Vec<_>>();
        for player_id in candidate_players {
            let Some((candidate_player, candidate_team)) = self
                .active_candidates
                .get(&player_id)
                .map(|candidate| (candidate.player.clone(), candidate.is_team_0))
            else {
                continue;
            };
            if touched_players.contains(&candidate_player) {
                // The candidate's own player touched the ball: not a whiff.
                self.active_candidates.discard(&player_id);
            } else if touched_teams.contains(&!candidate_team) {
                if let Some(candidate) = self
                    .active_candidates
                    .finalize(&player_id, FinalizeReason::Completed)
                {
                    self.emit_candidate(candidate, frame, WhiffEventKind::BeatenToBall);
                }
            } else {
                self.active_candidates.discard(&player_id);
            }
        }
    }

    fn emit_candidate(
        &mut self,
        candidate: ActiveWhiffCandidate,
        frame: &FrameInfo,
        kind: WhiffEventKind,
    ) {
        let (time, frame_number) = match kind {
            WhiffEventKind::Whiff => (candidate.closest_time, candidate.closest_frame),
            WhiffEventKind::BeatenToBall => (frame.time, frame.frame_number),
        };
        let event = WhiffEvent {
            kind,
            time,
            frame: frame_number,
            resolved_time: frame.time,
            resolved_frame: frame.frame_number,
            player: candidate.player.clone(),
            player_position: Some(candidate.closest_position),
            is_team_0: candidate.is_team_0,
            closest_approach_distance: candidate.closest_approach_distance,
            forward_alignment: candidate.forward_alignment,
            approach_speed: candidate.approach_speed,
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
            let distance = Self::hitbox_distance(ball_position, player);

            if let (Some(candidate), Some(distance)) =
                (self.active_candidates.get_mut(&player_id), distance)
            {
                if distance < candidate.closest_approach_distance {
                    candidate.closest_approach_distance = distance;
                    candidate.closest_time = frame.time;
                    candidate.closest_frame = frame.frame_number;
                    if let Some(position) = player.position() {
                        candidate.closest_position = position.to_array();
                    }
                    if let Some(updated) =
                        Self::whiff_candidate(frame, ball_position, ball_velocity, player)
                    {
                        candidate.forward_alignment = updated.forward_alignment;
                        candidate.approach_speed = updated.approach_speed;
                        candidate.dodge_active |= updated.dodge_active;
                        candidate.aerial |= updated.aerial;
                    }
                }

                if distance > WHIFF_EXIT_DISTANCE
                    || frame.time - candidate.start_time > WHIFF_MAX_CANDIDATE_SECONDS
                {
                    if let Some(candidate) = self
                        .active_candidates
                        .finalize(&player_id, FinalizeReason::Completed)
                    {
                        self.emit_candidate(candidate, frame, WhiffEventKind::Whiff);
                    }
                }
                continue;
            }

            if let Some(candidate) =
                Self::whiff_candidate(frame, ball_position, ball_velocity, player)
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
            return Ok(());
        }
        self.finish_touched_candidates(frame, touch_state);
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
    }
}

#[cfg(test)]
#[path = "whiff_tests.rs"]
mod tests;
