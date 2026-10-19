use super::*;

fn replay() -> boxcars::Replay {
    let bytes = std::fs::read("assets/post-eac-ranked-duel-2026-04-28-a.replay").unwrap();
    boxcars::ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .parse()
        .unwrap()
}

fn frame() -> boxcars::Frame {
    boxcars::Frame {
        time: 20.0,
        delta: 0.1,
        new_actors: Vec::new(),
        updated_actors: Vec::new(),
        deleted_actors: Vec::new(),
    }
}

fn demo(processor: &ReplayProcessor<'_>) -> DemolishAttribute {
    let active = |actor| boxcars::ActiveActor {
        active: true,
        actor,
    };
    DemolishAttribute::Extended(boxcars::DemolishExtended {
        attacker_pri: active(processor.player_to_actor_id[&processor.team_one[0]]),
        self_demo: active(boxcars::ActorId(-1)),
        self_demolish: false,
        goal_explosion_owner: active(boxcars::ActorId(-1)),
        attacker: active(processor.get_car_actor_id(&processor.team_one[0]).unwrap()),
        victim: active(processor.get_car_actor_id(&processor.team_zero[0]).unwrap()),
        attacker_velocity: boxcars::Vector3f {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        victim_velocity: boxcars::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    })
}

#[test]
fn unresolved_demolition_can_be_retried_after_player_link_arrives() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    processor.demolishes.clear();
    let demo = demo(&processor);
    let car = demo.attacker_actor_id();
    let player = processor.car_to_player.remove(&car).unwrap();
    processor.try_push_demolish(&demo, &frame(), 400);
    assert!(processor.demolishes.is_empty());
    processor.car_to_player.insert(car, player);
    processor.try_push_demolish(&demo, &frame(), 401);
    assert_eq!(processor.demolishes.len(), 1);
    processor.try_push_demolish(&demo, &frame(), 401);
    assert_eq!(processor.demolishes.len(), 1);
}

#[test]
fn identical_raw_demolitions_with_different_players_are_distinct() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    processor.demolishes.clear();
    let demo = demo(&processor);
    processor.try_push_demolish(&demo, &frame(), 400);
    let attacker_actor = processor.car_to_player[&demo.attacker_actor_id()];
    let replacement = boxcars::RemoteId::Steam(u64::MAX);
    processor.insert_player_actor_id(replacement.clone(), attacker_actor);
    processor.try_push_demolish(&demo, &frame(), 401);
    assert_eq!(processor.demolishes.len(), 2);
    assert_eq!(processor.demolishes[1].attacker, replacement);
}

#[test]
fn resolved_duplicates_ignore_car_aliases_but_preserve_new_impulses_and_windows() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    processor.demolishes.clear();
    let mut demo = demo(&processor);
    processor.try_push_demolish(&demo, &frame(), 400);
    let alias = boxcars::ActorId(10000);
    processor
        .car_to_player
        .insert(alias, processor.car_to_player[&demo.attacker_actor_id()]);
    let DemolishAttribute::Extended(ref mut extended) = demo else {
        unreachable!()
    };
    extended.attacker.actor = alias;
    processor.try_push_demolish(&demo, &frame(), 401);
    assert_eq!(processor.demolishes.len(), 1);
    processor.try_push_demolish(&demo, &frame(), 400 + MAX_DEMOLISH_KNOWN_FRAMES_PASSED);
    assert_eq!(processor.demolishes.len(), 2);
    let DemolishAttribute::Extended(ref mut extended) = demo else {
        unreachable!()
    };
    extended.attacker_velocity.x = 2.0;
    processor.try_push_demolish(&demo, &frame(), 401 + MAX_DEMOLISH_KNOWN_FRAMES_PASSED);
    assert_eq!(processor.demolishes.len(), 3);
}

thread_local! {
    static DEMOLISH_WARNINGS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

struct WarningCounter;

impl log::Log for WarningCounter {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        if record.level() == log::Level::Warn
            && record
                .args()
                .to_string()
                .starts_with("Error building demolish info")
        {
            DEMOLISH_WARNINGS.with(|count| count.set(count.get() + 1));
        }
    }

    fn flush(&self) {}
}

fn count_warnings() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        log::set_logger(&WarningCounter).unwrap();
        log::set_max_level(log::LevelFilter::Warn);
    });
    DEMOLISH_WARNINGS.with(|count| count.set(0));
}

#[test]
fn persistent_failures_warn_once_per_window_and_retry_until_resolved() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    processor.demolishes.clear();
    let demo = demo(&processor);
    let car = demo.attacker_actor_id();
    let player = processor.car_to_player.remove(&car).unwrap();
    count_warnings();
    for index in 400..400 + MAX_DEMOLISH_KNOWN_FRAMES_PASSED {
        processor.try_push_demolish(&demo, &frame(), index);
    }
    assert_eq!(DEMOLISH_WARNINGS.get(), 1);
    assert!(processor.demolishes.is_empty());
    processor.try_push_demolish(&demo, &frame(), 400 + MAX_DEMOLISH_KNOWN_FRAMES_PASSED);
    assert_eq!(DEMOLISH_WARNINGS.get(), 2);
    processor.car_to_player.insert(car, player);
    processor.try_push_demolish(&demo, &frame(), 401 + MAX_DEMOLISH_KNOWN_FRAMES_PASSED);
    assert_eq!(processor.demolishes.len(), 1);
    assert!(processor.failed_demolishes.is_empty());
    processor.car_to_player.remove(&car);
    processor.try_push_demolish(&demo, &frame(), 402 + MAX_DEMOLISH_KNOWN_FRAMES_PASSED);
    assert_eq!(DEMOLISH_WARNINGS.get(), 3);
    processor.reset();
    assert!(processor.failed_demolishes.is_empty());
}

#[test]
fn resolved_duplicate_does_not_rebuild_missing_victim_location() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    processor.demolishes.clear();
    let demo = demo(&processor);
    processor.try_push_demolish(&demo, &frame(), 400);
    assert_eq!(processor.demolishes.len(), 1);
    processor
        .actor_state
        .actor_states
        .remove(&demo.victim_actor_id());
    processor.actor_state.recently_deleted_actor_states.clear();
    assert!(processor.build_demolish_info(&demo, &frame(), 401).is_err());
    count_warnings();
    processor.try_push_demolish(&demo, &frame(), 401);
    assert_eq!(DEMOLISH_WARNINGS.get(), 0);
    assert!(processor.failed_demolishes.is_empty());
    assert_eq!(processor.demolishes.len(), 1);
}

#[test]
fn missing_victim_state_is_retried_when_the_state_arrives() {
    let replay = replay();
    let mut processor = ReplayProcessor::new(&replay).unwrap();
    processor.process_long_enough_to_get_actor_ids().unwrap();
    processor.demolishes.clear();
    let demo = demo(&processor);
    let victim = demo.victim_actor_id();
    let state = processor.actor_state.actor_states.remove(&victim).unwrap();
    processor.actor_state.recently_deleted_actor_states.clear();
    processor.try_push_demolish(&demo, &frame(), 400);
    assert!(processor.demolishes.is_empty());
    processor.actor_state.actor_states.insert(victim, state);
    processor.try_push_demolish(&demo, &frame(), 401);
    assert_eq!(processor.demolishes.len(), 1);
    assert!(processor.failed_demolishes.is_empty());
}
