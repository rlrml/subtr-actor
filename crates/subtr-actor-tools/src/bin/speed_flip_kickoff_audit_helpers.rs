use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use subtr_actor::{GameplayPhase, PlayerId, ReplayMeta, ReplayStatsFrame, SpeedFlipEvent};

use super::types::DetectedSpeedFlip;

pub(crate) fn parse_replay(path: &str) -> anyhow::Result<boxcars::Replay> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read {}", Path::new(path).display()))?;
    boxcars::ParserBuilder::new(&bytes)
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .with_context(|| format!("failed to parse {}", Path::new(path).display()))
}

pub(crate) fn kickoff_start_indices(frames: &[ReplayStatsFrame]) -> Vec<usize> {
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

pub(crate) fn front_players(
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

pub(crate) fn detected_speed_flips(
    events: &[SpeedFlipEvent],
    is_team_0: bool,
    start_time: f32,
    end_time: f32,
    player_names: &HashMap<String, String>,
) -> Vec<DetectedSpeedFlip> {
    events
        .iter()
        .filter(|event| event.is_team_0 == is_team_0)
        .filter(|event| event.time >= start_time && event.time <= end_time)
        .map(|event| DetectedSpeedFlip {
            player: player_name(player_names, &event.player),
            time: event.time,
            time_since_kickoff_start: event.time - start_time,
            confidence: event.confidence,
            diagonal_score: event.diagonal_score,
            cancel_score: event.cancel_score,
            speed_score: event.speed_score,
            max_speed: event.max_speed,
        })
        .collect()
}

pub(crate) fn player_name_map(meta: &ReplayMeta) -> HashMap<String, String> {
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
