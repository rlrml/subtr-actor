use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use clap::Parser;
use subtr_actor::{
    EventPayload, KickoffEvent, KickoffTakerEvent, PlayerId, ReplayMeta, StatsTimelineCollector,
};

#[derive(Debug, Parser)]
#[command(about = "Dump per-kickoff taker timing, spawn position, and approach.")]
struct Args {
    /// Replay paths to dump.
    #[arg(value_name = "replay", num_args = 1..)]
    paths: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let Args { paths } = Args::parse();
    // TSV header
    println!("replay\ttaker\ttaker_id\tspawn\tapproach\ttime_to_ball\toutcome");
    for path in paths {
        let dumped = dump_replay(&path);
        if let Err(error) = dumped {
            eprintln!("skip {path}: {error:?}");
        }
    }
    Ok(())
}

fn dump_replay(path: &str) -> anyhow::Result<()> {
    let replay = parse_replay(path)?;
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .map_err(|error| anyhow::anyhow!("failed to build stats timeline for {path}: {error:?}"))?;
    let names = player_name_map(&timeline.replay_meta);
    let short = Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_owned());
    for event in &timeline.events.events {
        if let EventPayload::Kickoff(kickoff) = &event.payload {
            emit_taker(&short, &names, kickoff, kickoff.team_zero_taker.as_ref());
            emit_taker(&short, &names, kickoff, kickoff.team_one_taker.as_ref());
        }
    }
    Ok(())
}

fn emit_taker(
    replay: &str,
    names: &HashMap<String, String>,
    _kickoff: &KickoffEvent,
    taker: Option<&KickoffTakerEvent>,
) {
    let Some(taker) = taker else {
        return;
    };
    let name = player_name(names, &taker.player);
    let id = player_id_string(&taker.player);
    let spawn = taker.spawn_position.as_label_value();
    let approach = taker.approach.as_label_value();
    let time_to_ball = taker
        .time_to_ball
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "NA".to_owned());
    let outcome = taker.outcome.as_label_value();
    println!("{replay}\t{name}\t{id}\t{spawn}\t{approach}\t{time_to_ball}\t{outcome}");
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

fn player_name_map(meta: &ReplayMeta) -> HashMap<String, String> {
    meta.team_zero
        .iter()
        .chain(meta.team_one.iter())
        .map(|player| (player_id_string(&player.remote_id), player.name.clone()))
        .collect()
}

fn player_name(names: &HashMap<String, String>, player_id: &PlayerId) -> String {
    let id = player_id_string(player_id);
    names.get(&id).cloned().unwrap_or(id)
}

fn player_id_string(player_id: &PlayerId) -> String {
    match serde_json::to_value(player_id) {
        Ok(serde_json::Value::Object(map)) if map.len() == 1 => {
            let (kind, value) = map.into_iter().next().expect("map has one value");
            let id = value
                .as_object()
                .and_then(|object| object.get("online_id"))
                .and_then(json_scalar_text)
                .or_else(|| json_scalar_text(&value))
                .unwrap_or_else(|| value.to_string());
            format!("{kind}:{id}")
        }
        Ok(value) => value.to_string(),
        Err(_) => format!("{player_id:?}"),
    }
}

fn json_scalar_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}
