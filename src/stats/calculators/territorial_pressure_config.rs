use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TerritorialPressureCalculatorConfig {
    pub neutral_zone_half_width_y: f32,
    pub min_establish_seconds: f32,
    pub min_establish_third_seconds: f32,
    pub relief_grace_seconds: f32,
    pub confirmed_relief_grace_seconds: f32,
}

impl Default for TerritorialPressureCalculatorConfig {
    fn default() -> Self {
        Self {
            neutral_zone_half_width_y: DEFAULT_TERRITORIAL_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y,
            min_establish_seconds: DEFAULT_TERRITORIAL_PRESSURE_MIN_ESTABLISH_SECONDS,
            min_establish_third_seconds: DEFAULT_TERRITORIAL_PRESSURE_MIN_ESTABLISH_THIRD_SECONDS,
            relief_grace_seconds: DEFAULT_TERRITORIAL_PRESSURE_RELIEF_GRACE_SECONDS,
            confirmed_relief_grace_seconds:
                DEFAULT_TERRITORIAL_PRESSURE_CONFIRMED_RELIEF_GRACE_SECONDS,
        }
    }
}
