use std::collections::{HashMap, HashSet};

use subtr_actor::stats::analysis_graph::StatsProjectionState;
use subtr_actor::*;

const TEST_BOOST_ZERO_BAND_RAW: f32 = 1.0;
const TEST_BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;
const REPLAY_FORMAT_EVOLUTION_DOC: &str = include_str!("../../docs/replay-format-evolution.md");

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn replay_format_fixture_paths() -> Vec<String> {
    let fixture_filter = std::env::var("SUBTR_ACTOR_REPLAY_FORMAT_FIXTURE").ok();
    REPLAY_FORMAT_EVOLUTION_DOC
        .lines()
        .filter_map(|line| {
            let start = line.find("| `")? + 3;
            let rest = &line[start..];
            let end = rest.find("` |")?;
            let fixture = &rest[..end];
            fixture
                .ends_with(".replay")
                .then(|| format!("assets/{fixture}"))
        })
        .filter(|path| {
            fixture_filter
                .as_ref()
                .map(|filter| path.contains(filter))
                .unwrap_or(true)
        })
        .collect()
}

fn asset_replay_fixture_paths() -> Vec<String> {
    let fixture_filter = std::env::var("SUBTR_ACTOR_REPLAY_FIXTURE").ok();
    let mut replay_paths = std::fs::read_dir("assets")
        .expect("expected checked-in replay asset directory")
        .filter_map(|entry| {
            let entry = entry.expect("expected replay asset directory entry");
            let path = entry.path();
            (path
                .extension()
                .is_some_and(|extension| extension == "replay"))
            .then(|| {
                path.to_str()
                    .expect("expected replay fixture path to be valid UTF-8")
                    .to_owned()
            })
        })
        .filter(|path| {
            fixture_filter
                .as_ref()
                .map(|filter| path.contains(filter))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    replay_paths.sort();
    replay_paths
}

fn frame_total_goals(frame: &ReplayStatsFrame) -> i32 {
    frame.team_zero.core.goals + frame.team_one.core.goals
}

fn player_snapshot_by_name<'a>(
    frame: &'a ReplayStatsFrame,
    player_name: &str,
) -> &'a PlayerStatsSnapshot {
    frame
        .players
        .iter()
        .find(|player| player.name == player_name)
        .unwrap_or_else(|| {
            panic!(
                "Missing player {player_name} in frame {} (t={:.3})",
                frame.frame_number, frame.time
            )
        })
}

fn player_names(frame: &ReplayStatsFrame) -> HashSet<&str> {
    frame
        .players
        .iter()
        .map(|player| player.name.as_str())
        .collect()
}

fn normalized_team_stats_for_live_play_comparison(
    snapshot: &TeamStatsSnapshot,
) -> TeamStatsSnapshot {
    let mut normalized = snapshot.clone();
    normalized.core = CoreTeamStats::default();
    normalize_boost_for_live_play_comparison(&mut normalized.boost);
    normalized.demo = DemoTeamStats::default();
    normalized
}

fn default_team_stats_snapshot() -> TeamStatsSnapshot {
    TeamStatsSnapshot {
        fifty_fifty: FiftyFiftyTeamStats::default(),
        possession: PossessionTeamStats::default(),
        pressure: PressureTeamStats::default(),
        territorial_pressure: TerritorialPressureTeamStats::default(),
        rotation: RotationTeamStats::default(),
        rush: RushTeamStats::default(),
        core: CoreTeamStats::default(),
        backboard: BackboardTeamStats::default(),
        double_tap: DoubleTapTeamStats::default(),
        one_timer: OneTimerTeamStats::default(),
        pass: PassTeamStats::default(),
        ball_carry: BallCarryStats::default(),
        air_dribble: AirDribbleStats::default(),
        boost: BoostStats::default(),
        bump: BumpTeamStats::default(),
        half_volley: HalfVolleyTeamStats::default(),
        movement: MovementStats::default(),
        powerslide: PowerslideStats::default(),
        demo: DemoTeamStats::default(),
    }
}

fn default_player_stats_snapshot(
    player_id: PlayerId,
    name: impl Into<String>,
    is_team_0: bool,
) -> PlayerStatsSnapshot {
    PlayerStatsSnapshot {
        player_id,
        name: name.into(),
        is_team_0,
        core: CorePlayerStats::default(),
        backboard: BackboardPlayerStats::default(),
        ceiling_shot: CeilingShotStats::default(),
        wall_aerial: WallAerialStats::default(),
        wall_aerial_shot: WallAerialShotStats::default(),
        double_tap: DoubleTapPlayerStats::default(),
        one_timer: OneTimerPlayerStats::default(),
        pass: PassPlayerStats::default(),
        fifty_fifty: FiftyFiftyPlayerStats::default(),
        speed_flip: SpeedFlipStats::default(),
        half_flip: HalfFlipStats::default(),
        half_volley: HalfVolleyPlayerStats::default(),
        wavedash: WavedashStats::default(),
        touch: TouchStats::default(),
        whiff: WhiffStats::default(),
        flick: FlickStats::default(),
        musty_flick: MustyFlickStats::default(),
        dodge_reset: DodgeResetStats::default(),
        ball_carry: BallCarryStats::default(),
        air_dribble: AirDribbleStats::default(),
        boost: BoostStats::default(),
        bump: BumpPlayerStats::default(),
        movement: MovementStats::default(),
        positioning: PositioningStats::default(),
        rotation: RotationPlayerStats::default(),
        powerslide: PowerslideStats::default(),
        demo: DemoPlayerStats::default(),
    }
}

fn empty_stats_timeline_config() -> StatsTimelineConfig {
    StatsTimelineConfig {
        most_back_forward_threshold_y: 0.0,
        level_ball_depth_margin: 0.0,
        pressure_neutral_zone_half_width_y: 0.0,
        territorial_pressure_neutral_zone_half_width_y: 0.0,
        territorial_pressure_min_establish_seconds: 0.0,
        territorial_pressure_min_establish_third_seconds: 0.0,
        territorial_pressure_relief_grace_seconds: 0.0,
        territorial_pressure_confirmed_relief_grace_seconds: 0.0,
        rotation_role_depth_margin: 0.0,
        rotation_first_man_ambiguity_margin: 0.0,
        rotation_first_man_debounce_seconds: 0.0,
        rush_max_start_y: 0.0,
        rush_attack_support_distance_y: 0.0,
        rush_defender_distance_y: 0.0,
        rush_min_possession_retained_seconds: 0.0,
        aerial_goal_min_ball_z: 0.0,
        high_aerial_goal_min_ball_z: 0.0,
        long_distance_goal_max_attacking_y: 0.0,
        own_half_goal_max_attacking_y: 0.0,
        empty_net_min_defender_y_margin: 0.0,
        empty_net_min_defender_distance: 0.0,
        empty_net_max_touch_attacking_y: 0.0,
        flick_goal_max_event_to_goal_seconds: 0.0,
        double_tap_goal_max_event_to_goal_seconds: 0.0,
        one_timer_goal_max_event_to_goal_seconds: 0.0,
        air_dribble_goal_max_end_to_goal_seconds: 0.0,
        flip_reset_goal_max_event_to_goal_seconds: 0.0,
        half_volley_max_bounce_to_touch_seconds: 0.0,
        half_volley_min_ball_speed: 0.0,
        half_volley_goal_max_touch_to_goal_seconds: 0.0,
        half_volley_goal_min_goal_alignment: 0.0,
    }
}

fn normalized_player_stats_for_live_play_comparison(
    snapshot: &PlayerStatsSnapshot,
) -> PlayerStatsSnapshot {
    let mut normalized = snapshot.clone();
    normalized.core = CorePlayerStats::default();
    normalize_boost_for_live_play_comparison(&mut normalized.boost);
    normalized.demo = DemoPlayerStats::default();
    normalized
}

fn normalize_boost_for_live_play_comparison(boost: &mut BoostStats) {
    boost.amount_used = 0.0;
    boost.amount_collected_inactive = 0.0;
    boost.big_pads_collected_inactive = 0;
    boost.small_pads_collected_inactive = 0;
    boost
        .labeled_amounts
        .entries
        .retain(|entry| !has_inactive_boost_activity_label(&entry.labels));
    boost
        .labeled_counts
        .entries
        .retain(|entry| !has_inactive_boost_activity_label(&entry.labels));
}

fn has_inactive_boost_activity_label(labels: &[StatLabel]) -> bool {
    labels
        .iter()
        .any(|label| label.key == "activity" && label.value == "inactive")
}

fn complete_movement_breakdowns_for_comparison(movement: &MovementStats) -> MovementStats {
    movement.clone().with_complete_labeled_tracked_time()
}

/// Check that a cumulative stat field never decreases between consecutive frames
/// for any player in the timeline.
fn assert_player_boost_field_monotonic(
    timeline: &ReplayStatsTimeline,
    field_name: &str,
    getter: fn(&BoostStats) -> f64,
) {
    for window in timeline.frames.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        for prev_player in &prev.players {
            let Some(curr_player) = curr
                .players
                .iter()
                .find(|p| p.player_id == prev_player.player_id)
            else {
                continue;
            };
            let prev_val = getter(&prev_player.boost);
            let curr_val = getter(&curr_player.boost);
            assert!(
                curr_val >= prev_val - 1e-4,
                "Player {} {field_name} decreased from {prev_val:.4} to {curr_val:.4} \
                 between frames {} (t={:.3}) and {} (t={:.3})",
                prev_player.name,
                prev.frame_number,
                prev.time,
                curr.frame_number,
                curr.time,
            );
        }
    }
}

/// Check that amount_collected_big + amount_collected_small ≈ amount_collected
/// for every player on every frame.
fn assert_boost_bucket_sums_consistent(timeline: &ReplayStatsTimeline) {
    for frame in &timeline.frames {
        for player in &frame.players {
            let bucket_sum =
                player.boost.amount_collected_big + player.boost.amount_collected_small;
            let diff = (bucket_sum - player.boost.amount_collected).abs();
            assert!(
                diff < 1.0,
                "Player {} bucket mismatch at frame {} (t={:.3}): \
                 big({:.1}) + small({:.1}) = {:.1} vs amount_collected({:.1}), diff={:.1}",
                player.name,
                frame.frame_number,
                frame.time,
                player.boost.amount_collected_big,
                player.boost.amount_collected_small,
                bucket_sum,
                player.boost.amount_collected,
                diff,
            );
        }
    }
}

/// Check that the boost accounting identity holds on every frame:
/// amount_used = max(0, amount_obtained - current_boost), so the
/// implied current boost = amount_obtained - amount_used must be in
/// [0, 255].  If a boost source was missed (e.g. a kickoff respawn),
/// amount_obtained would be too low and current_boost would go negative.
fn assert_boost_accounting_consistent(timeline: &ReplayStatsTimeline) {
    for frame in &timeline.frames {
        for player in &frame.players {
            let obtained = player.boost.amount_obtained();
            let implied_current = obtained - player.boost.amount_used;
            assert!(
                implied_current >= -1.0,
                "Player {} has negative implied boost {:.1} at frame {} (t={:.3}): \
                 obtained({:.1}) - used({:.1}) = {:.1}  [missing boost source?]",
                player.name,
                implied_current,
                frame.frame_number,
                frame.time,
                obtained,
                player.boost.amount_used,
                implied_current,
            );
            assert!(
                implied_current <= 256.0,
                "Player {} has impossible implied boost {:.1} at frame {} (t={:.3}): \
                 obtained({:.1}) - used({:.1}) = {:.1}  [over-counted boost source?]",
                player.name,
                implied_current,
                frame.frame_number,
                frame.time,
                obtained,
                player.boost.amount_used,
                implied_current,
            );
        }
    }
}

/// Check that pad counts imply the same nominal boost total as
/// collected boost plus tracked overfill.
fn assert_boost_pickup_nominal_amounts_consistent(timeline: &ReplayStatsTimeline) {
    fn assert_stats(
        scope: &str,
        frame_number: usize,
        time: f32,
        stats: &BoostStats,
        is_live_play: bool,
    ) {
        let violations = boost_invariant_violations(stats);
        let violations = if is_live_play {
            violations
        } else {
            violations
                .into_iter()
                .filter(|violation| violation.kind != BoostInvariantKind::UsedSplitAmounts)
                .collect()
        };
        assert!(
            violations.is_empty(),
            "{scope} boost invariant violations at frame {frame_number} (t={time:.3}, is_live_play={is_live_play}): {violations:?}"
        );
    }

    for frame in &timeline.frames {
        assert_stats(
            "team_zero",
            frame.frame_number,
            frame.time,
            &frame.team_zero.boost,
            frame.is_live_play,
        );
        assert_stats(
            "team_one",
            frame.frame_number,
            frame.time,
            &frame.team_one.boost,
            frame.is_live_play,
        );
        for player in &frame.players {
            assert_stats(
                &format!("player {}", player.name),
                frame.frame_number,
                frame.time,
                &player.boost,
                frame.is_live_play,
            );
        }
    }
}

/// Check that amount_respawned is within reasonable bounds.
/// Each kickoff/demo grants ~85 raw.  A 7-min game with 15 kickoffs + 10 demos ≈ 2125.
fn assert_boost_respawns_reasonable(timeline: &ReplayStatsTimeline, max_raw: f32) {
    let last_frame = timeline.frames.last().expect("non-empty frames");
    for player in &last_frame.players {
        assert!(
            player.boost.amount_respawned <= max_raw,
            "Player {} has unreasonable amount_respawned: {:.1} (max expected {max_raw:.0})",
            player.name,
            player.boost.amount_respawned,
        );
    }
}

/// Dump final boost stats for every player (diagnostics).
fn dump_final_boost_stats(timeline: &ReplayStatsTimeline) {
    let last_frame = timeline.frames.last().expect("non-empty frames");
    for p in &last_frame.players {
        eprintln!(
            "FINAL {} | collected:{:.0} big_amt:{:.0} small_amt:{:.0} \
             respawn:{:.0} used:{:.0} overfill:{:.0} | \
             big:{} small:{} stolen_big:{} stolen_small:{}",
            p.name,
            p.boost.amount_collected,
            p.boost.amount_collected_big,
            p.boost.amount_collected_small,
            p.boost.amount_respawned,
            p.boost.amount_used,
            p.boost.overfill_total,
            p.boost.big_pads_collected,
            p.boost.small_pads_collected,
            p.boost.big_pads_stolen,
            p.boost.small_pads_stolen,
        );
    }
}

#[derive(Clone, Default)]
struct DerivedBoostLedgerStats {
    stats: BoostStats,
    current_boost_amount: Option<f32>,
    current_boost_before: Option<f32>,
    current_boost_frame: Option<usize>,
    previous_boost_amount: Option<f32>,
}

fn boost_ledger_label<'a>(event: &'a BoostLedgerEvent, key: &str) -> Option<&'a str> {
    event
        .labels
        .iter()
        .find(|label| label.key == key)
        .map(|label| label.value)
}

fn apply_boost_pickup_count(accumulator: &mut DerivedBoostLedgerStats, event: &BoostLedgerEvent) {
    if event.count == 0 {
        return;
    }
    let Some(pad_size @ ("big" | "small")) = boost_ledger_label(event, "pad_size") else {
        return;
    };
    let activity = boost_ledger_label(event, "activity").unwrap_or("unknown");

    match (activity, pad_size) {
        ("inactive", "big") => accumulator.stats.big_pads_collected_inactive += event.count,
        ("inactive", "small") => accumulator.stats.small_pads_collected_inactive += event.count,
        (_, "big") => accumulator.stats.big_pads_collected += event.count,
        (_, "small") => accumulator.stats.small_pads_collected += event.count,
        _ => {}
    }
}

fn apply_boost_ledger_event(accumulator: &mut DerivedBoostLedgerStats, event: &BoostLedgerEvent) {
    let pad_size = boost_ledger_label(event, "pad_size");
    let activity = boost_ledger_label(event, "activity").unwrap_or("active");
    let field_half = boost_ledger_label(event, "field_half");

    match event.transaction {
        BoostLedgerTransactionKind::Collected => {
            apply_boost_pickup_count(accumulator, event);
            if activity == "inactive" {
                accumulator.stats.amount_collected_inactive += event.amount;
                return;
            }
            accumulator.stats.amount_collected += event.amount;
            match pad_size {
                Some("big") => accumulator.stats.amount_collected_big += event.amount,
                Some("small") => accumulator.stats.amount_collected_small += event.amount,
                _ => {}
            }
        }
        BoostLedgerTransactionKind::Stolen => {
            accumulator.stats.amount_stolen += event.amount;
            match pad_size {
                Some("big") => {
                    accumulator.stats.big_pads_stolen += event.count;
                    accumulator.stats.amount_stolen_big += event.amount;
                }
                Some("small") => {
                    accumulator.stats.small_pads_stolen += event.count;
                    accumulator.stats.amount_stolen_small += event.amount;
                }
                _ => {}
            }
        }
        BoostLedgerTransactionKind::Overfill => {
            accumulator.stats.overfill_total += event.amount;
            if field_half == Some("opponent") {
                accumulator.stats.overfill_from_stolen += event.amount;
            }
        }
        BoostLedgerTransactionKind::Respawn => {
            accumulator.stats.amount_respawned += event.amount;
        }
        BoostLedgerTransactionKind::Used => {
            accumulator.stats.amount_used += event.amount;
        }
        BoostLedgerTransactionKind::UsedAllocation => {
            match boost_ledger_label(event, "vertical_state") {
                Some("grounded") => accumulator.stats.amount_used_while_grounded += event.amount,
                Some("aerial") => accumulator.stats.amount_used_while_airborne += event.amount,
                _ => {}
            }
            if boost_ledger_label(event, "supersonic") == Some("true") {
                accumulator.stats.amount_used_while_supersonic += event.amount;
            }
        }
    }
}

fn interval_fraction_in_boost_range(
    start_boost: f32,
    end_boost: f32,
    min_boost: f32,
    max_boost: f32,
) -> f32 {
    if (end_boost - start_boost).abs() <= f32::EPSILON {
        return if (start_boost >= min_boost) && (start_boost < max_boost) {
            1.0
        } else {
            0.0
        };
    }

    let t_at_min = (min_boost - start_boost) / (end_boost - start_boost);
    let t_at_max = (max_boost - start_boost) / (end_boost - start_boost);
    let interval_start = t_at_min.min(t_at_max).max(0.0);
    let interval_end = t_at_min.max(t_at_max).min(1.0);
    (interval_end - interval_start).max(0.0)
}

fn apply_boost_state_event(accumulator: &mut DerivedBoostLedgerStats, event: &BoostStateEvent) {
    accumulator.current_boost_amount = Some(event.boost_amount);
    accumulator.current_boost_before = event.boost_before;
    accumulator.current_boost_frame = Some(event.frame);
}

fn add_boost_state_sample(
    stats: &mut BoostStats,
    previous_boost_amount: f32,
    boost_amount: f32,
    dt: f32,
) {
    let average_boost_amount = (previous_boost_amount + boost_amount) * 0.5;
    stats.tracked_time += dt;
    stats.boost_integral += average_boost_amount * dt;
    stats.time_zero_boost += dt
        * interval_fraction_in_boost_range(
            previous_boost_amount,
            boost_amount,
            0.0,
            TEST_BOOST_ZERO_BAND_RAW,
        );
    stats.time_hundred_boost += dt
        * interval_fraction_in_boost_range(
            previous_boost_amount,
            boost_amount,
            TEST_BOOST_FULL_BAND_MIN_RAW,
            BOOST_MAX_AMOUNT + 1.0,
        );
    stats.time_boost_0_25 += dt
        * interval_fraction_in_boost_range(
            previous_boost_amount,
            boost_amount,
            0.0,
            boost_percent_to_amount(25.0),
        );
    stats.time_boost_25_50 += dt
        * interval_fraction_in_boost_range(
            previous_boost_amount,
            boost_amount,
            boost_percent_to_amount(25.0),
            boost_percent_to_amount(50.0),
        );
    stats.time_boost_50_75 += dt
        * interval_fraction_in_boost_range(
            previous_boost_amount,
            boost_amount,
            boost_percent_to_amount(50.0),
            boost_percent_to_amount(75.0),
        );
    stats.time_boost_75_100 += dt
        * interval_fraction_in_boost_range(
            previous_boost_amount,
            boost_amount,
            boost_percent_to_amount(75.0),
            BOOST_MAX_AMOUNT + 1.0,
        );
}

fn apply_boost_state_sample(
    accumulator: &mut DerivedBoostLedgerStats,
    dt: f32,
    frame_number: usize,
) -> Option<(f32, f32)> {
    if accumulator.current_boost_frame != Some(frame_number) {
        return None;
    }
    let boost_amount = accumulator.current_boost_amount?;
    let previous_boost_amount = accumulator.current_boost_before.unwrap_or(boost_amount);
    add_boost_state_sample(
        &mut accumulator.stats,
        previous_boost_amount,
        boost_amount,
        dt,
    );
    accumulator.previous_boost_amount = Some(boost_amount);
    Some((previous_boost_amount, boost_amount))
}

fn assert_boost_ledger_derived_stats_match(
    scope: &str,
    actual: &BoostStats,
    expected: &BoostStats,
) {
    type BoostFloatFieldAccessor = fn(&BoostStats) -> f32;

    let float_fields: [(&str, BoostFloatFieldAccessor); 21] = [
        ("tracked_time", |stats| stats.tracked_time),
        ("boost_integral", |stats| stats.boost_integral),
        ("time_zero_boost", |stats| stats.time_zero_boost),
        ("time_hundred_boost", |stats| stats.time_hundred_boost),
        ("time_boost_0_25", |stats| stats.time_boost_0_25),
        ("time_boost_25_50", |stats| stats.time_boost_25_50),
        ("time_boost_50_75", |stats| stats.time_boost_50_75),
        ("time_boost_75_100", |stats| stats.time_boost_75_100),
        ("amount_collected", |stats| stats.amount_collected),
        ("amount_collected_inactive", |stats| {
            stats.amount_collected_inactive
        }),
        ("amount_stolen", |stats| stats.amount_stolen),
        ("amount_collected_big", |stats| stats.amount_collected_big),
        ("amount_stolen_big", |stats| stats.amount_stolen_big),
        ("amount_collected_small", |stats| {
            stats.amount_collected_small
        }),
        ("amount_stolen_small", |stats| stats.amount_stolen_small),
        ("amount_respawned", |stats| stats.amount_respawned),
        ("overfill_total", |stats| stats.overfill_total),
        ("overfill_from_stolen", |stats| stats.overfill_from_stolen),
        ("amount_used", |stats| stats.amount_used),
        ("amount_used_while_grounded", |stats| {
            stats.amount_used_while_grounded
        }),
        ("amount_used_while_airborne", |stats| {
            stats.amount_used_while_airborne
        }),
    ];
    for (field, getter) in float_fields {
        let actual_value = getter(actual);
        let expected_value = getter(expected);
        assert!(
            (actual_value - expected_value).abs() < 0.001,
            "{scope} {field}: actual {actual_value:.3} != ledger-derived {expected_value:.3}",
        );
    }
    assert!(
        (actual.amount_used_while_supersonic - expected.amount_used_while_supersonic).abs() < 0.001,
        "{scope} amount_used_while_supersonic: actual {:.3} != ledger-derived {:.3}",
        actual.amount_used_while_supersonic,
        expected.amount_used_while_supersonic,
    );
    assert_eq!(
        actual.big_pads_collected, expected.big_pads_collected,
        "{scope} big_pads_collected"
    );
    assert_eq!(
        actual.small_pads_collected, expected.small_pads_collected,
        "{scope} small_pads_collected"
    );
    assert_eq!(
        actual.big_pads_stolen, expected.big_pads_stolen,
        "{scope} big_pads_stolen"
    );
    assert_eq!(
        actual.small_pads_stolen, expected.small_pads_stolen,
        "{scope} small_pads_stolen"
    );
    assert_eq!(
        actual.big_pads_collected_inactive, expected.big_pads_collected_inactive,
        "{scope} big_pads_collected_inactive"
    );
    assert_eq!(
        actual.small_pads_collected_inactive, expected.small_pads_collected_inactive,
        "{scope} small_pads_collected_inactive"
    );
}

#[derive(Clone, Default)]
struct DerivedQualityMechanicStats {
    count: u32,
    high_confidence_count: u32,
    last_time: Option<f32>,
    last_frame: Option<usize>,
    last_resolved_time: Option<f32>,
    last_resolved_frame: Option<usize>,
    last_quality: Option<f32>,
    best_quality: f32,
    cumulative_quality: f32,
}

impl DerivedQualityMechanicStats {
    fn record(
        &mut self,
        frame: usize,
        time: f32,
        resolved_frame: usize,
        resolved_time: f32,
        confidence: f32,
        high_confidence: bool,
    ) {
        self.count += 1;
        if high_confidence {
            self.high_confidence_count += 1;
        }
        self.last_time = Some(time);
        self.last_frame = Some(frame);
        self.last_resolved_time = Some(resolved_time);
        self.last_resolved_frame = Some(resolved_frame);
        self.last_quality = Some(confidence);
        self.best_quality = self.best_quality.max(confidence);
        self.cumulative_quality += confidence;
    }
}

#[derive(Clone, Default)]
struct DerivedQualityMechanicFrameStats {
    count: u32,
    high_confidence_count: u32,
    is_last_player: bool,
    last_time: Option<f32>,
    last_frame: Option<usize>,
    time_since_last: Option<f32>,
    frames_since_last: Option<usize>,
    last_quality: Option<f32>,
    best_quality: f32,
    cumulative_quality: f32,
}

impl DerivedQualityMechanicFrameStats {
    fn from_accumulator(
        accumulator: Option<&DerivedQualityMechanicStats>,
        frame: &ReplayStatsFrame,
        is_last_player: bool,
    ) -> Self {
        let Some(accumulator) = accumulator else {
            return Self::default();
        };
        let is_resolution_frame = accumulator.last_resolved_frame == Some(frame.frame_number);
        Self {
            count: accumulator.count,
            high_confidence_count: accumulator.high_confidence_count,
            is_last_player,
            last_time: accumulator.last_time,
            last_frame: accumulator.last_frame,
            time_since_last: if is_resolution_frame {
                Some(0.0)
            } else {
                accumulator
                    .last_time
                    .map(|time| (frame.time - time).max(0.0))
            },
            frames_since_last: if is_resolution_frame {
                Some(0)
            } else {
                accumulator
                    .last_frame
                    .map(|last_frame| frame.frame_number.saturating_sub(last_frame))
            },
            last_quality: accumulator.last_quality,
            best_quality: accumulator.best_quality,
            cumulative_quality: accumulator.cumulative_quality,
        }
    }
}

fn assert_half_flip_derived_stats_match(
    scope: &str,
    actual: &HalfFlipStats,
    expected: &DerivedQualityMechanicFrameStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} half_flip.count");
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} half_flip.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_half_flip, expected.is_last_player,
        "{scope} half_flip.is_last_half_flip"
    );
    assert_eq!(
        actual.last_half_flip_frame, expected.last_frame,
        "{scope} half_flip.last_half_flip_frame"
    );
    assert!(
        match (actual.last_half_flip_time, expected.last_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_flip.last_half_flip_time: actual {:?} expected {:?}",
        actual.last_half_flip_time,
        expected.last_time,
    );
    assert_eq!(
        actual.frames_since_last_half_flip, expected.frames_since_last,
        "{scope} half_flip.frames_since_last_half_flip",
    );
    assert!(
        match (actual.time_since_last_half_flip, expected.time_since_last) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_flip.time_since_last_half_flip: actual {:?} expected {:?}",
        actual.time_since_last_half_flip,
        expected.last_time,
    );
    assert!(
        match (actual.last_quality, expected.last_quality) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_flip.last_quality: actual {:?} expected {:?}",
        actual.last_quality,
        expected.last_quality,
    );
    assert!(
        (actual.best_quality - expected.best_quality).abs() < 0.001,
        "{scope} half_flip.best_quality: actual {:.3} expected {:.3}",
        actual.best_quality,
        expected.best_quality,
    );
    assert!(
        (actual.cumulative_quality - expected.cumulative_quality).abs() < 0.001,
        "{scope} half_flip.cumulative_quality: actual {:.3} expected {:.3}",
        actual.cumulative_quality,
        expected.cumulative_quality,
    );
}

fn assert_wavedash_derived_stats_match(
    scope: &str,
    actual: &WavedashStats,
    expected: &DerivedQualityMechanicFrameStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} wavedash.count");
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} wavedash.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_wavedash, expected.is_last_player,
        "{scope} wavedash.is_last_wavedash"
    );
    assert_eq!(
        actual.last_wavedash_frame, expected.last_frame,
        "{scope} wavedash.last_wavedash_frame"
    );
    assert!(
        match (actual.last_wavedash_time, expected.last_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wavedash.last_wavedash_time: actual {:?} expected {:?}",
        actual.last_wavedash_time,
        expected.last_time,
    );
    assert_eq!(
        actual.frames_since_last_wavedash, expected.frames_since_last,
        "{scope} wavedash.frames_since_last_wavedash",
    );
    assert!(
        match (actual.time_since_last_wavedash, expected.time_since_last) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wavedash.time_since_last_wavedash: actual {:?} expected {:?}",
        actual.time_since_last_wavedash,
        expected.last_time,
    );
    assert!(
        match (actual.last_quality, expected.last_quality) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wavedash.last_quality: actual {:?} expected {:?}",
        actual.last_quality,
        expected.last_quality,
    );
    assert!(
        (actual.best_quality - expected.best_quality).abs() < 0.001,
        "{scope} wavedash.best_quality: actual {:.3} expected {:.3}",
        actual.best_quality,
        expected.best_quality,
    );
    assert!(
        (actual.cumulative_quality - expected.cumulative_quality).abs() < 0.001,
        "{scope} wavedash.cumulative_quality: actual {:.3} expected {:.3}",
        actual.cumulative_quality,
        expected.cumulative_quality,
    );
}

fn assert_speed_flip_derived_stats_match(
    scope: &str,
    actual: &SpeedFlipStats,
    expected: &DerivedQualityMechanicFrameStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} speed_flip.count");
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} speed_flip.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_speed_flip, expected.is_last_player,
        "{scope} speed_flip.is_last_speed_flip"
    );
    assert_eq!(
        actual.last_speed_flip_frame, expected.last_frame,
        "{scope} speed_flip.last_speed_flip_frame"
    );
    assert!(
        match (actual.last_speed_flip_time, expected.last_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} speed_flip.last_speed_flip_time: actual {:?} expected {:?}",
        actual.last_speed_flip_time,
        expected.last_time,
    );
    assert_eq!(
        actual.frames_since_last_speed_flip, expected.frames_since_last,
        "{scope} speed_flip.frames_since_last_speed_flip",
    );
    assert!(
        match (actual.time_since_last_speed_flip, expected.time_since_last) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} speed_flip.time_since_last_speed_flip: actual {:?} expected {:?}",
        actual.time_since_last_speed_flip,
        expected.time_since_last,
    );
    assert!(
        match (actual.last_quality, expected.last_quality) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} speed_flip.last_quality: actual {:?} expected {:?}",
        actual.last_quality,
        expected.last_quality,
    );
    assert!(
        (actual.best_quality - expected.best_quality).abs() < 0.001,
        "{scope} speed_flip.best_quality: actual {:.3} expected {:.3}",
        actual.best_quality,
        expected.best_quality,
    );
    assert!(
        (actual.cumulative_quality - expected.cumulative_quality).abs() < 0.001,
        "{scope} speed_flip.cumulative_quality: actual {:.3} expected {:.3}",
        actual.cumulative_quality,
        expected.cumulative_quality,
    );
}
