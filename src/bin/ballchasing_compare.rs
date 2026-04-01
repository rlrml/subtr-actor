use anyhow::{bail, Context};
use reqwest::blocking::Client;
use serde_json::Value;
use subtr_actor::ballchasing::{
    compare_replay_against_ballchasing, parse_replay_bytes, recommended_match_config,
};

fn normalize_replay_id(input: &str) -> &str {
    input
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(input)
        .split('?')
        .next()
        .unwrap_or(input)
}

fn main() -> anyhow::Result<()> {
    let replay_id = std::env::args()
        .nth(1)
        .context("Usage: cargo run --bin ballchasing_compare -- <ballchasing-replay-id-or-url>")?;
    let api_key = std::env::var("BALLCHASING_API_KEY")
        .context("BALLCHASING_API_KEY must be set to fetch Ballchasing replay data")?;
    let replay_id = normalize_replay_id(&replay_id);
    let client = Client::new();

    let replay_json_url = format!("https://ballchasing.com/api/replays/{replay_id}");
    let replay_file_url = format!("https://ballchasing.com/api/replays/{replay_id}/file");

    let ballchasing: Value = client
        .get(&replay_json_url)
        .header("Authorization", &api_key)
        .send()
        .with_context(|| format!("Failed to fetch {replay_json_url}"))?
        .error_for_status()
        .with_context(|| format!("Ballchasing returned an error for {replay_json_url}"))?
        .json()
        .with_context(|| format!("Failed to decode JSON from {replay_json_url}"))?;

    let replay_bytes = client
        .get(&replay_file_url)
        .header("Authorization", &api_key)
        .send()
        .with_context(|| format!("Failed to fetch {replay_file_url}"))?
        .error_for_status()
        .with_context(|| format!("Ballchasing returned an error for {replay_file_url}"))?
        .bytes()
        .with_context(|| format!("Failed to read replay bytes from {replay_file_url}"))?;

    let replay = parse_replay_bytes(replay_bytes.as_ref())?;
    let report =
        compare_replay_against_ballchasing(&replay, &ballchasing, &recommended_match_config())
            .map_err(|error| anyhow::Error::new(error.variant))
            .context("Failed to compute comparable stats from replay")?;

    if report.is_match() {
        println!("Ballchasing comparison matched for replay {replay_id}");
        return Ok(());
    }

    for mismatch in report.mismatches() {
        eprintln!("{mismatch}");
    }

    bail!(
        "Ballchasing comparison found {} mismatches for replay {}",
        report.mismatches().len(),
        replay_id
    )
}
