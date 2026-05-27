use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ActiveRush {
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) last_time: f32,
    pub(super) last_frame: usize,
    pub(super) is_team_0: bool,
    pub(super) attackers: usize,
    pub(super) defenders: usize,
    pub(super) counted: bool,
}

impl ActiveRush {
    pub(super) fn retained_possession_time(&self) -> f32 {
        (self.last_time - self.start_time).max(0.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RushCalculator {
    pub(super) config: RushCalculatorConfig,
    pub(super) stats: RushStats,
    pub(super) events: Vec<RushEvent>,
    pub(super) active_rush: Option<ActiveRush>,
}
