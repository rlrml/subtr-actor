use super::*;

#[path = "touch_state_calculator.rs"]
mod touch_state_calculator;
#[path = "touch_state_confirmed.rs"]
mod touch_state_confirmed;
#[path = "touch_state_explicit.rs"]
mod touch_state_explicit;
#[path = "touch_state_proximity.rs"]
mod touch_state_proximity;
#[path = "touch_state_proximity_distance.rs"]
mod touch_state_proximity_distance;
#[path = "touch_state_types.rs"]
mod touch_state_types;
#[path = "touch_state_update.rs"]
mod touch_state_update;
#[path = "touch_state_velocity.rs"]
mod touch_state_velocity;

pub use touch_state_calculator::TouchStateCalculator;
pub(crate) use touch_state_proximity::touch_distance;
pub(crate) use touch_state_proximity_distance::collision_distance;
pub use touch_state_types::TouchState;

#[cfg(test)]
#[path = "touch_state_tests.rs"]
mod tests;
