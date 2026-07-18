use super::*;
use boxcars::RemoteId;

fn rigid_body(z: f32) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: boxcars::Vector3f { x: 0.0, y: 0.0, z },
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: None,
        angular_velocity: None,
    }
}

fn player(player_id: PlayerId, z: f32) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(z)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        dodge_torque: None,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn availability(
    node: &mut ThreatFeaturesNode,
    player_id: &PlayerId,
    time: f32,
    z: f32,
    control: PlayerControlSample,
    refreshed: bool,
) -> bool {
    let players = PlayerFrameState {
        players: vec![player(player_id.clone(), z)],
    };
    let controls = PlayerControlState {
        players: HashMap::from([(player_id.clone(), control)]),
    };
    let events = FrameEventsState {
        dodge_refreshed_events: refreshed
            .then(|| DodgeRefreshedEvent {
                time,
                frame: 0,
                player: player_id.clone(),
                player_position: None,
                is_team_0: true,
                counter_value: 1,
            })
            .into_iter()
            .collect(),
        ..FrameEventsState::default()
    };
    node.update_dodge_availability(
        &FrameInfo {
            time,
            ..FrameInfo::default()
        },
        &players,
        &controls,
        &events,
    )[player_id]
}

#[test]
fn dodge_is_available_on_ground_then_expires_after_takeoff() {
    let player_id = RemoteId::SplitScreen(1);
    let mut node = ThreatFeaturesNode::new();

    assert!(availability(
        &mut node,
        &player_id,
        0.0,
        PLAYER_GROUND_Z_THRESHOLD,
        PlayerControlSample::default(),
        false,
    ));
    assert!(availability(
        &mut node,
        &player_id,
        0.1,
        PLAYER_GROUND_Z_THRESHOLD + 100.0,
        PlayerControlSample::default(),
        false,
    ));
    assert!(!availability(
        &mut node,
        &player_id,
        1.36,
        PLAYER_GROUND_Z_THRESHOLD + 100.0,
        PlayerControlSample::default(),
        false,
    ));
}

#[test]
fn dodge_or_double_jump_consumes_availability() {
    for control in [
        PlayerControlSample {
            dodge_active: true,
            ..PlayerControlSample::default()
        },
        PlayerControlSample {
            double_jump_active: true,
            ..PlayerControlSample::default()
        },
    ] {
        let player_id = RemoteId::SplitScreen(2);
        let mut node = ThreatFeaturesNode::new();
        assert!(availability(
            &mut node,
            &player_id,
            0.0,
            PLAYER_GROUND_Z_THRESHOLD,
            PlayerControlSample::default(),
            false,
        ));
        assert!(!availability(
            &mut node,
            &player_id,
            0.1,
            PLAYER_GROUND_Z_THRESHOLD + 100.0,
            control,
            false,
        ));
    }
}

#[test]
fn flip_reset_refreshes_and_removes_the_standard_timeout() {
    let player_id = RemoteId::SplitScreen(3);
    let mut node = ThreatFeaturesNode::new();
    assert!(availability(
        &mut node,
        &player_id,
        0.0,
        PLAYER_GROUND_Z_THRESHOLD,
        PlayerControlSample::default(),
        false,
    ));
    assert!(availability(
        &mut node,
        &player_id,
        2.0,
        PLAYER_GROUND_Z_THRESHOLD + 100.0,
        PlayerControlSample::default(),
        true,
    ));
    assert!(availability(
        &mut node,
        &player_id,
        10.0,
        PLAYER_GROUND_Z_THRESHOLD + 100.0,
        PlayerControlSample::default(),
        false,
    ));
    assert!(!availability(
        &mut node,
        &player_id,
        10.1,
        PLAYER_GROUND_Z_THRESHOLD + 100.0,
        PlayerControlSample {
            dodge_active: true,
            ..PlayerControlSample::default()
        },
        false,
    ));
}
