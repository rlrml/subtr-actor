use super::*;
use crate::stats::reducers::*;
use std::collections::HashSet;

fn empty_frame_info() -> FrameInfo {
    FrameInfo::default()
}

fn empty_gameplay_state() -> GameplayState {
    GameplayState::default()
}

fn empty_ball_frame_state() -> BallFrameState {
    BallFrameState::default()
}

fn empty_player_frame_state() -> PlayerFrameState {
    PlayerFrameState::default()
}

fn empty_frame_events_state() -> FrameEventsState {
    FrameEventsState::default()
}

#[test]
fn resolves_all_reducer_nodes_with_default_signal_nodes() {
    let mut graph = graph_with_all_analysis_nodes();
    graph.resolve().expect("graph should resolve");

    let names: HashSet<_> = graph.node_names().collect();
    assert_eq!(names.len(), 25);
    assert!(names.contains("touch_state"));
    assert!(names.contains("possession_state"));
    assert!(names.contains("backboard_bounce_state"));
    assert!(names.contains("fifty_fifty_state"));
    assert!(names.contains("match_stats"));
    assert!(names.contains("live_play"));
    assert!(names.contains("touch"));
}

#[test]
fn evaluates_all_reducer_nodes_against_an_empty_sample() {
    let mut graph = graph_with_all_analysis_nodes();
    graph.set_root_state(empty_frame_info());
    graph.set_root_state(empty_gameplay_state());
    graph.set_root_state(empty_ball_frame_state());
    graph.set_root_state(empty_player_frame_state());
    graph.set_root_state(empty_frame_events_state());
    graph
        .evaluate()
        .expect("graph should evaluate an empty sample");

    assert!(graph.state::<TouchState>().is_some());
    assert!(graph.state::<PossessionState>().is_some());
    assert!(graph.state::<BackboardCalculator>().is_some());
    assert!(graph.state::<MatchStatsCalculator>().is_some());
}
