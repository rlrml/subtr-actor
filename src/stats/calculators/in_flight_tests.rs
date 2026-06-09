use super::*;

/// A minimal in-flight candidate for exercising the ledger. It accumulates a
/// count, commits once that count crosses a threshold, and decides its boundary
/// response based on whether it has committed.
#[derive(Debug, Clone, PartialEq)]
struct TestCandidate {
    label: &'static str,
    start_time: f32,
    start_frame: usize,
    samples: u32,
    committed: bool,
    /// Whether this candidate survives a `LivePlayEnded` boundary instead of
    /// resolving to it (used to exercise `Disposition::Keep` from a boundary).
    survives_live_play_end: bool,
}

impl TestCandidate {
    fn new(label: &'static str, start_time: f32, start_frame: usize) -> Self {
        Self {
            label,
            start_time,
            start_frame,
            samples: 0,
            committed: false,
            survives_live_play_end: false,
        }
    }
}

impl InFlightItem for TestCandidate {
    fn recognition(&self) -> Recognition {
        Recognition::new(self.start_time, self.start_frame, self.committed)
    }

    fn on_boundary(&mut self, boundary: Boundary) -> Disposition {
        if boundary == Boundary::LivePlayEnded && self.survives_live_play_end {
            return Disposition::Keep;
        }
        if self.committed {
            Disposition::Finalize(FinalizeReason::Boundary(boundary))
        } else {
            Disposition::Discard
        }
    }
}

#[test]
fn advance_keeps_finalizes_and_discards_in_one_pass() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    ledger.arm(TestCandidate::new("keep", 0.0, 0));
    ledger.arm(TestCandidate::new("finalize", 0.0, 0));
    ledger.arm(TestCandidate::new("discard", 0.0, 0));

    let finalized = ledger.advance(1.0, |candidate| match candidate.label {
        "finalize" => Disposition::Finalize(FinalizeReason::Completed),
        "discard" => Disposition::Discard,
        _ => Disposition::Keep,
    });

    assert_eq!(finalized.len(), 1);
    assert_eq!(finalized[0].0.label, "finalize");
    assert_eq!(finalized[0].1, FinalizeReason::Completed);
    // Only the kept candidate remains in flight.
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.in_flight()[0].label, "keep");
}

#[test]
fn advance_can_mutate_in_flight_state() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    ledger.arm(TestCandidate::new("a", 0.0, 0));

    let finalized = ledger.advance(0.5, |candidate| {
        candidate.samples += 1;
        Disposition::Keep
    });
    assert!(finalized.is_empty());
    assert_eq!(ledger.in_flight()[0].samples, 1);
}

#[test]
fn apply_boundary_finalizes_committed_and_discards_speculative() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    let mut committed = TestCandidate::new("committed", 0.0, 0);
    committed.committed = true;
    ledger.arm(committed);
    ledger.arm(TestCandidate::new("speculative", 0.0, 0));

    let finalized = ledger.apply_boundary(Boundary::GoalScored);

    assert_eq!(finalized.len(), 1);
    assert_eq!(finalized[0].0.label, "committed");
    assert_eq!(
        finalized[0].1,
        FinalizeReason::Boundary(Boundary::GoalScored)
    );
    // The speculative candidate was discarded; nothing is left in flight.
    assert!(ledger.is_empty());
}

#[test]
fn apply_boundary_can_keep_surviving_items() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    let mut survivor = TestCandidate::new("survivor", 0.0, 0);
    survivor.committed = true;
    survivor.survives_live_play_end = true;
    ledger.arm(survivor);

    let finalized = ledger.apply_boundary(Boundary::LivePlayEnded);
    assert!(finalized.is_empty());
    assert_eq!(ledger.len(), 1);

    // A different boundary still resolves it.
    let finalized = ledger.apply_boundary(Boundary::ReplayEnded);
    assert_eq!(finalized.len(), 1);
    assert!(ledger.is_empty());
}

#[test]
fn finish_flushes_everything_still_in_flight() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    let mut committed = TestCandidate::new("committed", 0.0, 0);
    committed.committed = true;
    ledger.arm(committed);
    ledger.arm(TestCandidate::new("speculative", 0.0, 0));

    let finalized = ledger.finish();
    assert_eq!(finalized.len(), 1);
    assert_eq!(
        finalized[0].1,
        FinalizeReason::Boundary(Boundary::ReplayEnded)
    );
    assert!(ledger.is_empty(), "finish must leave nothing in flight");
}

#[test]
fn finalize_all_drains_and_logs_every_item() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    ledger.arm(TestCandidate::new("a", 1.0, 10));
    ledger.arm(TestCandidate::new("b", 1.0, 11));

    let finalized = ledger.finalize_all(FinalizeReason::Superseded);
    assert_eq!(finalized.len(), 2);
    assert!(finalized.iter().all(|(_, reason)| *reason == FinalizeReason::Superseded));
    assert!(ledger.is_empty());
    // Both are now queryable as having happened (committed, by recognition time).
    assert!(ledger.happened_within(1.0, 1.0, true));
}

#[test]
fn happened_within_counts_finalized_events_by_recognition_time() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    let mut candidate = TestCandidate::new("c", 1.0, 10);
    candidate.committed = true;
    ledger.arm(candidate);
    // Finalizes at a later "now"; the recognition time (1.0) is what's logged.
    let finalized = ledger.advance(2.0, |_| Disposition::Finalize(FinalizeReason::Completed));
    assert_eq!(finalized.len(), 1);

    // Recently after recognition: counts.
    assert!(ledger.happened_within(2.0, 5.0, true));
    // Window that excludes the recognition time: does not count.
    assert!(!ledger.happened_within(9.0, 1.0, true));
}

#[test]
fn happened_within_is_latent_for_in_flight_candidates() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    ledger.arm(TestCandidate::new("speculative", 1.0, 10));

    // Without committed_only, an in-flight (not yet finalized) candidate counts.
    assert!(ledger.happened_within(1.5, 2.0, false));
    // With committed_only, a speculative candidate must not count.
    assert!(!ledger.happened_within(1.5, 2.0, true));

    // Once it commits, the committed query sees it.
    ledger.in_flight_mut()[0].committed = true;
    assert!(ledger.happened_within(1.5, 2.0, true));
}

#[test]
fn history_is_pruned_outside_the_window() {
    let mut ledger = InFlightLedger::<TestCandidate>::with_history_window(3.0);
    let mut candidate = TestCandidate::new("old", 0.0, 0);
    candidate.committed = true;
    ledger.arm(candidate);
    ledger.advance(0.0, |_| Disposition::Finalize(FinalizeReason::Completed));
    assert!(ledger.happened_within(0.0, 5.0, true));

    // Advancing time well past the window prunes the old recognition.
    ledger.advance(100.0, |_| Disposition::Keep);
    assert!(!ledger.happened_within(100.0, 50.0, true));
}

#[test]
fn discarded_items_do_not_appear_in_history() {
    let mut ledger = InFlightLedger::<TestCandidate>::new();
    ledger.arm(TestCandidate::new("speculative", 1.0, 10));
    let finalized = ledger.advance(1.5, |_| Disposition::Discard);
    assert!(finalized.is_empty());
    // A discarded candidate never "happened".
    assert!(!ledger.happened_within(1.5, 5.0, false));
}

#[test]
fn keyed_arm_replaces_and_looks_up_by_key() {
    let mut ledger = KeyedInFlightLedger::<u32, TestCandidate>::new();
    ledger.arm(7, TestCandidate::new("first", 0.0, 0));
    assert!(ledger.contains(&7));
    assert_eq!(ledger.get(&7).unwrap().label, "first");

    // Arming the same key replaces the item.
    ledger.arm(7, TestCandidate::new("second", 1.0, 1));
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.get(&7).unwrap().label, "second");

    ledger.get_mut(&7).unwrap().samples += 3;
    assert_eq!(ledger.get(&7).unwrap().samples, 3);
}

#[test]
fn keyed_advance_resolves_per_key() {
    let mut ledger = KeyedInFlightLedger::<u32, TestCandidate>::new();
    ledger.arm(1, TestCandidate::new("keep", 0.0, 0));
    ledger.arm(2, TestCandidate::new("finalize", 0.0, 0));
    ledger.arm(3, TestCandidate::new("discard", 0.0, 0));

    let finalized = ledger.advance(1.0, |_key, candidate| match candidate.label {
        "finalize" => Disposition::Finalize(FinalizeReason::Completed),
        "discard" => Disposition::Discard,
        _ => Disposition::Keep,
    });

    assert_eq!(finalized.len(), 1);
    assert_eq!(finalized[0].0, 2);
    assert_eq!(finalized[0].1.label, "finalize");
    assert_eq!(finalized[0].2, FinalizeReason::Completed);
    assert_eq!(ledger.len(), 1);
    assert!(ledger.contains(&1));
}

#[test]
fn keyed_finalize_logs_and_boundary_flushes() {
    let mut ledger = KeyedInFlightLedger::<u32, TestCandidate>::new();
    let mut committed = TestCandidate::new("c", 1.0, 10);
    committed.committed = true;
    ledger.arm(1, committed);

    // Imperative finalize removes, logs, and returns the item.
    let item = ledger.finalize(&1, FinalizeReason::Completed);
    assert!(item.is_some());
    assert!(ledger.is_empty());
    assert!(ledger.happened_within(1.0, 1.0, true));

    // Boundary handling mirrors the unkeyed ledger.
    let mut committed2 = TestCandidate::new("c2", 2.0, 21);
    committed2.committed = true;
    ledger.arm(2, TestCandidate::new("speculative", 2.0, 20));
    ledger.arm(3, committed2);
    let finalized = ledger.apply_boundary(Boundary::GoalScored);
    assert_eq!(finalized.len(), 1);
    assert_eq!(finalized[0].0, 3);
    assert!(ledger.is_empty(), "speculative discarded, committed finalized");
}

#[test]
fn keyed_entry_or_insert_with_inserts_once() {
    let mut ledger = KeyedInFlightLedger::<u32, TestCandidate>::new();
    ledger
        .entry_or_insert_with(5, || TestCandidate::new("created", 0.0, 0))
        .samples += 1;
    ledger
        .entry_or_insert_with(5, || TestCandidate::new("not-created", 9.0, 9))
        .samples += 1;
    assert_eq!(ledger.len(), 1);
    let candidate = ledger.get(&5).unwrap();
    assert_eq!(candidate.label, "created");
    assert_eq!(candidate.samples, 2);
}

#[test]
fn keyed_clear_discards_without_logging() {
    let mut ledger = KeyedInFlightLedger::<u32, TestCandidate>::new();
    ledger.arm(1, TestCandidate::new("a", 1.0, 10));
    ledger.arm(2, TestCandidate::new("b", 1.0, 11));
    ledger.clear();
    assert!(ledger.is_empty());
    assert!(!ledger.happened_within(1.0, 5.0, false));
}
