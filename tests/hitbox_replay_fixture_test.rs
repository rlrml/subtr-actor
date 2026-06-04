mod common;

use subtr_actor::{car_hitbox_for_body_id, ReplayProcessor};

const HITBOX_FIXTURES: &[&str] = &[
    "assets/post-eac-ranked-duel-2026-04-28-a.replay",
    "assets/post-eac-ranked-duel-2026-04-28-b.replay",
    "assets/post-eac-ranked-doubles-2026-04-28.replay",
    "assets/post-eac-ranked-standard-2026-04-28.replay",
    "assets/post-eac-private-2026-04-28.replay",
    "assets/recent-ranked-doubles-2026-03-10.replay",
    "assets/recent-ranked-standard-2026-03-10-a.replay",
    "assets/recent-ranked-standard-2026-03-10-b.replay",
    "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g1.replay",
    "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g2.replay",
    "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g3.replay",
    "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g4.replay",
    "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
];

#[test]
fn fixture_player_loadout_body_ids_are_threaded_to_player_meta() {
    let mut checked_players = 0usize;
    let mut missing_body_ids = Vec::new();
    let mut unknown_body_ids = std::collections::BTreeSet::new();
    let mut resolved_body_ids = std::collections::BTreeSet::new();

    for fixture in HITBOX_FIXTURES {
        let replay = common::parse_replay(fixture);
        let mut processor =
            ReplayProcessor::new(&replay).unwrap_or_else(|error| panic!("{fixture}: {error:?}"));
        let meta = processor
            .process_and_get_replay_meta()
            .unwrap_or_else(|error| panic!("{fixture}: {error:?}"));

        for player in meta.player_order() {
            checked_players += 1;
            let Some(body_id) = player.car_body_id else {
                missing_body_ids.push(format!("{fixture}: {}", player.name));
                continue;
            };

            match car_hitbox_for_body_id(body_id) {
                Some(hitbox) => {
                    resolved_body_ids.insert((body_id, format!("{:?}", hitbox.family)));
                    assert_eq!(
                        player.car_hitbox_family.as_deref(),
                        Some(format!("{:?}", hitbox.family).as_str()),
                        "{fixture}: {} body id {body_id}",
                        player.name
                    );
                }
                None => {
                    unknown_body_ids.insert(body_id);
                    assert_eq!(
                        player.car_hitbox_family, None,
                        "{fixture}: {} body id {body_id}",
                        player.name
                    );
                }
            }
        }
    }

    assert!(
        checked_players > 0,
        "expected to inspect at least one replay player"
    );
    assert!(
        missing_body_ids.is_empty(),
        "missing car_body_id values:\n{}",
        missing_body_ids.join("\n")
    );
    assert_eq!(
        resolved_body_ids,
        std::collections::BTreeSet::from([
            (23, "Octane".to_string()),
            (25, "Octane".to_string()),
            (403, "Dominus".to_string()),
            (4284, "Octane".to_string()),
            (4770, "Dominus".to_string()),
            (11315, "Dominus".to_string()),
        ])
    );
    assert_eq!(
        unknown_body_ids,
        std::collections::BTreeSet::new(),
        "unexpected unknown body ids"
    );
}
