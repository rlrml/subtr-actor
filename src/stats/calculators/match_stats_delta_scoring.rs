use super::match_stats_delta_goal::{
    goal_after_kickoff_delta, goal_ball_air_time_delta, goal_buildup_delta,
};
use super::match_stats_delta_player_fields::{
    record_player_context_boost_leadup_delta, record_player_context_core_delta,
    record_player_context_position_delta, record_player_context_scoring_touch_delta,
};
use super::*;

pub(super) fn team_scoring_context_delta(
    current: &TeamScoringContextStats,
    previous: &TeamScoringContextStats,
) -> TeamScoringContextStats {
    TeamScoringContextStats {
        goal_after_kickoff: goal_after_kickoff_delta(
            &current.goal_after_kickoff,
            &previous.goal_after_kickoff,
        ),
        goal_buildup: goal_buildup_delta(&current.goal_buildup, &previous.goal_buildup),
        goal_ball_air_time: goal_ball_air_time_delta(
            &current.goal_ball_air_time,
            &previous.goal_ball_air_time,
        ),
    }
}

pub(super) fn player_scoring_context_delta(
    current: &PlayerScoringContextStats,
    previous: &PlayerScoringContextStats,
) -> PlayerScoringContextStats {
    let mut delta = PlayerScoringContextStats::default();
    record_player_context_core_delta(&mut delta, current, previous);
    record_player_context_boost_leadup_delta(&mut delta, current, previous);
    record_player_context_position_delta(&mut delta, current, previous);
    record_player_context_scoring_touch_delta(&mut delta, current, previous);
    delta.goal_after_kickoff =
        goal_after_kickoff_delta(&current.goal_after_kickoff, &previous.goal_after_kickoff);
    delta.goal_buildup = goal_buildup_delta(&current.goal_buildup, &previous.goal_buildup);
    delta.goal_ball_air_time =
        goal_ball_air_time_delta(&current.goal_ball_air_time, &previous.goal_ball_air_time);
    delta
}
