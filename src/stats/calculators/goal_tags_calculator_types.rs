use super::*;

macro_rules! impl_goal_tag_calculator {
    ($calculator:ident, $config:ident) => {
        impl Default for $calculator {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $calculator {
            pub fn new() -> Self {
                Self::with_config($config::default())
            }

            pub fn with_config(config: $config) -> Self {
                Self {
                    config,
                    events: Vec::new(),
                }
            }

            pub fn config(&self) -> &$config {
                &self.config
            }

            pub fn events(&self) -> &[GoalTagEvent] {
                &self.events
            }
        }
    };
}

impl_goal_tag_calculator!(AerialGoalCalculator, AerialGoalCalculatorConfig);
impl_goal_tag_calculator!(HighAerialGoalCalculator, HighAerialGoalCalculatorConfig);
impl_goal_tag_calculator!(LongDistanceGoalCalculator, LongDistanceGoalCalculatorConfig);
impl_goal_tag_calculator!(OwnHalfGoalCalculator, OwnHalfGoalCalculatorConfig);
impl_goal_tag_calculator!(EmptyNetGoalCalculator, EmptyNetGoalCalculatorConfig);
impl_goal_tag_calculator!(FlickGoalCalculator, FlickGoalCalculatorConfig);
impl_goal_tag_calculator!(DoubleTapGoalCalculator, DoubleTapGoalCalculatorConfig);
impl_goal_tag_calculator!(OneTimerGoalCalculator, OneTimerGoalCalculatorConfig);
impl_goal_tag_calculator!(PassingGoalCalculator, PassingGoalCalculatorConfig);
impl_goal_tag_calculator!(AirDribbleGoalCalculator, AirDribbleGoalCalculatorConfig);
impl_goal_tag_calculator!(FlipResetGoalCalculator, FlipResetGoalCalculatorConfig);

impl Default for CounterAttackGoalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterAttackGoalCalculator {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn events(&self) -> &[GoalTagEvent] {
        &self.events
    }
}

impl Default for HalfVolleyGoalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl HalfVolleyGoalCalculator {
    pub fn new() -> Self {
        Self::with_config(HalfVolleyGoalCalculatorConfig::default())
    }

    pub fn with_config(config: HalfVolleyGoalCalculatorConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
        }
    }

    pub fn config(&self) -> &HalfVolleyGoalCalculatorConfig {
        &self.config
    }

    pub fn events(&self) -> &[GoalTagEvent] {
        &self.events
    }
}
