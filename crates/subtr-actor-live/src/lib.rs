//! Shared live-frame model for subtr-actor integrations.
//!
//! Live samplers (the BakkesMod plugin today, a WebSocket exporter next)
//! convert host game state into the owned [`LiveFrame`] model. This crate then
//! derives per-frame graph events with [`LiveEventGenerator`] and exposes the
//! frame through [`LiveProcessorView`] so the same analysis pipeline that
//! processes replays can run on live data.

#![allow(clippy::result_large_err)]

mod generator;
mod meta;
mod model;
mod view;

pub use generator::*;
pub use meta::*;
pub use model::*;
pub use view::*;

use subtr_actor::{FrameEventsState, FrameInput, LivePlayState, ReplayMeta};

/// Builds the analysis-graph [`FrameInput`] for one live frame.
///
/// Also returns the derived per-frame events and live-play state so callers
/// (e.g. a live export server) can forward them alongside the frame.
pub fn frame_input_from_live_frame(
    generator: &mut LiveEventGenerator,
    history: &mut LiveEventHistory,
    replay_meta: Option<&ReplayMeta>,
    frame: LiveFrame,
) -> (FrameInput, FrameEventsState, LivePlayState) {
    let (frame_events, live_play) = generator.frame_events(&frame);
    history.append_frame_events(&frame_events);
    let frame_number = frame.frame_number as usize;
    let time = frame.time;
    let dt = frame.dt;
    let view = LiveProcessorView::new(replay_meta, frame, frame_events.clone(), history);
    let frame_input =
        FrameInput::timeline_with_live_play_state(&view, frame_number, time, dt, live_play.clone());
    (frame_input, frame_events, live_play)
}
