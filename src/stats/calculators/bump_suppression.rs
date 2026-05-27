use super::*;

impl BumpCalculator {
    pub(super) fn is_recent_demo_pair(
        &self,
        frame_events: &FrameEventsState,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        frame_events.demo_events.iter().any(|demo| {
            (&demo.attacker == left && &demo.victim == right)
                || (&demo.attacker == right && &demo.victim == left)
        }) || frame_events.active_demos.iter().any(|demo| {
            (&demo.attacker == left && &demo.victim == right)
                || (&demo.attacker == right && &demo.victim == left)
        })
    }

    pub(super) fn is_recent_fifty_fifty_pair(
        frame: &FrameInfo,
        fifty_fifty_state: &FiftyFiftyState,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        if fifty_fifty_state
            .active_event
            .as_ref()
            .is_some_and(|event| Self::active_fifty_fifty_matches_pair(event, left, right))
        {
            return true;
        }

        fifty_fifty_state
            .resolved_events
            .iter()
            .any(|event| Self::resolved_fifty_fifty_matches_pair(event, left, right))
            || fifty_fifty_state
                .last_resolved_event
                .as_ref()
                .is_some_and(|event| {
                    frame.time - event.resolve_time <= BUMP_FIFTY_FIFTY_SUPPRESSION_WINDOW_SECONDS
                        && Self::resolved_fifty_fifty_matches_pair(event, left, right)
                })
    }

    pub(super) fn should_count_bump(
        &mut self,
        initiator: &PlayerId,
        victim: &PlayerId,
        frame_number: usize,
    ) -> bool {
        let key = (initiator.clone(), victim.clone());
        let already_counted = self
            .last_seen_pair_frame
            .get(&key)
            .map(|previous_frame| {
                frame_number.saturating_sub(*previous_frame) <= BUMP_REPEAT_FRAME_WINDOW
            })
            .unwrap_or(false);
        self.last_seen_pair_frame.insert(key, frame_number);
        !already_counted
    }
}
