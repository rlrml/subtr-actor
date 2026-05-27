use subtr_actor::boost_amount_to_percent;

#[path = "conversion_stats_boost.rs"]
mod boost;
#[path = "conversion_stats_core.rs"]
mod core;
#[path = "conversion_stats_movement.rs"]
mod movement;
#[path = "conversion_stats_positioning_demo.rs"]
mod positioning_demo;

pub(super) use boost::comparable_boost_from_stats;
pub(super) use core::{comparable_core_from_player, comparable_core_from_team};
pub(super) use movement::comparable_movement_from_stats;
pub(super) use positioning_demo::{
    comparable_demo_from_player, comparable_demo_from_team, comparable_positioning_from_stats,
};

pub(super) fn raw_boost_amount_as_comparable_units(value: f32) -> f64 {
    boost_amount_to_percent(value) as f64
}

pub(super) fn sum_present(values: impl IntoIterator<Item = Option<f64>>) -> Option<f64> {
    let mut saw_value = false;
    let sum = values.into_iter().fold(0.0, |acc, value| match value {
        Some(value) => {
            saw_value = true;
            acc + value
        }
        None => acc,
    });
    saw_value.then_some(sum)
}
