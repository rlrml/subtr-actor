use super::*;

impl BoostCalculator {
    pub(super) fn emit_pickup_comparison_event(
        &mut self,
        comparison: BoostPickupComparison,
        inferred: Option<PendingBoostPickupEvent>,
        reported: Option<PendingBoostPickupEvent>,
    ) {
        let reference = inferred.as_ref().or(reported.as_ref()).unwrap();
        let pad_type = reported
            .as_ref()
            .map(|event| event.pad_type)
            .or_else(|| inferred.as_ref().map(|event| event.pad_type))
            .unwrap_or(reference.pad_type);
        let field_half = reported
            .as_ref()
            .map(|event| event.field_half)
            .or_else(|| inferred.as_ref().map(|event| event.field_half))
            .unwrap_or(reference.field_half);
        let activity = reported
            .as_ref()
            .map(|event| event.activity)
            .or_else(|| inferred.as_ref().map(|event| event.activity))
            .unwrap_or(reference.activity);
        let event_frame = inferred
            .as_ref()
            .map(|event| event.frame)
            .or_else(|| reported.as_ref().map(|event| event.frame))
            .unwrap_or(reference.frame);
        let event_time = inferred
            .as_ref()
            .map(|event| event.time)
            .or_else(|| reported.as_ref().map(|event| event.time))
            .unwrap_or(reference.time);
        self.pickup_comparison_events
            .push(BoostPickupComparisonEvent {
                comparison,
                frame: event_frame,
                time: event_time,
                player_id: reference.player_id.clone(),
                is_team_0: reference.is_team_0,
                pad_type,
                field_half,
                activity,
                reported_frame: reported.as_ref().map(|event| event.frame),
                reported_time: reported.as_ref().map(|event| event.time),
                inferred_frame: inferred.as_ref().map(|event| event.frame),
                inferred_time: inferred.as_ref().map(|event| event.time),
                boost_before: inferred.as_ref().and_then(|event| event.boost_before),
                boost_after: inferred.as_ref().and_then(|event| event.boost_after),
            });
    }
}
