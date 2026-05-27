use std::path::Path;

use anyhow::Context;

pub(crate) fn normalize_ballchasing_id(input: &str) -> String {
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

pub(crate) fn load_ids_file(path: &Path) -> anyhow::Result<Vec<String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read ids file {}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(normalize_ballchasing_id)
        .collect())
}

pub(crate) fn ballchasing_api_key() -> anyhow::Result<String> {
    std::env::var("BALLCHASING_API_KEY")
        .context("BALLCHASING_API_KEY must be set for Ballchasing API calls")
}
