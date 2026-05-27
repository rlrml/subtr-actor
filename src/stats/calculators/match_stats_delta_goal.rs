use super::match_stats_delta_helpers::{optional_delta, sample_delta};
use super::*;

pub(super) fn goal_after_kickoff_delta(
    current: &GoalAfterKickoffStats,
    previous: &GoalAfterKickoffStats,
) -> GoalAfterKickoffStats {
    GoalAfterKickoffStats {
        kickoff_goal_count: current
            .kickoff_goal_count
            .saturating_sub(previous.kickoff_goal_count),
        short_goal_count: current
            .short_goal_count
            .saturating_sub(previous.short_goal_count),
        medium_goal_count: current
            .medium_goal_count
            .saturating_sub(previous.medium_goal_count),
        long_goal_count: current
            .long_goal_count
            .saturating_sub(previous.long_goal_count),
        goal_times: sample_delta(&current.goal_times, &previous.goal_times),
    }
}

pub(super) fn goal_buildup_delta(
    current: &GoalBuildupStats,
    previous: &GoalBuildupStats,
) -> GoalBuildupStats {
    GoalBuildupStats {
        counter_attack_goal_count: current
            .counter_attack_goal_count
            .saturating_sub(previous.counter_attack_goal_count),
        sustained_pressure_goal_count: current
            .sustained_pressure_goal_count
            .saturating_sub(previous.sustained_pressure_goal_count),
        other_buildup_goal_count: current
            .other_buildup_goal_count
            .saturating_sub(previous.other_buildup_goal_count),
    }
}

pub(super) fn goal_ball_air_time_delta(
    current: &GoalBallAirTimeStats,
    previous: &GoalBallAirTimeStats,
) -> GoalBallAirTimeStats {
    GoalBallAirTimeStats {
        goal_ball_air_time_sample_count: current
            .goal_ball_air_time_sample_count
            .saturating_sub(previous.goal_ball_air_time_sample_count),
        cumulative_goal_ball_air_time: current.cumulative_goal_ball_air_time
            - previous.cumulative_goal_ball_air_time,
        last_goal_ball_air_time: optional_delta(
            current.last_goal_ball_air_time,
            previous.last_goal_ball_air_time,
        ),
        goal_ball_air_times: sample_delta(
            &current.goal_ball_air_times,
            &previous.goal_ball_air_times,
        ),
    }
}
