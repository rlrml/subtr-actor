use super::*;
use crate::{
    EventMeta, EventPayload, EventScope, EventTiming, TimelineEvent, TimelineEventKind,
    stats_timeline_event_label,
};

fn event(id: &str, lifecycle: EventLifecycle, time: f32) -> Event {
    Event {
        meta: EventMeta {
            id: id.to_owned(),
            stream: "timeline".to_owned(),
            label: stats_timeline_event_label("timeline"),
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

fn upserted_ids(transactions: &[EventTransaction]) -> Vec<&str> {
    transactions
        .iter()
        .map(|transaction| match transaction {
            EventTransaction::Upsert { event, .. } => event.meta.id.as_str(),
            EventTransaction::Retract { id, .. } => panic!("unexpected retract of {id}"),
        })
        .collect()
}

#[test]
fn new_events_upsert_and_unchanged_events_are_silent() {
    let mut log = TimelineTransactionLog::new();
    let confirmed_a = event("a", EventLifecycle::Confirmed, 1.0);
    let confirmed_b = event("b", EventLifecycle::Confirmed, 2.0);

    log.apply_projection(&[confirmed_a.clone(), confirmed_b.clone()])
        .expect("fresh projection applies");
    assert_eq!(upserted_ids(log.transactions()), ["a", "b"]);

    // Re-projecting identical content records nothing.
    log.apply_projection(&[confirmed_a, confirmed_b])
        .expect("identical projection applies");
    assert_eq!(log.transaction_count(), 2);
}

#[test]
fn confirmed_events_can_be_revised_and_finalized() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[event("a", EventLifecycle::Confirmed, 1.0)])
        .expect("fresh projection applies");

    // Content revision of a confirmed event is an upsert.
    log.apply_projection(&[event("a", EventLifecycle::Confirmed, 1.5)])
        .expect("confirmed revision applies");
    // Confirmed -> Finalized is an upsert carrying the new lifecycle.
    log.apply_projection(&[event("a", EventLifecycle::Finalized, 1.5)])
        .expect("finalization applies");

    assert_eq!(upserted_ids(log.transactions()), ["a", "a", "a"]);
    assert_eq!(
        log.current_event("a").expect("a is current").meta.lifecycle,
        EventLifecycle::Finalized
    );
}

#[test]
fn finalized_content_change_is_an_error_and_applies_nothing() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[event("a", EventLifecycle::Finalized, 1.0)])
        .expect("fresh projection applies");

    let error = log
        .apply_projection(&[event("a", EventLifecycle::Finalized, 2.0)])
        .expect_err("changing a finalized event must fail");
    assert!(matches!(
        error.variant,
        SubtrActorErrorVariant::TimelineEventInvariantViolation(_)
    ));
    // The offending projection was rejected atomically.
    assert_eq!(log.transaction_count(), 1);
    assert_eq!(
        log.current_event("a").expect("a is current"),
        &event("a", EventLifecycle::Finalized, 1.0)
    );
}

#[test]
fn lifecycle_downgrade_is_a_finalized_content_change() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[event("a", EventLifecycle::Finalized, 1.0)])
        .expect("fresh projection applies");
    log.apply_projection(&[event("a", EventLifecycle::Confirmed, 1.0)])
        .expect_err("finalized -> confirmed must fail");
}

#[test]
fn vanished_event_is_an_error() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[
        event("a", EventLifecycle::Confirmed, 1.0),
        event("b", EventLifecycle::Finalized, 2.0),
    ])
    .expect("fresh projection applies");

    for kept in ["a", "b"] {
        let error = log
            .apply_projection(&[log.current_event(kept).expect("current").clone()])
            .expect_err("dropping an event must fail");
        assert!(matches!(
            error.variant,
            SubtrActorErrorVariant::TimelineEventInvariantViolation(_)
        ));
    }
    assert_eq!(log.current_len(), 2);
}

#[test]
fn duplicate_id_in_projection_is_an_error() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[
        event("a", EventLifecycle::Confirmed, 1.0),
        event("a", EventLifecycle::Confirmed, 1.0),
    ])
    .expect_err("duplicate ids must fail");
}

#[test]
fn accept_latest_policy_retracts_vanished_and_accepts_finalized_changes() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[
        event("a", EventLifecycle::Finalized, 1.0),
        event("b", EventLifecycle::Finalized, 2.0),
    ])
    .expect("fresh projection applies");

    // Release behavior: the violating projection wins wholesale — the changed
    // finalized event is upserted and the vanished one retracted.
    log.apply_projection_with_policy(
        &[event("a", EventLifecycle::Finalized, 9.0)],
        InvariantViolationPolicy::AcceptLatest,
    )
    .expect("accept-latest never errors");

    let new_transactions = log.transactions_since(2);
    assert_eq!(new_transactions.len(), 2);
    assert!(matches!(
        &new_transactions[0],
        EventTransaction::Retract { id, .. } if id == "b"
    ));
    assert!(matches!(
        &new_transactions[1],
        EventTransaction::Upsert { event, .. } if event.meta.id == "a"
    ));
    assert_eq!(log.current_len(), 1);
    assert!(log.current_event("b").is_none());
}

#[test]
fn seq_is_the_strictly_increasing_append_position() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[event("a", EventLifecycle::Confirmed, 1.0)])
        .expect("fresh projection applies");
    log.apply_projection(&[
        event("a", EventLifecycle::Finalized, 1.0),
        event("b", EventLifecycle::Confirmed, 2.0),
    ])
    .expect("revision applies");

    let seqs: Vec<u64> = log
        .transactions()
        .iter()
        .map(|transaction| transaction.seq())
        .collect();
    assert_eq!(seqs, [0, 1, 2], "seq must be the append position");
    // A cursor at a previously observed count resumes exactly past that seq.
    assert_eq!(log.transactions_since(1)[0].seq(), 1);
}

#[test]
fn current_events_is_the_chronologically_sorted_reduced_view() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[
        event("late", EventLifecycle::Confirmed, 9.0),
        event("early", EventLifecycle::Confirmed, 1.0),
    ])
    .expect("fresh projection applies");
    let ids: Vec<&str> = log
        .current_events()
        .iter()
        .map(|event| event.meta.id.as_str())
        .collect();
    assert_eq!(ids, ["early", "late"]);
}

#[test]
fn transactions_since_clamps_past_the_end() {
    let mut log = TimelineTransactionLog::new();
    log.apply_projection(&[event("a", EventLifecycle::Confirmed, 1.0)])
        .expect("fresh projection applies");
    assert_eq!(log.transactions_since(0).len(), 1);
    assert!(log.transactions_since(1).is_empty());
    assert!(log.transactions_since(99).is_empty());
}
