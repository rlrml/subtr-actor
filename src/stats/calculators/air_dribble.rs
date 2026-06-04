use super::*;

const AIR_DRIBBLE_MIN_BALL_Z: f32 = 300.0;
pub(crate) const AIR_DRIBBLE_MIN_PLAYER_Z: f32 = 100.0;
const AIR_DRIBBLE_MAX_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 3.0;
const AIR_DRIBBLE_MAX_ABOVE_CAR_GAP: f32 = 360.0;
const AIR_DRIBBLE_MAX_BELOW_CAR_GAP: f32 = 100.0;
pub(crate) const AIR_DRIBBLE_MIN_DURATION: f32 = 0.65;
const AIR_DRIBBLE_MIN_TOUCHES: u32 = 3;
const AIR_DRIBBLE_MIN_AIR_TOUCHES: u32 = 2;
const WALL_TAKEOFF_MIN_Z: f32 = 120.0;
const SIDE_WALL_START_ABS_X: f32 = 3200.0;
const BACK_WALL_START_ABS_Y: f32 = 4600.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum AirDribbleOrigin {
    GroundToAir,
    WallToAir,
}

impl AirDribbleOrigin {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::GroundToAir => "ground_to_air",
            Self::WallToAir => "wall_to_air",
        }
    }
}

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
}

#[cfg(test)]
#[path = "air_dribble_tests.rs"]
mod tests;
