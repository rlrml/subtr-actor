use super::*;

fn frame(frame_number: usize) -> FrameInfo {
    FrameInfo {
        frame_number,
        time: frame_number as f32 * 0.1,
        dt: 0.1,
        seconds_remaining: None,
    }
}

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

fn ball(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
    })
}

fn touch(player: PlayerId, is_team_0: bool, gap: f32) -> TouchEvent {
    TouchEvent {
        time: 0.1,
        frame: 1,
        team_is_team_0: is_team_0,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(gap),
        dodge_contact: false,
    }
}

#[test]
fn backboard_bounce_uses_primary_touch_not_last_contested_candidate() {
    let primary_player = boxcars::RemoteId::Steam(1);
    let secondary_player = boxcars::RemoteId::Steam(2);
    let primary_touch = touch(primary_player.clone(), true, 0.0);
    let secondary_touch = touch(secondary_player, false, 3.0);
    let touch_state = TouchState {
        touch_events: vec![primary_touch.clone(), secondary_touch],
        last_touch: Some(primary_touch),
        last_touch_player: Some(primary_player.clone()),
        last_touch_team_is_team_0: Some(true),
    };
    let mut calculator = BackboardBounceCalculator::new();

    calculator.update(
        &frame(1),
        &ball(
            glam::Vec3::new(0.0, 4700.0, 650.0),
            glam::Vec3::new(0.0, 500.0, 0.0),
        ),
        &touch_state,
        &LivePlayState::active_play(),
    );
    let state = calculator.update(
        &frame(2),
        &ball(
            glam::Vec3::new(0.0, 4800.0, 650.0),
            glam::Vec3::new(0.0, -300.0, 0.0),
        ),
        &TouchState::default(),
        &LivePlayState::active_play(),
    );

    let [bounce] = state.bounce_events.as_slice() else {
        panic!("expected exactly one backboard bounce");
    };
    assert_eq!(bounce.player, primary_player);
    assert!(bounce.is_team_0);
}
