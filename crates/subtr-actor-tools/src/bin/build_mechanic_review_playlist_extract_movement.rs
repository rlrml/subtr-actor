use subtr_actor::{HalfFlipCalculator, SpeedFlipCalculator, WavedashCalculator};

use super::candidate::{confidence_pct, event_json, MechanicCandidate};
use super::players::player_id_string;

pub(crate) fn push_speed_flip_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<SpeedFlipCalculator>() else {
        return;
    };
    candidates.extend(calculator.events().iter().map(|event| {
        let confidence = event.confidence;
        MechanicCandidate {
            mechanic: "speed_flip",
            mechanic_label: "Speed Flip",
            detector: "builtin:speed_flip",
            player_id: Some(player_id_string(&event.player)),
            is_team_0: Some(event.is_team_0),
            event_time: event.time,
            event_frame: event.frame,
            start_time: (event.time - 0.5).max(0.0),
            end_time: event.time,
            confidence: Some(confidence),
            reason: format!(
                "{}% confidence; max speed {:.0}; diagonal {:.2}; cancel {:.2}",
                confidence_pct(confidence),
                event.max_speed,
                event.diagonal_score,
                event.cancel_score
            ),
            event: event_json(event),
        }
    }));
}

pub(crate) fn push_half_flip_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<HalfFlipCalculator>() else {
        return;
    };
    candidates.extend(calculator.events().iter().map(|event| {
        let confidence = event.confidence;
        MechanicCandidate {
            mechanic: "half_flip",
            mechanic_label: "Half Flip",
            detector: "builtin:half_flip",
            player_id: Some(player_id_string(&event.player)),
            is_team_0: Some(event.is_team_0),
            event_time: event.time,
            event_frame: event.frame,
            start_time: (event.time - 0.65).max(0.0),
            end_time: event.time,
            confidence: Some(confidence),
            reason: format!(
                "{}% confidence; backward {:.2}; reorientation {:.2}; speed delta {:+.0}",
                confidence_pct(confidence),
                event.start_backward_alignment,
                event.best_reorientation_alignment,
                event.end_speed - event.start_speed
            ),
            event: event_json(event),
        }
    }));
}

pub(crate) fn push_wavedash_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<WavedashCalculator>() else {
        return;
    };
    candidates.extend(calculator.events().iter().map(|event| {
        let confidence = event.confidence;
        MechanicCandidate {
            mechanic: "wavedash",
            mechanic_label: "Wavedash",
            detector: "builtin:wavedash",
            player_id: Some(player_id_string(&event.player)),
            is_team_0: Some(event.is_team_0),
            event_time: event.time,
            event_frame: event.frame,
            start_time: event.dodge_time,
            end_time: event.time,
            confidence: Some(confidence),
            reason: format!(
                "{}% confidence; landing {:.2}s after dodge; speed gain {:.0}",
                confidence_pct(confidence),
                event.time_since_dodge,
                event.horizontal_speed_gain
            ),
            event: event_json(event),
        }
    }));
}
