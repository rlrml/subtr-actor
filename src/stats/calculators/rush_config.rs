use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct RushCalculatorConfig {
    pub max_start_y: f32,
    pub attack_support_distance_y: f32,
    pub defender_distance_y: f32,
    pub min_possession_retained_seconds: f32,
}

impl Default for RushCalculatorConfig {
    fn default() -> Self {
        Self {
            max_start_y: DEFAULT_RUSH_MAX_START_Y,
            attack_support_distance_y: DEFAULT_RUSH_ATTACK_SUPPORT_DISTANCE_Y,
            defender_distance_y: DEFAULT_RUSH_DEFENDER_DISTANCE_Y,
            min_possession_retained_seconds: DEFAULT_RUSH_MIN_POSSESSION_RETAINED_SECONDS,
        }
    }
}
