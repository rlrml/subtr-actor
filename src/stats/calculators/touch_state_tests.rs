use super::*;

fn rigid_body(position: glam::Vec3, velocity: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn frame(frame_number: usize) -> FrameInfo {
    FrameInfo {
        frame_number,
        time: frame_number as f32 * 0.1,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn ball(velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z), velocity),
    })
}

fn players(player_id: PlayerId) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![PlayerSample {
            player_id,
            is_team_0: true,
            rigid_body: Some(rigid_body(
                glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z),
                glam::Vec3::ZERO,
            )),
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
        }],
    }
}

#[test]
fn emits_consecutive_same_player_touch_candidates_without_cooldown() {
    let player_id = boxcars::RemoteId::Steam(1);
    let players = players(player_id.clone());
    let mut calculator = TouchStateCalculator::new();
    let live_play = LivePlayState {
        gameplay_phase: GameplayPhase::ActivePlay,
        is_live_play: true,
    };

    calculator.update(
        &frame(0),
        &ball(glam::Vec3::ZERO),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );
    let first_touch = calculator.update(
        &frame(1),
        &ball(glam::Vec3::new(300.0, 0.0, 0.0)),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );
    let second_touch = calculator.update(
        &frame(2),
        &ball(glam::Vec3::new(650.0, 0.0, 0.0)),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );

    assert_eq!(first_touch.touch_events.len(), 1);
    assert_eq!(first_touch.touch_events[0].player, Some(player_id.clone()));
    assert_eq!(first_touch.touch_events[0].frame, 1);

    assert_eq!(second_touch.touch_events.len(), 1);
    assert_eq!(second_touch.touch_events[0].player, Some(player_id.clone()));
    assert_eq!(second_touch.touch_events[0].frame, 2);
    assert_eq!(second_touch.last_touch_player, Some(player_id));
}

#[test]
fn explicit_touch_events_feed_touch_state_without_velocity_candidate() {
    let player_id = boxcars::RemoteId::Steam(1);
    let players = players(player_id.clone());
    let mut calculator = TouchStateCalculator::new();
    let live_play = LivePlayState {
        gameplay_phase: GameplayPhase::ActivePlay,
        is_live_play: true,
    };
    let events = FrameEventsState {
        touch_events: vec![TouchEvent {
            time: 0.1,
            frame: 1,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            closest_approach_distance: None,
            dodge_contact: false,
        }],
        ..FrameEventsState::default()
    };

    let touch_state = calculator.update(
        &frame(1),
        &ball(glam::Vec3::ZERO),
        &players,
        &events,
        &live_play,
    );

    assert_eq!(touch_state.touch_events.len(), 1);
    assert_eq!(touch_state.touch_events[0].player, Some(player_id.clone()));
    assert_eq!(touch_state.touch_events[0].frame, 1);
    assert_eq!(touch_state.last_touch_player, Some(player_id));
}

#[test]
fn explicit_touch_events_are_enriched_with_proximity_distance() {
    let player_id = boxcars::RemoteId::Steam(1);
    let players = players(player_id.clone());
    let mut calculator = TouchStateCalculator::new();
    let live_play = LivePlayState {
        gameplay_phase: GameplayPhase::ActivePlay,
        is_live_play: true,
    };
    let events = FrameEventsState {
        touch_events: vec![TouchEvent {
            time: 0.1,
            frame: 1,
            team_is_team_0: true,
            player: Some(player_id),
            closest_approach_distance: None,
            dodge_contact: false,
        }],
        ..FrameEventsState::default()
    };

    let touch_state = calculator.update(
        &frame(1),
        &ball(glam::Vec3::ZERO),
        &players,
        &events,
        &live_play,
    );

    assert_eq!(
        touch_state.touch_events[0].closest_approach_distance,
        Some(0.0)
    );
}
