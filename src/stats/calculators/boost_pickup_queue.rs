use super::*;

impl BoostCalculator {
    fn matching_pending_pickup_index(
        pending: &VecDeque<PendingBoostPickupEvent>,
        event: &PendingBoostPickupEvent,
        pending_is_inferred: bool,
    ) -> Option<usize> {
        pending
            .iter()
            .enumerate()
            .filter(|(_, pending_event)| {
                pending_event.player_id == event.player_id
                    && if pending_is_inferred {
                        pending_event.pad_type.is_compatible_with(event.pad_type)
                    } else {
                        event.pad_type.is_compatible_with(pending_event.pad_type)
                    }
                    && pending_event.frame.abs_diff(event.frame) <= Self::PICKUP_MATCH_FRAME_WINDOW
            })
            .min_by_key(|(_, pending_event)| pending_event.frame.abs_diff(event.frame))
            .map(|(index, _)| index)
    }

    pub(super) fn record_inferred_pickup(&mut self, event: PendingBoostPickupEvent) {
        self.pending_inferred_pickups.push_back(event);
    }

    pub(super) fn record_reported_pickup(&mut self, event: PendingBoostPickupEvent) {
        if let Some(index) =
            Self::matching_pending_pickup_index(&self.pending_inferred_pickups, &event, true)
        {
            let inferred = self
                .pending_inferred_pickups
                .remove(index)
                .expect("matched inferred pickup index should exist");
            self.emit_pickup_comparison_event(
                BoostPickupComparison::Both,
                Some(inferred),
                Some(event),
            );
        } else {
            self.emit_pickup_comparison_event(BoostPickupComparison::Both, None, Some(event));
        }
    }

    pub(super) fn flush_stale_pickup_comparisons(&mut self, current_frame: usize) {
        while self
            .pending_inferred_pickups
            .front()
            .is_some_and(|event| event.frame + Self::PICKUP_MATCH_FRAME_WINDOW < current_frame)
        {
            self.pending_inferred_pickups.pop_front();
        }
    }
}
