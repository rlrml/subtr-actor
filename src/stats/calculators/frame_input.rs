use super::*;

#[path = "frame_input_accessors.rs"]
mod frame_input_accessors;
#[path = "frame_input_components.rs"]
mod frame_input_components;
#[path = "frame_input_constructors.rs"]
mod frame_input_constructors;
#[path = "frame_input_events.rs"]
mod frame_input_events;

#[derive(Debug, Clone)]
pub struct FrameInput {
    frame_info: FrameInfo,
    gameplay_state: GameplayState,
    ball_frame_state: BallFrameState,
    player_frame_state: PlayerFrameState,
    frame_events_state: FrameEventsState,
    live_play_state: Option<LivePlayState>,
}

impl FrameInput {
    /// Builds a frame input from already-materialized frame component states.
    ///
    /// Replay callers should usually use [`FrameInput::timeline`] or
    /// [`FrameInput::aggregate`]. Live callers can construct these same
    /// component states directly from their sampled game state.
    pub fn from_parts(
        frame_info: FrameInfo,
        gameplay_state: GameplayState,
        ball_frame_state: BallFrameState,
        player_frame_state: PlayerFrameState,
        frame_events_state: FrameEventsState,
    ) -> Self {
        Self {
            frame_info,
            gameplay_state,
            ball_frame_state,
            player_frame_state,
            frame_events_state,
            live_play_state: None,
        }
    }

    /// Builds a frame input with an explicitly sampled live-play state.
    ///
    /// Replay processing should let the graph derive live play from replicated
    /// gameplay fields. Live callers can use this when the host integration has
    /// a stronger source of truth for whether analysis should run on a frame.
    pub fn from_parts_with_live_play_state(
        frame_info: FrameInfo,
        gameplay_state: GameplayState,
        ball_frame_state: BallFrameState,
        player_frame_state: PlayerFrameState,
        frame_events_state: FrameEventsState,
        live_play_state: LivePlayState,
    ) -> Self {
        Self {
            frame_info,
            gameplay_state,
            ball_frame_state,
            player_frame_state,
            frame_events_state,
            live_play_state: Some(live_play_state),
        }
    }
}
