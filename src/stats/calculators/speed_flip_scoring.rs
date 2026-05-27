use super::*;

impl SpeedFlipCalculator {
    pub(super) fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }
        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    pub(super) fn diagonal_score(local_angular_velocity: glam::Vec3) -> f32 {
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

    pub(super) fn forward_speed_alignment(player: &PlayerSample) -> Option<f32> {
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

    pub(super) fn forward_xy(player: &PlayerSample) -> Option<glam::Vec2> {
        let rigid_body = player.rigid_body.as_ref()?;
        let forward_xy = (quat_to_glam(&rigid_body.rotation) * glam::Vec3::X)
            .truncate()
            .normalize_or_zero();
        (forward_xy.length_squared() > f32::EPSILON).then_some(forward_xy)
    }

    pub(super) fn boost_alignment(player: &PlayerSample) -> Option<f32> {
        player
            .boost_active
            .then(|| Self::forward_speed_alignment(player))
            .flatten()
    }

    pub(super) fn candidate_alignment(
        _ball: &BallFrameState,
        player: &PlayerSample,
        _is_kickoff: bool,
    ) -> Option<f32> {
        Self::forward_speed_alignment(player)
    }
}
