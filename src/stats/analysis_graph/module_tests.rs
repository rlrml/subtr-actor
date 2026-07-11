use super::*;
use crate::stats::calculators::{FinalizationHorizon, LivePlayState};
use crate::{
    AerialGoalCalculator, AirDribbleCalculator, AirDribbleGoalCalculator, BackboardCalculator,
    BallCarryCalculator, BumpCalculator, CenterCalculator, CounterAttackGoalCalculator,
    DoubleTapGoalCalculator, EmptyNetGoalCalculator, EventLifecycle, EventTransaction,
    FlickCalculator, FlickGoalCalculator, FlipIntoBallGoalCalculator, FlipResetGoalCalculator,
    HalfVolleyCalculator, HalfVolleyGoalCalculator, HighAerialGoalCalculator,
    LongDistanceGoalCalculator, MatchStatsCalculator, OneTimerCalculator, OneTimerGoalCalculator,
    OwnHalfGoalCalculator, PassCalculator, PassingGoalCalculator, PlayerVerticalState,
    PossessionState, ReplayStatsTimelineEvents, RotationCalculator, StatsTimelineCollector,
    SustainedPressureGoalCalculator, TerritorialPressureCalculator, TouchState,
    WallAerialCalculator, WallAerialShotCalculator, builtin_analysis_node_json,
    builtin_analysis_nodes_json, builtin_stats_module_names,
};
use crate::{ProcessorView, TimeAdvance};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const ANALYSIS_GRAPH_REAL_REPLAY_FIXTURE: &str = "assets/post-eac-ranked-duel-2026-04-28-a.replay";

fn parse_replay(path: &str) -> boxcars::Replay {
    let replay_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let data = std::fs::read(&replay_path)
        .unwrap_or_else(|_| panic!("Failed to read replay file: {}", replay_path.display()));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {}", replay_path.display()))
}

/// Every event a node emits must have a corresponding registered
/// [`EventDefinition`]. Sourced by walking the actual analysis nodes — the
/// nodes are the authoritative producers of their events, so this can't drift
/// the way a separate name-keyed producer registry could.
#[test]
fn every_emitted_event_has_a_registered_definition() {
    let registered: HashSet<&str> = crate::all_event_definitions()
        .iter()
        .map(|definition| definition.id)
        .collect();
    for node in all_analysis_nodes() {
        for emitted in node.emitted_events() {
            assert!(
                registered.contains(emitted.event.id),
                "event {:?} is emitted by node {:?} but is missing from the definition registry",
                emitted.event.id,
                node.name(),
            );
        }
    }
}

/// Stream ownership is statically auditable: every projected timeline stream
/// is declared (with its finalization horizon) by exactly one analysis node,
/// and the declaration's producer names that node. The graph separately
/// rejects, at projection time, any event on a stream its node did not
/// declare, so the declarations walked here describe exactly what can reach
/// the timeline.
#[test]
fn every_projected_stream_is_declared_by_exactly_one_node() {
    let mut owner_by_stream: HashMap<&'static str, &'static str> = HashMap::new();
    for node in all_analysis_nodes() {
        for emitted in node.emitted_events() {
            let Some(projected) = emitted.projected else {
                continue;
            };
            assert_eq!(
                emitted.producer.node_name,
                node.name(),
                "stream {:?} declaration must name its own node",
                projected.stream,
            );
            if let Some(previous_owner) = owner_by_stream.insert(projected.stream, node.name()) {
                panic!(
                    "stream {:?} is declared by both {previous_owner:?} and {:?}",
                    projected.stream,
                    node.name(),
                );
            }
        }
    }
    assert!(
        owner_by_stream.len() >= 40,
        "the stream catalog should be non-trivial, got {} streams",
        owner_by_stream.len(),
    );
}

fn builtin_analysis_node_name_set() -> HashSet<&'static str> {
    builtin_analysis_node_names().iter().copied().collect()
}

#[test]
fn all_analysis_nodes_matches_builtin_registry() {
    let node_names = all_analysis_nodes()
        .into_iter()
        .map(|node| node.name())
        .collect::<HashSet<_>>();

    assert_eq!(node_names, builtin_analysis_node_name_set());
    assert!(node_names.contains("kickoff"));
}

#[test]
fn resolves_all_analysis_nodes_with_default_signal_nodes() {
    let requested_node_names = all_analysis_nodes()
        .into_iter()
        .map(|node| node.name())
        .collect::<HashSet<_>>();

    let mut graph = graph_with_all_analysis_nodes();
    graph.resolve().expect("graph should resolve");

    let names: HashSet<_> = graph.node_names().collect();
    for name in requested_node_names {
        assert!(names.contains(name), "resolved graph should include {name}");
    }
    for name in [
        "player_vertical_state",
        "touch_state",
        "possession_state",
        "backboard_bounce_state",
        "fifty_fifty_state",
        "live_play",
        "kickoff",
        "controlled_play",
    ] {
        assert!(
            names.contains(name),
            "resolved graph should include default dependency node {name}"
        );
    }
}

#[test]
fn every_builtin_analysis_node_name_builds() {
    for name in builtin_analysis_node_names() {
        let mut graph = graph_with_builtin_analysis_nodes([*name])
            .unwrap_or_else(|_| panic!("builtin analysis node should build: {name}"));
        graph
            .resolve()
            .unwrap_or_else(|_| panic!("builtin analysis node should resolve: {name}"));
    }
}

#[test]
fn materialized_timeline_frame_state_is_terminal_export_only() {
    let materialized_frame_state = TypeId::of::<StatsTimelineFrameState>();

    for node in all_analysis_nodes() {
        let node_name = node.name();
        for dependency in node.dependencies() {
            assert_ne!(
                dependency.state_type_id(),
                materialized_frame_state,
                "{node_name} must depend on specific calculator states, not \
                 StatsTimelineFrameState"
            );
        }
    }
}

#[test]
fn every_builtin_stats_module_is_graph_callable() {
    // Drives the real module -> node mapping in the stats collector, which is the
    // only place stats-module names (e.g. `core`, `air_dribble`) are translated to
    // the analysis nodes that provide them.
    for module_name in builtin_stats_module_names() {
        crate::StatsCollector::try_only_modules([*module_name]).unwrap_or_else(|_| {
            panic!("stats module should build an analysis graph: {module_name}")
        });
    }
}

#[test]
fn duplicate_node_requests_share_one_provider() {
    let mut graph = graph_with_builtin_analysis_nodes(["ball_carry", "ball_carry"])
        .expect("duplicate node requests should be accepted");
    graph
        .resolve()
        .expect("duplicate node requests should not duplicate providers");

    let names = graph.node_names().collect::<Vec<_>>();
    assert_eq!(
        names.iter().filter(|name| **name == "ball_carry").count(),
        1
    );
    assert!(graph.state::<BallCarryCalculator>().is_some());
}

#[test]
fn air_dribble_and_ball_carry_resolve_as_separate_providers() {
    let mut graph = graph_with_builtin_analysis_nodes(["air_dribble", "ball_carry"])
        .expect("air dribble and ball carry requests should be accepted");
    graph
        .resolve()
        .expect("air dribble and ball carry should resolve");

    let names = graph.node_names().collect::<Vec<_>>();
    assert_eq!(
        names.iter().filter(|name| **name == "air_dribble").count(),
        1
    );
    assert_eq!(
        names.iter().filter(|name| **name == "ball_carry").count(),
        1
    );
    assert!(graph.state::<AirDribbleCalculator>().is_some());
    assert!(graph.state::<BallCarryCalculator>().is_some());
}

#[test]
fn every_resolved_shared_graph_node_name_is_directly_callable() {
    let mut graph = graph_with_all_analysis_nodes();
    graph.resolve().expect("shared graph should resolve");

    let builtin_names = builtin_analysis_node_names()
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    for name in graph.node_names() {
        assert!(
            builtin_names.contains(name),
            "resolved shared graph node should be callable by name: {name}"
        );
    }
}

#[test]
fn continuous_ball_control_is_directly_callable() {
    assert!(builtin_analysis_node_names().contains(&"continuous_ball_control"));
    let mut graph = graph_with_builtin_analysis_nodes(["continuous_ball_control"])
        .expect("continuous ball control should be a builtin analysis node");
    graph
        .resolve()
        .expect("continuous ball control node should resolve");

    let names: HashSet<_> = graph.node_names().collect();
    assert!(names.contains("continuous_ball_control"));
}

#[test]
#[ignore = "broad real-replay JSON smoke is slow; graph construction and focused replay regressions run in CI"]
fn every_builtin_analysis_node_has_shared_json_output_on_real_replay() {
    let replay = parse_replay(ANALYSIS_GRAPH_REAL_REPLAY_FIXTURE);
    let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("graph should evaluate a real replay");

    assert_all_reducer_states_are_present(&graph);

    for name in builtin_analysis_node_names() {
        let value = builtin_analysis_node_json(name, &graph)
            .unwrap_or_else(|_| panic!("builtin analysis node should serialize: {name}"));
        assert!(
            !value.is_null(),
            "builtin analysis node should expose non-null JSON: {name}"
        );
    }
    let all_nodes = builtin_analysis_nodes_json(&graph)
        .expect("all builtin analysis nodes should serialize together");
    let all_nodes = all_nodes
        .as_object()
        .expect("all builtin analysis node JSON should be an object");
    assert_eq!(all_nodes.len(), builtin_analysis_node_names().len());
    for name in builtin_analysis_node_names() {
        assert_eq!(
            all_nodes.get(*name),
            Some(
                &builtin_analysis_node_json(name, &graph)
                    .unwrap_or_else(|_| panic!("builtin analysis node should serialize: {name}"))
            ),
            "all-node analysis JSON should include node {name}"
        );
    }

    assert_eq!(
        builtin_analysis_node_json("core", &graph).expect("core should serialize"),
        builtin_analysis_node_json("match_stats", &graph).expect("match_stats should serialize")
    );
    assert!(
        builtin_analysis_node_json("not_a_node", &graph).is_err(),
        "unknown analysis nodes should be rejected"
    );
}

fn assert_all_reducer_states_are_present(graph: &AnalysisGraph) {
    assert!(graph.state::<PlayerVerticalState>().is_some());
    assert!(graph.state::<TouchState>().is_some());
    assert!(graph.state::<PossessionState>().is_some());
    assert!(graph.state::<BackboardCalculator>().is_some());
    assert!(graph.state::<BumpCalculator>().is_some());
    assert!(graph.state::<MatchStatsCalculator>().is_some());
    assert!(graph.state::<OneTimerCalculator>().is_some());
    assert!(graph.state::<CenterCalculator>().is_some());
    assert!(graph.state::<HalfVolleyCalculator>().is_some());
    assert!(graph.state::<WallAerialCalculator>().is_some());
    assert!(graph.state::<WallAerialShotCalculator>().is_some());
    assert!(graph.state::<PassCalculator>().is_some());
    assert!(graph.state::<RotationCalculator>().is_some());
    assert!(graph.state::<TerritorialPressureCalculator>().is_some());
    assert!(graph.state::<FlickCalculator>().is_some());
    assert!(graph.state::<AerialGoalCalculator>().is_some());
    assert!(graph.state::<HighAerialGoalCalculator>().is_some());
    assert!(graph.state::<LongDistanceGoalCalculator>().is_some());
    assert!(graph.state::<OwnHalfGoalCalculator>().is_some());
    assert!(graph.state::<EmptyNetGoalCalculator>().is_some());
    assert!(graph.state::<CounterAttackGoalCalculator>().is_some());
    assert!(graph.state::<SustainedPressureGoalCalculator>().is_some());
    assert!(graph.state::<FlickGoalCalculator>().is_some());
    assert!(graph.state::<DoubleTapGoalCalculator>().is_some());
    assert!(graph.state::<OneTimerGoalCalculator>().is_some());
    assert!(graph.state::<PassingGoalCalculator>().is_some());
    assert!(graph.state::<AirDribbleGoalCalculator>().is_some());
    assert!(graph.state::<FlipResetGoalCalculator>().is_some());
    assert!(graph.state::<FlipIntoBallGoalCalculator>().is_some());
    assert!(graph.state::<HalfVolleyGoalCalculator>().is_some());
}

#[test]
#[ignore = "covered by every_builtin_analysis_node_has_shared_json_output_on_real_replay; run explicitly when debugging graph state materialization"]
fn evaluates_all_reducer_nodes_against_a_real_replay() {
    let replay = parse_replay(ANALYSIS_GRAPH_REAL_REPLAY_FIXTURE);
    let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("graph should evaluate a real replay");

    assert_all_reducer_states_are_present(&graph);
}

#[test]
#[ignore = "full-graph real-replay parity is slow; run explicitly when changing interim event projection"]
fn interim_event_projection_matches_finish_only_on_real_replay() {
    let replay = parse_replay(ANALYSIS_GRAPH_REAL_REPLAY_FIXTURE);
    let finish_only = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("finish-only graph should evaluate a real replay");
    // Interim run: same node set, but with a ~1s driver-owned projection
    // cadence layered on by the collector. Any lifecycle-invariant violation
    // during an interim projection (a finalized event changing or any event
    // vanishing) propagates out of the graph as an error in debug builds, so
    // this `expect` doubles as the zero-violations assertion for the whole
    // interim run.
    let interim = collector::AnalysisNodeCollector::new(graph_with_all_analysis_nodes())
        .with_projection_interval(1.0)
        .process_replay(&replay)
        .expect(
            "interim-projection graph should evaluate a real replay without invariant violations",
        )
        .into_graph();

    // Ids are cadence-invariant, so the final event sets — ids included —
    // must be identical whether events were projected throughout the match or
    // only at finish. The reduced view is the graph store's current events
    // (finish's single projection is the only batch surface).
    let finish_only_events: Vec<crate::Event> = finish_only
        .event_transaction_log()
        .current_events()
        .into_iter()
        .cloned()
        .collect();
    let interim_events: Vec<crate::Event> = interim
        .event_transaction_log()
        .current_events()
        .into_iter()
        .cloned()
        .collect();
    assert_eq!(
        finish_only_events, interim_events,
        "interim projections must not change the final event set"
    );

    // After finish, every event in the reduced view is finalized.
    assert!(
        interim_events
            .iter()
            .all(|event| event.meta.lifecycle == EventLifecycle::Finalized),
        "every event must be finalized after finish"
    );

    // Replaying the transaction log: seq strictly increasing, no retracts,
    // and per-id lifecycle only ever moves Confirmed -> Confirmed/Finalized
    // (never away from Finalized; the store also enforces this, so this is a
    // belt-and-braces readback of what a live consumer would have observed).
    let mut lifecycle_by_id: std::collections::HashMap<&str, EventLifecycle> =
        std::collections::HashMap::new();
    let mut last_seq = None;
    for transaction in interim.event_transaction_log().transactions() {
        assert!(
            last_seq.is_none_or(|last| transaction.seq() > last),
            "transaction seq must be strictly increasing"
        );
        last_seq = Some(transaction.seq());
        match transaction {
            EventTransaction::Retract { id, .. } => {
                panic!("no event should be retracted during a normal match, got {id}")
            }
            EventTransaction::Upsert { event, .. } => {
                let previous = lifecycle_by_id.insert(&event.meta.id, event.meta.lifecycle);
                assert_ne!(
                    previous,
                    Some(EventLifecycle::Finalized),
                    "revision of already-finalized event {}",
                    event.meta.id
                );
            }
        }
    }
    assert!(
        lifecycle_by_id
            .values()
            .all(|lifecycle| *lifecycle == EventLifecycle::Finalized),
        "every id observed mid-run must end finalized"
    );
    assert_eq!(
        lifecycle_by_id.len(),
        interim_events.len(),
        "the transaction log must cover exactly the final event set"
    );
}

/// Observation harness for the finalization-horizon test: wraps the
/// interim-projection collector, samples the live-play signal each frame, and
/// records the game time at which each event id was first observed
/// `Finalized` in the graph's transaction log.
struct HorizonProbe {
    inner: collector::AnalysisNodeCollector,
    log_cursor: usize,
    first_finalized_at: HashMap<String, f32>,
    /// Live-play transitions as chronological `(time, is_live_play)` samples;
    /// each entry's state holds until the next entry's time.
    live_play_transitions: Vec<(f32, bool)>,
    last_frame_time: f32,
}

impl HorizonProbe {
    fn new(inner: collector::AnalysisNodeCollector) -> Self {
        Self {
            inner,
            log_cursor: 0,
            first_finalized_at: HashMap::new(),
            live_play_transitions: Vec::new(),
            last_frame_time: 0.0,
        }
    }

    fn drain_new_transactions(&mut self, observed_at: f32) {
        let log = self.inner.graph().event_transaction_log();
        for transaction in log.transactions_since(self.log_cursor) {
            if let EventTransaction::Upsert { event, .. } = transaction
                && event.meta.lifecycle == EventLifecycle::Finalized
            {
                self.first_finalized_at
                    .entry(event.meta.id.clone())
                    .or_insert(observed_at);
            }
        }
        self.log_cursor = log.transaction_count();
    }
}

impl Collector for HorizonProbe {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        let advance = self
            .inner
            .process_frame(processor, frame, frame_number, current_time)?;
        self.last_frame_time = current_time;
        let is_live = self
            .inner
            .graph()
            .state::<LivePlayState>()
            .is_some_and(|state| state.is_live_play);
        if self.live_play_transitions.last().map(|&(_, live)| live) != Some(is_live) {
            self.live_play_transitions.push((current_time, is_live));
        }
        self.drain_new_transactions(current_time);
        Ok(advance)
    }

    fn finish_replay(&mut self, processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        self.inner.finish_replay(processor)?;
        // Whatever the finish projection finalizes is only guaranteed
        // observable at match end.
        self.drain_new_transactions(self.last_frame_time);
        Ok(())
    }
}

/// The `NextStoppage` deadline for an event ending at `end_time`: the moment
/// live play resumes after the first stoppage at or after the event's end (if
/// the event ends during a stoppage, that stoppage counts), or match end if
/// play never resumes.
fn next_stoppage_resumption(transitions: &[(f32, bool)], end_time: f32, match_end: f32) -> f32 {
    if transitions.is_empty() {
        return match_end;
    }
    let mut index = transitions
        .partition_point(|&(time, _)| time <= end_time)
        .saturating_sub(1);
    if transitions[index].1 {
        // Live at the event's end: advance to the first stoppage after it.
        match transitions[index + 1..].iter().position(|&(_, live)| !live) {
            Some(offset) => index += 1 + offset,
            None => return match_end,
        }
    }
    transitions[index + 1..]
        .iter()
        .find(|&&(_, live)| live)
        .map(|&(time, _)| time)
        .unwrap_or(match_end)
}

/// Empirical enforcement of the declared per-stream finalization horizons
/// (see [`FinalizationHorizon`]): drives the full graph over a real replay at
/// a 1s projection cadence, records when each event first became `Finalized`,
/// and fails if any event finalized later than its stream's declared horizon.
///
/// Slack: one projection interval — an event can settle immediately after a
/// projection ran and only be observed at the next one — plus a small epsilon
/// for frame-time granularity. The per-stream max observed lag is printed so
/// drift toward a horizon stays visible before it becomes a failure.
#[test]
#[ignore = "real-replay horizon instrumentation is slow; run explicitly when changing stream horizons or projection lifecycles"]
fn declared_finalization_horizons_hold_on_real_replay() {
    const PROJECTION_INTERVAL_SECONDS: f32 = 1.0;
    const SLACK_SECONDS: f32 = PROJECTION_INTERVAL_SECONDS + 0.1;

    let horizon_by_stream: HashMap<&'static str, FinalizationHorizon> = all_analysis_nodes()
        .iter()
        .flat_map(|node| node.emitted_events())
        .filter_map(|emitted| emitted.projected)
        .map(|projected| (projected.stream, projected.horizon))
        .collect();

    let replay = parse_replay(ANALYSIS_GRAPH_REAL_REPLAY_FIXTURE);
    let probe = HorizonProbe::new(
        collector::AnalysisNodeCollector::new(graph_with_all_analysis_nodes())
            .with_projection_interval(PROJECTION_INTERVAL_SECONDS),
    )
    .process_replay(&replay)
    .expect("horizon probe should evaluate a real replay");

    let match_end = probe.last_frame_time;
    let transitions = &probe.live_play_transitions;

    let mut max_lag_by_stream: std::collections::BTreeMap<String, f32> =
        std::collections::BTreeMap::new();
    let mut violations: Vec<String> = Vec::new();
    for event in probe.inner.graph().event_transaction_log().current_events() {
        let stream = event.meta.stream.as_str();
        let horizon = *horizon_by_stream
            .get(stream)
            .unwrap_or_else(|| panic!("stream {stream:?} has no declared horizon"));
        let (_, end_time) = event.meta.timing.end();
        let finalized_at = *probe
            .first_finalized_at
            .get(&event.meta.id)
            .unwrap_or_else(|| panic!("event {:?} was never observed finalized", event.meta.id));
        let lag = finalized_at - end_time;
        let stream_max = max_lag_by_stream
            .entry(stream.to_owned())
            .or_insert(f32::NEG_INFINITY);
        *stream_max = stream_max.max(lag);

        let deadline = match horizon {
            FinalizationHorizon::EndPlus(seconds) => Some(end_time + seconds),
            FinalizationHorizon::NextStoppage => {
                Some(next_stoppage_resumption(transitions, end_time, match_end))
            }
            FinalizationHorizon::MatchEnd => None,
        };
        if let Some(deadline) = deadline
            && finalized_at > deadline + SLACK_SECONDS
        {
            violations.push(format!(
                "{id} ({stream}, {horizon:?}): end={end_time:.2}s deadline={deadline:.2}s \
                 finalized={finalized_at:.2}s (lag {lag:.2}s)",
                id = event.meta.id,
            ));
        }
    }

    println!("per-stream max observed finalization lag (finalized - end, seconds):");
    for (stream, lag) in &max_lag_by_stream {
        let horizon = horizon_by_stream[stream.as_str()];
        println!("  {stream:<22} {lag:>8.2}  (declared {horizon:?})");
    }

    assert!(
        violations.is_empty(),
        "finalization horizon violations:\n{}",
        violations.join("\n")
    );
}

#[test]
#[ignore = "full graph versus legacy timeline replay parity is slow; run explicitly when changing graph/timeline transfer"]
fn full_analysis_graph_matches_stats_timeline_events_on_real_replay() {
    let replay = parse_replay(ANALYSIS_GRAPH_REAL_REPLAY_FIXTURE);
    let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("full graph should evaluate a real replay");
    let graph_events = ReplayStatsTimelineEvents {
        events: graph
            .event_transaction_log()
            .current_events()
            .into_iter()
            .cloned()
            .collect(),
    };

    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("stats timeline collector should evaluate the same replay");

    assert_eq!(graph_events, timeline.events);
}
