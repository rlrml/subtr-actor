mod comparable_types;
mod config;
mod conversion;
mod model;

pub(crate) use config::StatMatcher;
pub use config::{recommended_ballchasing_match_config, MatchConfig};
pub(crate) use conversion::{
    build_actual_comparable_stats, build_expected_comparable_stats,
    compute_ballchasing_comparable_stats,
};
