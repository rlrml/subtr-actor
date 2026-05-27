pub type PlayerId = boxcars::RemoteId;

#[path = "replay_types_boost.rs"]
mod replay_types_boost;
#[path = "replay_types_demolish.rs"]
mod replay_types_demolish;
#[path = "replay_types_goal.rs"]
mod replay_types_goal;
#[path = "replay_types_meta.rs"]
mod replay_types_meta;
#[path = "replay_types_player_stats.rs"]
mod replay_types_player_stats;
#[path = "replay_types_touch.rs"]
mod replay_types_touch;

pub use replay_types_boost::*;
pub use replay_types_demolish::*;
pub use replay_types_goal::*;
pub use replay_types_meta::*;
pub use replay_types_player_stats::*;
pub use replay_types_touch::*;

#[cfg(test)]
#[path = "replay_types_tests.rs"]
mod tests;
