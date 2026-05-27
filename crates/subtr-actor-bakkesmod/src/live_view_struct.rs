use super::*;

pub(crate) struct SaLiveProcessorView<'a> {
    pub(super) replay_meta: Option<&'a ReplayMeta>,
    pub(super) frame: &'a SaLiveFrame,
    pub(super) players: &'a [SaPlayerFrame],
    pub(super) player_ids: Vec<PlayerId>,
    pub(super) events: FrameEventsState,
    pub(super) event_history: &'a SaLiveEventHistory,
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

    pub(super) fn missing<T>(property: &'static str) -> SubtrActorResult<T> {
        SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState { property })
    }

    pub(crate) fn player_index(player_id: &PlayerId) -> Option<u32> {
        match player_id {
            RemoteId::SplitScreen(index) => Some(*index),
            _ => None,
        }
    }

    pub(super) fn player(&self, player_id: &PlayerId) -> SubtrActorResult<&SaPlayerFrame> {
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
