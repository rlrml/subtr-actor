use super::*;

#[derive(Serialize)]
pub(super) struct CoreTeamStatsSnapshot {
    score: i32,
    goals: i32,
    assists: i32,
    saves: i32,
    shots: i32,
    kickoff_goal_count: u32,
    short_goal_count: u32,
    medium_goal_count: u32,
    long_goal_count: u32,
    goal_times: Vec<f32>,
    goal_ball_air_time_sample_count: u32,
    cumulative_goal_ball_air_time: f32,
    average_goal_ball_air_time: f32,
    median_goal_ball_air_time: f32,
    last_goal_ball_air_time: Option<f32>,
    goal_ball_air_times: Vec<f32>,
    counter_attack_goal_count: u32,
    sustained_pressure_goal_count: u32,
    other_buildup_goal_count: u32,
}

impl From<CoreTeamStats> for CoreTeamStatsSnapshot {
    fn from(stats: CoreTeamStats) -> Self {
        let scoring = &stats.scoring_context;
        Self {
            score: stats.score,
            goals: stats.goals,
            assists: stats.assists,
            saves: stats.saves,
            shots: stats.shots,
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
