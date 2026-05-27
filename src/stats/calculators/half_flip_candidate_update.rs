use super::*;

impl HalfFlipCalculator {
    pub(super) fn update_candidate(
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
            update_forward_scores(candidate, forward, velocity_direction);
        }

        candidate.latest_time = frame.time;
        candidate.latest_frame = frame.frame_number;
    }
}

fn update_forward_scores(
    candidate: &mut ActiveHalfFlipCandidate,
    forward: glam::Vec3,
    velocity_direction: glam::Vec2,
) {
    let forward_xy = forward.truncate().normalize_or_zero();
    if forward_xy.length_squared() <= f32::EPSILON {
        return;
    }
    candidate.best_forward_reversal = candidate
        .best_forward_reversal
        .max((-candidate.start_forward_xy.dot(forward_xy)).clamp(-1.0, 1.0));
    if velocity_direction.length_squared() > f32::EPSILON {
        candidate.best_reorientation_alignment = candidate
            .best_reorientation_alignment
            .max(forward_xy.dot(velocity_direction));
    }
}
