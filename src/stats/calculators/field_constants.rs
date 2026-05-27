use super::*;

pub(crate) const CAR_MAX_SPEED: f32 = 2300.0;
pub(crate) const SUPERSONIC_SPEED_THRESHOLD: f32 = 2200.0;
pub(crate) const BOOST_SPEED_THRESHOLD: f32 = 1410.0;
pub(crate) const POWERSLIDE_MAX_Z_THRESHOLD: f32 = 40.0;
pub(crate) const BALL_RADIUS_Z: f32 = 92.75;
pub(crate) const BALL_CARRY_MIN_BALL_Z: f32 = BALL_RADIUS_Z + 5.0;
pub(crate) const BALL_CARRY_MAX_BALL_Z: f32 = 600.0;
pub(crate) const BALL_CARRY_MAX_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 1.4;
pub(crate) const BALL_CARRY_MAX_VERTICAL_GAP: f32 = 220.0;
pub(crate) const BALL_CARRY_MIN_DURATION: f32 = 1.0;
pub(crate) const WALL_CONTACT_MIN_PLAYER_Z: f32 = 120.0;
pub(crate) const SIDE_WALL_CONTACT_ABS_X: f32 = 3600.0;
pub(crate) const BACK_WALL_CONTACT_ABS_Y: f32 = 5000.0;
pub(crate) const BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X: f32 = 900.0;
pub(crate) const FIELD_ZONE_BOUNDARY_Y: f32 = BOOST_PAD_SIDE_LANE_Y;
pub(crate) const DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y: f32 = 236.0;
pub(crate) const SMALL_PAD_AMOUNT_RAW: f32 = BOOST_MAX_AMOUNT * 12.0 / 100.0;
pub(crate) const BOOST_ZERO_BAND_RAW: f32 = 1.0;
pub(crate) const BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;
pub(crate) const STANDARD_PAD_MATCH_RADIUS_SMALL: f32 = 450.0;
pub(crate) const STANDARD_PAD_MATCH_RADIUS_BIG: f32 = 1000.0;
pub(crate) const BOOST_PAD_MIDFIELD_TOLERANCE_Y: f32 = 128.0;
pub(crate) const BOOST_PAD_SMALL_Z: f32 = 70.0;
pub(crate) const BOOST_PAD_BIG_Z: f32 = 73.0;
pub(crate) const BOOST_PAD_BACK_CORNER_X: f32 = 3072.0;
pub(crate) const BOOST_PAD_BACK_CORNER_Y: f32 = 4096.0;
pub(crate) const BOOST_PAD_BACK_LANE_X: f32 = 1792.0;
pub(crate) const BOOST_PAD_BACK_LANE_Y: f32 = 4184.0;
pub(crate) const BOOST_PAD_BACK_MID_X: f32 = 940.0;
pub(crate) const BOOST_PAD_BACK_MID_Y: f32 = 3308.0;
pub(crate) const BOOST_PAD_CENTER_BACK_Y: f32 = 2816.0;
pub(crate) const BOOST_PAD_SIDE_WALL_X: f32 = 3584.0;
pub(crate) const BOOST_PAD_SIDE_WALL_Y: f32 = 2484.0;
pub(crate) const BOOST_PAD_SIDE_LANE_X: f32 = 1788.0;
pub(crate) const BOOST_PAD_SIDE_LANE_Y: f32 = 2300.0;
pub(crate) const BOOST_PAD_FRONT_LANE_X: f32 = 2048.0;
pub(crate) const BOOST_PAD_FRONT_LANE_Y: f32 = 1036.0;
pub(crate) const BOOST_PAD_CENTER_X: f32 = 1024.0;
pub(crate) const BOOST_PAD_CENTER_MID_Y: f32 = 1024.0;
pub(crate) const BOOST_PAD_GOAL_LINE_Y: f32 = 4240.0;

pub(crate) fn normalized_y(is_team_0: bool, position: glam::Vec3) -> f32 {
    if is_team_0 {
        position.y
    } else {
        -position.y
    }
}

pub(crate) fn is_enemy_side(is_team_0: bool, position: glam::Vec3) -> bool {
    normalized_y(is_team_0, position) > BOOST_PAD_MIDFIELD_TOLERANCE_Y
}

pub(crate) fn player_is_on_wall(position: glam::Vec3) -> bool {
    let is_side_wall = position.x.abs() >= SIDE_WALL_CONTACT_ABS_X;
    let is_back_wall = position.y.abs() >= BACK_WALL_CONTACT_ABS_Y
        && position.x.abs() > BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X;

    position.z >= WALL_CONTACT_MIN_PLAYER_Z && (is_side_wall || is_back_wall)
}
