use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerScoringContextStats {
    pub goals_conceded_while_last_defender: u32,
    pub goals_for_while_most_back: u32,
    pub goals_against_while_most_back: u32,
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
    pub(in crate::stats::calculators::match_stats) fn record_goal_against_snapshot(
        &mut self,
        boost_amount: Option<f32>,
        position: Option<GoalContextPosition>,
        boost_leadup: Option<BoostLeadupStats>,
    ) {
        if let Some(boost_amount) = boost_amount {
            self.goal_against_boost_sample_count += 1;
            self.cumulative_boost_on_goals_against += boost_amount;
            self.last_boost_on_goal_against = Some(boost_amount);
        }

        if let Some(boost_leadup) = boost_leadup {
            self.goal_against_boost_leadup_sample_count += 1;
            self.cumulative_average_boost_in_goal_against_leadup += boost_leadup.average_boost;
            self.cumulative_min_boost_in_goal_against_leadup += boost_leadup.min_boost;
            self.last_average_boost_in_goal_against_leadup = Some(boost_leadup.average_boost);
            self.last_min_boost_in_goal_against_leadup = Some(boost_leadup.min_boost);
        }

        if let Some(position) = position {
            self.goal_against_position_sample_count += 1;
            self.cumulative_goal_against_position_x += position.x;
            self.cumulative_goal_against_position_y += position.y;
            self.cumulative_goal_against_position_z += position.z;
            self.last_goal_against_position = Some(position);
        }
    }

    pub(in crate::stats::calculators::match_stats) fn record_scoring_goal_last_touch_position(
        &mut self,
        position: GoalContextPosition,
    ) {
        self.scoring_goal_last_touch_position_sample_count += 1;
        self.cumulative_scoring_goal_last_touch_position_x += position.x;
        self.cumulative_scoring_goal_last_touch_position_y += position.y;
        self.cumulative_scoring_goal_last_touch_position_z += position.z;
        self.last_scoring_goal_last_touch_position = Some(position);
    }

    pub(in crate::stats::calculators::match_stats) fn record_goal_ball_air_time(
        &mut self,
        ball_air_time: f32,
    ) {
        self.goal_ball_air_time.record_goal(ball_air_time);
    }
}
