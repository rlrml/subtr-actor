use super::*;

fn replay() -> boxcars::Replay {
    let bytes = std::fs::read("assets/post-eac-ranked-duel-2026-04-28-a.replay").unwrap();
    boxcars::ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .parse()
        .unwrap()
}

fn empty_frame() -> boxcars::Frame {
    boxcars::Frame {
        time: 20.0,
        delta: 0.1,
        new_actors: Vec::new(),
        updated_actors: Vec::new(),
        deleted_actors: Vec::new(),
    }
}

fn spawn(actor_id: boxcars::ActorId, object_id: boxcars::ObjectId) -> boxcars::NewActor {
    boxcars::NewActor {
        actor_id,
        object_id,
        name_id: None,
        initial_trajectory: boxcars::Trajectory {
            location: None,
            rotation: None,
        },
    }
}

fn active(actor: boxcars::ActorId) -> boxcars::ActiveActor {
    boxcars::ActiveActor {
        active: true,
        actor,
    }
}

#[test]
fn component_id_reuse_removes_old_links_before_applying_new_links() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    let old_car = processor.get_car_actor_id(&processor.team_zero[0]).unwrap();
    let new_car = processor.get_car_actor_id(&processor.team_one[0]).unwrap();
    let components = [
        processor.car_to_boost[&old_car],
        processor.car_to_dodge[&old_car],
        processor.car_to_jump[&old_car],
        processor.car_to_double_jump[&old_car],
    ];
    let mut frame = empty_frame();
    frame.deleted_actors = components.to_vec();
    for component in components {
        frame.new_actors.push(spawn(
            component,
            processor.actor_state.actor_states[&component].object_id,
        ));
        frame.updated_actors.push(boxcars::UpdatedAttribute {
            actor_id: component,
            object_id: *processor.get_object_id_for_key(VEHICLE_KEY).unwrap(),
            stream_id: boxcars::StreamId(0),
            attribute: boxcars::Attribute::ActiveActor(active(new_car)),
        });
    }
    processor.actor_state.process_frame(&frame, 400).unwrap();
    processor.update_mappings(&frame).unwrap();
    for (links, component) in [
        &processor.car_to_boost,
        &processor.car_to_dodge,
        &processor.car_to_jump,
        &processor.car_to_double_jump,
    ]
    .into_iter()
    .zip(components)
    {
        assert!(!links.contains_key(&old_car));
        assert_eq!(links.get(&new_car), Some(&component));
    }
}

#[test]
fn car_deletion_clears_components_but_preserves_same_frame_demolition() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    let victim = processor.team_zero[0].clone();
    let attacker = processor.team_one[0].clone();
    let victim_car = processor.get_car_actor_id(&victim).unwrap();
    let attacker_car = processor.get_car_actor_id(&attacker).unwrap();
    let mut frame = empty_frame();
    frame.deleted_actors.push(victim_car);
    frame.updated_actors.push(boxcars::UpdatedAttribute {
        actor_id: attacker_car,
        object_id: *processor
            .get_object_id_for_key(DEMOLISH_EXTENDED_KEY)
            .unwrap(),
        stream_id: boxcars::StreamId(0),
        attribute: boxcars::Attribute::DemolishExtended(Box::new(boxcars::DemolishExtended {
            attacker_pri: active(processor.player_to_actor_id[&attacker]),
            self_demo: active(boxcars::ActorId(-1)),
            self_demolish: false,
            goal_explosion_owner: active(boxcars::ActorId(-1)),
            attacker: active(attacker_car),
            victim: active(victim_car),
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
        })),
    });
    processor.actor_state.process_frame(&frame, 400).unwrap();
    processor.update_mappings(&frame).unwrap();
    for links in [
        &processor.car_to_boost,
        &processor.car_to_dodge,
        &processor.car_to_jump,
        &processor.car_to_double_jump,
    ] {
        assert!(!links.contains_key(&victim_car));
    }
    processor.demolishes.clear();
    processor.update_demolishes(&frame, 400).unwrap();
    assert_eq!(processor.demolishes.len(), 1);
    assert_eq!(processor.demolishes[0].victim, victim);
    assert_eq!(processor.demolishes[0].attacker, attacker);
}

#[test]
fn replacement_ball_is_available_in_its_spawn_frame() {
    let replay = replay();
    for reuse_id in [false, true] {
        let mut processor = ReplayProcessor::new(&replay).unwrap();
        processor.process_long_enough_to_get_actor_ids().unwrap();
        let old_ball = processor.get_ball_actor_id().unwrap();
        let new_ball = if reuse_id {
            old_ball
        } else {
            boxcars::ActorId(10000)
        };
        let mut frame = empty_frame();
        frame.deleted_actors.push(old_ball);
        frame.new_actors.push(spawn(
            new_ball,
            processor.actor_state.actor_states[&old_ball].object_id,
        ));
        processor.actor_state.process_frame(&frame, 400).unwrap();
        processor.update_ball_id(&frame).unwrap();
        assert_eq!(processor.get_ball_actor_id().unwrap(), new_ball);
    }
}

#[test]
fn same_frame_player_actor_reuse_preserves_the_new_car_link() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    let player = processor.team_zero[0].clone();
    let player_actor = processor.player_to_actor_id[&player];
    let old_car = processor.get_car_actor_id(&player).unwrap();
    let new_car = boxcars::ActorId(10000);
    let mut frame = empty_frame();
    frame.deleted_actors.push(player_actor);
    frame.new_actors = vec![
        spawn(
            player_actor,
            processor.actor_state.actor_states[&player_actor].object_id,
        ),
        spawn(
            new_car,
            processor.actor_state.actor_states[&old_car].object_id,
        ),
    ];
    frame.updated_actors.push(boxcars::UpdatedAttribute {
        actor_id: new_car,
        object_id: *processor
            .get_object_id_for_key(PLAYER_REPLICATION_KEY)
            .unwrap(),
        stream_id: boxcars::StreamId(0),
        attribute: boxcars::Attribute::ActiveActor(active(player_actor)),
    });
    frame.updated_actors.push(boxcars::UpdatedAttribute {
        actor_id: player_actor,
        object_id: *processor.get_object_id_for_key(UNIQUE_ID_KEY).unwrap(),
        stream_id: boxcars::StreamId(0),
        attribute: processor.actor_state.actor_states[&player_actor].attributes
            [processor.get_object_id_for_key(UNIQUE_ID_KEY).unwrap()]
        .0
        .clone(),
    });
    processor.actor_state.process_frame(&frame, 400).unwrap();
    processor.update_mappings(&frame).unwrap();
    assert_eq!(processor.get_car_actor_id(&player).unwrap(), new_car);
    assert_eq!(
        processor.get_player_id_from_car_id(&new_car).unwrap(),
        player
    );
}

#[test]
fn reused_actor_ids_do_not_resolve_previous_players() {
    let replay = replay();
    for reuse_player in [false, true] {
        let mut processor = ReplayProcessor::new(&replay).unwrap();
        processor.process_long_enough_to_get_actor_ids().unwrap();
        let player = processor.team_zero[0].clone();
        let car = processor.get_car_actor_id(&player).unwrap();
        let actor = if reuse_player {
            processor.player_to_actor_id[&player]
        } else {
            car
        };
        let object = processor.actor_state.actor_states[&actor].object_id;
        let mut frame = empty_frame();
        frame.deleted_actors.push(actor);
        processor.actor_state.process_frame(&frame, 400).unwrap();
        processor.update_mappings(&frame).unwrap();
        assert_eq!(processor.get_player_id_from_car_id(&car).unwrap(), player);
        frame.deleted_actors.clear();
        frame.new_actors.push(spawn(actor, object));
        processor.actor_state.process_frame(&frame, 401).unwrap();
        processor.update_mappings(&frame).unwrap();
        assert!(processor.get_player_id_from_car_id(&car).is_err());
        assert!(processor.get_car_actor_id(&player).is_err());
    }
}
