use super::*;

fn rigid_body(position: glam::Vec3, velocity: Option<glam::Vec3>) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: velocity.map(|velocity| glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn touch_event(frame: usize, time: f32) -> TouchEvent {
    TouchEvent {
        time,
        frame,
        team_is_team_0: true,
        player: None,
        player_position: None,
        closest_approach_distance: None,
        dodge_contact: false,
    }
}

#[test]
fn touch_event_timestamp_ordering_uses_frame_then_time() {
    let earlier = touch_event(10, 1.0);
    let later_frame = touch_event(11, 0.9);
    let later_time_same_frame = touch_event(10, 1.1);

    assert!(TouchEvent::timestamp_ordering(&earlier, &later_frame).is_lt());
    assert!(TouchEvent::timestamp_ordering(&earlier, &later_time_same_frame).is_lt());
    assert!(TouchEvent::timestamp_ordering(&later_frame, &earlier).is_gt());
}

#[test]
fn shot_event_metadata_calculates_speed_distance_and_goal_alignment() {
    let ball = rigid_body(
        glam::Vec3::new(300.0, 1000.0, 120.0),
        Some(glam::Vec3::new(0.0, 2000.0, 0.0)),
    );
    let player = rigid_body(
        glam::Vec3::new(300.0, 850.0, 20.0),
        Some(glam::Vec3::new(0.0, 1200.0, 0.0)),
    );

    let metadata = ShotEventMetadata::from_rigid_bodies(true, &ball, Some(&player));

    assert_eq!(metadata.shot_touch_position, ball.location);
    assert_eq!(metadata.ball_position, ball.location);
    assert_eq!(metadata.ball_velocity, ball.linear_velocity);
    assert_eq!(metadata.ball_speed, Some(2000.0));
    assert_eq!(metadata.player_position, Some(player.location));
    assert_eq!(metadata.player_speed, Some(1200.0));
    assert!((metadata.player_distance_to_ball.unwrap() - 180.27756).abs() < 0.001);
    assert_eq!(metadata.target_goal_position.y, 5120.0);
    assert_eq!(metadata.target_goal_position.z, ball.location.z);
    assert_eq!(metadata.distance_to_goal_line, 4120.0);
    assert!(metadata.distance_to_goal_center > 4130.0);
    assert!(metadata.ball_goal_alignment.unwrap() > 0.99);
    assert!(metadata.ball_speed_toward_goal.unwrap() > 1990.0);
}

#[test]
fn shot_event_metadata_uses_orange_goal_direction() {
    let ball = rigid_body(
        glam::Vec3::new(0.0, -1000.0, 120.0),
        Some(glam::Vec3::new(0.0, -1500.0, 0.0)),
    );

    let metadata = ShotEventMetadata::from_rigid_bodies(false, &ball, None);

    assert_eq!(metadata.target_goal_position.y, -5120.0);
    assert_eq!(metadata.distance_to_goal_line, 4120.0);
    assert_eq!(metadata.player_position, None);
    assert!(metadata.ball_goal_alignment.unwrap() > 0.99);
    assert!(metadata.ball_speed_toward_goal.unwrap() > 1490.0);
}

fn date_header(value: &str) -> Vec<(String, boxcars::HeaderProp)> {
    vec![(
        "Date".to_string(),
        boxcars::HeaderProp::Str(value.to_string()),
    )]
}

#[test]
fn parse_header_date_handles_replay_and_rfc3339_formats() {
    assert_eq!(
        parse_header_date("2026-04-28 14-30-00"),
        Some((2026, 4, 28))
    );
    assert_eq!(
        parse_header_date("2026-04-17T15:01:25-07:00"),
        Some((2026, 4, 17))
    );
    assert_eq!(parse_header_date("not-a-date"), None);
    assert_eq!(parse_header_date(""), None);
}

#[test]
fn season_for_date_returns_most_recent_started_season() {
    let f21 = ReplaySeason::new(SeasonEra::FreeToPlay, 21);
    // Exactly on a boundary resolves to that season.
    assert_eq!(season_for_date((2026, 1, 14)), Some(f21));
    // Between boundaries resolves to the earlier one.
    assert_eq!(season_for_date((2026, 4, 17)), Some(f21));
    assert_eq!(
        season_for_date((2024, 12, 4)),
        Some(ReplaySeason::new(SeasonEra::FreeToPlay, 17))
    );
    // Legacy era resolves before the free-to-play reset.
    assert_eq!(
        season_for_date((2019, 9, 1)),
        Some(ReplaySeason::new(SeasonEra::Legacy, 9))
    );
    // Dates before the first known season are unclassified.
    assert_eq!(season_for_date((2015, 1, 1)), None);
}

#[test]
fn season_code_round_trips_era_and_number() {
    assert_eq!(ReplaySeason::new(SeasonEra::FreeToPlay, 21).code(), "f21");
    assert_eq!(ReplaySeason::new(SeasonEra::Legacy, 14).code(), "s14");
}

#[test]
fn season_from_headers_resolves_from_date() {
    assert_eq!(
        season_from_headers(&date_header("2026-04-17T15:01:25-07:00")),
        Some(ReplaySeason::new(SeasonEra::FreeToPlay, 21))
    );
    assert_eq!(season_from_headers(&[]), None);
}
