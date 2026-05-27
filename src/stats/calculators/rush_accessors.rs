use super::*;

impl RushCalculator {
    pub fn new() -> Self {
        Self::with_config(RushCalculatorConfig::default())
    }

    pub fn with_config(config: RushCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &RushCalculatorConfig {
        &self.config
    }

    pub fn stats(&self) -> &RushStats {
        &self.stats
    }

    pub fn events(&self) -> &[RushEvent] {
        &self.events
    }
}
