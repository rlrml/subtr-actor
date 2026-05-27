use subtr_actor::{BallCarryCalculator, BallCarryKind, DoubleTapCalculator, OneTimerCalculator};

use super::candidate::{event_json, MechanicCandidate};
use super::players::player_id_string;

pub(crate) fn push_one_timer_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<OneTimerCalculator>() else {
        return;
    };
    candidates.extend(calculator.events().iter().map(|event| MechanicCandidate {
        mechanic: "one_timer",
        mechanic_label: "One Timer",
        detector: "builtin:one_timer",
        player_id: Some(player_id_string(&event.player)),
        is_team_0: Some(event.is_team_0),
        event_time: event.time,
        event_frame: event.frame,
        start_time: event.pass_start_time,
        end_time: event.time,
        confidence: None,
        reason: format!(
            "pass from {}; {:.1}s pass, {:.0}uu travel, {:.0}uu/s shot, {:.2} goal alignment",
            player_id_string(&event.passer),
            event.pass_duration,
            event.pass_travel_distance,
            event.ball_speed,
            event.goal_alignment
        ),
        event: event_json(event),
    }));
}

pub(crate) fn push_air_dribble_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<BallCarryCalculator>() else {
        return;
    };
    candidates.extend(
        calculator
            .carry_events()
            .iter()
            .filter(|event| event.kind == BallCarryKind::AirDribble)
            .map(|event| MechanicCandidate {
                mechanic: "air_dribble",
                mechanic_label: "Air Dribble",
                detector: "builtin:ball_carry",
                player_id: Some(player_id_string(&event.player_id)),
                is_team_0: Some(event.is_team_0),
                event_time: event.end_time,
                event_frame: event.end_frame,
                start_time: event.start_time,
                end_time: event.end_time,
                confidence: None,
                reason: format!(
                    "{:.1}s airborne control; {:.0}uu path; avg gap {:.0}h/{:.0}v",
                    event.duration,
                    event.path_distance,
                    event.average_horizontal_gap,
                    event.average_vertical_gap
                ),
                event: event_json(event),
            }),
    );
}

pub(crate) fn push_double_tap_candidates(
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    candidates: &mut Vec<MechanicCandidate>,
) {
    let Some(calculator) = graph.state::<DoubleTapCalculator>() else {
        return;
    };
    candidates.extend(calculator.events().iter().map(|event| MechanicCandidate {
        mechanic: "double_tap",
        mechanic_label: "Double Tap",
        detector: "builtin:double_tap",
        player_id: Some(player_id_string(&event.player)),
        is_team_0: Some(event.is_team_0),
        event_time: event.time,
        event_frame: event.frame,
        start_time: event.backboard_time,
        end_time: event.time,
        confidence: None,
        reason: format!(
            "same-player touch {:.2}s after backboard bounce",
            event.time - event.backboard_time
        ),
        event: event_json(event),
    }));
}
