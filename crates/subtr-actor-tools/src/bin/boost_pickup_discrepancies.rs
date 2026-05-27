use std::collections::HashMap;

use anyhow::anyhow;
use clap::Parser;
use subtr_actor::{
    stats::analysis_graph::collect_builtin_analysis_graph_for_replay, BoostCalculator,
    BoostPickupComparison, PlayerId, ReplayProcessor,
};

#[path = "boost_pickup_discrepancies_helpers.rs"]
mod helpers;
#[path = "boost_pickup_discrepancies_types.rs"]
mod types;

use helpers::{
    count_events, event_sort_key, parse_replay, print_jsonl_record, resolve_replay_path,
};
use types::{Args, PickupRecord, SummaryRecord};

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
