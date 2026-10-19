//! Dump per-frame replay telemetry for the whiff-learning workflow.
//!
//! For each replay, writes one gzipped columnar JSON file containing the exact
//! per-frame inputs the whiff calculator consumes (frame clock, live-play flag,
//! ball rigid body, every player's rigid body, boost/dodge/jump state, and the
//! oriented-hitbox ball contact gap computed by the library geometry), plus the
//! attributed touches and the events emitted by the current whiff and
//! beaten-to-ball detectors. Missing samples are written as `null` so downstream
//! code never mistakes an unobserved car for a stationary one.
//!
//! The Python workflow under `scripts/whiff_model/` mines review windows,
//! renders evidence, and computes model features from these files.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};
use subtr_actor_tools::whiff_model::telemetry::{ReplayWhiffTelemetry, collect_replay_telemetry};

const DUMP_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Parser)]
#[command(about = "Dump per-frame whiff-learning telemetry for a manifest of replays.")]
struct Args {
    /// JSONL manifest of replays (`{"path": ..., "ballchasing_id": ...}` rows).
    #[arg(long)]
    manifest: Option<PathBuf>,
    /// Additional replay paths.
    replay_paths: Vec<PathBuf>,
    /// Output directory; one `<replay_id>.frames.json.gz` per replay.
    #[arg(long)]
    out_dir: PathBuf,
    /// Re-dump replays whose output already exists.
    #[arg(long)]
    overwrite: bool,
    /// Worker threads (defaults to available parallelism).
    #[arg(long)]
    threads: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestRow {
    path: String,
    #[serde(default)]
    ballchasing_id: Option<String>,
}

#[derive(Debug, Clone)]
struct ReplayJob {
    path: PathBuf,
    replay_id: String,
}

fn replay_content_id(path: &Path) -> anyhow::Result<String> {
    // Cheap stable content identity; collisions are irrelevant at corpus scale.
    let data = std::fs::read(path)?;
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in &data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    Ok(format!("fnv1a64:{hash:016x}:{}", data.len()))
}

fn dump_replay(job: &ReplayJob, out_dir: &Path) -> anyhow::Result<Value> {
    let data = std::fs::read(&job.path)
        .with_context(|| format!("reading replay {}", job.path.display()))?;
    let replay = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .with_context(|| format!("parsing replay {}", job.path.display()))?;

    let ReplayWhiffTelemetry {
        telemetry: node,
        whiff_events: whiffs,
        beaten_to_ball_events: beaten,
    } = collect_replay_telemetry(&replay)?;

    let content_id = replay_content_id(&job.path)?;
    let summary = json!({
        "replay_id": job.replay_id,
        "frames": node.frames.frame.len(),
        "players": node.players.len(),
        "touches": node.touches.len(),
        "whiffs": whiffs.len(),
        "beaten_to_ball": beaten.len(),
    });
    let document = json!({
        "format_version": DUMP_FORMAT_VERSION,
        "replay_id": job.replay_id,
        "source_path": job.path.display().to_string(),
        "content_id": content_id,
        "frames": &node.frames,
        "players": &node.players,
        "touches": &node.touches,
        "whiff_events": whiffs,
        "beaten_to_ball_events": beaten,
    });
    let out_path = out_dir.join(format!("{}.frames.json.gz", job.replay_id));
    let tmp_path = out_path.with_extension("gz.part");
    {
        let file = std::fs::File::create(&tmp_path)
            .with_context(|| format!("creating {}", tmp_path.display()))?;
        let mut encoder = flate2::write::GzEncoder::new(
            std::io::BufWriter::new(file),
            flate2::Compression::default(),
        );
        serde_json::to_writer(&mut encoder, &document)?;
        encoder.finish()?.flush()?;
    }
    std::fs::rename(&tmp_path, &out_path)?;
    Ok(summary)
}

fn load_jobs(args: &Args) -> anyhow::Result<Vec<ReplayJob>> {
    let mut jobs = Vec::new();
    if let Some(manifest) = &args.manifest {
        let file = std::fs::File::open(manifest)
            .with_context(|| format!("opening manifest {}", manifest.display()))?;
        for (line_number, line) in std::io::BufReader::new(file).lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let row: ManifestRow = serde_json::from_str(&line)
                .with_context(|| format!("bad manifest row on line {}", line_number + 1))?;
            let path = PathBuf::from(&row.path);
            let replay_id = row.ballchasing_id.unwrap_or_else(|| file_stem(&path));
            jobs.push(ReplayJob { path, replay_id });
        }
    }
    for path in &args.replay_paths {
        jobs.push(ReplayJob {
            path: path.clone(),
            replay_id: file_stem(path),
        });
    }
    Ok(jobs)
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    std::fs::create_dir_all(&args.out_dir)?;
    let jobs: Vec<ReplayJob> = load_jobs(&args)?
        .into_iter()
        .filter(|job| {
            args.overwrite
                || !args
                    .out_dir
                    .join(format!("{}.frames.json.gz", job.replay_id))
                    .exists()
        })
        .collect();
    let threads = args
        .threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()))
        .max(1);
    eprintln!("dumping {} replays with {threads} threads", jobs.len());

    let next = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            let sender = sender.clone();
            let jobs = &jobs;
            let next = &next;
            let out_dir = &args.out_dir;
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::SeqCst);
                    let Some(job) = jobs.get(index) else { break };
                    let result = dump_replay(job, out_dir);
                    if sender.send((job.replay_id.clone(), result)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);
    });

    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(args.out_dir.join("dump_log.jsonl"))?;
    let mut failures = 0;
    for (replay_id, result) in receiver {
        let line = match result {
            Ok(summary) => json!({"ok": true, "summary": summary}),
            Err(error) => {
                failures += 1;
                eprintln!("{replay_id}: {error:#}");
                json!({"ok": false, "replay_id": replay_id, "error": format!("{error:#}")})
            }
        };
        writeln!(log, "{line}")?;
    }
    eprintln!("done; {failures} failures");
    Ok(())
}
