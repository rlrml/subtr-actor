use super::stats_timeline_events_mechanic_build_moments::append_moment_mechanic_events;
use super::stats_timeline_events_mechanic_build_spans::{
    append_ball_carry_events, append_span_mechanic_events,
};
use super::stats_timeline_events_mechanic_build_spans_tail::append_wavedash_events;
use super::*;

pub(super) fn build_mechanic_events(sources: &MechanicEventSources<'_>) -> Vec<MechanicEvent> {
    let mut events = Vec::new();
    append_ball_carry_events(&mut events, sources.ball_carry);
    append_span_mechanic_events(&mut events, sources);
    append_moment_mechanic_events(&mut events, sources);
    append_wavedash_events(&mut events, sources.wavedash);
    sort_mechanic_events(&mut events);
    events
}

fn sort_mechanic_events(events: &mut [MechanicEvent]) {
    events.sort_by(|left, right| {
        let left_time = mechanic_event_start_time(left);
        let right_time = mechanic_event_start_time(right);
        left_time
            .total_cmp(&right_time)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
}
