//! Drives the full analysis graph from consumed live-export messages.
//!
//! [`LiveGraphDriver`] owns the same node set the BakkesMod live path runs
//! ([`graph_with_all_analysis_nodes`]) plus a [`TimelineTransactionCursor`],
//! and mirrors that FFI's frame loop: sync roster meta into the graph when its
//! signature changes, build a timeline [`FrameInput`] over the store's
//! [`LiveProcessorView`](subtr_actor_live::LiveProcessorView), evaluate, and
//! drain the graph's newly-appended [`EventTransaction`]s.
//!
//! Projection cadence is owned by this driver: after evaluating a frame it
//! calls [`AnalysisGraph::project_events_now`] at most once per
//! [`DEFAULT_EVENT_PROJECTION_INTERVAL_SECONDS`] of game time (configurable
//! via [`LiveGraphDriver::with_event_projection_interval`]). Each projection
//! appends an `Upsert` the first time an event is observable — marked
//! `Confirmed` while later evidence may still revise it — and again on each
//! revision and on the `Finalized` transition; `MatchEnd` triggers
//! [`AnalysisGraph::finish`], whose final projection finalizes every event.
//! Consumers key on `meta.id` and read the lifecycle from the carried event.

use subtr_actor::stats::analysis_graph::{AnalysisGraph, graph_with_all_analysis_nodes};
use subtr_actor::{EventTransaction, FrameInput, PlayerId, SubtrActorError};
use subtr_actor_live::{LiveMatchMeta, ServerMessage, TimelineTransactionCursor};

use crate::store::{Applied, LiveStateStore};

/// Failure evaluating the analysis graph over a consumed message.
#[derive(Debug)]
pub enum DriverError {
    Graph(SubtrActorError),
}

impl std::fmt::Display for DriverError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverError::Graph(error) => write!(formatter, "analysis graph: {error:?}"),
        }
    }
}

impl std::error::Error for DriverError {}

impl From<SubtrActorError> for DriverError {
    fn from(error: SubtrActorError) -> Self {
        Self::Graph(error)
    }
}

/// Outputs emitted by [`LiveGraphDriver::on_message`] via the sink.
#[derive(Debug, Clone)]
pub enum DriverOutput {
    /// A match roster was first observed.
    MatchStarted { meta: LiveMatchMeta },
    /// The roster changed mid-match.
    RosterChanged { meta: LiveMatchMeta },
    /// Newly-appended stats-timeline transactions, in log order (never
    /// empty, `seq` strictly increasing). An `Upsert` inserts or replaces by
    /// `meta.id` (the carried event's `meta.lifecycle` says whether it may
    /// still be revised); a `Retract` removes the id.
    EventTransactions(Vec<EventTransaction>),
    /// The match ended; the graph was finished, drained, and reset.
    MatchEnded,
    /// The server restarted; graph and store state were reset.
    ServerRestarted,
}

type RosterSignature = Vec<(PlayerId, bool, Option<String>)>;

/// Default game-time interval between event projections. Each projection
/// re-scans all committed calculator events (see
/// `AnalysisGraph::project_events_now`), so ~1s keeps the amortized cost
/// negligible while events still stream out promptly.
pub const DEFAULT_EVENT_PROJECTION_INTERVAL_SECONDS: f32 = 1.0;

/// Owns the analysis graph and incremental transaction cursor for one
/// consumer.
pub struct LiveGraphDriver {
    graph: AnalysisGraph,
    cursor: TimelineTransactionCursor,
    projection_interval: f32,
    /// Game time of the last event projection, for the cadence throttle.
    last_projection_time: Option<f32>,
    meta_initialized: bool,
    meta_signature: Option<RosterSignature>,
}

impl Default for LiveGraphDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveGraphDriver {
    pub fn new() -> Self {
        Self::with_event_projection_interval(DEFAULT_EVENT_PROJECTION_INTERVAL_SECONDS)
    }

    /// A driver that projects the graph's events at most once per
    /// `interval_seconds` of game time.
    pub fn with_event_projection_interval(interval_seconds: f32) -> Self {
        Self {
            graph: graph_with_all_analysis_nodes(),
            cursor: TimelineTransactionCursor::new(),
            projection_interval: interval_seconds,
            last_projection_time: None,
            meta_initialized: false,
            meta_signature: None,
        }
    }

    /// Applies one server message to `store` and reacts to it: roster syncs,
    /// per-frame graph evaluation, and match-lifecycle finish/reset.
    pub fn on_message(
        &mut self,
        store: &mut LiveStateStore,
        message: ServerMessage,
        sink: &mut dyn FnMut(DriverOutput),
    ) -> Result<(), DriverError> {
        match store.apply(message) {
            Applied::Info | Applied::Heartbeat => {}
            Applied::MatchStart => {
                self.sync_meta_from_store(store)?;
                if let Some(meta) = store.meta() {
                    sink(DriverOutput::MatchStarted { meta: meta.clone() });
                }
            }
            Applied::RosterChange => {
                self.sync_meta_from_store(store)?;
                if let Some(meta) = store.meta() {
                    sink(DriverOutput::RosterChanged { meta: meta.clone() });
                }
            }
            Applied::Snapshot => {
                // A snapshot only positions the store (history + latest
                // frame); graph evaluation waits for the next `Frame`.
                self.sync_meta_from_store(store)?;
            }
            Applied::Frame => {
                self.evaluate_latest_frame(store)?;
                self.project_if_due(store)?;
                self.emit_new_events(sink);
            }
            Applied::MatchEnd => {
                // Mirror the FFI finish gate: a graph that never saw a frame
                // (meta never synced) has nothing to finish.
                if self.meta_initialized {
                    // `finish` runs the final, finalize-everything projection
                    // into the graph's transaction log.
                    self.graph.finish()?;
                    self.emit_new_events(sink);
                }
                self.reset();
                sink(DriverOutput::MatchEnded);
            }
            Applied::ServerRestart => {
                self.reset();
                sink(DriverOutput::ServerRestarted);
            }
        }
        Ok(())
    }

    fn evaluate_latest_frame(&mut self, store: &LiveStateStore) -> Result<(), DriverError> {
        let Some(latest) = store.latest() else {
            return Ok(());
        };
        // Mirror the BakkesMod FFI's `sync_live_replay_meta`: the roster is
        // derived from the frame's own players every frame, signature-guarded,
        // so meta reaches the graph even if a roster message was missed.
        let frame_meta = LiveMatchMeta::from_player_frames(&latest.frame.players);
        self.sync_meta(&frame_meta)?;

        let frame_number = latest.frame.frame_number as usize;
        let time = latest.frame.time;
        let dt = latest.frame.dt;
        let live_play = latest.live_play.clone();
        let input = {
            let view = store
                .view()
                .expect("store has a latest frame, so it has a view");
            FrameInput::timeline_with_live_play_state(&view, frame_number, time, dt, live_play)
        };
        self.graph.evaluate_with_state(&input)?;
        Ok(())
    }

    /// Runs an event projection if the driver-owned cadence says one is due:
    /// on the first evaluated frame, once `projection_interval` of game time
    /// has elapsed, or when game time goes backwards (a reset the store
    /// applied without a `MatchEnd`), which projects immediately and
    /// re-anchors the throttle rather than waiting out a stale interval.
    fn project_if_due(&mut self, store: &LiveStateStore) -> Result<(), DriverError> {
        let Some(latest) = store.latest() else {
            return Ok(());
        };
        let now = latest.frame.time;
        let due = self
            .last_projection_time
            .is_none_or(|last| now - last >= self.projection_interval || now < last);
        if due {
            self.graph.project_events_now()?;
            self.last_projection_time = Some(now);
        }
        Ok(())
    }

    fn sync_meta_from_store(&mut self, store: &LiveStateStore) -> Result<(), DriverError> {
        if let Some(meta) = store.meta() {
            self.sync_meta(meta)?;
        }
        Ok(())
    }

    /// Pushes roster meta into the graph when its signature changed,
    /// mirroring the BakkesMod FFI's `sync_live_replay_meta`.
    fn sync_meta(&mut self, meta: &LiveMatchMeta) -> Result<(), DriverError> {
        let signature = meta.signature();
        if self.meta_initialized && self.meta_signature.as_ref() == Some(&signature) {
            return Ok(());
        }
        self.graph.on_replay_meta(&meta.replay_meta())?;
        self.meta_initialized = true;
        self.meta_signature = Some(signature);
        Ok(())
    }

    fn emit_new_events(&mut self, sink: &mut dyn FnMut(DriverOutput)) {
        let drained = self.cursor.drain(self.graph.event_transaction_log());
        if !drained.is_empty() {
            sink(DriverOutput::EventTransactions(drained.to_vec()));
        }
    }

    /// Rebuilds the graph and rewinds the transaction cursor, for the next
    /// match (or after a server restart).
    fn reset(&mut self) {
        self.graph = graph_with_all_analysis_nodes();
        self.cursor.reset();
        self.last_projection_time = None;
        self.meta_initialized = false;
        self.meta_signature = None;
    }
}
