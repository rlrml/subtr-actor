use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyGoalCalculatorConfig {
    pub max_touch_to_goal_seconds: f32,
    pub min_goal_alignment: f32,
}

impl Default for HalfVolleyGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_touch_to_goal_seconds: DEFAULT_HALF_VOLLEY_GOAL_MAX_TOUCH_TO_GOAL_SECONDS,
            min_goal_alignment: DEFAULT_HALF_VOLLEY_GOAL_MIN_GOAL_ALIGNMENT,
        }
    }
}
