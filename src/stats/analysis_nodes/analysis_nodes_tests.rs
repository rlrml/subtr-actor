use super::*;
use crate::stats::reducers::*;
use std::collections::HashSet;

fn empty_sample() -> CoreSample {
    CoreSample {
        frame_number: 0,
        time: 0.0,
        dt: 0.0,
        seconds_remaining: None,
        game_state: None,
        ball_has_been_hit: None,
        kickoff_countdown_time: None,
        team_zero_score: None,
        team_one_score: None,
        possession_team_is_team_0: None,
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: None,
        ball: None,
        players: Vec::new(),
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: Vec::new(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

#[test]
fn resolves_all_reducer_nodes_with_default_signal_nodes() {
    let mut graph = graph_with_all_analysis_nodes();
    graph.resolve().expect("graph should resolve");

    let names: HashSet<_> = graph.node_names().collect();
    assert_eq!(names.len(), 23);
    assert!(names.contains("touch_state"));
    assert!(names.contains("possession_state"));
    assert!(names.contains("backboard_bounce_state"));
    assert!(names.contains("fifty_fifty_state"));
    assert!(names.contains("match_stats"));
    assert!(names.contains("touch"));
}

#[test]
fn evaluates_all_reducer_nodes_against_an_empty_sample() {
    let mut graph = graph_with_all_analysis_nodes();
    graph.set_root_state(empty_sample());
    graph
        .evaluate()
        .expect("graph should evaluate an empty sample");

    assert!(graph.state::<TouchState>().is_some());
    assert!(graph.state::<PossessionState>().is_some());
    assert!(graph.state::<BackboardCalculator>().is_some());
    assert!(graph.state::<MatchStatsCalculator>().is_some());
}
