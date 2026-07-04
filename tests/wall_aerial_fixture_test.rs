//! Whole-replay fixtures for wall-aerial detection.
//!
//! Both replays were false-positive reports of the same species: a normal
//! aerial jumped from the **floor near** a wall read as a wall aerial, because
//! "on the wall" was a loose position band (|x| >= 3600 against a wall at
//! 4096) that airborne cars satisfied. The detector now requires genuine
//! surface contact — proximity to the wall plane/corner arc plus the car roof
//! leaning into the field — so those false positives are gone while the real
//! wall aerials remain. See `src/stats/calculators/wall_aerial.rs`.

use subtr_actor::{
    EventPayload, StatsTimelineEventCollector, WallAerialEvent, wall_outward_normal_and_distance,
};

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn wall_aerial_events(replay: &boxcars::Replay) -> Vec<WallAerialEvent> {
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(replay)
        .expect("stats timeline should build");
    timeline
        .events
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::WallAerial(event) => Some(event.clone()),
            _ => None,
        })
        .collect()
}

/// Every recorded wall contact must sit on an actual wall surface: within the
/// contact distance mirrored from `wall_aerial.rs`
/// (`WALL_SURFACE_CONTACT_MAX_DISTANCE`), at wall-contact height.
fn assert_contacts_are_on_the_wall(events: &[WallAerialEvent]) {
    for event in events {
        let [x, y, z] = event.wall_contact_position;
        let (_, wall_distance) = wall_outward_normal_and_distance(glam::Vec3::new(x, y, z));
        assert!(
            z >= 120.0 && wall_distance <= 60.0,
            "wall aerial at {:.3}s (team0={}) was not launched from the wall: \
             contact={:?} is {wall_distance:.0}uu off the surface",
            event.time,
            event.is_team_0,
            event.wall_contact_position,
        );
    }
}

fn assert_no_event_near(events: &[WallAerialEvent], lo: f32, hi: f32, description: &str) {
    assert!(
        !events.iter().any(|e| (lo..=hi).contains(&e.time)),
        "the {description} floor-launch false positive reappeared: {:#?}",
        events
            .iter()
            .filter(|e| (lo..=hi).contains(&e.time))
            .collect::<Vec<_>>(),
    );
}

#[test]
#[ignore = "whole-replay fixture (run with `--ignored`): processes a full ~6-minute \
            replay (~18s). Asserts both reported floor-launch false positives are gone, every \
            wall aerial is launched from the wall, and the genuine wall aerials remain."]
fn wall_aerials_require_real_wall_contact_and_keep_the_genuine_ones() {
    // Reviewed ground truth: five genuine wall aerials.
    //
    // The loose-band detector reported two extra floor-launched aerials — one by
    // VantaTV2 (~0:45.8 of viewer time, "wall contact" logged at |x|=3267,
    // ~800uu off the wall) and one by Tapatio1776 (~3:11, a ground jump in the
    // corner after riding the back wall down, never closer than ~266uu to the
    // wall during the climb).
    let replay = parse_replay("assets/wall-aerial-false-positive-2026-06-26.replay");
    let events = wall_aerial_events(&replay);

    assert_eq!(
        events.len(),
        5,
        "expected 5 wall aerials, got {}: {:#?}",
        events.len(),
        events
            .iter()
            .map(|e| (e.time, e.is_team_0, e.wall, e.wall_contact_position))
            .collect::<Vec<_>>(),
    );
    assert_contacts_are_on_the_wall(&events);
    // Viewer time ~0:45.8 is ~56.5s of absolute processing time.
    assert_no_event_near(&events, 54.0, 59.0, "VantaTV2 (~0:45.8)");
    assert_no_event_near(&events, 199.0, 204.0, "Tapatio1776 (~3:11)");

    // The two genuine VantaTV2 wall aerials must remain: the one that ends ~1:27
    // and the one ~4:30 into the game.
    let has_team0_aerial_near = |lo: f32, hi: f32| {
        events
            .iter()
            .any(|e| e.is_team_0 && (lo..=hi).contains(&e.time))
    };
    assert!(
        has_team0_aerial_near(97.0, 99.5),
        "missing the genuine VantaTV2 wall aerial around 1:27",
    );
    assert!(
        has_team0_aerial_near(279.0, 282.0),
        "missing the genuine VantaTV2 wall aerial around 4:30",
    );
}

#[test]
#[ignore = "whole-replay fixture (run with `--ignored`): processes a full ~9-minute \
            replay (~25s). Asserts the reported ground-jump aerials near the wall are not \
            wall aerials while the genuine wall rides remain."]
fn ground_jump_aerials_near_the_wall_are_not_wall_aerials() {
    // Reviewed ground truth: eight genuine wall aerials.
    //
    // The loose-band detector reported two extra: colonelpanic8 (~1:47 of
    // viewer time) rode the left wall, *landed*, and then jumped an aerial from
    // the floor whose climb spent >0.3s inside the |x| >= 3600 band (never
    // closer than ~430uu to the wall); Dizzle (~4:07) did the same in a corner.
    // The genuine "Ur on ..." wall ride at ~1:54 also depends on the
    // takeoff-to-touch window measuring from the true wall-leave frame.
    let replay = parse_replay("assets/wall-aerial-ground-jump-2026-07-03.replay");
    let events = wall_aerial_events(&replay);

    assert_eq!(
        events.len(),
        8,
        "expected 8 wall aerials, got {}: {:#?}",
        events.len(),
        events
            .iter()
            .map(|e| (e.time, e.is_team_0, e.wall, e.wall_contact_position))
            .collect::<Vec<_>>(),
    );
    assert_contacts_are_on_the_wall(&events);
    // Viewer time is ~11.8s behind absolute processing time in this replay:
    // ~1:47 viewer = ~117.4s absolute, ~4:07 viewer = ~258.3s absolute.
    assert_no_event_near(&events, 115.0, 120.0, "colonelpanic8 (~1:47)");
    assert_no_event_near(&events, 256.0, 260.0, "Dizzle (~4:07)");

    // The genuine wall ride right after the first false positive must remain:
    // "Ur on ..." rides the left wall to z~650 and touches the ball ~2.3s after
    // leaving the wall (~1:54 viewer time).
    assert!(
        events
            .iter()
            .any(|e| !e.is_team_0 && (124.0..=126.0).contains(&e.time)),
        "missing the genuine wall aerial around 1:54",
    );
}
