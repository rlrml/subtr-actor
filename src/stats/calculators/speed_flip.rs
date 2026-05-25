use super::*;

const SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS: f32 = 1.1;
const SPEED_FLIP_EVALUATION_SECONDS: f32 = 0.32;
const SPEED_FLIP_MAX_CANDIDATE_SECONDS: f32 = 0.55;
const SPEED_FLIP_MAX_GROUND_Z: f32 = 80.0;
const SPEED_FLIP_KICKOFF_MOTION_SPEED: f32 = 100.0;
const SPEED_FLIP_MIN_ALIGNMENT: f32 = 0.72;
const SPEED_FLIP_DODGE_ACCELERATION_SAMPLE_SECONDS: f32 = 0.18;
const SPEED_FLIP_MIN_FORWARD_DODGE_DELTA: f32 = 80.0;
const SPEED_FLIP_MIN_FORWARD_DODGE_DELTA_ALIGNMENT: f32 = 0.35;
const SPEED_FLIP_MIN_CONFIDENCE: f32 = 0.45;
const SPEED_FLIP_HIGH_CONFIDENCE: f32 = 0.75;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SpeedFlipEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub time_since_kickoff_start: f32,
    pub start_position: [f32; 3],
    pub end_position: [f32; 3],
    pub start_speed: f32,
    pub max_speed: f32,
    pub best_alignment: f32,
    pub diagonal_score: f32,
    pub cancel_score: f32,
    pub speed_score: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SpeedFlipStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_speed_flip: bool,
    pub last_speed_flip_time: Option<f32>,
    pub last_speed_flip_frame: Option<usize>,
    pub time_since_last_speed_flip: Option<f32>,
    pub frames_since_last_speed_flip: Option<usize>,
    pub last_quality: Option<f32>,
    pub best_quality: f32,
    pub cumulative_quality: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl SpeedFlipStats {
    pub fn average_quality(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_quality / self.count as f32
        }
    }

    fn record_event(&mut self, event: &SpeedFlipEvent) {
        self.labeled_event_counts.increment([confidence_band_label(
            event.confidence >= SPEED_FLIP_HIGH_CONFIDENCE,
        )]);
        self.sync_legacy_counts();
        self.last_speed_flip_time = Some(event.time);
        self.last_speed_flip_frame = Some(event.frame);
        self.last_quality = Some(event.confidence);
        self.best_quality = self.best_quality.max(event.confidence);
        self.cumulative_quality += event.confidence;
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&CONFIDENCE_BAND_LABELS],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.high_confidence_count = self.event_count_with_labels(&[confidence_band_label(true)]);
    }
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
    start_velocity_xy: glam::Vec2,
    start_forward_xy: glam::Vec2,
    start_speed: f32,
    max_speed: f32,
    best_alignment: f32,
    best_boost_alignment: f32,
    boost_alignment_sample_count: u32,
    best_dodge_forward_delta: f32,
    best_dodge_delta_alignment: f32,
    dodge_acceleration_sample_count: u32,
    best_diagonal_score: f32,
    min_forward_z: f32,
    latest_forward_z: f32,
    latest_time: f32,
    latest_frame: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpeedFlipCalculator {
    player_stats: HashMap<PlayerId, SpeedFlipStats>,
    events: Vec<SpeedFlipEvent>,
    active_candidates: HashMap<PlayerId, ActiveSpeedFlipCandidate>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    kickoff_approach_active_last_frame: bool,
    current_kickoff_start_time: Option<f32>,
    current_last_speed_flip_player: Option<PlayerId>,
}

impl SpeedFlipCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, SpeedFlipStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[SpeedFlipEvent] {
        &self.events
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

    fn candidate_alignment(
        _ball: &BallFrameState,
        player: &PlayerSample,
        _is_kickoff: bool,
    ) -> Option<f32> {
        Self::forward_speed_alignment(player)
    }

    fn apply_event(&mut self, event: SpeedFlipEvent) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_speed_flip = false;
        }

        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(&event);
        stats.is_last_speed_flip = true;
        stats.time_since_last_speed_flip = Some(0.0);
        stats.frames_since_last_speed_flip = Some(0);

        self.current_last_speed_flip_player = Some(event.player.clone());
        self.events.push(event);
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_speed_flip = false;
            stats.time_since_last_speed_flip = stats
                .last_speed_flip_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_speed_flip = stats
                .last_speed_flip_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_speed_flip_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_speed_flip = true;
            }
        }
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
        _live_play: bool,
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
        let Some(start_velocity_xy) = player.velocity().map(|velocity| velocity.truncate()) else {
            return;
        };
        let Some(start_forward_xy) = Self::forward_xy(player) else {
            return;
        };

        let rotation = quat_to_glam(&rigid_body.rotation);
        let local_angular_velocity = rigid_body
            .angular_velocity
            .as_ref()
            .map(vec_to_glam)
            .map(|angular_velocity| rotation.inverse() * angular_velocity)
            .unwrap_or(glam::Vec3::ZERO);
        let best_diagonal_score = Self::diagonal_score(local_angular_velocity);
        let forward_z = (rotation * glam::Vec3::X).z;

        self.active_candidates.insert(
            player.player_id.clone(),
            ActiveSpeedFlipCandidate {
                is_team_0: player.is_team_0,
                is_kickoff,
                kickoff_start_time,
                start_time: frame.time,
                start_frame: frame.frame_number,
                start_position: player_position.to_array(),
                end_position: player_position.to_array(),
                start_velocity_xy,
                start_forward_xy,
                start_speed,
                max_speed: start_speed,
                best_alignment,
                best_boost_alignment: Self::boost_alignment(player).unwrap_or(best_alignment),
                boost_alignment_sample_count: u32::from(player.boost_active),
                best_dodge_forward_delta: 0.0,
                best_dodge_delta_alignment: -1.0,
                dodge_acceleration_sample_count: 0,
                best_diagonal_score,
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
            candidate.best_boost_alignment = candidate.best_boost_alignment.max(boost_alignment);
            candidate.boost_alignment_sample_count += 1;
        }
        if frame.time > candidate.start_time
            && frame.time - candidate.start_time <= SPEED_FLIP_DODGE_ACCELERATION_SAMPLE_SECONDS
        {
            if let Some(velocity) = player.velocity() {
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
        if candidate.dodge_acceleration_sample_count == 0
            || candidate.best_dodge_forward_delta < SPEED_FLIP_MIN_FORWARD_DODGE_DELTA
            || candidate.best_dodge_delta_alignment < SPEED_FLIP_MIN_FORWARD_DODGE_DELTA_ALIGNMENT
        {
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

        if boost_alignment_score < 0.25 {
            return None;
        }
        if cancel_score < 0.35 || confidence < SPEED_FLIP_MIN_CONFIDENCE {
            return None;
        }

        Some(SpeedFlipEvent {
            time: candidate.start_time,
            frame: candidate.start_frame,
            player: player_id.clone(),
            is_team_0: candidate.is_team_0,
            time_since_kickoff_start,
            start_position: candidate.start_position,
            end_position: candidate.end_position,
            start_speed: candidate.start_speed,
            max_speed: candidate.max_speed,
            best_alignment: candidate.best_alignment,
            diagonal_score: candidate.best_diagonal_score,
            cancel_score,
            speed_score,
            confidence,
        })
    }

    fn finalize_candidates(&mut self, frame: &FrameInfo, force_all: bool) {
        let mut finished_candidates = Vec::new();

        for (player_id, candidate) in &self.active_candidates {
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
            let Some(candidate) = self.active_candidates.remove(&player_id) else {
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
        live_play: bool,
    ) -> SubtrActorResult<()> {
        let kickoff_approach_active = Self::kickoff_approach_active(gameplay);
        if !live_play && !kickoff_approach_active {
            self.active_candidates.clear();
            self.current_kickoff_start_time = None;
            self.kickoff_approach_active_last_frame = false;
            return Ok(());
        }

        self.begin_sample(frame);

        if kickoff_approach_active && !self.kickoff_approach_active_last_frame {
            self.reset_kickoff_state();
        }

        self.update_kickoff_start_time(frame, kickoff_approach_active, players);

        for player in &players.players {
            self.maybe_start_candidate(frame, gameplay, ball, player, live_play);
        }

        for (player_id, candidate) in &mut self.active_candidates {
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
