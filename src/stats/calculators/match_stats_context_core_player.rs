use super::*;

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
