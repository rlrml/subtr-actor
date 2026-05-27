use super::*;

#[path = "demo_calculator.rs"]
mod demo_calculator;
#[path = "demo_record.rs"]
mod demo_record;
#[path = "demo_stats.rs"]
mod demo_stats;
#[path = "demo_update.rs"]
mod demo_update;

pub use demo_calculator::DemoCalculator;
pub use demo_stats::{DemoPlayerStats, DemoTeamStats};

const DEMO_REPEAT_FRAME_WINDOW: usize = 8;

#[cfg(test)]
#[path = "demo_tests.rs"]
mod tests;
