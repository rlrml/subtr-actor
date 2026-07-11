use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};
use subtr_actor::{
    BoostPadEvent, BoostPadEventKind, DemoEventSample, DemolishInfo, DodgeRefreshedEvent,
    FrameEventsState, FrameInput, GameplayPhase, GoalEvent, LivePlayState, PlayerStatEvent,
    PlayerStatEventKind, ProcessorView, ReplayFrameInputBuilder, TouchEvent,
    boost_amount_to_percent, car_hitbox_for_body_id,
};

use super::*;
use crate::generator::{LiveEventHistory, explicit_demolish_events, frame_info, player_state};
use crate::meta::LiveMatchMeta;
use crate::model::{
    LiveCameraState, LiveControllerInput, LiveDemolishEvent, LiveEventTiming, LiveFrame,
    LiveMatchStats, LivePlayerFrame,
};

fn vec3(x: f32, y: f32, z: f32) -> Vector3f {
    Vector3f { x, y, z }
}

fn test_rigid_body(location: Vector3f, linear_velocity: Vector3f) -> RigidBody {
    RigidBody {
        location,
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        sleeping: false,
        linear_velocity: Some(linear_velocity),
        angular_velocity: Some(vec3(0.0, 0.0, 0.0)),
    }
}

fn player_at_index(player_index: u32, is_team_0: bool, location: Vector3f) -> LivePlayerFrame {
    LivePlayerFrame {
        player_index,
        is_team_0,
        rigid_body: Some(test_rigid_body(location, vec3(0.0, 0.0, 0.0))),
        boost_amount: 33.0,
        last_boost_amount: 33.0,
        match_stats: Some(LiveMatchStats {
            goals: player_index as i32,
            assists: player_index as i32 + 1,
            saves: player_index as i32 + 2,
            shots: player_index as i32 + 3,
            score: player_index as i32 + 100,
        }),
        ..LivePlayerFrame::default()
    }
}

fn live_frame(frame_number: u64, ball: RigidBody, players: Vec<LivePlayerFrame>) -> LiveFrame {
    LiveFrame {
        frame_number,
        time: frame_number as f32 * 0.1,
        dt: 0.1,
        seconds_remaining: Some(299),
        ball_has_been_hit: Some(true),
        ball: Some(ball),
        players,
        ..LiveFrame::default()
    }
}

fn demolish_event(attacker: u32, victim: u32) -> LiveDemolishEvent {
    LiveDemolishEvent {
        timing: LiveEventTiming::default(),
        attacker: RemoteId::SplitScreen(attacker),
        victim: RemoteId::SplitScreen(victim),
        attacker_velocity: vec3(2300.0, 0.0, 0.0),
        victim_velocity: vec3(0.0, 0.0, 0.0),
        victim_location: vec3(120.0, 0.0, 92.75),
        active_duration_seconds: 0.25,
    }
}

#[test]
fn live_processor_view_exposes_sampled_jump_state() {
    let mut player = player_at_index(3, true, vec3(0.0, 0.0, 120.0));
    player.jump_active = 1;
    player.double_jump_active = 1;
    player.dodge_active = 1;
    let frame = live_frame(
        1,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        vec![player],
    );
    let event_history = LiveEventHistory::default();
    let view = LiveProcessorView::new(None, frame, FrameEventsState::default(), &event_history);
    let player_id = RemoteId::SplitScreen(3);

    assert_eq!(view.get_jump_active(&player_id).unwrap(), 1);
    assert_eq!(view.get_double_jump_active(&player_id).unwrap(), 1);
    assert_eq!(view.get_dodge_active(&player_id).unwrap(), 1);
}

#[test]
fn live_processor_view_resolves_player_hitbox_from_car_body_id() {
    let mut player = player_at_index(3, true, vec3(0.0, 0.0, 120.0));
    player.car_body_id = Some(403);
    let players = vec![player];
    let expected = car_hitbox_for_body_id(403).expect("fixture car body should map to hitbox");
    let meta = LiveMatchMeta::from_player_frames(&players);
    let frame = live_frame(
        1,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        players.clone(),
    );
    let event_history = LiveEventHistory::default();
    let view = LiveProcessorView::new(None, frame, FrameEventsState::default(), &event_history);

    assert_eq!(
        view.get_player_car_hitbox(&RemoteId::SplitScreen(3)).family,
        expected.family
    );
    assert_eq!(
        player_state(&players).players[0].hitbox.family,
        expected.family
    );
    let replay_meta = meta.replay_meta();
    assert_eq!(replay_meta.team_zero[0].car_body_id, Some(403));
    assert_eq!(
        replay_meta.team_zero[0].car_hitbox_family.as_deref(),
        Some("Dominus")
    );
}

#[test]
fn live_processor_view_satisfies_processor_surface_from_live_frame() {
    let mut players = vec![
        player_at_index(2, true, vec3(-100.0, 20.0, 92.75)),
        player_at_index(5, false, vec3(120.0, 40.0, 92.75)),
    ];
    players[0].name = Some("Blue View".to_owned());
    players[0].boost_amount = 72.0;
    players[0].last_boost_amount = 68.0;
    players[0].boost_active = 1;
    players[0].jump_active = 1;
    players[0].double_jump_active = 1;
    players[0].dodge_active = 1;
    players[0].powerslide_active = true;
    if let Some(rigid_body) = players[0].rigid_body.as_mut() {
        rigid_body.linear_velocity = Some(vec3(0.0, 400.0, 0.0));
    }
    players[1].name = Some("Orange View".to_owned());

    let mut frame = live_frame(
        11,
        test_rigid_body(vec3(10.0, 20.0, 120.0), vec3(300.0, 0.0, 0.0)),
        players.clone(),
    );
    frame.seconds_remaining = Some(241);
    frame.game_state = Some(7);
    frame.kickoff_countdown_time = Some(3);
    frame.team_zero_score = Some(2);
    frame.team_one_score = Some(4);
    frame.possession_team_is_team_0 = Some(true);
    frame.scored_on_team_is_team_0 = Some(false);
    let frame_time = frame.time;

    let touch_events = vec![TouchEvent {
        touch_id: None,
        time: frame_time,
        frame: frame.frame_number as usize,
        player: Some(RemoteId::SplitScreen(2)),
        player_position: None,
        team_is_team_0: true,
        closest_approach_distance: Some(8.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }];
    let dodge_refreshed_events = vec![DodgeRefreshedEvent {
        time: frame_time,
        frame: frame.frame_number as usize,
        player: RemoteId::SplitScreen(2),
        player_position: None,
        is_team_0: true,
        counter_value: 9,
    }];
    let boost_pad_events = vec![BoostPadEvent {
        time: frame_time,
        frame: frame.frame_number as usize,
        pad_id: "34".to_owned(),
        player: Some(RemoteId::SplitScreen(2)),
        player_position: None,
        kind: BoostPadEventKind::PickedUp { sequence: 1 },
    }];
    let player_stat_events = vec![PlayerStatEvent {
        time: frame_time,
        frame: frame.frame_number as usize,
        player: RemoteId::SplitScreen(2),
        player_position: None,
        is_team_0: true,
        kind: PlayerStatEventKind::Shot,
        shot: None,
    }];
    let goal_events = vec![GoalEvent {
        time: frame_time,
        frame: frame.frame_number as usize,
        scoring_team_is_team_0: true,
        player: Some(RemoteId::SplitScreen(2)),
        player_position: None,
        team_zero_score: Some(3),
        team_one_score: Some(4),
    }];
    let demo_events = vec![DemolishInfo {
        frame: frame.frame_number as usize,
        time: frame_time,
        seconds_remaining: 241,
        attacker: RemoteId::SplitScreen(2),
        victim: RemoteId::SplitScreen(5),
        attacker_location: None,
        attacker_velocity: vec3(2300.0, 0.0, 0.0),
        victim_velocity: vec3(0.0, 200.0, 0.0),
        victim_location: vec3(120.0, 40.0, 92.75),
    }];
    let frame_events = FrameEventsState {
        active_demos: vec![DemoEventSample {
            attacker: RemoteId::SplitScreen(2),
            victim: RemoteId::SplitScreen(5),
        }],
        demo_events,
        boost_pad_events,
        touch_events,
        dodge_refreshed_counter_available: false,
        dodge_refreshed_events,
        player_stat_events,
        goal_events,
    };
    let replay_meta = LiveMatchMeta::from_player_frames(&players).replay_meta();
    let mut event_history = LiveEventHistory::default();
    event_history.append_frame_events(&frame_events);
    let view = LiveProcessorView::new(Some(&replay_meta), frame, frame_events, &event_history);
    let blue_id = RemoteId::SplitScreen(2);
    let orange_id = RemoteId::SplitScreen(5);

    assert_eq!(view.get_replay_meta().unwrap().player_count(), 2);
    assert_eq!(view.player_count(), 2);
    assert_eq!(
        view.iter_player_ids_in_order().cloned().collect::<Vec<_>>(),
        vec![blue_id.clone(), orange_id.clone()]
    );
    assert_eq!(view.current_in_game_team_player_counts(), [1, 1]);
    assert_eq!(view.get_seconds_remaining().unwrap(), 241);
    assert_eq!(view.get_replicated_state_name().unwrap(), 7);
    assert_eq!(view.get_replicated_game_state_time_remaining().unwrap(), 3);
    assert!(view.get_ball_has_been_hit().unwrap());
    assert!(!view.get_ignore_ball_syncing().unwrap());
    assert_eq!(view.get_team_scores().unwrap(), (2, 4));
    assert_eq!(view.get_ball_hit_team_num().unwrap(), 0);
    assert_eq!(view.get_scored_on_team_num().unwrap(), 1);

    assert_eq!(
        view.get_normalized_ball_rigid_body().unwrap().location.z,
        120.0
    );
    assert_eq!(
        view.get_velocity_applied_ball_rigid_body(frame_time)
            .unwrap()
            .linear_velocity
            .unwrap()
            .x,
        300.0
    );
    assert_eq!(
        view.get_velocity_applied_ball_rigid_body(frame_time + 0.5)
            .unwrap()
            .location
            .x,
        160.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame_time, 0.0)
            .unwrap()
            .location
            .x,
        10.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame_time + 0.5, 0.0)
            .unwrap()
            .location
            .x,
        160.0
    );
    assert_eq!(
        view.get_interpolated_ball_rigid_body(frame_time + 0.5, 0.5)
            .unwrap()
            .location
            .x,
        10.0
    );
    assert_eq!(
        view.get_normalized_player_rigid_body(&blue_id)
            .unwrap()
            .location
            .x,
        -100.0
    );
    assert_eq!(
        view.get_velocity_applied_player_rigid_body(&blue_id, frame_time)
            .unwrap()
            .location
            .z,
        92.75
    );
    assert_eq!(
        view.get_velocity_applied_player_rigid_body(&blue_id, frame_time + 0.5)
            .unwrap()
            .location
            .y,
        220.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame_time, 0.0)
            .unwrap()
            .location
            .y,
        20.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame_time + 0.5, 0.0)
            .unwrap()
            .location
            .y,
        220.0
    );
    assert_eq!(
        view.get_interpolated_player_rigid_body(&blue_id, frame_time + 0.5, 0.5)
            .unwrap()
            .location
            .y,
        20.0
    );

    assert_eq!(view.get_player_name(&blue_id).unwrap(), "Blue View");
    assert_eq!(view.get_player_team_key(&blue_id).unwrap(), "0");
    assert_eq!(view.get_player_team_key(&orange_id).unwrap(), "1");
    assert!(view.get_player_is_team_0(&blue_id).unwrap());
    assert!(!view.get_player_is_team_0(&orange_id).unwrap());
    assert_eq!(
        view.get_player_id_from_car_id(&boxcars::ActorId(2))
            .unwrap(),
        blue_id
    );
    assert!(
        view.get_player_id_from_car_id(&boxcars::ActorId(99))
            .is_err()
    );

    assert_eq!(view.get_player_boost_level(&blue_id).unwrap(), 72.0);
    assert_eq!(view.get_player_last_boost_level(&blue_id).unwrap(), 68.0);
    assert!(
        (view.get_player_boost_percentage(&blue_id).unwrap() - boost_amount_to_percent(72.0)).abs()
            < 1e-6
    );
    assert_eq!(view.get_boost_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_jump_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_double_jump_active(&blue_id).unwrap(), 1);
    assert_eq!(view.get_dodge_active(&blue_id).unwrap(), 1);
    assert!(view.get_powerslide_active(&blue_id).unwrap());
    assert_eq!(view.get_player_match_goals(&orange_id).unwrap(), 5);
    assert_eq!(view.get_player_match_assists(&orange_id).unwrap(), 6);
    assert_eq!(view.get_player_match_saves(&orange_id).unwrap(), 7);
    assert_eq!(view.get_player_match_shots(&orange_id).unwrap(), 8);
    assert_eq!(view.get_player_match_score(&orange_id).unwrap(), 105);

    assert!(view.get_throttle(&blue_id).is_err());
    assert!(view.get_steer(&blue_id).is_err());
    assert!(view.get_dodge_impulse(&blue_id).is_err());
    assert!(view.get_dodge_torque(&blue_id).is_err());
    assert!(view.get_camera_pitch(&blue_id).is_err());
    assert!(view.get_camera_yaw(&blue_id).is_err());

    let active_demos = view.get_active_demos().unwrap();
    assert_eq!(active_demos.len(), 1);
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].attacker_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(2)
    );
    assert_eq!(view.demolishes().len(), 1);
    assert_eq!(view.boost_pad_events().len(), 1);
    assert_eq!(view.touch_events().len(), 1);
    assert_eq!(view.dodge_refreshed_events().len(), 1);
    assert_eq!(view.player_stat_events().len(), 1);
    assert_eq!(view.goal_events().len(), 1);
    assert_eq!(view.current_frame_active_demo_events().len(), 1);
    assert_eq!(view.current_frame_demolish_events().len(), 1);
    assert_eq!(view.current_frame_boost_pad_events().len(), 1);
    assert_eq!(view.current_frame_touch_events().len(), 1);
    assert_eq!(view.current_frame_dodge_refreshed_events().len(), 1);
    assert_eq!(view.current_frame_player_stat_events().len(), 1);
    assert_eq!(view.current_frame_goal_events().len(), 1);
}

#[test]
fn live_processor_view_exposes_superset_input_camera_and_dodge_state() {
    let mut player = player_at_index(0, true, vec3(0.0, 0.0, 92.75));
    player.input = Some(LiveControllerInput {
        throttle: 1.0,
        steer: -1.0,
        ..LiveControllerInput::default()
    });
    player.camera = Some(LiveCameraState {
        pitch: Some(120),
        yaw: Some(200),
        ball_cam_active: Some(true),
    });
    player.dodge_impulse = Some([100.0, -50.0, 0.0]);
    player.dodge_torque = Some([0.4, -2.5, 0.0]);
    let frame = live_frame(
        3,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        vec![player],
    );
    let frame_time = frame.time;
    let frame_dt = frame.dt;
    let event_history = LiveEventHistory::default();
    let view = LiveProcessorView::new(None, frame, FrameEventsState::default(), &event_history);
    let player_id = RemoteId::SplitScreen(0);

    assert_eq!(view.get_throttle(&player_id).unwrap(), 255);
    assert_eq!(view.get_steer(&player_id).unwrap(), 0);
    assert_eq!(
        view.get_dodge_impulse(&player_id).unwrap(),
        (100.0, -50.0, 0.0)
    );
    assert_eq!(view.get_dodge_torque(&player_id).unwrap(), (0.4, -2.5, 0.0));
    assert_eq!(view.get_camera_pitch(&player_id).unwrap(), 120);
    assert_eq!(view.get_camera_yaw(&player_id).unwrap(), 200);

    let input = FrameInput::timeline_with_live_play_state(
        &view,
        3,
        frame_time,
        frame_dt,
        LivePlayState {
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
        },
    );
    let player_frame = input.player_frame_state();
    assert_eq!(
        player_frame.players[0].dodge_torque,
        Some(glam::Vec3::new(0.4, -2.5, 0.0))
    );
}

#[test]
fn live_processor_view_maps_neutral_input_axes_to_replay_bytes() {
    let mut player = player_at_index(0, true, vec3(0.0, 0.0, 92.75));
    player.input = Some(LiveControllerInput::default());
    let frame = live_frame(
        1,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        vec![player],
    );
    let event_history = LiveEventHistory::default();
    let view = LiveProcessorView::new(None, frame, FrameEventsState::default(), &event_history);
    let player_id = RemoteId::SplitScreen(0);

    assert_eq!(view.get_throttle(&player_id).unwrap(), 128);
    assert_eq!(view.get_steer(&player_id).unwrap(), 128);
}

#[test]
fn live_processor_view_resolves_players_by_remote_id_when_present() {
    let remote_id = RemoteId::Epic("epic-player".to_owned());
    let mut player = player_at_index(4, true, vec3(0.0, 0.0, 92.75));
    player.remote_id = Some(remote_id.clone());
    assert_eq!(player.canonical_player_id(), remote_id);
    let frame = live_frame(
        1,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        vec![player],
    );
    let event_history = LiveEventHistory::default();
    let view = LiveProcessorView::new(None, frame, FrameEventsState::default(), &event_history);

    assert_eq!(
        view.iter_player_ids_in_order().cloned().collect::<Vec<_>>(),
        vec![remote_id.clone()]
    );
    assert_eq!(view.get_player_boost_level(&remote_id).unwrap(), 33.0);
    assert!(
        view.get_player_boost_level(&RemoteId::SplitScreen(4))
            .is_err()
    );
    assert_eq!(
        view.get_player_id_from_car_id(&boxcars::ActorId(4))
            .unwrap(),
        remote_id
    );
}

#[test]
fn live_processor_view_exposes_cumulative_history_for_aggregate_inputs() {
    fn sample_events(frame: usize, time: f32) -> FrameEventsState {
        FrameEventsState {
            demo_events: vec![DemolishInfo {
                frame,
                time,
                seconds_remaining: 300,
                attacker: RemoteId::SplitScreen(0),
                victim: RemoteId::SplitScreen(1),
                attacker_location: None,
                attacker_velocity: vec3(2300.0, 0.0, 0.0),
                victim_velocity: vec3(0.0, 0.0, 0.0),
                victim_location: vec3(120.0, 0.0, 92.75),
            }],
            boost_pad_events: vec![BoostPadEvent {
                time,
                frame,
                pad_id: "34".to_owned(),
                player: Some(RemoteId::SplitScreen(0)),
                player_position: None,
                kind: BoostPadEventKind::PickedUp {
                    sequence: frame as u8,
                },
            }],
            touch_events: vec![TouchEvent {
                touch_id: None,
                time,
                frame,
                team_is_team_0: true,
                player: Some(RemoteId::SplitScreen(0)),
                player_position: None,
                closest_approach_distance: Some(12.0),
                contact_local_ball_position: None,
                contact_local_hitbox_point: None,
                contact_world_hitbox_point: None,
                dodge_contact: false,
            }],
            dodge_refreshed_events: vec![DodgeRefreshedEvent {
                time,
                frame,
                player: RemoteId::SplitScreen(0),
                player_position: None,
                is_team_0: true,
                counter_value: frame as i32,
            }],
            player_stat_events: vec![PlayerStatEvent {
                time,
                frame,
                player: RemoteId::SplitScreen(0),
                player_position: None,
                is_team_0: true,
                kind: PlayerStatEventKind::Shot,
                shot: None,
            }],
            goal_events: vec![GoalEvent {
                time,
                frame,
                scoring_team_is_team_0: true,
                player: Some(RemoteId::SplitScreen(0)),
                player_position: None,
                team_zero_score: Some(frame as i32),
                team_one_score: Some(0),
            }],
            ..FrameEventsState::default()
        }
    }

    let players = vec![
        player_at_index(0, true, vec3(0.0, 0.0, 92.75)),
        player_at_index(1, false, vec3(120.0, 0.0, 92.75)),
    ];
    let frame = live_frame(
        3,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        players,
    );
    let frame_time = frame.time;
    let frame_dt = frame.dt;
    let previous_events = sample_events(1, 0.0);
    let between_sample_events = sample_events(2, 0.5);
    let current_events = FrameEventsState {
        active_demos: vec![DemoEventSample {
            attacker: RemoteId::SplitScreen(0),
            victim: RemoteId::SplitScreen(1),
        }],
        ..FrameEventsState::default()
    };
    let mut event_history = LiveEventHistory::default();
    let mut builder = ReplayFrameInputBuilder::default();
    event_history.append_frame_events(&previous_events);
    let previous_view = LiveProcessorView::new(
        None,
        frame.clone(),
        FrameEventsState::default(),
        &event_history,
    );
    let _ = builder.aggregate(&previous_view, 2, 0.0, frame_dt);
    event_history.append_frame_events(&between_sample_events);
    let view = LiveProcessorView::new(None, frame, current_events, &event_history);

    assert_eq!(view.demolishes().len(), 2);
    assert_eq!(view.boost_pad_events().len(), 2);
    assert_eq!(view.touch_events().len(), 2);
    assert_eq!(view.dodge_refreshed_events().len(), 2);
    assert_eq!(view.player_stat_events().len(), 2);
    assert_eq!(view.goal_events().len(), 2);
    assert_eq!(view.current_frame_active_demo_events().len(), 1);
    assert_eq!(view.current_frame_demolish_events().len(), 0);
    assert_eq!(view.current_frame_boost_pad_events().len(), 0);
    assert_eq!(view.current_frame_touch_events().len(), 0);
    assert_eq!(view.current_frame_dodge_refreshed_events().len(), 0);
    assert_eq!(view.current_frame_player_stat_events().len(), 0);
    assert_eq!(view.current_frame_goal_events().len(), 0);

    let aggregate_input = builder.aggregate(&view, 3, frame_time, frame_dt);
    let aggregate_events = aggregate_input.frame_events_state();
    assert_eq!(aggregate_events.active_demos.len(), 1);
    assert_eq!(
        aggregate_events.active_demos[0].attacker,
        RemoteId::SplitScreen(0)
    );
    assert_eq!(aggregate_events.demo_events[0].frame, 2);
    assert_eq!(aggregate_events.boost_pad_events[0].frame, 2);
    assert_eq!(aggregate_events.touch_events[0].frame, 2);
    assert_eq!(aggregate_events.dodge_refreshed_events[0].frame, 2);
    assert_eq!(aggregate_events.player_stat_events[0].frame, 2);
    assert_eq!(aggregate_events.goal_events[0].frame, 2);
}

#[test]
fn live_processor_view_resolves_demo_car_actor_ids() {
    let players = vec![
        player_at_index(2, true, vec3(0.0, 0.0, 92.75)),
        player_at_index(5, false, vec3(120.0, 0.0, 92.75)),
    ];
    let frame = live_frame(
        7,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        players.clone(),
    );
    let frame_info = frame_info(&frame);
    let demo_events = explicit_demolish_events(
        &frame_info,
        &player_state(&players),
        &[demolish_event(2, 5)],
    );
    let event_history = LiveEventHistory::default();
    let view = LiveProcessorView::new(
        None,
        frame,
        FrameEventsState {
            active_demos: vec![DemoEventSample {
                attacker: RemoteId::SplitScreen(2),
                victim: RemoteId::SplitScreen(5),
            }],
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    let active_demos = view.get_active_demos().unwrap();
    assert_eq!(active_demos.len(), 1);
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].attacker_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(2)
    );
    assert_eq!(
        view.get_player_id_from_car_id(&active_demos[0].victim_actor_id())
            .unwrap(),
        RemoteId::SplitScreen(5)
    );
    assert_eq!(active_demos[0].attacker_velocity().x, 2300.0);
}

#[test]
fn live_frame_input_can_build_active_demos_from_processor_view() {
    let players = vec![
        player_at_index(0, true, vec3(0.0, 0.0, 92.75)),
        player_at_index(1, false, vec3(120.0, 0.0, 92.75)),
    ];
    let frame = live_frame(
        7,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        players.clone(),
    );
    let frame_time = frame.time;
    let frame_dt = frame.dt;
    let frame_info = frame_info(&frame);
    let demo_events = explicit_demolish_events(
        &frame_info,
        &player_state(&players),
        &[demolish_event(0, 1)],
    );
    let event_history = LiveEventHistory::default();
    let view = LiveProcessorView::new(
        None,
        frame,
        FrameEventsState {
            active_demos: vec![DemoEventSample {
                attacker: RemoteId::SplitScreen(0),
                victim: RemoteId::SplitScreen(1),
            }],
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    let input = FrameInput::timeline_with_live_play_state(
        &view,
        7,
        frame_time,
        frame_dt,
        LivePlayState {
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
        },
    );

    let frame_events = input.frame_events_state();
    assert_eq!(frame_events.active_demos.len(), 1);
    assert_eq!(
        frame_events.active_demos[0].attacker,
        RemoteId::SplitScreen(0)
    );
    assert_eq!(
        frame_events.active_demos[0].victim,
        RemoteId::SplitScreen(1)
    );
}

#[test]
fn live_processor_view_does_not_treat_inactive_demo_events_as_active() {
    let players = vec![
        player_at_index(0, true, vec3(0.0, 0.0, 92.75)),
        player_at_index(1, false, vec3(120.0, 0.0, 92.75)),
    ];
    let frame = live_frame(
        7,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        players,
    );
    let frame_time = frame.time;
    let frame_dt = frame.dt;
    let demo_events = vec![DemolishInfo {
        frame: 4,
        time: 0.4,
        seconds_remaining: 299,
        attacker: RemoteId::SplitScreen(0),
        victim: RemoteId::SplitScreen(1),
        attacker_location: None,
        attacker_velocity: vec3(2300.0, 0.0, 0.0),
        victim_velocity: vec3(0.0, 0.0, 0.0),
        victim_location: vec3(120.0, 0.0, 92.75),
    }];
    let event_history = LiveEventHistory::default();
    let view = LiveProcessorView::new(
        None,
        frame,
        FrameEventsState {
            demo_events,
            ..FrameEventsState::default()
        },
        &event_history,
    );

    assert!(
        view.get_active_demos().unwrap().is_empty(),
        "historical or expired live demo events should not be reported as active demos"
    );
    let input = FrameInput::timeline_with_live_play_state(
        &view,
        7,
        frame_time,
        frame_dt,
        LivePlayState {
            gameplay_phase: GameplayPhase::ActivePlay,
            is_live_play: true,
        },
    );
    let frame_events = input.frame_events_state();
    assert!(frame_events.active_demos.is_empty());
    assert_eq!(frame_events.demo_events.len(), 1);
}
