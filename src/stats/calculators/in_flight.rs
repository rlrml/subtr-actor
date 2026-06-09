//! Uniform lifecycle for *in-flight events* — analysis results that are
//! recognized on one frame but only finalized later, once enough subsequent
//! frames have been observed.
//!
//! Many calculators share the same shape: arm a pending/"active" candidate when
//! something is first recognized, fold in metadata over subsequent frames, then
//! emit a finalized event once a completion condition is met. Historically each
//! calculator hand-rolled this, including the easy-to-forget part: making sure a
//! candidate that is still in flight when a *boundary* occurs (a goal, play
//! leaving the live phase, or the end of the replay) is finalized or discarded
//! rather than silently leaking or absorbing data across the boundary.
//!
//! Two containers are offered, sharing the same vocabulary ([`Boundary`],
//! [`FinalizeReason`], [`Recognition`], [`Disposition`], [`InFlightItem`]) and
//! the same latent-query semantics:
//!   * [`InFlightLedger`] holds an unordered set of in-flight items;
//!   * [`KeyedInFlightLedger`] holds at most one in-flight item per key (e.g.
//!     per player), for calculators that look candidates up by subject.
//!
//! Both centralize boundary handling ([`apply_boundary`](InFlightLedger::apply_boundary),
//! `finish`) so a calculator cannot forget a boundary, and both log every
//! recognition so other nodes can ask "did this happen recently?" *latently* —
//! counting events that are recognized but not yet finalized, with a
//! `committed_only` gate so speculative candidates don't give false positives.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// Default span, in seconds, over which finalized recognitions are retained for
/// latent queries. Generous enough for "did X happen recently" questions while
/// keeping the history bounded.
pub const DEFAULT_HISTORY_WINDOW_SECONDS: f32 = 10.0;

/// A coarse game-flow boundary at which pending analysis work must resolve.
///
/// The set is deliberately limited to boundaries the replay model can actually
/// express. (There is no period/half/overtime concept in the data, so none is
/// offered here.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Boundary {
    /// Play left the live ("active") phase: a goal, a whistle, or a kickoff
    /// reset. Derived from `LivePlayState::is_live_play` going false.
    LivePlayEnded,
    /// A goal was scored. Derived from `FrameEventsState::goal_events`.
    GoalScored,
    /// The replay stream ended; nothing more will ever be observed. Delivered
    /// from each node's `finish`.
    ReplayEnded,
}

/// Why an in-flight item was finalized into an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinalizeReason {
    /// Reached its natural completion condition (window elapsed, follow-up
    /// observed, possession resolved, ...).
    Completed,
    /// Replaced by a newer candidate for the same subject before completing.
    Superseded,
    /// Cut short by a game-flow [`Boundary`] before natural completion. The
    /// resulting event's measured window is truncated at the boundary.
    Boundary(Boundary),
}

/// A cheap, early record that an event has been recognized.
///
/// A recognition is available from the moment a candidate is armed — before the
/// full event payload exists. This is what lets other analysis nodes ask
/// whether an event happened *recently* even when it is still in flight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Recognition {
    pub time: f32,
    pub frame: usize,
    /// `false` while the candidate is still speculative (it may yet be
    /// discarded rather than finalized); `true` once it is certain to emit.
    pub committed: bool,
}

impl Recognition {
    pub fn new(time: f32, frame: usize, committed: bool) -> Self {
        Self {
            time,
            frame,
            committed,
        }
    }

    /// A committed recognition (certain to produce an event).
    pub fn committed(time: f32, frame: usize) -> Self {
        Self::new(time, frame, true)
    }

    /// A speculative recognition (may still be discarded).
    pub fn speculative(time: f32, frame: usize) -> Self {
        Self::new(time, frame, false)
    }
}

/// What should happen to an in-flight item, as decided either by a per-frame
/// step or by a [`Boundary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    /// Keep accumulating; the item stays in flight.
    Keep,
    /// Finalize now, for the given reason. The item is removed and handed back
    /// to the caller to convert into an event.
    Finalize(FinalizeReason),
    /// Abandon the item without emitting anything.
    Discard,
}

/// An item that can be held in flight by a ledger.
///
/// The per-frame accumulation logic stays in the owning calculator (its inputs
/// are calculator-specific); the trait only covers what a ledger must do
/// uniformly: expose a [`Recognition`] for latent queries, and decide how to
/// respond when a [`Boundary`] forces resolution.
pub trait InFlightItem {
    fn recognition(&self) -> Recognition;

    /// How this item responds to `boundary`. Committed items that have earned
    /// an event typically return `Finalize(FinalizeReason::Boundary(boundary))`;
    /// speculative candidates that have not yet earned one typically `Discard`.
    fn on_boundary(&mut self, boundary: Boundary) -> Disposition;
}

/// Bounded, time-indexed record of finalized recognitions, shared by both
/// ledgers to answer latent "did X happen recently?" queries.
#[derive(Debug, Clone, PartialEq)]
struct RecognitionLog {
    finalized: VecDeque<Recognition>,
    history_window: f32,
    last_time: f32,
}

impl RecognitionLog {
    fn with_history_window(history_window: f32) -> Self {
        Self {
            finalized: VecDeque::new(),
            history_window,
            last_time: 0.0,
        }
    }

    fn observe_time(&mut self, now: f32) {
        self.last_time = self.last_time.max(now);
    }

    fn log(&mut self, item: &impl InFlightItem) {
        let mut recognition = item.recognition();
        recognition.committed = true;
        // Keep `last_time` roughly current even for imperative flows that don't
        // observe a frame time, so the history still prunes (recognition time is
        // a lower bound on the current time).
        self.last_time = self.last_time.max(recognition.time);
        self.finalized.push_back(recognition);
    }

    fn prune(&mut self) {
        let cutoff = self.last_time - self.history_window;
        while self.finalized.front().is_some_and(|rec| rec.time < cutoff) {
            self.finalized.pop_front();
        }
    }

    fn finalized_within(
        &self,
        now: f32,
        window: f32,
        committed_only: bool,
    ) -> impl Iterator<Item = Recognition> + '_ {
        self.finalized
            .iter()
            .copied()
            .filter(move |rec| in_window(rec, now, window, committed_only))
    }
}

fn in_window(rec: &Recognition, now: f32, window: f32, committed_only: bool) -> bool {
    rec.time <= now && now - rec.time <= window && (!committed_only || rec.committed)
}

/// Holds an unordered set of in-flight items, drives their lifecycle uniformly,
/// and records recognitions so finalized and in-flight events alike can be
/// queried latently.
#[derive(Debug, Clone, PartialEq)]
pub struct InFlightLedger<C> {
    active: Vec<C>,
    log: RecognitionLog,
}

impl<C> Default for InFlightLedger<C> {
    fn default() -> Self {
        Self::with_history_window(DEFAULT_HISTORY_WINDOW_SECONDS)
    }
}

impl<C> InFlightLedger<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_history_window(history_window: f32) -> Self {
        Self {
            active: Vec::new(),
            log: RecognitionLog::with_history_window(history_window),
        }
    }

    /// Arm a new in-flight item.
    pub fn arm(&mut self, item: C) {
        self.active.push(item);
    }

    /// The items currently in flight.
    pub fn in_flight(&self) -> &[C] {
        &self.active
    }

    /// Mutable access to the items currently in flight, for per-frame
    /// accumulation that does not change the in-flight set.
    pub fn in_flight_mut(&mut self) -> &mut [C] {
        &mut self.active
    }

    pub fn any_in_flight(&self) -> bool {
        !self.active.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }
}

impl<C: InFlightItem> InFlightLedger<C> {
    /// Advance every in-flight item with `step`, which folds in this frame's
    /// data and returns a [`Disposition`]. Finalized and discarded items are
    /// removed; finalized items are logged for latent queries and returned with
    /// their reason for the caller to turn into events. `now` is the current
    /// frame time, used to bound the latent-query history.
    pub fn advance(
        &mut self,
        now: f32,
        mut step: impl FnMut(&mut C) -> Disposition,
    ) -> Vec<(C, FinalizeReason)> {
        self.log.observe_time(now);
        let finalized = self.resolve_each(|item| step(item));
        self.log.prune();
        finalized
    }

    /// Apply `boundary` to every in-flight item via [`InFlightItem::on_boundary`].
    /// This is the uniform replacement for hand-placed flush calls: a single
    /// call resolves *every* pending item against the boundary, so none can be
    /// forgotten.
    pub fn apply_boundary(&mut self, boundary: Boundary) -> Vec<(C, FinalizeReason)> {
        let finalized = self.resolve_each(|item| item.on_boundary(boundary));
        self.log.prune();
        finalized
    }

    /// Finalize everything still in flight at end of stream. Calling this from a
    /// node's `finish` guarantees no in-flight item is ever silently dropped.
    pub fn finish(&mut self) -> Vec<(C, FinalizeReason)> {
        self.apply_boundary(Boundary::ReplayEnded)
    }

    /// Imperatively finalize every in-flight item with `reason`, returning them
    /// for the caller to convert into events. For calculators that drive
    /// finalization from specific code paths (e.g. emit-early, patch-in-place)
    /// rather than a per-frame step or a [`Boundary`].
    pub fn finalize_all(&mut self, reason: FinalizeReason) -> Vec<(C, FinalizeReason)> {
        let mut finalized = Vec::with_capacity(self.active.len());
        for item in std::mem::take(&mut self.active) {
            self.log.log(&item);
            finalized.push((item, reason));
        }
        self.log.prune();
        finalized
    }

    fn resolve_each(
        &mut self,
        mut decide: impl FnMut(&mut C) -> Disposition,
    ) -> Vec<(C, FinalizeReason)> {
        let mut finalized = Vec::new();
        let mut i = 0;
        while i < self.active.len() {
            match decide(&mut self.active[i]) {
                Disposition::Keep => i += 1,
                Disposition::Discard => {
                    self.active.remove(i);
                }
                Disposition::Finalize(reason) => {
                    let item = self.active.remove(i);
                    self.log.log(&item);
                    finalized.push((item, reason));
                }
            }
        }
        finalized
    }

    /// Whether an event was recognized within `window` seconds before `now`,
    /// counting both finalized events and items still in flight.
    ///
    /// With `committed_only`, speculative (not-yet-committed) in-flight items
    /// are ignored — use it for "did X *happen*?" questions that must not be
    /// fooled by a candidate that may still be discarded. Without it, the query
    /// is fully latent: a just-recognized, not-yet-finalized candidate counts.
    pub fn happened_within(&self, now: f32, window: f32, committed_only: bool) -> bool {
        self.recognitions_within(now, window, committed_only)
            .next()
            .is_some()
    }

    /// All recognitions within `window` seconds before `now`, across in-flight
    /// and finalized items. See [`happened_within`](Self::happened_within) for
    /// the meaning of `committed_only`.
    pub fn recognitions_within(
        &self,
        now: f32,
        window: f32,
        committed_only: bool,
    ) -> impl Iterator<Item = Recognition> + '_ {
        let active = self
            .active
            .iter()
            .map(C::recognition)
            .filter(move |rec| in_window(rec, now, window, committed_only));
        active.chain(self.log.finalized_within(now, window, committed_only))
    }
}

/// Holds at most one in-flight item per key, drives their lifecycle uniformly,
/// and records recognitions for latent queries. The keyed analogue of
/// [`InFlightLedger`], for calculators that track a candidate per subject (e.g.
/// per player).
#[derive(Debug, Clone)]
pub struct KeyedInFlightLedger<K, C> {
    active: HashMap<K, C>,
    log: RecognitionLog,
}

impl<K: Eq + Hash, C: PartialEq> PartialEq for KeyedInFlightLedger<K, C> {
    fn eq(&self, other: &Self) -> bool {
        self.active == other.active && self.log == other.log
    }
}

impl<K, C> Default for KeyedInFlightLedger<K, C> {
    fn default() -> Self {
        Self::with_history_window(DEFAULT_HISTORY_WINDOW_SECONDS)
    }
}

impl<K, C> KeyedInFlightLedger<K, C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_history_window(history_window: f32) -> Self {
        Self {
            active: HashMap::new(),
            log: RecognitionLog::with_history_window(history_window),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }
}

impl<K: Eq + Hash + Clone, C> KeyedInFlightLedger<K, C> {
    /// Arm (or replace) the in-flight item for `key`.
    pub fn arm(&mut self, key: K, item: C) {
        self.active.insert(key, item);
    }

    pub fn contains(&self, key: &K) -> bool {
        self.active.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&C> {
        self.active.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut C> {
        self.active.get_mut(key)
    }

    /// Access the in-flight item for `key`, inserting one produced by `default`
    /// if absent. Mirrors `HashMap::entry(..).or_insert_with(..)`.
    pub fn entry_or_insert_with(&mut self, key: K, default: impl FnOnce() -> C) -> &mut C {
        match self.active.entry(key) {
            Entry::Occupied(occupied) => occupied.into_mut(),
            Entry::Vacant(vacant) => vacant.insert(default()),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> + '_ {
        self.active.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &C> + '_ {
        self.active.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut C> + '_ {
        self.active.values_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &C)> + '_ {
        self.active.iter()
    }

    /// Remove the item for `key` without recording it as having happened (it is
    /// abandoned, not finalized).
    pub fn discard(&mut self, key: &K) -> Option<C> {
        self.active.remove(key)
    }

    /// Discard every in-flight item without finalizing any (e.g. abandoning all
    /// candidates when their precondition no longer holds).
    pub fn clear(&mut self) {
        self.active.clear();
    }
}

impl<K: Eq + Hash + Clone, C: InFlightItem> KeyedInFlightLedger<K, C> {
    /// Finalize the item for `key`, logging it for latent queries and returning
    /// it for the caller to convert into an event.
    pub fn finalize(&mut self, key: &K, _reason: FinalizeReason) -> Option<C> {
        let item = self.active.remove(key)?;
        self.log.log(&item);
        Some(item)
    }

    /// Advance every in-flight item with `step`, which receives the key and a
    /// mutable item and returns a [`Disposition`]. `now` bounds the latent-query
    /// history. Finalized items are returned with their key and reason.
    pub fn advance(
        &mut self,
        now: f32,
        mut step: impl FnMut(&K, &mut C) -> Disposition,
    ) -> Vec<(K, C, FinalizeReason)> {
        self.log.observe_time(now);
        let finalized = self.resolve_each(|key, item| step(key, item));
        self.log.prune();
        finalized
    }

    /// Apply `boundary` to every in-flight item via [`InFlightItem::on_boundary`].
    pub fn apply_boundary(&mut self, boundary: Boundary) -> Vec<(K, C, FinalizeReason)> {
        let finalized = self.resolve_each(|_key, item| item.on_boundary(boundary));
        self.log.prune();
        finalized
    }

    /// Finalize everything still in flight at end of stream.
    pub fn finish(&mut self) -> Vec<(K, C, FinalizeReason)> {
        self.apply_boundary(Boundary::ReplayEnded)
    }

    fn resolve_each(
        &mut self,
        mut decide: impl FnMut(&K, &mut C) -> Disposition,
    ) -> Vec<(K, C, FinalizeReason)> {
        let mut finalized = Vec::new();
        let mut remove_finalize: Vec<(K, FinalizeReason)> = Vec::new();
        let mut remove_discard: Vec<K> = Vec::new();
        for (key, item) in self.active.iter_mut() {
            match decide(key, item) {
                Disposition::Keep => {}
                Disposition::Discard => remove_discard.push(key.clone()),
                Disposition::Finalize(reason) => remove_finalize.push((key.clone(), reason)),
            }
        }
        for key in remove_discard {
            self.active.remove(&key);
        }
        for (key, reason) in remove_finalize {
            if let Some(item) = self.active.remove(&key) {
                self.log.log(&item);
                finalized.push((key, item, reason));
            }
        }
        finalized
    }

    /// See [`InFlightLedger::happened_within`].
    pub fn happened_within(&self, now: f32, window: f32, committed_only: bool) -> bool {
        self.recognitions_within(now, window, committed_only)
            .next()
            .is_some()
    }

    /// All recognitions within `window` seconds before `now`, across in-flight
    /// and finalized items.
    pub fn recognitions_within(
        &self,
        now: f32,
        window: f32,
        committed_only: bool,
    ) -> impl Iterator<Item = Recognition> + '_ {
        let active = self
            .active
            .values()
            .map(C::recognition)
            .filter(move |rec| in_window(rec, now, window, committed_only));
        active.chain(self.log.finalized_within(now, window, committed_only))
    }
}

#[cfg(test)]
#[path = "in_flight_tests.rs"]
mod tests;
