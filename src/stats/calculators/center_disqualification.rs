use super::*;

impl CenterCalculator {
    pub(super) fn player_has_disqualifying_event(
        events: &FrameEventsState,
        player: &PlayerId,
        is_team_0: bool,
    ) -> bool {
        has_player_shot(events, player, is_team_0)
            || has_player_or_team_goal(events, player, is_team_0)
    }

    pub(super) fn clear_disqualified_pending_center(&mut self, events: &FrameEventsState) {
        let should_clear = self.pending_touch.as_ref().is_some_and(|pending| {
            Self::player_has_disqualifying_event(events, &pending.player, pending.is_team_0)
        });
        if should_clear {
            self.pending_touch = None;
        }
    }
}

fn has_player_shot(events: &FrameEventsState, player: &PlayerId, is_team_0: bool) -> bool {
    events.player_stat_events.iter().any(|event| {
        event.kind == PlayerStatEventKind::Shot
            && event.player == *player
            && event.is_team_0 == is_team_0
    })
}

fn has_player_or_team_goal(events: &FrameEventsState, player: &PlayerId, is_team_0: bool) -> bool {
    events
        .goal_events
        .iter()
        .any(|event| match event.player.as_ref() {
            Some(scorer) => scorer == player,
            None => event.scoring_team_is_team_0 == is_team_0,
        })
}
