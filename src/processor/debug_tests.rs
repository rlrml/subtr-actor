use super::*;

#[test]
fn debug_helpers_tolerate_missing_actors_and_object_names() {
    let bytes = std::fs::read("assets/post-eac-ranked-duel-2026-04-28-a.replay").unwrap();
    let replay = boxcars::ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .parse()
        .unwrap();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    let actor = processor.get_car_actor_id(&processor.team_zero[0]).unwrap();
    let attributes = &processor.actor_state.actor_states[&actor].attributes;
    assert!(!processor.map_attribute_keys(attributes).is_empty());
    processor.object_id_to_name.clear();
    assert!(processor.map_attribute_keys(attributes).is_empty());
    assert_eq!(processor.actor_state_string(&actor), "{}");
    processor.print_actors_by_id([actor, boxcars::ActorId(i32::MAX)].iter());
    processor.print_actors_of_type("missing actor type");
    processor.print_actors_of_type(CAR_TYPE);
    processor.print_all_actors();
}
