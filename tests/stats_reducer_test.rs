use std::collections::HashMap;

use boxcars::HeaderProp;
use subtr_actor::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn epic_id(name: &str) -> PlayerId {
    boxcars::RemoteId::Epic(name.to_string())
}

fn sample_player(player_id: PlayerId, is_team_0: bool) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        rigid_body: None,
        boost_amount: None,
        boost_active: false,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

#[test]
fn test_powerslide_reducer_collects_duration_and_presses() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let reducer = ReducerCollector::new(PowerslideReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with powerslide reducer")
        .into_inner();

    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.total_duration > 0.0),
        "Expected at least one player to have non-zero powerslide duration"
    );
    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.press_count > 0),
        "Expected at least one player to have non-zero powerslide press count"
    );
    assert!(
        reducer.team_zero_stats().total_duration > 0.0
            || reducer.team_one_stats().total_duration > 0.0,
        "Expected at least one team to have non-zero powerslide duration"
    );
}

#[test]
fn test_pressure_reducer_tracks_ball_side_time() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let reducer = ReducerCollector::new(PressureReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with pressure reducer")
        .into_inner();

    assert!(
        reducer.team_zero_side_duration() > 0.0,
        "Expected non-zero tracked time on team zero side"
    );
    assert!(
        reducer.team_one_side_duration() > 0.0,
        "Expected non-zero tracked time on team one side"
    );
    assert!(
        reducer.total_tracked_duration() > 0.0,
        "Expected pressure reducer to track some ball time"
    );
}

#[test]
fn test_tuple_reducers_compose_under_frame_rate_decorator() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let mut collector = ReducerCollector::new((PowerslideReducer::new(), PressureReducer::new()));

    FrameRateDecorator::new_from_fps(10.0, &mut collector)
        .process_replay(&replay)
        .expect("Failed to process replay with composed reducers");

    let (powerslide, pressure) = collector.into_inner();

    assert!(
        powerslide
            .player_stats()
            .values()
            .any(|stats| stats.press_count > 0),
        "Expected composed powerslide reducer to record presses"
    );
    assert!(
        pressure.total_tracked_duration() > 0.0,
        "Expected composed pressure reducer to track ball-side time"
    );
}

#[test]
fn test_match_stats_reducer_builds_core_stats_and_timeline() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let reducer = ReducerCollector::new(MatchStatsReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with match stats reducer")
        .into_inner();

    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.shots > 0 || stats.goals > 0 || stats.saves > 0),
        "Expected at least one player to have non-zero core stats"
    );
    assert!(
        !reducer.timeline().is_empty(),
        "Expected match stats reducer to emit timeline events"
    );
    assert!(
        reducer.team_zero_stats().shots > 0 || reducer.team_one_stats().shots > 0,
        "Expected at least one team to have non-zero shot totals"
    );
}

#[test]
fn test_match_stats_reducer_prefers_exact_goal_event_times() {
    let player_id = epic_id("goal-scorer");
    let mut reducer = MatchStatsReducer::new();

    reducer
        .on_sample(&StatsSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![sample_player(player_id.clone(), true)],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: Some(1),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: Some(false),
            ball: None,
            players: vec![PlayerSample {
                match_goals: Some(1),
                match_shots: Some(1),
                match_score: Some(100),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: vec![GoalEvent {
                time: 1.25,
                frame: 2,
                scoring_team_is_team_0: true,
                team_zero_score: Some(1),
                team_one_score: Some(0),
            }],
        })
        .unwrap();

    let goal_event = reducer
        .timeline()
        .iter()
        .find(|event| event.kind == TimelineEventKind::Goal)
        .expect("Expected a goal timeline event");
    assert_eq!(goal_event.player_id.as_ref(), Some(&player_id));
    assert!((goal_event.time - 1.25).abs() < 0.001);
}

#[test]
fn test_movement_reducer_collects_distance_and_speed_buckets() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let reducer = ReducerCollector::new(MovementReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with movement reducer")
        .into_inner();

    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.total_distance > 0.0),
        "Expected at least one player to have non-zero total distance"
    );
    assert!(
        reducer.team_zero_stats().tracked_time > 0.0 && reducer.team_one_stats().tracked_time > 0.0,
        "Expected both teams to accumulate movement tracked time"
    );
}

#[test]
fn test_positioning_reducer_collects_distances_and_percent_buckets() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let reducer = ReducerCollector::new(PositioningReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with positioning reducer")
        .into_inner();

    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.average_distance_to_ball() > 0.0),
        "Expected at least one player to have non-zero average distance to ball"
    );
    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.time_defensive_half > 0.0 || stats.time_offensive_half > 0.0),
        "Expected at least one player to accumulate positioning bucket time"
    );
}

#[test]
fn test_positioning_reducer_uses_touch_event_boundaries_for_possession_buckets() {
    let mut reducer = PositioningReducer::new();
    let team_zero_id = epic_id("team-zero-positioning");
    let team_one_id = epic_id("team-one-positioning");
    let ball = BallSample {
        rigid_body: boxcars::RigidBody {
            sleeping: false,
            location: boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 92.75,
            },
            rotation: boxcars::Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            angular_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        },
    };
    let team_zero_player = PlayerSample {
        rigid_body: Some(boxcars::RigidBody {
            sleeping: false,
            location: boxcars::Vector3f {
                x: 10.0,
                y: -100.0,
                z: 17.0,
            },
            rotation: boxcars::Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            angular_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }),
        ..sample_player(team_zero_id.clone(), true)
    };
    let team_one_player = PlayerSample {
        rigid_body: Some(boxcars::RigidBody {
            sleeping: false,
            location: boxcars::Vector3f {
                x: 20.0,
                y: 100.0,
                z: 17.0,
            },
            rotation: boxcars::Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            angular_velocity: Some(boxcars::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }),
        ..sample_player(team_one_id.clone(), false)
    };

    reducer
        .on_sample(&StatsSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            ball: Some(ball.clone()),
            players: vec![team_zero_player.clone(), team_one_player.clone()],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 1.0,
                frame: 1,
                team_is_team_0: true,
                player: Some(team_zero_id.clone()),
            }],
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            ball: Some(ball.clone()),
            players: vec![team_zero_player.clone(), team_one_player.clone()],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            ball: Some(ball.clone()),
            players: vec![team_zero_player.clone(), team_one_player.clone()],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 3.0,
                frame: 3,
                team_is_team_0: false,
                player: Some(team_one_id.clone()),
            }],
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 4,
            time: 4.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            ball: Some(ball),
            players: vec![team_zero_player, team_one_player],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let team_zero_stats = reducer.player_stats().get(&team_zero_id).unwrap();
    let team_one_stats = reducer.player_stats().get(&team_one_id).unwrap();
    assert_eq!(team_zero_stats.time_has_possession, 2.0);
    assert_eq!(team_zero_stats.time_no_possession, 1.0);
    assert_eq!(team_one_stats.time_has_possession, 1.0);
    assert_eq!(team_one_stats.time_no_possession, 2.0);
}

#[test]
fn test_boost_reducer_collects_amounts_and_buckets() {
    let replay = parse_replay("assets/replays/new_boost_format.replay");
    let reducer = ReducerCollector::new(BoostReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with boost reducer")
        .into_inner();

    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.amount_collected > 0.0),
        "Expected at least one player to collect some boost"
    );
    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.average_boost_amount() > 0.0),
        "Expected at least one player to have non-zero average boost amount"
    );
    assert!(
        reducer.team_zero_stats().bpm() > 0.0 || reducer.team_one_stats().bpm() > 0.0,
        "Expected at least one team to have non-zero BPM"
    );
}

#[test]
fn test_boost_reducer_uses_exact_pad_events_for_size_and_overfill() {
    let player_id = epic_id("boost-player");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&StatsSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 0.0,
                        y: 1000.0,
                        z: 17.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 0.0,
                        y: 1000.0,
                        z: 17.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 3,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 0.0,
                        y: 1000.0,
                        z: 17.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 3,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 1);
    assert_eq!(stats.small_pads_collected, 0);
    assert!((stats.amount_collected_big - 55.0).abs() < 0.001);
    assert!((stats.overfill_total - 200.0).abs() < 0.001);
}

#[test]
fn test_boost_reducer_uses_canonical_pad_layout_for_stolen_classification() {
    let player_id = epic_id("midline-player");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&StatsSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(0.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 3584.0,
                        y: 50.0,
                        z: 73.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 3584.0,
                        y: 50.0,
                        z: 73.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_centerline_big".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 3,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 3584.0,
                        y: 50.0,
                        z: 73.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 3,
                pad_id: "VehiclePickup_Boost_TA_centerline_big".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 1);
    assert_eq!(stats.amount_stolen, 0.0);
    assert_eq!(stats.big_pads_stolen, 0);
}

#[test]
fn test_boost_reducer_treats_midfield_fallback_pads_as_neutral() {
    let player_id = epic_id("fallback-midfield-player");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&StatsSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(0.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 5000.0,
                        y: 64.0,
                        z: 73.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 5000.0,
                        y: 64.0,
                        z: 73.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_nonstandard_midfield".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 3,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(boxcars::RigidBody {
                    sleeping: false,
                    location: boxcars::Vector3f {
                        x: 5000.0,
                        y: 64.0,
                        z: 73.0,
                    },
                    rotation: boxcars::Quaternion {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    },
                    linear_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    angular_velocity: Some(boxcars::Vector3f {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                }),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 3,
                pad_id: "VehiclePickup_Boost_TA_nonstandard_midfield".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 1);
    assert_eq!(stats.amount_stolen, 0.0);
    assert_eq!(stats.big_pads_stolen, 0);
}

#[test]
fn test_possession_reducer_tracks_team_possession_time() {
    let mut reducer = PossessionReducer::new();
    let team_zero = sample_player(epic_id("team-zero"), true);
    let team_one = sample_player(epic_id("team-one"), false);

    reducer
        .on_sample(&StatsSample {
            frame_number: 1,
            time: 1.0,
            dt: 1.5,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();
    reducer
        .on_sample(&StatsSample {
            frame_number: 2,
            time: 2.0,
            dt: 0.5,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![team_zero, team_one],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    assert_eq!(reducer.stats().tracked_time, 2.0);
    assert_eq!(reducer.stats().team_zero_time, 2.0);
    assert_eq!(reducer.stats().team_one_time, 0.0);
    assert!((reducer.stats().team_zero_pct() - 100.0).abs() < 0.001);
    assert!((reducer.stats().team_one_pct() - 0.0).abs() < 0.001);
}

#[test]
fn test_possession_reducer_uses_touch_event_boundaries() {
    let mut reducer = PossessionReducer::new();
    let team_zero = sample_player(epic_id("team-zero"), true);
    let team_one = sample_player(epic_id("team-one"), false);

    reducer
        .on_sample(&StatsSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 1.0,
                frame: 1,
                team_is_team_0: true,
                player: Some(team_zero.player_id.clone()),
            }],
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 3.0,
                frame: 3,
                team_is_team_0: false,
                player: Some(team_one.player_id.clone()),
            }],
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&StatsSample {
            frame_number: 4,
            time: 4.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            ball: None,
            players: vec![team_zero, team_one],
            active_demos: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    assert_eq!(reducer.stats().tracked_time, 3.0);
    assert_eq!(reducer.stats().team_zero_time, 2.0);
    assert_eq!(reducer.stats().team_one_time, 1.0);
}

#[test]
fn test_demo_reducer_counts_events_and_dedupes_consecutive_frames() {
    let attacker = epic_id("attacker");
    let victim = epic_id("victim");
    let mut reducer = DemoReducer::new();

    for frame_number in [10, 11, 30] {
        reducer
            .on_sample(&StatsSample {
                frame_number,
                time: frame_number as f32 / 10.0,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                team_zero_score: None,
                team_one_score: None,
                possession_team_is_team_0: None,
                scored_on_team_is_team_0: None,
                ball: None,
                players: vec![
                    sample_player(attacker.clone(), true),
                    sample_player(victim.clone(), false),
                ],
                active_demos: vec![DemoEventSample {
                    attacker: attacker.clone(),
                    victim: victim.clone(),
                }],
                boost_pad_events: Vec::new(),
                touch_events: Vec::new(),
                goal_events: Vec::new(),
            })
            .unwrap();
    }

    let attacker_stats = reducer.player_stats().get(&attacker).unwrap();
    let victim_stats = reducer.player_stats().get(&victim).unwrap();
    assert_eq!(attacker_stats.demos_inflicted, 2);
    assert_eq!(victim_stats.demos_taken, 2);
    assert_eq!(reducer.team_zero_stats().demos_inflicted, 2);
    assert_eq!(reducer.team_one_stats().demos_inflicted, 0);
    assert_eq!(reducer.timeline().len(), 4);
    assert_eq!(reducer.timeline()[0].kind, TimelineEventKind::Kill);
    assert_eq!(reducer.timeline()[1].kind, TimelineEventKind::Death);
}

#[test]
fn test_settings_reducer_extracts_player_camera_settings_from_replay_meta() {
    let player_id = epic_id("settings-player");
    let mut reducer = SettingsReducer::new();
    let replay_meta = ReplayMeta {
        team_zero: vec![PlayerInfo {
            remote_id: player_id.clone(),
            name: "settings-player".to_string(),
            stats: Some(HashMap::from([
                ("SteeringSensitivity".to_string(), HeaderProp::Float(1.4)),
                ("CameraFOV".to_string(), HeaderProp::Float(110.0)),
                ("CameraHeight".to_string(), HeaderProp::Int(100)),
                ("CameraPitch".to_string(), HeaderProp::Float(-3.0)),
                ("CameraDistance".to_string(), HeaderProp::Int(270)),
                ("CameraStiffness".to_string(), HeaderProp::Float(0.45)),
                ("CameraSwivelSpeed".to_string(), HeaderProp::Float(5.3)),
                ("CameraTransitionSpeed".to_string(), HeaderProp::Float(1.2)),
            ])),
        }],
        team_one: Vec::new(),
        all_headers: Vec::new(),
    };

    reducer.on_replay_meta(&replay_meta).unwrap();

    let settings = reducer.player_settings().get(&player_id).unwrap();
    assert_eq!(settings.steering_sensitivity, Some(1.4));
    assert_eq!(settings.camera_fov, Some(110.0));
    assert_eq!(settings.camera_height, Some(100.0));
    assert_eq!(settings.camera_pitch, Some(-3.0));
    assert_eq!(settings.camera_distance, Some(270.0));
    assert_eq!(settings.camera_stiffness, Some(0.45));
    assert_eq!(settings.camera_swivel_speed, Some(5.3));
    assert_eq!(settings.camera_transition_speed, Some(1.2));
}

#[test]
fn test_demo_reducer_collects_real_demolitions_from_replay() {
    let replay = parse_replay("assets/replays/test/new_demolition_format.replay");
    let reducer = ReducerCollector::new(DemoReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with demo reducer")
        .into_inner();

    let total_demos =
        reducer.team_zero_stats().demos_inflicted + reducer.team_one_stats().demos_inflicted;
    assert_eq!(total_demos, 10, "Expected to recover all demolitions");
    assert_eq!(
        reducer
            .timeline()
            .iter()
            .filter(|event| event.kind == TimelineEventKind::Kill)
            .count(),
        10,
        "Expected one kill timeline event per demolish"
    );
    assert_eq!(
        reducer
            .timeline()
            .iter()
            .filter(|event| event.kind == TimelineEventKind::Death)
            .count(),
        10,
        "Expected one death timeline event per demolish"
    );
}
