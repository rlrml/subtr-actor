//! Regression test for a flip kickoff misclassified as `BoostIntoBall`.
//!
//! In the first kickoff of replay `019eb428-4ea3-7fa1-a5b8-4cf01115684e`,
//! XXGerar51 side-flips on approach (dodge at ~17.48s: forward component
//! ~0.05, side component ~0.98) and then dodges a second time into the ball
//! at first touch (~18.93s). `observe_player_approach` recorded
//! `first_dodge_time` with `get_or_insert` but overwrote the dodge direction
//! components unconditionally, so the second dodge's direction (measured
//! nearly anti-parallel to the nose at ball contact) replaced the kickoff
//! flip's. Both classification thresholds then failed and, with boost active,
//! `classify_approach` returned `BoostIntoBall`. The fix captures the dodge
//! direction only for the first dodge, letting the side flip classify as
//! `DiagonalFlip` through the normal threshold path.

mod common;

use subtr_actor::{
    clip_replay_around_times, EventPayload, KickoffApproach, PlayerId, StatsTimelineCollector,
};

const REPLAY: &str = "assets/boost-into-ball-misclassification-2026-06-11.replay";

/// The first kickoff starts at ~13.94 s in this replay.
const FIRST_KICKOFF_START: f32 = 13.94;

#[test]
fn clip_flip_kickoff_not_classified_as_boost_into_ball() {
    let replay = common::parse_replay(REPLAY);

    let clip = clip_replay_around_times(&replay, 0.0, FIRST_KICKOFF_START + 8.0, 0, 90)
        .expect("clip should build");

    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&clip.to_replay())
        .expect("expected a stats timeline for the clip");

    let xxgerar51: Option<&PlayerId> = timeline
        .replay_meta
        .player_order()
        .find(|p| p.name.eq_ignore_ascii_case("xxgerar51"))
        .map(|p| &p.remote_id);
    let xxgerar51 = xxgerar51.expect("XXGerar51 should appear in the replay");

    let kickoffs: Vec<_> = timeline
        .events
        .events
        .iter()
        .filter_map(|e| match &e.payload {
            EventPayload::Kickoff(k) => Some(k.as_ref()),
            _ => None,
        })
        .collect();

    let first_kickoff = kickoffs
        .iter()
        .find(|k| (k.start_time - FIRST_KICKOFF_START).abs() < 2.0)
        .unwrap_or_else(|| {
            panic!(
                "expected a kickoff near t={FIRST_KICKOFF_START}; found: {:?}",
                kickoffs.iter().map(|k| k.start_time).collect::<Vec<_>>()
            )
        });

    let taker = first_kickoff
        .team_one_taker
        .as_ref()
        .expect("team-one taker should be resolved for the first kickoff");

    assert_eq!(
        &taker.player, xxgerar51,
        "expected XXGerar51 to be the team-one taker"
    );

    assert_eq!(
        taker.approach,
        KickoffApproach::DiagonalFlip,
        "XXGerar51 side-flipped on the first kickoff and should classify as \
         DiagonalFlip, not {:?}",
        taker.approach
    );
}
