use subtr_actor::CeilingShotCalculator;

use super::candidate::{confidence_pct, event_json, MechanicCandidate};
use super::players::player_id_string;

pub(crate) fn push_ceiling_shot_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<CeilingShotCalculator>() else {
        return;
    };
    candidates.extend(calculator.events().iter().map(|event| {
        let confidence = event.confidence;
        MechanicCandidate {
            mechanic: "ceiling_shot",
            mechanic_label: "Ceiling Shot",
            detector: "builtin:ceiling_shot",
            player_id: Some(player_id_string(&event.player)),
            is_team_0: Some(event.is_team_0),
            event_time: event.time,
            event_frame: event.frame,
            start_time: event.ceiling_contact_time,
            end_time: event.time,
            confidence: Some(confidence),
            reason: format!(
                "{}% confidence; touch {:.2}s after ceiling; ball speed +{:.0}",
                confidence_pct(confidence),
                event.time_since_ceiling_contact,
                event.ball_speed_change
            ),
            event: event_json(event),
        }
    }));
}
