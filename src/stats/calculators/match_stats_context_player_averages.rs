use super::*;

impl PlayerScoringContextStats {
    pub(super) fn average_boost_on_goals_against(&self) -> f32 {
        if self.goal_against_boost_sample_count == 0 {
            0.0
        } else {
            self.cumulative_boost_on_goals_against / self.goal_against_boost_sample_count as f32
        }
    }

    pub(super) fn average_boost_in_goal_against_leadup(&self) -> f32 {
        if self.goal_against_boost_leadup_sample_count == 0 {
            0.0
        } else {
            self.cumulative_average_boost_in_goal_against_leadup
                / self.goal_against_boost_leadup_sample_count as f32
        }
    }

    pub(super) fn average_min_boost_in_goal_against_leadup(&self) -> f32 {
        if self.goal_against_boost_leadup_sample_count == 0 {
            0.0
        } else {
            self.cumulative_min_boost_in_goal_against_leadup
                / self.goal_against_boost_leadup_sample_count as f32
        }
    }

    pub(super) fn average_goal_against_position_x(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_x / self.goal_against_position_sample_count as f32
        }
    }

    pub(super) fn average_goal_against_position_y(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_y / self.goal_against_position_sample_count as f32
        }
    }

    pub(super) fn average_goal_against_position_z(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_z / self.goal_against_position_sample_count as f32
        }
    }

    pub(super) fn average_scoring_goal_last_touch_position_x(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_x
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }

    pub(super) fn average_scoring_goal_last_touch_position_y(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_y
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }

    pub(super) fn average_scoring_goal_last_touch_position_z(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_z
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }
}
