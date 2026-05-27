use super::*;

pub(crate) struct AirDribblePolicy;

impl AirDribblePolicy {
    pub(crate) fn is_sample(
        player_position: glam::Vec3,
        ball_position: glam::Vec3,
        horizontal_gap: f32,
        vertical_gap: f32,
    ) -> bool {
        ball_position.z >= AIR_DRIBBLE_MIN_BALL_Z
            && player_position.z >= AIR_DRIBBLE_MIN_PLAYER_Z
            && !player_is_on_wall(player_position)
            && horizontal_gap <= AIR_DRIBBLE_MAX_HORIZONTAL_GAP
            && (-AIR_DRIBBLE_MAX_BELOW_CAR_GAP..=AIR_DRIBBLE_MAX_ABOVE_CAR_GAP)
                .contains(&vertical_gap)
    }

    pub(crate) fn is_air_touch_position(player_position: glam::Vec3) -> bool {
        player_position.z > PLAYER_GROUND_Z_THRESHOLD && !player_is_on_wall(player_position)
    }

    pub(crate) fn kind_requires_airborne(kind: BallCarryKind) -> bool {
        kind == BallCarryKind::AirDribble
    }

    pub(crate) fn is_valid_sequence(
        sequence: &CompletedBallControlSequence<BallCarryKind>,
    ) -> bool {
        sequence.kind != BallCarryKind::AirDribble
            || (sequence.touch_count >= AIR_DRIBBLE_MIN_TOUCHES
                && sequence.air_touch_count >= AIR_DRIBBLE_MIN_AIR_TOUCHES)
    }

    pub(crate) fn origin(start_position: glam::Vec3) -> AirDribbleOrigin {
        if start_position.z >= WALL_TAKEOFF_MIN_Z
            && (start_position.x.abs() >= SIDE_WALL_START_ABS_X
                || start_position.y.abs() >= BACK_WALL_START_ABS_Y)
        {
            AirDribbleOrigin::WallToAir
        } else {
            AirDribbleOrigin::GroundToAir
        }
    }

    pub(crate) fn apply_event(stats: &mut AirDribbleStats, event: &BallCarryEvent) {
        stats.record_event(event);
        stats.total_time += event.duration;
        stats.total_straight_line_distance += event.straight_line_distance;
        stats.total_path_distance += event.path_distance;
        stats.longest_time = stats.longest_time.max(event.duration);
        stats.furthest_distance = stats.furthest_distance.max(event.straight_line_distance);
        stats.fastest_speed = stats.fastest_speed.max(event.average_speed);
        stats.speed_sum += event.average_speed;
        stats.average_horizontal_gap_sum += event.average_horizontal_gap;
        stats.average_vertical_gap_sum += event.average_vertical_gap;
        stats.total_touch_count += event.touch_count;
        stats.max_touch_count = stats.max_touch_count.max(event.touch_count);
    }
}
