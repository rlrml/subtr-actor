use super::*;

#[test]
fn maps_stats_modules_to_their_providing_nodes() {
    // Most modules share their providing node's name; `core` and `flip_reset`
    // are the only second-view exceptions.
    assert_eq!(stats_module_analysis_node_name("core"), "match_stats");
    assert_eq!(
        stats_module_analysis_node_name("air_dribble"),
        "air_dribble"
    );
    assert_eq!(stats_module_analysis_node_name("boost"), "boost");
    assert_eq!(stats_module_analysis_node_name("ball_carry"), "ball_carry");
}

#[test]
fn air_dribble_and_ball_carry_modules_use_separate_nodes() {
    let selection = BuiltinModuleSelection::from_names(["air_dribble", "ball_carry"])
        .expect("air_dribble and ball_carry are builtin stats modules");
    let mut graph = selection.graph().expect("selection should build a graph");
    graph.resolve().expect("graph should resolve");

    let air_dribble_nodes = graph
        .node_names()
        .filter(|name| *name == "air_dribble")
        .count();
    let ball_carry_nodes = graph
        .node_names()
        .filter(|name| *name == "ball_carry")
        .count();
    assert_eq!(air_dribble_nodes, 1);
    assert_eq!(ball_carry_nodes, 1);
    assert!(graph.state::<AirDribbleCalculator>().is_some());
    assert!(graph.state::<BallCarryCalculator>().is_some());
}
