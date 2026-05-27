use super::*;

#[path = "live_view_event_slices.rs"]
mod live_view_event_slices;
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

pub(crate) struct SaLiveProcessorView<'a> {
    replay_meta: Option<&'a ReplayMeta>,
    frame: &'a SaLiveFrame,
    players: &'a [SaPlayerFrame],
    player_ids: Vec<PlayerId>,
    events: FrameEventsState,
    event_history: &'a SaLiveEventHistory,
}

impl<'a> SaLiveProcessorView<'a> {
    pub(crate) fn new(
        replay_meta: Option<&'a ReplayMeta>,
        frame: &'a SaLiveFrame,
        players: &'a [SaPlayerFrame],
        events: FrameEventsState,
        event_history: &'a SaLiveEventHistory,
    ) -> Self {
        Self {
            replay_meta,
            frame,
            players,
            player_ids: players
                .iter()
                .map(|player| player_id(player.player_index))
                .collect(),
            events,
            event_history,
        }
    }

    fn missing<T>(property: &'static str) -> SubtrActorResult<T> {
        SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState { property })
    }

    pub(crate) fn player_index(player_id: &PlayerId) -> Option<u32> {
        match player_id {
            RemoteId::SplitScreen(index) => Some(*index),
            _ => None,
        }
    }

    fn player(&self, player_id: &PlayerId) -> SubtrActorResult<&SaPlayerFrame> {
        let Some(index) = Self::player_index(player_id) else {
            return Self::missing("live player");
        };
        self.players
            .iter()
            .find(|player| player.player_index == index)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: "live player",
                })
            })
    }
}

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
