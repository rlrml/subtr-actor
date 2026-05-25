use super::*;
use crate::{
    builtin_analysis_node_json, builtin_stats_module_names, AerialGoalCalculator,
    AirDribbleGoalCalculator, BackboardCalculator, BallCarryCalculator, BumpCalculator,
    CenterCalculator, CounterAttackGoalCalculator, DoubleTapGoalCalculator, EmptyNetGoalCalculator,
    FlickCalculator, FlickGoalCalculator, FlipResetGoalCalculator, HalfVolleyCalculator,
    HalfVolleyGoalCalculator, HighAerialGoalCalculator, LongDistanceGoalCalculator,
    MatchStatsCalculator, OneTimerCalculator, OneTimerGoalCalculator, OwnHalfGoalCalculator,
    PassCalculator, PassingGoalCalculator, PlayerVerticalState, PossessionState, RotationCalculator,
    StatsTimelineCollector, TouchState, WallAerialCalculator, WallAerialShotCalculator,
};
use std::collections::HashSet;
use std::path::Path;

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

#[test]
fn resolves_all_reducer_nodes_with_default_signal_nodes() {
    let mut graph = graph_with_all_analysis_nodes();
    graph.resolve().expect("graph should resolve");

    let names: HashSet<_> = graph.node_names().collect();
    assert_eq!(names.len(), 58);
    assert!(names.contains("player_vertical_state"));
    assert!(names.contains("touch_state"));
    assert!(names.contains("possession_state"));
    assert!(names.contains("continuous_ball_control"));
    assert!(names.contains("backboard_bounce_state"));
    assert!(names.contains("fifty_fifty_state"));
    assert!(names.contains("match_stats"));
    assert!(names.contains("live_play"));
    assert!(names.contains("touch"));
    assert!(names.contains("bump"));
    assert!(names.contains("whiff"));
    assert!(names.contains("wavedash"));
    assert!(names.contains("half_flip"));
    assert!(names.contains("half_volley"));
    assert!(names.contains("wall_aerial"));
    assert!(names.contains("wall_aerial_shot"));
    assert!(names.contains("one_timer"));
    assert!(names.contains("center"));
    assert!(names.contains("pass"));
    assert!(names.contains("rotation"));
    assert!(names.contains("flick"));
    assert!(names.contains("aerial_goal"));
    assert!(names.contains("high_aerial_goal"));
    assert!(names.contains("long_distance_goal"));
    assert!(names.contains("own_half_goal"));
    assert!(names.contains("empty_net_goal"));
    assert!(names.contains("counter_attack_goal"));
    assert!(names.contains("flick_goal"));
    assert!(names.contains("double_tap_goal"));
    assert!(names.contains("one_timer_goal"));
    assert!(names.contains("passing_goal"));
    assert!(names.contains("air_dribble_goal"));
    assert!(names.contains("flip_reset_goal"));
    assert!(names.contains("half_volley_goal"));
    assert!(names.contains("stats_timeline_frame"));
    assert!(names.contains("stats_timeline_events"));
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
    assert!(builtin_analysis_node_aliases()
        .iter()
        .any(|alias| alias.alias == "core" && alias.node_name == "match_stats"));

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
    assert!(builtin_analysis_node_aliases()
        .iter()
        .any(|alias| alias.alias == "air_dribble" && alias.node_name == "ball_carry"));

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
fn every_builtin_analysis_node_has_shared_json_output_on_real_replay() {
    let replay = parse_replay("assets/rlcs.replay");
    let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("graph should evaluate a real replay");

    for name in builtin_analysis_node_names() {
        let value = builtin_analysis_node_json(name, &graph)
            .unwrap_or_else(|_| panic!("builtin analysis node should serialize: {name}"));
        assert!(
            !value.is_null(),
            "builtin analysis node should expose non-null JSON: {name}"
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

#[test]
fn evaluates_all_reducer_nodes_against_a_real_replay() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("graph should evaluate a real replay");

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
    assert!(graph.state::<FlickCalculator>().is_some());
    assert!(graph.state::<AerialGoalCalculator>().is_some());
    assert!(graph.state::<HighAerialGoalCalculator>().is_some());
    assert!(graph.state::<LongDistanceGoalCalculator>().is_some());
    assert!(graph.state::<OwnHalfGoalCalculator>().is_some());
    assert!(graph.state::<EmptyNetGoalCalculator>().is_some());
    assert!(graph.state::<CounterAttackGoalCalculator>().is_some());
    assert!(graph.state::<FlickGoalCalculator>().is_some());
    assert!(graph.state::<DoubleTapGoalCalculator>().is_some());
    assert!(graph.state::<OneTimerGoalCalculator>().is_some());
    assert!(graph.state::<PassingGoalCalculator>().is_some());
    assert!(graph.state::<AirDribbleGoalCalculator>().is_some());
    assert!(graph.state::<FlipResetGoalCalculator>().is_some());
    assert!(graph.state::<HalfVolleyGoalCalculator>().is_some());
}

#[test]
fn full_analysis_graph_matches_stats_timeline_events_on_real_replay() {
    let replay = parse_replay("assets/rlcs.replay");
    let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("full graph should evaluate a real replay");
    let graph_events = graph
        .state::<StatsTimelineEventsState>()
        .expect("full graph should expose stats timeline events")
        .events
        .clone();

    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("stats timeline collector should evaluate the same replay");

    assert_eq!(graph_events, timeline.events);
}
