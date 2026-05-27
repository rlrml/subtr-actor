use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AerialGoalCalculatorConfig {
    pub min_ball_z: f32,
}

impl Default for AerialGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_ball_z: DEFAULT_AERIAL_GOAL_MIN_BALL_Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HighAerialGoalCalculatorConfig {
    pub min_ball_z: f32,
}

impl Default for HighAerialGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_ball_z: DEFAULT_HIGH_AERIAL_GOAL_MIN_BALL_Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LongDistanceGoalCalculatorConfig {
    pub max_attacking_y: f32,
}

impl Default for LongDistanceGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_attacking_y: DEFAULT_LONG_DISTANCE_GOAL_MAX_ATTACKING_Y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OwnHalfGoalCalculatorConfig {
    pub max_attacking_y: f32,
}

impl Default for OwnHalfGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_attacking_y: DEFAULT_OWN_HALF_GOAL_MAX_ATTACKING_Y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct EmptyNetGoalCalculatorConfig {
    pub min_defender_y_margin: f32,
    pub min_defender_distance: f32,
    pub max_touch_attacking_y: f32,
}

impl Default for EmptyNetGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_defender_y_margin: DEFAULT_EMPTY_NET_MIN_DEFENDER_Y_MARGIN,
            min_defender_distance: DEFAULT_EMPTY_NET_MIN_DEFENDER_DISTANCE,
            max_touch_attacking_y: DEFAULT_EMPTY_NET_MAX_TOUCH_ATTACKING_Y,
        }
    }
}
