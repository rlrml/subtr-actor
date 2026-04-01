use super::*;

const SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS: f32 = 1.1;
const SPEED_FLIP_EVALUATION_SECONDS: f32 = 0.22;
const SPEED_FLIP_MAX_CANDIDATE_SECONDS: f32 = 0.4;
const SPEED_FLIP_MAX_GROUND_Z: f32 = 80.0;
const SPEED_FLIP_MIN_START_SPEED: f32 = 700.0;
const SPEED_FLIP_MIN_ALIGNMENT: f32 = 0.72;
const SPEED_FLIP_MIN_CONFIDENCE: f32 = 0.45;
const SPEED_FLIP_HIGH_CONFIDENCE: f32 = 0.75;
const SPEED_FLIP_TARGET_DIAGONAL_RATIO: f32 = 0.42;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpeedFlipEvent {
    pub time: f32,
    pub frame: usize,
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
}

impl SpeedFlipStats {
    pub fn average_quality(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_quality / self.count as f32
        }
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
    start_speed: f32,
    max_speed: f32,
    best_alignment: f32,
    best_diagonal_score: f32,
    min_forward_z: f32,
    latest_forward_z: f32,
    latest_time: f32,
    latest_frame: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpeedFlipReducer {
    player_stats: HashMap<PlayerId, SpeedFlipStats>,
    events: Vec<SpeedFlipEvent>,
    active_candidates: HashMap<PlayerId, ActiveSpeedFlipCandidate>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    kickoff_approach_active_last_frame: bool,
    current_kickoff_start_time: Option<f32>,
    current_last_speed_flip_player: Option<PlayerId>,
    live_play_tracker: LivePlayTracker,
}

impl SpeedFlipReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, SpeedFlipStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[SpeedFlipEvent] {
        &self.events
    }

    fn kickoff_approach_active(sample: &StatsSample) -> bool {
        sample.is_live_play() && sample.ball_has_been_hit == Some(false)
    }

    fn player_by_id<'a>(sample: &'a StatsSample, player_id: &PlayerId) -> Option<&'a PlayerSample> {
        sample
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
        if pitch_rate <= f32::EPSILON {
            return 0.0;
        }

        let pitch_score = Self::normalize_score(pitch_rate, 3.5, 8.5);
        let ratio = side_spin / pitch_rate;
        let ratio_score = 1.0
            - ((ratio - SPEED_FLIP_TARGET_DIAGONAL_RATIO).abs() / SPEED_FLIP_TARGET_DIAGONAL_RATIO)
                .clamp(0.0, 1.0);

        pitch_score * ratio_score
    }

    fn horizontal_alignment_to_target(
        player: &PlayerSample,
        target_position: glam::Vec3,
    ) -> Option<f32> {
        let velocity = player.velocity()?;
        let player_position = player.position()?;
        let to_target = target_position - player_position;
        let velocity_xy = velocity.truncate().normalize_or_zero();
        let to_target_xy = to_target.truncate().normalize_or_zero();
        if velocity_xy.length_squared() <= f32::EPSILON
            || to_target_xy.length_squared() <= f32::EPSILON
        {
            return None;
        }

        Some(velocity_xy.dot(to_target_xy))
    }

    fn kickoff_alignment_target(sample: &StatsSample) -> glam::Vec3 {
        sample
            .ball
            .as_ref()
            .map(BallSample::position)
            .unwrap_or(glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z))
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

    fn candidate_alignment(
        sample: &StatsSample,
        player: &PlayerSample,
        is_kickoff: bool,
    ) -> Option<f32> {
        if is_kickoff {
            Self::horizontal_alignment_to_target(player, Self::kickoff_alignment_target(sample))
        } else {
            Self::forward_speed_alignment(player)
        }
    }

    fn apply_event(&mut self, event: SpeedFlipEvent) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_speed_flip = false;
        }

        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        if event.confidence >= SPEED_FLIP_HIGH_CONFIDENCE {
            stats.high_confidence_count += 1;
        }
        stats.is_last_speed_flip = true;
        stats.last_speed_flip_time = Some(event.time);
        stats.last_speed_flip_frame = Some(event.frame);
        stats.time_since_last_speed_flip = Some(0.0);
        stats.frames_since_last_speed_flip = Some(0);
        stats.last_quality = Some(event.confidence);
        stats.best_quality = stats.best_quality.max(event.confidence);
        stats.cumulative_quality += event.confidence;

        self.current_last_speed_flip_player = Some(event.player.clone());
        self.events.push(event);
    }

    fn begin_sample(&mut self, sample: &StatsSample) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_speed_flip = false;
            stats.time_since_last_speed_flip = stats
                .last_speed_flip_time
                .map(|time| (sample.time - time).max(0.0));
            stats.frames_since_last_speed_flip = stats
                .last_speed_flip_frame
                .map(|frame| sample.frame_number.saturating_sub(frame));
        }

        if let Some(player_id) = self.current_last_speed_flip_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_speed_flip = true;
            }
        }
    }

    fn reset_kickoff_state(&mut self, sample: &StatsSample) {
        self.active_candidates.clear();
        self.current_kickoff_start_time = Some(sample.time);
    }

    fn maybe_start_candidate(&mut self, sample: &StatsSample, player: &PlayerSample) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        let is_kickoff = Self::kickoff_approach_active(sample);
        let kickoff_start_time = if is_kickoff {
            let Some(kickoff_start_time) = self.current_kickoff_start_time else {
                return;
            };
            if sample.time - kickoff_start_time > SPEED_FLIP_MAX_START_AFTER_KICKOFF_SECONDS {
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
        if start_speed < SPEED_FLIP_MIN_START_SPEED {
            return;
        }

        let Some(best_alignment) = Self::candidate_alignment(sample, player, is_kickoff) else {
            return;
        };
        if best_alignment < SPEED_FLIP_MIN_ALIGNMENT {
            return;
        }

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
                start_time: sample.time,
                start_frame: sample.frame_number,
                start_position: player_position.to_array(),
                end_position: player_position.to_array(),
                start_speed,
                max_speed: start_speed,
                best_alignment,
                best_diagonal_score,
                min_forward_z: forward_z,
                latest_forward_z: forward_z,
                latest_time: sample.time,
                latest_frame: sample.frame_number,
            },
        );
    }

    fn update_candidate(
        candidate: &mut ActiveSpeedFlipCandidate,
        sample: &StatsSample,
        player: &PlayerSample,
    ) {
        let Some(rigid_body) = player.rigid_body.as_ref() else {
            return;
        };

        if let Some(player_position) = player.position() {
            candidate.end_position = player_position.to_array();
        }
        candidate.max_speed = candidate.max_speed.max(player.speed().unwrap_or(0.0));
        if let Some(alignment) = Self::candidate_alignment(sample, player, candidate.is_kickoff) {
            candidate.best_alignment = candidate.best_alignment.max(alignment);
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
        candidate.latest_time = sample.time;
        candidate.latest_frame = sample.frame_number;
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
        let cancel_score = 0.35 * Self::normalize_score(-candidate.min_forward_z, 0.22, 0.75)
            + 0.40 * Self::normalize_score(cancel_recovery, 0.18, 0.7)
            + 0.25 * Self::normalize_score(candidate.latest_forward_z, -0.55, -0.05);
        let speed_score = 0.55 * Self::normalize_score(candidate.max_speed, 1450.0, 1900.0)
            + 0.45
                * Self::normalize_score(candidate.max_speed - candidate.start_speed, 180.0, 650.0);
        let alignment_score = Self::normalize_score(candidate.best_alignment, 0.78, 0.98);
        let confidence = 0.30 * candidate.best_diagonal_score
            + 0.30 * cancel_score
            + 0.20 * speed_score
            + 0.15 * alignment_score
            + 0.05 * timeliness_score;

        if cancel_score < 0.45 || confidence < SPEED_FLIP_MIN_CONFIDENCE {
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

    fn finalize_candidates(&mut self, sample: &StatsSample, force_all: bool) {
        let mut finished_candidates = Vec::new();

        for (player_id, candidate) in &self.active_candidates {
            let duration = sample.time - candidate.start_time;
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
}

impl StatsReducer for SpeedFlipReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            self.active_candidates.clear();
            self.current_kickoff_start_time = None;
            self.kickoff_approach_active_last_frame = false;
            return Ok(());
        }

        self.begin_sample(sample);

        let kickoff_approach_active = Self::kickoff_approach_active(sample);
        if kickoff_approach_active && !self.kickoff_approach_active_last_frame {
            self.reset_kickoff_state(sample);
        }

        for player in &sample.players {
            self.maybe_start_candidate(sample, player);
        }

        for (player_id, candidate) in &mut self.active_candidates {
            let Some(player) = Self::player_by_id(sample, player_id) else {
                continue;
            };
            Self::update_candidate(candidate, sample, player);
        }

        self.finalize_candidates(sample, false);

        self.active_candidates.retain(|_, candidate| {
            sample.time - candidate.start_time <= SPEED_FLIP_MAX_CANDIDATE_SECONDS
        });

        if !kickoff_approach_active {
            self.current_kickoff_start_time = None;
        }

        self.kickoff_approach_active_last_frame = kickoff_approach_active;
        Ok(())
    }
}

#[cfg(test)]
#[path = "speed_flip_test.rs"]
mod tests;
