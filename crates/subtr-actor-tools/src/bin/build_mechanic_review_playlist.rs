use anyhow::anyhow;
use serde_json::json;
use subtr_actor::{
    playlist_generation::{PlaybackBound, PlaybackBoundKind, PlaylistManifestItem},
    stats::analysis_graph::collect_builtin_analysis_graph_for_replay,
    CeilingShotCalculator, Collector, DodgeResetCalculator, FlipResetTracker, HalfFlipCalculator,
    ReplayProcessor, SpeedFlipCalculator, WavedashCalculator,
};

#[path = "build_mechanic_review_playlist_args.rs"]
mod args;
#[path = "build_mechanic_review_playlist_args_default.rs"]
mod args_default;
#[path = "build_mechanic_review_playlist_args_query.rs"]
mod args_query;
#[path = "build_mechanic_review_playlist_candidate.rs"]
mod candidate;
#[path = "build_mechanic_review_playlist_config.rs"]
mod config;
#[path = "build_mechanic_review_playlist_constants.rs"]
mod constants;
#[path = "build_mechanic_review_playlist_extract_flick.rs"]
mod extract_flick;
#[path = "build_mechanic_review_playlist_extract_touch.rs"]
mod extract_touch;
#[path = "build_mechanic_review_playlist_goal_scan.rs"]
mod goal_scan;
#[path = "build_mechanic_review_playlist_manifest.rs"]
mod manifest;
#[path = "build_mechanic_review_playlist_mechanics.rs"]
mod mechanics;
#[path = "build_mechanic_review_playlist_players.rs"]
mod players;
#[path = "build_mechanic_review_playlist_source_api.rs"]
mod source_api;
#[path = "build_mechanic_review_playlist_source_ballchasing.rs"]
mod source_ballchasing;
#[path = "build_mechanic_review_playlist_source_collect.rs"]
mod source_collect;
#[path = "build_mechanic_review_playlist_source_ids.rs"]
mod source_ids;
#[path = "build_mechanic_review_playlist_source_parse.rs"]
mod source_parse;
#[path = "build_mechanic_review_playlist_source_types.rs"]
mod source_types;

use candidate::{
    confidence_pct, enforce_min_clip_duration, event_json, followup_goal_for_candidate,
    include_candidate, replay_duration_seconds, MechanicCandidate,
};
use config::{parse_args, Config};
use extract_flick::{push_flick_candidates, push_musty_flick_candidates};
use extract_touch::{
    push_air_dribble_candidates, push_double_tap_candidates, push_one_timer_candidates,
};
use goal_scan::GoalScanCollector;
use manifest::{build_manifest, write_manifest};
use mechanics::graph_node_names_for_mechanics;
use players::{player_display_map, player_id_string, player_team_label};
use source_types::ReplaySourceInput;

fn extract_candidates(
    replay: &boxcars::Replay,
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    mechanics: &[&str],
    config: &Config,
) -> anyhow::Result<Vec<MechanicCandidate>> {
    let mut candidates = Vec::new();

    for mechanic in mechanics {
        match *mechanic {
            "flick" => {
                push_flick_candidates(graph, &mut candidates);
            }
            "musty_flick" => {
                push_musty_flick_candidates(graph, &mut candidates);
            }
            "one_timer" => {
                push_one_timer_candidates(graph, &mut candidates);
            }
            "air_dribble" => {
                push_air_dribble_candidates(graph, &mut candidates);
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
                    candidates.extend(calculator.on_ball_events().iter().map(|event| {
                        MechanicCandidate {
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
                        }
                    }));
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
                push_double_tap_candidates(graph, &mut candidates);
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

pub(crate) fn build_items_for_source(
    source: &ReplaySourceInput,
    replay: &boxcars::Replay,
    config: &Config,
    mechanics: &[&str],
) -> anyhow::Result<Vec<PlaylistManifestItem>> {
    let graph_nodes = graph_node_names_for_mechanics(mechanics);
    let graph = collect_builtin_analysis_graph_for_replay(replay, graph_nodes).map_err(|err| {
        anyhow!(
            "failed to collect mechanic stats for {}: {err:?}",
            source.label
        )
    })?;
    let mut processor = ReplayProcessor::new(replay).map_err(|err| {
        anyhow!(
            "failed to build replay processor for {}: {err:?}",
            source.label
        )
    })?;
    processor.process(&mut GoalScanCollector).map_err(|err| {
        anyhow!(
            "failed to process replay goals for {}: {err:?}",
            source.label
        )
    })?;
    let replay_meta = processor.get_replay_meta().map_err(|err| {
        anyhow!(
            "failed to read replay metadata for {}: {err:?}",
            source.label
        )
    })?;
    let replay_duration = replay_duration_seconds(replay);
    let players = player_display_map(&replay_meta);
    let candidates = extract_candidates(replay, &graph, mechanics, config)?;

    let mut items = Vec::new();
    for candidate in candidates {
        let player = candidate
            .player_id
            .as_ref()
            .and_then(|player_id| players.get(player_id));
        let player_label = player
            .map(|display| display.name.as_str())
            .or(candidate.player_id.as_deref())
            .unwrap_or("team event");
        let start_time = (candidate.start_time - config.before_seconds).max(0.0);
        let followup_goal = followup_goal_for_candidate(&candidate, &processor.goal_events, config);
        let padded_end_time = followup_goal
            .map(|goal| goal.time + config.goal_tail_seconds)
            .unwrap_or(candidate.end_time + config.after_seconds)
            .max(candidate.end_time + config.after_seconds)
            .max(start_time);
        let (start_time, end_time) = enforce_min_clip_duration(
            start_time,
            padded_end_time,
            replay_duration,
            config.min_clip_seconds,
        );
        let score = candidate
            .confidence
            .map(|confidence| format!(" {}%", confidence_pct(confidence)))
            .unwrap_or_default();
        let id = format!(
            "{}:{}:{}:{}",
            candidate.mechanic,
            source.source_id,
            candidate.event_frame,
            candidate.player_id.as_deref().unwrap_or("team")
        );

        items.push(PlaylistManifestItem {
            id: id.clone(),
            replay: source.source_id.clone(),
            start: PlaybackBound {
                kind: PlaybackBoundKind::Time,
                value: start_time,
            },
            end: PlaybackBound {
                kind: PlaybackBoundKind::Time,
                value: end_time,
            },
            label: format!("{}{score} - {player_label}", candidate.mechanic_label),
            meta: json!({
                "itemId": id,
                "mechanic": candidate.mechanic,
                "mechanicLabel": candidate.mechanic_label,
                "detector": candidate.detector,
                "confidence": candidate.confidence,
                "reason": candidate.reason,
                "playerId": candidate.player_id,
                "playerName": player.map(|display| display.name.clone()),
                "team": player.map(|display| display.team).or_else(|| candidate.is_team_0.map(player_team_label)),
                "target": {
                    "kind": "player-span",
                    "playerId": candidate.player_id,
                    "startTime": start_time,
                    "endTime": end_time,
                    "mechanicStartTime": candidate.start_time,
                    "mechanicEndTime": candidate.end_time,
                    "eventTime": candidate.event_time,
                    "eventFrame": candidate.event_frame,
                    "goalTime": followup_goal.map(|goal| goal.time),
                    "goalFrame": followup_goal.map(|goal| goal.frame),
                },
                "followupGoal": followup_goal.map(event_json),
                "event": candidate.event,
            }),
        });
    }

    Ok(items)
}

fn main() -> anyhow::Result<()> {
    let config = parse_args()?;
    let manifest = build_manifest(&config)?;
    write_manifest(&manifest, config.output.as_deref())?;
    eprintln!(
        "wrote {} mechanic candidates across {} replays",
        manifest.items.len(),
        manifest.replays.len()
    );
    Ok(())
}
