//! Incremental draining of normalized stats-timeline events.
//!
//! The analysis graph's single event surface is the graph-central append-only
//! [`TimelineTransactionLog`] (`AnalysisGraph::event_transaction_log`): interim
//! projections and finish's finalize-everything projection both feed it, and
//! its reduced `current_events` view is what batch consumers read after
//! finish. Log entries carry the event lifecycle (`Confirmed` events may
//! still be revised in place; `Finalized` events are immutable — the log
//! enforces both). Two consumption styles exist:
//!
//! - [`TimelineEventDrain`] mirrors the BakkesMod FFI's timeline drain over
//!   the reduced list: the first version of an event id wins and later
//!   in-place revisions of an already-drained id are *not* re-emitted.
//! - [`TimelineTransactionCursor`] reads the transaction log itself, so a
//!   consumer sees every upsert (new event, revision, finalization) and
//!   retract exactly once, in order.

use std::collections::HashSet;

use subtr_actor::{Event, EventTransaction, TimelineTransactionLog};

/// Drains newly-appended timeline events, deduplicating by `event.meta.id`.
#[derive(Debug, Clone, Default)]
pub struct TimelineEventDrain {
    seen_event_ids: HashSet<String>,
}

impl TimelineEventDrain {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns clones of the events whose `meta.id` has not been seen before,
    /// preserving input order, and marks those ids as seen.
    pub fn drain_new<'a, I>(&mut self, events: I) -> Vec<Event>
    where
        I: IntoIterator<Item = &'a Event>,
    {
        events
            .into_iter()
            .filter(|event| self.seen_event_ids.insert(event.meta.id.clone()))
            .cloned()
            .collect()
    }

    /// Number of distinct event ids drained so far.
    pub fn seen_count(&self) -> usize {
        self.seen_event_ids.len()
    }

    /// Forgets all seen ids (e.g. when the underlying graph is rebuilt for a
    /// new match, whose cumulative event list starts over).
    pub fn reset(&mut self) {
        self.seen_event_ids.clear();
    }
}

/// A cursor over a [`TimelineTransactionLog`]: each [`drain`](Self::drain)
/// returns exactly the transactions appended since the previous drain.
///
/// The log lives inside the graph (read through
/// `AnalysisGraph::event_transaction_log`), so the cursor holds only a
/// position — pair one cursor with one graph, and [`reset`](Self::reset) it
/// whenever the graph is rebuilt (new match), which starts a fresh, empty
/// log.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimelineTransactionCursor {
    seen: usize,
}

impl TimelineTransactionCursor {
    pub fn new() -> Self {
        Self::default()
    }

    /// The transactions appended since the last `drain`, advancing the cursor
    /// past them.
    pub fn drain<'a>(&mut self, log: &'a TimelineTransactionLog) -> &'a [EventTransaction] {
        let new = log.transactions_since(self.seen);
        self.seen = log.transaction_count();
        new
    }

    /// Number of transactions consumed so far.
    pub fn seen_count(&self) -> usize {
        self.seen
    }

    /// Rewinds to the start (for a rebuilt graph whose log starts over).
    pub fn reset(&mut self) {
        self.seen = 0;
    }
}

#[cfg(test)]
#[path = "drain_tests.rs"]
mod tests;
