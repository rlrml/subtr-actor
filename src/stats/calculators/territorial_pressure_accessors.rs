use super::*;

impl TerritorialPressureCalculator {
    pub fn new() -> Self {
        Self::with_config(TerritorialPressureCalculatorConfig::default())
    }

    pub fn with_config(config: TerritorialPressureCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn stats(&self) -> &TerritorialPressureStats {
        &self.stats
    }

    pub fn events(&self) -> &[TerritorialPressureEvent] {
        &self.events
    }

    pub fn config(&self) -> &TerritorialPressureCalculatorConfig {
        &self.config
    }

    pub(super) fn pressure_team_for_ball_y(&self, ball_y: f32) -> Option<bool> {
        if ball_y > self.config.neutral_zone_half_width_y {
            Some(true)
        } else if ball_y < -self.config.neutral_zone_half_width_y {
            Some(false)
        } else {
            None
        }
    }

    pub(super) fn normalized_ball_y(team_is_team_0: bool, ball_y: f32) -> f32 {
        if team_is_team_0 {
            ball_y
        } else {
            -ball_y
        }
    }
}
