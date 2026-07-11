use subtr_actor::{
    Event, EventLifecycle, EventMeta, EventPayload, EventScope, EventTiming, TimelineEvent,
    TimelineEventKind, TimelineTransactionLog,
};

use super::*;

fn event(id: &str, time: f32) -> Event {
    event_with_lifecycle(id, EventLifecycle::Confirmed, time)
}

fn event_with_lifecycle(id: &str, lifecycle: EventLifecycle, time: f32) -> Event {
    Event {
        meta: EventMeta {
            id: id.to_owned(),
            stream: "timeline".to_owned(),
            label: "Timeline".to_owned(),
            scope: EventScope::Match,
            lifecycle,
            timing: EventTiming::Moment { frame: 10, time },
            primary_player: None,
            secondary_player: None,
            player_position: None,
            ball_position: None,
            team_is_team_0: None,
            confidence: None,
            properties: Vec::new(),
        },
        payload: EventPayload::Timeline(TimelineEvent {
            time,
            frame: Some(10),
            kind: TimelineEventKind::Goal,
            player_id: None,
            player_position: None,
            is_team_0: None,
        }),
    }
}

#[test]
fn drains_only_unseen_ids_in_order() {
    let mut drain = TimelineEventDrain::new();
    let first = [event("a", 1.0), event("b", 2.0)];
    let drained = drain.drain_new(&first);
    assert_eq!(
        drained
            .iter()
            .map(|e| e.meta.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );

    // Cumulative list grows; only the new id comes out.
    let second = [event("a", 1.0), event("b", 2.0), event("c", 3.0)];
    let drained = drain.drain_new(&second);
    assert_eq!(
        drained
            .iter()
            .map(|e| e.meta.id.as_str())
            .collect::<Vec<_>>(),
        ["c"]
    );
    assert_eq!(drain.seen_count(), 3);
}

#[test]
fn revised_events_with_seen_ids_are_not_re_emitted() {
    let mut drain = TimelineEventDrain::new();
    drain.drain_new(&[event("a", 1.0)]);
    // Same id, revised in place (e.g. a promote-only upgrade): not re-emitted.
    let drained = drain.drain_new(&[event("a", 99.0)]);
    assert!(drained.is_empty());
}

#[test]
fn duplicate_ids_within_one_poll_are_deduped() {
    let mut drain = TimelineEventDrain::new();
    let drained = drain.drain_new(&[event("a", 1.0), event("a", 1.0)]);
    assert_eq!(drained.len(), 1);
}

#[test]
fn reset_forgets_seen_ids() {
    let mut drain = TimelineEventDrain::new();
    drain.drain_new(&[event("a", 1.0)]);
    drain.reset();
    assert_eq!(drain.seen_count(), 0);
    assert_eq!(drain.drain_new(&[event("a", 1.0)]).len(), 1);
}

#[test]
fn transaction_cursor_drains_each_transaction_exactly_once() {
    let mut log = TimelineTransactionLog::new();
    let mut cursor = TimelineTransactionCursor::new();

    log.apply_projection(&[event("a", 1.0)]).expect("applies");
    let drained = cursor.drain(&log);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].event_id(), "a");

    // Nothing new: nothing drained.
    assert!(cursor.drain(&log).is_empty());

    // A revision and a new event drain in append order, once.
    log.apply_projection(&[event("a", 1.5), event("b", 2.0)])
        .expect("applies");
    let drained = cursor.drain(&log);
    assert_eq!(
        drained
            .iter()
            .map(|transaction| transaction.event_id())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(cursor.seen_count(), 3);
    assert!(cursor.drain(&log).is_empty());
}

#[test]
fn transaction_cursor_observes_finalization_as_an_upsert() {
    let mut log = TimelineTransactionLog::new();
    let mut cursor = TimelineTransactionCursor::new();

    log.apply_projection(&[event("a", 1.0)]).expect("applies");
    cursor.drain(&log);

    log.apply_projection(&[event_with_lifecycle("a", EventLifecycle::Finalized, 1.0)])
        .expect("applies");
    let drained = cursor.drain(&log);
    assert_eq!(drained.len(), 1);
    match &drained[0] {
        subtr_actor::EventTransaction::Upsert { event, .. } => {
            assert_eq!(event.meta.lifecycle, EventLifecycle::Finalized);
        }
        other => panic!("expected an upsert, got {other:?}"),
    }
}

#[test]
fn transaction_cursor_reset_rewinds_to_the_start() {
    let mut log = TimelineTransactionLog::new();
    let mut cursor = TimelineTransactionCursor::new();
    log.apply_projection(&[event("a", 1.0)]).expect("applies");
    cursor.drain(&log);

    cursor.reset();
    assert_eq!(cursor.seen_count(), 0);
    assert_eq!(cursor.drain(&log).len(), 1);
}
