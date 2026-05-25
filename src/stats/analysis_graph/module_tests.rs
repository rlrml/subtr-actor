use super::*;
use crate::{
    AerialGoalCalculator, AirDribbleGoalCalculator, BackboardCalculator, BumpCalculator,
    CenterCalculator, CounterAttackGoalCalculator, DoubleTapGoalCalculator, EmptyNetGoalCalculator,
    FlickCalculator, FlickGoalCalculator, FlipResetGoalCalculator, HalfVolleyCalculator,
    HalfVolleyGoalCalculator, HighAerialGoalCalculator, LongDistanceGoalCalculator,
    MatchStatsCalculator, OneTimerCalculator, OneTimerGoalCalculator, OwnHalfGoalCalculator,
    PassCalculator, PlayerVerticalState, PossessionState, RotationCalculator, TouchState,
    WallAerialCalculator, WallAerialShotCalculator,
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
    assert_eq!(names.len(), 55);
    assert!(names.contains("player_vertical_state"));
    assert!(names.contains("touch_state"));
    assert!(names.contains("possession_state"));
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
    assert!(names.contains("air_dribble_goal"));
    assert!(names.contains("flip_reset_goal"));
    assert!(names.contains("half_volley_goal"));
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
    assert!(graph.state::<AirDribbleGoalCalculator>().is_some());
    assert!(graph.state::<FlipResetGoalCalculator>().is_some());
    assert!(graph.state::<HalfVolleyGoalCalculator>().is_some());
}
