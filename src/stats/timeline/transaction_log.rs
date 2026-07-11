//! Keyed differential store for stats-timeline events.
//!
//! Each projection of the timeline (interim capture or `finish`) produces the
//! full current event set. [`TimelineTransactionLog`] diffs that set against
//! its previous state, appends one [`EventTransaction`] per change, and
//! enforces the lifecycle invariants that make incremental emission sound:
//!
//! - a [`Finalized`](EventLifecycle::Finalized) event's content never changes
//!   and it never disappears from a later projection;
//! - a [`Confirmed`](EventLifecycle::Confirmed) event never disappears from a
//!   later projection (it may only be revised or finalized).
//!
//! Violations are loud bugs, not accepted churn: in debug builds (and
//! therefore under `cargo test`) they surface as a [`SubtrActorError`] that
//! propagates out of the analysis graph the same way any node error does. In
//! release builds they are logged to stderr and the latest projection is
//! accepted, so a production live consumer degrades to eventually-consistent
//! output instead of aborting a match. (`debug_assertions` is the
//! repo-idiomatic switch for "strict in tests, tolerant in production" —
//! there is no logging facade in this crate to hang a softer policy on.)

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::{Event, EventLifecycle, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

/// One change to the reduced timeline-event view.
///
/// `seq` is the transaction's position in its log: assigned monotonically at
/// append time, strictly increasing, and never reused — a consumer can order,
/// deduplicate, or resume from transactions by `seq` alone.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum EventTransaction {
    /// A new event appeared, or an existing `meta.id`'s content was revised
    /// (including the `Confirmed` -> `Finalized` transition, which is visible
    /// in the carried event's `meta.lifecycle`).
    Upsert { seq: u64, event: Box<Event> },
    /// The event with this id was removed from the view. Nothing emits this
    /// today — it exists as a documented escape hatch (it is also the
    /// release-mode accept-latest fallback for an event that illegally
    /// vanished) — but consumers must handle it.
    Retract { seq: u64, id: String },
}

impl EventTransaction {
    /// The `meta.id` this transaction is about.
    pub fn event_id(&self) -> &str {
        match self {
            EventTransaction::Upsert { event, .. } => &event.meta.id,
            EventTransaction::Retract { id, .. } => id,
        }
    }

    /// The transaction's monotonic position in its log.
    pub fn seq(&self) -> u64 {
        match self {
            EventTransaction::Upsert { seq, .. } | EventTransaction::Retract { seq, .. } => *seq,
        }
    }
}

/// How [`TimelineTransactionLog::apply_projection_with_policy`] reacts to an
/// invariant violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantViolationPolicy {
    /// Return a [`SubtrActorError`] without recording any change from the
    /// offending projection (debug/test behavior).
    Error,
    /// Log the violation to stderr and accept the latest projection anyway
    /// (release behavior): a changed finalized event is upserted, a vanished
    /// event is retracted.
    AcceptLatest,
}

/// Differential store over successive full projections of the timeline event
/// set, keyed by `meta.id`.
///
/// The log is append-only; consumers keep their own cursor and read new
/// entries with [`transactions_since`](Self::transactions_since). The reduced
/// current view (the latest accepted projection) is what
/// `StatsTimelineEventsState::events` holds.
#[derive(Debug, Clone, Default)]
pub struct TimelineTransactionLog {
    /// Latest accepted version of each event, by `meta.id`.
    current: HashMap<String, Event>,
    /// Append-only change log.
    transactions: Vec<EventTransaction>,
}

impl TimelineTransactionLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Diffs `projection` (the full current event set) against the previous
    /// projection, appending transactions and enforcing the lifecycle
    /// invariants. Violations follow the build-dependent policy documented on
    /// the module: error in debug builds, log-and-accept in release builds.
    pub fn apply_projection(&mut self, projection: &[Event]) -> SubtrActorResult<()> {
        let policy = if cfg!(debug_assertions) {
            InvariantViolationPolicy::Error
        } else {
            InvariantViolationPolicy::AcceptLatest
        };
        self.apply_projection_with_policy(projection, policy)
    }

    /// [`apply_projection`](Self::apply_projection) with an explicit violation
    /// policy (exposed so tests can exercise the release-mode accept-latest
    /// path from a debug build).
    pub fn apply_projection_with_policy(
        &mut self,
        projection: &[Event],
        policy: InvariantViolationPolicy,
    ) -> SubtrActorResult<()> {
        // Validate the whole projection before recording anything, so the
        // error policy rejects an invalid projection atomically instead of
        // half-applying it.
        let mut projected_ids: HashSet<&str> = HashSet::with_capacity(projection.len());
        for event in projection {
            if !projected_ids.insert(event.meta.id.as_str()) {
                self.violation(
                    policy,
                    format!("projection contains duplicate event id {:?}", event.meta.id),
                )?;
            }
            if let Some(previous) = self.current.get(&event.meta.id)
                && previous != event
                && previous.meta.lifecycle == EventLifecycle::Finalized
            {
                self.violation(
                    policy,
                    format!(
                        "finalized event {:?} changed content in a later projection",
                        event.meta.id
                    ),
                )?;
            }
        }
        let vanished: Vec<String> = self
            .current
            .keys()
            .filter(|id| !projected_ids.contains(id.as_str()))
            .cloned()
            .collect();
        for id in &vanished {
            let lifecycle = self.current[id].meta.lifecycle;
            self.violation(
                policy,
                format!("{lifecycle:?} event {id:?} disappeared from a later projection"),
            )?;
        }

        // Record the accepted projection. Under the error policy this point is
        // only reached when there were no violations; under accept-latest the
        // projection wins wholesale (vanished events are retracted).
        let mut vanished = vanished;
        vanished.sort_unstable();
        for id in vanished {
            self.current.remove(&id);
            let seq = self.transactions.len() as u64;
            self.transactions
                .push(EventTransaction::Retract { seq, id });
        }
        for event in projection {
            let changed = self.current.get(&event.meta.id) != Some(event);
            if changed {
                self.current.insert(event.meta.id.clone(), event.clone());
                let seq = self.transactions.len() as u64;
                self.transactions.push(EventTransaction::Upsert {
                    seq,
                    event: Box::new(event.clone()),
                });
            }
        }
        Ok(())
    }

    /// Total number of transactions appended so far. A consumer that has read
    /// everything up to a previously observed count can pass that count to
    /// [`transactions_since`](Self::transactions_since) to get only the new
    /// entries. Because [`EventTransaction::seq`] is the append position, this
    /// is also one past the last appended `seq`.
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// The transactions appended after the first `seen` (a count previously
    /// returned by [`transaction_count`](Self::transaction_count); values past
    /// the end yield an empty slice).
    pub fn transactions_since(&self, seen: usize) -> &[EventTransaction] {
        &self.transactions[seen.min(self.transactions.len())..]
    }

    /// Every transaction appended since the log was created.
    pub fn transactions(&self) -> &[EventTransaction] {
        &self.transactions
    }

    /// The latest accepted version of `id`, if it is currently in the view.
    pub fn current_event(&self, id: &str) -> Option<&Event> {
        self.current.get(id)
    }

    /// The reduced current view, sorted chronologically (start time, then
    /// stream, then id — the same order the finish-time
    /// `StatsTimelineEventsState::events` list uses).
    pub fn current_events(&self) -> Vec<&Event> {
        let mut events: Vec<&Event> = self.current.values().collect();
        events.sort_by(|left, right| {
            left.meta
                .timing
                .start()
                .1
                .total_cmp(&right.meta.timing.start().1)
                .then_with(|| left.meta.stream.cmp(&right.meta.stream))
                .then_with(|| left.meta.id.cmp(&right.meta.id))
        });
        events
    }

    /// Number of events in the reduced current view.
    pub fn current_len(&self) -> usize {
        self.current.len()
    }

    /// Handles one invariant violation according to `policy`: `Error` returns
    /// (aborting the projection before it was recorded), `AcceptLatest` logs
    /// and lets the caller proceed.
    fn violation(&self, policy: InvariantViolationPolicy, message: String) -> SubtrActorResult<()> {
        match policy {
            InvariantViolationPolicy::Error => SubtrActorError::new_result(
                SubtrActorErrorVariant::TimelineEventInvariantViolation(message),
            ),
            InvariantViolationPolicy::AcceptLatest => {
                eprintln!(
                    "subtr-actor: timeline invariant violation (accepting latest): {message}"
                );
                Ok(())
            }
        }
    }
}

#[cfg(test)]
#[path = "transaction_log_tests.rs"]
mod tests;
