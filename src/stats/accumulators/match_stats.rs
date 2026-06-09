use super::*;

const GOAL_AFTER_KICKOFF_BUCKET_KICKOFF_MAX_SECONDS: f32 = 10.0;
const GOAL_AFTER_KICKOFF_BUCKET_SHORT_MAX_SECONDS: f32 = 20.0;
const GOAL_AFTER_KICKOFF_BUCKET_MEDIUM_MAX_SECONDS: f32 = 40.0;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalAfterKickoffStats {
    pub kickoff_goal_count: u32,
    pub short_goal_count: u32,
    pub medium_goal_count: u32,
    pub long_goal_count: u32,
    #[serde(default, skip_serializing)]
    pub(crate) goal_times: Vec<f32>,
}

impl GoalAfterKickoffStats {
    pub fn goal_times(&self) -> &[f32] {
        &self.goal_times
    }

    pub fn record_goal(&mut self, time_after_kickoff: f32) {
        let clamped_time = time_after_kickoff.max(0.0);
        self.goal_times.push(clamped_time);
        self.goal_times.sort_by(|left, right| left.total_cmp(right));
        if clamped_time < GOAL_AFTER_KICKOFF_BUCKET_KICKOFF_MAX_SECONDS {
            self.kickoff_goal_count += 1;
        } else if clamped_time < GOAL_AFTER_KICKOFF_BUCKET_SHORT_MAX_SECONDS {
            self.short_goal_count += 1;
        } else if clamped_time < GOAL_AFTER_KICKOFF_BUCKET_MEDIUM_MAX_SECONDS {
            self.medium_goal_count += 1;
        } else {
            self.long_goal_count += 1;
        }
    }

    pub fn average_goal_time_after_kickoff(&self) -> f32 {
        if self.goal_times.is_empty() {
            0.0
        } else {
            self.goal_times.iter().sum::<f32>() / self.goal_times.len() as f32
        }
    }

    pub fn median_goal_time_after_kickoff(&self) -> f32 {
        if self.goal_times.is_empty() {
            return 0.0;
        }

        let mut sorted_times = self.goal_times.clone();
        sorted_times.sort_by(|a, b| a.total_cmp(b));
        let midpoint = sorted_times.len() / 2;
        if sorted_times.len().is_multiple_of(2) {
            (sorted_times[midpoint - 1] + sorted_times[midpoint]) * 0.5
        } else {
            sorted_times[midpoint]
        }
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        self.kickoff_goal_count += other.kickoff_goal_count;
        self.short_goal_count += other.short_goal_count;
        self.medium_goal_count += other.medium_goal_count;
        self.long_goal_count += other.long_goal_count;
        self.goal_times.extend(other.goal_times.iter().copied());
        self.goal_times.sort_by(|left, right| left.total_cmp(right));
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalBallAirTimeStats {
    pub goal_ball_air_time_sample_count: u32,
    pub cumulative_goal_ball_air_time: f32,
    pub last_goal_ball_air_time: Option<f32>,
    #[serde(default, skip_serializing)]
    pub(crate) goal_ball_air_times: Vec<f32>,
}

impl GoalBallAirTimeStats {
    pub fn goal_ball_air_times(&self) -> &[f32] {
        &self.goal_ball_air_times
    }

    pub fn record_goal(&mut self, ball_air_time: f32) {
        let clamped_time = ball_air_time.max(0.0);
        self.goal_ball_air_time_sample_count += 1;
        self.cumulative_goal_ball_air_time += clamped_time;
        self.last_goal_ball_air_time = Some(clamped_time);
        self.goal_ball_air_times.push(clamped_time);
        self.goal_ball_air_times
            .sort_by(|left, right| left.total_cmp(right));
    }

    pub fn average_goal_ball_air_time(&self) -> f32 {
        if self.goal_ball_air_time_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_ball_air_time / self.goal_ball_air_time_sample_count as f32
        }
    }

    pub fn median_goal_ball_air_time(&self) -> f32 {
        if self.goal_ball_air_times.is_empty() {
            return 0.0;
        }

        let mut sorted_times = self.goal_ball_air_times.clone();
        sorted_times.sort_by(|a, b| a.total_cmp(b));
        let midpoint = sorted_times.len() / 2;
        if sorted_times.len().is_multiple_of(2) {
            (sorted_times[midpoint - 1] + sorted_times[midpoint]) * 0.5
        } else {
            sorted_times[midpoint]
        }
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        self.goal_ball_air_time_sample_count += other.goal_ball_air_time_sample_count;
        self.cumulative_goal_ball_air_time += other.cumulative_goal_ball_air_time;
        self.last_goal_ball_air_time = other
            .last_goal_ball_air_time
            .or(self.last_goal_ball_air_time);
        self.goal_ball_air_times
            .extend(other.goal_ball_air_times.iter().copied());
        self.goal_ball_air_times
            .sort_by(|left, right| left.total_cmp(right));
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalBuildupStats {
    pub counter_attack_goal_count: u32,
    pub sustained_pressure_goal_count: u32,
    pub other_buildup_goal_count: u32,
}

impl GoalBuildupStats {
    pub(crate) fn record(&mut self, kind: GoalBuildupKind) {
        match kind {
            GoalBuildupKind::CounterAttack => self.counter_attack_goal_count += 1,
            GoalBuildupKind::SustainedPressure => self.sustained_pressure_goal_count += 1,
            GoalBuildupKind::Other => self.other_buildup_goal_count += 1,
        }
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        self.counter_attack_goal_count += other.counter_attack_goal_count;
        self.sustained_pressure_goal_count += other.sustained_pressure_goal_count;
        self.other_buildup_goal_count += other.other_buildup_goal_count;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerScoringContextStats {
    pub goals_conceded_while_last_defender: u32,
    pub goals_for_while_most_back: u32,
    pub goals_against_while_most_back: u32,
    pub caught_ahead_of_play_on_conceded_goals: u32,
    pub goal_against_boost_sample_count: u32,
    pub cumulative_boost_on_goals_against: f32,
    pub last_boost_on_goal_against: Option<f32>,
    pub goal_against_boost_leadup_sample_count: u32,
    pub cumulative_average_boost_in_goal_against_leadup: f32,
    pub cumulative_min_boost_in_goal_against_leadup: f32,
    pub last_average_boost_in_goal_against_leadup: Option<f32>,
    pub last_min_boost_in_goal_against_leadup: Option<f32>,
    pub goal_against_position_sample_count: u32,
    pub cumulative_goal_against_position_x: f32,
    pub cumulative_goal_against_position_y: f32,
    pub cumulative_goal_against_position_z: f32,
    pub last_goal_against_position: Option<GoalContextPosition>,
    pub scoring_goal_last_touch_position_sample_count: u32,
    pub cumulative_scoring_goal_last_touch_position_x: f32,
    pub cumulative_scoring_goal_last_touch_position_y: f32,
    pub cumulative_scoring_goal_last_touch_position_z: f32,
    pub last_scoring_goal_last_touch_position: Option<GoalContextPosition>,
    #[serde(flatten)]
    pub goal_after_kickoff: GoalAfterKickoffStats,
    #[serde(flatten)]
    pub goal_buildup: GoalBuildupStats,
    #[serde(default, flatten)]
    pub goal_ball_air_time: GoalBallAirTimeStats,
}

impl PlayerScoringContextStats {
    pub(crate) fn record_goal_against_snapshot(
        &mut self,
        boost_amount: Option<f32>,
        position: Option<GoalContextPosition>,
        boost_leadup: Option<(f32, f32)>,
    ) {
        if let Some(boost_amount) = boost_amount {
            self.goal_against_boost_sample_count += 1;
            self.cumulative_boost_on_goals_against += boost_amount;
            self.last_boost_on_goal_against = Some(boost_amount);
        }

        if let Some((average_boost, min_boost)) = boost_leadup {
            self.goal_against_boost_leadup_sample_count += 1;
            self.cumulative_average_boost_in_goal_against_leadup += average_boost;
            self.cumulative_min_boost_in_goal_against_leadup += min_boost;
            self.last_average_boost_in_goal_against_leadup = Some(average_boost);
            self.last_min_boost_in_goal_against_leadup = Some(min_boost);
        }

        if let Some(position) = position {
            self.goal_against_position_sample_count += 1;
            self.cumulative_goal_against_position_x += position.x;
            self.cumulative_goal_against_position_y += position.y;
            self.cumulative_goal_against_position_z += position.z;
            self.last_goal_against_position = Some(position);
        }
    }

    pub(crate) fn record_scoring_goal_last_touch_position(
        &mut self,
        position: GoalContextPosition,
    ) {
        self.scoring_goal_last_touch_position_sample_count += 1;
        self.cumulative_scoring_goal_last_touch_position_x += position.x;
        self.cumulative_scoring_goal_last_touch_position_y += position.y;
        self.cumulative_scoring_goal_last_touch_position_z += position.z;
        self.last_scoring_goal_last_touch_position = Some(position);
    }

    pub(crate) fn record_goal_ball_air_time(&mut self, ball_air_time: f32) {
        self.goal_ball_air_time.record_goal(ball_air_time);
    }

    fn average_boost_on_goals_against(&self) -> f32 {
        if self.goal_against_boost_sample_count == 0 {
            0.0
        } else {
            self.cumulative_boost_on_goals_against / self.goal_against_boost_sample_count as f32
        }
    }

    fn average_boost_in_goal_against_leadup(&self) -> f32 {
        if self.goal_against_boost_leadup_sample_count == 0 {
            0.0
        } else {
            self.cumulative_average_boost_in_goal_against_leadup
                / self.goal_against_boost_leadup_sample_count as f32
        }
    }

    fn average_min_boost_in_goal_against_leadup(&self) -> f32 {
        if self.goal_against_boost_leadup_sample_count == 0 {
            0.0
        } else {
            self.cumulative_min_boost_in_goal_against_leadup
                / self.goal_against_boost_leadup_sample_count as f32
        }
    }

    fn average_goal_against_position_x(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_x / self.goal_against_position_sample_count as f32
        }
    }

    fn average_goal_against_position_y(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_y / self.goal_against_position_sample_count as f32
        }
    }

    fn average_goal_against_position_z(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_z / self.goal_against_position_sample_count as f32
        }
    }

    fn average_scoring_goal_last_touch_position_x(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_x
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }

    fn average_scoring_goal_last_touch_position_y(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_y
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }

    fn average_scoring_goal_last_touch_position_z(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_z
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CorePlayerStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    #[serde(flatten)]
    pub scoring_context: PlayerScoringContextStats,
}

impl CorePlayerStats {
    pub fn shooting_percentage(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.goals as f32 * 100.0 / self.shots as f32
        }
    }

    pub fn average_goal_time_after_kickoff(&self) -> f32 {
        self.scoring_context
            .goal_after_kickoff
            .average_goal_time_after_kickoff()
    }

    pub fn median_goal_time_after_kickoff(&self) -> f32 {
        self.scoring_context
            .goal_after_kickoff
            .median_goal_time_after_kickoff()
    }

    pub fn average_boost_on_goals_against(&self) -> f32 {
        self.scoring_context.average_boost_on_goals_against()
    }

    pub fn average_boost_in_goal_against_leadup(&self) -> f32 {
        self.scoring_context.average_boost_in_goal_against_leadup()
    }

    pub fn average_min_boost_in_goal_against_leadup(&self) -> f32 {
        self.scoring_context
            .average_min_boost_in_goal_against_leadup()
    }

    pub fn average_goal_against_position_x(&self) -> f32 {
        self.scoring_context.average_goal_against_position_x()
    }

    pub fn average_goal_against_position_y(&self) -> f32 {
        self.scoring_context.average_goal_against_position_y()
    }

    pub fn average_goal_against_position_z(&self) -> f32 {
        self.scoring_context.average_goal_against_position_z()
    }

    pub fn average_scoring_goal_last_touch_position_x(&self) -> f32 {
        self.scoring_context
            .average_scoring_goal_last_touch_position_x()
    }

    pub fn average_scoring_goal_last_touch_position_y(&self) -> f32 {
        self.scoring_context
            .average_scoring_goal_last_touch_position_y()
    }

    pub fn average_scoring_goal_last_touch_position_z(&self) -> f32 {
        self.scoring_context
            .average_scoring_goal_last_touch_position_z()
    }

    pub fn average_goal_ball_air_time(&self) -> f32 {
        self.scoring_context
            .goal_ball_air_time
            .average_goal_ball_air_time()
    }

    pub fn median_goal_ball_air_time(&self) -> f32 {
        self.scoring_context
            .goal_ball_air_time
            .median_goal_ball_air_time()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TeamScoringContextStats {
    #[serde(flatten)]
    pub goal_after_kickoff: GoalAfterKickoffStats,
    #[serde(flatten)]
    pub goal_buildup: GoalBuildupStats,
    #[serde(default, flatten)]
    pub goal_ball_air_time: GoalBallAirTimeStats,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CoreTeamStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    #[serde(flatten)]
    pub scoring_context: TeamScoringContextStats,
}

impl CoreTeamStats {
    pub fn shooting_percentage(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.goals as f32 * 100.0 / self.shots as f32
        }
    }

    pub fn average_goal_time_after_kickoff(&self) -> f32 {
        self.scoring_context
            .goal_after_kickoff
            .average_goal_time_after_kickoff()
    }

    pub fn median_goal_time_after_kickoff(&self) -> f32 {
        self.scoring_context
            .goal_after_kickoff
            .median_goal_time_after_kickoff()
    }

    pub fn average_goal_ball_air_time(&self) -> f32 {
        self.scoring_context
            .goal_ball_air_time
            .average_goal_ball_air_time()
    }

    pub fn median_goal_ball_air_time(&self) -> f32 {
        self.scoring_context
            .goal_ball_air_time
            .median_goal_ball_air_time()
    }
}

pub(crate) fn player_id_sort_key(player_id: &PlayerId) -> String {
    match player_id {
        boxcars::RemoteId::PlayStation(id) => {
            format!("playstation:{}:{}:{:?}", id.online_id, id.name, id.unknown1)
        }
        boxcars::RemoteId::PsyNet(id) => format!("psynet:{}:{:?}", id.online_id, id.unknown1),
        boxcars::RemoteId::SplitScreen(id) => format!("splitscreen:{id}"),
        boxcars::RemoteId::Steam(id) => format!("steam:{id}"),
        boxcars::RemoteId::Switch(id) => format!("switch:{}:{:?}", id.online_id, id.unknown1),
        boxcars::RemoteId::Xbox(id) => format!("xbox:{id}"),
        boxcars::RemoteId::QQ(id) => format!("qq:{id}"),
        boxcars::RemoteId::Epic(id) => format!("epic:{id}"),
    }
}

#[derive(Debug, Clone, Default)]
pub struct CoreStatsAccumulator {
    player_stats: HashMap<PlayerId, CorePlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
}

impl CoreStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CorePlayerStats> {
        &self.player_stats
    }

    pub fn player_stats_for(&self, player_id: &PlayerId) -> CorePlayerStats {
        self.player_stats
            .get(player_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn ensure_player(&mut self, player_id: PlayerId, is_team_0: bool) {
        self.player_teams.insert(player_id.clone(), is_team_0);
        self.player_stats.entry(player_id).or_default();
    }

    pub fn team_zero_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(true)
    }

    pub fn team_one_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(false)
    }

    pub fn team_stats_for_side(&self, is_team_0: bool) -> CoreTeamStats {
        let mut player_stats: Vec<_> = self
            .player_stats
            .iter()
            .filter(|(player_id, _)| self.player_teams.get(*player_id) == Some(&is_team_0))
            .collect();
        player_stats.sort_by_cached_key(|(player_id, _)| player_id_sort_key(player_id));

        let mut stats = player_stats.into_iter().fold(
            CoreTeamStats::default(),
            |mut stats, (_, player_stats)| {
                stats.score += player_stats.score;
                stats.goals += player_stats.goals;
                stats.assists += player_stats.assists;
                stats.saves += player_stats.saves;
                stats.shots += player_stats.shots;
                stats
                    .scoring_context
                    .goal_after_kickoff
                    .merge(&player_stats.scoring_context.goal_after_kickoff);
                stats
                    .scoring_context
                    .goal_buildup
                    .merge(&player_stats.scoring_context.goal_buildup);
                stats
                    .scoring_context
                    .goal_ball_air_time
                    .merge(&player_stats.scoring_context.goal_ball_air_time);
                stats
            },
        );
        stats
            .scoring_context
            .goal_after_kickoff
            .goal_times
            .sort_by(|left, right| left.total_cmp(right));
        stats
            .scoring_context
            .goal_ball_air_time
            .goal_ball_air_times
            .sort_by(|left, right| left.total_cmp(right));
        stats
    }

    pub fn apply_scoreboard_event(&mut self, event: &CorePlayerScoreboardEvent) {
        self.player_teams
            .insert(event.player.clone(), event.is_team_0);
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.score += event.score_delta;
        stats.goals += event.goals_delta;
        stats.assists += event.assists_delta;
        stats.saves += event.saves_delta;
        stats.shots += event.shots_delta;
    }

    pub fn apply_goal_context_event(&mut self, event: &CorePlayerGoalContextEvent) {
        self.player_teams
            .insert(event.player.clone(), event.is_team_0);
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        let scoring_context = &mut stats.scoring_context;
        if event.goals_conceded_while_last_defender {
            scoring_context.goals_conceded_while_last_defender += 1;
        }
        if event.goals_for_while_most_back {
            scoring_context.goals_for_while_most_back += 1;
        }
        if event.goals_against_while_most_back {
            scoring_context.goals_against_while_most_back += 1;
        }
        if event.caught_ahead_of_play_on_conceded_goal {
            scoring_context.caught_ahead_of_play_on_conceded_goals += 1;
        }
        scoring_context.record_goal_against_snapshot(
            event.goal_against_boost_amount,
            event.goal_against_position,
            match (
                event.goal_against_average_boost_in_leadup,
                event.goal_against_min_boost_in_leadup,
            ) {
                (Some(average), Some(minimum)) => Some((average, minimum)),
                _ => None,
            },
        );
        if let Some(position) = event.scoring_goal_last_touch_position {
            scoring_context.record_scoring_goal_last_touch_position(position);
        }
        if let Some(time_after_kickoff) = event.time_after_kickoff {
            scoring_context
                .goal_after_kickoff
                .record_goal(time_after_kickoff);
        }
        if let Some(goal_buildup) = event.goal_buildup {
            scoring_context.goal_buildup.record(goal_buildup);
        }
        if let Some(ball_air_time_before_goal) = event.ball_air_time_before_goal {
            scoring_context.record_goal_ball_air_time(ball_air_time_before_goal);
        }
    }
}
