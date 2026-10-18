use super::*;

#[test]
fn counter_expansion_is_bounded_across_signed_extremes() {
    for (previous, current, expected) in [
        (3, 5, 2),
        (5, 5, 0),
        (5, 0, 0),
        (i32::MAX, i32::MIN, 0),
        (i32::MIN, i32::MAX, MAX_COUNTER_EVENTS_PER_UPDATE),
        (0, i32::MAX, MAX_COUNTER_EVENTS_PER_UPDATE),
        (i32::MAX - 1, i32::MAX, 1),
    ] {
        assert_eq!(bounded_counter_delta(previous, current), expected);
    }
}

#[test]
fn replicated_counter_updates_are_clamped_without_replaying_the_backlog() {
    let bytes = std::fs::read("assets/post-eac-ranked-duel-2026-04-28-a.replay").unwrap();
    let replay = boxcars::ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .parse()
        .unwrap();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    let player = processor.team_zero[0].clone();
    let player_actor = processor.player_to_actor_id[&player];
    let car_actor = processor.get_car_actor_id(&player).unwrap();
    let shots_key = *processor.get_object_id_for_key(MATCH_SHOTS_KEY).unwrap();
    let dodge_key = processor
        .cached_object_ids
        .dodges_refreshed_counter
        .unwrap();
    processor
        .player_stat_counters
        .insert((player.clone(), PlayerStatEventKind::Shot), i32::MIN);
    processor.dodge_refreshed_counters.insert(player, i32::MIN);
    let mut frame = boxcars::Frame {
        time: 20.0,
        delta: 0.1,
        new_actors: Vec::new(),
        deleted_actors: Vec::new(),
        updated_actors: [(player_actor, shots_key), (car_actor, dodge_key)]
            .map(|(actor_id, object_id)| boxcars::UpdatedAttribute {
                actor_id,
                object_id,
                stream_id: boxcars::StreamId(0),
                attribute: boxcars::Attribute::Int(i32::MAX - 1),
            })
            .to_vec(),
    };
    processor.update_player_stat_events(&frame, 400).unwrap();
    processor
        .update_dodge_refreshed_events(&frame, 400)
        .unwrap();
    assert_eq!(
        processor.current_frame_player_stat_events.len(),
        MAX_COUNTER_EVENTS_PER_UPDATE as usize
    );
    assert_eq!(
        processor.current_frame_dodge_refreshed_events.len(),
        MAX_COUNTER_EVENTS_PER_UPDATE as usize
    );
    assert_eq!(
        processor
            .current_frame_dodge_refreshed_events
            .last()
            .unwrap()
            .counter_value,
        i32::MAX - 1
    );

    for update in &mut frame.updated_actors {
        update.attribute = boxcars::Attribute::Int(i32::MAX);
    }
    processor.update_player_stat_events(&frame, 401).unwrap();
    processor
        .update_dodge_refreshed_events(&frame, 401)
        .unwrap();
    assert_eq!(processor.current_frame_player_stat_events.len(), 1);
    assert_eq!(processor.current_frame_dodge_refreshed_events.len(), 1);
    assert_eq!(
        processor.current_frame_dodge_refreshed_events[0].counter_value,
        i32::MAX
    );
}
