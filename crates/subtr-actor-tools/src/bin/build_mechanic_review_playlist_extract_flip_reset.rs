use anyhow::anyhow;
use subtr_actor::{Collector, DodgeResetCalculator, FlipResetTracker};

use super::candidate::{confidence_pct, event_json, MechanicCandidate};
use super::players::player_id_string;

pub(crate) fn push_flip_reset_candidates(
    replay: &boxcars::Replay,
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) -> anyhow::Result<()> {
    let tracker = FlipResetTracker::new()
        .process_replay(replay)
        .map_err(|err| anyhow!("failed to collect flip reset tracker events: {err:?}"))?;
    candidates.extend(tracker.flip_reset_events().iter().map(|event| {
        let confidence = event.confidence;
        MechanicCandidate {
            mechanic: "flip_reset",
            mechanic_label: "Flip Reset",
            detector: "builtin:flip_reset_tracker",
            player_id: Some(player_id_string(&event.player)),
            is_team_0: Some(event.is_team_0),
            event_time: event.time,
            event_frame: event.frame,
            start_time: event.time,
            end_time: event.time,
            confidence: Some(confidence),
            reason: format!(
                "{}% confidence; underside ball contact; closest approach {:.0}uu",
                confidence_pct(confidence),
                event.closest_approach_distance
            ),
            event: event_json(event),
        }
    }));

    if let Some(calculator) = graph.state::<DodgeResetCalculator>() {
        candidates.extend(
            calculator
                .on_ball_events()
                .iter()
                .map(|event| MechanicCandidate {
                    mechanic: "flip_reset",
                    mechanic_label: "Flip Reset",
                    detector: "builtin:dodge_reset:on_ball",
                    player_id: Some(player_id_string(&event.player)),
                    is_team_0: Some(event.is_team_0),
                    event_time: event.time,
                    event_frame: event.frame,
                    start_time: event.time,
                    end_time: event.time,
                    confidence: None,
                    reason: format!(
                        "dodge refresh while close to the ball; counter value {}",
                        event.counter_value
                    ),
                    event: event_json(event),
                }),
        );
    }
    Ok(())
}
