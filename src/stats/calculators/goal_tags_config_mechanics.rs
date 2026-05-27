use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlickGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for FlickGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_FLICK_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for DoubleTapGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_DOUBLE_TAP_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for OneTimerGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_ONE_TIMER_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassingGoalCalculatorConfig {
    pub max_pass_to_goal_seconds: f32,
}

impl Default for PassingGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_pass_to_goal_seconds: DEFAULT_PASSING_GOAL_MAX_PASS_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AirDribbleGoalCalculatorConfig {
    pub max_end_to_goal_seconds: f32,
}

impl Default for AirDribbleGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_end_to_goal_seconds: DEFAULT_AIR_DRIBBLE_GOAL_MAX_END_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlipResetGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for FlipResetGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_FLIP_RESET_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}
