//! Walk a directory of `.replay` files and run the full subtr-actor pipeline
//! against each one, isolating failures so a single bad replay (parse panic,
//! collector error, etc.) cannot abort the sweep. Used to shake out parsing
//! errors across many game modes.
//!
//! Usage: `cargo run -p subtr-actor-tools --bin parse_sweep -- [DIR]`
//! (DIR defaults to `/tmp/replay-sweep`).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use subtr_actor::{ReplayDataCollector, ReplayProcessor, StatsTimelineCollector};

const DEFAULT_DIR: &str = "/tmp/replay-sweep";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Parse,
    Meta,
    ReplayData,
    Timeline,
}

impl Stage {
    fn label(self) -> &'static str {
        match self {
            Stage::Parse => "parse",
            Stage::Meta => "meta",
            Stage::ReplayData => "replay_data",
            Stage::Timeline => "timeline",
        }
    }
}

/// Run `f`, converting both `Err` and panics into a `String` error message.
fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic>".to_string());
            Err(format!("PANIC: {msg}"))
        }
    }
}

fn parse_replay(data: &[u8]) -> Result<boxcars::Replay, String> {
    boxcars::ParserBuilder::new(data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .map_err(|e| format!("{e}"))
}

/// Run all stages for one replay, returning the first failing stage + message.
fn sweep_one(path: &Path) -> Result<(), (Stage, String)> {
    let data = std::fs::read(path).map_err(|e| (Stage::Parse, format!("read error: {e}")))?;

    let replay = guarded(|| parse_replay(&data)).map_err(|m| (Stage::Parse, m))?;

    guarded(|| {
        #[allow(clippy::result_large_err)] // SubtrActorError is large; boxing it is out of scope
        fn process_meta(replay: &boxcars::Replay) -> subtr_actor::SubtrActorResult<()> {
            ReplayProcessor::new(replay)
                .and_then(|mut p| p.process_and_get_replay_meta().map(|_| ()))
        }
        process_meta(&replay).map_err(|e| format!("{e:?}"))
    })
    .map_err(|m| (Stage::Meta, m))?;

    guarded(|| {
        ReplayDataCollector::new()
            .get_replay_data(&replay)
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    })
    .map_err(|m| (Stage::ReplayData, m))?;

    guarded(|| {
        StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    })
    .map_err(|m| (Stage::Timeline, m))?;

    Ok(())
}

/// Collapse a message to a coarse signature for aggregation (drop digits and
/// hex-ish tokens so per-replay actor ids/counts collapse together).
fn signature(stage: Stage, msg: &str) -> String {
    let mut out = String::with_capacity(msg.len());
    let mut prev_num = false;
    for ch in msg.chars() {
        if ch.is_ascii_digit() {
            if !prev_num {
                out.push('#');
            }
            prev_num = true;
        } else {
            prev_num = false;
            out.push(ch);
        }
    }
    let trimmed: String = out.chars().take(180).collect();
    format!("[{}] {trimmed}", stage.label())
}

fn main() {
    let target = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| DEFAULT_DIR.to_string()),
    );

    // Accept either a single `.replay` file (so callers can wrap each one in an
    // external `timeout` to bound true hangs) or a directory to sweep.
    let mut paths: Vec<PathBuf> = if target.is_file() {
        vec![target]
    } else {
        let mut found: Vec<PathBuf> = std::fs::read_dir(&target)
            .unwrap_or_else(|e| panic!("failed to read dir {}: {e}", target.display()))
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|e| e == "replay").unwrap_or(false))
            .collect();
        found.sort();
        found
    };
    paths.sort();

    // Keep panic output quiet; we report failures ourselves.
    std::panic::set_hook(Box::new(|_| {}));

    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut by_signature: std::collections::BTreeMap<String, Vec<(String, String)>> =
        std::collections::BTreeMap::new();

    for path in &paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match sweep_one(path) {
            Ok(()) => {
                pass += 1;
                println!("PASS  {name}");
            }
            Err((stage, msg)) => {
                fail += 1;
                println!("FAIL  [{}] {name}: {msg}", stage.label());
                by_signature
                    .entry(signature(stage, &msg))
                    .or_default()
                    .push((name, msg));
            }
        }
    }

    println!("\n================ SUMMARY ================");
    println!("total={} pass={pass} fail={fail}", paths.len());
    if !by_signature.is_empty() {
        println!("\n--- distinct failure signatures ---");
        for (sig, hits) in &by_signature {
            println!("\n({}x) {sig}", hits.len());
            for (name, msg) in hits {
                println!("    - {name}: {msg}");
            }
        }
    }
}
