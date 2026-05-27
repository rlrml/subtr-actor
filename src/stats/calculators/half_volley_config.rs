use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyCalculatorConfig {
    pub max_bounce_to_touch_seconds: f32,
    pub min_ball_speed: f32,
}

impl Default for HalfVolleyCalculatorConfig {
    fn default() -> Self {
        Self {
            max_bounce_to_touch_seconds: DEFAULT_HALF_VOLLEY_MAX_BOUNCE_TO_TOUCH_SECONDS,
            min_ball_speed: DEFAULT_HALF_VOLLEY_MIN_BALL_SPEED,
        }
    }
}
