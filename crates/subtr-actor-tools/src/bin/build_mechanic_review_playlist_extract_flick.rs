use subtr_actor::{FlickCalculator, MustyFlickCalculator};

use super::candidate::{confidence_pct, event_json, MechanicCandidate};
use super::players::player_id_string;

pub(crate) fn push_flick_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<FlickCalculator>() else {
        return;
    };
    candidates.extend(calculator.events().iter().map(|event| {
        let confidence = event.confidence;
        MechanicCandidate {
            mechanic: "flick",
            mechanic_label: "Flick",
            detector: "builtin:flick",
            player_id: Some(player_id_string(&event.player)),
            is_team_0: Some(event.is_team_0),
            event_time: event.time,
            event_frame: event.frame,
            start_time: event.setup_start_time,
            end_time: event.time,
            confidence: Some(confidence),
            reason: format!(
                "{}% confidence; {:.1}s setup with {} touches; ball speed +{:.0}",
                confidence_pct(confidence),
                event.setup_duration,
                event.setup_touch_count,
                event.ball_speed_change
            ),
            event: event_json(event),
        }
    }));
}

pub(crate) fn push_musty_flick_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<MustyFlickCalculator>() else {
        return;
    };
    candidates.extend(calculator.events().iter().map(|event| {
        let confidence = event.confidence;
        MechanicCandidate {
            mechanic: "musty_flick",
            mechanic_label: "Musty Flick",
            detector: "builtin:musty_flick",
            player_id: Some(player_id_string(&event.player)),
            is_team_0: Some(event.is_team_0),
            event_time: event.time,
            event_frame: event.frame,
            start_time: event.dodge_time,
            end_time: event.time,
            confidence: Some(confidence),
            reason: format!(
                "{}% confidence; dodge-to-touch {:.2}s; pitch rate {:.1}; ball speed +{:.0}",
                confidence_pct(confidence),
                event.time_since_dodge,
                event.pitch_rate,
                event.ball_speed_change
            ),
            event: event_json(event),
        }
    }));
}
