use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use subtr_actor::{
    playlist_generation::{
        PlaybackBound, PlaybackBoundKind, PlaylistAdvanceMode, PlaylistEndMode, PlaylistManifest,
        PlaylistManifestItem, PlaylistManifestReplay, PlaylistManifestReplayLocator,
        PlaylistPlaybackOptions,
    },
    stats::analysis_graph::collect_builtin_analysis_graph_for_replay,
    BallCarryCalculator, BallCarryKind, CeilingShotCalculator, Collector, DodgeResetCalculator,
    DoubleTapCalculator, FlickCalculator, FlipResetTracker, GoalEvent, MustyFlickCalculator,
    OneTimerCalculator, PlayerId, PlayerInfo, ReplayMeta, ReplayProcessor, SpeedFlipCalculator,
    SubtrActorResult, TimeAdvance, WavedashCalculator,
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

impl Default for Config {
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
            download_delay: Duration::from_millis(DEFAULT_DOWNLOAD_DELAY_MS),
            mechanics: Vec::new(),
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
        _processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        Ok(TimeAdvance::NextFrame)
    }
}

fn usage() -> &'static str {
    "Usage:
  cargo run -p subtr-actor-tools --bin build_mechanic_review_playlist -- [options]

Build a mechanic-review playlist from configurable heuristic mechanic events.

Sources:
  --id <ballchasing-id-or-url>       Add one Ballchasing replay.
  --ids-file <path>                  Add Ballchasing replay ids/URLs, one per line.
  --replay-path <path>               Add a local .replay file.

When no sources are given, the tool searches Ballchasing for recent 1v1 replays:
  --count <n>                        Number of Ballchasing replays to search/download. Default: 10.
  --playlist <playlist>              Ballchasing playlist filter. Default: ranked-duels.
  --query <key=value>                Extra Ballchasing /replays query param. Repeatable.
  --sort-by <field>                  Ballchasing sort field. Default: replay-date.
  --sort-dir <asc|desc>              Ballchasing sort direction. Default: desc.

Output:
  --output <path>                    Write playlist JSON to path. Defaults to stdout.
  --cache-dir <path>                 Replay cache directory. Default: .cache/mechanic-review-replays.
  --min-confidence <f32>             Minimum detector confidence for scored events. Default: 0.55.
  --before-seconds <f32>             Clip lead-in before setup start. Default: 10.
  --after-seconds <f32>              Clip tail after flick event. Default: 3.5.
  --goal-lookahead-seconds <f32>     Extend clips through same-team goals this many seconds after the mechanic event. Default: 10.
  --goal-tail-seconds <f32>          Clip tail after an included goal explosion. Default: 3.
  --min-clip-seconds <f32>           Minimum emitted clip duration, extended within replay bounds. Default: 8.
  --max-items <n>                    Limit emitted candidates.
  --download-delay-ms <n>            Delay between uncached Ballchasing downloads. Default: 1100.
  --mechanic <name>                  Include a mechanic detector. Repeatable. Default: core review set.
  --mechanics <a,b,c>                Include comma-separated mechanic detectors.
  --list-mechanics                   Print supported mechanic detector names.
  --help                            Show this help.

Supported mechanics:
  flick, musty_flick, one_timer, air_dribble, flip_reset, ceiling_shot,
  double_tap, speed_flip, wavedash, default, all.

Ballchasing API calls require BALLCHASING_API_KEY."
}

fn parse_args() -> anyhow::Result<Config> {
    let mut config = Config::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{}", usage());
                std::process::exit(0);
            }
            "--id" => config.ids.push(require_arg(&mut args, "--id")?),
            "--ids-file" => {
                config.ids_file = Some(PathBuf::from(require_arg(&mut args, "--ids-file")?))
            }
            "--replay-path" => config
                .replay_paths
                .push(PathBuf::from(require_arg(&mut args, "--replay-path")?)),
            "--output" | "-o" => {
                config.output = Some(PathBuf::from(require_arg(&mut args, "--output")?))
            }
            "--cache-dir" => {
                config.cache_dir = PathBuf::from(require_arg(&mut args, "--cache-dir")?)
            }
            "--count" => config.count = parse_arg(&mut args, "--count")?,
            "--playlist" => config.playlist = require_arg(&mut args, "--playlist")?,
            "--sort-by" => config.sort_by = require_arg(&mut args, "--sort-by")?,
            "--sort-dir" => config.sort_dir = require_arg(&mut args, "--sort-dir")?,
            "--query" => {
                let raw = require_arg(&mut args, "--query")?;
                let (key, value) = raw
                    .split_once('=')
                    .ok_or_else(|| anyhow!("--query expects key=value"))?;
                config.query_params.push((key.to_owned(), value.to_owned()));
            }
            "--min-confidence" => config.min_confidence = parse_arg(&mut args, "--min-confidence")?,
            "--before-seconds" => config.before_seconds = parse_arg(&mut args, "--before-seconds")?,
            "--after-seconds" => config.after_seconds = parse_arg(&mut args, "--after-seconds")?,
            "--goal-lookahead-seconds" => {
                config.goal_lookahead_seconds = parse_arg(&mut args, "--goal-lookahead-seconds")?
            }
            "--goal-tail-seconds" => {
                config.goal_tail_seconds = parse_arg(&mut args, "--goal-tail-seconds")?
            }
            "--min-clip-seconds" => {
                config.min_clip_seconds = parse_arg(&mut args, "--min-clip-seconds")?
            }
            "--max-items" => config.max_items = Some(parse_arg(&mut args, "--max-items")?),
            "--download-delay-ms" => {
                let delay_ms: u64 = parse_arg(&mut args, "--download-delay-ms")?;
                config.download_delay = Duration::from_millis(delay_ms);
            }
            "--mechanic" => config.mechanics.push(require_arg(&mut args, "--mechanic")?),
            "--mechanics" => {
                let raw = require_arg(&mut args, "--mechanics")?;
                config.mechanics.extend(
                    raw.split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_owned),
                );
            }
            "--list-mechanics" => {
                println!("{}", ALL_MECHANICS.join("\n"));
                std::process::exit(0);
            }
            other => bail!("Unknown argument {other}\n\n{}", usage()),
        }
    }

    if config.count == 0 {
        bail!("--count must be at least 1");
    }
    if config.before_seconds < 0.0
        || config.after_seconds < 0.0
        || config.goal_lookahead_seconds < 0.0
        || config.goal_tail_seconds < 0.0
        || config.min_clip_seconds < 0.0
    {
        bail!("clip padding must be non-negative");
    }
    Ok(config)
}

fn require_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> anyhow::Result<String> {
    args.next()
        .with_context(|| format!("{flag} requires a value"))
}

fn parse_arg<T>(args: &mut impl Iterator<Item = String>, flag: &str) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let raw = require_arg(args, flag)?;
    raw.parse::<T>()
        .map_err(|err| anyhow!("invalid value for {flag}: {err}"))
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
            "{BALLCHASING_API_BASE_URL}/replays/{}/file",
            replay_id
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
            meta: serde_json::to_value(&summary_meta(&summary))?,
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
            match value {
                Value::String(value) => format!("{kind}:{value}"),
                other => format!("{kind}:{other}"),
            }
        }
        Ok(value) => value.to_string(),
        Err(_) => format!("{player_id:?}"),
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
            name if ALL_MECHANICS.contains(&name) => vec![ALL_MECHANICS
                .iter()
                .copied()
                .find(|candidate| *candidate == name)
                .expect("mechanic is known")],
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
    if is_team_0 {
        "blue"
    } else {
        "orange"
    }
}

fn include_candidate(candidate: &MechanicCandidate, config: &Config) -> bool {
    candidate
        .confidence
        .map(|confidence| confidence >= config.min_confidence)
        .unwrap_or(true)
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

fn event_json<T: Serialize>(event: &T) -> Value {
    serde_json::to_value(event).unwrap_or_else(|_| json!({ "serializationError": true }))
}

fn extract_candidates(
    replay: &boxcars::Replay,
    graph: &subtr_actor::stats::analysis_graph::AnalysisGraph,
    mechanics: &[&str],
    config: &Config,
) -> anyhow::Result<Vec<MechanicCandidate>> {
    let mut candidates = Vec::new();

    for mechanic in mechanics {
        match *mechanic {
            "flick" => {
                let Some(calculator) = graph.state::<FlickCalculator>() else {
                    continue;
                };
                candidates.extend(calculator.events().iter().map(|event| {
                    let confidence = event.confidence;
                    MechanicCandidate {
                        mechanic: "flick",
                        mechanic_label: "Flick",
                        detector: "builtin:flick",
                        player_id: Some(player_id_string(&event.player)),
                        is_team_0: Some(event.is_team_0),
                        event_time: event.time,
                        event_frame: event.frame,
                        start_time: event.setup_start_time,
                        end_time: event.time,
                        confidence: Some(confidence),
                        reason: format!(
                            "{}% confidence; {:.1}s setup with {} touches; ball speed +{:.0}",
                            confidence_pct(confidence),
                            event.setup_duration,
                            event.setup_touch_count,
                            event.ball_speed_change
                        ),
                        event: event_json(event),
                    }
                }));
            }
            "musty_flick" => {
                let Some(calculator) = graph.state::<MustyFlickCalculator>() else {
                    continue;
                };
                candidates.extend(calculator.events().iter().map(|event| {
                    let confidence = event.confidence;
                    MechanicCandidate {
                        mechanic: "musty_flick",
                        mechanic_label: "Musty Flick",
                        detector: "builtin:musty_flick",
                        player_id: Some(player_id_string(&event.player)),
                        is_team_0: Some(event.is_team_0),
                        event_time: event.time,
                        event_frame: event.frame,
                        start_time: event.dodge_time,
                        end_time: event.time,
                        confidence: Some(confidence),
                        reason: format!(
                            "{}% confidence; dodge-to-touch {:.2}s; pitch rate {:.1}; ball speed +{:.0}",
                            confidence_pct(confidence),
                            event.time_since_dodge,
                            event.pitch_rate,
                            event.ball_speed_change
                        ),
                        event: event_json(event),
                    }
                }));
            }
            "one_timer" => {
                let Some(calculator) = graph.state::<OneTimerCalculator>() else {
                    continue;
                };
                candidates.extend(calculator.events().iter().map(|event| MechanicCandidate {
                    mechanic: "one_timer",
                    mechanic_label: "One Timer",
                    detector: "builtin:one_timer",
                    player_id: Some(player_id_string(&event.player)),
                    is_team_0: Some(event.is_team_0),
                    event_time: event.time,
                    event_frame: event.frame,
                    start_time: event.pass_start_time,
                    end_time: event.time,
                    confidence: None,
                    reason: format!(
                        "pass from {}; {:.1}s pass, {:.0}uu travel, {:.0}uu/s shot, {:.2} goal alignment",
                        player_id_string(&event.passer),
                        event.pass_duration,
                        event.pass_travel_distance,
                        event.ball_speed,
                        event.goal_alignment
                    ),
                    event: event_json(event),
                }));
            }
            "air_dribble" => {
                let Some(calculator) = graph.state::<BallCarryCalculator>() else {
                    continue;
                };
                candidates.extend(
                    calculator
                        .carry_events()
                        .iter()
                        .filter(|event| event.kind == BallCarryKind::AirDribble)
                        .map(|event| MechanicCandidate {
                            mechanic: "air_dribble",
                            mechanic_label: "Air Dribble",
                            detector: "builtin:ball_carry",
                            player_id: Some(player_id_string(&event.player_id)),
                            is_team_0: Some(event.is_team_0),
                            event_time: event.end_time,
                            event_frame: event.end_frame,
                            start_time: event.start_time,
                            end_time: event.end_time,
                            confidence: None,
                            reason: format!(
                                "{:.1}s airborne control; {:.0}uu path; avg gap {:.0}h/{:.0}v",
                                event.duration,
                                event.path_distance,
                                event.average_horizontal_gap,
                                event.average_vertical_gap
                            ),
                            event: event_json(event),
                        }),
                );
            }
            "flip_reset" => {
                let tracker = FlipResetTracker::new()
                    .process_replay(replay)
                    .map_err(|err| {
                        anyhow!("failed to collect flip reset tracker events: {err:?}")
                    })?;
                candidates.extend(tracker.flip_reset_events().iter().map(|event| {
                    let confidence = event.confidence;
                    MechanicCandidate {
                        mechanic: "flip_reset",
                        mechanic_label: "Flip Reset",
                        detector: "builtin:flip_reset_tracker",
                        player_id: Some(player_id_string(&event.player)),
                        is_team_0: Some(event.is_team_0),
                        event_time: event.time,
                        event_frame: event.frame,
                        start_time: event.time,
                        end_time: event.time,
                        confidence: Some(confidence),
                        reason: format!(
                            "{}% confidence; underside ball contact; closest approach {:.0}uu",
                            confidence_pct(confidence),
                            event.closest_approach_distance
                        ),
                        event: event_json(event),
                    }
                }));

                if let Some(calculator) = graph.state::<DodgeResetCalculator>() {
                    candidates.extend(calculator.on_ball_events().iter().map(|event| {
                        MechanicCandidate {
                            mechanic: "flip_reset",
                            mechanic_label: "Flip Reset",
                            detector: "builtin:dodge_reset:on_ball",
                            player_id: Some(player_id_string(&event.player)),
                            is_team_0: Some(event.is_team_0),
                            event_time: event.time,
                            event_frame: event.frame,
                            start_time: event.time,
                            end_time: event.time,
                            confidence: None,
                            reason: format!(
                                "dodge refresh while close to the ball; counter value {}",
                                event.counter_value
                            ),
                            event: event_json(event),
                        }
                    }));
                }
            }
            "ceiling_shot" => {
                let Some(calculator) = graph.state::<CeilingShotCalculator>() else {
                    continue;
                };
                candidates.extend(calculator.events().iter().map(|event| {
                    let confidence = event.confidence;
                    MechanicCandidate {
                        mechanic: "ceiling_shot",
                        mechanic_label: "Ceiling Shot",
                        detector: "builtin:ceiling_shot",
                        player_id: Some(player_id_string(&event.player)),
                        is_team_0: Some(event.is_team_0),
                        event_time: event.time,
                        event_frame: event.frame,
                        start_time: event.ceiling_contact_time,
                        end_time: event.time,
                        confidence: Some(confidence),
                        reason: format!(
                            "{}% confidence; touch {:.2}s after ceiling; ball speed +{:.0}",
                            confidence_pct(confidence),
                            event.time_since_ceiling_contact,
                            event.ball_speed_change
                        ),
                        event: event_json(event),
                    }
                }));
            }
            "double_tap" => {
                let Some(calculator) = graph.state::<DoubleTapCalculator>() else {
                    continue;
                };
                candidates.extend(calculator.events().iter().map(|event| MechanicCandidate {
                    mechanic: "double_tap",
                    mechanic_label: "Double Tap",
                    detector: "builtin:double_tap",
                    player_id: Some(player_id_string(&event.player)),
                    is_team_0: Some(event.is_team_0),
                    event_time: event.time,
                    event_frame: event.frame,
                    start_time: event.backboard_time,
                    end_time: event.time,
                    confidence: None,
                    reason: format!(
                        "same-player touch {:.2}s after backboard bounce",
                        event.time - event.backboard_time
                    ),
                    event: event_json(event),
                }));
            }
            "speed_flip" => {
                let Some(calculator) = graph.state::<SpeedFlipCalculator>() else {
                    continue;
                };
                candidates.extend(calculator.events().iter().map(|event| {
                    let confidence = event.confidence;
                    MechanicCandidate {
                        mechanic: "speed_flip",
                        mechanic_label: "Speed Flip",
                        detector: "builtin:speed_flip",
                        player_id: Some(player_id_string(&event.player)),
                        is_team_0: Some(event.is_team_0),
                        event_time: event.time,
                        event_frame: event.frame,
                        start_time: (event.time - 0.5).max(0.0),
                        end_time: event.time,
                        confidence: Some(confidence),
                        reason: format!(
                            "{}% confidence; max speed {:.0}; diagonal {:.2}; cancel {:.2}",
                            confidence_pct(confidence),
                            event.max_speed,
                            event.diagonal_score,
                            event.cancel_score
                        ),
                        event: event_json(event),
                    }
                }));
            }
            "wavedash" => {
                let Some(calculator) = graph.state::<WavedashCalculator>() else {
                    continue;
                };
                candidates.extend(calculator.events().iter().map(|event| {
                    let confidence = event.confidence;
                    MechanicCandidate {
                        mechanic: "wavedash",
                        mechanic_label: "Wavedash",
                        detector: "builtin:wavedash",
                        player_id: Some(player_id_string(&event.player)),
                        is_team_0: Some(event.is_team_0),
                        event_time: event.time,
                        event_frame: event.frame,
                        start_time: event.dodge_time,
                        end_time: event.time,
                        confidence: Some(confidence),
                        reason: format!(
                            "{}% confidence; landing {:.2}s after dodge; speed gain {:.0}",
                            confidence_pct(confidence),
                            event.time_since_dodge,
                            event.horizontal_speed_gain
                        ),
                        event: event_json(event),
                    }
                }));
            }
            _ => {}
        }
    }

    candidates.retain(|candidate| include_candidate(candidate, config));
    candidates.sort_by(|left, right| {
        left.start_time
            .total_cmp(&right.start_time)
            .then_with(|| left.mechanic.cmp(right.mechanic))
            .then_with(|| left.event_frame.cmp(&right.event_frame))
    });
    Ok(candidates)
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

        items.push(PlaylistManifestItem {
            id: id.clone(),
            replay: source.source_id.clone(),
            start: PlaybackBound {
                kind: PlaybackBoundKind::Time,
                value: start_time,
            },
            end: PlaybackBound {
                kind: PlaybackBoundKind::Time,
                value: end_time,
            },
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
