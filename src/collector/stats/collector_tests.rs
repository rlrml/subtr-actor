use super::*;

#[test]
fn maps_stats_modules_to_their_providing_nodes() {
    // Most modules share their providing node's name; `core` and `air_dribble`
    // are the only second-view exceptions.
    assert_eq!(stats_module_analysis_node_name("core"), "match_stats");
    assert_eq!(stats_module_analysis_node_name("air_dribble"), "ball_carry");
    assert_eq!(stats_module_analysis_node_name("boost"), "boost");
    assert_eq!(stats_module_analysis_node_name("ball_carry"), "ball_carry");
}

#[test]
fn air_dribble_and_ball_carry_modules_share_one_node() {
    let selection = BuiltinModuleSelection::from_names(["air_dribble", "ball_carry"])
        .expect("air_dribble and ball_carry are builtin stats modules");
    let mut graph = selection.graph().expect("selection should build a graph");
    graph.resolve().expect("graph should resolve");

    let ball_carry_nodes = graph
        .node_names()
        .filter(|name| *name == "ball_carry")
        .count();
    assert_eq!(ball_carry_nodes, 1);
    assert!(graph.state::<BallCarryCalculator>().is_some());
}
