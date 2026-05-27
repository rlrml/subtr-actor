use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use clap::Parser;
use serde::Serialize;
use subtr_actor::{
    stats::analysis_graph::collect_builtin_analysis_graph_for_replay, BoostCalculator,
    BoostPickupActivity, BoostPickupComparison, BoostPickupComparisonEvent, BoostPickupPadType,
    PlayerId, ReplayProcessor,
};

#[path = "boost_pickup_discrepancies_types.rs"]
mod types;

use types::{Args, PickupCountBreakdown, PickupRecord, SummaryRecord};

fn parse_replay(path: &Path) -> anyhow::Result<boxcars::Replay> {
    let data =
        std::fs::read(path).with_context(|| format!("failed to read replay {}", path.display()))?;
    boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .always_check_crc()
        .parse()
        .with_context(|| format!("failed to parse replay {}", path.display()))
}

fn resolve_replay_path(arg: &str) -> PathBuf {
    let path = PathBuf::from(arg);
    if path.exists() {
        return path;
    }

    let fixture_replay = PathBuf::from(format!("assets/{arg}.replay"));
    if fixture_replay.exists() {
        return fixture_replay;
    }

    path
}

fn increment_breakdown(counts: &mut PickupCountBreakdown, event: &BoostPickupComparisonEvent) {
    counts.total += 1;
    match event.comparison {
        BoostPickupComparison::Both => counts.both += 1,
        BoostPickupComparison::Ghost => counts.ghost += 1,
        BoostPickupComparison::Missed => counts.missed += 1,
    }
    match event.pad_type {
        BoostPickupPadType::Big => counts.big += 1,
        BoostPickupPadType::Small => counts.small += 1,
        BoostPickupPadType::Ambiguous => counts.ambiguous += 1,
    }
    match event.activity {
        BoostPickupActivity::Active => counts.active += 1,
        BoostPickupActivity::Inactive => counts.inactive += 1,
        BoostPickupActivity::Unknown => counts.unknown_activity += 1,
    }
}

fn count_events<'a>(
    events: impl IntoIterator<Item = &'a BoostPickupComparisonEvent>,
) -> PickupCountBreakdown {
    let mut counts = PickupCountBreakdown::default();
    for event in events {
        increment_breakdown(&mut counts, event);
    }
    counts
}

fn print_jsonl_record<T: Serialize>(record: &T) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    let result = writeln!(stdout, "{}", serde_json::to_string(record)?);
    match result {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn event_sort_key(event: &BoostPickupComparisonEvent) -> (usize, String, &'static str) {
    let comparison = match event.comparison {
        BoostPickupComparison::Both => "both",
        BoostPickupComparison::Ghost => "ghost",
        BoostPickupComparison::Missed => "missed",
    };
    (event.frame, format!("{:?}", event.player_id), comparison)
}

fn print_pickups_jsonl(
    label: &str,
    replay: &boxcars::Replay,
    include_all: bool,
) -> anyhow::Result<()> {
    let graph = collect_builtin_analysis_graph_for_replay(replay, ["boost"])
        .map_err(|err| anyhow!("failed to collect boost stats for {label}: {err:?}"))?;
    let boost = graph
        .state::<BoostCalculator>()
        .ok_or_else(|| anyhow!("boost calculator missing from analysis graph"))?;
    let processor = ReplayProcessor::new(replay)
        .map_err(|err| anyhow!("failed to build replay processor for {label}: {err:?}"))?;
    let replay_meta = processor
        .get_replay_meta()
        .map_err(|err| anyhow!("failed to read replay metadata for {label}: {err:?}"))?;
    let player_names: HashMap<PlayerId, String> = replay_meta
        .team_zero
        .iter()
        .chain(replay_meta.team_one.iter())
        .map(|player| (player.remote_id.clone(), player.name.clone()))
        .collect();

    let mut events = boost.pickup_comparison_events().to_vec();
    events.sort_by_key(event_sort_key);
    let emitted_events = events
        .iter()
        .filter(|event| include_all || event.comparison != BoostPickupComparison::Both)
        .collect::<Vec<_>>();

    print_jsonl_record(&SummaryRecord {
        record_type: "summary",
        replay: label,
        emitted: if include_all { "all" } else { "discrepancies" },
        all_events: count_events(&events),
        emitted_events: count_events(emitted_events.iter().copied()),
    })?;

    for event in emitted_events {
        print_jsonl_record(&PickupRecord {
            record_type: "pickup",
            replay: label,
            comparison: event.comparison,
            frame: event.frame,
            time: event.time,
            player_id: &event.player_id,
            player: player_names
                .get(&event.player_id)
                .cloned()
                .unwrap_or_else(|| format!("{:?}", event.player_id)),
            team: if event.is_team_0 { "blue" } else { "orange" },
            pad_type: event.pad_type,
            field_half: event.field_half,
            activity: event.activity,
            reported_frame: event.reported_frame,
            reported_time: event.reported_time,
            inferred_frame: event.inferred_frame,
            inferred_time: event.inferred_time,
            boost_before: event.boost_before,
            boost_after: event.boost_after,
        })?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let Args {
        include_all,
        replay_args,
    } = Args::parse();

    for arg in replay_args {
        let replay_path = resolve_replay_path(&arg);
        let replay = parse_replay(&replay_path)?;
        print_pickups_jsonl(&replay_path.display().to_string(), &replay, include_all)?;
    }

    Ok(())
}
