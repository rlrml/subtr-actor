use super::*;

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
