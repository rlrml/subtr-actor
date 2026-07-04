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

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.2,
        seconds_remaining: None,
    }
}

fn ball(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
    })
}

fn player_at(position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, glam::Vec3::ZERO)),
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

fn players(position: glam::Vec3) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player_at(position)],
    }
}

fn wall_rigid_body(position: glam::Vec3, up: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        rotation: glam_to_quat(&glam::Quat::from_rotation_arc(
            glam::Vec3::Z,
            up.normalize(),
        )),
        ..rigid_body(position, glam::Vec3::ZERO)
    }
}

/// A player riding the wall at `position`: roof pointed at the field along the
/// inward wall normal, as wheels-on-wall contact implies.
fn player_on_wall_at(position: glam::Vec3) -> PlayerSample {
    let (outward, _) = wall_outward_normal_and_distance(position);
    PlayerSample {
        rigid_body: Some(wall_rigid_body(position, -outward)),
        ..player_at(position)
    }
}

fn players_on_wall(position: glam::Vec3) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player_on_wall_at(position)],
    }
}

fn controlled_touch_state() -> TouchState {
    TouchState {
        last_touch_player: Some(boxcars::RemoteId::Steam(1)),
        last_touch_team_is_team_0: Some(true),
        ..TouchState::default()
    }
}

fn touch_state_with_touch(frame: usize, time: f32) -> TouchState {
    let player = boxcars::RemoteId::Steam(1);
    TouchState {
        touch_events: vec![TouchEvent {
            touch_id: None,
            time,
            frame,
            team_is_team_0: true,
            player: Some(player.clone()),
            player_position: None,
            closest_approach_distance: Some(0.0),
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
            dodge_contact: false,
        }],
        last_touch: Some(TouchEvent {
            touch_id: None,
            time,
            frame,
            team_is_team_0: true,
            player: Some(player.clone()),
            player_position: None,
            closest_approach_distance: Some(0.0),
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
            dodge_contact: false,
        }),
        last_touch_player: Some(player),
        last_touch_team_is_team_0: Some(true),
    }
}

#[test]
fn records_controlled_wall_aerial_play_after_wall_carry_setup() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3520.0, 0.0, 330.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 250.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3500.0, 0.0, 340.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 270.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 0.6),
            &ball(
                glam::Vec3::new(3400.0, 0.0, 390.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3350.0, 0.0, 320.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 0.8),
            &ball(
                glam::Vec3::new(3300.0, 0.0, 430.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(3250.0, 0.0, 350.0)),
            &touch_state_with_touch(4, 0.8),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let event = calculator.events().first().expect("wall aerial event");
    assert_eq!(event.player, player.clone());
    assert_eq!(event.wall, WallAerialWall::Left);
    assert!(event.setup_duration >= WALL_AERIAL_MIN_WALL_CONTACT_DURATION);

    let stats = calculator.player_stats().get(&player).unwrap();
    assert_eq!(stats.count, 1);
}

#[test]
fn records_soft_controlled_wall_aerial_continuation_after_wall_setup() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3520.0, 0.0, 330.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 250.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3500.0, 0.0, 340.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 270.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 0.6),
            &ball(
                glam::Vec3::new(3400.0, 0.0, 390.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3350.0, 0.0, 320.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 0.8),
            &ball(
                glam::Vec3::new(3300.0, 0.0, 430.0),
                glam::Vec3::new(0.0, 0.0, -130.0),
            ),
            &players(glam::Vec3::new(3250.0, 0.0, 350.0)),
            &touch_state_with_touch(4, 0.8),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let event = calculator
        .events()
        .first()
        .expect("soft wall aerial continuation event");
    assert_eq!(event.player, player.clone());
    assert_eq!(event.wall, WallAerialWall::Left);
    assert_eq!(event.ball_speed_change, 0.0);

    let stats = calculator.player_stats().get(&player).unwrap();
    assert_eq!(stats.count, 1);
}

#[test]
fn rejects_low_wall_setup_touch_that_never_becomes_aerial_continuation() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3520.0, 0.0, 250.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 220.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3500.0, 0.0, 260.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 240.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 0.6),
            &ball(
                glam::Vec3::new(3400.0, 0.0, 270.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3350.0, 0.0, 260.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 0.8),
            &ball(
                glam::Vec3::new(3300.0, 0.0, 290.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(3250.0, 0.0, 280.0)),
            &touch_state_with_touch(4, 0.8),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn consumes_wall_setup_after_first_aerial_attempt() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3520.0, 0.0, 330.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 250.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3500.0, 0.0, 340.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 270.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 0.6),
            &ball(
                glam::Vec3::new(3400.0, 0.0, 390.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3350.0, 0.0, 320.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 0.8),
            &ball(
                glam::Vec3::new(3300.0, 0.0, 430.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(3250.0, 0.0, 350.0)),
            &touch_state_with_touch(4, 0.8),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(5, 2.6),
            &ball(
                glam::Vec3::new(3200.0, 0.0, 620.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(3150.0, 0.0, 520.0)),
            &TouchState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(6, 2.8),
            &ball(
                glam::Vec3::new(3100.0, 0.0, 700.0),
                glam::Vec3::new(0.0, 1000.0, 80.0),
            ),
            &players(glam::Vec3::new(3050.0, 0.0, 580.0)),
            &touch_state_with_touch(6, 2.8),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(calculator.events().len(), 1);
}

#[test]
fn preserves_completed_wall_setup_while_sliding_off_wall() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3980.0, 0.0, 420.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 300.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3920.0, 0.0, 560.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 480.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 0.6),
            &ball(
                glam::Vec3::new(3500.0, 0.0, 850.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3850.0, 0.0, 760.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 0.8),
            &ball(
                glam::Vec3::new(3300.0, 0.0, 900.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3500.0, 0.0, 860.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(5, 1.2),
            &ball(
                glam::Vec3::new(3000.0, 0.0, 900.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(2800.0, 0.0, 760.0)),
            &touch_state_with_touch(5, 1.2),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let event = calculator
        .events()
        .first()
        .expect("wall aerial event after wall slide");
    assert_eq!(event.player, player.clone());
    assert!(event.setup_duration >= WALL_AERIAL_MIN_WALL_CONTACT_DURATION);
}

#[test]
fn records_wall_aerial_when_first_continuation_touch_is_delayed() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3980.0, 0.0, 420.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 300.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3920.0, 0.0, 560.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 480.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 0.6),
            &ball(
                glam::Vec3::new(3800.0, 0.0, 720.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3500.0, 0.0, 720.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 2.4),
            &ball(
                glam::Vec3::new(2600.0, 0.0, 900.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(2500.0, 0.0, 780.0)),
            &touch_state_with_touch(4, 2.4),
            &LivePlayState::active_play(),
        )
        .unwrap();

    let event = calculator
        .events()
        .first()
        .expect("delayed wall aerial continuation event");
    assert_eq!(event.player, player.clone());
    assert!(event.time_since_takeoff > 1.6);
}

#[test]
fn rejects_stale_wall_contact_that_arms_much_later() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3980.0, 0.0, 420.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 300.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3920.0, 0.0, 560.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players_on_wall(glam::Vec3::new(4080.0, 0.0, 480.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 2.0),
            &ball(
                glam::Vec3::new(3000.0, 0.0, 900.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(2800.0, 0.0, 760.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 2.2),
            &ball(
                glam::Vec3::new(2900.0, 0.0, 920.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(2700.0, 0.0, 760.0)),
            &touch_state_with_touch(4, 2.2),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_wall_aerial_play_without_wall_control_setup() {
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3500.0, 0.0, 330.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3650.0, 0.0, 260.0)),
            &TouchState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball(
                glam::Vec3::new(3350.0, 0.0, 420.0),
                glam::Vec3::new(0.0, 700.0, 0.0),
            ),
            &players(glam::Vec3::new(3250.0, 0.0, 350.0)),
            &touch_state_with_touch(2, 0.2),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_aerial_that_starts_near_wall_but_never_on_it() {
    // A normal aerial that launches from the floor *near* the side wall: the car
    // climbs past the wall without ever touching its surface, then aerials up
    // and away to the ball. It must not be detected as a wall aerial.
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    calculator
        .update(
            &frame(1, 0.0),
            &ball(
                glam::Vec3::new(3380.0, 0.0, 330.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3490.0, 0.0, 250.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.4),
            &ball(
                glam::Vec3::new(3360.0, 0.0, 340.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3470.0, 0.0, 270.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 0.6),
            &ball(
                glam::Vec3::new(3300.0, 0.0, 430.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
            ),
            &players(glam::Vec3::new(3350.0, 0.0, 320.0)),
            &controlled_touch_state(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 0.8),
            &ball(
                glam::Vec3::new(3200.0, 0.0, 480.0),
                glam::Vec3::new(0.0, 700.0, 80.0),
            ),
            &players(glam::Vec3::new(3250.0, 0.0, 350.0)),
            &touch_state_with_touch(4, 0.8),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn rejects_ground_jump_aerial_that_climbs_through_the_near_wall_band() {
    // Regression for a real replay: the player rode the left wall, came back to
    // the ground, then jumped into an aerial from the floor near the wall. The
    // climb spends >0.3s several hundred uu from the wall surface with the roof
    // pointing straight up — it must not read as a wall ride, so leaving the
    // area must not arm a wall-aerial takeoff.
    let player = boxcars::RemoteId::Steam(1);
    let mut calculator = WallAerialCalculator::new();

    for (index, (x, z)) in [(3665.0, 130.0), (3655.0, 200.0), (3640.0, 270.0)]
        .iter()
        .enumerate()
    {
        calculator
            .update(
                &frame(index + 1, index as f32 * 0.2),
                &ball(
                    glam::Vec3::new(3400.0, -600.0, 600.0),
                    glam::Vec3::new(0.0, 0.0, 0.0),
                ),
                &players(glam::Vec3::new(*x, 0.0, *z)),
                &controlled_touch_state(),
                &LivePlayState::active_play(),
            )
            .unwrap();
    }
    calculator
        .update(
            &frame(4, 0.6),
            &ball(
                glam::Vec3::new(3450.0, -500.0, 700.0),
                glam::Vec3::new(0.0, -700.0, 80.0),
            ),
            &players(glam::Vec3::new(3520.0, -400.0, 560.0)),
            &touch_state_with_touch(4, 0.6),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player).is_none());
    assert!(calculator.events().is_empty());
}

// Riding the wall requires actual surface contact: proximity to the wall plane
// (or corner arc) plus the car roof leaning into the field.

#[test]
fn surface_contact_requires_wall_proximity() {
    let inward_up = glam::Vec3::new(-1.0, 0.0, 0.0);
    // Pinned to the side wall (car pivot ~17uu off the surface).
    assert_eq!(
        wall_aerial_surface_contact(&wall_rigid_body(
            glam::Vec3::new(4079.0, 0.0, 600.0),
            inward_up,
        )),
        Some(WallSurface::Side),
    );
    // Airborne inside the old near-wall band: hundreds of uu from the surface.
    assert_eq!(
        wall_aerial_surface_contact(&wall_rigid_body(
            glam::Vec3::new(3650.0, 0.0, 600.0),
            inward_up,
        )),
        None,
    );
}

#[test]
fn surface_contact_requires_roof_toward_field() {
    // At the wall but flying past it with the roof up: not riding.
    assert_eq!(
        wall_aerial_surface_contact(&wall_rigid_body(
            glam::Vec3::new(4079.0, 0.0, 600.0),
            glam::Vec3::Z,
        )),
        None,
    );
    // Roof pointing out of the field is just as wrong.
    assert_eq!(
        wall_aerial_surface_contact(&wall_rigid_body(
            glam::Vec3::new(4079.0, 0.0, 600.0),
            glam::Vec3::X,
        )),
        None,
    );
}

#[test]
fn surface_contact_follows_each_walls_own_normal() {
    // End wall: the inward normal is -y.
    assert_eq!(
        wall_aerial_surface_contact(&wall_rigid_body(
            glam::Vec3::new(2000.0, 5103.0, 600.0),
            glam::Vec3::new(0.0, -1.0, 0.0),
        )),
        Some(WallSurface::Back),
    );
    // A side-wall orientation on the end wall does not count.
    assert_eq!(
        wall_aerial_surface_contact(&wall_rigid_body(
            glam::Vec3::new(2000.0, 5103.0, 600.0),
            glam::Vec3::new(-1.0, 0.0, 0.0),
        )),
        None,
    );
    // Mid corner arc: the inward normal is the 45° diagonal toward the field
    // (the coarse surface tie-breaks to Side).
    assert_eq!(
        wall_aerial_surface_contact(&wall_rigid_body(
            glam::Vec3::new(3746.6, 4770.6, 600.0),
            glam::Vec3::new(-1.0, -1.0, 0.0),
        )),
        Some(WallSurface::Side),
    );
    // The goal mouth is an opening, not a wall surface.
    assert_eq!(
        wall_aerial_surface_contact(&wall_rigid_body(
            glam::Vec3::new(0.0, 5103.0, 600.0),
            glam::Vec3::new(0.0, -1.0, 0.0),
        )),
        None,
    );
}

// Which wall a position is on follows from the position alone: the residual of
// each axis beyond the corner-arc start is the outward wall normal.

#[test]
fn classifies_side_walls_relative_to_attack_direction() {
    let plus_x_wall = glam::Vec3::new(4096.0, 0.0, 600.0);
    let minus_x_wall = glam::Vec3::new(-4096.0, 0.0, 600.0);
    // Team 0 attacks toward +y, so the +x wall is their left.
    assert_eq!(
        wall_aerial_wall_classification(true, plus_x_wall),
        WallAerialWall::Left,
    );
    assert_eq!(
        wall_aerial_wall_classification(true, minus_x_wall),
        WallAerialWall::Right,
    );
    // Team 1 attacks toward -y, so the same +x wall is on their right.
    assert_eq!(
        wall_aerial_wall_classification(false, plus_x_wall),
        WallAerialWall::Right,
    );
}

#[test]
fn classifies_end_walls_as_front_or_back() {
    let plus_y_wall = glam::Vec3::new(1500.0, 5120.0, 600.0);
    let minus_y_wall = glam::Vec3::new(1500.0, -5120.0, 600.0);
    // For team 0 the +y end wall is the opponent's (front) end.
    assert_eq!(
        wall_aerial_wall_classification(true, plus_y_wall),
        WallAerialWall::Front,
    );
    assert_eq!(
        wall_aerial_wall_classification(true, minus_y_wall),
        WallAerialWall::Back,
    );
    // Team 1's attack direction is flipped, so the same walls swap roles.
    assert_eq!(
        wall_aerial_wall_classification(false, plus_y_wall),
        WallAerialWall::Back,
    );
}

#[test]
fn classifies_corner_arc_positions_as_corners() {
    // The midpoint of the +x/+y corner arc: the radial (outward-normal)
    // direction is a 45° diagonal.
    let mid_corner = glam::Vec3::new(3758.6, 4782.6, 600.0);
    assert_eq!(
        wall_aerial_wall_classification(true, mid_corner),
        WallAerialWall::FrontLeft,
    );
    assert_eq!(
        wall_aerial_wall_classification(false, mid_corner),
        WallAerialWall::BackRight,
    );
}

#[test]
fn side_wall_positions_near_a_corner_stay_side_walls() {
    // Deep along the flat side wall, short of the corner-arc start: still side.
    assert_eq!(
        wall_aerial_wall_classification(true, glam::Vec3::new(4096.0, 3800.0, 600.0)),
        WallAerialWall::Left,
    );
    // Just onto the corner arc near its side-wall end: the outward normal still
    // points mostly sideways.
    assert_eq!(
        wall_aerial_wall_classification(true, glam::Vec3::new(4050.0, 4300.0, 600.0)),
        WallAerialWall::Left,
    );
}
