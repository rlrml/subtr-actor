use super::*;

const MUSTY_MAX_DODGE_TO_TOUCH_SECONDS: f32 = 0.22;
const MUSTY_MIN_PLAYER_HEIGHT: f32 = 80.0;
const MUSTY_AERIAL_HEIGHT: f32 = 180.0;
const MUSTY_MIN_FORWARD_APPROACH_SPEED: f32 = 150.0;
const MUSTY_MIN_BALL_SPEED_CHANGE: f32 = 150.0;
const MUSTY_MIN_REAR_ALIGNMENT: f32 = 0.15;
const MUSTY_MIN_TOP_ALIGNMENT: f32 = 0.10;
const MUSTY_MIN_LOCAL_Z: f32 = 5.0;
const MUSTY_MAX_LOCAL_X: f32 = 60.0;
const MUSTY_MAX_LOCAL_Y: f32 = 170.0;
const MUSTY_MIN_PITCH_RATE: f32 = 2.5;
const MUSTY_MIN_PITCH_DOMINANCE_RATIO: f32 = 1.1;
const MUSTY_MIN_DODGE_START_FORWARD_Z: f32 = -0.25;
const MUSTY_MIN_CONFIDENCE: f32 = 0.55;
const MUSTY_HIGH_CONFIDENCE: f32 = 0.80;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MustyFlickEvent {
    pub time: f32,
    pub frame: usize,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub dodge_time: f32,
    pub dodge_frame: usize,
    pub time_since_dodge: f32,
    pub confidence: f32,
    pub local_ball_position: [f32; 3],
    pub rear_alignment: f32,
    pub top_alignment: f32,
    pub forward_approach_speed: f32,
    pub pitch_rate: f32,
    pub ball_speed_change: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MustyFlickStats {
    pub count: u32,
    pub aerial_count: u32,
    pub high_confidence_count: u32,
    pub is_last_musty: bool,
    pub last_musty_time: Option<f32>,
    pub last_musty_frame: Option<usize>,
    pub time_since_last_musty: Option<f32>,
    pub frames_since_last_musty: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
}

impl MustyFlickStats {
    pub fn average_confidence(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_confidence / self.count as f32
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RecentDodgeStart {
    time: f32,
    frame: usize,
    forward_z: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MustyFlickCalculator {
    player_stats: HashMap<PlayerId, MustyFlickStats>,
    events: Vec<MustyFlickEvent>,
    recent_dodge_starts: HashMap<PlayerId, RecentDodgeStart>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    previous_ball_velocity: Option<glam::Vec3>,
    current_last_musty_player: Option<PlayerId>,
    live_play_tracker: LivePlayTracker,
}

impl MustyFlickCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, MustyFlickStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[MustyFlickEvent] {
        &self.events
    }

    fn begin_sample(&mut self, sample: &CoreSample) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_musty = false;
            stats.time_since_last_musty = stats
                .last_musty_time
                .map(|time| (sample.time - time).max(0.0));
            stats.frames_since_last_musty = stats
                .last_musty_frame
                .map(|frame| sample.frame_number.saturating_sub(frame));
        }
    }

    fn ball_speed_change(sample: &CoreSample, previous_ball_velocity: Option<glam::Vec3>) -> f32 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = sample.ball.as_ref() else {
            return 0.0;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return 0.0;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * sample.dt.max(0.0));
        let residual_linear_impulse =
            ball.velocity() - previous_ball_velocity - expected_linear_delta;
        residual_linear_impulse.length()
    }

    fn track_dodge_starts(&mut self, sample: &CoreSample) {
        for player in &sample.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if !player.dodge_active || was_dodge_active {
                continue;
            }

            let Some(rigid_body) = player.rigid_body.as_ref() else {
                continue;
            };
            let forward = quat_to_glam(&rigid_body.rotation) * glam::Vec3::X;
            self.recent_dodge_starts.insert(
                player.player_id.clone(),
                RecentDodgeStart {
                    time: sample.time,
                    frame: sample.frame_number,
                    forward_z: forward.z,
                },
            );
        }
    }

    fn prune_recent_dodge_starts(&mut self, current_time: f32) {
        self.recent_dodge_starts
            .retain(|_, dodge| current_time - dodge.time <= MUSTY_MAX_DODGE_TO_TOUCH_SECONDS);
    }

    fn musty_candidate(
        &self,
        sample: &CoreSample,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        dodge_start: RecentDodgeStart,
        ball_speed_change: f32,
    ) -> Option<MustyFlickEvent> {
        let ball = sample.ball.as_ref()?;
        let player_rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        if player_position.z < MUSTY_MIN_PLAYER_HEIGHT {
            return None;
        }

        let time_since_dodge = touch_event.time - dodge_start.time;
        if !(0.0..=MUSTY_MAX_DODGE_TO_TOUCH_SECONDS).contains(&time_since_dodge) {
            return None;
        }
        if dodge_start.forward_z < MUSTY_MIN_DODGE_START_FORWARD_Z {
            return None;
        }

        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let relative_ball_position = ball.position() - player_position;
        let to_ball = relative_ball_position.normalize_or_zero();
        if to_ball.length_squared() <= f32::EPSILON {
            return None;
        }

        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        if local_ball_position.x > MUSTY_MAX_LOCAL_X
            || local_ball_position.y.abs() > MUSTY_MAX_LOCAL_Y
            || local_ball_position.z < MUSTY_MIN_LOCAL_Z
        {
            return None;
        }

        let forward = player_rotation * glam::Vec3::X;
        let up = player_rotation * glam::Vec3::Z;
        let rear_alignment = (-forward).dot(to_ball);
        let top_alignment = up.dot(to_ball);
        if rear_alignment < MUSTY_MIN_REAR_ALIGNMENT || top_alignment < MUSTY_MIN_TOP_ALIGNMENT {
            return None;
        }

        let forward_approach_speed = player.velocity().unwrap_or(glam::Vec3::ZERO).dot(to_ball);
        if forward_approach_speed < MUSTY_MIN_FORWARD_APPROACH_SPEED {
            return None;
        }
        if ball_speed_change < MUSTY_MIN_BALL_SPEED_CHANGE {
            return None;
        }

        let angular_velocity = player_rigid_body
            .angular_velocity
            .as_ref()
            .map(vec_to_glam)
            .unwrap_or(glam::Vec3::ZERO);
        let local_angular_velocity = player_rotation.inverse() * angular_velocity;
        let pitch_rate = local_angular_velocity.y.abs();
        let other_spin = local_angular_velocity
            .x
            .abs()
            .max(local_angular_velocity.z.abs());
        if pitch_rate < MUSTY_MIN_PITCH_RATE
            || pitch_rate < other_spin * MUSTY_MIN_PITCH_DOMINANCE_RATIO
        {
            return None;
        }

        let timing_score =
            (1.0 - time_since_dodge / MUSTY_MAX_DODGE_TO_TOUCH_SECONDS).clamp(0.0, 1.0);
        let rear_score = ((rear_alignment - MUSTY_MIN_REAR_ALIGNMENT) / 0.70).clamp(0.0, 1.0);
        let top_score = ((top_alignment - MUSTY_MIN_TOP_ALIGNMENT) / 0.70).clamp(0.0, 1.0);
        let approach_score =
            ((forward_approach_speed - MUSTY_MIN_FORWARD_APPROACH_SPEED) / 900.0).clamp(0.0, 1.0);
        let pitch_score = ((pitch_rate - MUSTY_MIN_PITCH_RATE) / 8.0).clamp(0.0, 1.0);
        let pitch_dominance_ratio = if other_spin <= f32::EPSILON {
            pitch_rate
        } else {
            pitch_rate / other_spin
        };
        let pitch_dominance_score =
            ((pitch_dominance_ratio - MUSTY_MIN_PITCH_DOMINANCE_RATIO) / 2.5).clamp(0.0, 1.0);
        let impulse_score =
            ((ball_speed_change - MUSTY_MIN_BALL_SPEED_CHANGE) / 900.0).clamp(0.0, 1.0);
        let setup_score =
            ((dodge_start.forward_z - MUSTY_MIN_DODGE_START_FORWARD_Z) / 1.25).clamp(0.0, 1.0);

        let confidence = 0.17 * timing_score
            + 0.17 * rear_score
            + 0.14 * top_score
            + 0.15 * approach_score
            + 0.12 * pitch_score
            + 0.08 * pitch_dominance_score
            + 0.10 * impulse_score
            + 0.07 * setup_score;
        if confidence < MUSTY_MIN_CONFIDENCE {
            return None;
        }

        Some(MustyFlickEvent {
            time: touch_event.time,
            frame: touch_event.frame,
            player: player.player_id.clone(),
            is_team_0: player.is_team_0,
            dodge_time: dodge_start.time,
            dodge_frame: dodge_start.frame,
            time_since_dodge,
            confidence,
            local_ball_position: local_ball_position.to_array(),
            rear_alignment,
            top_alignment,
            forward_approach_speed,
            pitch_rate,
            ball_speed_change,
        })
    }

    fn apply_touch_events(&mut self, sample: &CoreSample, touch_events: &[TouchEvent]) {
        let ball_speed_change = Self::ball_speed_change(sample, self.previous_ball_velocity);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let Some(player) = sample
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
            else {
                continue;
            };
            let Some(dodge_start) = self.recent_dodge_starts.get(player_id).copied() else {
                continue;
            };
            let Some(event) =
                self.musty_candidate(sample, player, touch_event, dodge_start, ball_speed_change)
            else {
                continue;
            };

            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.count += 1;
            if player
                .position()
                .is_some_and(|position| position.z >= MUSTY_AERIAL_HEIGHT)
            {
                stats.aerial_count += 1;
            }
            if event.confidence >= MUSTY_HIGH_CONFIDENCE {
                stats.high_confidence_count += 1;
            }
            stats.is_last_musty = true;
            stats.last_musty_time = Some(event.time);
            stats.last_musty_frame = Some(event.frame);
            stats.time_since_last_musty = Some((sample.time - event.time).max(0.0));
            stats.frames_since_last_musty = Some(sample.frame_number.saturating_sub(event.frame));
            stats.last_confidence = Some(event.confidence);
            stats.best_confidence = stats.best_confidence.max(event.confidence);
            stats.cumulative_confidence += event.confidence;

            self.current_last_musty_player = Some(player_id.clone());
            self.events.push(event);
        }

        if let Some(player_id) = self.current_last_musty_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_musty = true;
            }
        }
    }

    fn reset_live_play_state(&mut self, sample: &CoreSample) {
        self.current_last_musty_player = None;
        self.recent_dodge_starts.clear();
        self.previous_dodge_active.clear();
        self.previous_ball_velocity = sample.ball.as_ref().map(BallSample::velocity);
    }

    fn on_sample_internal(
        &mut self,
        sample: &CoreSample,
        touch_events: &[TouchEvent],
    ) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            self.reset_live_play_state(sample);
            return Ok(());
        }

        self.begin_sample(sample);
        self.prune_recent_dodge_starts(sample.time);
        self.track_dodge_starts(sample);
        self.apply_touch_events(sample, touch_events);
        self.previous_ball_velocity = sample.ball.as_ref().map(BallSample::velocity);
        Ok(())
    }

    pub fn update(
        &mut self,
        sample: &CoreSample,
        touch_events: &[TouchEvent],
    ) -> SubtrActorResult<()> {
        self.on_sample_internal(sample, touch_events)
    }
}

#[cfg(test)]
#[path = "../reducers/musty_flick_test.rs"]
mod tests;
