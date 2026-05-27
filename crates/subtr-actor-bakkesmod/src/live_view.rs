use super::*;

#[path = "live_view_event_slices.rs"]
mod live_view_event_slices;
#[path = "live_view_struct.rs"]
mod live_view_struct;
#[macro_use]
#[path = "live_view_processor_ball.rs"]
mod live_view_processor_ball;
#[macro_use]
#[path = "live_view_processor_events.rs"]
mod live_view_processor_events;
#[macro_use]
#[path = "live_view_processor_frame_events.rs"]
mod live_view_processor_frame_events;
#[macro_use]
#[path = "live_view_processor_game.rs"]
mod live_view_processor_game;
#[macro_use]
#[path = "live_view_processor_player_core.rs"]
mod live_view_processor_player_core;
#[macro_use]
#[path = "live_view_processor_player_stats.rs"]
mod live_view_processor_player_stats;

pub(crate) use live_view_event_slices::{frame_event_slices, SaFrameEventSlices};
pub(crate) use live_view_struct::SaLiveProcessorView;

impl ProcessorGameView for SaLiveProcessorView<'_> {
    sa_live_processor_game_methods!();
}

impl ProcessorBallView for SaLiveProcessorView<'_> {
    sa_live_processor_ball_methods!();
}

impl ProcessorPlayerCoreView for SaLiveProcessorView<'_> {
    sa_live_processor_player_core_methods!();
}

impl ProcessorPlayerStatsView for SaLiveProcessorView<'_> {
    sa_live_processor_player_stats_methods!();
}

impl ProcessorEventHistoryView for SaLiveProcessorView<'_> {
    sa_live_processor_event_methods!();
}

impl ProcessorFrameEventView for SaLiveProcessorView<'_> {
    sa_live_processor_frame_event_methods!();
}
