use super::*;

#[test]
fn simultaneous_touch_contacts_preserve_pending_takeoff_counts_for_all_touched_players() {
    let primary_player = PlayerId::Steam(1);
    let secondary_player = PlayerId::Steam(2);
    let stale_player = PlayerId::Steam(3);
    let mut tracker = ContinuousBallControlTracker::<BallCarryKind>::default();
    tracker
        .pending_takeoff_touches
        .insert(primary_player.clone(), 1);
    tracker.pending_takeoff_touches.insert(stale_player, 4);

    tracker.track_touch_contacts(&[
        ContinuousBallControlTouch {
            player_id: primary_player.clone(),
            is_airborne: false,
        },
        ContinuousBallControlTouch {
            player_id: secondary_player.clone(),
            is_airborne: false,
        },
    ]);

    assert_eq!(
        tracker.pending_takeoff_touches.get(&primary_player),
        Some(&2)
    );
    assert_eq!(
        tracker.pending_takeoff_touches.get(&secondary_player),
        Some(&1)
    );
    assert_eq!(tracker.pending_takeoff_touches.len(), 2);
}

#[test]
fn airborne_simultaneous_touch_preserves_existing_pending_takeoff_without_incrementing() {
    let primary_player = PlayerId::Steam(1);
    let secondary_player = PlayerId::Steam(2);
    let mut tracker = ContinuousBallControlTracker::<BallCarryKind>::default();
    tracker
        .pending_takeoff_touches
        .insert(primary_player.clone(), 1);

    tracker.track_touch_contacts(&[
        ContinuousBallControlTouch {
            player_id: primary_player.clone(),
            is_airborne: true,
        },
        ContinuousBallControlTouch {
            player_id: secondary_player.clone(),
            is_airborne: false,
        },
    ]);

    assert_eq!(
        tracker.pending_takeoff_touches.get(&primary_player),
        Some(&1)
    );
    assert_eq!(
        tracker.pending_takeoff_touches.get(&secondary_player),
        Some(&1)
    );
}
