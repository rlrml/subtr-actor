use serde_json::Value;

#[path = "conversion_json_boost.rs"]
mod boost;
#[path = "conversion_json_core_demo.rs"]
mod core_demo;
#[path = "conversion_json_movement.rs"]
mod movement;
#[path = "conversion_json_positioning.rs"]
mod positioning;

pub(super) use boost::comparable_boost_from_json;
pub(super) use core_demo::{
    comparable_core_from_json, comparable_demo_from_json, comparable_team_demo_from_json,
};
pub(super) use movement::comparable_movement_from_json;
pub(super) use positioning::comparable_positioning_from_json;

pub(super) fn json_number(stats: Option<&Value>, field: &str) -> Option<f64> {
    stats
        .and_then(|stats| stats.get(field))
        .and_then(Value::as_f64)
}
