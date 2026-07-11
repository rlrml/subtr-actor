//! Shared machinery for projecting calculator state into keyed timeline
//! events.
//!
//! Every analysis node that owns one or more event streams builds its
//! [`AnalysisNode::project_events`](crate::AnalysisNode::project_events)
//! result through an [`EventAssembler`], which mints the cadence-invariant
//! event ids and stamps the shared [`EventMeta`] fields. The helpers here are
//! deliberately node-agnostic: nodes depend on this module (and their own
//! calculator state), never on each other's projection code.

use std::collections::HashMap;

use crate::stats::timeline::types::stats_timeline_event_label;
use crate::{Event, EventLifecycle, EventMeta, EventPayload, EventTiming, PlayerId};

/// Builds one node's projection, minting cadence-invariant ids of the form
/// `{stream}:{birth_anchor_frame}:{disambiguator}`.
///
/// # Id determinism
///
/// A 1s-interval interim run and a finish-only run over the same frames must
/// mint identical ids for the same underlying events — that is what makes the
/// live output structurally equal to the batch output. Both id components are
/// pure functions of *calculator state*, never of projection timing:
///
/// - `birth_anchor_frame` is a per-stream field of the committed payload (the
///   moment frame, or a span's start frame as committed) that is immutable
///   from the instant the entry first appears in its calculator's accessor.
///   For streams whose presented timing can be revised (a touch retimed onto
///   its ball-movement span), the anchor deliberately reads the immutable
///   payload field, not `meta.timing`.
/// - `disambiguator` is the index among events sharing `(stream, anchor)`, in
///   the order the projection visits them, which every projection site keeps
///   equal to calculator commit order: calculators' event lists are
///   append-only (a later event's insertion can never shift an earlier one),
///   the `timeline` stream's time re-sort is stable and same-anchor entries
///   share a time so their commit order survives it, and the open-inclusive
///   player-span streams are visited via
///   `PlayerSpanTracker::projected_events_by_player`, whose (player,
///   per-player commit order) ordering is identical whether a span is
///   observed while still open or only after it closed.
///
/// Because the disambiguator only counts within one `(stream, anchor)` group,
/// per-node assemblers mint exactly the ids the former single bulk assembler
/// did: id assignment never crosses a stream boundary, and each stream is
/// projected by exactly one node.
pub(crate) struct EventAssembler {
    /// Next disambiguator per `(stream, birth_anchor_frame)`.
    next_disambiguator: HashMap<(&'static str, usize), usize>,
    pub(crate) events: Vec<Event>,
}

impl EventAssembler {
    pub(crate) fn new() -> Self {
        Self {
            next_disambiguator: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Appends one event. `lifecycle` is the projection site's rule for how
    /// settled the event is given current calculator state; finish-time
    /// projections are finalized wholesale by `AnalysisGraph::finish`, never
    /// here.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn push(
        &mut self,
        stream: &'static str,
        birth_anchor_frame: usize,
        lifecycle: EventLifecycle,
        timing: EventTiming,
        payload: EventPayload,
        primary_player: Option<PlayerId>,
        secondary_player: Option<PlayerId>,
        team_is_team_0: Option<bool>,
        player_position: Option<[f32; 3]>,
        ball_position: Option<[f32; 3]>,
        confidence: Option<f32>,
    ) {
        let disambiguator = self
            .next_disambiguator
            .entry((stream, birth_anchor_frame))
            .or_insert(0);
        let id = format!("{stream}:{birth_anchor_frame}:{disambiguator}");
        *disambiguator += 1;
        let scope = payload.scope();
        self.events.push(Event {
            meta: EventMeta {
                id,
                stream: stream.to_owned(),
                label: stats_timeline_event_label(stream),
                scope,
                lifecycle,
                timing,
                primary_player,
                secondary_player,
                player_position,
                ball_position,
                team_is_team_0,
                confidence,
                properties: Vec::new(),
            },
            payload,
        });
    }

    /// The finished projection, in visitation order. Chronological ordering is
    /// a *read-side* concern: the graph's transaction log keys by id and its
    /// reduced view ([`TimelineTransactionLog::current_events`]) sorts
    /// chronologically, so per-node projections do not need to.
    ///
    /// [`TimelineTransactionLog::current_events`]: crate::TimelineTransactionLog::current_events
    pub(crate) fn into_events(self) -> Vec<Event> {
        self.events
    }
}

/// Lifecycle rule for a `PlayerSpanTracker`-backed span: closed spans are
/// immutable, the open span's end still advances every frame.
pub(crate) fn span_lifecycle(closed: bool) -> EventLifecycle {
    if closed {
        EventLifecycle::Finalized
    } else {
        EventLifecycle::Confirmed
    }
}

pub(crate) fn moment(frame: usize, time: f32) -> EventTiming {
    EventTiming::Moment { frame, time }
}

pub(crate) fn span(
    start_frame: usize,
    end_frame: usize,
    start_time: f32,
    end_time: f32,
) -> EventTiming {
    EventTiming::Span {
        start_frame,
        end_frame,
        start_time,
        end_time,
    }
}

#[cfg(test)]
#[path = "projection_tests.rs"]
mod tests;
