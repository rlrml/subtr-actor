use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use subtr_actor::ballchasing::parse_replay_file;
use subtr_actor::{BallFrame, DodgeRefreshedEvent, PlayerFrame, PlayerId, ReplayDataCollector};

const DEFAULT_INPUT_DIR: &str = "data/flip-reset-ground-truth-exact/replays";
const DEFAULT_OUTPUT_PATH: &str =
    "data/flip-reset-ground-truth-exact/flip-reset-playlist-manifest.json";
const DEFAULT_MAX_REPLAYS: usize = 30;
const DEFAULT_BEFORE_SECONDS: f32 = 5.0;
const DEFAULT_AFTER_SECONDS: f32 = 5.0;
const BALL_RADIUS_UU: f32 = 93.0;

#[derive(Debug, Clone)]
struct Config {
    input_dir: PathBuf,
    output_path: PathBuf,
    max_replays: usize,
    before_seconds: f32,
    after_seconds: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::from(DEFAULT_INPUT_DIR),
            output_path: PathBuf::from(DEFAULT_OUTPUT_PATH),
            max_replays: DEFAULT_MAX_REPLAYS,
            before_seconds: DEFAULT_BEFORE_SECONDS,
            after_seconds: DEFAULT_AFTER_SECONDS,
        }
    }
}

impl Config {
    fn from_args() -> Result<Self> {
        let mut config = Self::default();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--input-dir" => {
                    config.input_dir =
                        PathBuf::from(args.next().context("Expected a value after --input-dir")?);
                }
                "--output-path" => {
                    config.output_path = PathBuf::from(
                        args.next()
                            .context("Expected a value after --output-path")?,
                    );
                }
                "--max-replays" => {
                    config.max_replays = args
                        .next()
                        .context("Expected a value after --max-replays")?
                        .parse()
                        .context("Failed to parse --max-replays as an integer")?;
                }
                "--before-seconds" => {
                    config.before_seconds = args
                        .next()
                        .context("Expected a value after --before-seconds")?
                        .parse()
                        .context("Failed to parse --before-seconds as a number")?;
                }
                "--after-seconds" => {
                    config.after_seconds = args
                        .next()
                        .context("Expected a value after --after-seconds")?
                        .parse()
                        .context("Failed to parse --after-seconds as a number")?;
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => anyhow::bail!("Unrecognized argument: {other}"),
            }
        }

        if config.max_replays == 0 {
            anyhow::bail!("--max-replays must be greater than zero");
        }
        if config.before_seconds < 0.0 || config.after_seconds < 0.0 {
            anyhow::bail!("clip windows must be non-negative");
        }

        Ok(config)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct ManifestBound {
    kind: &'static str,
    value: f32,
}

#[derive(Debug, Clone, Serialize)]
struct ManifestReplay {
    id: String,
    path: String,
    label: String,
}

#[derive(Debug, Clone, Serialize)]
struct ManifestItem {
    replay: String,
    start: ManifestBound,
    end: ManifestBound,
    label: String,
    meta: ClipMeta,
}

#[derive(Debug, Clone, Serialize)]
struct ManifestMeta {
    mechanic: &'static str,
    source: &'static str,
    selected_replay_count: usize,
    selected_clip_count: usize,
    before_seconds: f32,
    after_seconds: f32,
    input_dir: String,
}

#[derive(Debug, Clone, Serialize)]
struct PlaylistManifest {
    label: String,
    replays: Vec<ManifestReplay>,
    items: Vec<ManifestItem>,
    meta: ManifestMeta,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Vec3Data {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Serialize)]
struct ClipMeta {
    mechanic: &'static str,
    source: &'static str,
    replay_id: String,
    replay_path: String,
    event_index: usize,
    event_frame: usize,
    event_time: f32,
    counter_value: i32,
    player_id: String,
    player_name: Option<String>,
    is_team_zero: bool,
    ball_position: Option<Vec3Data>,
    player_position: Option<Vec3Data>,
    marker_position: Option<Vec3Data>,
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run --bin build_flip_reset_playlist_manifest -- [--input-dir PATH] [--output-path PATH] [--max-replays N] [--before-seconds N] [--after-seconds N]"
    );
}

fn list_replay_paths(input_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut replay_paths = fs::read_dir(input_dir)
        .with_context(|| format!("Failed to read replay directory: {}", input_dir.display()))?
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry.path()),
            Err(_) => None,
        })
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "replay")
        })
        .collect::<Vec<_>>();
    replay_paths.sort();
    Ok(replay_paths)
}

fn replay_id_from_path(path: &Path) -> Result<String> {
    Ok(path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(ToOwned::to_owned)
        .with_context(|| {
            format!(
                "Replay path is missing a valid file stem: {}",
                path.display()
            )
        })?)
}

fn relative_replay_path(replay_path: &Path, output_path: &Path) -> String {
    let manifest_parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    if let Ok(relative) = replay_path.strip_prefix(manifest_parent) {
        return relative.to_string_lossy().into_owned();
    }

    replay_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| replay_path.to_string_lossy().into_owned())
}

fn bound_at_time(value: f32) -> ManifestBound {
    ManifestBound {
        kind: "time",
        value,
    }
}

fn vec3_from_boxcars(value: &boxcars::Vector3f) -> Vec3Data {
    Vec3Data {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

fn format_player_id(player_id: &PlayerId) -> String {
    let Ok(value) = serde_json::to_value(player_id) else {
        return format!("{player_id:?}");
    };

    if let Some((kind, entry_value)) = value.as_object().and_then(|entries| entries.iter().next()) {
        return match entry_value {
            Value::String(inner) => format!("{kind}:{inner}"),
            other => format!("{kind}:{}", other),
        };
    }

    format!("{player_id:?}")
}

fn find_player_name(replay_data: &subtr_actor::ReplayData, player_id: &PlayerId) -> Option<String> {
    let formatted_player_id = format_player_id(player_id);
    replay_data
        .meta
        .player_order()
        .find(|player_info| format_player_id(&player_info.remote_id) == formatted_player_id)
        .map(|player_info| player_info.name.clone())
}

fn get_ball_position(
    replay_data: &subtr_actor::ReplayData,
    frame_index: usize,
) -> Option<Vec3Data> {
    match replay_data.frame_data.ball_data.frames().get(frame_index)? {
        BallFrame::Data { rigid_body } => Some(vec3_from_boxcars(&rigid_body.location)),
        BallFrame::Empty => None,
    }
}

fn get_player_position(
    replay_data: &subtr_actor::ReplayData,
    player_id: &PlayerId,
    frame_index: usize,
) -> Option<Vec3Data> {
    let (_, player_data) = replay_data
        .frame_data
        .players
        .iter()
        .find(|(candidate_id, _)| candidate_id == player_id)?;

    match player_data.frames().get(frame_index)? {
        PlayerFrame::Data { rigid_body, .. } => Some(vec3_from_boxcars(&rigid_body.location)),
        PlayerFrame::Empty => None,
    }
}

fn derive_marker_position(
    ball_position: Option<Vec3Data>,
    player_position: Option<Vec3Data>,
) -> Option<Vec3Data> {
    let ball = ball_position?;
    let player = player_position?;
    let offset = glam::Vec3::new(player.x - ball.x, player.y - ball.y, player.z - ball.z);
    let direction = offset.try_normalize()?;

    Some(Vec3Data {
        x: ball.x + direction.x * BALL_RADIUS_UU,
        y: ball.y + direction.y * BALL_RADIUS_UU,
        z: ball.z + direction.z * BALL_RADIUS_UU,
    })
}

fn build_manifest_item(
    replay_id: &str,
    replay_path: &Path,
    replay_data: &subtr_actor::ReplayData,
    event: &DodgeRefreshedEvent,
    event_index: usize,
    config: &Config,
) -> ManifestItem {
    let duration = replay_data.frame_data.duration();
    let start_time = (event.time - config.before_seconds).max(0.0);
    let end_time = (event.time + config.after_seconds).min(duration);
    let player_id = format_player_id(&event.player);
    let player_name = find_player_name(replay_data, &event.player);
    let ball_position = get_ball_position(replay_data, event.frame);
    let player_position = get_player_position(replay_data, &event.player, event.frame);
    let marker_position = derive_marker_position(ball_position, player_position);
    let clip_label = player_name
        .as_ref()
        .map(|name| format!("{name} flip reset @ {:.2}s", event.time))
        .unwrap_or_else(|| format!("Flip reset {} @ {:.2}s", event_index + 1, event.time));

    ManifestItem {
        replay: replay_id.to_owned(),
        start: bound_at_time(start_time),
        end: bound_at_time(end_time),
        label: clip_label,
        meta: ClipMeta {
            mechanic: "flip_reset",
            source: "ground_truth_exact",
            replay_id: replay_id.to_owned(),
            replay_path: replay_path.to_string_lossy().into_owned(),
            event_index,
            event_frame: event.frame,
            event_time: event.time,
            counter_value: event.counter_value,
            player_id,
            player_name,
            is_team_zero: event.is_team_0,
            ball_position,
            player_position,
            marker_position,
        },
    }
}

fn main() -> Result<()> {
    let config = Config::from_args()?;
    let replay_paths = list_replay_paths(&config.input_dir)?;

    let mut replays = Vec::new();
    let mut items = Vec::new();

    for replay_path in replay_paths {
        if replays.len() >= config.max_replays {
            break;
        }

        let replay = parse_replay_file(&replay_path)
            .with_context(|| format!("Failed to parse replay: {}", replay_path.display()))?;
        let replay_data = ReplayDataCollector::new()
            .get_replay_data(&replay)
            .map_err(|error| error.variant)
            .with_context(|| {
                format!(
                    "Failed to collect replay data for {}",
                    replay_path.display()
                )
            })?;

        if replay_data.dodge_refreshed_events.is_empty() {
            continue;
        }

        let replay_id = replay_id_from_path(&replay_path)?;
        replays.push(ManifestReplay {
            id: replay_id.clone(),
            path: relative_replay_path(&replay_path, &config.output_path),
            label: replay_id.clone(),
        });

        for (event_index, event) in replay_data.dodge_refreshed_events.iter().enumerate() {
            items.push(build_manifest_item(
                &replay_id,
                &replay_path,
                &replay_data,
                event,
                event_index,
                &config,
            ));
        }
    }

    if replays.is_empty() {
        anyhow::bail!(
            "No positive replays with exact dodge refresh events were found in {}",
            config.input_dir.display()
        );
    }

    let manifest = PlaylistManifest {
        label: format!("Exact flip reset review ({})", replays.len()),
        meta: ManifestMeta {
            mechanic: "flip_reset",
            source: "ground_truth_exact",
            selected_replay_count: replays.len(),
            selected_clip_count: items.len(),
            before_seconds: config.before_seconds,
            after_seconds: config.after_seconds,
            input_dir: config.input_dir.to_string_lossy().into_owned(),
        },
        replays,
        items,
    };

    if let Some(parent) = config.output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(&manifest)
        .context("Failed to serialize flip reset playlist manifest")?;
    fs::write(&config.output_path, json)
        .with_context(|| format!("Failed to write {}", config.output_path.display()))?;

    println!(
        "Wrote {} clips across {} replays to {}",
        manifest.meta.selected_clip_count,
        manifest.meta.selected_replay_count,
        config.output_path.display()
    );

    Ok(())
}
