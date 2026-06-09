//! Regression test for contested 50/50 kickoff touch attribution.
//!
//! On a contested kickoff both cars reach the ball at essentially the same
//! instant, but the losing challenger's contact lands a frame off the ball's
//! measurable trajectory deviation (the winning car already redirected it), and
//! the replay frequently reports only the winning team's `BallHitTeamNum` marker.
//! Previously the challenger was credited no touch at all: `expected_taker_by_team`
//! had no first-touch tiebreak, the taker fell back to an unrelated geometric
//! heuristic, and `kickoff_type`/`kickoff_direction` collapsed to `unknown`.
//!
//! `assets/post-eac-ranked-doubles-2026-04-28.replay` contains two such kickoffs
//! (verified frame-by-frame): `Ragnar` reaches the ball a frame before the
//! deviation while only blue's marker fires, and `2Fum2Tastic` reaches it a frame
//! after the deviation with no orange marker at all. Both challengers must now be
//! credited a touch, which resolves the kickoff type for both teams.

mod common;

use subtr_actor::{EventPayload, KickoffEvent, KickoffType, PlayerId, StatsTimelineCollector};

const DOUBLES_REPLAY: &str = "assets/post-eac-ranked-doubles-2026-04-28.replay";

struct ContestedKickoffCase {
    /// Inclusive window (seconds) used to locate the kickoff regardless of small
    /// timing shifts.
    start_time_range: (f32, f32),
    /// The losing challenger who previously lost their touch on this 50/50.
    challenger_name: &'static str,
}

const CONTESTED_KICKOFFS: [ContestedKickoffCase; 2] = [
    ContestedKickoffCase {
        start_time_range: (84.0, 87.0),
        challenger_name: "Ragnar",
    },
    ContestedKickoffCase {
        start_time_range: (153.0, 156.0),
        challenger_name: "2Fum2Tastic",
    },
];

#[test]
#[ignore = "replay-backed kickoff touch parity is slow; run explicitly when changing touch attribution"]
fn contested_kickoff_credits_both_fifty_fifty_challengers() {
    let replay = common::parse_replay(DOUBLES_REPLAY);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("expected a stats timeline for the doubles replay");

    let name_of = |player_id: &PlayerId| -> Option<String> {
        timeline
            .replay_meta
            .player_order()
            .find(|player| &player.remote_id == player_id)
            .map(|player| player.name.clone())
    };

    let kickoffs: Vec<&KickoffEvent> = timeline
        .events
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::Kickoff(kickoff) => Some(kickoff.as_ref()),
            _ => None,
        })
        .collect();
    assert!(
        !kickoffs.is_empty(),
        "expected kickoff events in the replay"
    );

    for case in &CONTESTED_KICKOFFS {
        let (lo, hi) = case.start_time_range;
        let kickoff = kickoffs
            .iter()
            .find(|kickoff| kickoff.start_time >= lo && kickoff.start_time <= hi)
            .unwrap_or_else(|| panic!("expected a kickoff with start_time in [{lo}, {hi}]"));

        let team_zero_taker = kickoff
            .team_zero_taker
            .as_ref()
            .expect("contested kickoff should resolve a team-zero taker");
        let team_one_taker = kickoff
            .team_one_taker
            .as_ref()
            .expect("contested kickoff should resolve a team-one taker");

        // Both sides of the 50/50 must be credited a kickoff touch.
        assert!(
            team_zero_taker.first_touch_time.is_some(),
            "team-zero taker should be credited a touch on the {} kickoff",
            case.challenger_name
        );
        assert!(
            team_one_taker.first_touch_time.is_some(),
            "the contesting challenger {} should be credited a touch (was dropped before the fix)",
            case.challenger_name
        );

        // Once both challengers are credited, the kickoff type/direction resolve
        // instead of collapsing to `unknown`.
        assert_ne!(
            kickoff.kickoff_type,
            KickoffType::Unknown,
            "kickoff type should resolve for the {} contested kickoff",
            case.challenger_name
        );

        // The named challenger is one of the credited takers.
        let taker_names: Vec<Option<String>> = vec![
            name_of(&team_zero_taker.player),
            name_of(&team_one_taker.player),
        ];
        assert!(
            taker_names
                .iter()
                .flatten()
                .any(|name| name == case.challenger_name),
            "expected {} to be a credited kickoff taker, got {:?}",
            case.challenger_name,
            taker_names
        );
    }
}
