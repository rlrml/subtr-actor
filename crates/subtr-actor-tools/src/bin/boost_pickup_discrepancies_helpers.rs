use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Serialize;
use subtr_actor::{
    BoostPickupActivity, BoostPickupComparison, BoostPickupComparisonEvent, BoostPickupPadType,
};

use super::types::PickupCountBreakdown;

pub(crate) fn parse_replay(path: &Path) -> anyhow::Result<boxcars::Replay> {
    let data =
        std::fs::read(path).with_context(|| format!("failed to read replay {}", path.display()))?;
    boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .always_check_crc()
        .parse()
        .with_context(|| format!("failed to parse replay {}", path.display()))
}

pub(crate) fn resolve_replay_path(arg: &str) -> PathBuf {
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

pub(crate) fn count_events<'a>(
    events: impl IntoIterator<Item = &'a BoostPickupComparisonEvent>,
) -> PickupCountBreakdown {
    let mut counts = PickupCountBreakdown::default();
    for event in events {
        increment_breakdown(&mut counts, event);
    }
    counts
}

pub(crate) fn print_jsonl_record<T: Serialize>(record: &T) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    let result = writeln!(stdout, "{}", serde_json::to_string(record)?);
    match result {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn event_sort_key(event: &BoostPickupComparisonEvent) -> (usize, String, &'static str) {
    let comparison = match event.comparison {
        BoostPickupComparison::Both => "both",
        BoostPickupComparison::Ghost => "ghost",
        BoostPickupComparison::Missed => "missed",
    };
    (event.frame, format!("{:?}", event.player_id), comparison)
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
