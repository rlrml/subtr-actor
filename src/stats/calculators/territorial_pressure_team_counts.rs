use super::*;

pub(super) struct TeamCounts {
    pub(super) session_count: u32,
    pub(super) opponent_session_count: u32,
    pub(super) session_time: f32,
    pub(super) opponent_session_time: f32,
    pub(super) offensive_half_time: f32,
    pub(super) offensive_third_time: f32,
    pub(super) longest_session_time: f32,
    pub(super) opponent_longest_session_time: f32,
}

pub(super) fn territorial_pressure_team_counts(
    stats: &TerritorialPressureStats,
    is_team_zero: bool,
) -> TeamCounts {
    if is_team_zero {
        TeamCounts {
            session_count: stats.team_zero_session_count,
            opponent_session_count: stats.team_one_session_count,
            session_time: stats.team_zero_session_time,
            opponent_session_time: stats.team_one_session_time,
            offensive_half_time: stats.team_zero_offensive_half_time,
            offensive_third_time: stats.team_zero_offensive_third_time,
            longest_session_time: stats.team_zero_longest_session_time,
            opponent_longest_session_time: stats.team_one_longest_session_time,
        }
    } else {
        TeamCounts {
            session_count: stats.team_one_session_count,
            opponent_session_count: stats.team_zero_session_count,
            session_time: stats.team_one_session_time,
            opponent_session_time: stats.team_zero_session_time,
            offensive_half_time: stats.team_one_offensive_half_time,
            offensive_third_time: stats.team_one_offensive_third_time,
            longest_session_time: stats.team_one_longest_session_time,
            opponent_longest_session_time: stats.team_zero_longest_session_time,
        }
    }
}
