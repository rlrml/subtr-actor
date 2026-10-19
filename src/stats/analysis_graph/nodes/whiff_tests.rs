use super::*;
use crate::PlayerId;

fn whiff_event() -> WhiffEvent {
    WhiffEvent {
        kind: WhiffEventKind::Whiff,
        start_time: 1.0,
        start_frame: 10,
        time: 1.2,
        frame: 12,
        resolved_time: 1.5,
        resolved_frame: 15,
        resolution_reason: WhiffResolutionReason::SeparatedFromBall,
        player: boxcars::RemoteId::Steam(1),
        player_position: Some([100.0, 200.0, 17.0]),
        is_team_0: true,
        closest_approach_distance: 90.0,
        forward_alignment: 0.9,
        approach_speed: 1000.0,
        closing_speed_at_closest: Some(900.0),
        velocity_alignment_at_closest: Some(0.95),
        local_ball_position_at_closest: Some([120.0, 10.0, 75.0]),
        resolved_distance: Some(380.0),
        dodge_active: false,
        aerial: false,
        confidence: None,
        disrupted_by: None,
    }
}

#[test]
fn projection_spans_attempt_start_to_resolution_and_keeps_closest_anchor() {
    let mut assembler = EventAssembler::new();
    push_projected_whiff(&mut assembler, &whiff_event());
    let [projected] = assembler.events.as_slice() else {
        panic!("expected one projected whiff");
    };

    assert_eq!(projected.meta.id, "whiff:10:0");
    assert_eq!(
        projected.meta.timing,
        EventTiming::Span {
            start_frame: 10,
            end_frame: 15,
            start_time: 1.0,
            end_time: 1.5,
        }
    );
    let EventPayload::Whiff(event) = &projected.payload else {
        panic!("expected whiff payload");
    };
    assert_eq!(event.frame, 12);
    assert_eq!(event.time, 1.2);
}

fn parse_replay(path: &str) -> boxcars::Replay {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let data = std::fs::read(&path).expect("fixture replay");
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .expect("fixture replay parses")
}

fn player_matches(player: &PlayerId, id: &str) -> bool {
    serde_json::to_string(player).is_ok_and(|text| text.contains(id))
}

/// Interactions in this fixture were labeled by the vision-assisted review in
/// `scripts/whiff_model` (they are training examples, so this guards behavior,
/// not accuracy). The previous proximity detector missed the high aerial miss
/// and the renewed miss one second before another, and fired on the
/// successful contact while the player kept pushing the ball.
#[test]
fn deployed_detector_on_labeled_doubles_fixture() {
    let replay = parse_replay("assets/recent-ranked-doubles-2026-03-10.replay");
    let graph = crate::stats::analysis_graph::collect_analysis_graph_for_replay(
        &replay,
        crate::stats::analysis_graph::graph_with_builtin_analysis_nodes(["whiff"]).unwrap(),
    )
    .unwrap();
    let events = graph.state::<WhiffCalculator>().unwrap().events();
    let near = |id: &str, time: f32| {
        events
            .iter()
            .filter(|e| player_matches(&e.player, id) && (e.time - time).abs() < 0.4)
            .map(|e| e.kind)
            .collect::<Vec<_>>()
    };
    let clean_misses = [
        ("14d2c197a569431cbd227f56203b852e", 176.1),
        ("14d2c197a569431cbd227f56203b852e", 177.27),
        ("688865443571342319", 275.58),
        ("6e829ecc728345feb48e2f4179ea1695", 328.31),
    ];
    for (id, time) in clean_misses {
        assert_eq!(near(id, time), [WhiffEventKind::Whiff], "{id} at {time}");
    }
    assert_eq!(
        near("14d2c197a569431cbd227f56203b852e", 18.97),
        [WhiffEventKind::DisruptedAttempt]
    );
    let disrupted = events
        .iter()
        .find(|e| e.kind == WhiffEventKind::DisruptedAttempt && (e.time - 18.97).abs() < 0.4)
        .unwrap();
    assert!(disrupted.disrupted_by.is_some());
    assert!(near("688865443571342319", 268.43).is_empty());
    for event in events {
        let delay = event.resolved_time - event.time;
        assert!((0.6 - 1e-3..0.75).contains(&delay), "{delay}");
    }
}
