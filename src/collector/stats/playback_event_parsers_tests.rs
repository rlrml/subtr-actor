use super::*;

#[test]
fn legacy_beaten_to_ball_whiff_payload_remains_readable() {
    let event = parse_whiff_event(&serde_json::json!({
        "kind": "beaten_to_ball",
        "time": 12.5,
        "frame": 750,
        "resolved_time": 12.7,
        "resolved_frame": 762,
        "player": { "Steam": 76561198000000001_u64 },
        "is_team_0": true,
        "closest_approach_distance": 140.0,
        "forward_alignment": 0.8,
        "approach_speed": 1100.0,
        "dodge_active": false,
        "aerial": false
    }))
    .expect("legacy whiff payload should parse");

    assert_eq!(event.kind, WhiffEventKind::BeatenToBall);
    assert_eq!(event.start_time, event.time);
    assert_eq!(event.start_frame, event.frame);
    assert_eq!(
        event.resolution_reason,
        WhiffResolutionReason::LegacyUnknown
    );
    assert_eq!(event.resolved_time, 12.7);
    assert_eq!(event.resolved_frame, 762);
    assert_eq!(event.closing_speed_at_closest, None);
    assert_eq!(event.velocity_alignment_at_closest, None);
    assert_eq!(event.local_ball_position_at_closest, None);
    assert_eq!(event.resolved_distance, None);
}

#[test]
fn enriched_whiff_payload_preserves_attempt_and_evidence_fields() {
    let event = parse_whiff_event(&serde_json::json!({
        "kind": "whiff",
        "start_time": 12.1,
        "start_frame": 726,
        "time": 12.5,
        "frame": 750,
        "resolved_time": 12.7,
        "resolved_frame": 762,
        "resolution_reason": "separated_from_ball",
        "player": { "Steam": 76561198000000001_u64 },
        "player_position": [100.0, 200.0, 17.0],
        "is_team_0": true,
        "closest_approach_distance": 140.0,
        "forward_alignment": 0.8,
        "approach_speed": 1100.0,
        "closing_speed_at_closest": 950.0,
        "velocity_alignment_at_closest": 0.9,
        "local_ball_position_at_closest": [150.0, 20.0, 75.0],
        "resolved_distance": 380.0,
        "dodge_active": false,
        "aerial": false
    }))
    .expect("enriched whiff payload should parse");

    assert_eq!(event.start_time, 12.1);
    assert_eq!(event.start_frame, 726);
    assert_eq!(
        event.resolution_reason,
        WhiffResolutionReason::SeparatedFromBall
    );
    assert_eq!(event.closing_speed_at_closest, Some(950.0));
    assert_eq!(event.velocity_alignment_at_closest, Some(0.9));
    assert_eq!(
        event.local_ball_position_at_closest,
        Some([150.0, 20.0, 75.0])
    );
    assert_eq!(event.resolved_distance, Some(380.0));
}
