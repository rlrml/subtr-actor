//! Dump a threat-model training dataset from a manifest of replays.
//!
//! For each replay, samples both teams' attacking-normalized
//! `ThreatModelFeatures` rows at `--sample-hz` during live play (through the
//! same ndarray threat feature adder the runtime model consumes -- including
//! causal history, never reimplemented) and joins each row with the
//! replay-time distance to the next goal for/against that side plus the time
//! to replay end, so the Python training pipeline can compute
//! scored-within-tau labels with censoring downstream.
//!
//! Manifest rows are JSON objects, one per line:
//! `{"path": ..., "ballchasing_id": ..., "playlist": ..., "date": ...,
//!   "min_rank_tier": ..., "max_rank_tier": ..., "median_rank_tier": ...,
//!   "team_size": ..., ...}`.
//! Unknown keys are ignored.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use subtr_actor::{
    Collector, ExpectedGoalsCalculator, LiveThreatSampleFilter, NDArrayCollector,
    ThreatModelFeatures,
};

#[derive(Debug, Parser)]
#[command(about = "Dump per-frame threat features and goal-time labels for threat-model training.")]
struct Args {
    /// JSONL manifest of replays to process.
    #[arg(long)]
    manifest: PathBuf,
    /// Output CSV path.
    #[arg(long)]
    out: PathBuf,
    /// Live-play sampling rate in rows-per-second (per team).
    #[arg(long, default_value_t = 4.0)]
    sample_hz: f32,
    /// Export complete player vectors as well as the legacy model context.
    #[arg(long)]
    entity_features: bool,
    /// Also export strictly pre-contact entity features for every detected touch.
    #[arg(long)]
    touch_out: Option<PathBuf>,
    /// Process at most this many manifest rows.
    #[arg(long)]
    limit: Option<usize>,
    /// Worker threads (defaults to available parallelism).
    #[arg(long)]
    threads: Option<usize>,
    /// Write per-team observed goals and cumulative threat for count-scale audits.
    #[arg(long)]
    episode_summary: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestRow {
    path: String,
    #[serde(default)]
    ballchasing_id: Option<String>,
    #[serde(default)]
    playlist: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    min_rank_tier: Option<i64>,
    #[serde(default)]
    max_rank_tier: Option<i64>,
    #[serde(default)]
    median_rank_tier: Option<f64>,
    #[serde(default)]
    team_size: Option<u32>,
}

impl ManifestRow {
    fn replay_id(&self) -> String {
        self.ballchasing_id.clone().unwrap_or_else(|| {
            Path::new(&self.path)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| self.path.clone())
        })
    }
}

struct ReplayRows {
    replay_id: String,
    lines: Vec<String>,
    value_min: f32,
    value_max: f32,
    goal_count: usize,
    /// One summary row per team, in `episode_summary_header` column order.
    episode_summary_lines: Vec<String>,
    touch_lines: Vec<String>,
    audit: serde_json::Value,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    anyhow::ensure!(args.sample_hz > 0.0, "--sample-hz must be positive");

    let manifest_file = std::fs::File::open(&args.manifest)
        .with_context(|| format!("failed to open manifest {}", args.manifest.display()))?;
    let mut rows: Vec<ManifestRow> = Vec::new();
    for (line_number, line) in std::io::BufReader::new(manifest_file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: ManifestRow = serde_json::from_str(&line)
            .with_context(|| format!("bad manifest row on line {}", line_number + 1))?;
        rows.push(row);
    }
    if let Some(limit) = args.limit {
        rows.truncate(limit);
    }

    let mut out = std::io::BufWriter::new(
        std::fs::File::create(&args.out)
            .with_context(|| format!("failed to create {}", args.out.display()))?,
    );
    writeln!(out, "{}", header(args.entity_features))?;
    let mut audit_out = std::io::BufWriter::new(std::fs::File::create(
        args.out.with_extension("audit.jsonl"),
    )?);

    let mut episode_summary_out = args
        .episode_summary
        .as_ref()
        .map(|path| -> anyhow::Result<_> {
            let mut writer = std::io::BufWriter::new(
                std::fs::File::create(path)
                    .with_context(|| format!("failed to create {}", path.display()))?,
            );
            writeln!(writer, "{}", episode_summary_header())?;
            Ok(writer)
        })
        .transpose()?;

    let mut touch_out = args
        .touch_out
        .as_ref()
        .map(|path| -> anyhow::Result<_> {
            let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);
            writeln!(writer, "{},time_to_next_touch", header(true))?;
            Ok(writer)
        })
        .transpose()?;
    let entity_features = args.entity_features;
    let sample_interval = 1.0 / args.sample_hz;
    let threads = args
        .threads
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(1)
        })
        .max(1);

    let next_index = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel::<Result<ReplayRows, (String, String)>>();

    let mut processed = 0usize;
    let mut skipped = 0usize;
    let mut total_rows = 0usize;
    std::thread::scope(|scope| -> anyhow::Result<()> {
        for _ in 0..threads.min(rows.len().max(1)) {
            let sender = sender.clone();
            let rows = &rows;
            let next_index = &next_index;
            scope.spawn(move || {
                loop {
                    let index = next_index.fetch_add(1, Ordering::Relaxed);
                    let Some(row) = rows.get(index) else {
                        break;
                    };
                    let result = process_replay(row, sample_interval, entity_features)
                        .map_err(|error| (row.replay_id(), format!("{error:#}")));
                    if sender.send(result).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);

        for result in receiver {
            match result {
                Ok(replay_rows) => {
                    writeln!(audit_out, "{}", replay_rows.audit)?;
                    processed += 1;
                    total_rows += replay_rows.lines.len();
                    for line in &replay_rows.lines {
                        writeln!(out, "{line}")?;
                    }
                    if let Some(writer) = touch_out.as_mut() {
                        for line in &replay_rows.touch_lines {
                            writeln!(writer, "{line}")?;
                        }
                    }
                    if let Some(writer) = episode_summary_out.as_mut() {
                        for line in &replay_rows.episode_summary_lines {
                            writeln!(writer, "{line}")?;
                        }
                    }
                    eprintln!(
                        "ok {}: {} rows, {} goals, V in [{:.4}, {:.4}]",
                        replay_rows.replay_id,
                        replay_rows.lines.len(),
                        replay_rows.goal_count,
                        replay_rows.value_min,
                        replay_rows.value_max,
                    );
                }
                Err((replay_id, error)) => {
                    writeln!(
                        audit_out,
                        "{}",
                        serde_json::json!({"replay_id": replay_id, "error": error})
                    )?;
                    skipped += 1;
                    eprintln!("skip {replay_id}: {error}");
                }
            }
        }
        Ok(())
    })?;
    out.flush()?;
    audit_out.flush()?;
    if let Some(writer) = touch_out.as_mut() {
        writer.flush()?;
    }
    if let Some(writer) = episode_summary_out.as_mut() {
        writer.flush()?;
    }

    eprintln!(
        "done: {processed} replays processed, {skipped} skipped, {total_rows} rows -> {}",
        args.out.display()
    );
    Ok(())
}

fn header(entity_features: bool) -> String {
    let mut columns = vec![
        "replay_id",
        "playlist",
        "date",
        "min_rank_tier",
        "max_rank_tier",
        "median_rank_tier",
        "team_size",
        "is_team0",
        "time",
    ];
    columns.extend(if entity_features {
        ThreatModelFeatures::entity_feature_names()
    } else {
        ThreatModelFeatures::feature_names()
    });
    columns.extend([
        "time_to_next_goal_for",
        "time_to_next_goal_against",
        "time_to_replay_end",
        "time_to_live_end",
        "live_end_resolved",
    ]);
    columns.join(",")
}

fn episode_summary_header() -> &'static str {
    "replay_id,is_team0,episode_count,episode_threat_integral,threat_integral,peak_sum,goals,touch_xg"
}

fn process_replay(
    row: &ManifestRow,
    sample_interval: f32,
    entity_features: bool,
) -> anyhow::Result<ReplayRows> {
    anyhow::ensure!(
        row.team_size.is_none_or(|team_size| team_size == 2),
        "threat model only supports 2v2 replays"
    );
    let feature_count = if entity_features {
        subtr_actor::THREAT_ENTITY_FEATURE_COUNT
    } else {
        subtr_actor::THREAT_MODEL_FEATURE_COUNT
    };
    let feature_name = if entity_features {
        "ThreatEntityFeatures"
    } else {
        "ThreatModelFeatures"
    };
    let replay = parse_replay(&row.path)?;
    let collector = NDArrayCollector::<f32>::from_strings(
        &["CurrentTime", feature_name, "ThreatModelValues"],
        &[],
    )
    .map_err(|error| anyhow::anyhow!("failed to configure threat ndarray: {error:?}"))?
    .with_analysis_frame_filter(LiveThreatSampleFilter::new(sample_interval))
    .process_replay(&replay)
    .map_err(|error| anyhow::anyhow!("failed to process replay: {error:?}"))?;
    let calculator = collector
        .analysis_state::<ExpectedGoalsCalculator>()
        .context("expected_goals state missing from graph")?;
    let goals = calculator.goal_records().to_vec();
    let segments = calculator.live_segments().to_vec();
    anyhow::ensure!(
        segments
            .iter()
            .filter(|segment| segment.goal_time.is_some())
            .count()
            == goals.len(),
        "goal-to-live-segment coverage mismatch"
    );
    let touches = calculator.touch_events().to_vec();
    let episodes = calculator.episode_events().to_vec();
    let full_integrals = calculator.team_threat_integrals();
    let replay_end_time = calculator.last_frame_time().unwrap_or(0.0);
    let (meta, matrix) = collector
        .get_meta_and_ndarray()
        .map_err(|error| anyhow::anyhow!("failed to build threat ndarray: {error:?}"))?;
    anyhow::ensure!(
        matrix.nrows() > 0,
        "replay did not produce any valid 2v2 live-play threat rows"
    );
    let replay_id = row.replay_id();
    let metadata_prefix = format!(
        "{},{},{},{},{},{},{}",
        csv_field(&replay_id),
        csv_field(row.playlist.as_deref().unwrap_or("")),
        csv_field(row.date.as_deref().unwrap_or("")),
        optional_int(row.min_rank_tier),
        optional_int(row.max_rank_tier),
        row.median_rank_tier
            .map(|value| value.to_string())
            .unwrap_or_default(),
        row.team_size
            .map(|value| value.to_string())
            .unwrap_or_default(),
    );

    let mut lines = Vec::with_capacity(matrix.nrows() * 2);
    let mut value_min = f32::INFINITY;
    let mut value_max = f32::NEG_INFINITY;
    for matrix_row in matrix.rows() {
        let sample_time = matrix_row[0];
        let segment = segments
            .iter()
            .find(|segment| sample_time >= segment.start_time && sample_time <= segment.end_time)
            .context("sample outside live segments")?;
        if segment
            .goal_time
            .is_some_and(|goal_time| sample_time > goal_time)
        {
            continue;
        }
        for (team_index, is_team_0) in [true, false].into_iter().enumerate() {
            let feature_start = 1 + team_index * feature_count;
            let feature_end = feature_start + feature_count;
            let threat_value = matrix_row[1 + 2 * feature_count + team_index];
            value_min = value_min.min(threat_value);
            value_max = value_max.max(threat_value);
            let mut line = format!("{metadata_prefix},{},{sample_time:.4}", u8::from(is_team_0),);
            for value in matrix_row.slice(ndarray::s![feature_start..feature_end]) {
                line.push_str(&format!(",{value:.6}"));
            }
            line.push_str(&format!(
                ",{},{},{:.4},{:.4},{}",
                segment_goal_time(segment, sample_time, is_team_0),
                segment_goal_time(segment, sample_time, !is_team_0),
                (replay_end_time - sample_time).max(0.0),
                (segment.end_time - sample_time).max(0.0),
                u8::from(segment.resolved),
            ));
            lines.push(line);
        }
    }

    let mut touch_lines = Vec::new();
    for is_team_0 in [true, false] {
        let mut team_touches = touches
            .iter()
            .filter(|touch| touch.team_is_team_0 == is_team_0)
            .collect::<Vec<_>>();
        team_touches.sort_by(|left, right| left.time.total_cmp(&right.time));
        for (index, touch) in team_touches.iter().enumerate() {
            let Some(segment) = segments
                .iter()
                .find(|segment| touch.time >= segment.start_time && touch.time <= segment.end_time)
            else {
                continue;
            };
            if segment
                .goal_time
                .is_some_and(|goal_time| touch.time > goal_time)
            {
                continue;
            }
            let Some(features) = touch.pre_touch_features else {
                continue;
            };
            let mut line = format!(
                "{metadata_prefix},{},{:.4}",
                u8::from(is_team_0),
                touch.time
            );
            for value in features.to_entity_array() {
                line.push_str(&format!(",{value:.6}"));
            }
            let next_touch = team_touches
                .get(index + 1)
                .filter(|next| next.time <= segment.end_time)
                .map(|next| format!("{:.4}", next.time - touch.time))
                .unwrap_or_default();
            line.push_str(&format!(
                ",{},{},{:.4},{:.4},{},{}",
                segment_goal_time(segment, touch.time, is_team_0),
                segment_goal_time(segment, touch.time, !is_team_0),
                (replay_end_time - touch.time).max(0.0),
                (segment.end_time - touch.time).max(0.0),
                u8::from(segment.resolved),
                next_touch
            ));
            touch_lines.push(line);
        }
    }

    // Per-team episode summary through the exact shipping episode state
    // machine: counts, episode threat integrals, and peaks come straight off
    // the calculator's emitted episode events (and its full-match integral
    // state), never recomputed here.
    let episode_summary_lines = [true, false]
        .into_iter()
        .map(|is_team_0| {
            let episodes = episodes.iter()
                .filter(|episode| episode.team_is_team_0 == is_team_0);
            let episode_count = episodes.clone().count();
            let episode_threat_integral: f64 = episodes.clone().map(|episode| f64::from(episode.threat_integral)).sum();
            let peak_sum: f64 = episodes.map(|episode| f64::from(episode.peak_value)).sum();
            let threat_integral = full_integrals[usize::from(!is_team_0)];
            let touch_xg: f64 = touches.iter()
                .filter(|touch| touch.team_is_team_0 == is_team_0)
                .filter_map(|touch| touch.xg)
                .map(f64::from)
                .sum();
            let goal_count = goals
                .iter()
                .filter(|goal| goal.scoring_team_is_team_0 == is_team_0)
                .count();
            format!(
                "{},{},{episode_count},{episode_threat_integral:.6},{threat_integral:.6},{peak_sum:.6},{goal_count},{touch_xg:.6}",
                csv_field(&replay_id),
                u8::from(is_team_0),
            )
        })
        .collect();

    let audit = serde_json::json!({
        "replay_id": replay_id,
        "player_ids": meta.replay_meta.player_order().map(|player| &player.remote_id).collect::<Vec<_>>(),
        "goals": goals,
        "segments": segments,
        "touches_detected": touches.len(),
        "touches_exported": touch_lines.len(),
        "missing_pre_contact_history": touches.iter().filter(|touch| touch.pre_touch_features.is_none()).count(),
        "missing_history_lags": touches.iter().filter(|touch| touch.pre_touch_features.is_none()).map(|touch| touch.detection_time - touch.time).collect::<Vec<_>>(),
    });
    Ok(ReplayRows {
        replay_id,
        lines,
        value_min: if value_min.is_finite() {
            value_min
        } else {
            0.0
        },
        value_max: if value_max.is_finite() {
            value_max
        } else {
            0.0
        },
        goal_count: goals.len(),
        episode_summary_lines,
        touch_lines,
        audit,
    })
}

fn segment_goal_time(
    segment: &subtr_actor::ThreatLiveSegment,
    time: f32,
    is_team_0: bool,
) -> String {
    if segment.scoring_team_is_team_0 == Some(is_team_0) {
        format!(
            "{:.4}",
            (segment
                .goal_time
                .unwrap_or(segment.end_time)
                .min(segment.end_time)
                - time)
                .max(0.0)
        )
    } else {
        String::new()
    }
}

fn optional_int(value: Option<i64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
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
