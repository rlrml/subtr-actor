use super::*;

#[path = "view_replay_processor_ball.rs"]
mod view_replay_processor_ball;
#[path = "view_replay_processor_events.rs"]
mod view_replay_processor_events;
#[path = "view_replay_processor_frame_events.rs"]
mod view_replay_processor_frame_events;
#[path = "view_replay_processor_game.rs"]
mod view_replay_processor_game;
#[path = "view_replay_processor_player_core.rs"]
mod view_replay_processor_player_core;
#[path = "view_replay_processor_player_stats.rs"]
mod view_replay_processor_player_stats;
#[path = "view_traits_ball.rs"]
mod view_traits_ball;
#[path = "view_traits_events.rs"]
mod view_traits_events;
#[path = "view_traits_frame_events.rs"]
mod view_traits_frame_events;
#[path = "view_traits_game.rs"]
mod view_traits_game;
#[path = "view_traits_player_core.rs"]
mod view_traits_player_core;
#[path = "view_traits_player_stats.rs"]
mod view_traits_player_stats;

pub use view_traits_ball::ProcessorBallView;
pub use view_traits_events::ProcessorEventHistoryView;
pub use view_traits_frame_events::ProcessorFrameEventView;
pub use view_traits_game::ProcessorGameView;
pub use view_traits_player_core::ProcessorPlayerCoreView;
pub use view_traits_player_stats::ProcessorPlayerStatsView;

/// Read-only processor surface consumed by collectors and stat calculators.
///
/// `ReplayProcessor` still owns replay traversal and actor-state mutation, but
/// collectors should depend on this trait so the same collection pipeline can
/// later be driven by non-replay state sources.
pub trait ProcessorView:
    ProcessorGameView
    + ProcessorBallView
    + ProcessorPlayerCoreView
    + ProcessorPlayerStatsView
    + ProcessorEventHistoryView
    + ProcessorFrameEventView
{
}

impl<T> ProcessorView for T where
    T: ProcessorGameView
        + ProcessorBallView
        + ProcessorPlayerCoreView
        + ProcessorPlayerStatsView
        + ProcessorEventHistoryView
        + ProcessorFrameEventView
{
}
