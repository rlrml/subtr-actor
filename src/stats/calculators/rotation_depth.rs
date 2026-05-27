use super::*;

pub(crate) fn play_depth_state(
    is_team_0: bool,
    player_position: glam::Vec3,
    ball_position: glam::Vec3,
    margin: f32,
) -> PlayDepthState {
    let player_y = normalized_y(is_team_0, player_position);
    let ball_y = normalized_y(is_team_0, ball_position);
    let delta = player_y - ball_y;
    if delta < -margin {
        PlayDepthState::BehindPlay
    } else if delta > margin {
        PlayDepthState::AheadOfPlay
    } else {
        PlayDepthState::LevelWithPlay
    }
}
