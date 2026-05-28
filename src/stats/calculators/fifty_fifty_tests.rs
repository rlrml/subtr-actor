use super::*;

fn rigid_body(position: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn player(player_id: PlayerId, is_team_0: bool, position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        rigid_body: Some(rigid_body(position)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn touch(player_id: PlayerId, is_team_0: bool, dodge_contact: bool) -> TouchEvent {
    TouchEvent {
        time: 1.0,
        frame: 10,
        team_is_team_0: is_team_0,
        player: Some(player_id),
        closest_approach_distance: Some(0.0),
        dodge_contact,
    }
}

#[test]
fn contested_touch_links_initial_touch_events_and_dodge_contact_tags() {
    let blue_player = boxcars::RemoteId::Steam(1);
    let orange_player = boxcars::RemoteId::Steam(2);
    let active = FiftyFiftyCalculator::contested_touch(
        &FrameInfo {
            frame_number: 10,
            time: 1.0,
            dt: 0.1,
            seconds_remaining: None,
        },
        &PlayerFrameState {
            players: vec![
                player(blue_player.clone(), true, glam::Vec3::new(0.0, -100.0, 0.0)),
                player(
                    orange_player.clone(),
                    false,
                    glam::Vec3::new(0.0, 100.0, 0.0),
                ),
            ],
        },
        &[
            touch(blue_player.clone(), true, true),
            touch(orange_player.clone(), false, false),
        ],
        false,
    )
    .unwrap();

    assert_eq!(active.team_zero_touch_time, Some(1.0));
    assert_eq!(active.team_zero_touch_frame, Some(10));
    assert!(active.team_zero_dodge_contact);
    assert_eq!(active.team_one_touch_time, Some(1.0));
    assert_eq!(active.team_one_touch_frame, Some(10));
    assert!(!active.team_one_dodge_contact);
}

#[test]
fn fifty_fifty_player_labels_inherit_own_touch_dodge_state() {
    let blue_player = boxcars::RemoteId::Steam(1);
    let orange_player = boxcars::RemoteId::Steam(2);
    let event = FiftyFiftyEvent {
        start_time: 1.0,
        start_frame: 10,
        resolve_time: 1.5,
        resolve_frame: 15,
        is_kickoff: false,
        team_zero_player: Some(blue_player.clone()),
        team_one_player: Some(orange_player.clone()),
        team_zero_touch_time: Some(1.0),
        team_zero_touch_frame: Some(10),
        team_zero_dodge_contact: true,
        team_one_touch_time: Some(1.0),
        team_one_touch_frame: Some(10),
        team_one_dodge_contact: false,
        team_zero_position: [0.0, -100.0, 0.0],
        team_one_position: [0.0, 100.0, 0.0],
        midpoint: [0.0, 0.0, 0.0],
        plane_normal: [0.0, 1.0, 0.0],
        winning_team_is_team_0: Some(true),
        possession_team_is_team_0: Some(true),
    };
    let mut calculator = FiftyFiftyCalculator::new();

    calculator
        .update(&FiftyFiftyState {
            resolved_events: vec![event],
            ..FiftyFiftyState::default()
        })
        .unwrap();

    assert_eq!(
        calculator
            .player_stats()
            .get(&blue_player)
            .unwrap()
            .event_count_with_labels(&[StatLabel::new("dodge_state", "dodge")]),
        1
    );
    assert_eq!(
        calculator
            .player_stats()
            .get(&orange_player)
            .unwrap()
            .event_count_with_labels(&[StatLabel::new("dodge_state", "no_dodge")]),
        1
    );
    assert_eq!(
        calculator
            .stats()
            .event_count_with_labels(&[StatLabel::new("team_zero_dodge_state", "dodge")]),
        1
    );
}
