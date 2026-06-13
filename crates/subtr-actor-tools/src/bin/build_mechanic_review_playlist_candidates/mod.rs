use anyhow::anyhow;
use subtr_actor::{
    BallCarryCalculator, BallCarryKind, CeilingShotCalculator, Collector, DodgeResetCalculator,
    DoubleTapCalculator, FlickCalculator, FlipResetTracker, HalfFlipCalculator,
    MustyFlickCalculator, OneTimerCalculator, SpeedFlipCalculator, WavedashCalculator,
    stats::analysis_graph::AnalysisGraph,
};

use super::{Config, MechanicCandidate, event_json, player_id_string};

fn confidence_pct(confidence: f32) -> u32 {
    (confidence * 100.0).round().clamp(0.0, 100.0) as u32
}

fn include_candidate(candidate: &MechanicCandidate, config: &Config) -> bool {
    candidate
        .confidence
        .map(|confidence| confidence >= config.min_confidence)
        .unwrap_or(true)
}

pub(super) fn extract_candidates(
    replay: &boxcars::Replay,
    graph: &AnalysisGraph,
    mechanics: &[&str],
    config: &Config,
) -> anyhow::Result<Vec<MechanicCandidate>> {
    let mut candidates = Vec::new();

    for mechanic in mechanics {
        match *mechanic {
            "flick" => {
                let Some(calculator) = graph.state::<FlickCalculator>() else {
                    continue;
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
            "musty_flick" => {
                let Some(calculator) = graph.state::<MustyFlickCalculator>() else {
                    continue;
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
            "one_timer" => {
                let Some(calculator) = graph.state::<OneTimerCalculator>() else {
                    continue;
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
            "air_dribble" => {
                let Some(calculator) = graph.state::<BallCarryCalculator>() else {
                    continue;
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
            "flip_reset" => {
                let tracker = FlipResetTracker::new()
                    .process_replay(replay)
                    .map_err(|err| {
                        anyhow!("failed to collect flip reset tracker events: {err:?}")
                    })?;
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
                            .events()
                            .iter()
                            .filter(|event| event.on_ball)
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
            }
            "ceiling_shot" => {
                let Some(calculator) = graph.state::<CeilingShotCalculator>() else {
                    continue;
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
            "double_tap" => {
                let Some(calculator) = graph.state::<DoubleTapCalculator>() else {
                    continue;
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
            "speed_flip" => {
                let Some(calculator) = graph.state::<SpeedFlipCalculator>() else {
                    continue;
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
            "half_flip" => {
                let Some(calculator) = graph.state::<HalfFlipCalculator>() else {
                    continue;
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
            "wavedash" => {
                let Some(calculator) = graph.state::<WavedashCalculator>() else {
                    continue;
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
            _ => {}
        }
    }

    candidates.retain(|candidate| include_candidate(candidate, config));
    candidates.sort_by(|left, right| {
        left.start_time
            .total_cmp(&right.start_time)
            .then_with(|| left.mechanic.cmp(right.mechanic))
            .then_with(|| left.event_frame.cmp(&right.event_frame))
    });
    Ok(candidates)
}
