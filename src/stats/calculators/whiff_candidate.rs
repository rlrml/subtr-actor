use super::*;

impl WhiffCalculator {
    pub(super) fn whiff_candidate(
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
            closest_approach_distance: distance,
            forward_alignment,
            approach_speed,
            dodge_active: player.dodge_active,
            aerial: player_position.z > POWERSLIDE_MAX_Z_THRESHOLD,
        })
    }
}
