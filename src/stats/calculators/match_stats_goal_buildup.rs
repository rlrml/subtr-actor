use super::match_stats_goal_buildup_classify::{
    classify_goal_buildup_from_times, current_goal_buildup_attack_time, goal_buildup_zone_times,
};
use super::*;

impl MatchStatsCalculator {
    pub(super) fn prune_goal_buildup_samples(&mut self, current_time: f32) {
        self.goal_buildup_samples
            .retain(|entry| current_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS);
        self.goal_buildup_pressure_events
            .retain(|entry| current_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS);
    }

    pub(super) fn record_goal_buildup_sample(&mut self, frame: &FrameInfo, ball: &BallFrameState) {
        let Some(ball) = ball.sample() else {
            return;
        };
        if frame.dt <= 0.0 {
            return;
        }
        self.goal_buildup_samples.push(GoalBuildupSample {
            time: frame.time,
            dt: frame.dt,
            ball_y: ball.position().y,
        });
    }

    pub(super) fn record_goal_buildup_pressure_events(&mut self, events: &FrameEventsState) {
        self.goal_buildup_pressure_events.extend(
            events
                .player_stat_events
                .iter()
                .filter(|event| event.kind == PlayerStatEventKind::Shot)
                .map(|event| GoalBuildupPressureEvent {
                    time: event.time,
                    is_team_0: event.is_team_0,
                }),
        );
    }

    pub(super) fn classify_goal_buildup(
        &self,
        goal_time: f32,
        scoring_team_is_team_0: bool,
    ) -> GoalBuildupKind {
        let relevant_samples = self.relevant_goal_buildup_samples(goal_time);
        if relevant_samples.is_empty() {
            return GoalBuildupKind::Other;
        }

        let zone_times = goal_buildup_zone_times(&relevant_samples, scoring_team_is_team_0);
        let current_attack_time =
            current_goal_buildup_attack_time(&relevant_samples, scoring_team_is_team_0);
        let opponent_shot_in_lookback =
            self.opponent_shot_in_goal_buildup_lookback(goal_time, scoring_team_is_team_0);

        classify_goal_buildup_from_times(zone_times, current_attack_time, opponent_shot_in_lookback)
    }

    fn relevant_goal_buildup_samples(&self, goal_time: f32) -> Vec<&GoalBuildupSample> {
        self.goal_buildup_samples
            .iter()
            .filter(|entry| entry.time <= goal_time)
            .filter(|entry| goal_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS)
            .collect()
    }

    fn opponent_shot_in_goal_buildup_lookback(
        &self,
        goal_time: f32,
        scoring_team_is_team_0: bool,
    ) -> bool {
        self.goal_buildup_pressure_events.iter().any(|entry| {
            entry.time <= goal_time
                && goal_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS
                && entry.is_team_0 != scoring_team_is_team_0
        })
    }
}

#[derive(Default)]
pub(super) struct GoalBuildupZoneTimes {
    pub(super) defensive_half: f32,
    pub(super) defensive_third: f32,
    pub(super) offensive_half: f32,
    pub(super) offensive_third: f32,
}
