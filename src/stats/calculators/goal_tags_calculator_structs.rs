use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct AerialGoalCalculator {
    pub(super) config: AerialGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HighAerialGoalCalculator {
    pub(super) config: HighAerialGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LongDistanceGoalCalculator {
    pub(super) config: LongDistanceGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OwnHalfGoalCalculator {
    pub(super) config: OwnHalfGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmptyNetGoalCalculator {
    pub(super) config: EmptyNetGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CounterAttackGoalCalculator {
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlickGoalCalculator {
    pub(super) config: FlickGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoubleTapGoalCalculator {
    pub(super) config: DoubleTapGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OneTimerGoalCalculator {
    pub(super) config: OneTimerGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PassingGoalCalculator {
    pub(super) config: PassingGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AirDribbleGoalCalculator {
    pub(super) config: AirDribbleGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlipResetGoalCalculator {
    pub(super) config: FlipResetGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HalfVolleyGoalCalculator {
    pub(super) config: HalfVolleyGoalCalculatorConfig,
    pub(super) events: Vec<GoalTagEvent>,
}
