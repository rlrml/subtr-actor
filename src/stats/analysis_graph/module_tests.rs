use super::*;
use crate::{
    AerialGoalCalculator, AirDribbleGoalCalculator, BackboardCalculator, BallCarryCalculator,
    BumpCalculator, CenterCalculator, CounterAttackGoalCalculator, DoubleTapGoalCalculator,
    EmptyNetGoalCalculator, FlickCalculator, FlickGoalCalculator, FlipIntoBallGoalCalculator,
    FlipResetGoalCalculator, HalfVolleyCalculator, HalfVolleyGoalCalculator,
    HighAerialGoalCalculator, LongDistanceGoalCalculator, MatchStatsCalculator, OneTimerCalculator,
    OneTimerGoalCalculator, OwnHalfGoalCalculator, PassCalculator, PassingGoalCalculator,
    PlayerVerticalState, PossessionState, RotationCalculator, StatsTimelineCollector,
    SustainedPressureGoalCalculator, TerritorialPressureCalculator, TouchState,
    WallAerialCalculator, WallAerialShotCalculator, builtin_analysis_node_json,
    builtin_analysis_nodes_json, builtin_stats_module_names,
};
use std::any::TypeId;
use std::collections::HashSet;
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

fn canonical_builtin_analysis_node_name_set() -> HashSet<&'static str> {
    builtin_analysis_node_names()
        .iter()
        .filter_map(|name| canonical_builtin_analysis_node_name(name))
        .collect()
}

#[test]
fn all_analysis_nodes_matches_builtin_registry() {
    let node_names = all_analysis_nodes()
        .into_iter()
        .map(|node| node.name())
        .collect::<HashSet<_>>();

    assert_eq!(node_names, canonical_builtin_analysis_node_name_set());
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
    for module_name in builtin_stats_module_names() {
        let mut graph = graph_with_builtin_analysis_nodes([*module_name])
            .unwrap_or_else(|_| panic!("stats module should be graph-callable: {module_name}"));
        graph
            .resolve()
            .unwrap_or_else(|_| panic!("stats module graph should resolve: {module_name}"));
    }
}

#[test]
fn core_alias_and_match_stats_share_one_provider() {
    assert!(
        builtin_analysis_node_aliases()
            .iter()
            .any(|alias| alias.alias == "core" && alias.node_name == "match_stats")
    );

    let mut graph = graph_with_builtin_analysis_nodes(["core", "match_stats"])
        .expect("core alias and match_stats should be accepted together");
    graph
        .resolve()
        .expect("core alias and match_stats should not duplicate providers");

    let names = graph.node_names().collect::<Vec<_>>();
    assert_eq!(
        names.iter().filter(|name| **name == "match_stats").count(),
        1
    );
    assert!(!names.contains(&"core"));
}

#[test]
fn air_dribble_alias_and_ball_carry_share_one_provider() {
    assert!(builtin_analysis_node_names().contains(&"air_dribble"));
    assert!(
        builtin_analysis_node_aliases()
            .iter()
            .any(|alias| alias.alias == "air_dribble" && alias.node_name == "ball_carry")
    );

    let mut graph = graph_with_builtin_analysis_nodes(["air_dribble", "ball_carry"])
        .expect("air_dribble alias and ball_carry should be accepted together");
    graph
        .resolve()
        .expect("air_dribble alias and ball_carry should not duplicate providers");

    let names = graph.node_names().collect::<Vec<_>>();
    assert_eq!(
        names.iter().filter(|name| **name == "ball_carry").count(),
        1
    );
    assert!(!names.contains(&"air_dribble"));
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
    for alias in builtin_analysis_node_aliases() {
        let value = builtin_analysis_node_json(alias.alias, &graph).unwrap_or_else(|_| {
            panic!(
                "builtin analysis node alias should serialize: {} -> {}",
                alias.alias, alias.node_name
            )
        });
        assert!(
            !value.is_null(),
            "builtin analysis node alias should expose non-null JSON: {}",
            alias.alias
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
#[ignore = "full graph versus legacy timeline replay parity is slow; run explicitly when changing graph/timeline transfer"]
fn full_analysis_graph_matches_stats_timeline_events_on_real_replay() {
    let replay = parse_replay(ANALYSIS_GRAPH_REAL_REPLAY_FIXTURE);
    let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("full graph should evaluate a real replay");
    let graph_events = graph
        .state::<StatsTimelineEventsState>()
        .expect("full graph should expose stats timeline events")
        .events
        .clone();

    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("stats timeline collector should evaluate the same replay");

    assert_eq!(graph_events, timeline.events);
}
