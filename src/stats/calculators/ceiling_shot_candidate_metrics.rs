use super::*;

pub(crate) struct CeilingShotCandidateMetrics {
    pub(super) time_since_ceiling_contact: f32,
    pub(super) touch_position: [f32; 3],
    pub(super) local_ball_position: [f32; 3],
    pub(super) separation_from_ceiling: f32,
    pub(super) forward_alignment: f32,
    pub(super) forward_approach_speed: f32,
    pub(super) ball_speed_change: f32,
    pub(super) confidence: f32,
}

impl CeilingShotCandidateMetrics {
    pub(super) fn new(
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        recent_contact: RecentCeilingContact,
        ball_speed_change: f32,
    ) -> Option<Self> {
        let ball = ball.sample()?;
        let player_position = player.position()?;
        let player_rigid_body = player.rigid_body.as_ref()?;
        let ball_position = ball.position();

        if player_position.z < CEILING_SHOT_MIN_PLAYER_HEIGHT
            || ball_position.z < CEILING_SHOT_MIN_BALL_HEIGHT
        {
            return None;
        }

        let time_since_ceiling_contact = touch_event.time - recent_contact.time;
        if !(0.0..=CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS)
            .contains(&time_since_ceiling_contact)
        {
            return None;
        }

        let separation_from_ceiling = SOCCAR_CEILING_Z - player_position.z;
        if separation_from_ceiling < CEILING_SHOT_MIN_TOUCH_SEPARATION {
            return None;
        }

        let relative_ball_position = ball_position - player_position;
        if relative_ball_position.length_squared() <= f32::EPSILON {
            return None;
        }

        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        if local_ball_position.x < -120.0
            || local_ball_position.y.abs() > 260.0
            || local_ball_position.z.abs() > 240.0
        {
            return None;
        }

        let to_ball = relative_ball_position.normalize_or_zero();
        let forward = player_rotation * glam::Vec3::X;
        let forward_alignment = forward.dot(to_ball);
        if forward_alignment < CEILING_SHOT_MIN_FORWARD_ALIGNMENT {
            return None;
        }

        let forward_approach_speed = player.velocity().unwrap_or(glam::Vec3::ZERO).dot(to_ball);
        if forward_approach_speed < CEILING_SHOT_MIN_FORWARD_APPROACH_SPEED {
            return None;
        }
        if ball_speed_change < CEILING_SHOT_MIN_BALL_SPEED_CHANGE {
            return None;
        }

        let confidence = Self::confidence(
            time_since_ceiling_contact,
            separation_from_ceiling,
            player_position.z.max(ball_position.z),
            forward_alignment,
            forward_approach_speed,
            ball_speed_change,
            recent_contact.roof_alignment,
        );
        if confidence < CEILING_SHOT_MIN_CONFIDENCE {
            return None;
        }

        Some(Self {
            time_since_ceiling_contact,
            touch_position: ball_position.to_array(),
            local_ball_position: local_ball_position.to_array(),
            separation_from_ceiling,
            forward_alignment,
            forward_approach_speed,
            ball_speed_change,
            confidence,
        })
    }
}
