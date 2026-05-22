use super::*;
use crate::GoalEvent;

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
