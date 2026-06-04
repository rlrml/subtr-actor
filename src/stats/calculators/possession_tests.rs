use super::*;

fn touch(frame: usize, time: f32, player: PlayerId, team_is_team_0: bool) -> TouchEvent {
    TouchEvent {
        time,
        frame,
        team_is_team_0,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
    }
}

#[test]
fn tracker_uses_latest_touch_player_for_team_independent_of_slice_order() {
    let earlier_player = PlayerId::Steam(1);
    let later_player = PlayerId::Steam(2);
    let mut tracker = PossessionTracker::default();
    let later_touch = touch(20, 2.0, later_player.clone(), true);
    let earlier_touch = touch(10, 1.0, earlier_player, true);

    let state = tracker.update(2.0, &[later_touch, earlier_touch]);

    assert_eq!(state.current_team_is_team_0, Some(true));
    assert_eq!(state.current_player, Some(later_player));
}
