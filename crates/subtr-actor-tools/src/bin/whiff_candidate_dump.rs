//! Dump the core whiff detector's candidates, features, and events.
//!
//! Runs the production `WhiffCalculator` with candidate recording enabled and
//! writes one gzipped JSON file per replay. `scripts/whiff_model` trains on
//! these vectors, so training and inference share the core feature code.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};
use subtr_actor::stats::analysis_graph::{
    WhiffNode, collect_analysis_graph_for_replay, graph_with_builtin_analysis_nodes,
};
use subtr_actor::{
    BeatenToBallCalculator, WHIFF_ATTEMPT_CLASSES, WHIFF_FEATURE_NAMES, WHIFF_FEATURE_VERSION,
    WHIFF_MODEL_VERSION, WhiffCalculator,
};
use subtr_actor_tools::whiff_model::telemetry::{events_with_player_ids, player_id_string};

#[derive(Debug, Parser)]
#[command(about = "Dump core whiff candidates with features for a manifest of replays.")]
struct Args {
    /// JSONL manifest of replays (`{"path": ..., "ballchasing_id": ...}` rows).
    #[arg(long)]
    manifest: Option<PathBuf>,
    /// Additional replay paths.
    replay_paths: Vec<PathBuf>,
    /// Output directory; one `<replay_id>.candidates.json.gz` per replay.
    #[arg(long)]
    out_dir: PathBuf,
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

struct ReplayJob {
    path: PathBuf,
    replay_id: String,
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn load_jobs(args: &Args) -> anyhow::Result<Vec<ReplayJob>> {
    let mut jobs = Vec::new();
    if let Some(manifest) = &args.manifest {
        let file = std::fs::File::open(manifest)
            .with_context(|| format!("opening manifest {}", manifest.display()))?;
        for line in std::io::BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let row: ManifestRow = serde_json::from_str(&line)?;
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

fn dump_replay(job: &ReplayJob, out_dir: &Path) -> anyhow::Result<usize> {
    let data = std::fs::read(&job.path)
        .with_context(|| format!("reading replay {}", job.path.display()))?;
    let replay = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .with_context(|| format!("parsing replay {}", job.path.display()))?;
    let mut graph = graph_with_builtin_analysis_nodes(["beaten_to_ball"])
        .map_err(|error| anyhow::anyhow!("building graph: {error:?}"))?;
    let mut calculator = WhiffCalculator::new();
    calculator.enable_candidate_recording();
    graph.push_node(WhiffNode::with_calculator(calculator));
    let graph = collect_analysis_graph_for_replay(&replay, graph)
        .map_err(|error| anyhow::anyhow!("processing replay: {error:?}"))?;
    let calculator = graph
        .state::<WhiffCalculator>()
        .ok_or_else(|| anyhow::anyhow!("whiff state missing"))?;
    let classes: Vec<String> = WHIFF_ATTEMPT_CLASSES
        .iter()
        .map(|class| format!("{class:?}"))
        .collect();
    let candidates: Vec<Value> = calculator
        .candidates()
        .iter()
        .map(|record| {
            json!({
                "player_id": player_id_string(&record.player),
                "is_team_0": record.is_team_0,
                "anchor_time": record.anchor_time,
                "anchor_frame": record.anchor_frame,
                "resolved_frame": record.resolved_frame,
                "features": record.features.iter().map(|v| if v.is_finite() { json!(v) } else { Value::Null }).collect::<Vec<_>>(),
                "probabilities": record.classification.probabilities,
                "class": format!("{:?}", record.classification.class),
            })
        })
        .collect();
    let beaten = graph
        .state::<BeatenToBallCalculator>()
        .map(|calculator| events_with_player_ids(calculator.events(), |event| &event.player))
        .unwrap_or_default();
    let document = json!({
        "replay_id": job.replay_id,
        "feature_version": WHIFF_FEATURE_VERSION,
        "feature_names": WHIFF_FEATURE_NAMES.as_slice(),
        "model_version": WHIFF_MODEL_VERSION,
        "classes": classes,
        "candidates": candidates,
        "whiff_events": events_with_player_ids(calculator.events(), |event| &event.player),
        "beaten_to_ball_events": beaten,
    });
    let out_path = out_dir.join(format!("{}.candidates.json.gz", job.replay_id));
    let tmp_path = out_path.with_extension("gz.part");
    {
        let file = std::fs::File::create(&tmp_path)?;
        let mut encoder = flate2::write::GzEncoder::new(
            std::io::BufWriter::new(file),
            flate2::Compression::fast(),
        );
        serde_json::to_writer(&mut encoder, &document)?;
        encoder.finish()?.flush()?;
    }
    std::fs::rename(&tmp_path, &out_path)?;
    Ok(candidates.len())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    std::fs::create_dir_all(&args.out_dir)?;
    let jobs = load_jobs(&args)?;
    let threads = args
        .threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()))
        .max(1);
    let next = AtomicUsize::new(0);
    let failures = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::SeqCst);
                    let Some(job) = jobs.get(index) else { break };
                    match dump_replay(job, &args.out_dir) {
                        Ok(count) => eprintln!("{} {count} candidates", job.replay_id),
                        Err(error) => {
                            failures.fetch_add(1, Ordering::SeqCst);
                            eprintln!("{} FAILED: {error:#}", job.replay_id);
                        }
                    }
                }
            });
        }
    });
    let failures = failures.into_inner();
    anyhow::ensure!(failures == 0, "{failures} replays failed");
    Ok(())
}
