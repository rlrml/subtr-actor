use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};
use subtr_actor::{GameplayPhase, GoalEvent};

use super::*;
use crate::meta::LiveMatchMeta;
use crate::model::{
    LiveBoostPadEvent, LiveBoostPadEventKind, LiveDemolishEvent, LiveDodgeRefreshedEvent,
    LiveEventTiming, LiveFrame, LiveGoalEvent, LivePlayerFrame, LiveTouchEvent,
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

fn two_player_frame(frame_number: u64) -> LiveFrame {
    live_frame(
        frame_number,
        test_rigid_body(vec3(0.0, 0.0, 92.75), vec3(0.0, 0.0, 0.0)),
        vec![
            player_at_index(0, true, vec3(0.0, 0.0, 92.75)),
            player_at_index(1, false, vec3(120.0, 0.0, 92.75)),
        ],
    )
}

fn touch_event(player_index: u32) -> LiveTouchEvent {
    LiveTouchEvent {
        timing: LiveEventTiming::default(),
        player: Some(RemoteId::SplitScreen(player_index)),
        is_team_0: true,
        closest_approach_distance: Some(12.0),
    }
}

fn dodge_refresh_event(player_index: u32, counter_value: i32) -> LiveDodgeRefreshedEvent {
    LiveDodgeRefreshedEvent {
        timing: LiveEventTiming::default(),
        player: RemoteId::SplitScreen(player_index),
        is_team_0: true,
        counter_value,
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

fn goal_event(scoring_team_is_team_0: bool, scores: Option<(i32, i32)>) -> LiveGoalEvent {
    LiveGoalEvent {
        timing: LiveEventTiming::default(),
        scoring_team_is_team_0,
        player: Some(RemoteId::SplitScreen(0)),
        team_zero_score: scores.map(|(team_zero, _)| team_zero),
        team_one_score: scores.map(|(_, team_one)| team_one),
    }
}

fn boost_pad_pickup(pad_id: &str, sequence: u8) -> LiveBoostPadEvent {
    LiveBoostPadEvent {
        timing: LiveEventTiming::default(),
        pad_id: pad_id.to_owned(),
        kind: LiveBoostPadEventKind::PickedUp,
        sequence,
        player: Some(RemoteId::SplitScreen(0)),
    }
}

#[test]
fn explicit_touch_marks_kickoff_waiting_frame_as_active_play() {
    let mut generator = LiveEventGenerator::default();
    let mut frame = two_player_frame(1);
    frame.ball_has_been_hit = Some(false);
    frame.events.touches = vec![touch_event(0)];

    let (frame_events, live_play) = generator.frame_events(&frame);

    assert_eq!(live_play.gameplay_phase, GameplayPhase::ActivePlay);
    assert!(live_play.is_live_play);
    assert_eq!(frame_events.touch_events.len(), 1);
    assert_eq!(
        frame_events.touch_events[0].player,
        Some(RemoteId::SplitScreen(0))
    );
    assert_eq!(
        frame_events.touch_events[0].closest_approach_distance,
        Some(12.0)
    );
}

#[test]
fn explicit_touch_marks_stale_kickoff_countdown_frame_as_active_play() {
    const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 53;

    let mut generator = LiveEventGenerator::default();
    let mut frame = two_player_frame(1);
    frame.game_state = Some(GAME_STATE_KICKOFF_COUNTDOWN);
    frame.kickoff_countdown_time = Some(3);
    frame.ball_has_been_hit = Some(false);
    frame.events.touches = vec![touch_event(0)];

    let (frame_events, live_play) = generator.frame_events(&frame);

    assert_eq!(live_play.gameplay_phase, GameplayPhase::ActivePlay);
    assert!(live_play.is_live_play);
    assert_eq!(frame_events.touch_events.len(), 1);
}

#[test]
fn explicit_live_play_overrides_derived_state() {
    let mut generator = LiveEventGenerator::default();
    let mut frame = two_player_frame(1);
    frame.live_play = Some(false);

    let (_, live_play) = generator.frame_events(&frame);

    assert_eq!(live_play.gameplay_phase, GameplayPhase::Unknown);
    assert!(!live_play.is_live_play);
}

#[test]
fn duplicate_explicit_touches_are_suppressed_within_one_frame() {
    let mut generator = LiveEventGenerator::default();
    let mut frame = two_player_frame(1);
    let mut second_touch = touch_event(0);
    second_touch.closest_approach_distance = Some(16.0);
    frame.events.touches = vec![touch_event(0), second_touch];

    let (frame_events, _) = generator.frame_events(&frame);

    assert_eq!(frame_events.touch_events.len(), 1);
    assert_eq!(
        frame_events.touch_events[0].closest_approach_distance,
        Some(12.0)
    );
}

#[test]
fn dodge_refresh_synthesizes_touch_when_no_touch_events_exist() {
    let mut generator = LiveEventGenerator::default();
    let mut frame = live_frame(
        1,
        test_rigid_body(vec3(0.0, 3000.0, 800.0), vec3(0.0, 0.0, 0.0)),
        vec![
            player_at_index(0, true, vec3(0.0, 0.0, 92.75)),
            player_at_index(1, false, vec3(120.0, 0.0, 92.75)),
        ],
    );
    frame.ball_has_been_hit = Some(false);
    frame.events.dodge_refreshes = vec![dodge_refresh_event(0, 7)];

    let (frame_events, live_play) = generator.frame_events(&frame);

    assert!(live_play.is_live_play);
    assert_eq!(frame_events.dodge_refreshed_events.len(), 1);
    assert_eq!(frame_events.touch_events.len(), 1);
    assert!(frame_events.touch_events[0].dodge_contact);
    assert_eq!(
        frame_events.touch_events[0].player,
        Some(RemoteId::SplitScreen(0))
    );
}

#[test]
fn dodge_refresh_counter_values_deduplicate_across_frames() {
    let mut generator = LiveEventGenerator::default();
    let mut first = two_player_frame(1);
    first.events.dodge_refreshes = vec![dodge_refresh_event(0, 7)];
    let (first_events, _) = generator.frame_events(&first);
    assert_eq!(first_events.dodge_refreshed_events.len(), 1);

    let mut second = two_player_frame(2);
    second.events.dodge_refreshes = vec![dodge_refresh_event(0, 7)];
    let (second_events, _) = generator.frame_events(&second);
    assert!(second_events.dodge_refreshed_events.is_empty());

    let mut third = two_player_frame(3);
    third.events.dodge_refreshes = vec![dodge_refresh_event(0, 8)];
    let (third_events, _) = generator.frame_events(&third);
    assert_eq!(third_events.dodge_refreshed_events.len(), 1);
    assert_eq!(third_events.dodge_refreshed_events[0].counter_value, 8);
}

#[test]
fn goal_events_deduplicate_by_score_and_time_window() {
    let mut generator = LiveEventGenerator::default();
    let mut first = two_player_frame(1);
    first.events.goals = vec![goal_event(true, Some((1, 0)))];
    let (first_events, _) = generator.frame_events(&first);
    assert_eq!(first_events.goal_events.len(), 1);

    let mut duplicate = two_player_frame(2);
    duplicate.events.goals = vec![goal_event(true, Some((1, 0)))];
    let (duplicate_events, _) = generator.frame_events(&duplicate);
    assert!(duplicate_events.goal_events.is_empty());

    let mut advanced = two_player_frame(3);
    advanced.events.goals = vec![goal_event(false, Some((1, 1)))];
    let (advanced_events, _) = generator.frame_events(&advanced);
    assert_eq!(advanced_events.goal_events.len(), 1);
}

#[test]
fn goal_events_without_scores_deduplicate_within_time_window() {
    let previous = GoalEvent {
        time: 1.0,
        frame: 10,
        scoring_team_is_team_0: true,
        player: None,
        player_position: None,
        team_zero_score: None,
        team_one_score: None,
    };
    let mut candidate = previous.clone();
    candidate.time = previous.time + GOAL_EVENT_DEDUPE_WINDOW_SECONDS - 0.1;
    assert!(goal_event_is_duplicate(&previous, &candidate));

    candidate.time = previous.time + GOAL_EVENT_DEDUPE_WINDOW_SECONDS + 0.1;
    assert!(!goal_event_is_duplicate(&previous, &candidate));

    candidate.time = previous.time;
    candidate.scoring_team_is_team_0 = false;
    assert!(!goal_event_is_duplicate(&previous, &candidate));
}

#[test]
fn boost_pad_pickup_sequences_are_suppressed_until_respawn() {
    let mut generator = LiveEventGenerator::default();
    let mut first = two_player_frame(1);
    first.events.boost_pad_events = vec![boost_pad_pickup("34", 1)];
    let (first_events, _) = generator.frame_events(&first);
    assert_eq!(first_events.boost_pad_events.len(), 1);

    let mut repeat = two_player_frame(2);
    repeat.events.boost_pad_events = vec![boost_pad_pickup("34", 1)];
    let (repeat_events, _) = generator.frame_events(&repeat);
    assert!(repeat_events.boost_pad_events.is_empty());

    let mut next_sequence = two_player_frame(3);
    next_sequence.events.boost_pad_events = vec![boost_pad_pickup("34", 2)];
    let (next_sequence_events, _) = generator.frame_events(&next_sequence);
    assert_eq!(next_sequence_events.boost_pad_events.len(), 1);

    // Same sequence after the minimum respawn window is a fresh pickup.
    let respawn_frame_number = 1 + (MIN_BOOST_PAD_RESPAWN_SECONDS as u64 + 1) * 10;
    let mut respawned = two_player_frame(respawn_frame_number);
    respawned.events.boost_pad_events = vec![boost_pad_pickup("34", 1)];
    let (respawned_events, _) = generator.frame_events(&respawned);
    assert_eq!(respawned_events.boost_pad_events.len(), 1);
}

#[test]
fn demolish_events_deduplicate_within_known_frame_window() {
    let mut generator = LiveEventGenerator::default();
    let mut first = two_player_frame(1);
    first.events.demolishes = vec![demolish_event(0, 1)];
    let (first_events, _) = generator.frame_events(&first);
    assert_eq!(first_events.demo_events.len(), 1);
    assert_eq!(first_events.active_demos.len(), 1);

    let mut repeat = two_player_frame(2);
    repeat.events.demolishes = vec![demolish_event(0, 1)];
    let (repeat_events, _) = generator.frame_events(&repeat);
    assert!(repeat_events.demo_events.is_empty());

    let past_window_frame_number = 2 + MAX_DEMOLISH_KNOWN_FRAMES_PASSED as u64;
    let mut past_window = two_player_frame(past_window_frame_number);
    past_window.events.demolishes = vec![demolish_event(0, 1)];
    let (past_window_events, _) = generator.frame_events(&past_window);
    assert_eq!(past_window_events.demo_events.len(), 1);
}

#[test]
fn active_demos_expire_after_their_reported_duration() {
    let mut generator = LiveEventGenerator::default();
    let mut first = two_player_frame(1);
    first.events.demolishes = vec![demolish_event(0, 1)];
    let (first_events, _) = generator.frame_events(&first);
    assert_eq!(first_events.active_demos.len(), 1);

    let second = two_player_frame(2);
    let (second_events, _) = generator.frame_events(&second);
    assert_eq!(second_events.active_demos.len(), 1);

    let fourth = two_player_frame(4);
    let (fourth_events, _) = generator.frame_events(&fourth);
    assert!(fourth_events.active_demos.is_empty());
}

#[test]
fn frame_input_from_live_frame_returns_events_and_live_play() {
    let mut generator = LiveEventGenerator::default();
    let mut history = LiveEventHistory::default();
    let mut frame = two_player_frame(1);
    frame.events.touches = vec![touch_event(0)];
    frame.events.demolishes = vec![demolish_event(0, 1)];

    let (frame_input, frame_events, live_play) =
        crate::frame_input_from_live_frame(&mut generator, &mut history, None, frame);

    assert!(live_play.is_live_play);
    assert_eq!(frame_events.touch_events.len(), 1);
    assert_eq!(frame_events.demo_events.len(), 1);
    assert_eq!(history.touch_events.len(), 1);
    assert_eq!(history.demo_events.len(), 1);
    let input_events = frame_input.frame_events_state();
    assert_eq!(input_events.touch_events.len(), 1);
    assert_eq!(input_events.demo_events.len(), 1);
    assert_eq!(input_events.active_demos.len(), 1);
    assert_eq!(frame_input.player_frame_state().players.len(), 2);
    assert_eq!(frame_input.live_play_state(), Some(live_play));
}

#[test]
fn live_match_meta_builds_replay_meta_and_signature() {
    let mut players = vec![
        player_at_index(0, true, vec3(0.0, 0.0, 92.75)),
        player_at_index(1, false, vec3(120.0, 0.0, 92.75)),
    ];
    players[1].name = Some("Orange".to_owned());
    let meta = LiveMatchMeta::from_player_frames(&players);
    let replay_meta = meta.replay_meta();

    assert_eq!(replay_meta.team_zero.len(), 1);
    assert_eq!(replay_meta.team_one.len(), 1);
    assert_eq!(replay_meta.team_zero[0].name, "Player 0");
    assert_eq!(replay_meta.team_one[0].name, "Orange");
    assert_eq!(
        replay_meta.team_zero[0].car_hitbox_family.as_deref(),
        Some("Octane")
    );
    assert_eq!(
        meta.signature(),
        vec![
            (RemoteId::SplitScreen(0), true, None),
            (RemoteId::SplitScreen(1), false, Some("Orange".to_owned())),
        ]
    );

    players[0].name = Some("Blue".to_owned());
    assert_ne!(
        LiveMatchMeta::from_player_frames(&players).signature(),
        meta.signature()
    );
}

#[test]
fn has_duplicate_player_indices_detects_collisions() {
    let players = vec![
        player_at_index(0, true, vec3(0.0, 0.0, 92.75)),
        player_at_index(1, false, vec3(120.0, 0.0, 92.75)),
    ];
    assert!(!has_duplicate_player_indices(&players));

    let duplicated = vec![
        player_at_index(0, true, vec3(0.0, 0.0, 92.75)),
        player_at_index(0, false, vec3(120.0, 0.0, 92.75)),
    ];
    assert!(has_duplicate_player_indices(&duplicated));
}
