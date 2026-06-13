mod comparable_types;
mod config;
mod conversion;
mod model;

pub(crate) use config::StatMatcher;
pub use config::{MatchConfig, recommended_match_config};
pub(crate) use conversion::{
    build_actual_comparable_stats, build_expected_comparable_stats, compute_comparable_stats,
};
