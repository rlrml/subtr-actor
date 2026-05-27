use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct NumericDelta {
    pub(crate) path: String,
    pub(crate) actual: f64,
    pub(crate) expected: f64,
    pub(crate) delta: f64,
    pub(crate) abs_delta: f64,
}

#[derive(Debug, Serialize)]
pub(crate) struct OutputReport {
    pub(crate) is_match: bool,
    pub(crate) mismatch_count: usize,
    pub(crate) mismatches: Vec<String>,
    pub(crate) deltas: Vec<NumericDelta>,
}

#[derive(Debug, Parser)]
#[command(about = "Compare a replay against exported Ballchasing JSON with numeric deltas.")]
pub(crate) struct Args {
    /// Replay file to compare.
    pub(crate) replay_path: PathBuf,

    /// Ballchasing JSON path to compare against.
    pub(crate) ballchasing_json_path: PathBuf,

    /// Optional output directory for comparison artifacts.
    pub(crate) output_dir: Option<PathBuf>,
}
