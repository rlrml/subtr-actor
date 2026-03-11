use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use subtr_actor::ballchasing::parse_replay_bytes;
use subtr_actor::constants::DODGES_REFRESHED_COUNTER_KEY;
use subtr_actor::{FlipResetTuningManifest, FlipResetTuningReplay, ReplayDataCollector};

#[derive(Debug, Clone)]
struct Config {
    count: usize,
    seed_scan_limit: usize,
    per_player_limit: usize,
    recent_page_limit: usize,
    player_page_limit: usize,
    seed_players: Vec<String>,
    playlist: String,
    min_rank: String,
    output_dir: PathBuf,
    sleep_millis: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            count: 30,
            seed_scan_limit: 80,
            per_player_limit: 20,
            recent_page_limit: 3,
            player_page_limit: 2,
            seed_players: Vec::new(),
            playlist: "ranked-doubles".to_owned(),
            min_rank: "supersonic-legend".to_owned(),
            output_dir: PathBuf::from("target/flip-reset-positive-tuning-set"),
            sleep_millis: 250,
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
                "--seed-scan-limit" => {
                    config.seed_scan_limit = args
                        .next()
                        .context("Expected a value after --seed-scan-limit")?
                        .parse()
                        .context("Failed to parse --seed-scan-limit as an integer")?;
                }
                "--per-player-limit" => {
                    config.per_player_limit = args
                        .next()
                        .context("Expected a value after --per-player-limit")?
                        .parse()
                        .context("Failed to parse --per-player-limit as an integer")?;
                }
                "--recent-page-limit" => {
                    config.recent_page_limit = args
                        .next()
                        .context("Expected a value after --recent-page-limit")?
                        .parse()
                        .context("Failed to parse --recent-page-limit as an integer")?;
                }
                "--player-page-limit" => {
                    config.player_page_limit = args
                        .next()
                        .context("Expected a value after --player-page-limit")?
                        .parse()
                        .context("Failed to parse --player-page-limit as an integer")?;
                }
                "--seed-player" => {
                    config.seed_players.push(
                        args.next()
                            .context("Expected a value after --seed-player")?,
                    );
                }
                "--playlist" => {
                    config.playlist = args.next().context("Expected a value after --playlist")?;
                }
                "--min-rank" => {
                    config.min_rank = args.next().context("Expected a value after --min-rank")?;
                }
                "--output-dir" => {
                    config.output_dir =
                        PathBuf::from(args.next().context("Expected a value after --output-dir")?);
                }
                "--sleep-millis" => {
                    config.sleep_millis = args
                        .next()
                        .context("Expected a value after --sleep-millis")?
                        .parse()
                        .context("Failed to parse --sleep-millis as an integer")?;
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
    date: Option<String>,
    title: Option<String>,
    uploader: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SearchQuery {
    Recent,
    PlayerName(String),
}

impl SearchQuery {
    fn label(&self) -> String {
        match self {
            Self::Recent => "recent".to_owned(),
            Self::PlayerName(player_name) => format!("player:{player_name}"),
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run --bin build_flip_reset_tuning_set -- [--count N] [--seed-scan-limit N] [--per-player-limit N] [--recent-page-limit N] [--player-page-limit N] [--seed-player NAME]... [--playlist PLAYLIST] [--min-rank RANK] [--output-dir PATH] [--sleep-millis N]"
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
            let date = item
                .get("date")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
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
                date,
                title,
                uploader,
            })
        })
        .collect()
}

fn search_replays(client: &Client, api_key: &str, config: &Config, query: &SearchQuery) -> Result<Vec<ReplaySummary>> {
    let count = match query {
        SearchQuery::Recent => config.seed_scan_limit.min(200),
        SearchQuery::PlayerName(_) => config.per_player_limit.min(200),
    };
    let max_pages = match query {
        SearchQuery::Recent => config.recent_page_limit,
        SearchQuery::PlayerName(_) => config.player_page_limit,
    };
    let mut next_url = Some("https://ballchasing.com/api/replays".to_owned());
    let mut is_first_page = true;
    let mut results = Vec::new();
    let mut page_index = 0usize;

    while let Some(url) = next_url.take() {
        if page_index >= max_pages {
            break;
        }
        let mut request = client.get(&url).header("Authorization", api_key);
        if is_first_page {
            request = request
                .query(&[
                    ("min-rank", config.min_rank.as_str()),
                    ("sort-by", "replay-date"),
                    ("sort-dir", "desc"),
                ])
                .query(&[("count", count)]);
            if !config.playlist.trim().is_empty() && config.playlist != "any" {
                request = request.query(&[("playlist", config.playlist.as_str())]);
            }
            if let SearchQuery::PlayerName(player_name) = query {
                request = request.query(&[("player-name", player_name.as_str())]);
            }
        }

        let response: Value = request
            .send()
            .with_context(|| format!("Failed to query Ballchasing for {}", query.label()))?
            .error_for_status()
            .with_context(|| format!("Ballchasing returned an error for {}", query.label()))?
            .json()
            .with_context(|| format!("Failed to decode Ballchasing JSON for {}", query.label()))?;
        sleep(Duration::from_millis(config.sleep_millis));
        results.extend(extract_replay_summaries(&response));
        next_url = response
            .get("next")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        is_first_page = false;
        page_index += 1;
    }

    Ok(results)
}

fn cached_replay_path(output_dir: &Path, replay_id: &str) -> PathBuf {
    output_dir.join("cache").join(format!("{replay_id}.replay"))
}

fn saved_positive_replay_path(output_dir: &Path, replay_id: &str) -> PathBuf {
    output_dir
        .join("replays")
        .join(format!("{replay_id}.replay"))
}

fn fetch_replay_bytes(
    client: &Client,
    api_key: &str,
    config: &Config,
    replay_id: &str,
) -> Result<Vec<u8>> {
    let replay_path = cached_replay_path(&config.output_dir, replay_id);
    if replay_path.exists() {
        return fs::read(&replay_path)
            .with_context(|| format!("Failed to read cached replay {}", replay_path.display()));
    }

    let parent = replay_path
        .parent()
        .context("Cached replay path should have a parent directory")?;
    fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;

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
    sleep(Duration::from_millis(config.sleep_millis));
    fs::write(&replay_path, replay_bytes.as_ref())
        .with_context(|| format!("Failed to write cached replay {}", replay_path.display()))?;
    Ok(replay_bytes.to_vec())
}

fn replay_supports_exact_dodge_refreshes(replay: &boxcars::Replay) -> bool {
    replay
        .objects
        .iter()
        .any(|name| name == DODGES_REFRESHED_COUNTER_KEY)
}

fn positive_replay_entry(
    config: &Config,
    summary: &ReplaySummary,
    replay_bytes: &[u8],
) -> Result<Option<FlipResetTuningReplay>> {
    let replay = parse_replay_bytes(replay_bytes)?;
    if !replay_supports_exact_dodge_refreshes(&replay) {
        return Ok(None);
    }

    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .map_err(|error| anyhow::Error::new(error.variant))
        .context("Failed to compute replay data")?;
    let exact_dodge_refresh_count = replay_data.dodge_refreshed_events.len();
    if exact_dodge_refresh_count == 0 {
        return Ok(None);
    }

    let positive_path = saved_positive_replay_path(&config.output_dir, &summary.id);
    let parent = positive_path
        .parent()
        .context("Positive replay path should have a parent directory")?;
    fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    fs::write(&positive_path, replay_bytes).with_context(|| {
        format!(
            "Failed to write positive replay {}",
            positive_path.display()
        )
    })?;

    Ok(Some(FlipResetTuningReplay {
        replay_id: summary.id.clone(),
        replay_path: positive_path
            .strip_prefix(&config.output_dir)
            .unwrap_or(&positive_path)
            .to_string_lossy()
            .to_string(),
        exact_dodge_refresh_count,
        date: summary.date.clone(),
        title: summary.title.clone(),
        uploader: summary.uploader.clone(),
        player_names: replay_data
            .meta
            .player_order()
            .map(|player| player.name.clone())
            .collect(),
    }))
}

fn manifest_path(output_dir: &Path) -> PathBuf {
    output_dir.join("manifest.json")
}

fn player_name_is_searchable(player_name: &str) -> bool {
    let trimmed = player_name.trim();
    trimmed
        .chars()
        .filter(|character| character.is_alphanumeric())
        .count()
        >= 2
}

fn save_manifest(config: &Config, replays: &[FlipResetTuningReplay]) -> Result<()> {
    fs::create_dir_all(&config.output_dir)
        .with_context(|| format!("Failed to create {}", config.output_dir.display()))?;
    let manifest = FlipResetTuningManifest {
        playlist: config.playlist.clone(),
        min_rank: config.min_rank.clone(),
        replays: replays.to_vec(),
    };
    let json = serde_json::to_string_pretty(&manifest).context("Failed to encode manifest")?;
    let path = manifest_path(&config.output_dir);
    fs::write(&path, json).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn load_manifest(output_dir: &Path) -> Result<Vec<FlipResetTuningReplay>> {
    let path = manifest_path(output_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let manifest: FlipResetTuningManifest =
        serde_json::from_slice(&bytes).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(manifest.replays)
}

fn main() -> Result<()> {
    let config = Config::from_args()?;
    let api_key =
        std::env::var("BALLCHASING_API_KEY").context("BALLCHASING_API_KEY must be set")?;
    let client = Client::builder()
        .build()
        .context("Failed to build HTTP client")?;

    let mut seen_replay_ids = HashSet::new();
    let mut searched_queries = HashSet::new();
    let mut replay_queue = VecDeque::new();
    let mut search_queue = VecDeque::from([SearchQuery::Recent]);
    for seed_player in &config.seed_players {
        if player_name_is_searchable(seed_player) {
            search_queue.push_back(SearchQuery::PlayerName(seed_player.clone()));
        }
    }
    let mut positive_replays = load_manifest(&config.output_dir)?;
    let mut positive_player_names = HashSet::new();
    for replay in &positive_replays {
        seen_replay_ids.insert(replay.replay_id.clone());
        for player_name in &replay.player_names {
            if player_name_is_searchable(player_name) && positive_player_names.insert(player_name.clone()) {
                search_queue.push_back(SearchQuery::PlayerName(player_name.clone()));
            }
        }
    }

    while positive_replays.len() < config.count {
        if replay_queue.is_empty() {
            let Some(query) = search_queue.pop_front() else {
                break;
            };
            if !searched_queries.insert(query.clone()) {
                continue;
            }
            eprintln!("searching {}", query.label());
            for summary in search_replays(&client, &api_key, &config, &query)? {
                if seen_replay_ids.insert(summary.id.clone()) {
                    replay_queue.push_back(summary);
                }
            }
            continue;
        }

        let Some(summary) = replay_queue.pop_front() else {
            continue;
        };
        eprintln!(
            "[{}/{} positives] checking {} ({})",
            positive_replays.len(),
            config.count,
            summary.id,
            summary.uploader.as_deref().unwrap_or("unknown uploader")
        );
        let replay_bytes = fetch_replay_bytes(&client, &api_key, &config, &summary.id)
            .with_context(|| format!("Failed to fetch replay {}", summary.id))?;
        let Some(entry) = positive_replay_entry(&config, &summary, &replay_bytes)
            .with_context(|| format!("Failed to inspect replay {}", summary.id))?
        else {
            continue;
        };

        eprintln!(
            "  kept {} with {} exact dodge refreshes",
            entry.replay_id, entry.exact_dodge_refresh_count
        );
        for player_name in &entry.player_names {
            if player_name_is_searchable(player_name)
                && positive_player_names.insert(player_name.clone())
            {
                search_queue.push_back(SearchQuery::PlayerName(player_name.clone()));
            }
        }
        positive_replays.push(entry);
        save_manifest(&config, &positive_replays)?;
    }

    save_manifest(&config, &positive_replays)?;
    println!(
        "Saved {} positive flip-reset replays to {}",
        positive_replays.len(),
        config.output_dir.display()
    );
    println!("Manifest: {}", manifest_path(&config.output_dir).display());
    Ok(())
}
