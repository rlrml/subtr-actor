use super::*;

pub(crate) fn add_role_time(stats: &mut RotationPlayerStats, role_state: RoleState, dt: f32) {
    match role_state {
        RoleState::FirstMan => stats.time_first_man += dt,
        RoleState::SecondMan => stats.time_second_man += dt,
        RoleState::ThirdMan => stats.time_third_man += dt,
        RoleState::Ambiguous => stats.time_ambiguous_role += dt,
        RoleState::Unknown => {}
    }
}

pub(crate) fn add_depth_time(
    stats: &mut RotationPlayerStats,
    depth_state: PlayDepthState,
    dt: f32,
) {
    match depth_state {
        PlayDepthState::BehindPlay => stats.time_behind_play += dt,
        PlayDepthState::LevelWithPlay => stats.time_level_with_play += dt,
        PlayDepthState::AheadOfPlay => stats.time_ahead_of_play += dt,
        PlayDepthState::Unknown => {}
    }
}
