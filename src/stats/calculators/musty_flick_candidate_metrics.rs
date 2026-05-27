use super::*;
use super::musty_flick_candidate_confidence::MustyFlickConfidenceInputs;

pub(crate) struct MustyFlickCandidateMetrics {
    pub(super) aerial: bool,
    pub(super) time_since_dodge: f32,
    pub(super) confidence: f32,
    pub(super) local_ball_position: [f32; 3],
    pub(super) rear_alignment: f32,
    pub(super) top_alignment: f32,
    pub(super) forward_approach_speed: f32,
    pub(super) pitch_rate: f32,
    pub(super) ball_speed_change: f32,
}

impl MustyFlickCandidateMetrics {
    pub(super) fn new(
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        dodge_start: RecentDodgeStart,
        ball_speed_change: f32,
    ) -> Option<Self> {
        let ball = ball.sample()?;
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
        if forward_approach_speed < MUSTY_MIN_FORWARD_APPROACH_SPEED
            || ball_speed_change < MUSTY_MIN_BALL_SPEED_CHANGE
        {
            return None;
        }

        let spin = MustyFlickCandidateSpin::new(player_rigid_body, player_rotation)?;

        let confidence = Self::confidence(MustyFlickConfidenceInputs {
            time_since_dodge,
            rear_alignment,
            top_alignment,
            forward_approach_speed,
            pitch_rate: spin.pitch_rate,
            other_spin: spin.other_spin,
            ball_speed_change,
            dodge_start_forward_z: dodge_start.forward_z,
        });
        if confidence < MUSTY_MIN_CONFIDENCE {
            return None;
        }

        Some(Self {
            aerial: player_position.z >= MUSTY_AERIAL_HEIGHT,
            time_since_dodge,
            confidence,
            local_ball_position: local_ball_position.to_array(),
            rear_alignment,
            top_alignment,
            forward_approach_speed,
            pitch_rate: spin.pitch_rate,
            ball_speed_change,
        })
    }
}
