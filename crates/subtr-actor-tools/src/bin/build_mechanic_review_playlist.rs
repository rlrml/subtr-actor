use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, anyhow, bail};
use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

mod build_mechanic_review_playlist_candidates;
use build_mechanic_review_playlist_candidates::extract_candidates;
use subtr_actor::{
    Collector, GoalEvent, PlayerId, PlayerInfo, ProcessorView, ReplayMeta, ReplayProcessor,
    SubtrActorResult, TimeAdvance,
    interop::player::{
        PlaybackBound, PlaybackBoundKind, PlaylistAdvanceMode, PlaylistEndMode, PlaylistManifest,
        PlaylistManifestItem, PlaylistManifestReplay, PlaylistManifestReplayLocator,
        PlaylistPlaybackOptions,
    },
    stats::analysis_graph::collect_builtin_analysis_graph_for_replay,
};

const BALLCHASING_API_BASE_URL: &str = "https://ballchasing.com/api";
const DEFAULT_PLAYLIST: &str = "ranked-duels";
const DEFAULT_COUNT: usize = 10;
const DEFAULT_MIN_CONFIDENCE: f32 = 0.55;
const DEFAULT_BEFORE_SECONDS: f32 = 10.0;
const DEFAULT_AFTER_SECONDS: f32 = 3.5;
const DEFAULT_GOAL_LOOKAHEAD_SECONDS: f32 = 10.0;
const DEFAULT_GOAL_TAIL_SECONDS: f32 = 3.0;
const DEFAULT_MIN_CLIP_SECONDS: f32 = 8.0;
const DEFAULT_DOWNLOAD_DELAY_MS: u64 = 1100;
const DEFAULT_MECHANICS: &[&str] = &[
    "flick",
    "musty_flick",
    "one_timer",
    "air_dribble",
    "flip_reset",
    "ceiling_shot",
    "double_tap",
];
const ALL_MECHANICS: &[&str] = &[
    "flick",
    "musty_flick",
    "one_timer",
    "air_dribble",
    "flip_reset",
    "ceiling_shot",
    "double_tap",
    "speed_flip",
    "half_flip",
    "wavedash",
];

#[derive(Debug)]
struct Config {
    ids: Vec<String>,
    replay_paths: Vec<PathBuf>,
    ids_file: Option<PathBuf>,
    output: Option<PathBuf>,
    cache_dir: PathBuf,
    count: usize,
    playlist: String,
    sort_by: String,
    sort_dir: String,
    query_params: Vec<(String, String)>,
    min_confidence: f32,
    before_seconds: f32,
    after_seconds: f32,
    goal_lookahead_seconds: f32,
    goal_tail_seconds: f32,
    min_clip_seconds: f32,
    max_items: Option<usize>,
    download_delay: Duration,
    mechanics: Vec<String>,
}

impl Config {
    fn from_args(args: Args) -> anyhow::Result<Self> {
        if args.list_mechanics {
            println!("{}", ALL_MECHANICS.join("\n"));
            std::process::exit(0);
        }

        let mut mechanics = args.mechanic;
        mechanics.extend(args.mechanics);

        let config = Self {
            ids: args.ids,
            replay_paths: args.replay_paths,
            ids_file: args.ids_file,
            output: args.output,
            cache_dir: args.cache_dir,
            count: args.count,
            playlist: args.playlist,
            sort_by: args.sort_by,
            sort_dir: args.sort_dir,
            query_params: args.query_params,
            min_confidence: args.min_confidence,
            before_seconds: args.before_seconds,
            after_seconds: args.after_seconds,
            goal_lookahead_seconds: args.goal_lookahead_seconds,
            goal_tail_seconds: args.goal_tail_seconds,
            min_clip_seconds: args.min_clip_seconds,
            max_items: args.max_items,
            download_delay: Duration::from_millis(args.download_delay_ms),
            mechanics,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.count == 0 {
            bail!("--count must be at least 1");
        }
        if self.before_seconds < 0.0
            || self.after_seconds < 0.0
            || self.goal_lookahead_seconds < 0.0
            || self.goal_tail_seconds < 0.0
            || self.min_clip_seconds < 0.0
        {
            bail!("clip padding must be non-negative");
        }
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[command(about = "Build a mechanic-review playlist from heuristic mechanic events.")]
struct Args {
    /// Add one Ballchasing replay id or URL.
    #[arg(long = "id", value_name = "ballchasing-id-or-url")]
    ids: Vec<String>,

    /// Add Ballchasing replay ids or URLs from a file, one per line.
    #[arg(long, value_name = "path")]
    ids_file: Option<PathBuf>,

    /// Add a local .replay file.
    #[arg(long = "replay-path", value_name = "path")]
    replay_paths: Vec<PathBuf>,

    /// Write playlist JSON to path. Defaults to stdout.
    #[arg(short, long, value_name = "path")]
    output: Option<PathBuf>,

    /// Replay cache directory.
    #[arg(
        long,
        value_name = "path",
        default_value = ".cache/mechanic-review-replays"
    )]
    cache_dir: PathBuf,

    /// Number of Ballchasing replays to search/download when no sources are given.
    #[arg(long, default_value_t = DEFAULT_COUNT)]
    count: usize,

    /// Ballchasing playlist filter.
    #[arg(long, default_value = DEFAULT_PLAYLIST)]
    playlist: String,

    /// Ballchasing sort field.
    #[arg(long, default_value = "replay-date")]
    sort_by: String,

    /// Ballchasing sort direction.
    #[arg(long, default_value = "desc", value_name = "asc|desc")]
    sort_dir: String,

    /// Extra Ballchasing /replays query param. Repeatable.
    #[arg(long = "query", value_name = "key=value", value_parser = parse_query_param)]
    query_params: Vec<(String, String)>,

    /// Minimum detector confidence for scored events.
    #[arg(long, default_value_t = DEFAULT_MIN_CONFIDENCE)]
    min_confidence: f32,

    /// Clip lead-in before setup start.
    #[arg(long, default_value_t = DEFAULT_BEFORE_SECONDS)]
    before_seconds: f32,

    /// Clip tail after mechanic event.
    #[arg(long, default_value_t = DEFAULT_AFTER_SECONDS)]
    after_seconds: f32,

    /// Extend clips through same-team goals this many seconds after the mechanic event.
    #[arg(long, default_value_t = DEFAULT_GOAL_LOOKAHEAD_SECONDS)]
    goal_lookahead_seconds: f32,

    /// Clip tail after an included goal explosion.
    #[arg(long, default_value_t = DEFAULT_GOAL_TAIL_SECONDS)]
    goal_tail_seconds: f32,

    /// Minimum emitted clip duration, extended within replay bounds.
    #[arg(long, default_value_t = DEFAULT_MIN_CLIP_SECONDS)]
    min_clip_seconds: f32,

    /// Limit emitted candidates.
    #[arg(long)]
    max_items: Option<usize>,

    /// Delay between uncached Ballchasing downloads.
    #[arg(long, default_value_t = DEFAULT_DOWNLOAD_DELAY_MS)]
    download_delay_ms: u64,

    /// Include a mechanic detector. Repeatable.
    #[arg(long = "mechanic", value_name = "name")]
    mechanic: Vec<String>,

    /// Include comma-separated mechanic detectors.
    #[arg(long = "mechanics", value_name = "a,b,c", value_delimiter = ',')]
    mechanics: Vec<String>,

    /// Print supported mechanic detector names.
    #[arg(long)]
    list_mechanics: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            ids: Vec::new(),
            replay_paths: Vec::new(),
            ids_file: None,
            output: None,
            cache_dir: PathBuf::from(".cache/mechanic-review-replays"),
            count: DEFAULT_COUNT,
            playlist: DEFAULT_PLAYLIST.to_owned(),
            sort_by: "replay-date".to_owned(),
            sort_dir: "desc".to_owned(),
            query_params: Vec::new(),
            min_confidence: DEFAULT_MIN_CONFIDENCE,
            before_seconds: DEFAULT_BEFORE_SECONDS,
            after_seconds: DEFAULT_AFTER_SECONDS,
            goal_lookahead_seconds: DEFAULT_GOAL_LOOKAHEAD_SECONDS,
            goal_tail_seconds: DEFAULT_GOAL_TAIL_SECONDS,
            min_clip_seconds: DEFAULT_MIN_CLIP_SECONDS,
            max_items: None,
            download_delay_ms: DEFAULT_DOWNLOAD_DELAY_MS,
            mechanic: Vec::new(),
            mechanics: Vec::new(),
            list_mechanics: false,
        }
    }
}

#[derive(Debug, Clone)]
struct ReplaySourceInput {
    source_id: String,
    locator: PlaylistManifestReplayLocator,
    bytes_path: PathBuf,
    label: String,
    meta: Value,
}

#[derive(Debug, Deserialize)]
struct BallchasingReplayList {
    list: Vec<BallchasingReplaySummary>,
}

#[derive(Debug, Deserialize)]
struct BallchasingReplaySummary {
    id: String,
    #[serde(default)]
    replay_title: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    playlist_id: Option<String>,
    #[serde(default)]
    playlist_name: Option<String>,
    #[serde(default)]
    duration: Option<f32>,
}

#[derive(Clone)]
struct PlayerDisplay {
    name: String,
    team: &'static str,
}

#[derive(Debug, Clone)]
struct MechanicCandidate {
    mechanic: &'static str,
    mechanic_label: &'static str,
    detector: &'static str,
    player_id: Option<String>,
    is_team_0: Option<bool>,
    event_time: f32,
    event_frame: usize,
    start_time: f32,
    end_time: f32,
    confidence: Option<f32>,
    reason: String,
    event: Value,
}

struct GoalScanCollector;

impl Collector for GoalScanCollector {
    fn process_frame(
        &mut self,
        _processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        Ok(TimeAdvance::NextFrame)
    }
}

fn parse_args() -> anyhow::Result<Config> {
    Config::from_args(Args::parse())
}

fn parse_query_param(raw: &str) -> Result<(String, String), String> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| "--query expects key=value".to_owned())?;
    Ok((key.to_owned(), value.to_owned()))
}

fn normalize_ballchasing_id(input: &str) -> String {
    input
        .trim()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(input)
        .split('?')
        .next()
        .unwrap_or(input)
        .to_ascii_lowercase()
}

fn load_ids_file(path: &Path) -> anyhow::Result<Vec<String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read ids file {}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(normalize_ballchasing_id)
        .collect())
}

fn ballchasing_api_key() -> anyhow::Result<String> {
    std::env::var("BALLCHASING_API_KEY")
        .context("BALLCHASING_API_KEY must be set for Ballchasing API calls")
}

fn search_ballchasing_replays(
    client: &Client,
    api_key: &str,
    config: &Config,
) -> anyhow::Result<Vec<BallchasingReplaySummary>> {
    let mut request = client
        .get(format!("{BALLCHASING_API_BASE_URL}/replays"))
        .header("Authorization", api_key)
        .query(&[
            ("playlist", config.playlist.as_str()),
            ("count", &config.count.to_string()),
            ("sort-by", config.sort_by.as_str()),
            ("sort-dir", config.sort_dir.as_str()),
        ]);

    for (key, value) in &config.query_params {
        request = request.query(&[(key.as_str(), value.as_str())]);
    }

    let response = request
        .send()
        .context("failed to search Ballchasing replays")?
        .error_for_status()
        .context("Ballchasing replay search returned an error")?
        .json::<BallchasingReplayList>()
        .context("failed to decode Ballchasing replay search response")?;
    Ok(response.list)
}

fn download_ballchasing_replay(
    client: &Client,
    api_key: &str,
    replay_id: &str,
    path: &Path,
) -> anyhow::Result<()> {
    let bytes = client
        .get(format!(
            "{BALLCHASING_API_BASE_URL}/replays/{replay_id}/file"
        ))
        .header("Authorization", api_key)
        .send()
        .with_context(|| format!("failed to download Ballchasing replay {replay_id}"))?
        .error_for_status()
        .with_context(|| format!("Ballchasing replay download failed for {replay_id}"))?
        .bytes()
        .with_context(|| format!("failed to read replay bytes for {replay_id}"))?;
    std::fs::write(path, &bytes)
        .with_context(|| format!("failed to write replay cache {}", path.display()))?;
    Ok(())
}

fn collect_sources(config: &Config) -> anyhow::Result<Vec<ReplaySourceInput>> {
    std::fs::create_dir_all(&config.cache_dir)
        .with_context(|| format!("failed to create cache dir {}", config.cache_dir.display()))?;
    let cache_dir = std::fs::canonicalize(&config.cache_dir).with_context(|| {
        format!(
            "failed to canonicalize cache dir {}",
            config.cache_dir.display()
        )
    })?;
    let client = Client::new();

    let mut ids = config.ids.clone();
    if let Some(ids_file) = &config.ids_file {
        ids.extend(load_ids_file(ids_file)?);
    }

    let summaries = if ids.is_empty() && config.replay_paths.is_empty() {
        let api_key = ballchasing_api_key()?;
        search_ballchasing_replays(&client, &api_key, config)?
    } else {
        ids.into_iter()
            .map(|id| BallchasingReplaySummary {
                id: normalize_ballchasing_id(&id),
                replay_title: None,
                date: None,
                playlist_id: None,
                playlist_name: None,
                duration: None,
            })
            .collect()
    };

    let mut sources = Vec::new();
    let mut api_key = None;

    for (index, summary) in summaries.into_iter().enumerate() {
        let replay_id = normalize_ballchasing_id(&summary.id);
        let cache_path = cache_dir.join(format!("ballchasing-{replay_id}.replay"));
        if !cache_path.exists() {
            let key = match &api_key {
                Some(key) => key,
                None => {
                    api_key = Some(ballchasing_api_key()?);
                    api_key.as_ref().expect("api key just set")
                }
            };
            download_ballchasing_replay(&client, key, &replay_id, &cache_path)?;
            if index + 1 < config.count {
                std::thread::sleep(config.download_delay);
            }
        }

        let label = summary
            .replay_title
            .clone()
            .unwrap_or_else(|| format!("Ballchasing {replay_id}"));
        sources.push(ReplaySourceInput {
            source_id: format!("ballchasing:{replay_id}"),
            locator: PlaylistManifestReplayLocator::ballchasing(
                replay_id.clone(),
                cache_path.display().to_string(),
            ),
            bytes_path: cache_path.clone(),
            label,
            meta: serde_json::to_value(summary_meta(&summary))?,
        });
    }

    for path in &config.replay_paths {
        let canonical = std::fs::canonicalize(path)
            .with_context(|| format!("failed to canonicalize replay path {}", path.display()))?;
        let label = canonical
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("local replay")
            .to_owned();
        sources.push(ReplaySourceInput {
            source_id: format!("path:{}", canonical.display()),
            locator: PlaylistManifestReplayLocator::path(canonical.display().to_string()),
            bytes_path: canonical.clone(),
            label,
            meta: json!({ "source": "local" }),
        });
    }

    Ok(sources)
}

fn summary_meta(summary: &BallchasingReplaySummary) -> Value {
    json!({
        "source": "ballchasing",
        "ballchasingId": summary.id,
        "title": summary.replay_title,
        "date": summary.date,
        "playlistId": summary.playlist_id,
        "playlistName": summary.playlist_name,
        "duration": summary.duration,
    })
}

fn parse_replay_file(path: &Path) -> anyhow::Result<boxcars::Replay> {
    let data =
        std::fs::read(path).with_context(|| format!("failed to read replay {}", path.display()))?;
    boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .always_check_crc()
        .parse()
        .with_context(|| format!("failed to parse replay {}", path.display()))
}

fn player_id_string(player_id: &PlayerId) -> String {
    match serde_json::to_value(player_id) {
        Ok(Value::Object(map)) if map.len() == 1 => {
            let (kind, value) = map.into_iter().next().expect("map has one value");
            let platform = player_id_platform_label(&kind);
            let id = player_id_value_text(&value);
            format!("{platform}:{id}")
        }
        Ok(value) => value.to_string(),
        Err(_) => format!("{player_id:?}"),
    }
}

fn player_id_platform_label(kind: &str) -> &str {
    match kind {
        "PlayStation" => "ps4",
        "PsyNet" => "psynet",
        "SplitScreen" => "splitscreen",
        "Steam" => "steam",
        "Switch" => "switch",
        "Xbox" => "xbox",
        "QQ" => "qq",
        "Epic" => "epic",
        other => other,
    }
}

fn player_id_value_text(value: &Value) -> String {
    if let Some(online_id) = value
        .as_object()
        .and_then(|object| object.get("online_id"))
        .and_then(json_scalar_text)
    {
        return online_id;
    }
    json_scalar_text(value).unwrap_or_else(|| value.to_string())
}

fn json_scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn player_display_map(meta: &ReplayMeta) -> HashMap<String, PlayerDisplay> {
    meta.team_zero
        .iter()
        .map(|player| (player, "blue"))
        .chain(meta.team_one.iter().map(|player| (player, "orange")))
        .map(|(player, team)| {
            (
                player_id_string(&player.remote_id),
                player_display(player, team),
            )
        })
        .collect()
}

fn player_display(player: &PlayerInfo, team: &'static str) -> PlayerDisplay {
    PlayerDisplay {
        name: player.name.clone(),
        team,
    }
}

fn resolve_mechanics(config: &Config) -> anyhow::Result<Vec<&'static str>> {
    let requested: Vec<String> = if config.mechanics.is_empty() {
        DEFAULT_MECHANICS
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    } else {
        config.mechanics.clone()
    };

    let mut resolved = Vec::new();
    for raw in requested {
        let normalized = raw.trim().replace('-', "_").to_ascii_lowercase();
        let names: Vec<&str> = match normalized.as_str() {
            "default" => DEFAULT_MECHANICS.to_vec(),
            "all" => ALL_MECHANICS.to_vec(),
            name if ALL_MECHANICS.contains(&name) => vec![
                ALL_MECHANICS
                    .iter()
                    .copied()
                    .find(|candidate| *candidate == name)
                    .expect("mechanic is known"),
            ],
            other => bail!(
                "unknown mechanic {other}; supported mechanics are: {}, default, all",
                ALL_MECHANICS.join(", ")
            ),
        };
        for name in names {
            if !resolved.contains(&name) {
                resolved.push(name);
            }
        }
    }
    Ok(resolved)
}

fn graph_node_names_for_mechanics(mechanics: &[&str]) -> Vec<&'static str> {
    let mut names = Vec::new();
    for mechanic in mechanics {
        let node = match *mechanic {
            "flick" => Some("flick"),
            "musty_flick" => Some("musty_flick"),
            "one_timer" => Some("one_timer"),
            "air_dribble" => Some("ball_carry"),
            "ceiling_shot" => Some("ceiling_shot"),
            "double_tap" => Some("double_tap"),
            "speed_flip" => Some("speed_flip"),
            "half_flip" => Some("half_flip"),
            "wavedash" => Some("wavedash"),
            "flip_reset" => Some("dodge_reset"),
            _ => None,
        };
        if let Some(node) = node {
            if !names.contains(&node) {
                names.push(node);
            }
        }
    }
    names
}

fn confidence_pct(confidence: f32) -> u32 {
    (confidence * 100.0).round().clamp(0.0, 100.0) as u32
}

fn player_team_label(is_team_0: bool) -> &'static str {
    if is_team_0 { "blue" } else { "orange" }
}

fn followup_goal_for_candidate<'a>(
    candidate: &MechanicCandidate,
    goal_events: &'a [GoalEvent],
    config: &Config,
) -> Option<&'a GoalEvent> {
    goal_events
        .iter()
        .filter(|goal| {
            candidate
                .is_team_0
                .map(|is_team_0| goal.scoring_team_is_team_0 == is_team_0)
                .unwrap_or(true)
        })
        .filter(|goal| goal.time >= candidate.event_time)
        .filter(|goal| goal.time - candidate.event_time <= config.goal_lookahead_seconds)
        .min_by(|left, right| left.time.total_cmp(&right.time))
}

fn replay_duration_seconds(replay: &boxcars::Replay) -> f32 {
    replay
        .network_frames
        .as_ref()
        .and_then(|frames| frames.frames.last())
        .map(|frame| frame.time)
        .unwrap_or(0.0)
}

fn enforce_min_clip_duration(
    start_time: f32,
    end_time: f32,
    replay_duration: f32,
    min_clip_seconds: f32,
) -> (f32, f32) {
    let mut start_time = start_time.clamp(0.0, replay_duration.max(0.0));
    let mut end_time = end_time.clamp(start_time, replay_duration.max(start_time));
    let duration = end_time - start_time;
    if duration >= min_clip_seconds {
        return (start_time, end_time);
    }

    let missing = min_clip_seconds - duration;
    let extend_after = missing.min((replay_duration - end_time).max(0.0));
    end_time += extend_after;
    let remaining = missing - extend_after;
    start_time = (start_time - remaining).max(0.0);
    (start_time, end_time)
}

fn frame_index_at_or_after(frames: &[boxcars::Frame], time: f32) -> Option<usize> {
    if frames.is_empty() || !time.is_finite() {
        return None;
    }

    let index = frames.partition_point(|frame| frame.time < time);
    Some(index.min(frames.len().saturating_sub(1)))
}

fn playback_bounds_for_clip(
    replay: &boxcars::Replay,
    start_time: f32,
    end_time: f32,
) -> (PlaybackBound, PlaybackBound) {
    let frames = replay
        .network_frames
        .as_ref()
        .map(|network_frames| network_frames.frames.as_slice());

    if let Some(frames) = frames {
        if let (Some(start_frame), Some(end_frame)) = (
            frame_index_at_or_after(frames, start_time),
            frame_index_at_or_after(frames, end_time),
        ) {
            return (
                PlaybackBound {
                    kind: PlaybackBoundKind::Frame,
                    value: start_frame as f32,
                },
                PlaybackBound {
                    kind: PlaybackBoundKind::Frame,
                    value: end_frame.max(start_frame.saturating_add(1)) as f32,
                },
            );
        }
    }

    (
        PlaybackBound {
            kind: PlaybackBoundKind::Time,
            value: start_time,
        },
        PlaybackBound {
            kind: PlaybackBoundKind::Time,
            value: end_time,
        },
    )
}

fn event_json<T: Serialize>(event: &T) -> Value {
    serde_json::to_value(event).unwrap_or_else(|_| json!({ "serializationError": true }))
}

fn build_items_for_source(
    source: &ReplaySourceInput,
    replay: &boxcars::Replay,
    config: &Config,
    mechanics: &[&str],
) -> anyhow::Result<Vec<PlaylistManifestItem>> {
    let graph_nodes = graph_node_names_for_mechanics(mechanics);
    let graph = collect_builtin_analysis_graph_for_replay(replay, graph_nodes).map_err(|err| {
        anyhow!(
            "failed to collect mechanic stats for {}: {err:?}",
            source.label
        )
    })?;
    let mut processor = ReplayProcessor::new(replay).map_err(|err| {
        anyhow!(
            "failed to build replay processor for {}: {err:?}",
            source.label
        )
    })?;
    processor.process(&mut GoalScanCollector).map_err(|err| {
        anyhow!(
            "failed to process replay goals for {}: {err:?}",
            source.label
        )
    })?;
    let replay_meta = processor.get_replay_meta().map_err(|err| {
        anyhow!(
            "failed to read replay metadata for {}: {err:?}",
            source.label
        )
    })?;
    let replay_duration = replay_duration_seconds(replay);
    let players = player_display_map(&replay_meta);
    let candidates = extract_candidates(replay, &graph, mechanics, config)?;

    let mut items = Vec::new();
    for candidate in candidates {
        let player = candidate
            .player_id
            .as_ref()
            .and_then(|player_id| players.get(player_id));
        let player_label = player
            .map(|display| display.name.as_str())
            .or(candidate.player_id.as_deref())
            .unwrap_or("team event");
        let start_time = (candidate.start_time - config.before_seconds).max(0.0);
        let followup_goal = followup_goal_for_candidate(&candidate, &processor.goal_events, config);
        let padded_end_time = followup_goal
            .map(|goal| goal.time + config.goal_tail_seconds)
            .unwrap_or(candidate.end_time + config.after_seconds)
            .max(candidate.end_time + config.after_seconds)
            .max(start_time);
        let (start_time, end_time) = enforce_min_clip_duration(
            start_time,
            padded_end_time,
            replay_duration,
            config.min_clip_seconds,
        );
        let score = candidate
            .confidence
            .map(|confidence| format!(" {}%", confidence_pct(confidence)))
            .unwrap_or_default();
        let id = format!(
            "{}:{}:{}:{}",
            candidate.mechanic,
            source.source_id,
            candidate.event_frame,
            candidate.player_id.as_deref().unwrap_or("team")
        );
        let (start_bound, end_bound) = playback_bounds_for_clip(replay, start_time, end_time);

        items.push(PlaylistManifestItem {
            id: id.clone(),
            replay: source.source_id.clone(),
            start: start_bound,
            end: end_bound,
            label: format!("{}{score} - {player_label}", candidate.mechanic_label),
            meta: json!({
                "itemId": id,
                "mechanic": candidate.mechanic,
                "mechanicLabel": candidate.mechanic_label,
                "detector": candidate.detector,
                "confidence": candidate.confidence,
                "reason": candidate.reason,
                "playerId": candidate.player_id,
                "playerName": player.map(|display| display.name.clone()),
                "team": player.map(|display| display.team).or_else(|| candidate.is_team_0.map(player_team_label)),
                "target": {
                    "kind": "player-span",
                    "playerId": candidate.player_id,
                    "startTime": start_time,
                    "endTime": end_time,
                    "mechanicStartTime": candidate.start_time,
                    "mechanicEndTime": candidate.end_time,
                    "eventTime": candidate.event_time,
                    "eventFrame": candidate.event_frame,
                    "goalTime": followup_goal.map(|goal| goal.time),
                    "goalFrame": followup_goal.map(|goal| goal.frame),
                },
                "followupGoal": followup_goal.map(event_json),
                "event": candidate.event,
            }),
        });
    }

    Ok(items)
}

fn build_manifest(config: &Config) -> anyhow::Result<PlaylistManifest> {
    let mechanics = resolve_mechanics(config)?;
    let sources = collect_sources(config)?;
    if sources.is_empty() {
        bail!("no replay sources were selected");
    }

    let mut replays = Vec::new();
    let mut items = Vec::new();
    for source in &sources {
        let replay = parse_replay_file(&source.bytes_path)?;
        replays.push(PlaylistManifestReplay {
            id: source.source_id.clone(),
            label: source.label.clone(),
            path: source.bytes_path.display().to_string(),
            locator: source.locator.clone(),
            meta: source.meta.clone(),
        });
        items.extend(build_items_for_source(source, &replay, config, &mechanics)?);
        if let Some(max_items) = config.max_items {
            if items.len() >= max_items {
                items.truncate(max_items);
                break;
            }
        }
    }

    let candidate_count = items.len();

    Ok(PlaylistManifest {
        version: 1,
        kind: "mechanic-review-playlist".to_owned(),
        label: "Mechanic review candidates".to_owned(),
        playback: PlaylistPlaybackOptions {
            advance_mode: PlaylistAdvanceMode::Manual,
            end_mode: PlaylistEndMode::Stop,
        },
        replays,
        items,
        page: None,
        meta: json!({
            "mechanics": mechanics,
            "sourceReplayCount": sources.len(),
            "candidateCount": candidate_count,
            "minConfidence": config.min_confidence,
            "clipPadding": {
                "beforeSeconds": config.before_seconds,
                "afterSeconds": config.after_seconds,
                "goalLookaheadSeconds": config.goal_lookahead_seconds,
                "goalTailSeconds": config.goal_tail_seconds,
                "minClipSeconds": config.min_clip_seconds,
            },
            "generatedBy": "build_mechanic_review_playlist",
        }),
    })
}

fn write_manifest(manifest: &PlaylistManifest, output: Option<&Path>) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(manifest)?;
    match output {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create output dir {}", parent.display())
                    })?;
                }
            }
            std::fs::write(path, format!("{json}\n"))
                .with_context(|| format!("failed to write playlist {}", path.display()))?;
        }
        None => println!("{json}"),
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let config = parse_args()?;
    let manifest = build_manifest(&config)?;
    write_manifest(&manifest, config.output.as_deref())?;
    eprintln!(
        "wrote {} mechanic candidates across {} replays",
        manifest.items.len(),
        manifest.replays.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(time: f32) -> boxcars::Frame {
        boxcars::Frame {
            time,
            delta: 0.1,
            new_actors: Vec::new(),
            deleted_actors: Vec::new(),
            updated_actors: Vec::new(),
        }
    }

    #[test]
    fn frame_index_at_or_after_clamps_to_available_replay_frames() {
        let frames = vec![frame(8.0), frame(8.5), frame(9.0), frame(10.0)];

        assert_eq!(frame_index_at_or_after(&frames, 0.0), Some(0));
        assert_eq!(frame_index_at_or_after(&frames, 8.5), Some(1));
        assert_eq!(frame_index_at_or_after(&frames, 8.6), Some(2));
        assert_eq!(frame_index_at_or_after(&frames, 99.0), Some(3));
    }

    #[test]
    fn frame_index_at_or_after_rejects_empty_or_invalid_inputs() {
        assert_eq!(frame_index_at_or_after(&[], 8.0), None);
        assert_eq!(frame_index_at_or_after(&[frame(8.0)], f32::NAN), None);
    }

    #[test]
    fn player_id_string_extracts_online_id_from_platform_objects() {
        assert_eq!(
            player_id_string(&PlayerId::PlayStation(boxcars::Ps4Id {
                online_id: 6788998483854448235,
                name: "KvonUnknown".to_owned(),
                unknown1: vec![98, 50, 117],
            })),
            "ps4:6788998483854448235"
        );
        assert_eq!(
            player_id_string(&PlayerId::Switch(boxcars::SwitchId {
                online_id: 123456789,
                unknown1: vec![1, 2, 3],
            })),
            "switch:123456789"
        );
    }
}
