//! Whole-replay fixture for wall-aerial detection.
//!
//! This replay was the original report: its *first* "wall aerial" was a normal
//! aerial that merely launched from the **floor near** the side wall (the car
//! was never on the wall). The detector now requires the player to genuinely
//! ride the wall surface before leaving it, so that false positive is gone while
//! the real wall aerials remain. See `src/stats/calculators/wall_aerial.rs`.

use subtr_actor::{EventPayload, StatsTimelineEventCollector, WallAerialEvent};

const REPLAY: &str = "assets/wall-aerial-false-positive-2026-06-26.replay";

/// Reviewed ground truth for this replay: six genuine wall aerials.
///
/// The pre-fix detector reported a seventh — a false positive where VantaTV2
/// launched an aerial from the floor *near* the side wall (~0:45.8 into the
/// game, "wall contact" logged at |x|=3267, ~800uu off the wall surface) — and
/// at the same time *missed* several real wall aerials, because it only tracked
/// the last-toucher's ball-control setup. Keying the setup on actual on-wall
/// presence (no ball carry required) both drops the false positive and recovers
/// the genuinely wall-launched aerials by every player.
const EXPECTED_WALL_AERIALS: usize = 6;

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

/// A position counts as "on the wall" when it is at or beyond the side-wall
/// (|x| >= 3600) or back-wall (|y| >= 5000, outside the goal mouth) contact
/// thresholds, at wall-contact height. These mirror the constants in
/// `wall_aerial.rs` (`SIDE_WALL_CONTACT_ABS_X`, `BACK_WALL_CONTACT_ABS_Y`).
fn is_on_wall(position: [f32; 3]) -> bool {
    let [x, y, z] = position;
    if z < 120.0 {
        return false;
    }
    let on_side = x.abs() >= 3600.0;
    let on_back = y.abs() >= 5000.0 && x.abs() > 900.0;
    on_side || on_back
}

#[test]
#[ignore = "whole-replay fixture (run with `--ignored`): processes a full ~6-minute \
            replay (~18s). Asserts the reported floor-launch false positive is gone, every \
            wall aerial is launched from the wall, and the genuine wall aerials remain."]
fn wall_aerials_require_real_wall_contact_and_keep_the_genuine_ones() {
    let replay = parse_replay(REPLAY);
    let events = wall_aerial_events(&replay);

    assert_eq!(
        events.len(),
        EXPECTED_WALL_AERIALS,
        "expected {EXPECTED_WALL_AERIALS} wall aerials, got {}: {:#?}",
        events.len(),
        events
            .iter()
            .map(|e| (e.time, e.is_team_0, e.wall, e.wall_contact_position))
            .collect::<Vec<_>>(),
    );

    // Core invariant the fix guarantees: every detected wall aerial actually left
    // the wall surface. The reported false positive failed exactly this — its
    // "wall contact" was logged in the air at |x|=3267, never on the wall.
    for event in &events {
        assert!(
            is_on_wall(event.wall_contact_position),
            "wall aerial at {:.3}s (team0={}) was not launched from the wall: contact={:?}",
            event.time,
            event.is_team_0,
            event.wall_contact_position,
        );
    }

    // The reported false positive happened ~0:45.8 of viewer time (~56.5s of
    // absolute processing time). No wall aerial should be detected there now.
    assert!(
        !events.iter().any(|e| (54.0..=59.0).contains(&e.time)),
        "the reported floor-launch false positive (~56.5s) reappeared: {:#?}",
        events
            .iter()
            .filter(|e| (54.0..=59.0).contains(&e.time))
            .collect::<Vec<_>>(),
    );

    // The two genuine VantaTV2 wall aerials must remain: the one that ends ~1:27
    // and the one ~4:30 into the game. (The attack-relative wall label is #259's
    // concern; here we only assert the aerials are still detected.)
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
