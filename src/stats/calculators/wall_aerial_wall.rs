use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum WallAerialWall {
    Side,
    Back,
}

impl WallAerialWall {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Side => "side",
            Self::Back => "back",
        }
    }
}

pub(crate) fn wall_aerial_wall_for_position(position: glam::Vec3) -> Option<WallAerialWall> {
    if position.z < WALL_CONTACT_MIN_PLAYER_Z {
        return None;
    }
    if position.y.abs() >= BACK_WALL_CONTACT_ABS_Y
        && position.x.abs() > BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X
    {
        return Some(WallAerialWall::Back);
    }
    if position.x.abs() >= SIDE_WALL_CONTACT_ABS_X {
        return Some(WallAerialWall::Side);
    }
    None
}

pub(super) fn wall_aerial_setup_wall_for_position(position: glam::Vec3) -> Option<WallAerialWall> {
    if position.z < WALL_CONTACT_MIN_PLAYER_Z {
        return None;
    }
    if position.y.abs() >= WALL_AERIAL_SETUP_BACK_WALL_START_ABS_Y
        && position.x.abs() > BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X
    {
        return Some(WallAerialWall::Back);
    }
    if position.x.abs() >= WALL_AERIAL_SETUP_SIDE_WALL_START_ABS_X {
        return Some(WallAerialWall::Side);
    }
    None
}

pub(crate) fn wall_aerial_normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
    if max_value <= min_value {
        return 0.0;
    }
    ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
}

pub(crate) fn wall_aerial_goal_alignment(
    is_team_0: bool,
    ball_position: glam::Vec3,
    ball_velocity: glam::Vec3,
) -> f32 {
    const GOAL_CENTER_Y: f32 = 5120.0;

    let target_y = if is_team_0 {
        GOAL_CENTER_Y
    } else {
        -GOAL_CENTER_Y
    };
    let goal_direction =
        (glam::Vec3::new(0.0, target_y, ball_position.z) - ball_position).normalize_or_zero();
    goal_direction.dot(ball_velocity.normalize_or_zero())
}
