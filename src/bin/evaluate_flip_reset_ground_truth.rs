use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use subtr_actor::ballchasing::parse_replay_bytes;
use subtr_actor::constants::DODGES_REFRESHED_COUNTER_KEY;
use subtr_actor::{
    DodgeRefreshedEvent, FlipResetEvent, FlipResetFollowupDodgeEvent, FlipResetTuningManifest,
    FlipResetTuningReplay, PlayerId, PostWallDodgeEvent, ReplayData, ReplayDataCollector,
};

#[derive(Debug, Clone, Copy)]
struct MatchWindow {
    before_seconds: f32,
    after_seconds: f32,
}

impl MatchWindow {
    fn from_symmetric(seconds: f32) -> Self {
        Self {
            before_seconds: seconds,
            after_seconds: seconds,
        }
    }

    fn contains(self, signed_delta_seconds: f32) -> bool {
        signed_delta_seconds >= -self.before_seconds && signed_delta_seconds <= self.after_seconds
    }

    fn normalized_cost(self, signed_delta_seconds: f32) -> f32 {
        if signed_delta_seconds < 0.0 {
            let denom = self.before_seconds.max(0.001);
            (-signed_delta_seconds / denom).clamp(0.0, 1.0)
        } else {
            let denom = self.after_seconds.max(0.001);
            (signed_delta_seconds / denom).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum HeuristicSource {
    Strict,
    Followup,
    Combined,
    PostWall,
    All,
}

impl HeuristicSource {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "strict" => Ok(Self::Strict),
            "followup" => Ok(Self::Followup),
            "combined" => Ok(Self::Combined),
            "post-wall" => Ok(Self::PostWall),
            "all" => Ok(Self::All),
            other => anyhow::bail!(
                "Unrecognized heuristic source: {other}. Expected one of: strict, followup, combined, post-wall, all"
            ),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Followup => "followup",
            Self::Combined => "combined",
            Self::PostWall => "post-wall",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone)]
struct Config {
    count: usize,
    scan_limit: usize,
    manifest: Option<PathBuf>,
    playlist: String,
    min_rank: String,
    cache_dir: PathBuf,
    match_window: MatchWindow,
    debounce_seconds: f32,
    heuristic_source: HeuristicSource,
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
            match_window: MatchWindow {
                before_seconds: 0.20,
                after_seconds: 0.05,
            },
            debounce_seconds: 0.10,
            heuristic_source: HeuristicSource::Strict,
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
                    config.match_window = MatchWindow::from_symmetric(
                        args.next()
                            .context("Expected a value after --match-window")?
                            .parse()
                            .context("Failed to parse --match-window as a float")?,
                    );
                }
                "--match-window-before" => {
                    config.match_window.before_seconds = args
                        .next()
                        .context("Expected a value after --match-window-before")?
                        .parse()
                        .context("Failed to parse --match-window-before as a float")?;
                }
                "--match-window-after" => {
                    config.match_window.after_seconds = args
                        .next()
                        .context("Expected a value after --match-window-after")?
                        .parse()
                        .context("Failed to parse --match-window-after as a float")?;
                }
                "--debounce" => {
                    config.debounce_seconds = args
                        .next()
                        .context("Expected a value after --debounce")?
                        .parse()
                        .context("Failed to parse --debounce as a float")?;
                }
                "--heuristic" => {
                    config.heuristic_source = HeuristicSource::parse(
                        &args.next().context("Expected a value after --heuristic")?,
                    )?;
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
struct CandidateEvent {
    time: f32,
    player: PlayerId,
    confidence: Option<f32>,
    source: &'static str,
}

#[derive(Debug, Clone)]
struct ReplayEvaluation {
    replay_id: String,
    title: Option<String>,
    supports_exact_counter: bool,
    exact_count: usize,
    raw_heuristic_count: usize,
    debounced_heuristic_count: usize,
    player_matches: usize,
    false_negatives: usize,
    false_positives: usize,
    unmatched_exacts: Vec<ExactEventSummary>,
    unmatched_heuristics: Vec<HeuristicEventSummary>,
    player_matched_signed_deltas: Vec<f32>,
}

impl ReplayEvaluation {
    fn timing_penalty(&self, match_window: MatchWindow) -> f32 {
        if self.player_matched_signed_deltas.is_empty() {
            return 0.0;
        }
        self.player_matched_signed_deltas
            .iter()
            .map(|delta| match_window.normalized_cost(*delta))
            .sum::<f32>()
            / self.player_matched_signed_deltas.len() as f32
    }

    fn temporal_loss(&self, match_window: MatchWindow) -> f32 {
        3.0 * self.false_negatives as f32
            + 1.0 * self.false_positives as f32
            + 0.25 * self.timing_penalty(match_window)
    }
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
    confidence: Option<f32>,
    source: &'static str,
}

#[derive(Debug, Default)]
struct AggregateMetrics {
    scanned_replay_count: usize,
    supported_replay_count: usize,
    replay_with_exact_refresh_count: usize,
    total_exact: usize,
    total_raw_heuristic: usize,
    total_debounced_heuristic: usize,
    total_player_matches: usize,
    total_false_negatives: usize,
    total_false_positives: usize,
    player_matched_signed_deltas: Vec<f32>,
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
        self.total_raw_heuristic += evaluation.raw_heuristic_count;
        self.total_debounced_heuristic += evaluation.debounced_heuristic_count;
        self.total_player_matches += evaluation.player_matches;
        self.total_false_negatives += evaluation.false_negatives;
        self.total_false_positives += evaluation.false_positives;
        self.player_matched_signed_deltas
            .extend(evaluation.player_matched_signed_deltas.iter().copied());
        self.evaluations.push(evaluation);
    }

    fn player_precision(&self) -> f32 {
        if self.total_debounced_heuristic == 0 {
            return 0.0;
        }
        self.total_player_matches as f32 / self.total_debounced_heuristic as f32
    }

    fn player_recall(&self) -> f32 {
        if self.total_exact == 0 {
            return 0.0;
        }
        self.total_player_matches as f32 / self.total_exact as f32
    }

    fn average_abs_delta(&self) -> Option<f32> {
        if self.player_matched_signed_deltas.is_empty() {
            return None;
        }
        Some(
            self.player_matched_signed_deltas
                .iter()
                .map(|delta| delta.abs())
                .sum::<f32>()
                / self.player_matched_signed_deltas.len() as f32,
        )
    }

    fn average_signed_delta(&self) -> Option<f32> {
        if self.player_matched_signed_deltas.is_empty() {
            return None;
        }
        Some(
            self.player_matched_signed_deltas.iter().sum::<f32>()
                / self.player_matched_signed_deltas.len() as f32,
        )
    }

    fn timing_penalty(&self, match_window: MatchWindow) -> f32 {
        if self.player_matched_signed_deltas.is_empty() {
            return 0.0;
        }
        self.player_matched_signed_deltas
            .iter()
            .map(|delta| match_window.normalized_cost(*delta))
            .sum::<f32>()
            / self.player_matched_signed_deltas.len() as f32
    }

    fn temporal_loss(&self, match_window: MatchWindow) -> f32 {
        3.0 * self.total_false_negatives as f32
            + 1.0 * self.total_false_positives as f32
            + 0.25 * self.timing_penalty(match_window)
    }
}

#[derive(Debug)]
struct MatchResult {
    matched_signed_deltas: Vec<f32>,
    unmatched_exact_indices: Vec<usize>,
    unmatched_heuristic_indices: Vec<usize>,
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run --bin evaluate_flip_reset_ground_truth -- [--manifest PATH] [--count N] [--scan-limit N] [--playlist PLAYLIST] [--min-rank RANK] [--cache-dir PATH] [--heuristic strict|followup|combined|post-wall|all] [--debounce SECONDS] [--match-window SECONDS] [--match-window-before SECONDS] [--match-window-after SECONDS]"
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

fn player_names_by_id(replay_data: &ReplayData) -> HashMap<PlayerId, String> {
    replay_data
        .meta
        .player_order()
        .map(|player| (player.remote_id.clone(), player.name.clone()))
        .collect()
}

fn candidate_priority(event: &CandidateEvent) -> (i32, i32) {
    let confidence_bucket = (event.confidence.unwrap_or(0.0) * 1000.0).round() as i32;
    let source_priority = match event.source {
        "strict" => 4,
        "followup" => 3,
        "post-wall" => 2,
        _ => 1,
    };
    (confidence_bucket, source_priority)
}

fn sort_by_time_ascending<T>(left: &T, right: &T, key: impl Fn(&T) -> f32) -> Ordering {
    key(left)
        .partial_cmp(&key(right))
        .unwrap_or(Ordering::Equal)
}

fn debounce_candidate_events(
    raw_events: &[CandidateEvent],
    debounce_seconds: f32,
) -> Vec<CandidateEvent> {
    let mut by_player: HashMap<PlayerId, Vec<CandidateEvent>> = HashMap::new();
    for event in raw_events {
        by_player
            .entry(event.player.clone())
            .or_default()
            .push(event.clone());
    }

    let mut debounced = Vec::new();
    for mut player_events in by_player.into_values() {
        player_events
            .sort_by(|left, right| sort_by_time_ascending(left, right, |event| event.time));
        let mut player_events = player_events.into_iter();
        let Some(first_event) = player_events.next() else {
            continue;
        };
        let mut best_event = first_event.clone();
        let mut last_time = first_event.time;
        for event in player_events {
            if event.time - last_time <= debounce_seconds {
                if candidate_priority(&event) > candidate_priority(&best_event) {
                    best_event = event.clone();
                }
                last_time = event.time;
                continue;
            }
            debounced.push(best_event);
            best_event = event.clone();
            last_time = event.time;
        }
        debounced.push(best_event);
    }

    debounced.sort_by(|left, right| sort_by_time_ascending(left, right, |event| event.time));
    debounced
}

fn greedy_match_exact_events(
    exact_events: &[DodgeRefreshedEvent],
    heuristic_events: &[CandidateEvent],
    match_window: MatchWindow,
) -> MatchResult {
    let mut matched_exact = vec![false; exact_events.len()];
    let mut matched_heuristic = vec![false; heuristic_events.len()];
    let mut matched_signed_deltas = Vec::new();

    for (exact_index, exact_event) in exact_events.iter().enumerate() {
        let mut best_candidate = None;
        let mut best_abs_delta = f32::INFINITY;
        let mut best_priority = (i32::MIN, i32::MIN);

        for (heuristic_index, heuristic_event) in heuristic_events.iter().enumerate() {
            if matched_heuristic[heuristic_index] || heuristic_event.player != exact_event.player {
                continue;
            }
            let signed_delta = heuristic_event.time - exact_event.time;
            if !match_window.contains(signed_delta) {
                continue;
            }
            let abs_delta = signed_delta.abs();
            let priority = candidate_priority(heuristic_event);
            if abs_delta < best_abs_delta
                || (abs_delta == best_abs_delta && priority > best_priority)
            {
                best_candidate = Some((heuristic_index, signed_delta));
                best_abs_delta = abs_delta;
                best_priority = priority;
            }
        }

        let Some((heuristic_index, signed_delta)) = best_candidate else {
            continue;
        };

        matched_exact[exact_index] = true;
        matched_heuristic[heuristic_index] = true;
        matched_signed_deltas.push(signed_delta);
    }

    MatchResult {
        matched_signed_deltas,
        unmatched_exact_indices: matched_exact
            .iter()
            .enumerate()
            .filter_map(|(index, matched)| (!matched).then_some(index))
            .collect(),
        unmatched_heuristic_indices: matched_heuristic
            .iter()
            .enumerate()
            .filter_map(|(index, matched)| (!matched).then_some(index))
            .collect(),
    }
}

fn replay_supports_exact_dodge_refreshes(replay: &boxcars::Replay) -> bool {
    replay
        .objects
        .iter()
        .any(|name| name == DODGES_REFRESHED_COUNTER_KEY)
}

fn candidate_from_strict(event: &FlipResetEvent) -> CandidateEvent {
    CandidateEvent {
        time: event.time,
        player: event.player.clone(),
        confidence: Some(event.confidence),
        source: "strict",
    }
}

fn candidate_from_followup(event: &FlipResetFollowupDodgeEvent) -> CandidateEvent {
    CandidateEvent {
        time: event.time,
        player: event.player.clone(),
        confidence: Some(event.candidate_touch_confidence),
        source: "followup",
    }
}

fn candidate_from_post_wall(event: &PostWallDodgeEvent) -> CandidateEvent {
    CandidateEvent {
        time: event.time,
        player: event.player.clone(),
        confidence: None,
        source: "post-wall",
    }
}

fn collect_candidate_events(
    replay_data: &ReplayData,
    heuristic_source: HeuristicSource,
) -> Vec<CandidateEvent> {
    let mut events = Vec::new();
    match heuristic_source {
        HeuristicSource::Strict => {
            events.extend(
                replay_data
                    .flip_reset_events
                    .iter()
                    .map(candidate_from_strict),
            );
        }
        HeuristicSource::Followup => {
            events.extend(
                replay_data
                    .flip_reset_followup_dodge_events
                    .iter()
                    .map(candidate_from_followup),
            );
        }
        HeuristicSource::Combined => {
            events.extend(
                replay_data
                    .flip_reset_events
                    .iter()
                    .map(candidate_from_strict),
            );
            events.extend(
                replay_data
                    .flip_reset_followup_dodge_events
                    .iter()
                    .map(candidate_from_followup),
            );
        }
        HeuristicSource::PostWall => {
            events.extend(
                replay_data
                    .post_wall_dodge_events
                    .iter()
                    .map(candidate_from_post_wall),
            );
        }
        HeuristicSource::All => {
            events.extend(
                replay_data
                    .flip_reset_events
                    .iter()
                    .map(candidate_from_strict),
            );
            events.extend(
                replay_data
                    .flip_reset_followup_dodge_events
                    .iter()
                    .map(candidate_from_followup),
            );
            events.extend(
                replay_data
                    .post_wall_dodge_events
                    .iter()
                    .map(candidate_from_post_wall),
            );
        }
    }

    events.sort_by(|left, right| sort_by_time_ascending(left, right, |event| event.time));
    events
}

fn evaluate_replay(
    summary: &ReplaySummary,
    replay_bytes: &[u8],
    heuristic_source: HeuristicSource,
    match_window: MatchWindow,
    debounce_seconds: f32,
) -> Result<ReplayEvaluation> {
    let replay = parse_replay_bytes(replay_bytes)?;
    let supports_exact_counter = replay_supports_exact_dodge_refreshes(&replay);
    if !supports_exact_counter {
        return Ok(ReplayEvaluation {
            replay_id: summary.id.clone(),
            title: summary.title.clone(),
            supports_exact_counter,
            exact_count: 0,
            raw_heuristic_count: 0,
            debounced_heuristic_count: 0,
            player_matches: 0,
            false_negatives: 0,
            false_positives: 0,
            unmatched_exacts: Vec::new(),
            unmatched_heuristics: Vec::new(),
            player_matched_signed_deltas: Vec::new(),
        });
    }

    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .map_err(|error| anyhow::Error::new(error.variant))
        .context("Failed to compute replay data")?;
    let player_names = player_names_by_id(&replay_data);
    let exact_events = &replay_data.dodge_refreshed_events;
    let raw_heuristic_events = collect_candidate_events(&replay_data, heuristic_source);
    let heuristic_events = debounce_candidate_events(&raw_heuristic_events, debounce_seconds);
    let match_result = greedy_match_exact_events(exact_events, &heuristic_events, match_window);

    let unmatched_exacts = match_result
        .unmatched_exact_indices
        .iter()
        .map(|index| {
            let event = &exact_events[*index];
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
    let unmatched_heuristics = match_result
        .unmatched_heuristic_indices
        .iter()
        .map(|index| {
            let event = &heuristic_events[*index];
            HeuristicEventSummary {
                time: event.time,
                player: player_names
                    .get(&event.player)
                    .cloned()
                    .unwrap_or_else(|| format!("{:?}", event.player)),
                confidence: event.confidence,
                source: event.source,
            }
        })
        .collect();

    Ok(ReplayEvaluation {
        replay_id: summary.id.clone(),
        title: summary.title.clone(),
        supports_exact_counter,
        exact_count: exact_events.len(),
        raw_heuristic_count: raw_heuristic_events.len(),
        debounced_heuristic_count: heuristic_events.len(),
        player_matches: match_result.matched_signed_deltas.len(),
        false_negatives: match_result.unmatched_exact_indices.len(),
        false_positives: match_result.unmatched_heuristic_indices.len(),
        unmatched_exacts,
        unmatched_heuristics,
        player_matched_signed_deltas: match_result.matched_signed_deltas,
    })
}

fn evaluate_manifest_replay(
    manifest_path: &Path,
    replay: &FlipResetTuningReplay,
    heuristic_source: HeuristicSource,
    match_window: MatchWindow,
    debounce_seconds: f32,
) -> Result<ReplayEvaluation> {
    let replay_path = replay.replay_path_from_manifest(manifest_path);
    let replay_bytes = fs::read(&replay_path)
        .with_context(|| format!("Failed to read {}", replay_path.display()))?;
    let summary = ReplaySummary {
        id: replay.replay_id.clone(),
        title: replay.title.clone(),
        uploader: replay.uploader.clone(),
    };
    evaluate_replay(
        &summary,
        &replay_bytes,
        heuristic_source,
        match_window,
        debounce_seconds,
    )
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
        "Heuristic source: {}, raw events: {}, debounced events: {}, exact refresh events: {}",
        config.heuristic_source.as_str(),
        metrics.total_raw_heuristic,
        metrics.total_debounced_heuristic,
        metrics.total_exact
    );
    println!(
        "Temporal match window: [-{:.2}s, +{:.2}s], debounce: {:.2}s",
        config.match_window.before_seconds,
        config.match_window.after_seconds,
        config.debounce_seconds
    );
    println!(
        "Player matches: {}, FN: {}, FP: {}, precision: {:.3}, recall: {:.3}",
        metrics.total_player_matches,
        metrics.total_false_negatives,
        metrics.total_false_positives,
        metrics.player_precision(),
        metrics.player_recall()
    );
    println!(
        "Avg signed delta: {}, avg |delta|: {}, normalized timing penalty: {:.3}, temporal loss: {:.3}",
        metrics
            .average_signed_delta()
            .map(|value| format!("{value:.3}s"))
            .unwrap_or_else(|| "n/a".to_owned()),
        metrics
            .average_abs_delta()
            .map(|value| format!("{value:.3}s"))
            .unwrap_or_else(|| "n/a".to_owned()),
        metrics.timing_penalty(config.match_window),
        metrics.temporal_loss(config.match_window)
    );
    println!("Loss formula: 3 * FN + 1 * FP + 0.25 * mean(normalized timing error)");

    let mut worst_losses: Vec<_> = metrics.evaluations.iter().collect();
    worst_losses.sort_by(|left, right| {
        right
            .temporal_loss(config.match_window)
            .partial_cmp(&left.temporal_loss(config.match_window))
            .unwrap_or(Ordering::Equal)
    });
    println!("Worst-loss replays:");
    for evaluation in worst_losses.into_iter().take(10) {
        println!(
            "  {} exact={} raw={} debounced={} matched={} fn={} fp={} loss={:.3} title={}",
            evaluation.replay_id,
            evaluation.exact_count,
            evaluation.raw_heuristic_count,
            evaluation.debounced_heuristic_count,
            evaluation.player_matches,
            evaluation.false_negatives,
            evaluation.false_positives,
            evaluation.temporal_loss(config.match_window),
            evaluation.title.as_deref().unwrap_or("<untitled>")
        );
        for event in evaluation.unmatched_exacts.iter().take(2) {
            println!(
                "    missed exact t={:.2} player={} counter={}",
                event.time, event.player, event.counter_value
            );
        }
        for event in evaluation.unmatched_heuristics.iter().take(2) {
            let confidence = event
                .confidence
                .map(|value| format!("{value:.3}"))
                .unwrap_or_else(|| "n/a".to_owned());
            println!(
                "    extra {} t={:.2} player={} confidence={}",
                event.source, event.time, event.player, confidence
            );
        }
    }

    let mut best_recall: Vec<_> = metrics.evaluations.iter().collect();
    best_recall.sort_by(|left, right| {
        let left_recall = if left.exact_count == 0 {
            0.0
        } else {
            left.player_matches as f32 / left.exact_count as f32
        };
        let right_recall = if right.exact_count == 0 {
            0.0
        } else {
            right.player_matches as f32 / right.exact_count as f32
        };
        right_recall
            .partial_cmp(&left_recall)
            .unwrap_or(Ordering::Equal)
    });
    println!("Best-recall replays:");
    for evaluation in best_recall.into_iter().take(5) {
        let recall = if evaluation.exact_count == 0 {
            0.0
        } else {
            evaluation.player_matches as f32 / evaluation.exact_count as f32
        };
        println!(
            "  {} matched={} exact={} debounced={} recall={:.3} title={}",
            evaluation.replay_id,
            evaluation.player_matches,
            evaluation.exact_count,
            evaluation.debounced_heuristic_count,
            recall,
            evaluation.title.as_deref().unwrap_or("<untitled>")
        );
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
                evaluate_manifest_replay(
                    manifest_path,
                    replay,
                    config.heuristic_source,
                    config.match_window,
                    config.debounce_seconds,
                )
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
            let evaluation = evaluate_replay(
                summary,
                &replay_bytes,
                config.heuristic_source,
                config.match_window,
                config.debounce_seconds,
            )
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
