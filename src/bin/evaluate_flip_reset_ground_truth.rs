use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use subtr_actor::ballchasing::parse_replay_bytes;
use subtr_actor::constants::DODGES_REFRESHED_COUNTER_KEY;
use subtr_actor::{
    DodgeRefreshedEvent, FlipResetEvent, FlipResetTuningManifest, FlipResetTuningReplay,
    ReplayDataCollector,
};

#[derive(Debug, Clone)]
struct Config {
    count: usize,
    scan_limit: usize,
    manifest: Option<PathBuf>,
    playlist: String,
    min_rank: String,
    cache_dir: PathBuf,
    match_window_seconds: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            count: 50,
            scan_limit: 250,
            manifest: None,
            playlist: "ranked-standard".to_owned(),
            min_rank: "supersonic-legend".to_owned(),
            cache_dir: PathBuf::from("target/flip-reset-ground-truth"),
            match_window_seconds: 0.20,
        }
    }
}

impl Config {
    fn from_args() -> Result<Self> {
        let mut config = Self::default();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--count" => {
                    config.count = args
                        .next()
                        .context("Expected a value after --count")?
                        .parse()
                        .context("Failed to parse --count as an integer")?;
                }
                "--playlist" => {
                    config.playlist = args.next().context("Expected a value after --playlist")?;
                }
                "--min-rank" => {
                    config.min_rank = args.next().context("Expected a value after --min-rank")?;
                }
                "--scan-limit" => {
                    config.scan_limit = args
                        .next()
                        .context("Expected a value after --scan-limit")?
                        .parse()
                        .context("Failed to parse --scan-limit as an integer")?;
                }
                "--manifest" => {
                    config.manifest = Some(PathBuf::from(
                        args.next().context("Expected a value after --manifest")?,
                    ));
                }
                "--cache-dir" => {
                    config.cache_dir =
                        PathBuf::from(args.next().context("Expected a value after --cache-dir")?);
                }
                "--match-window" => {
                    config.match_window_seconds = args
                        .next()
                        .context("Expected a value after --match-window")?
                        .parse()
                        .context("Failed to parse --match-window as a float")?;
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => anyhow::bail!("Unrecognized argument: {other}"),
            }
        }
        Ok(config)
    }
}

#[derive(Debug, Clone)]
struct ReplaySummary {
    id: String,
    title: Option<String>,
    uploader: Option<String>,
}

#[derive(Debug, Clone)]
struct ReplayEvaluation {
    replay_id: String,
    title: Option<String>,
    supports_exact_counter: bool,
    exact_count: usize,
    heuristic_count: usize,
    player_matches: usize,
    team_matches: usize,
    time_only_matches: usize,
    unmatched_exacts: Vec<ExactEventSummary>,
    unmatched_heuristics: Vec<HeuristicEventSummary>,
    player_matched_deltas: Vec<f32>,
}

#[derive(Debug, Clone)]
struct ExactEventSummary {
    time: f32,
    player: String,
    counter_value: i32,
}

#[derive(Debug, Clone)]
struct HeuristicEventSummary {
    time: f32,
    player: String,
    confidence: f32,
}

#[derive(Debug, Default)]
struct AggregateMetrics {
    scanned_replay_count: usize,
    supported_replay_count: usize,
    replay_with_exact_refresh_count: usize,
    total_exact: usize,
    total_heuristic: usize,
    total_player_matches: usize,
    total_team_matches: usize,
    total_time_only_matches: usize,
    player_matched_deltas: Vec<f32>,
    evaluations: Vec<ReplayEvaluation>,
}

impl AggregateMetrics {
    fn add(&mut self, evaluation: ReplayEvaluation) {
        self.scanned_replay_count += 1;
        if !evaluation.supports_exact_counter {
            return;
        }
        self.supported_replay_count += 1;
        if evaluation.exact_count > 0 {
            self.replay_with_exact_refresh_count += 1;
        }
        self.total_exact += evaluation.exact_count;
        self.total_heuristic += evaluation.heuristic_count;
        self.total_player_matches += evaluation.player_matches;
        self.total_team_matches += evaluation.team_matches;
        self.total_time_only_matches += evaluation.time_only_matches;
        self.player_matched_deltas
            .extend(evaluation.player_matched_deltas.iter().copied());
        self.evaluations.push(evaluation);
    }

    fn player_precision(&self) -> f32 {
        if self.total_heuristic == 0 {
            return 0.0;
        }
        self.total_player_matches as f32 / self.total_heuristic as f32
    }

    fn player_recall(&self) -> f32 {
        if self.total_exact == 0 {
            return 0.0;
        }
        self.total_player_matches as f32 / self.total_exact as f32
    }

    fn team_precision(&self) -> f32 {
        if self.total_heuristic == 0 {
            return 0.0;
        }
        self.total_team_matches as f32 / self.total_heuristic as f32
    }

    fn team_recall(&self) -> f32 {
        if self.total_exact == 0 {
            return 0.0;
        }
        self.total_team_matches as f32 / self.total_exact as f32
    }

    fn time_only_precision(&self) -> f32 {
        if self.total_heuristic == 0 {
            return 0.0;
        }
        self.total_time_only_matches as f32 / self.total_heuristic as f32
    }

    fn time_only_recall(&self) -> f32 {
        if self.total_exact == 0 {
            return 0.0;
        }
        self.total_time_only_matches as f32 / self.total_exact as f32
    }

    fn average_delta(&self) -> Option<f32> {
        if self.player_matched_deltas.is_empty() {
            return None;
        }
        Some(
            self.player_matched_deltas.iter().sum::<f32>()
                / self.player_matched_deltas.len() as f32,
        )
    }
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run --bin evaluate_flip_reset_ground_truth -- [--manifest PATH] [--count N] [--scan-limit N] [--playlist PLAYLIST] [--min-rank RANK] [--cache-dir PATH] [--match-window SECONDS]"
    );
}

fn extract_replay_summaries(response: &Value) -> Vec<ReplaySummary> {
    response
        .get("list")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let id = item.get("id")?.as_str()?.to_owned();
            let title = item
                .get("title")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let uploader = item
                .get("uploader")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            Some(ReplaySummary {
                id,
                title,
                uploader,
            })
        })
        .collect()
}

fn search_recent_replays(
    client: &Client,
    api_key: &str,
    config: &Config,
) -> Result<Vec<ReplaySummary>> {
    let mut results = Vec::new();
    let mut next_url = Some(format!(
        "https://ballchasing.com/api/replays?playlist={}&min-rank={}&sort-by=replay-date&sort-dir=desc&count={}",
        config.playlist, config.min_rank, config.scan_limit.min(200)
    ));

    while results.len() < config.scan_limit {
        let Some(url) = next_url.take() else {
            break;
        };
        let response: Value = client
            .get(&url)
            .header("Authorization", api_key)
            .send()
            .with_context(|| format!("Failed to fetch {url}"))?
            .error_for_status()
            .with_context(|| format!("Ballchasing returned an error for {url}"))?
            .json()
            .with_context(|| format!("Failed to decode JSON from {url}"))?;
        results.extend(extract_replay_summaries(&response));
        next_url = response
            .get("next")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        if next_url.is_none() {
            break;
        }
    }

    results.truncate(config.scan_limit);
    Ok(results)
}

fn load_manifest(path: &Path) -> Result<FlipResetTuningManifest> {
    let manifest_bytes =
        fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    serde_json::from_slice(&manifest_bytes)
        .with_context(|| format!("Failed to parse {}", path.display()))
}

fn cached_replay_path(cache_dir: &Path, replay_id: &str) -> PathBuf {
    cache_dir.join(format!("{replay_id}.replay"))
}

fn fetch_replay_bytes(
    client: &Client,
    api_key: &str,
    cache_dir: &Path,
    replay_id: &str,
) -> Result<Vec<u8>> {
    fs::create_dir_all(cache_dir)
        .with_context(|| format!("Failed to create cache directory {}", cache_dir.display()))?;
    let replay_path = cached_replay_path(cache_dir, replay_id);
    if replay_path.exists() {
        return fs::read(&replay_path)
            .with_context(|| format!("Failed to read cached replay {}", replay_path.display()));
    }

    let url = format!("https://ballchasing.com/api/replays/{replay_id}/file");
    let replay_bytes = client
        .get(&url)
        .header("Authorization", api_key)
        .send()
        .with_context(|| format!("Failed to fetch {url}"))?
        .error_for_status()
        .with_context(|| format!("Ballchasing returned an error for {url}"))?
        .bytes()
        .with_context(|| format!("Failed to read replay bytes from {url}"))?;
    fs::write(&replay_path, replay_bytes.as_ref())
        .with_context(|| format!("Failed to write cached replay {}", replay_path.display()))?;
    Ok(replay_bytes.to_vec())
}

fn player_names_by_id(
    replay_data: &subtr_actor::ReplayData,
) -> HashMap<subtr_actor::PlayerId, String> {
    replay_data
        .meta
        .player_order()
        .map(|player| (player.remote_id.clone(), player.name.clone()))
        .collect()
}

fn greedy_match_exact_events(
    exact_events: &[DodgeRefreshedEvent],
    heuristic_events: &[FlipResetEvent],
    match_window_seconds: f32,
    events_can_match: impl Fn(&DodgeRefreshedEvent, &FlipResetEvent) -> bool,
) -> (usize, Vec<f32>, Vec<usize>, Vec<usize>) {
    let mut matched_exact = vec![false; exact_events.len()];
    let mut matched_heuristic = vec![false; heuristic_events.len()];
    let mut matched_deltas = Vec::new();
    let mut matches = 0usize;

    for (exact_index, exact_event) in exact_events.iter().enumerate() {
        let mut best_candidate = None;
        let mut best_delta = f32::INFINITY;

        for (heuristic_index, heuristic_event) in heuristic_events.iter().enumerate() {
            if matched_heuristic[heuristic_index] || !events_can_match(exact_event, heuristic_event)
            {
                continue;
            }
            let delta = (heuristic_event.time - exact_event.time).abs();
            if delta > match_window_seconds || delta >= best_delta {
                continue;
            }
            best_delta = delta;
            best_candidate = Some(heuristic_index);
        }

        let Some(heuristic_index) = best_candidate else {
            continue;
        };

        matched_exact[exact_index] = true;
        matched_heuristic[heuristic_index] = true;
        matched_deltas.push(best_delta);
        matches += 1;
    }

    let unmatched_exacts = matched_exact
        .iter()
        .enumerate()
        .filter_map(|(index, matched)| (!matched).then_some(index))
        .collect();
    let unmatched_heuristics = matched_heuristic
        .iter()
        .enumerate()
        .filter_map(|(index, matched)| (!matched).then_some(index))
        .collect();

    (
        matches,
        matched_deltas,
        unmatched_exacts,
        unmatched_heuristics,
    )
}

fn replay_supports_exact_dodge_refreshes(replay: &boxcars::Replay) -> bool {
    replay
        .objects
        .iter()
        .any(|name| name == DODGES_REFRESHED_COUNTER_KEY)
}

fn evaluate_replay(
    summary: &ReplaySummary,
    replay_bytes: &[u8],
    match_window_seconds: f32,
) -> Result<ReplayEvaluation> {
    let replay = parse_replay_bytes(replay_bytes)?;
    let supports_exact_counter = replay_supports_exact_dodge_refreshes(&replay);
    if !supports_exact_counter {
        return Ok(ReplayEvaluation {
            replay_id: summary.id.clone(),
            title: summary.title.clone(),
            supports_exact_counter,
            exact_count: 0,
            heuristic_count: 0,
            player_matches: 0,
            team_matches: 0,
            time_only_matches: 0,
            unmatched_exacts: Vec::new(),
            unmatched_heuristics: Vec::new(),
            player_matched_deltas: Vec::new(),
        });
    }
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .map_err(|error| anyhow::Error::new(error.variant))
        .context("Failed to compute replay data")?;
    let player_names = player_names_by_id(&replay_data);
    let exact_events = &replay_data.dodge_refreshed_events;
    let heuristic_events = &replay_data.flip_reset_events;
    let (
        player_matches,
        player_matched_deltas,
        unmatched_exact_indices,
        unmatched_heuristic_indices,
    ) = greedy_match_exact_events(
        exact_events,
        heuristic_events,
        match_window_seconds,
        |exact, heuristic| heuristic.player == exact.player,
    );
    let (team_matches, _, _, _) = greedy_match_exact_events(
        exact_events,
        heuristic_events,
        match_window_seconds,
        |exact, heuristic| heuristic.is_team_0 == exact.is_team_0,
    );
    let (time_only_matches, _, _, _) = greedy_match_exact_events(
        exact_events,
        heuristic_events,
        match_window_seconds,
        |_exact, _heuristic| true,
    );

    let unmatched_exacts = unmatched_exact_indices
        .into_iter()
        .map(|index| {
            let event = &exact_events[index];
            ExactEventSummary {
                time: event.time,
                player: player_names
                    .get(&event.player)
                    .cloned()
                    .unwrap_or_else(|| format!("{:?}", event.player)),
                counter_value: event.counter_value,
            }
        })
        .collect();
    let unmatched_heuristics = unmatched_heuristic_indices
        .into_iter()
        .map(|index| {
            let event = &heuristic_events[index];
            HeuristicEventSummary {
                time: event.time,
                player: player_names
                    .get(&event.player)
                    .cloned()
                    .unwrap_or_else(|| format!("{:?}", event.player)),
                confidence: event.confidence,
            }
        })
        .collect();

    Ok(ReplayEvaluation {
        replay_id: summary.id.clone(),
        title: summary.title.clone(),
        supports_exact_counter,
        exact_count: exact_events.len(),
        heuristic_count: heuristic_events.len(),
        player_matches,
        team_matches,
        time_only_matches,
        unmatched_exacts,
        unmatched_heuristics,
        player_matched_deltas,
    })
}

fn evaluate_manifest_replay(
    manifest_path: &Path,
    replay: &FlipResetTuningReplay,
    match_window_seconds: f32,
) -> Result<ReplayEvaluation> {
    let replay_path = replay.replay_path_from_manifest(manifest_path);
    let replay_bytes = fs::read(&replay_path)
        .with_context(|| format!("Failed to read {}", replay_path.display()))?;
    let summary = ReplaySummary {
        id: replay.replay_id.clone(),
        title: replay.title.clone(),
        uploader: replay.uploader.clone(),
    };
    evaluate_replay(&summary, &replay_bytes, match_window_seconds)
}

fn print_summary(config: &Config, metrics: &AggregateMetrics, replays: &[ReplaySummary]) {
    if let Some(manifest_path) = &config.manifest {
        println!(
            "Evaluated local tuning set from {}",
            manifest_path.display()
        );
    } else {
        println!(
            "Scanned {} recent public {} replays at rank {}",
            replays.len(),
            config.playlist,
            config.min_rank
        );
    }
    println!(
        "Supported replays with the exact counter: {} / {} scanned",
        metrics.supported_replay_count, metrics.scanned_replay_count
    );
    println!(
        "Supported replays with exact dodge refreshes: {} / {}",
        metrics.replay_with_exact_refresh_count, metrics.supported_replay_count
    );
    println!(
        "Exact refresh events: {}, strict heuristic events: {}, matched within {:.2}s: {}",
        metrics.total_exact,
        metrics.total_heuristic,
        config.match_window_seconds,
        metrics.total_player_matches
    );
    println!(
        "Player precision/recall: {:.3} / {:.3}, team precision/recall: {:.3} / {:.3}, time-only precision/recall: {:.3} / {:.3}, avg |player delta|: {}",
        metrics.player_precision(),
        metrics.player_recall(),
        metrics.team_precision(),
        metrics.team_recall(),
        metrics.time_only_precision(),
        metrics.time_only_recall(),
        metrics
            .average_delta()
            .map(|value| format!("{value:.3}s"))
            .unwrap_or_else(|| "n/a".to_owned())
    );

    let mut worst_false_negatives: Vec<_> = metrics
        .evaluations
        .iter()
        .filter(|evaluation| !evaluation.unmatched_exacts.is_empty())
        .collect();
    worst_false_negatives.sort_by_key(|evaluation| usize::MAX - evaluation.unmatched_exacts.len());
    println!("Worst false-negative replays:");
    for evaluation in worst_false_negatives.into_iter().take(10) {
        println!(
            "  {} exact={} heuristic={} player-matched={} title={}",
            evaluation.replay_id,
            evaluation.exact_count,
            evaluation.heuristic_count,
            evaluation.player_matches,
            evaluation.title.as_deref().unwrap_or("<untitled>")
        );
        for event in evaluation.unmatched_exacts.iter().take(3) {
            println!(
                "    missed exact t={:.2} player={} counter={}",
                event.time, event.player, event.counter_value
            );
        }
    }

    let mut worst_false_positives: Vec<_> = metrics
        .evaluations
        .iter()
        .filter(|evaluation| !evaluation.unmatched_heuristics.is_empty())
        .collect();
    worst_false_positives
        .sort_by_key(|evaluation| usize::MAX - evaluation.unmatched_heuristics.len());
    println!("Worst false-positive replays:");
    for evaluation in worst_false_positives.into_iter().take(10) {
        println!(
            "  {} exact={} heuristic={} player-matched={} title={}",
            evaluation.replay_id,
            evaluation.exact_count,
            evaluation.heuristic_count,
            evaluation.player_matches,
            evaluation.title.as_deref().unwrap_or("<untitled>")
        );
        for event in evaluation.unmatched_heuristics.iter().take(3) {
            println!(
                "    extra heuristic t={:.2} player={} confidence={:.3}",
                event.time, event.player, event.confidence
            );
        }
    }
}

fn main() -> Result<()> {
    let config = Config::from_args()?;

    let mut metrics = AggregateMetrics::default();
    let mut replay_summaries = Vec::new();
    if let Some(manifest_path) = &config.manifest {
        let manifest = load_manifest(manifest_path)?;
        for (index, replay) in manifest.replays.iter().enumerate() {
            eprintln!(
                "[{}/{}] evaluating local replay {}",
                index + 1,
                manifest.replays.len(),
                replay.replay_id
            );
            metrics.add(
                evaluate_manifest_replay(manifest_path, replay, config.match_window_seconds)
                    .with_context(|| format!("Failed to evaluate replay {}", replay.replay_id))?,
            );
            replay_summaries.push(ReplaySummary {
                id: replay.replay_id.clone(),
                title: replay.title.clone(),
                uploader: replay.uploader.clone(),
            });
        }
    } else {
        let api_key =
            std::env::var("BALLCHASING_API_KEY").context("BALLCHASING_API_KEY must be set")?;
        let client = Client::builder()
            .build()
            .context("Failed to build HTTP client")?;
        replay_summaries = search_recent_replays(&client, &api_key, &config)?;
        if replay_summaries.is_empty() {
            anyhow::bail!("Ballchasing search returned no replays");
        }

        for (index, summary) in replay_summaries.iter().enumerate() {
            eprintln!(
                "[{}/{}] evaluating {} ({})",
                index + 1,
                replay_summaries.len(),
                summary.id,
                summary.uploader.as_deref().unwrap_or("unknown uploader")
            );
            let replay_bytes =
                fetch_replay_bytes(&client, &api_key, &config.cache_dir, &summary.id)
                    .with_context(|| format!("Failed to fetch replay {}", summary.id))?;
            let evaluation =
                evaluate_replay(summary, &replay_bytes, config.match_window_seconds)
                    .with_context(|| format!("Failed to evaluate replay {}", summary.id))?;
            metrics.add(evaluation);
            if metrics.supported_replay_count >= config.count {
                break;
            }
        }
    }

    print_summary(&config, &metrics, &replay_summaries);
    Ok(())
}
