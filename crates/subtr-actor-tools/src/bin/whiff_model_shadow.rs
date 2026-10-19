//! Run the experimental telemetry whiff model over replays, in shadow mode.
//!
//! Writes one JSON line per causal candidate (player, anchor, features, class
//! probabilities, thresholded class) so results can be compared with the
//! Python trainer's predictions and with the production whiff detector. The
//! production detector is not changed.

use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use serde_json::json;
use subtr_actor_tools::whiff_model::features::{NumericTelemetry, replay_candidates};
use subtr_actor_tools::whiff_model::model::WhiffModel;
use subtr_actor_tools::whiff_model::telemetry::collect_replay_telemetry;

#[derive(Debug, Parser)]
#[command(about = "Score whiff candidates with an exported experimental model.")]
struct Args {
    /// Exported `model.json` from `whifflab.py train`.
    #[arg(long)]
    model: PathBuf,
    /// Output JSONL path.
    #[arg(long)]
    out: PathBuf,
    /// Replays to score.
    replay_paths: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let model = WhiffModel::from_json(
        &std::fs::read_to_string(&args.model)
            .with_context(|| format!("reading {}", args.model.display()))?,
    )?;
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args.out)?);
    for path in &args.replay_paths {
        let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let replay = boxcars::ParserBuilder::new(&data[..])
            .always_check_crc()
            .must_parse_network_data()
            .parse()
            .with_context(|| format!("parsing {}", path.display()))?;
        let collected = collect_replay_telemetry(&replay)?;
        let numeric = NumericTelemetry::from_telemetry(&collected.telemetry);
        let replay_id = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let candidates = replay_candidates(&numeric);
        let mut counts = std::collections::BTreeMap::<String, usize>::new();
        for candidate in &candidates {
            let proba = model.predict_proba(&candidate.features);
            let class = model.predict_class(&candidate.features);
            *counts.entry(class.to_owned()).or_default() += 1;
            let features: Vec<Option<f64>> = candidate
                .features
                .iter()
                .map(|v| v.is_finite().then_some(*v))
                .collect();
            writeln!(
                out,
                "{}",
                json!({
                    "replay_id": replay_id,
                    "player_id": numeric.players[candidate.player_index].id,
                    "anchor_frame": collected.telemetry.frames.frame[candidate.anchor_index],
                    "anchor_time": candidate.anchor_time,
                    "features": features,
                    "proba": proba,
                    "class": class,
                })
            )?;
        }
        eprintln!(
            "{replay_id}: {} candidates, predicted {counts:?}, production whiff events {}",
            candidates.len(),
            collected.whiff_events.len()
        );
    }
    Ok(())
}
