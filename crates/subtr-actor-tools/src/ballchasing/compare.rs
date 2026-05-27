#[path = "compare_core.rs"]
mod core;
#[path = "compare_file.rs"]
mod file;
#[path = "compare_parse.rs"]
mod parse;
#[path = "compare_types.rs"]
mod types;

pub use core::{
    compare_replay_against_ballchasing, compare_replay_against_ballchasing_with_breakdown,
};
pub use file::{
    compare_fixture_directory, compare_replay_against_ballchasing_json,
    compare_replay_against_ballchasing_json_with_breakdown,
};
pub use parse::{parse_replay_bytes, parse_replay_file};
pub use types::{BallchasingComparableStats, BallchasingComparisonBreakdown};
