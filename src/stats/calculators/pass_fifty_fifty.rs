use super::*;

impl PassCalculator {
    pub(super) fn touch_from_fifty_fifty(
        touch: &TouchEvent,
        fifty_fifty_state: &FiftyFiftyState,
    ) -> bool {
        fifty_fifty_state
            .active_event
            .as_ref()
            .is_some_and(|event| {
                Self::fifty_fifty_involves_touch(
                    event.start_time,
                    event.last_touch_time,
                    event.team_zero_player.as_ref(),
                    event.team_one_player.as_ref(),
                    touch,
                )
            })
            || fifty_fifty_state
                .last_resolved_event
                .as_ref()
                .is_some_and(|event| {
                    Self::fifty_fifty_involves_touch(
                        event.start_time,
                        event.resolve_time,
                        event.team_zero_player.as_ref(),
                        event.team_one_player.as_ref(),
                        touch,
                    )
                })
    }

    fn fifty_fifty_involves_touch(
        start_time: f32,
        end_time: f32,
        team_zero_player: Option<&PlayerId>,
        team_one_player: Option<&PlayerId>,
        touch: &TouchEvent,
    ) -> bool {
        if touch.time < start_time || touch.time > end_time {
            return false;
        }

        match (touch.team_is_team_0, touch.player.as_ref()) {
            (true, Some(player)) => team_zero_player == Some(player),
            (false, Some(player)) => team_one_player == Some(player),
            _ => false,
        }
    }
}
