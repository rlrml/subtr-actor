use super::*;
use crate::{
    BackboardCalculator, MatchStatsCalculator, PlayerVerticalState, PossessionState, TouchState,
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
    assert_eq!(names.len(), 30);
    assert!(names.contains("player_vertical_state"));
    assert!(names.contains("touch_state"));
    assert!(names.contains("possession_state"));
    assert!(names.contains("backboard_bounce_state"));
    assert!(names.contains("fifty_fifty_state"));
    assert!(names.contains("match_stats"));
    assert!(names.contains("live_play"));
    assert!(names.contains("touch"));
}

#[test]
fn evaluates_all_reducer_nodes_against_a_real_replay() {
    let replay = parse_replay("assets/rlcs.replay");
    let graph = collect_analysis_graph_for_replay(&replay, graph_with_all_analysis_nodes())
        .expect("graph should evaluate a real replay");

    assert!(graph.state::<PlayerVerticalState>().is_some());
    assert!(graph.state::<TouchState>().is_some());
    assert!(graph.state::<PossessionState>().is_some());
    assert!(graph.state::<BackboardCalculator>().is_some());
    assert!(graph.state::<MatchStatsCalculator>().is_some());
}
