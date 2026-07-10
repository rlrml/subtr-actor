//! Measure per-replay event rates to judge how many human-labeling candidates
//! various gates would produce.
//!
//! For each replay it prints WhiffEvent counts (split whiff vs beaten_to_ball),
//! total touch count, touch counts broken down by classification tag (group:value,
//! especially the `possession` control/advance split) plus the number of touches
//! carrying no `possession=control` tag ("attempt-like" touches), and the replay
//! duration / player count so rates can be normalized. It then prints an aggregate
//! summary (total, mean per replay, mean per minute).

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;
use serde_json::{Value, json};
use subtr_actor::{
    BeatenToBallEvent, EventPayload, PlayerId, StatsTimelineCollector, WhiffEvent, WhiffEventKind,
};

/// Match tolerance (seconds) between a legacy beaten-to-ball whiff and a new
/// touch-anchored BeatenToBallEvent for the same loser.
const COVERAGE_MATCH_TOLERANCE_SECONDS: f32 = 0.75;

/// Seconds of context to include before a candidate's anchor time in review
/// playlist clips.
const PLAYLIST_LEAD_SECONDS: f32 = 4.0;
/// Seconds of context to include after a candidate's anchor time in review
/// playlist clips.
const PLAYLIST_TAIL_SECONDS: f32 = 2.0;

#[derive(Debug, Parser)]
#[command(about = "Measure per-replay whiff and touch event rates for a set of replays.")]
struct Args {
    /// Replay files to audit.
    replay_paths: Vec<PathBuf>,
    /// Directory to glob `*.replay` files from (in addition to any positional paths).
    #[arg(long)]
    dir: Option<PathBuf>,
    /// Write a beaten-to-ball mechanics-review playlist JSON to this path.
    #[arg(long, value_name = "output.json")]
    emit_playlist: Option<PathBuf>,
    /// Labeling dataset name; selects the labels.jsonl used for resume and the
    /// review endpoint path.
    #[arg(long, default_value = "beaten-to-ball")]
    dataset: String,
    /// Include candidates already present in the dataset's labels.jsonl.
    #[arg(long)]
    include_labeled: bool,
}

#[derive(Debug, Default, Clone)]
struct ReplayReport {
    label: String,
    /// Source path, so the playlist emitter can reference the replay file.
    path: PathBuf,
    duration_minutes: f32,
    player_count: usize,
    whiff_total: usize,
    whiff_whiff: usize,
    whiff_beaten_to_ball: usize,
    /// New touch-anchored BeatenToBallEvent stream (distinct from the legacy
    /// WhiffEventKind::BeatenToBall resolution of whiff candidates).
    beaten_to_ball_events: usize,
    touch_total: usize,
    /// Count per `group:value` tag key.
    tag_counts: BTreeMap<String, usize>,
    /// Touches with no `possession=control` tag ("attempt-like").
    touch_no_control: usize,
    /// Legacy beaten-to-ball whiffs, for coverage comparison.
    legacy_beaten: Vec<WhiffEvent>,
    /// New touch-anchored beaten-to-ball events, for coverage comparison.
    new_beaten: Vec<BeatenToBallEvent>,
    /// Player display names keyed by formatted player-id string.
    player_names: BTreeMap<String, String>,
}

/// Which legacy timestamp field a coverage match used as the anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchAnchor {
    ResolvedTime,
    Time,
}

impl MatchAnchor {
    fn label(self) -> &'static str {
        match self {
            MatchAnchor::ResolvedTime => "resolved_time",
            MatchAnchor::Time => "time",
        }
    }

    fn legacy_anchor(self, legacy: &WhiffEvent) -> f32 {
        match self {
            MatchAnchor::ResolvedTime => legacy.resolved_time,
            MatchAnchor::Time => legacy.time,
        }
    }
}

/// Whether any new BeatenToBallEvent matches a given legacy event under the
/// chosen anchor: same loser player, event time within tolerance.
fn legacy_is_matched(
    legacy: &WhiffEvent,
    new_events: &[BeatenToBallEvent],
    anchor: MatchAnchor,
) -> bool {
    let anchor_time = anchor.legacy_anchor(legacy);
    new_events.iter().any(|new| {
        new.player == legacy.player
            && (new.time - anchor_time).abs() <= COVERAGE_MATCH_TOLERANCE_SECONDS
    })
}

/// Count matched legacy beaten-to-ball events across all replays under an anchor.
fn count_matched(reports: &[ReplayReport], anchor: MatchAnchor) -> usize {
    reports
        .iter()
        .map(|report| {
            report
                .legacy_beaten
                .iter()
                .filter(|legacy| legacy_is_matched(legacy, &report.new_beaten, anchor))
                .count()
        })
        .sum()
}

fn collect_replay_paths(args: &Args) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = args.replay_paths.clone();
    if let Some(dir) = &args.dir {
        let mut globbed: Vec<PathBuf> = std::fs::read_dir(dir)
            .with_context(|| format!("reading dir {}", dir.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("replay"))
            .collect();
        globbed.sort();
        paths.extend(globbed);
    }
    Ok(paths)
}

fn parse_replay(path: &PathBuf) -> anyhow::Result<boxcars::Replay> {
    let data = std::fs::read(path).with_context(|| format!("reading replay {}", path.display()))?;
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .with_context(|| format!("parsing replay {}", path.display()))
}

fn audit_replay(path: &PathBuf) -> anyhow::Result<ReplayReport> {
    let label = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("<replay>")
        .to_owned();
    let parsed = parse_replay(path)?;
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&parsed)
        .map_err(|error| anyhow::anyhow!("failed to build stats timeline: {error:?}"))?;

    let first_time = timeline.frames.first().map(|frame| frame.time);
    let last_time = timeline.frames.last().map(|frame| frame.time);
    let duration_minutes = match (first_time, last_time) {
        (Some(first), Some(last)) => ((last - first).max(0.0)) / 60.0,
        _ => 0.0,
    };
    let player_count = timeline.replay_meta.team_zero.len() + timeline.replay_meta.team_one.len();
    let player_names: BTreeMap<String, String> = timeline
        .replay_meta
        .team_zero
        .iter()
        .chain(timeline.replay_meta.team_one.iter())
        .map(|player| (player_id_string(&player.remote_id), player.name.clone()))
        .collect();

    let mut report = ReplayReport {
        label,
        path: path.clone(),
        duration_minutes,
        player_count,
        player_names,
        ..ReplayReport::default()
    };

    for event in &timeline.events.events {
        match &event.payload {
            EventPayload::Whiff(whiff) => {
                report.whiff_total += 1;
                match whiff.kind {
                    WhiffEventKind::Whiff => report.whiff_whiff += 1,
                    WhiffEventKind::BeatenToBall => {
                        report.whiff_beaten_to_ball += 1;
                        report.legacy_beaten.push(whiff.clone());
                    }
                }
            }
            EventPayload::BeatenToBall(beaten) => {
                report.beaten_to_ball_events += 1;
                report.new_beaten.push(beaten.clone());
            }
            EventPayload::Touch(touch) => {
                report.touch_total += 1;
                for tag in &touch.tags {
                    let key = format!("{}:{}", tag.group, tag.value);
                    *report.tag_counts.entry(key).or_insert(0) += 1;
                }
                if !touch.has_tag("possession", "control") {
                    report.touch_no_control += 1;
                }
            }
            _ => {}
        }
    }

    Ok(report)
}

fn print_replay_report(report: &ReplayReport) {
    println!("== {} ==", report.label);
    println!(
        "  duration: {:.2} min    players: {}",
        report.duration_minutes, report.player_count
    );
    println!(
        "  whiffs: {} total  (whiff={}, beaten_to_ball={})",
        report.whiff_total, report.whiff_whiff, report.whiff_beaten_to_ball
    );
    println!(
        "  beaten_to_ball events (touch-anchored): {}",
        report.beaten_to_ball_events
    );
    println!(
        "  touches: {} total  (no-control / attempt-like={})",
        report.touch_total, report.touch_no_control
    );
    println!("  touch tag breakdown (group:value = count):");
    let width = report
        .tag_counts
        .keys()
        .map(|key| key.len())
        .max()
        .unwrap_or(0);
    for (key, count) in &report.tag_counts {
        println!("    {key:<width$}  {count}");
    }
    println!();
}

fn print_aggregate(reports: &[ReplayReport]) {
    let replay_count = reports.len();
    if replay_count == 0 {
        println!("No replays successfully processed.");
        return;
    }

    let total_minutes: f32 = reports.iter().map(|report| report.duration_minutes).sum();
    let whiff_total: usize = reports.iter().map(|report| report.whiff_total).sum();
    let whiff_whiff: usize = reports.iter().map(|report| report.whiff_whiff).sum();
    let whiff_beaten: usize = reports
        .iter()
        .map(|report| report.whiff_beaten_to_ball)
        .sum();
    let beaten_events: usize = reports
        .iter()
        .map(|report| report.beaten_to_ball_events)
        .sum();
    let touch_total: usize = reports.iter().map(|report| report.touch_total).sum();
    let touch_no_control: usize = reports.iter().map(|report| report.touch_no_control).sum();

    let mut tag_totals: BTreeMap<String, usize> = BTreeMap::new();
    for report in reports {
        for (key, count) in &report.tag_counts {
            *tag_totals.entry(key.clone()).or_insert(0) += count;
        }
    }

    let per_replay = |total: usize| total as f32 / replay_count as f32;
    let per_minute = |total: usize| {
        if total_minutes > 0.0 {
            total as f32 / total_minutes
        } else {
            0.0
        }
    };

    println!("================= AGGREGATE ({replay_count} replays) =================");
    println!(
        "total replay duration: {total_minutes:.2} min    total players seen: {}",
        reports
            .iter()
            .map(|report| report.player_count)
            .sum::<usize>()
    );
    println!();

    let name_width = 32usize;
    println!(
        "{:<name_width$} {:>10} {:>14} {:>14}",
        "metric", "total", "mean/replay", "mean/min"
    );
    let row = |name: &str, total: usize| {
        println!(
            "{name:<name_width$} {total:>10} {:>14.3} {:>14.3}",
            per_replay(total),
            per_minute(total)
        );
    };
    row("whiffs (all)", whiff_total);
    row("  whiff", whiff_whiff);
    row("  beaten_to_ball", whiff_beaten);
    row("beaten_to_ball events (new)", beaten_events);
    row("touches (all)", touch_total);
    row("  touches no-control", touch_no_control);
    println!();
    println!("touch tag totals (group:value):");
    for (key, total) in &tag_totals {
        row(&format!("  {key}"), *total);
    }
}

/// Pick whichever legacy timestamp field yields better coverage as the anchor.
fn select_match_anchor(reports: &[ReplayReport]) -> MatchAnchor {
    let resolved_matched = count_matched(reports, MatchAnchor::ResolvedTime);
    let time_matched = count_matched(reports, MatchAnchor::Time);
    if time_matched > resolved_matched {
        MatchAnchor::Time
    } else {
        MatchAnchor::ResolvedTime
    }
}

fn print_coverage(reports: &[ReplayReport], anchor: MatchAnchor) {
    let legacy_total: usize = reports
        .iter()
        .map(|report| report.legacy_beaten.len())
        .sum();

    println!();
    println!("================= BEATEN-TO-BALL COVERAGE OVERLAP =================");
    if legacy_total == 0 {
        println!("No legacy beaten_to_ball whiff events to compare.");
        return;
    }

    let resolved_matched = count_matched(reports, MatchAnchor::ResolvedTime);
    let time_matched = count_matched(reports, MatchAnchor::Time);
    println!(
        "match tolerance: +/-{COVERAGE_MATCH_TOLERANCE_SECONDS:.2}s   \
         matches by resolved_time={resolved_matched}, by time={time_matched}   \
         using anchor: {}",
        anchor.label()
    );
    println!();

    let mut matched_total = 0usize;
    let mut unmatched_total = 0usize;
    let mut unmatched_lines: Vec<String> = Vec::new();

    for report in reports {
        if report.legacy_beaten.is_empty() {
            continue;
        }
        let stem = report
            .label
            .strip_suffix(".replay")
            .unwrap_or(&report.label);
        let mut matched = 0usize;
        let mut unmatched = 0usize;
        for legacy in &report.legacy_beaten {
            if legacy_is_matched(legacy, &report.new_beaten, anchor) {
                matched += 1;
            } else {
                unmatched += 1;
                unmatched_lines.push(format!(
                    "  UNMATCHED {stem}  time={:.3}  resolved_time={:.3}  player={:?}  \
                     closest_approach_distance={:.1}  approach_speed={:.1}",
                    legacy.time,
                    legacy.resolved_time,
                    legacy.player,
                    legacy.closest_approach_distance,
                    legacy.approach_speed,
                ));
            }
        }
        matched_total += matched;
        unmatched_total += unmatched;
        println!(
            "  {stem}: legacy_beaten={}  matched={matched}  unmatched={unmatched}  (new_events={})",
            report.legacy_beaten.len(),
            report.new_beaten.len()
        );
    }

    println!();
    println!(
        "AGGREGATE COVERAGE: {matched_total} of {legacy_total} legacy beaten_to_ball events \
         matched by the new detector  (unmatched={unmatched_total})"
    );
    println!();
    if unmatched_lines.is_empty() {
        println!("No unmatched legacy beaten_to_ball events.");
    } else {
        println!("Unmatched legacy beaten_to_ball events:");
        for line in &unmatched_lines {
            println!("{line}");
        }
    }
}

/// Format a `PlayerId` the same way `@rlrml/player` formats
/// `ReplayPlayerTrack.id` (and `build_mechanic_review_playlist` formats
/// perspective player ids): `<platform>:<online-id>`.
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

/// Minimal percent-encoding for URL query values (RFC 3986 unreserved set).
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Where a review-playlist candidate came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateProvenance {
    /// New touch-anchored event with no matching legacy event.
    NewOnly,
    /// New touch-anchored event that also matched a legacy event.
    Both,
    /// Legacy whiff-resolved event the new detector missed.
    LegacyOnly,
}

impl CandidateProvenance {
    fn label(self) -> &'static str {
        match self {
            CandidateProvenance::NewOnly => "new_only",
            CandidateProvenance::Both => "both",
            CandidateProvenance::LegacyOnly => "legacy_only",
        }
    }

    /// The source tag used in stable candidate ids.
    fn source(self) -> &'static str {
        match self {
            CandidateProvenance::NewOnly | CandidateProvenance::Both => "new",
            CandidateProvenance::LegacyOnly => "legacy",
        }
    }
}

/// One beaten-to-ball labeling candidate destined for the review playlist.
struct PlaylistCandidate {
    replay_stem: String,
    anchor_time: f32,
    anchor_frame: usize,
    provenance: CandidateProvenance,
    player_id: String,
    player_name: String,
    label: String,
    reason: String,
    payload: Value,
}

impl PlaylistCandidate {
    /// Stable id: `<stem12>:beaten_to_ball:<new|legacy>:<frame>:<player-id>`.
    fn id(&self) -> String {
        format!(
            "{}:beaten_to_ball:{}:{}:{}",
            stem12(&self.replay_stem),
            self.provenance.source(),
            self.anchor_frame,
            self.player_id
        )
    }
}

fn stem12(stem: &str) -> &str {
    &stem[..stem.len().min(12)]
}

fn replay_stem(report: &ReplayReport) -> &str {
    report
        .label
        .strip_suffix(".replay")
        .unwrap_or(&report.label)
}

fn display_name(report: &ReplayReport, player_id: &str) -> String {
    report
        .player_names
        .get(player_id)
        .cloned()
        .unwrap_or_else(|| player_id.to_owned())
}

fn build_playlist_candidates(
    reports: &[ReplayReport],
    anchor: MatchAnchor,
) -> Vec<PlaylistCandidate> {
    let mut candidates = Vec::new();
    for report in reports {
        let stem = replay_stem(report);

        for new in &report.new_beaten {
            let matched = report.legacy_beaten.iter().any(|legacy| {
                legacy.player == new.player
                    && (new.time - anchor.legacy_anchor(legacy)).abs()
                        <= COVERAGE_MATCH_TOLERANCE_SECONDS
            });
            let provenance = if matched {
                CandidateProvenance::Both
            } else {
                CandidateProvenance::NewOnly
            };
            let player_id = player_id_string(&new.player);
            let player_name = display_name(report, &player_id);
            let winner_name = display_name(report, &player_id_string(&new.winner));
            candidates.push(PlaylistCandidate {
                replay_stem: stem.to_owned(),
                anchor_time: new.time,
                anchor_frame: new.frame,
                provenance,
                player_id,
                label: format!("Beaten to ball — {player_name} (lost to {winner_name})"),
                player_name,
                reason: format!(
                    "margin_seconds={:.3} distance_at_touch={:.1}",
                    new.margin_seconds, new.distance_at_touch
                ),
                payload: serde_json::to_value(new).unwrap_or(Value::Null),
            });
        }

        for legacy in &report.legacy_beaten {
            if legacy_is_matched(legacy, &report.new_beaten, anchor) {
                continue;
            }
            let player_id = player_id_string(&legacy.player);
            let player_name = display_name(report, &player_id);
            candidates.push(PlaylistCandidate {
                replay_stem: stem.to_owned(),
                anchor_time: legacy.resolved_time,
                anchor_frame: legacy.resolved_frame,
                provenance: CandidateProvenance::LegacyOnly,
                player_id,
                label: format!("Legacy beaten-to-ball — {player_name}"),
                player_name,
                reason: format!(
                    "closest_approach_distance={:.1}",
                    legacy.closest_approach_distance
                ),
                payload: serde_json::to_value(legacy).unwrap_or(Value::Null),
            });
        }
    }

    candidates.sort_by(|left, right| {
        left.replay_stem
            .cmp(&right.replay_stem)
            .then(left.anchor_time.total_cmp(&right.anchor_time))
    });
    candidates
}

/// Candidate ids already labeled in `<repo-root>/labels/<dataset>/labels.jsonl`.
fn load_labeled_candidates(dataset: &str) -> anyhow::Result<HashSet<String>> {
    let labels_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("labels")
        .join(dataset)
        .join("labels.jsonl");
    if !labels_path.exists() {
        return Ok(HashSet::new());
    }
    let text = std::fs::read_to_string(&labels_path)
        .with_context(|| format!("reading labels {}", labels_path.display()))?;
    let mut labeled = HashSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("parsing labels line in {}", labels_path.display()))?;
        if let Some(candidate) = value.get("candidate").and_then(Value::as_str) {
            labeled.insert(candidate.to_owned());
        }
    }
    Ok(labeled)
}

fn candidate_item_json(candidate: &PlaylistCandidate, dataset: &str) -> Value {
    let id = candidate.id();
    let provenance = candidate.provenance.label();
    let review_endpoint = format!(
        "/review-labels/{dataset}?candidate={}&provenance={provenance}&replay={}&frame={}&player={}",
        urlencode(&id),
        stem12(&candidate.replay_stem),
        candidate.anchor_frame,
        urlencode(&candidate.player_id),
    );
    json!({
        "id": id,
        "replay": candidate.replay_stem,
        "start": { "kind": "time", "value": (candidate.anchor_time - PLAYLIST_LEAD_SECONDS).max(0.0) },
        "end": { "kind": "time", "value": candidate.anchor_time + PLAYLIST_TAIL_SECONDS },
        "label": candidate.label,
        "perspective": {
            "kind": "player",
            "playerId": candidate.player_id,
            "playerName": candidate.player_name,
            "ballCam": "on",
        },
        "meta": {
            "eventId": id,
            "eventType": "beaten_to_ball",
            "eventTypeLabel": "Beaten to ball",
            "playerName": candidate.player_name,
            "provenance": provenance,
            "reason": candidate.reason,
            "reviewEndpoint": review_endpoint,
            "target": {
                "eventTime": candidate.anchor_time,
                "eventFrame": candidate.anchor_frame,
            },
            "payload": candidate.payload,
        },
    })
}

fn emit_playlist(
    reports: &[ReplayReport],
    anchor: MatchAnchor,
    args: &Args,
    output: &Path,
) -> anyhow::Result<()> {
    let all_candidates = build_playlist_candidates(reports, anchor);
    let labeled = if args.include_labeled {
        HashSet::new()
    } else {
        load_labeled_candidates(&args.dataset)?
    };

    let total_before = all_candidates.len();
    let candidates: Vec<&PlaylistCandidate> = all_candidates
        .iter()
        .filter(|candidate| !labeled.contains(&candidate.id()))
        .collect();
    let skipped_labeled = total_before - candidates.len();

    let mut provenance_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut replay_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for candidate in &candidates {
        *provenance_counts
            .entry(candidate.provenance.label())
            .or_insert(0) += 1;
        *replay_counts.entry(&candidate.replay_stem).or_insert(0) += 1;
    }

    let replays: Vec<Value> = reports
        .iter()
        .filter(|report| replay_counts.contains_key(replay_stem(report)))
        .map(|report| {
            let path = std::fs::canonicalize(&report.path).unwrap_or_else(|_| report.path.clone());
            json!({
                "id": replay_stem(report),
                "path": path.display().to_string(),
                "label": replay_stem(report),
            })
        })
        .collect();

    let items: Vec<Value> = candidates
        .iter()
        .map(|candidate| candidate_item_json(candidate, &args.dataset))
        .collect();
    let item_count = items.len();

    let playlist = json!({
        "version": 1,
        "kind": "mechanic-review-playlist",
        "label": format!("Beaten-to-ball review ({})", args.dataset),
        "playback": { "advanceMode": "manual", "endMode": "stop" },
        "replays": replays,
        "items": items,
        "meta": {
            "dataset": args.dataset,
            "eventType": "beaten_to_ball",
            "candidateCount": item_count,
            "skippedLabeledCount": skipped_labeled,
            "provenanceCounts": provenance_counts,
            "generatedBy": "event_rate_audit",
        },
    });

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating output dir {}", parent.display()))?;
        }
    }
    let json = serde_json::to_string_pretty(&playlist)?;
    std::fs::write(output, format!("{json}\n"))
        .with_context(|| format!("writing playlist {}", output.display()))?;

    println!();
    println!("================= REVIEW PLAYLIST =================");
    println!("wrote {} items to {}", item_count, output.display());
    println!("skipped {skipped_labeled} already-labeled candidates");
    println!("items by provenance:");
    for (provenance, count) in &provenance_counts {
        println!("  {provenance:<12} {count}");
    }
    println!("items by replay:");
    for (stem, count) in &replay_counts {
        println!("  {stem}  {count}");
    }
    println!("total: {item_count}");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let paths = collect_replay_paths(&args)?;
    if paths.is_empty() {
        anyhow::bail!("no replay paths provided; pass positional paths and/or --dir");
    }

    let mut reports = Vec::new();
    for path in &paths {
        match audit_replay(path) {
            Ok(report) => {
                print_replay_report(&report);
                reports.push(report);
            }
            Err(error) => {
                eprintln!("FAILED {}: {error:#}", path.display());
            }
        }
    }

    print_aggregate(&reports);
    let anchor = select_match_anchor(&reports);
    print_coverage(&reports, anchor);
    if let Some(output) = &args.emit_playlist {
        emit_playlist(&reports, anchor, &args, output)?;
    }
    Ok(())
}
