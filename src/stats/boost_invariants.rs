use super::calculators::BoostStats;
use crate::*;

#[path = "boost_invariants_amounts.rs"]
mod boost_invariants_amounts;
#[path = "boost_invariants_checks.rs"]
mod boost_invariants_checks;
#[path = "boost_invariants_messages.rs"]
mod boost_invariants_messages;
#[path = "boost_invariants_types.rs"]
mod boost_invariants_types;

pub use boost_invariants_amounts::{
    nominal_pickup_amount_from_counts, nominal_stolen_pickup_amount_from_counts,
};
pub use boost_invariants_checks::boost_invariant_violations;
pub use boost_invariants_types::{BoostInvariantKind, BoostInvariantViolation};

#[cfg(test)]
#[path = "boost_invariants_tests.rs"]
mod tests;
