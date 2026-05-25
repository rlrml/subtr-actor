use super::*;

#[test]
fn event_timeline_graph_does_not_build_full_stats_frame_snapshots() {
    let mut graph = build_timeline_event_graph();
    graph
        .resolve()
        .expect("event timeline graph should resolve");
    let node_names = graph.node_names().collect::<Vec<_>>();

    assert!(node_names.contains(&"stats_timeline_events"));
    assert!(
        !node_names.contains(&"stats_timeline_frame"),
        "event timeline transfer should not evaluate the full partial-sum frame node"
    );
}
