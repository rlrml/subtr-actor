mod common;

use std::collections::BTreeMap;

use subtr_actor::{EventPayload, StatsTimelineEventCollector};

/// Replay-backed sanity check for the enriched player_possession span stream:
/// spans exist, carry sensible enrichment, and per-player spans never overlap.
#[test]
fn player_possession_spans_are_sane_for_post_eac_doubles_replay() {
    let replay = common::parse_replay("assets/post-eac-ranked-doubles-2026-04-28.replay");
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("failed to collect stats timeline for post-EAC doubles replay");

    let spans = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::PlayerPossession(event) => Some(event),
        _ => None,
    });
    assert!(
        spans.len() >= 20,
        "expected a real match to produce many player possession spans, got {}",
        spans.len()
    );

    let mut per_player: BTreeMap<String, Vec<(usize, usize)>> = BTreeMap::new();
    for span in &spans {
        assert!(
            span.duration > 0.0,
            "possession spans must have positive possessed duration"
        );
        assert!(
            span.duration <= span.end_time - span.start_time + 1e-3,
            "possessed duration cannot exceed the span's wall-clock window"
        );
        assert!(
            span.touch_count >= 1,
            "possession requires at least one touch by the owner"
        );
        assert!(
            span.aerial_touch_count + span.wall_touch_count <= span.touch_count,
            "classified touches cannot exceed the touch count"
        );
        assert!(
            span.advance_distance >= 0.0 && span.retreat_distance >= 0.0,
            "ball movement totals are nonnegative"
        );
        assert!(
            span.carry_time + span.air_dribble_time <= span.duration + 1e-3,
            "sustained-control time cannot exceed possessed duration"
        );
        per_player
            .entry(format!("{:?}", span.player_id))
            .or_default()
            .push((span.start_frame, span.end_frame));
    }

    // The possession tracker allows one owner at a time, so a single player's
    // spans must be disjoint in frame space.
    for (player, mut windows) in per_player {
        windows.sort_unstable();
        for pair in windows.windows(2) {
            assert!(
                pair[1].0 >= pair[0].1,
                "player {player} has overlapping possession spans: {pair:?}"
            );
        }
    }

    // Touch classification events feed the career first-touch metrics; make
    // sure the stream still carries first touches for this fixture.
    let touches = common::event_payloads(&timeline, |payload| match payload {
        EventPayload::Touch(event) => Some(event),
        _ => None,
    });
    assert!(
        touches.iter().any(|touch| touch.first_touch),
        "expected at least one first touch in a real match"
    );
}
