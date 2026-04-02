use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

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

fn sample_rigid_body(x: f32, y: f32, z: f32) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: boxcars::Vector3f { x, y, z },
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
    }
}

fn sample_rigid_body_with_linear_velocity(
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
) -> boxcars::RigidBody {
    let mut rigid_body = sample_rigid_body(x, y, z);
    rigid_body.linear_velocity = Some(boxcars::Vector3f {
        x: vx,
        y: vy,
        z: vz,
    });
    rigid_body
}

fn sample_stats(
    frame_number: usize,
    time: f32,
    dt: f32,
    ball: Option<boxcars::RigidBody>,
    players: Vec<PlayerSample>,
) -> CoreSample {
    CoreSample {
        frame_number,
        time,
        dt,
        seconds_remaining: Some(100),
        game_state: Some(0),
        ball_has_been_hit: Some(true),
        kickoff_countdown_time: None,
        team_zero_score: None,
        team_one_score: None,
        possession_team_is_team_0: None,
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: None,
        ball: ball.map(|rigid_body| BallSample { rigid_body }),
        players,
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: Vec::new(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

fn replay_player_boost_sample_at_frame(
    replay_data: &ReplayData,
    player_id: &PlayerId,
    frame_number: usize,
) -> Option<(usize, f32)> {
    let (_, player_data) = replay_data
        .frame_data
        .players
        .iter()
        .find(|(candidate, _)| candidate == player_id)?;
    match player_data.frames().get(frame_number)? {
        PlayerFrame::Data { boost_amount, .. } => Some((frame_number, *boost_amount)),
        PlayerFrame::Empty => None,
    }
}

#[derive(Clone)]
struct RecordingReducer {
    replay_meta_calls: Rc<RefCell<usize>>,
    sample_calls: Rc<RefCell<usize>>,
    finish_calls: Rc<RefCell<usize>>,
}

impl RecordingReducer {
    fn new(
        replay_meta_calls: Rc<RefCell<usize>>,
        sample_calls: Rc<RefCell<usize>>,
        finish_calls: Rc<RefCell<usize>>,
    ) -> Self {
        Self {
            replay_meta_calls,
            sample_calls,
            finish_calls,
        }
    }
}

impl StatsReducer for RecordingReducer {
    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        *self.replay_meta_calls.borrow_mut() += 1;
        Ok(())
    }

    fn on_sample(&mut self, _sample: &CoreSample) -> SubtrActorResult<()> {
        *self.sample_calls.borrow_mut() += 1;
        Ok(())
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        *self.finish_calls.borrow_mut() += 1;
        Ok(())
    }
}

#[test]
fn test_powerslide_reducer_collects_duration_and_presses() {
    let replay = parse_replay("assets/replays/rlcs.replay");
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
fn test_ball_carry_reducer_tracks_single_carry() {
    let player_id = epic_id("carry-player");
    let mut reducer = BallCarryReducer::new();

    let mut touch_sample = sample_stats(
        1,
        0.0,
        0.0,
        Some(sample_rigid_body(0.0, 0.0, 140.0)),
        vec![PlayerSample {
            rigid_body: Some(sample_rigid_body_with_linear_velocity(
                0.0, 0.0, 17.0, 400.0, 0.0, 0.0,
            )),
            ..sample_player(player_id.clone(), true)
        }],
    );
    touch_sample.touch_events.push(TouchEvent {
        time: 0.0,
        frame: 1,
        team_is_team_0: true,
        player: Some(player_id.clone()),
        closest_approach_distance: Some(40.0),
    });
    reducer.on_sample(&touch_sample).unwrap();

    for (frame_number, time, player_x) in [(2, 0.5, 50.0), (3, 1.0, 100.0), (4, 1.5, 150.0)] {
        reducer
            .on_sample(&sample_stats(
                frame_number,
                time,
                0.5,
                Some(sample_rigid_body(player_x, 0.0, 150.0)),
                vec![PlayerSample {
                    rigid_body: Some(sample_rigid_body_with_linear_velocity(
                        player_x, 0.0, 17.0, 600.0, 0.0, 0.0,
                    )),
                    ..sample_player(player_id.clone(), true)
                }],
            ))
            .unwrap();
    }

    reducer.finish().unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.carry_count, 1);
    assert!((stats.total_carry_time - 1.5).abs() < 0.001);
    assert!((stats.longest_carry_time - 1.5).abs() < 0.001);
    assert!((stats.furthest_carry_distance - 100.0).abs() < 0.001);
    assert!(stats.average_horizontal_gap() < 5.0);

    let event = reducer.carry_events().first().unwrap();
    assert_eq!(event.start_frame, 1);
    assert_eq!(event.end_frame, 4);
    assert!((event.duration - 1.5).abs() < 0.001);
    assert!((event.straight_line_distance - 100.0).abs() < 0.001);
    assert!((event.path_distance - 100.0).abs() < 0.001);
}

#[test]
fn test_ball_carry_reducer_requires_last_touch_owner() {
    let player_id = epic_id("carry-no-touch");
    let mut reducer = BallCarryReducer::new();

    for (frame_number, time) in [(1, 0.5), (2, 1.0), (3, 1.5)] {
        reducer
            .on_sample(&sample_stats(
                frame_number,
                time,
                0.5,
                Some(sample_rigid_body(time * 100.0, 0.0, 145.0)),
                vec![PlayerSample {
                    rigid_body: Some(sample_rigid_body_with_linear_velocity(
                        time * 100.0,
                        0.0,
                        17.0,
                        500.0,
                        0.0,
                        0.0,
                    )),
                    ..sample_player(player_id.clone(), true)
                }],
            ))
            .unwrap();
    }

    reducer.finish().unwrap();

    assert!(reducer.player_stats().get(&player_id).is_none());
    assert!(reducer.carry_events().is_empty());
}

#[test]
fn test_dodge_reset_reducer_tracks_on_ball_resets_separately() {
    let player_id = epic_id("flip-reset-player");
    let mut reducer = DodgeResetReducer::new();

    let mut sample = sample_stats(
        1,
        0.1,
        0.1,
        Some(sample_rigid_body(0.0, 0.0, 210.0)),
        vec![PlayerSample {
            rigid_body: Some(sample_rigid_body(0.0, 0.0, 300.0)),
            ..sample_player(player_id.clone(), true)
        }],
    );
    sample.dodge_refreshed_events.push(DodgeRefreshedEvent {
        time: sample.time,
        frame: sample.frame_number,
        player: player_id.clone(),
        is_team_0: true,
        counter_value: 1,
    });

    reducer.on_sample(&sample).unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.on_ball_count, 1);
}

#[test]
fn test_dodge_reset_reducer_keeps_nearby_non_ideal_geometry_as_on_ball() {
    let player_id = epic_id("nearby-reset-player");
    let mut reducer = DodgeResetReducer::new();

    let mut sample = sample_stats(
        1,
        0.1,
        0.1,
        Some(sample_rigid_body(140.0, 0.0, 330.0)),
        vec![PlayerSample {
            rigid_body: Some(sample_rigid_body(0.0, 0.0, 300.0)),
            ..sample_player(player_id.clone(), true)
        }],
    );
    sample.dodge_refreshed_events.push(DodgeRefreshedEvent {
        time: sample.time,
        frame: sample.frame_number,
        player: player_id.clone(),
        is_team_0: true,
        counter_value: 1,
    });

    reducer.on_sample(&sample).unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.on_ball_count, 1);
}

#[test]
fn test_dodge_reset_reducer_excludes_off_ball_resets_from_on_ball_count() {
    let player_id = epic_id("ceiling-reset-player");
    let mut reducer = DodgeResetReducer::new();

    let mut sample = sample_stats(
        1,
        0.1,
        0.1,
        Some(sample_rigid_body(0.0, 0.0, 650.0)),
        vec![PlayerSample {
            rigid_body: Some(sample_rigid_body(0.0, 0.0, 300.0)),
            ..sample_player(player_id.clone(), true)
        }],
    );
    sample.dodge_refreshed_events.push(DodgeRefreshedEvent {
        time: sample.time,
        frame: sample.frame_number,
        player: player_id.clone(),
        is_team_0: true,
        counter_value: 1,
    });

    reducer.on_sample(&sample).unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.on_ball_count, 0);
}

#[test]
fn test_ball_carry_reducer_flushes_active_carry_on_finish() {
    let player_id = epic_id("carry-finish");
    let mut manual = BallCarryReducer::new();
    let mut touch_sample = sample_stats(
        1,
        0.0,
        0.0,
        Some(sample_rigid_body(0.0, 0.0, 145.0)),
        vec![PlayerSample {
            rigid_body: Some(sample_rigid_body_with_linear_velocity(
                0.0, 0.0, 17.0, 450.0, 0.0, 0.0,
            )),
            ..sample_player(player_id.clone(), true)
        }],
    );
    touch_sample.touch_events.push(TouchEvent {
        time: 0.0,
        frame: 1,
        team_is_team_0: true,
        player: Some(player_id.clone()),
        closest_approach_distance: Some(30.0),
    });
    manual.on_sample(&touch_sample).unwrap();
    manual
        .on_sample(&sample_stats(
            2,
            0.5,
            0.5,
            Some(sample_rigid_body(60.0, 0.0, 150.0)),
            vec![PlayerSample {
                rigid_body: Some(sample_rigid_body_with_linear_velocity(
                    60.0, 0.0, 17.0, 450.0, 0.0, 0.0,
                )),
                ..sample_player(player_id.clone(), true)
            }],
        ))
        .unwrap();
    manual
        .on_sample(&sample_stats(
            3,
            1.0,
            0.5,
            Some(sample_rigid_body(120.0, 0.0, 150.0)),
            vec![PlayerSample {
                rigid_body: Some(sample_rigid_body_with_linear_velocity(
                    120.0, 0.0, 17.0, 450.0, 0.0, 0.0,
                )),
                ..sample_player(player_id.clone(), true)
            }],
        ))
        .unwrap();
    manual.finish().unwrap();

    let stats = manual.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.carry_count, 1);
    assert!((stats.total_carry_time - 1.0).abs() < 0.001);
    assert_eq!(manual.carry_events().len(), 1);
}

#[test]
fn test_powerslide_reducer_ignores_non_live_rising_edges() {
    let player_id = epic_id("powerslide-live-gating");
    let mut reducer = PowerslideReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![sample_player(player_id.clone(), true)],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                powerslide_active: true,
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(58),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                powerslide_active: true,
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.total_duration, 1.0);
    assert_eq!(stats.press_count, 0);
}

#[test]
fn test_powerslide_reducer_requires_ground_contact() {
    let player_id = epic_id("powerslide-ground-contact");
    let mut reducer = PowerslideReducer::new();
    let mut airborne_player = sample_player(player_id.clone(), true);
    airborne_player.powerslide_active = true;
    airborne_player.rigid_body = Some(sample_rigid_body(0.0, 0.0, 200.0));

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.5,
            seconds_remaining: Some(100),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![airborne_player],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .expect("Airborne handbrake sample should not fail");

    let mut grounded_player = sample_player(player_id.clone(), true);
    grounded_player.powerslide_active = true;
    grounded_player.rigid_body = Some(sample_rigid_body(0.0, 0.0, 17.0));

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.5,
            dt: 0.5,
            seconds_remaining: Some(99),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![grounded_player],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .expect("Grounded handbrake sample should not fail");

    let stats = reducer
        .player_stats()
        .get(&player_id)
        .expect("Player should have powerslide stats");
    assert_eq!(
        stats.press_count, 1,
        "Expected only grounded powerslide activation to count as a press"
    );
    assert_eq!(
        stats.total_duration, 0.5,
        "Expected only grounded powerslide time to be accumulated"
    );
}

#[test]
fn test_powerslide_reducer_allows_small_suspension_height() {
    let player_id = epic_id("powerslide-suspension-height");
    let mut reducer = PowerslideReducer::new();
    let mut player = sample_player(player_id.clone(), true);
    player.powerslide_active = true;
    player.rigid_body = Some(sample_rigid_body(0.0, 0.0, 30.0));

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.5,
            seconds_remaining: Some(100),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![player],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .expect("Low-hover handbrake sample should not fail");

    let stats = reducer
        .player_stats()
        .get(&player_id)
        .expect("Player should have powerslide stats");
    assert_eq!(stats.press_count, 1);
    assert_eq!(stats.total_duration, 0.5);
}

#[test]
fn test_pressure_reducer_tracks_ball_side_time() {
    let replay = parse_replay("assets/replays/rlcs.replay");
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
fn test_composite_stats_reducer_composes_children_under_frame_rate_decorator() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let replay_meta_calls_a = Rc::new(RefCell::new(0));
    let sample_calls_a = Rc::new(RefCell::new(0));
    let finish_calls_a = Rc::new(RefCell::new(0));
    let replay_meta_calls_b = Rc::new(RefCell::new(0));
    let sample_calls_b = Rc::new(RefCell::new(0));
    let finish_calls_b = Rc::new(RefCell::new(0));

    let composite = CompositeStatsReducer::new()
        .with_child(RecordingReducer::new(
            replay_meta_calls_a.clone(),
            sample_calls_a.clone(),
            finish_calls_a.clone(),
        ))
        .with_child(RecordingReducer::new(
            replay_meta_calls_b.clone(),
            sample_calls_b.clone(),
            finish_calls_b.clone(),
        ));
    let mut collector = ReducerCollector::new(composite);

    FrameRateDecorator::new_from_fps(10.0, &mut collector)
        .process_replay(&replay)
        .expect("Failed to process replay with composed reducers");

    let _ = collector.into_inner();

    assert_eq!(*replay_meta_calls_a.borrow(), 1);
    assert_eq!(*replay_meta_calls_b.borrow(), 1);
    assert!(*sample_calls_a.borrow() > 0);
    assert_eq!(*sample_calls_a.borrow(), *sample_calls_b.borrow());
    assert_eq!(*finish_calls_a.borrow(), 1);
    assert_eq!(*finish_calls_b.borrow(), 1);
}

#[test]
fn test_match_stats_reducer_builds_core_stats_and_timeline() {
    let replay = parse_replay("assets/replays/rlcs.replay");
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
fn test_match_stats_reducer_keeps_exact_timeline_under_sampling() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let full = ReducerCollector::new(MatchStatsReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with full match stats reducer")
        .into_inner();

    let mut sampled_collector = ReducerCollector::new(MatchStatsReducer::new());
    FrameRateDecorator::new_from_fps(1.0, &mut sampled_collector)
        .process_replay(&replay)
        .expect("Failed to process replay with sampled match stats reducer");
    let sampled = sampled_collector.into_inner();

    let assert_team_stats_match = |sampled: &CoreTeamStats,
                                   full: &CoreTeamStats,
                                   team_label: &str| {
        assert_eq!(
            sampled.score, full.score,
            "Expected {team_label} score totals to match under sampling"
        );
        assert_eq!(
            sampled.goals, full.goals,
            "Expected {team_label} goal totals to match under sampling"
        );
        assert_eq!(
            sampled.assists, full.assists,
            "Expected {team_label} assist totals to match under sampling"
        );
        assert_eq!(
            sampled.saves, full.saves,
            "Expected {team_label} save totals to match under sampling"
        );
        assert_eq!(
            sampled.shots, full.shots,
            "Expected {team_label} shot totals to match under sampling"
        );
        assert_eq!(
            sampled.goal_after_kickoff.kickoff_goal_count,
            full.goal_after_kickoff.kickoff_goal_count,
            "Expected {team_label} kickoff-goal bucket counts to match under sampling"
        );
        assert_eq!(
            sampled.goal_after_kickoff.short_goal_count, full.goal_after_kickoff.short_goal_count,
            "Expected {team_label} short-goal bucket counts to match under sampling"
        );
        assert_eq!(
            sampled.goal_after_kickoff.medium_goal_count, full.goal_after_kickoff.medium_goal_count,
            "Expected {team_label} medium-goal bucket counts to match under sampling"
        );
        assert_eq!(
            sampled.goal_after_kickoff.long_goal_count, full.goal_after_kickoff.long_goal_count,
            "Expected {team_label} long-goal bucket counts to match under sampling"
        );
        assert_eq!(
            sampled.goal_buildup, full.goal_buildup,
            "Expected {team_label} goal buildup classifications to match under sampling"
        );
    };

    assert_team_stats_match(
        &sampled.team_zero_stats(),
        &full.team_zero_stats(),
        "team zero",
    );
    assert_team_stats_match(
        &sampled.team_one_stats(),
        &full.team_one_stats(),
        "team one",
    );
    assert_eq!(
        sampled.timeline(),
        full.timeline(),
        "Expected buffered event delivery to preserve exact timeline events under sampling"
    );
}

#[test]
fn test_match_stats_reducer_prefers_exact_goal_event_times() {
    let player_id = epic_id("goal-scorer");
    let mut reducer = MatchStatsReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![sample_player(player_id.clone(), true)],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![sample_player(player_id.clone(), true)],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(1),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: Some(false),
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                match_goals: Some(1),
                match_shots: Some(1),
                match_score: Some(100),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: vec![GoalEvent {
                time: 1.25,
                frame: 2,
                scoring_team_is_team_0: true,
                player: Some(player_id.clone()),
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
fn test_match_stats_reducer_matches_goal_events_by_exact_scorer() {
    let scorer = epic_id("exact-scorer");
    let teammate = epic_id("teammate");
    let mut reducer = MatchStatsReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![
                sample_player(scorer.clone(), true),
                sample_player(teammate.clone(), true),
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![
                sample_player(scorer.clone(), true),
                sample_player(teammate.clone(), true),
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(1),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: Some(false),
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![
                PlayerSample {
                    match_goals: Some(1),
                    match_shots: Some(1),
                    ..sample_player(scorer.clone(), true)
                },
                sample_player(teammate.clone(), true),
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: vec![GoalEvent {
                time: 1.5,
                frame: 2,
                scoring_team_is_team_0: true,
                player: Some(scorer.clone()),
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
    assert_eq!(goal_event.player_id.as_ref(), Some(&scorer));
    assert!((goal_event.time - 1.5).abs() < 0.001);
}

#[test]
fn test_match_stats_reducer_tracks_goal_time_after_kickoff_buckets() {
    let scorer = epic_id("kickoff-timing-scorer");
    let mut reducer = MatchStatsReducer::new();

    let kickoff_sample =
        |frame_number: usize, time: f32, team_zero_score: i32, goals: i32| CoreSample {
            frame_number,
            time,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: Some(false),
            kickoff_countdown_time: Some(3),
            team_zero_score: Some(team_zero_score),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                match_goals: Some(goals),
                match_shots: Some(goals),
                ..sample_player(scorer.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        };

    let first_touch_sample =
        |frame_number: usize, time: f32, team_zero_score: i32, goals: i32, touch_time: f32| {
            CoreSample {
                frame_number,
                time,
                dt: 1.0,
                seconds_remaining: None,
                game_state: Some(0),
                ball_has_been_hit: Some(true),
                kickoff_countdown_time: Some(0),
                team_zero_score: Some(team_zero_score),
                team_one_score: Some(0),
                possession_team_is_team_0: None,
                scored_on_team_is_team_0: None,
                current_in_game_team_player_counts: None,
                ball: None,
                players: vec![PlayerSample {
                    match_goals: Some(goals),
                    match_shots: Some(goals),
                    ..sample_player(scorer.clone(), true)
                }],
                active_demos: Vec::new(),
                demo_events: Vec::new(),
                boost_pad_events: Vec::new(),
                touch_events: vec![TouchEvent {
                    time: touch_time,
                    frame: frame_number,
                    team_is_team_0: true,
                    player: Some(scorer.clone()),
                    closest_approach_distance: Some(0.0),
                }],
                dodge_refreshed_events: Vec::new(),
                player_stat_events: Vec::new(),
                goal_events: Vec::new(),
            }
        };

    let goal_sample =
        |frame_number: usize, time: f32, team_zero_score: i32, goals: i32, goal_time: f32| {
            CoreSample {
                frame_number,
                time,
                dt: 1.0,
                seconds_remaining: None,
                game_state: Some(0),
                ball_has_been_hit: Some(true),
                kickoff_countdown_time: Some(0),
                team_zero_score: Some(team_zero_score),
                team_one_score: Some(0),
                possession_team_is_team_0: None,
                scored_on_team_is_team_0: Some(false),
                current_in_game_team_player_counts: None,
                ball: None,
                players: vec![PlayerSample {
                    match_goals: Some(goals),
                    match_shots: Some(goals),
                    ..sample_player(scorer.clone(), true)
                }],
                active_demos: Vec::new(),
                demo_events: Vec::new(),
                boost_pad_events: Vec::new(),
                touch_events: Vec::new(),
                dodge_refreshed_events: Vec::new(),
                player_stat_events: Vec::new(),
                goal_events: vec![GoalEvent {
                    time: goal_time,
                    frame: frame_number,
                    scoring_team_is_team_0: true,
                    player: Some(scorer.clone()),
                    team_zero_score: Some(team_zero_score),
                    team_one_score: Some(0),
                }],
            }
        };

    for sample in [
        kickoff_sample(1, 0.0, 0, 0),
        first_touch_sample(2, 1.0, 0, 0, 1.0),
        goal_sample(3, 4.0, 1, 1, 4.0),
        kickoff_sample(4, 5.0, 1, 1),
        first_touch_sample(5, 6.0, 1, 1, 6.0),
        goal_sample(6, 21.0, 2, 2, 21.0),
        kickoff_sample(7, 22.0, 2, 2),
        first_touch_sample(8, 23.0, 2, 2, 23.0),
        goal_sample(9, 53.0, 3, 3, 53.0),
        kickoff_sample(10, 54.0, 3, 3),
        first_touch_sample(11, 55.0, 3, 3, 55.0),
        goal_sample(12, 105.0, 4, 4, 105.0),
    ] {
        reducer.on_sample(&sample).unwrap();
    }

    let player_stats = reducer
        .player_stats()
        .get(&scorer)
        .expect("Expected goal scorer stats to be present");
    assert!((player_stats.average_goal_time_after_kickoff() - 24.5).abs() < 0.001);
    assert!((player_stats.median_goal_time_after_kickoff() - 22.5).abs() < 0.001);
    assert_eq!(player_stats.goal_after_kickoff.kickoff_goal_count, 1);
    assert_eq!(player_stats.goal_after_kickoff.short_goal_count, 1);
    assert_eq!(player_stats.goal_after_kickoff.medium_goal_count, 1);
    assert_eq!(player_stats.goal_after_kickoff.long_goal_count, 1);

    let team_stats = reducer.team_zero_stats();
    assert!((team_stats.average_goal_time_after_kickoff() - 24.5).abs() < 0.001);
    assert!((team_stats.median_goal_time_after_kickoff() - 22.5).abs() < 0.001);
    assert_eq!(team_stats.goal_after_kickoff.kickoff_goal_count, 1);
    assert_eq!(team_stats.goal_after_kickoff.short_goal_count, 1);
    assert_eq!(team_stats.goal_after_kickoff.medium_goal_count, 1);
    assert_eq!(team_stats.goal_after_kickoff.long_goal_count, 1);
}

#[test]
fn test_match_stats_reducer_prefers_processor_stat_events_without_double_counting() {
    let player_id = epic_id("stat-event-player");
    let mut reducer = MatchStatsReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![sample_player(player_id.clone(), true)],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                match_shots: Some(1),
                match_saves: Some(1),
                match_assists: Some(1),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: vec![
                PlayerStatEvent {
                    time: 1.5,
                    frame: 2,
                    player: player_id.clone(),
                    is_team_0: true,
                    kind: PlayerStatEventKind::Shot,
                },
                PlayerStatEvent {
                    time: 1.6,
                    frame: 2,
                    player: player_id.clone(),
                    is_team_0: true,
                    kind: PlayerStatEventKind::Save,
                },
                PlayerStatEvent {
                    time: 1.7,
                    frame: 2,
                    player: player_id.clone(),
                    is_team_0: true,
                    kind: PlayerStatEventKind::Assist,
                },
            ],
            goal_events: Vec::new(),
        })
        .unwrap();

    let shot_events = reducer
        .timeline()
        .iter()
        .filter(|event| event.kind == TimelineEventKind::Shot)
        .count();
    let save_events = reducer
        .timeline()
        .iter()
        .filter(|event| event.kind == TimelineEventKind::Save)
        .count();
    let assist_events = reducer
        .timeline()
        .iter()
        .filter(|event| event.kind == TimelineEventKind::Assist)
        .count();

    assert_eq!(shot_events, 1);
    assert_eq!(save_events, 1);
    assert_eq!(assist_events, 1);
}

#[test]
fn test_movement_reducer_collects_distance_and_speed_buckets() {
    let replay = parse_replay("assets/replays/rlcs.replay");
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
fn test_movement_reducer_updates_position_baseline_through_non_live_time() {
    let player_id = epic_id("movement-live-gating");
    let mut reducer = MovementReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(100.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(58),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(110.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.tracked_time, 1.0);
    assert!((stats.total_distance - 10.0).abs() < 0.001);
}

#[test]
fn test_movement_reducer_tracks_pre_touch_kickoff_time() {
    let player_id = epic_id("movement-pre-touch-kickoff");
    let mut reducer = MovementReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: Some(58),
            ball_has_been_hit: Some(false),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(58),
            ball_has_been_hit: Some(false),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(1000.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.tracked_time, 1.0);
    assert_eq!(stats.total_distance, 1000.0);
}

#[test]
fn test_movement_reducer_uses_crossbar_plus_ball_radius_for_high_air_bucket() {
    let player_id = epic_id("movement-air-buckets");
    let mut reducer = MovementReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: Some(100),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 300.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: Some(99),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 800.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.time_low_air, 1.0);
    assert_eq!(stats.time_high_air, 1.0);
    assert_eq!(stats.time_on_ground, 0.0);
}

#[test]
fn test_positioning_reducer_collects_distances_and_percent_buckets() {
    let replay = parse_replay("assets/replays/rlcs.replay");
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
fn test_positioning_reducer_uses_field_marking_zone_boundary() {
    let player_id = epic_id("positioning-zone-boundary");
    let mut reducer = PositioningReducer::new();
    let ball = BallSample {
        rigid_body: sample_rigid_body(0.0, 0.0, 0.0),
    };

    for (frame_number, dt, y) in [
        (1, 0.0, 2000.0),
        (2, 1.0, 2000.0),
        (3, 0.0, 3000.0),
        (4, 1.0, 3000.0),
        (5, 0.0, -3000.0),
        (6, 1.0, -3000.0),
    ] {
        reducer
            .on_sample(&CoreSample {
                frame_number,
                time: frame_number as f32,
                dt,
                seconds_remaining: Some(100 - frame_number as i32),
                game_state: Some(0),
                ball_has_been_hit: None,
                kickoff_countdown_time: None,
                team_zero_score: None,
                team_one_score: None,
                possession_team_is_team_0: None,
                scored_on_team_is_team_0: None,
                current_in_game_team_player_counts: None,
                ball: Some(ball.clone()),
                players: vec![PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, y, 17.0)),
                    ..sample_player(player_id.clone(), true)
                }],
                active_demos: Vec::new(),
                demo_events: Vec::new(),
                boost_pad_events: Vec::new(),
                touch_events: Vec::new(),
                dodge_refreshed_events: Vec::new(),
                player_stat_events: Vec::new(),
                goal_events: Vec::new(),
            })
            .unwrap();
    }

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.time_neutral_zone, 1.0);
    assert_eq!(stats.time_offensive_zone, 1.0);
    assert_eq!(stats.time_defensive_zone, 1.0);
}

#[test]
fn test_positioning_reducer_interpolates_zone_half_and_ball_boundaries() {
    let player_id = epic_id("positioning-boundary-interpolation");
    let mut reducer = PositioningReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: Some(99),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(BallSample {
                rigid_body: sample_rigid_body(0.0, 0.0, 92.75),
            }),
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 2000.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: Some(98),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(BallSample {
                rigid_body: sample_rigid_body(0.0, 0.0, 92.75),
            }),
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 2600.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 0.0,
            seconds_remaining: Some(97),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(BallSample {
                rigid_body: sample_rigid_body(0.0, 0.0, 92.75),
            }),
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, -100.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 4,
            time: 4.0,
            dt: 1.0,
            seconds_remaining: Some(96),
            game_state: Some(0),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(BallSample {
                rigid_body: sample_rigid_body(0.0, 100.0, 92.75),
            }),
            players: vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 100.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert!((stats.time_neutral_zone - 1.5).abs() < 0.001);
    assert!((stats.time_offensive_zone - 0.5).abs() < 0.001);
    assert!((stats.time_defensive_half - 0.5).abs() < 0.001);
    assert!((stats.time_offensive_half - 1.5).abs() < 0.001);
    assert!((stats.time_behind_ball - 1.0).abs() < 0.001);
    assert!((stats.time_in_front_of_ball - 1.0).abs() < 0.001);
}

#[test]
fn test_positioning_reducer_requires_full_live_multi_player_team_for_role_buckets() {
    let front_id = epic_id("positioning-front");
    let back_id = epic_id("positioning-back");
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            1.0,
            0.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                    ..sample_player(front_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -1000.0, 17.0)),
                    ..sample_player(back_id.clone(), true)
                },
            ],
        ))
        .unwrap();

    reducer
        .on_sample(&sample_stats(
            2,
            2.0,
            1.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                    ..sample_player(front_id.clone(), true)
                },
                sample_player(back_id.clone(), true),
            ],
        ))
        .unwrap();

    let front_stats = reducer.player_stats().get(&front_id).unwrap();
    assert_eq!(front_stats.active_game_time, 1.0);
    assert_eq!(front_stats.time_most_forward, 0.0);
    assert_eq!(front_stats.time_most_back, 0.0);
    assert_eq!(front_stats.time_mid_role, 0.0);
    assert_eq!(front_stats.time_other_role, 0.0);
    assert_eq!(front_stats.time_no_teammates, 1.0);
    assert!(reducer.player_stats().get(&back_id).is_none());
}

#[test]
fn test_positioning_reducer_uses_current_in_game_roster_for_role_bucket_gating() {
    let front_id = epic_id("positioning-front-meta");
    let middle_id = epic_id("positioning-middle-meta");
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            1.0,
            0.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                    ..sample_player(front_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                    ..sample_player(middle_id.clone(), true)
                },
            ],
        ))
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: Some(100),
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([3, 0]),
            ball: ball.map(|rigid_body| BallSample { rigid_body }),
            players: vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                    ..sample_player(front_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                    ..sample_player(middle_id.clone(), true)
                },
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let front_stats = reducer.player_stats().get(&front_id).unwrap();
    let middle_stats = reducer.player_stats().get(&middle_id).unwrap();
    assert_eq!(front_stats.time_most_forward, 0.0);
    assert_eq!(front_stats.time_most_back, 0.0);
    assert_eq!(front_stats.time_mid_role, 0.0);
    assert_eq!(front_stats.time_other_role, 0.0);
    assert_eq!(middle_stats.time_most_forward, 0.0);
    assert_eq!(middle_stats.time_most_back, 0.0);
    assert_eq!(middle_stats.time_mid_role, 0.0);
    assert_eq!(middle_stats.time_other_role, 0.0);
}

#[test]
fn test_positioning_reducer_tracks_pre_touch_kickoff_time() {
    let player_id = epic_id("positioning-pre-touch-kickoff");
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            0.0,
            0.0,
            ball,
            vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 1000.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
        ))
        .unwrap();

    let mut pre_touch_sample = sample_stats(
        2,
        1.0,
        1.0,
        ball,
        vec![PlayerSample {
            rigid_body: Some(sample_rigid_body(0.0, 2000.0, 17.0)),
            ..sample_player(player_id.clone(), true)
        }],
    );
    pre_touch_sample.game_state = Some(58);
    pre_touch_sample.ball_has_been_hit = Some(false);
    reducer.on_sample(&pre_touch_sample).unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.active_game_time, 1.0);
    assert_eq!(stats.tracked_time, 1.0);
}

#[test]
fn test_positioning_reducer_ignores_frozen_kickoff_countdown_time() {
    let player_id = epic_id("positioning-kickoff-countdown");
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            0.0,
            0.0,
            ball,
            vec![PlayerSample {
                rigid_body: Some(sample_rigid_body(0.0, 1000.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
        ))
        .unwrap();

    let mut countdown_sample = sample_stats(
        2,
        1.0,
        1.0,
        ball,
        vec![PlayerSample {
            rigid_body: Some(sample_rigid_body(0.0, 2000.0, 17.0)),
            ..sample_player(player_id.clone(), true)
        }],
    );
    countdown_sample.game_state = Some(55);
    countdown_sample.kickoff_countdown_time = Some(3);
    countdown_sample.ball_has_been_hit = Some(false);
    reducer.on_sample(&countdown_sample).unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.active_game_time, 0.0);
    assert_eq!(stats.tracked_time, 0.0);
}

#[test]
fn test_positioning_reducer_allows_role_buckets_after_player_leaves_match() {
    let front_id = epic_id("positioning-front-leave");
    let back_id = epic_id("positioning-back-leave");
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            1.0,
            0.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 1200.0, 17.0)),
                    ..sample_player(front_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -1200.0, 17.0)),
                    ..sample_player(back_id.clone(), true)
                },
            ],
        ))
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: Some(100),
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([2, 0]),
            ball: ball.map(|rigid_body| BallSample { rigid_body }),
            players: vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 1200.0, 17.0)),
                    ..sample_player(front_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -1200.0, 17.0)),
                    ..sample_player(back_id.clone(), true)
                },
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let front_stats = reducer.player_stats().get(&front_id).unwrap();
    let back_stats = reducer.player_stats().get(&back_id).unwrap();
    assert_eq!(front_stats.time_most_forward, 1.0);
    assert_eq!(front_stats.time_most_back, 0.0);
    assert_eq!(back_stats.time_most_back, 1.0);
    assert_eq!(back_stats.time_most_forward, 0.0);
}

#[test]
fn test_positioning_reducer_records_mid_role_for_unclassified_middle_player() {
    let front_id = epic_id("positioning-front-middle");
    let middle_id = epic_id("positioning-middle-middle");
    let back_id = epic_id("positioning-back-middle");
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            1.0,
            0.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 1200.0, 17.0)),
                    ..sample_player(front_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                    ..sample_player(middle_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -1200.0, 17.0)),
                    ..sample_player(back_id.clone(), true)
                },
            ],
        ))
        .unwrap();

    reducer
        .on_sample(&sample_stats(
            2,
            2.0,
            1.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 1200.0, 17.0)),
                    ..sample_player(front_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                    ..sample_player(middle_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -1200.0, 17.0)),
                    ..sample_player(back_id.clone(), true)
                },
            ],
        ))
        .unwrap();

    let middle_stats = reducer.player_stats().get(&middle_id).unwrap();
    assert_eq!(middle_stats.active_game_time, 1.0);
    assert_eq!(middle_stats.time_mid_role, 1.0);
    assert_eq!(middle_stats.time_most_back, 0.0);
    assert_eq!(middle_stats.time_most_forward, 0.0);
    assert_eq!(middle_stats.time_other_role, 0.0);
}

#[test]
fn test_positioning_reducer_other_role_requires_full_team_clustered_within_threshold() {
    let player_ids = [
        epic_id("positioning-even-1"),
        epic_id("positioning-even-2"),
        epic_id("positioning-even-3"),
    ];
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            1.0,
            0.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -40.0, 17.0)),
                    ..sample_player(player_ids[0].clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                    ..sample_player(player_ids[1].clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 40.0, 17.0)),
                    ..sample_player(player_ids[2].clone(), true)
                },
            ],
        ))
        .unwrap();

    reducer
        .on_sample(&sample_stats(
            2,
            2.0,
            1.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -40.0, 17.0)),
                    ..sample_player(player_ids[0].clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                    ..sample_player(player_ids[1].clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 40.0, 17.0)),
                    ..sample_player(player_ids[2].clone(), true)
                },
            ],
        ))
        .unwrap();

    for player_id in player_ids {
        let stats = reducer.player_stats().get(&player_id).unwrap();
        assert_eq!(stats.time_other_role, 1.0);
        assert_eq!(stats.time_mid_role, 0.0);
        assert_eq!(stats.time_most_back, 0.0);
        assert_eq!(stats.time_most_forward, 0.0);
    }
}

#[test]
fn test_positioning_reducer_tracks_demo_and_no_teammate_role_gaps() {
    let live_id = epic_id("positioning-live-demo");
    let victim_id = epic_id("positioning-victim-demo");
    let opp_a = epic_id("positioning-opp-a");
    let opp_b = epic_id("positioning-opp-b");
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            1.0,
            0.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 1000.0, 17.0)),
                    ..sample_player(live_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -1000.0, 17.0)),
                    ..sample_player(victim_id.clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(1000.0, 1000.0, 17.0)),
                    ..sample_player(opp_a.clone(), false)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(1000.0, -1000.0, 17.0)),
                    ..sample_player(opp_b.clone(), false)
                },
            ],
        ))
        .unwrap();

    let mut victim_sample = sample_player(victim_id.clone(), true);
    victim_sample.rigid_body = None;

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: Some(100),
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([2, 2]),
            ball: ball.map(|rigid_body| BallSample { rigid_body }),
            players: vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 1000.0, 17.0)),
                    ..sample_player(live_id.clone(), true)
                },
                victim_sample,
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(1000.0, 1000.0, 17.0)),
                    ..sample_player(opp_a.clone(), false)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(1000.0, -1000.0, 17.0)),
                    ..sample_player(opp_b.clone(), false)
                },
            ],
            active_demos: vec![DemoEventSample {
                attacker: opp_a,
                victim: victim_id.clone(),
            }],
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let live_stats = reducer.player_stats().get(&live_id).unwrap();
    assert_eq!(live_stats.active_game_time, 1.0);
    assert_eq!(live_stats.tracked_time, 1.0);
    assert_eq!(live_stats.time_no_teammates, 1.0);
    assert_eq!(live_stats.time_demolished, 0.0);
    assert_eq!(
        live_stats.time_most_back
            + live_stats.time_most_forward
            + live_stats.time_mid_role
            + live_stats.time_other_role
            + live_stats.time_no_teammates
            + live_stats.time_demolished,
        live_stats.active_game_time
    );

    let victim_stats = reducer.player_stats().get(&victim_id).unwrap();
    assert_eq!(victim_stats.active_game_time, 1.0);
    assert_eq!(victim_stats.tracked_time, 0.0);
    assert_eq!(victim_stats.time_demolished, 1.0);
    assert_eq!(victim_stats.time_no_teammates, 0.0);
    assert_eq!(
        victim_stats.time_most_back
            + victim_stats.time_most_forward
            + victim_stats.time_mid_role
            + victim_stats.time_other_role
            + victim_stats.time_no_teammates
            + victim_stats.time_demolished,
        victim_stats.active_game_time
    );
}

#[test]
fn test_positioning_reducer_treats_single_teammate_demo_in_3v3_like_2v2() {
    #[derive(Default)]
    struct SingleTeammateDemoTracker {
        impacted_players_by_frame: HashMap<usize, Vec<PlayerId>>,
    }

    impl Collector for SingleTeammateDemoTracker {
        fn process_frame(
            &mut self,
            processor: &ReplayProcessor,
            _frame: &boxcars::Frame,
            frame_number: usize,
            current_time: f32,
        ) -> SubtrActorResult<TimeAdvance> {
            const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
            const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;

            let live_play = !matches!(
                processor.get_replicated_state_name().ok(),
                Some(GAME_STATE_KICKOFF_COUNTDOWN | GAME_STATE_GOAL_SCORED_REPLAY)
            ) && !matches!(processor.get_ball_has_been_hit().ok(), Some(false));
            if !live_play {
                return Ok(TimeAdvance::NextFrame);
            }

            let demoed_players: HashSet<_> = processor
                .get_active_demos()?
                .filter_map(|demo| {
                    processor
                        .get_player_id_from_car_id(&demo.victim_actor_id())
                        .ok()
                })
                .collect();
            if demoed_players.is_empty() {
                return Ok(TimeAdvance::NextFrame);
            }

            let roster_counts = processor.current_in_game_team_player_counts();
            let mut impacted_players = Vec::new();

            for is_team_0 in [true, false] {
                let team_roster_count = if is_team_0 {
                    roster_counts[0]
                } else {
                    roster_counts[1]
                };
                if team_roster_count != 3 {
                    continue;
                }

                let team_demo_count = demoed_players
                    .iter()
                    .filter(|player_id| {
                        processor.get_player_is_team_0(player_id).ok() == Some(is_team_0)
                    })
                    .count();
                if team_demo_count != 1 {
                    continue;
                }

                let live_teammates: Vec<_> = processor
                    .iter_player_ids_in_order()
                    .filter(|player_id| {
                        processor.get_player_is_team_0(player_id).ok() == Some(is_team_0)
                    })
                    .filter(|player_id| !demoed_players.contains(*player_id))
                    .filter(|player_id| {
                        processor
                            .get_interpolated_player_rigid_body(player_id, current_time, 0.0)
                            .ok()
                            .is_some_and(|rigid_body| !rigid_body.sleeping)
                    })
                    .cloned()
                    .collect();

                if live_teammates.len() == 2 {
                    impacted_players.extend(live_teammates);
                }
            }

            if !impacted_players.is_empty() {
                self.impacted_players_by_frame
                    .insert(frame_number, impacted_players);
            }

            Ok(TimeAdvance::NextFrame)
        }
    }

    fn find_player<'a>(
        frame: &'a ReplayStatsFrame,
        player_id: &PlayerId,
    ) -> &'a PlayerStatsSnapshot {
        frame
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .unwrap_or_else(|| panic!("Missing player snapshot for {player_id:?}"))
    }

    let replay = parse_replay("assets/replays/new_demolition_format.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to build stats timeline for positioning demo regression");

    let mut tracker = SingleTeammateDemoTracker::default();
    ReplayProcessor::new(&replay)
        .expect("Failed to construct replay processor")
        .process(&mut tracker)
        .expect("Failed to scan replay for single-teammate demo frames");

    let mut checked_players = 0usize;
    for window in timeline.frames.windows(2) {
        let previous = &window[0];
        let current = &window[1];
        let Some(player_ids) = tracker.impacted_players_by_frame.get(&current.frame_number) else {
            continue;
        };
        if !current.is_live_play {
            continue;
        }

        for player_id in player_ids {
            let previous_player = find_player(previous, player_id);
            let current_player = find_player(current, player_id);

            let no_teammates_delta = current_player.positioning.time_no_teammates
                - previous_player.positioning.time_no_teammates;
            let role_time_delta = (current_player.positioning.time_most_back
                - previous_player.positioning.time_most_back)
                + (current_player.positioning.time_most_forward
                    - previous_player.positioning.time_most_forward)
                + (current_player.positioning.time_mid_role
                    - previous_player.positioning.time_mid_role)
                + (current_player.positioning.time_other_role
                    - previous_player.positioning.time_other_role);

            assert!(
                no_teammates_delta.abs() < 1e-4,
                "Player {} accumulated {:.4}s of no-teammates time at frame {} (t={:.3}) with exactly one teammate demoed in 3v3",
                current_player.name,
                no_teammates_delta,
                current.frame_number,
                current.time
            );
            assert!(
                (role_time_delta - current.dt).abs() < 1e-4,
                "Player {} failed to accumulate 2v2 role time at frame {} (t={:.3}) with one teammate demoed: role_delta={:.4}, dt={:.4}",
                current_player.name,
                current.frame_number,
                current.time,
                role_time_delta,
                current.dt
            );
            checked_players += 1;
        }
    }

    assert!(
        checked_players > 0,
        "Expected new_demolition_format.replay to contain at least one live 3v3 frame with exactly one demoed teammate"
    );
}

#[test]
fn test_positioning_reducer_other_role_uses_two_car_length_default_threshold() {
    let player_ids = [
        epic_id("positioning-even-two-cars-1"),
        epic_id("positioning-even-two-cars-2"),
        epic_id("positioning-even-two-cars-3"),
    ];
    let mut reducer = PositioningReducer::new();
    let ball = Some(sample_rigid_body(0.0, 0.0, 92.75));

    reducer
        .on_sample(&sample_stats(
            1,
            1.0,
            0.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -100.0, 17.0)),
                    ..sample_player(player_ids[0].clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                    ..sample_player(player_ids[1].clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 100.0, 17.0)),
                    ..sample_player(player_ids[2].clone(), true)
                },
            ],
        ))
        .unwrap();

    reducer
        .on_sample(&sample_stats(
            2,
            2.0,
            1.0,
            ball,
            vec![
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, -100.0, 17.0)),
                    ..sample_player(player_ids[0].clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                    ..sample_player(player_ids[1].clone(), true)
                },
                PlayerSample {
                    rigid_body: Some(sample_rigid_body(0.0, 100.0, 17.0)),
                    ..sample_player(player_ids[2].clone(), true)
                },
            ],
        ))
        .unwrap();

    for player_id in player_ids {
        let stats = reducer.player_stats().get(&player_id).unwrap();
        assert_eq!(stats.time_other_role, 1.0);
        assert_eq!(stats.time_mid_role, 0.0);
        assert_eq!(stats.time_most_back, 0.0);
        assert_eq!(stats.time_most_forward, 0.0);
    }
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
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(ball.clone()),
            players: vec![team_zero_player.clone(), team_one_player.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 1.0,
                frame: 1,
                team_is_team_0: true,
                player: Some(team_zero_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(ball.clone()),
            players: vec![team_zero_player.clone(), team_one_player.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(ball.clone()),
            players: vec![team_zero_player.clone(), team_one_player.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 3.0,
                frame: 3,
                team_is_team_0: false,
                player: Some(team_one_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 4,
            time: 4.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(ball.clone()),
            players: vec![team_zero_player.clone(), team_one_player.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 4.0,
                frame: 4,
                team_is_team_0: false,
                player: Some(team_one_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 5,
            time: 5.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: Some(ball),
            players: vec![team_zero_player, team_one_player],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
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
fn test_boost_reducer_ignores_non_live_time_for_average_amount() {
    let player_id = epic_id("boost-live-gating");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(100.0),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(58),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.tracked_time, 1.0);
    assert_eq!(stats.average_boost_amount(), 200.0);
}

#[test]
fn test_boost_reducer_uses_actual_depletion_for_supersonic_usage() {
    let player_id = epic_id("boost-supersonic-usage");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body_with_linear_velocity(
                    0.0, 0.0, 17.0, 2300.0, 0.0, 0.0,
                )),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(190.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body_with_linear_velocity(
                    0.0, 0.0, 17.0, 2300.0, 0.0, 0.0,
                )),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.amount_used_while_supersonic, 10.0);
}

#[test]
fn test_boost_reducer_splits_observed_usage_between_ground_and_air() {
    let player_id = epic_id("boost-ground-air-usage");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(BOOST_KICKOFF_START_AMOUNT),
                boost_active: true,
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(45.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(20.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 400.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.amount_used, BOOST_KICKOFF_START_AMOUNT - 20.0);
    assert_eq!(stats.amount_used_while_grounded, 40.0);
    assert_eq!(stats.amount_used_while_airborne, 25.0);
    assert_eq!(
        stats.amount_used_while_grounded + stats.amount_used_while_airborne,
        stats.amount_used
    );
}

#[test]
fn test_boost_reducer_does_not_infer_supersonic_usage_without_depletion() {
    let player_id = epic_id("boost-supersonic-no-usage");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body_with_linear_velocity(
                    0.0, 0.0, 17.0, 2300.0, 0.0, 0.0,
                )),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body_with_linear_velocity(
                    0.0, 0.0, 17.0, 2300.0, 0.0, 0.0,
                )),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.amount_used_while_supersonic, 0.0);
}

#[test]
fn test_boost_reducer_requires_supersonic_speed_across_interval() {
    let player_id = epic_id("boost-supersonic-transition");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body_with_linear_velocity(
                    0.0, 0.0, 17.0, 2100.0, 0.0, 0.0,
                )),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(190.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body_with_linear_velocity(
                    0.0, 0.0, 17.0, 2300.0, 0.0, 0.0,
                )),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.amount_used_while_supersonic, 0.0);
}

#[test]
fn test_boost_reducer_interpolates_average_and_bucket_times() {
    let player_id = epic_id("boost-interpolation");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(0.0),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert!((stats.average_boost_amount() - 127.5).abs() < 1e-4);
    assert!((stats.time_zero_boost - (1.0 / 255.0)).abs() < 1e-4);
    assert!((stats.time_hundred_boost - (1.0 / 255.0)).abs() < 1e-4);
    assert!((stats.time_boost_0_25 - 0.25).abs() < 1e-4);
    assert!((stats.time_boost_25_50 - 0.25).abs() < 1e-4);
    assert!((stats.time_boost_50_75 - 0.25).abs() < 1e-4);
    assert!((stats.time_boost_75_100 - 0.25).abs() < 1e-4);
}

#[test]
fn test_boost_reducer_uses_exact_pad_events_for_size_and_overfill() {
    let player_id = epic_id("boost-player");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 3,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
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
fn test_boost_reducer_prefers_frame_last_boost_amount_for_pickups() {
    let player_id = epic_id("boost-frame-last-amount");
    let pad_id = "VehiclePickup_Boost_TA_63".to_string();
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                last_boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 2,
                pad_id: pad_id.clone(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 3,
                pad_id,
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 1);
    assert!((stats.amount_collected_big - 55.0).abs() < 0.001);
    assert!((stats.overfill_total - 200.0).abs() < 0.001);
}

#[test]
fn test_boost_reducer_ignores_non_live_pickups_by_default() {
    let player_id = epic_id("boost-non-live-pickup");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 0.0,
                frame: 1,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 10.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 10.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer
        .player_stats()
        .get(&player_id)
        .cloned()
        .unwrap_or_default();
    assert_eq!(stats.big_pads_collected, 0);
    assert_eq!(stats.small_pads_collected, 0);
    assert_eq!(stats.amount_collected, 0.0);
}

#[test]
fn test_boost_reducer_tracks_pre_touch_kickoff_time_and_pickups() {
    let player_id = epic_id("boost-pre-touch-kickoff");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 1.0,
            seconds_remaining: Some(300),
            game_state: Some(58),
            ball_has_been_hit: Some(false),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                boost_active: true,
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 0.0,
                frame: 1,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 10.0,
            dt: 10.0,
            seconds_remaining: Some(290),
            game_state: Some(58),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 10.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer
        .player_stats()
        .get(&player_id)
        .cloned()
        .unwrap_or_default();
    assert!((stats.amount_collected - 55.0).abs() < 0.001);
    assert_eq!(stats.tracked_time, 11.0);
}

#[test]
fn test_boost_reducer_dedupes_same_frame_pickup_payloads() {
    let player_id = epic_id("boost-duplicate-pickup-payload");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![
                BoostPadEvent {
                    time: 1.0,
                    frame: 1,
                    pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 1 },
                },
                BoostPadEvent {
                    time: 1.0,
                    frame: 1,
                    pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 2 },
                },
            ],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 1);
    assert_eq!(stats.amount_collected, 55.0);
}

#[test]
fn test_boost_reducer_requires_respawn_before_recounting_known_pad_pickups() {
    let player_id = epic_id("boost-known-pad-respawn");
    let pad_id = "VehiclePickup_Boost_TA_63".to_string();
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 1,
                pad_id: pad_id.clone(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 2,
                pad_id: pad_id.clone(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 12.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![
                BoostPadEvent {
                    time: 12.0,
                    frame: 3,
                    pad_id: pad_id.clone(),
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 2 },
                },
                BoostPadEvent {
                    time: 12.0,
                    frame: 3,
                    pad_id: pad_id.clone(),
                    player: Some(player_id.clone()),
                    kind: BoostPadEventKind::PickedUp { sequence: 3 },
                },
            ],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 2);
    assert_eq!(stats.amount_collected, 110.0);

    reducer
        .on_sample(&CoreSample {
            frame_number: 4,
            time: 22.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 22.0,
                frame: 4,
                pad_id,
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 2);
    assert_eq!(stats.amount_collected, 110.0);
}

#[test]
fn test_boost_reducer_ignores_early_available_for_known_small_pad() {
    let player_id = epic_id("boost-known-small-pad-early-available");
    let pad_id = "VehiclePickup_Boost_TA_20".to_string();
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(243.0),
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 1,
                pad_id: pad_id.clone(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 6.0,
            dt: 5.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 6.0,
                frame: 2,
                pad_id: pad_id.clone(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 7.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(243.0),
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 7.0,
                frame: 3,
                pad_id: pad_id.clone(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 2 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.small_pads_collected, 2);

    reducer
        .on_sample(&CoreSample {
            frame_number: 4,
            time: 10.0,
            dt: 3.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 10.0,
                frame: 4,
                pad_id: pad_id.clone(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 5,
            time: 10.5,
            dt: 0.5,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(243.0),
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 10.5,
                frame: 5,
                pad_id: pad_id.clone(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 3 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.small_pads_collected, 2);

    reducer
        .on_sample(&CoreSample {
            frame_number: 6,
            time: 11.0,
            dt: 0.5,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 6,
                pad_id: pad_id.clone(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 7,
            time: 11.5,
            dt: 0.5,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(243.0),
                rigid_body: Some(sample_rigid_body(0.0, 0.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.5,
                frame: 7,
                pad_id,
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 4 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.small_pads_collected, 3);
}

#[test]
fn test_boost_reducer_can_include_non_live_pickups_when_enabled() {
    let player_id = epic_id("boost-non-live-pickup-opt-in");
    let mut reducer = BoostReducer::with_config(BoostReducerConfig {
        include_non_live_pickups: true,
    });

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 0.0,
                frame: 1,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 10.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                rigid_body: Some(sample_rigid_body(3584.0, 0.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 10.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_63".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 1);
    assert!((stats.amount_collected_big - 55.0).abs() < 0.001);
}

#[test]
fn test_boost_reducer_uses_canonical_pad_layout_for_stolen_classification() {
    let player_id = epic_id("midline-player");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
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
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
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
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_centerline_big".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
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
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 3,
                pad_id: "VehiclePickup_Boost_TA_centerline_big".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.big_pads_collected, 1);
    assert_eq!(stats.amount_stolen, 0.0);
    assert_eq!(stats.big_pads_stolen, 0);
}

#[test]
fn test_boost_reducer_overrides_small_position_guess_when_gain_exceeds_small_pad() {
    // Player is positioned near a small pad (0, 1024) but gains more boost than
    // a small pad can provide. The sanity check should override to big.
    let player_id = epic_id("gain-override");
    let small_pad_position = sample_rigid_body(0.0, 1024.0, 70.0);
    let mut reducer = BoostReducer::new();

    // Frame 1: establish previous boost amount
    reducer
        .on_sample(&sample_stats(
            1,
            0.0,
            0.0,
            None,
            vec![PlayerSample {
                boost_amount: Some(0.0),
                rigid_body: Some(small_pad_position),
                ..sample_player(player_id.clone(), true)
            }],
        ))
        .unwrap();

    // Frame 2: pickup event with large boost gain (0 → 255, gain = 255)
    let mut frame2 = sample_stats(
        2,
        1.0,
        1.0,
        None,
        vec![PlayerSample {
            boost_amount: Some(255.0),
            rigid_body: Some(small_pad_position),
            ..sample_player(player_id.clone(), true)
        }],
    );
    frame2.boost_pad_events = vec![BoostPadEvent {
        time: 1.0,
        frame: 2,
        pad_id: "VehiclePickup_Boost_TA_near_small".to_string(),
        player: Some(player_id.clone()),
        kind: BoostPadEventKind::PickedUp { sequence: 1 },
    }];
    reducer.on_sample(&frame2).unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    // Gain of 255 exceeds SMALL_PAD_AMOUNT_RAW (~30.6), so must be big
    assert_eq!(
        stats.big_pads_collected, 1,
        "Expected big pad override from gain sanity check"
    );
    assert_eq!(stats.small_pads_collected, 0);
}

#[test]
fn test_boost_reducer_keeps_small_when_gain_is_consistent() {
    // Player is near a small pad and gains a small-pad-consistent amount.
    // Position inference should be trusted.
    let player_id = epic_id("small-consistent");
    let small_pad_position = sample_rigid_body(0.0, 1024.0, 70.0);
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&sample_stats(
            1,
            0.0,
            0.0,
            None,
            vec![PlayerSample {
                boost_amount: Some(100.0),
                rigid_body: Some(small_pad_position),
                ..sample_player(player_id.clone(), true)
            }],
        ))
        .unwrap();

    // Gain of 30.6 (100 → 130.6) is consistent with a small pad
    let mut frame2 = sample_stats(
        2,
        1.0,
        1.0,
        None,
        vec![PlayerSample {
            boost_amount: Some(130.6),
            rigid_body: Some(small_pad_position),
            ..sample_player(player_id.clone(), true)
        }],
    );
    frame2.boost_pad_events = vec![BoostPadEvent {
        time: 1.0,
        frame: 2,
        pad_id: "VehiclePickup_Boost_TA_small_ok".to_string(),
        player: Some(player_id.clone()),
        kind: BoostPadEventKind::PickedUp { sequence: 1 },
    }];
    reducer.on_sample(&frame2).unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    eprintln!(
        "small_pads={} big_pads={} collected={:.1} collected_small={:.1} collected_big={:.1}",
        stats.small_pads_collected,
        stats.big_pads_collected,
        stats.amount_collected,
        stats.amount_collected_small,
        stats.amount_collected_big
    );
    assert_eq!(
        stats.small_pads_collected, 1,
        "Expected small pad when gain is consistent"
    );
    assert_eq!(stats.big_pads_collected, 0);
}

#[test]
fn test_boost_reducer_trusts_big_position_with_small_gain_near_full_boost() {
    // Player is near a big pad but gains only a small amount because they were
    // nearly full. The position-based guess (big) should be trusted since the
    // gain is ambiguous — a big pad with overfill looks like a small gain.
    let player_id = epic_id("big-small-gain");
    let big_pad_position = sample_rigid_body(3584.0, 0.0, 73.0);
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&sample_stats(
            1,
            0.0,
            0.0,
            None,
            vec![PlayerSample {
                boost_amount: Some(240.0),
                rigid_body: Some(big_pad_position),
                ..sample_player(player_id.clone(), true)
            }],
        ))
        .unwrap();

    // Gain of 15 (240 → 255), less than small pad amount, but position says big
    let mut frame2 = sample_stats(
        2,
        1.0,
        1.0,
        None,
        vec![PlayerSample {
            boost_amount: Some(255.0),
            rigid_body: Some(big_pad_position),
            ..sample_player(player_id.clone(), true)
        }],
    );
    frame2.boost_pad_events = vec![BoostPadEvent {
        time: 1.0,
        frame: 2,
        pad_id: "VehiclePickup_Boost_TA_big_overfill".to_string(),
        player: Some(player_id.clone()),
        kind: BoostPadEventKind::PickedUp { sequence: 1 },
    }];
    reducer.on_sample(&frame2).unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(
        stats.big_pads_collected, 1,
        "Expected big pad from position even with small gain"
    );
    assert_eq!(stats.small_pads_collected, 0);
    // Overfill: big pad gives 255, but only 15 could be collected (255 - 240)
    assert!((stats.overfill_total - 240.0).abs() < 0.01);
}

#[test]
fn test_boost_reducer_aligns_canonical_pickups_with_boost_jump_frame() {
    let player_id = epic_id("canonical-pickup-alignment-player");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 50.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(255.0),
                last_boost_amount: Some(200.0),
                rigid_body: Some(sample_rigid_body(3584.0, 50.0, 73.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_centerline_big".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert!((stats.amount_collected - 55.0).abs() < 0.001);
    assert!((stats.amount_collected_big - 55.0).abs() < 0.001);
    assert!((stats.amount_collected_small - 0.0).abs() < 0.001);
    assert!(
        (stats.amount_collected - (stats.amount_collected_big + stats.amount_collected_small))
            .abs()
            < 0.001
    );
}

#[test]
fn test_boost_reducer_tracks_respawn_grants_and_used_amount() {
    let player_id = epic_id("boost-respawn-accounting");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: Some(55),
            ball_has_been_hit: Some(false),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(BOOST_KICKOFF_START_AMOUNT),
                rigid_body: Some(sample_rigid_body(0.0, -1000.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(60.0),
                rigid_body: Some(sample_rigid_body(0.0, -900.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(60.0),
                rigid_body: None,
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: vec![DemolishInfo {
                time: 2.0,
                seconds_remaining: 100,
                frame: 3,
                attacker: epic_id("attacker"),
                victim: player_id.clone(),
                attacker_velocity: boxcars::Vector3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_velocity: boxcars::Vector3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                victim_location: boxcars::Vector3f {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            }],
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 4,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: Some(0),
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![PlayerSample {
                boost_amount: Some(BOOST_KICKOFF_START_AMOUNT),
                rigid_body: Some(sample_rigid_body(0.0, -1200.0, 17.0)),
                ..sample_player(player_id.clone(), true)
            }],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    let stats = reducer.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.amount_respawned, BOOST_KICKOFF_START_AMOUNT * 2.0);
    assert_eq!(stats.amount_used, BOOST_KICKOFF_START_AMOUNT);
    assert_eq!(
        stats.amount_obtained() - BOOST_KICKOFF_START_AMOUNT,
        stats.amount_used
    );
}

#[test]
fn test_boost_reducer_final_replay_accounting_matches_current_boost() {
    // Legacy replay formats can drift by a couple of raw units relative to the
    // sampled boost state even when aligned to the stats frame number.
    const BOOST_ACCOUNTING_TOLERANCE_RAW: f32 = 3.0;

    for replay_path in [
        "assets/replays/old_boost_format.replay",
        "assets/replays/new_boost_format.replay",
        "assets/replays/rlcs.replay",
        "assets/replays/new_demolition_format.replay",
    ] {
        let replay = parse_replay(replay_path);
        let replay_data = ReplayDataCollector::new()
            .get_replay_data(&replay)
            .expect("Expected replay data");
        let reducer = ReducerCollector::new(BoostReducer::new())
            .process_replay(&replay)
            .expect("Expected boost reducer to process replay")
            .into_inner();
        let stats_timeline = StatsTimelineCollector::new()
            .get_replay_data(&replay)
            .expect("Expected stats timeline data");
        let stats_frame = stats_timeline
            .frames
            .iter()
            .rev()
            .find(|frame| {
                reducer.player_stats().keys().any(|player_id| {
                    replay_player_boost_sample_at_frame(&replay_data, player_id, frame.frame_number)
                        .is_some()
                })
            })
            .expect("Expected a stats frame with current boost data");
        let mut checked_players = 0usize;

        for (player_id, stats) in reducer.player_stats() {
            let Some((current_boost_frame, current_boost_amount)) =
                replay_player_boost_sample_at_frame(
                    &replay_data,
                    player_id,
                    stats_frame.frame_number,
                )
            else {
                continue;
            };
            checked_players += 1;
            let obtained_minus_current = stats.amount_obtained() - current_boost_amount;
            assert!(
                (obtained_minus_current - stats.amount_used).abs()
                    <= BOOST_ACCOUNTING_TOLERANCE_RAW,
                "Expected boost accounting invariant for {replay_path} player {player_id:?} at stats frame {} using replay frame {}: obtained={} current={} used={}",
                stats_frame.frame_number,
                current_boost_frame,
                stats.amount_obtained(),
                current_boost_amount,
                stats.amount_used,
            );
        }

        assert!(
            checked_players > 0,
            "Expected at least one player with current boost data in {replay_path}"
        );
    }
}

#[test]
fn test_boost_reducer_final_replay_collected_totals_match_size_buckets() {
    const BOOST_COLLECTION_TOLERANCE_RAW: f32 = 0.01;

    for replay_path in [
        "assets/replays/old_boost_format.replay",
        "assets/replays/new_boost_format.replay",
        "assets/replays/rlcs.replay",
        "assets/replays/new_demolition_format.replay",
    ] {
        let replay = parse_replay(replay_path);
        let reducer = ReducerCollector::new(BoostReducer::new())
            .process_replay(&replay)
            .expect("Expected boost reducer to process replay")
            .into_inner();

        for (player_id, stats) in reducer.player_stats() {
            assert!(
                (stats.amount_collected
                    - (stats.amount_collected_big + stats.amount_collected_small))
                    .abs()
                    <= BOOST_COLLECTION_TOLERANCE_RAW,
                "Expected collected totals to match size buckets for {replay_path} player {player_id:?}: collected={} big={} small={}",
                stats.amount_collected,
                stats.amount_collected_big,
                stats.amount_collected_small,
            );
        }
    }
}

#[test]
fn test_dodges_replay_first_counted_boost_frame_starts_at_kickoff_amount() {
    let replay = parse_replay("assets/replays/dodges_refreshed_counter.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Expected replay data");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected stats timeline data");

    for player_name in ["tykop", "mrtyzz."] {
        let player_id = timeline
            .replay_meta
            .player_order()
            .find(|info| info.name == player_name)
            .map(|info| info.remote_id.clone())
            .unwrap_or_else(|| panic!("Expected player id for {player_name}"));
        let (frame, player) = timeline
            .frames
            .iter()
            .find_map(|frame| {
                let player = frame
                    .players
                    .iter()
                    .find(|player| player.player_id == player_id)?;
                (player.boost.tracked_time > 0.0).then_some((frame, player))
            })
            .unwrap_or_else(|| panic!("Expected a counted boost frame for {player_name}"));
        let (_, current_boost_amount) =
            replay_player_boost_sample_at_frame(&replay_data, &player_id, frame.frame_number)
                .unwrap_or_else(|| {
                    panic!(
                        "Expected replay boost data for {player_name} at frame {}",
                        frame.frame_number
                    )
                });
        let kickoff_start_pct = boost_amount_to_percent(BOOST_KICKOFF_START_AMOUNT);

        assert!(
            (current_boost_amount - BOOST_KICKOFF_START_AMOUNT).abs() < 1e-3,
            "Expected {player_name} current boost at first counted frame to match kickoff start \
             ({kickoff_start_pct:.2}%), but at frame {} (t={:.3}) current_raw={:.3} \
             current_pct={:.2}% is_live_play={} game_state={:?}",
            frame.frame_number,
            frame.time,
            current_boost_amount,
            boost_amount_to_percent(current_boost_amount),
            frame.is_live_play,
            frame.game_state,
        );
        assert!(
            (player.boost.average_boost_amount() - BOOST_KICKOFF_START_AMOUNT).abs() < 1e-3,
            "Expected {player_name} first counted boost average to match kickoff start \
             ({kickoff_start_pct:.2}%), but at frame {} (t={:.3}) tracked_time={:.3} \
             avg_raw={:.3} avg_pct={:.2}% is_live_play={} game_state={:?}",
            frame.frame_number,
            frame.time,
            player.boost.tracked_time,
            player.boost.average_boost_amount(),
            boost_amount_to_percent(player.boost.average_boost_amount()),
            frame.is_live_play,
            frame.game_state,
        );
    }
}

#[test]
fn test_boost_reducer_treats_midfield_fallback_pads_as_neutral() {
    let player_id = epic_id("fallback-midfield-player");
    let mut reducer = BoostReducer::new();

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 0.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
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
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
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
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 1.0,
                frame: 2,
                pad_id: "VehiclePickup_Boost_TA_nonstandard_midfield".to_string(),
                player: Some(player_id.clone()),
                kind: BoostPadEventKind::PickedUp { sequence: 1 },
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 11.0,
            dt: 10.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
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
            demo_events: Vec::new(),
            boost_pad_events: vec![BoostPadEvent {
                time: 11.0,
                frame: 3,
                pad_id: "VehiclePickup_Boost_TA_nonstandard_midfield".to_string(),
                player: None,
                kind: BoostPadEventKind::Available,
            }],
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
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
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();
    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 2.0,
                frame: 2,
                team_is_team_0: true,
                player: Some(team_zero.player_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();
    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    assert_eq!(reducer.stats().tracked_time, 2.0);
    assert_eq!(reducer.stats().team_zero_time, 1.0);
    assert_eq!(reducer.stats().team_one_time, 0.0);
    assert_eq!(reducer.stats().neutral_time, 1.0);
    assert!((reducer.stats().team_zero_pct() - 50.0).abs() < 0.001);
    assert!((reducer.stats().team_one_pct() - 0.0).abs() < 0.001);
    assert!((reducer.stats().neutral_pct() - 50.0).abs() < 0.001);
}

#[test]
fn test_possession_reducer_uses_touch_event_boundaries() {
    let mut reducer = PossessionReducer::new();
    let team_zero = sample_player(epic_id("team-zero"), true);
    let team_one = sample_player(epic_id("team-one"), false);

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 1.0,
                frame: 1,
                team_is_team_0: true,
                player: Some(team_zero.player_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 3.0,
                frame: 3,
                team_is_team_0: false,
                player: Some(team_one.player_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 4,
            time: 4.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 4.0,
                frame: 4,
                team_is_team_0: false,
                player: Some(team_one.player_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 5,
            time: 5.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: Some(false),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    assert_eq!(reducer.stats().tracked_time, 4.0);
    assert_eq!(reducer.stats().team_zero_time, 3.0);
    assert_eq!(reducer.stats().team_one_time, 1.0);
    assert_eq!(reducer.stats().neutral_time, 0.0);
}

#[test]
fn test_possession_reducer_returns_to_neutral_after_unresolved_challenge() {
    let mut reducer = PossessionReducer::new();
    let team_zero = sample_player(epic_id("team-zero-challenge"), true);
    let team_one = sample_player(epic_id("team-one-challenge"), false);

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 1.0,
                frame: 1,
                team_is_team_0: true,
                player: Some(team_zero.player_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 2.0,
                frame: 2,
                team_is_team_0: false,
                player: Some(team_one.player_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 4,
            time: 4.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: None,
            team_one_score: None,
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    assert_eq!(reducer.stats().tracked_time, 3.0);
    assert_eq!(reducer.stats().team_zero_time, 2.0);
    assert_eq!(reducer.stats().team_one_time, 0.0);
    assert_eq!(reducer.stats().neutral_time, 1.0);
}

#[test]
fn test_possession_reducer_resets_after_goal_before_next_kickoff_touch() {
    let mut reducer = PossessionReducer::new();
    let team_zero = sample_player(epic_id("team-zero-goal-reset"), true);
    let team_one = sample_player(epic_id("team-one-goal-reset"), false);

    reducer
        .on_sample(&CoreSample {
            frame_number: 1,
            time: 1.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: vec![TouchEvent {
                time: 1.0,
                frame: 1,
                team_is_team_0: true,
                player: Some(team_zero.player_id.clone()),
                closest_approach_distance: None,
            }],
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 2,
            time: 2.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 3,
            time: 3.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(1),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: Some(false),
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: vec![GoalEvent {
                time: 3.0,
                frame: 3,
                scoring_team_is_team_0: true,
                player: Some(team_zero.player_id.clone()),
                team_zero_score: Some(1),
                team_one_score: Some(0),
            }],
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 4,
            time: 4.0,
            dt: 0.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: Some(3),
            team_zero_score: Some(1),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero.clone(), team_one.clone()],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    reducer
        .on_sample(&CoreSample {
            frame_number: 5,
            time: 5.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            team_zero_score: Some(1),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: None,
            ball: None,
            players: vec![team_zero, team_one],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        })
        .unwrap();

    assert_eq!(reducer.stats().tracked_time, 2.0);
    assert_eq!(reducer.stats().team_zero_time, 1.0);
    assert_eq!(reducer.stats().team_one_time, 0.0);
    assert_eq!(reducer.stats().neutral_time, 1.0);
}

#[test]
fn test_demo_reducer_counts_events_and_dedupes_consecutive_frames() {
    let attacker = epic_id("attacker");
    let victim = epic_id("victim");
    let mut reducer = DemoReducer::new();

    for frame_number in [10, 11, 30] {
        reducer
            .on_sample(&CoreSample {
                frame_number,
                time: frame_number as f32 / 10.0,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                ball_has_been_hit: None,
                kickoff_countdown_time: None,
                team_zero_score: None,
                team_one_score: None,
                possession_team_is_team_0: None,
                scored_on_team_is_team_0: None,
                current_in_game_team_player_counts: None,
                ball: None,
                players: vec![
                    sample_player(attacker.clone(), true),
                    sample_player(victim.clone(), false),
                ],
                active_demos: vec![DemoEventSample {
                    attacker: attacker.clone(),
                    victim: victim.clone(),
                }],
                demo_events: Vec::new(),
                boost_pad_events: Vec::new(),
                touch_events: Vec::new(),
                dodge_refreshed_events: Vec::new(),
                player_stat_events: Vec::new(),
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
    let replay = parse_replay("assets/replays/new_demolition_format.replay");
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

#[test]
fn test_demo_reducer_keeps_exact_timeline_under_sampling() {
    let replay = parse_replay("assets/replays/new_demolition_format.replay");
    let full = ReducerCollector::new(DemoReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with full demo reducer")
        .into_inner();

    let mut sampled_collector = ReducerCollector::new(DemoReducer::new());
    FrameRateDecorator::new_from_fps(1.0, &mut sampled_collector)
        .process_replay(&replay)
        .expect("Failed to process replay with sampled demo reducer");
    let sampled = sampled_collector.into_inner();

    assert_eq!(sampled.team_zero_stats(), full.team_zero_stats());
    assert_eq!(sampled.team_one_stats(), full.team_one_stats());
    assert_eq!(sampled.player_stats(), full.player_stats());
    assert_eq!(sampled.timeline(), full.timeline());
}
