use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use clap::Parser;
use serde::Serialize;
use subtr_actor::{GameplayPhase, PlayerId, ReplayMeta, ReplayStatsFrame, StatsTimelineCollector};

const DETECTION_WINDOW_SECONDS: f32 = 1.5;

#[derive(Debug, Serialize)]
struct Audit {
    replays: Vec<ReplayAudit>,
}

#[derive(Debug, Serialize)]
struct ReplayAudit {
    path: String,
    kickoff_count: usize,
    team_kickoff_opportunities: usize,
    detected_team_kickoffs: usize,
    speed_flip_event_count: usize,
    kickoffs: Vec<KickoffAudit>,
}

#[derive(Debug, Serialize)]
struct KickoffAudit {
    index: usize,
    start_time: f32,
    start_frame: usize,
    blue_front_players: Vec<String>,
    orange_front_players: Vec<String>,
    blue_detected: Vec<DetectedSpeedFlip>,
    orange_detected: Vec<DetectedSpeedFlip>,
}

#[derive(Debug, Serialize)]
struct DetectedSpeedFlip {
    player: String,
    time: f32,
    time_since_kickoff_start: f32,
    confidence: f32,
    diagonal_score: f32,
    cancel_score: f32,
    speed_score: f32,
    max_speed: f32,
}

#[derive(Debug, Parser)]
#[command(about = "Audit speed-flip detections during replay kickoffs.")]
struct Args {
    /// Replay paths to audit.
    #[arg(value_name = "replay", num_args = 1..)]
    paths: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let Args { paths } = Args::parse();

    let mut replays = Vec::new();
    for path in paths {
        replays.push(audit_replay(&path)?);
    }

    println!("{}", serde_json::to_string_pretty(&Audit { replays })?);
    Ok(())
}

fn audit_replay(path: &str) -> anyhow::Result<ReplayAudit> {
    let replay = parse_replay(path)?;
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .map_err(|error| anyhow::anyhow!("failed to build stats timeline for {path}: {error:?}"))?;
    let player_names = player_name_map(&timeline.replay_meta);
    let kickoff_start_indices = kickoff_start_indices(&timeline.frames);
    let mut kickoffs = Vec::new();

    for (index, frame_index) in kickoff_start_indices.into_iter().enumerate() {
        let frame = &timeline.frames[frame_index];
        let start_time = frame.time;
        let end_time = start_time + DETECTION_WINDOW_SECONDS;
        let blue_detected = timeline
            .events
            .speed_flip
            .iter()
            .filter(|event| event.is_team_0)
            .filter(|event| event.time >= start_time && event.time <= end_time)
            .map(|event| DetectedSpeedFlip {
                player: player_name(&player_names, &event.player),
                time: event.time,
                time_since_kickoff_start: event.time - start_time,
                confidence: event.confidence,
                diagonal_score: event.diagonal_score,
                cancel_score: event.cancel_score,
                speed_score: event.speed_score,
                max_speed: event.max_speed,
            })
            .collect();
        let orange_detected = timeline
            .events
            .speed_flip
            .iter()
            .filter(|event| !event.is_team_0)
            .filter(|event| event.time >= start_time && event.time <= end_time)
            .map(|event| DetectedSpeedFlip {
                player: player_name(&player_names, &event.player),
                time: event.time,
                time_since_kickoff_start: event.time - start_time,
                confidence: event.confidence,
                diagonal_score: event.diagonal_score,
                cancel_score: event.cancel_score,
                speed_score: event.speed_score,
                max_speed: event.max_speed,
            })
            .collect();

        kickoffs.push(KickoffAudit {
            index: index + 1,
            start_time,
            start_frame: frame.frame_number,
            blue_front_players: front_players(frame, true, &player_names),
            orange_front_players: front_players(frame, false, &player_names),
            blue_detected,
            orange_detected,
        });
    }

    let team_kickoff_opportunities = kickoffs
        .iter()
        .map(|kickoff| {
            usize::from(!kickoff.blue_front_players.is_empty())
                + usize::from(!kickoff.orange_front_players.is_empty())
        })
        .sum();
    let detected_team_kickoffs = kickoffs
        .iter()
        .map(|kickoff| {
            usize::from(!kickoff.blue_detected.is_empty())
                + usize::from(!kickoff.orange_detected.is_empty())
        })
        .sum();

    Ok(ReplayAudit {
        path: path.to_owned(),
        kickoff_count: kickoffs.len(),
        team_kickoff_opportunities,
        detected_team_kickoffs,
        speed_flip_event_count: timeline.events.speed_flip.len(),
        kickoffs,
    })
}

fn parse_replay(path: &str) -> anyhow::Result<boxcars::Replay> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read {}", Path::new(path).display()))?;
    boxcars::ParserBuilder::new(&bytes)
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .with_context(|| format!("failed to parse {}", Path::new(path).display()))
}

fn kickoff_start_indices(frames: &[ReplayStatsFrame]) -> Vec<usize> {
    frames
        .iter()
        .enumerate()
        .filter(|(index, frame)| {
            frame.gameplay_phase == GameplayPhase::KickoffWaitingForTouch
                && index
                    .checked_sub(1)
                    .and_then(|previous| frames.get(previous))
                    .is_none_or(|previous| {
                        previous.gameplay_phase != GameplayPhase::KickoffWaitingForTouch
                    })
        })
        .map(|(index, _)| index)
        .collect()
}

fn front_players(
    frame: &ReplayStatsFrame,
    is_team_0: bool,
    player_names: &HashMap<String, String>,
) -> Vec<String> {
    frame
        .players
        .iter()
        .filter(|player| player.is_team_0 == is_team_0)
        .map(|player| player_name(player_names, &player.player_id))
        .collect()
}

fn player_name_map(meta: &ReplayMeta) -> HashMap<String, String> {
    meta.team_zero
        .iter()
        .chain(meta.team_one.iter())
        .map(|player| (player_id_string(&player.remote_id), player.name.clone()))
        .collect()
}

fn player_name(player_names: &HashMap<String, String>, player_id: &PlayerId) -> String {
    let id = player_id_string(player_id);
    player_names.get(&id).cloned().unwrap_or(id)
}

fn player_id_string(player_id: &PlayerId) -> String {
    match serde_json::to_value(player_id) {
        Ok(serde_json::Value::Object(map)) if map.len() == 1 => {
            let (kind, value) = map.into_iter().next().expect("map has one value");
            match value {
                serde_json::Value::String(value) => format!("{kind}:{value}"),
                other => format!("{kind}:{other}"),
            }
        }
        Ok(value) => value.to_string(),
        Err(_) => format!("{player_id:?}"),
    }
}
