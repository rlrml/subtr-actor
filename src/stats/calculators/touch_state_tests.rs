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
    ball_at(glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z), velocity)
}

fn ball_at(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
    })
}

fn players(player_id: PlayerId) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![PlayerSample {
            player_id,
            is_team_0: true,
            hitbox: default_car_hitbox(),
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

fn player_sample(
    player_id: PlayerId,
    is_team_0: bool,
    position: glam::Vec3,
    velocity: glam::Vec3,
) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, velocity)),
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

#[test]
fn suppresses_same_player_touch_candidates_inside_cooldown() {
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
    let third_touch = calculator.update(
        &frame(4),
        &ball(glam::Vec3::new(1000.0, 0.0, 0.0)),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );

    assert_eq!(first_touch.touch_events.len(), 1);
    assert_eq!(first_touch.touch_events[0].player, Some(player_id.clone()));
    assert_eq!(first_touch.touch_events[0].frame, 1);

    assert!(second_touch.touch_events.is_empty());
    assert_eq!(second_touch.last_touch_player, Some(player_id.clone()));

    assert_eq!(third_touch.touch_events.len(), 1);
    assert_eq!(third_touch.touch_events[0].player, Some(player_id.clone()));
    assert_eq!(third_touch.touch_events[0].frame, 4);
    assert_eq!(third_touch.last_touch_player, Some(player_id));
}

#[test]
fn team_only_explicit_touch_events_without_physics_candidate_are_ignored() {
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
            player: None,
            player_position: None,
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

    assert!(touch_state.touch_events.is_empty());
    assert_eq!(touch_state.last_touch_player, None);
}

#[test]
fn player_explicit_touch_events_without_physics_candidate_are_accepted() {
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
            player_position: None,
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
    assert_eq!(touch_state.last_touch_player, Some(player_id));
}

#[test]
fn explicit_touch_events_respect_same_player_cooldown() {
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
        &FrameEventsState {
            touch_events: vec![TouchEvent {
                time: 0.1,
                frame: 1,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: None,
                dodge_contact: false,
            }],
            ..FrameEventsState::default()
        },
        &live_play,
    );
    let suppressed_touch = calculator.update(
        &frame(2),
        &ball(glam::Vec3::new(650.0, 0.0, 0.0)),
        &players,
        &FrameEventsState {
            touch_events: vec![TouchEvent {
                time: 0.2,
                frame: 2,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: None,
                dodge_contact: false,
            }],
            ..FrameEventsState::default()
        },
        &live_play,
    );
    let allowed_touch = calculator.update(
        &frame(4),
        &ball(glam::Vec3::new(1000.0, 0.0, 0.0)),
        &players,
        &FrameEventsState {
            touch_events: vec![TouchEvent {
                time: 0.4,
                frame: 4,
                team_is_team_0: true,
                player: Some(player_id.clone()),
                player_position: None,
                closest_approach_distance: None,
                dodge_contact: false,
            }],
            ..FrameEventsState::default()
        },
        &live_play,
    );

    assert_eq!(first_touch.touch_events.len(), 1);
    assert!(suppressed_touch.touch_events.is_empty());
    assert_eq!(suppressed_touch.last_touch_player, Some(player_id.clone()));
    assert_eq!(allowed_touch.touch_events.len(), 1);
    assert_eq!(allowed_touch.touch_events[0].frame, 4);
    assert_eq!(allowed_touch.last_touch_player, Some(player_id));
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
            player_position: None,
            closest_approach_distance: None,
            dodge_contact: false,
        }],
        ..FrameEventsState::default()
    };

    calculator.update(
        &frame(0),
        &ball(glam::Vec3::ZERO),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );
    let touch_state = calculator.update(
        &frame(1),
        &ball(glam::Vec3::new(300.0, 0.0, 0.0)),
        &players,
        &events,
        &live_play,
    );

    assert_eq!(
        touch_state.touch_events[0].closest_approach_distance,
        Some(0.0)
    );
}

#[test]
fn recent_touch_candidate_cache_preserves_best_scored_candidate_for_player() {
    let player_id = boxcars::RemoteId::Steam(1);
    let hitbox = default_car_hitbox();
    let strict_players = PlayerFrameState {
        players: vec![player_sample(
            player_id.clone(),
            true,
            glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z),
            glam::Vec3::ZERO,
        )],
    };
    let relaxed_players = PlayerFrameState {
        players: vec![player_sample(
            player_id.clone(),
            true,
            glam::Vec3::new(
                0.0,
                -(hitbox.width / 2.0 + BALL_COLLISION_RADIUS + 10.0),
                BALL_RADIUS_Z,
            ),
            glam::Vec3::ZERO,
        )],
    };
    let mut calculator = TouchStateCalculator::new();
    let frame_zero_ball = ball(glam::Vec3::ZERO);
    let frame_one_ball = ball(glam::Vec3::new(1500.0, 0.0, 0.0));
    let frame_two_ball = ball(glam::Vec3::new(3000.0, 0.0, 0.0));

    calculator.previous_ball_rigid_body = frame_zero_ball
        .sample()
        .map(|sample| (sample.rigid_body, 0.0));
    calculator.update_recent_touch_candidates(&frame(1), &frame_one_ball, &strict_players);
    calculator.previous_ball_rigid_body = frame_one_ball
        .sample()
        .map(|sample| (sample.rigid_body, 0.1));
    calculator.update_recent_touch_candidates(&frame(2), &frame_two_ball, &relaxed_players);

    let cached = calculator
        .candidate_for_player(&player_id)
        .expect("player should have a cached recent candidate");
    assert_eq!(cached.player, Some(player_id));
    assert_eq!(cached.frame, 1);
    assert_eq!(cached.closest_approach_distance, Some(0.0));
}

#[test]
fn current_frame_physics_candidate_wins_over_explicit_team_hint() {
    let hinted_player_id = boxcars::RemoteId::Steam(1);
    let physics_player_id = boxcars::RemoteId::Steam(2);
    let players = PlayerFrameState {
        players: vec![
            player_sample(
                hinted_player_id,
                true,
                glam::Vec3::new(1200.0, 0.0, BALL_RADIUS_Z),
                glam::Vec3::ZERO,
            ),
            player_sample(
                physics_player_id.clone(),
                false,
                glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z),
                glam::Vec3::ZERO,
            ),
        ],
    };
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
    let touch_state = calculator.update(
        &frame(1),
        &ball(glam::Vec3::new(300.0, 0.0, 0.0)),
        &players,
        &FrameEventsState {
            touch_events: vec![TouchEvent {
                time: 0.1,
                frame: 1,
                team_is_team_0: true,
                player: None,
                player_position: None,
                closest_approach_distance: None,
                dodge_contact: false,
            }],
            ..FrameEventsState::default()
        },
        &live_play,
    );

    assert_eq!(touch_state.touch_events.len(), 1);
    assert_eq!(touch_state.touch_events[0].player, Some(physics_player_id));
    assert!(!touch_state.touch_events[0].team_is_team_0);
}

#[test]
fn strict_contact_candidate_requires_velocity_deviation() {
    let player_id = boxcars::RemoteId::Steam(1);
    let players = players(player_id);
    let mut calculator = TouchStateCalculator::new();
    let live_play = LivePlayState {
        gameplay_phase: GameplayPhase::ActivePlay,
        is_live_play: true,
    };

    calculator.update(
        &frame(0),
        &ball_at(glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z), glam::Vec3::ZERO),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );
    let touch_state = calculator.update(
        &frame(1),
        &ball_at(
            glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z - 3.25),
            glam::Vec3::new(0.0, 0.0, -65.0),
        ),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );

    assert!(touch_state.touch_events.is_empty());
    assert_eq!(touch_state.last_touch_player, None);
}

#[test]
fn relaxed_gap_candidate_requires_large_velocity_deviation() {
    let player_id = boxcars::RemoteId::Steam(1);
    let hitbox = default_car_hitbox();
    let player_position = glam::Vec3::new(
        0.0,
        -(hitbox.width / 2.0 + BALL_COLLISION_RADIUS + 10.0),
        BALL_RADIUS_Z,
    );
    let players = PlayerFrameState {
        players: vec![player_sample(
            player_id.clone(),
            true,
            player_position,
            glam::Vec3::ZERO,
        )],
    };
    let live_play = LivePlayState {
        gameplay_phase: GameplayPhase::ActivePlay,
        is_live_play: true,
    };

    let mut low_velocity_calculator = TouchStateCalculator::new();
    low_velocity_calculator.update(
        &frame(0),
        &ball(glam::Vec3::ZERO),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );
    let low_velocity_touch = low_velocity_calculator.update(
        &frame(1),
        &ball(glam::Vec3::new(300.0, 0.0, 0.0)),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );

    let mut high_velocity_calculator = TouchStateCalculator::new();
    high_velocity_calculator.update(
        &frame(0),
        &ball(glam::Vec3::ZERO),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );
    let high_velocity_touch = high_velocity_calculator.update(
        &frame(1),
        &ball(glam::Vec3::new(1500.0, 0.0, 0.0)),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );

    assert!(low_velocity_touch.touch_events.is_empty());
    assert_eq!(high_velocity_touch.touch_events.len(), 1);
    assert_eq!(high_velocity_touch.touch_events[0].player, Some(player_id));
    let gap = high_velocity_touch.touch_events[0]
        .closest_approach_distance
        .expect("relaxed candidate should include contact gap");
    assert!(gap > 5.0 && gap <= 25.0, "unexpected relaxed gap: {gap}");
}

#[test]
fn simultaneous_close_candidates_are_all_emitted() {
    let team_zero_player_id = boxcars::RemoteId::Steam(1);
    let team_one_player_id = boxcars::RemoteId::Steam(2);
    let players = PlayerFrameState {
        players: vec![
            player_sample(
                team_zero_player_id.clone(),
                true,
                glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z),
                glam::Vec3::ZERO,
            ),
            player_sample(
                team_one_player_id.clone(),
                false,
                glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z),
                glam::Vec3::ZERO,
            ),
        ],
    };
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
    let touch_state = calculator.update(
        &frame(1),
        &ball(glam::Vec3::new(300.0, 0.0, 0.0)),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );

    assert_eq!(touch_state.touch_events.len(), 2);
    assert!(touch_state
        .touch_events
        .iter()
        .any(|touch| touch.player.as_ref() == Some(&team_zero_player_id)));
    assert!(touch_state
        .touch_events
        .iter()
        .any(|touch| touch.player.as_ref() == Some(&team_one_player_id)));
    assert!(touch_state
        .touch_events
        .iter()
        .any(|touch| touch.team_is_team_0));
    assert!(touch_state
        .touch_events
        .iter()
        .any(|touch| !touch.team_is_team_0));
}

#[test]
fn simultaneous_candidates_keep_best_candidate_as_last_touch() {
    let best_player_id = boxcars::RemoteId::Steam(1);
    let secondary_player_id = boxcars::RemoteId::Steam(2);
    let hitbox = default_car_hitbox();
    let players = PlayerFrameState {
        players: vec![
            player_sample(
                best_player_id.clone(),
                true,
                glam::Vec3::new(0.0, 0.0, BALL_RADIUS_Z),
                glam::Vec3::ZERO,
            ),
            player_sample(
                secondary_player_id.clone(),
                true,
                glam::Vec3::new(
                    0.0,
                    -(hitbox.width / 2.0 + BALL_COLLISION_RADIUS + 3.0),
                    BALL_RADIUS_Z,
                ),
                glam::Vec3::ZERO,
            ),
        ],
    };
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
    let touch_state = calculator.update(
        &frame(1),
        &ball(glam::Vec3::new(300.0, 0.0, 0.0)),
        &players,
        &FrameEventsState::default(),
        &live_play,
    );

    assert_eq!(touch_state.touch_events.len(), 2);
    assert_eq!(touch_state.last_touch_player, Some(best_player_id));
    let best_gap = touch_state
        .last_touch
        .as_ref()
        .and_then(|touch| touch.closest_approach_distance)
        .expect("last touch should include contact gap");
    let secondary_gap = touch_state
        .touch_events
        .iter()
        .find(|touch| touch.player.as_ref() == Some(&secondary_player_id))
        .and_then(|touch| touch.closest_approach_distance)
        .expect("secondary touch should include contact gap");
    assert!(best_gap < secondary_gap);
}

#[test]
fn primary_touch_event_only_uses_current_frame_touch_events() {
    let player_id = boxcars::RemoteId::Steam(1);
    let stale_touch = TouchEvent {
        time: 0.1,
        frame: 1,
        team_is_team_0: true,
        player: Some(player_id),
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    };
    let touch_state = TouchState {
        touch_events: Vec::new(),
        last_touch: Some(stale_touch),
        last_touch_player: None,
        last_touch_team_is_team_0: None,
    };

    assert!(touch_state.primary_touch_event().is_none());
}

#[test]
fn contested_touch_candidates_include_all_close_opponents() {
    let primary = TouchEvent {
        time: 0.1,
        frame: 1,
        team_is_team_0: true,
        player: Some(boxcars::RemoteId::Steam(1)),
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    };
    let close_opponent = TouchEvent {
        time: 0.1,
        frame: 1,
        team_is_team_0: false,
        player: Some(boxcars::RemoteId::Steam(2)),
        player_position: None,
        closest_approach_distance: Some(1.0),
        dodge_contact: false,
    };
    let second_close_opponent = TouchEvent {
        time: 0.1,
        frame: 1,
        team_is_team_0: false,
        player: Some(boxcars::RemoteId::Steam(3)),
        player_position: None,
        closest_approach_distance: Some(4.0),
        dodge_contact: false,
    };
    let loose_opponent = TouchEvent {
        time: 0.1,
        frame: 1,
        team_is_team_0: false,
        player: Some(boxcars::RemoteId::Steam(4)),
        player_position: None,
        closest_approach_distance: Some(8.0),
        dodge_contact: false,
    };
    let mut calculator = TouchStateCalculator::new();
    for event in [
        close_opponent.clone(),
        loose_opponent,
        second_close_opponent.clone(),
    ] {
        calculator
            .recent_touch_candidates
            .insert(event.player.clone().unwrap(), event);
    }

    let contested = calculator.contested_touch_candidates(&primary);

    assert_eq!(contested, vec![close_opponent, second_close_opponent]);
}
