use super::*;

fn touch(player_id: u64, contact_gap: f32) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time: 2.0,
        frame: 20,
        team_is_team_0: true,
        player: Some(PlayerId::Steam(player_id)),
        player_position: None,
        closest_approach_distance: Some(contact_gap),
        dodge_contact: false,
    }
}

#[test]
fn chronological_touch_events_orders_same_timestamp_contacts_primary_first() {
    let weaker = touch(1, 20.0);
    let best = touch(2, 0.0);
    let touch_events = vec![weaker.clone(), best.clone()];

    let ordered = chronological_touch_events(&touch_events);

    assert_eq!(ordered[0].player, best.player);
    assert_eq!(ordered[1].player, weaker.player);
}

#[test]
fn sequential_touch_events_orders_same_timestamp_contacts_primary_last() {
    let best = touch(1, 0.0);
    let weaker = touch(2, 20.0);
    let touch_events = vec![best.clone(), weaker.clone()];

    let ordered = sequential_touch_events(&touch_events);

    assert_eq!(ordered[0].player, weaker.player);
    assert_eq!(ordered[1].player, best.player);
}
