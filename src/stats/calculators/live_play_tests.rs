use super::*;
use crate::{GoalEvent, GAME_STATE_KICKOFF_COUNTDOWN};

#[test]
fn kickoff_waiting_for_first_touch_is_not_live_play() {
    let mut tracker = LivePlayTracker::default();
    let gameplay = GameplayState {
        ball_has_been_hit: Some(false),
        ..Default::default()
    };
    let state = tracker.state_parts(&gameplay, &FrameEventsState::default());

    assert_eq!(state.gameplay_phase, GameplayPhase::KickoffWaitingForTouch);
    assert!(!state.is_live_play);
    assert!(state.gameplay_phase.counts_toward_player_motion());
}

#[test]
fn match_time_remaining_does_not_force_kickoff_countdown() {
    let mut tracker = LivePlayTracker::default();
    let gameplay = GameplayState {
        game_state: Some(55),
        kickoff_countdown_time: Some(299),
        ball_has_been_hit: Some(true),
        ..Default::default()
    };
    let state = tracker.state_parts(&gameplay, &FrameEventsState::default());

    assert_eq!(state.gameplay_phase, GameplayPhase::ActivePlay);
    assert!(state.is_live_play);
}

#[test]
fn replicated_countdown_state_marks_kickoff_countdown() {
    let mut tracker = LivePlayTracker::default();
    let gameplay = GameplayState {
        game_state: Some(GAME_STATE_KICKOFF_COUNTDOWN),
        ball_has_been_hit: Some(true),
        ..Default::default()
    };
    let state = tracker.state_parts(&gameplay, &FrameEventsState::default());

    assert_eq!(state.gameplay_phase, GameplayPhase::KickoffCountdown);
    assert!(!state.is_live_play);
}

#[test]
fn active_game_state_with_zero_countdown_is_live_play() {
    let mut tracker = LivePlayTracker::default();
    let gameplay = GameplayState {
        game_state: Some(54),
        kickoff_countdown_time: Some(0),
        ball_has_been_hit: Some(true),
        ..Default::default()
    };
    let state = tracker.state_parts(&gameplay, &FrameEventsState::default());

    assert_eq!(state.gameplay_phase, GameplayPhase::ActivePlay);
    assert!(state.is_live_play);
}

#[test]
fn legacy_countdown_time_without_state_still_marks_kickoff_countdown() {
    let mut tracker = LivePlayTracker::default();
    let gameplay = GameplayState {
        kickoff_countdown_time: Some(3),
        ball_has_been_hit: Some(true),
        ..Default::default()
    };
    let state = tracker.state_parts(&gameplay, &FrameEventsState::default());

    assert_eq!(state.gameplay_phase, GameplayPhase::KickoffCountdown);
    assert!(!state.is_live_play);
}

#[test]
fn goal_event_enters_post_goal_phase() {
    let mut tracker = LivePlayTracker::default();
    let gameplay = GameplayState::default();
    let events = FrameEventsState {
        goal_events: vec![GoalEvent {
            time: 10.0,
            frame: 1,
            scoring_team_is_team_0: true,
            player: None,
            team_zero_score: None,
            team_one_score: None,
        }],
        ..Default::default()
    };

    let state = tracker.state_parts(&gameplay, &events);

    assert_eq!(state.gameplay_phase, GameplayPhase::PostGoal);
    assert!(!state.is_live_play);
}
