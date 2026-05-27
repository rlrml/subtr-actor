use super::builtins_snapshot_frame_core_player::CorePlayerStatsSnapshot;
use super::*;

impl From<&CorePlayerStats> for CorePlayerStatsSnapshot {
    fn from(stats: &CorePlayerStats) -> Self {
        let scoring = &stats.scoring_context;
        Self {
            score: stats.score,
            goals: stats.goals,
            assists: stats.assists,
            saves: stats.saves,
            shots: stats.shots,
            goals_conceded_while_last_defender: scoring.goals_conceded_while_last_defender,
            goals_for_while_most_back: scoring.goals_for_while_most_back,
            goals_against_while_most_back: scoring.goals_against_while_most_back,
            goal_against_boost_sample_count: scoring.goal_against_boost_sample_count,
            cumulative_boost_on_goals_against: scoring.cumulative_boost_on_goals_against,
            average_boost_on_goals_against: stats.average_boost_on_goals_against(),
            last_boost_on_goal_against: scoring.last_boost_on_goal_against,
            goal_against_boost_leadup_sample_count: scoring.goal_against_boost_leadup_sample_count,
            cumulative_average_boost_in_goal_against_leadup: scoring
                .cumulative_average_boost_in_goal_against_leadup,
            cumulative_min_boost_in_goal_against_leadup: scoring
                .cumulative_min_boost_in_goal_against_leadup,
            average_boost_in_goal_against_leadup: stats.average_boost_in_goal_against_leadup(),
            average_min_boost_in_goal_against_leadup: stats
                .average_min_boost_in_goal_against_leadup(),
            last_average_boost_in_goal_against_leadup: scoring
                .last_average_boost_in_goal_against_leadup,
            last_min_boost_in_goal_against_leadup: scoring.last_min_boost_in_goal_against_leadup,
            goal_against_position_sample_count: scoring.goal_against_position_sample_count,
            cumulative_goal_against_position_x: scoring.cumulative_goal_against_position_x,
            cumulative_goal_against_position_y: scoring.cumulative_goal_against_position_y,
            cumulative_goal_against_position_z: scoring.cumulative_goal_against_position_z,
            average_goal_against_position_x: stats.average_goal_against_position_x(),
            average_goal_against_position_y: stats.average_goal_against_position_y(),
            average_goal_against_position_z: stats.average_goal_against_position_z(),
            last_goal_against_position: scoring.last_goal_against_position,
            scoring_goal_last_touch_position_sample_count: scoring
                .scoring_goal_last_touch_position_sample_count,
            cumulative_scoring_goal_last_touch_position_x: scoring
                .cumulative_scoring_goal_last_touch_position_x,
            cumulative_scoring_goal_last_touch_position_y: scoring
                .cumulative_scoring_goal_last_touch_position_y,
            cumulative_scoring_goal_last_touch_position_z: scoring
                .cumulative_scoring_goal_last_touch_position_z,
            average_scoring_goal_last_touch_position_x: stats
                .average_scoring_goal_last_touch_position_x(),
            average_scoring_goal_last_touch_position_y: stats
                .average_scoring_goal_last_touch_position_y(),
            average_scoring_goal_last_touch_position_z: stats
                .average_scoring_goal_last_touch_position_z(),
            last_scoring_goal_last_touch_position: scoring.last_scoring_goal_last_touch_position,
            kickoff_goal_count: scoring.goal_after_kickoff.kickoff_goal_count,
            short_goal_count: scoring.goal_after_kickoff.short_goal_count,
            medium_goal_count: scoring.goal_after_kickoff.medium_goal_count,
            long_goal_count: scoring.goal_after_kickoff.long_goal_count,
            goal_times: scoring.goal_after_kickoff.goal_times().to_vec(),
            goal_ball_air_time_sample_count: scoring
                .goal_ball_air_time
                .goal_ball_air_time_sample_count,
            cumulative_goal_ball_air_time: scoring.goal_ball_air_time.cumulative_goal_ball_air_time,
            average_goal_ball_air_time: stats.average_goal_ball_air_time(),
            median_goal_ball_air_time: stats.median_goal_ball_air_time(),
            last_goal_ball_air_time: scoring.goal_ball_air_time.last_goal_ball_air_time,
            goal_ball_air_times: scoring.goal_ball_air_time.goal_ball_air_times().to_vec(),
            counter_attack_goal_count: scoring.goal_buildup.counter_attack_goal_count,
            sustained_pressure_goal_count: scoring.goal_buildup.sustained_pressure_goal_count,
            other_buildup_goal_count: scoring.goal_buildup.other_buildup_goal_count,
        }
    }
}
