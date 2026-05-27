use std::collections::{HashMap, HashSet};

use subtr_actor::*;

const TEST_BOOST_ZERO_BAND_RAW: f32 = 1.0;
const TEST_BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;
const REPLAY_FORMAT_EVOLUTION_DOC: &str = include_str!("../docs/replay-format-evolution.md");

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
        let violations = boost_invariant_violations(stats, None);
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

fn assert_quality_mechanic_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    const HALF_FLIP_HIGH_CONFIDENCE: f32 = 0.78;
    const WAVEDASH_HIGH_CONFIDENCE: f32 = 0.75;

    let mut half_flip_event_index = 0;
    let mut wavedash_event_index = 0;
    let mut half_flip_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut wavedash_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut half_flip_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut wavedash_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut last_half_flip_player: Option<PlayerId> = None;
    let mut last_wavedash_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            while half_flip_event_index < timeline.events.half_flip.len()
                && timeline.events.half_flip[half_flip_event_index].frame <= frame.frame_number
            {
                let event = &timeline.events.half_flip[half_flip_event_index];
                half_flip_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.frame,
                        event.time,
                        event.confidence,
                        event.confidence >= HALF_FLIP_HIGH_CONFIDENCE,
                    );
                last_half_flip_player = Some(event.player.clone());
                half_flip_event_index += 1;
            }

            while wavedash_event_index < timeline.events.wavedash.len()
                && timeline.events.wavedash[wavedash_event_index].frame <= frame.frame_number
            {
                let event = &timeline.events.wavedash[wavedash_event_index];
                wavedash_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.frame,
                        event.time,
                        event.confidence,
                        event.confidence >= WAVEDASH_HIGH_CONFIDENCE,
                    );
                last_wavedash_player = Some(event.player.clone());
                wavedash_event_index += 1;
            }

            for player in &frame.players {
                let half_flip_expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    half_flip_players.get(&player.player_id),
                    frame,
                    last_half_flip_player.as_ref() == Some(&player.player_id),
                );
                half_flip_frame_stats.insert(player.player_id.clone(), half_flip_expected.clone());
                assert_half_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.half_flip,
                    &half_flip_expected,
                );

                let wavedash_expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    wavedash_players.get(&player.player_id),
                    frame,
                    last_wavedash_player.as_ref() == Some(&player.player_id),
                );
                wavedash_frame_stats.insert(player.player_id.clone(), wavedash_expected.clone());
                assert_wavedash_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.wavedash,
                    &wavedash_expected,
                );
            }
        } else {
            for player in &frame.players {
                let half_flip_expected = half_flip_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_half_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.half_flip,
                    &half_flip_expected,
                );

                let wavedash_expected = wavedash_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_wavedash_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.wavedash,
                    &wavedash_expected,
                );
            }
            last_half_flip_player = None;
            last_wavedash_player = None;
        }
    }

    assert_eq!(
        half_flip_event_index,
        timeline.events.half_flip.len(),
        "{replay_path} unprocessed half-flip events"
    );
    assert_eq!(
        wavedash_event_index,
        timeline.events.wavedash.len(),
        "{replay_path} unprocessed wavedash events"
    );
}

fn assert_speed_flip_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    const SPEED_FLIP_HIGH_CONFIDENCE: f32 = 0.75;

    let mut speed_flip_event_index = 0;
    let mut speed_flip_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut speed_flip_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut last_speed_flip_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        let speed_flip_stats_advance = frame.is_live_play || frame.ball_has_been_hit == Some(false);
        if speed_flip_stats_advance {
            while speed_flip_event_index < timeline.events.speed_flip.len()
                && timeline.events.speed_flip[speed_flip_event_index].resolved_frame
                    <= frame.frame_number
            {
                let event = &timeline.events.speed_flip[speed_flip_event_index];
                speed_flip_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.resolved_frame,
                        event.resolved_time,
                        event.confidence,
                        event.confidence >= SPEED_FLIP_HIGH_CONFIDENCE,
                    );
                last_speed_flip_player = Some(event.player.clone());
                speed_flip_event_index += 1;
            }

            for player in &frame.players {
                let expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    speed_flip_players.get(&player.player_id),
                    frame,
                    last_speed_flip_player.as_ref() == Some(&player.player_id),
                );
                speed_flip_frame_stats.insert(player.player_id.clone(), expected.clone());
                assert_speed_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.speed_flip,
                    &expected,
                );
            }
        } else {
            for player in &frame.players {
                let expected = speed_flip_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_speed_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frozen frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.speed_flip,
                    &expected,
                );
            }
        }
    }

    assert_eq!(
        speed_flip_event_index,
        timeline.events.speed_flip.len(),
        "{replay_path} unprocessed speed-flip events"
    );
}

#[derive(Clone, Default)]
struct DerivedWhiffFrameStats {
    stats: WhiffStats,
}

impl DerivedWhiffFrameStats {
    fn record_event(&mut self, event: &WhiffEvent, frame: &ReplayStatsFrame) {
        match event.kind {
            WhiffEventKind::Whiff => {
                self.stats.whiff_count += 1;
                if event.aerial {
                    self.stats.aerial_whiff_count += 1;
                } else {
                    self.stats.grounded_whiff_count += 1;
                }
                if event.dodge_active {
                    self.stats.dodge_whiff_count += 1;
                }
                self.stats.last_whiff_time = Some(event.time);
                self.stats.last_whiff_frame = Some(event.frame);
                self.stats.time_since_last_whiff = Some((frame.time - event.time).max(0.0));
                self.stats.frames_since_last_whiff =
                    Some(frame.frame_number.saturating_sub(event.frame));
                self.stats.last_closest_approach_distance = Some(event.closest_approach_distance);
                self.stats.best_closest_approach_distance = Some(
                    self.stats
                        .best_closest_approach_distance
                        .map(|distance| distance.min(event.closest_approach_distance))
                        .unwrap_or(event.closest_approach_distance),
                );
                self.stats.cumulative_closest_approach_distance += event.closest_approach_distance;
                self.stats.is_last_whiff = true;
            }
            WhiffEventKind::BeatenToBall => {
                self.stats.beaten_to_ball_count += 1;
            }
        }
    }

    fn advance_live_frame(&mut self, frame: &ReplayStatsFrame, is_last_whiff_player: bool) {
        self.stats.is_last_whiff = is_last_whiff_player;
        self.stats.time_since_last_whiff = self
            .stats
            .last_whiff_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_whiff = self
            .stats
            .last_whiff_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }
}

fn assert_whiff_derived_stats_match(scope: &str, actual: &WhiffStats, expected: &WhiffStats) {
    assert_eq!(
        actual.whiff_count, expected.whiff_count,
        "{scope} whiff_count"
    );
    assert_eq!(
        actual.beaten_to_ball_count, expected.beaten_to_ball_count,
        "{scope} beaten_to_ball_count"
    );
    assert_eq!(
        actual.grounded_whiff_count, expected.grounded_whiff_count,
        "{scope} grounded_whiff_count"
    );
    assert_eq!(
        actual.aerial_whiff_count, expected.aerial_whiff_count,
        "{scope} aerial_whiff_count"
    );
    assert_eq!(
        actual.dodge_whiff_count, expected.dodge_whiff_count,
        "{scope} dodge_whiff_count"
    );
    assert_eq!(
        actual.is_last_whiff, expected.is_last_whiff,
        "{scope} is_last_whiff"
    );
    assert_eq!(
        actual.last_whiff_frame, expected.last_whiff_frame,
        "{scope} last_whiff_frame"
    );
    assert!(
        match (actual.last_whiff_time, expected.last_whiff_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} last_whiff_time: actual {:?} expected {:?}",
        actual.last_whiff_time,
        expected.last_whiff_time,
    );
    assert_eq!(
        actual.frames_since_last_whiff, expected.frames_since_last_whiff,
        "{scope} frames_since_last_whiff"
    );
    assert!(
        match (actual.time_since_last_whiff, expected.time_since_last_whiff) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} time_since_last_whiff: actual {:?} expected {:?}",
        actual.time_since_last_whiff,
        expected.time_since_last_whiff,
    );
    assert!(
        match (
            actual.last_closest_approach_distance,
            expected.last_closest_approach_distance,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} last_closest_approach_distance: actual {:?} expected {:?}",
        actual.last_closest_approach_distance,
        expected.last_closest_approach_distance,
    );
    assert!(
        match (
            actual.best_closest_approach_distance,
            expected.best_closest_approach_distance,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} best_closest_approach_distance: actual {:?} expected {:?}",
        actual.best_closest_approach_distance,
        expected.best_closest_approach_distance,
    );
    assert!(
        (actual.cumulative_closest_approach_distance
            - expected.cumulative_closest_approach_distance)
            .abs()
            < 0.001,
        "{scope} cumulative_closest_approach_distance: actual {:.3} expected {:.3}",
        actual.cumulative_closest_approach_distance,
        expected.cumulative_closest_approach_distance,
    );
}

fn assert_whiff_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.whiff.clone();
    events.sort_by(|left, right| {
        left.resolved_frame
            .cmp(&right.resolved_frame)
            .then_with(|| left.resolved_time.total_cmp(&right.resolved_time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedWhiffFrameStats> = HashMap::new();
    let mut frozen_players: HashMap<PlayerId, DerivedWhiffFrameStats> = HashMap::new();
    let mut last_whiff_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for player in players.values_mut() {
                player.advance_live_frame(frame, false);
            }

            while event_index < events.len()
                && events[event_index].resolved_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let player = players.entry(event.player.clone()).or_default();
                player.record_event(event, frame);
                if event.kind == WhiffEventKind::Whiff {
                    last_whiff_player = Some(event.player.clone());
                }
                event_index += 1;
            }

            if let Some(player_id) = last_whiff_player.as_ref() {
                if let Some(player) = players.get_mut(player_id) {
                    player.stats.is_last_whiff = true;
                }
            }

            for player in &frame.players {
                let expected = players.get(&player.player_id).cloned().unwrap_or_default();
                assert_whiff_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.whiff,
                    &expected.stats,
                );
                frozen_players.insert(player.player_id.clone(), expected);
            }
        } else {
            for player in &frame.players {
                let expected = frozen_players
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_whiff_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.whiff,
                    &expected.stats,
                );
            }
            last_whiff_player = None;
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed whiff events"
    );
}

#[derive(Clone, Default)]
struct DerivedBackboardPlayerStats {
    stats: BackboardPlayerStats,
}

impl DerivedBackboardPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_backboard_player: bool) {
        self.stats.is_last_backboard = is_last_backboard_player;
        self.stats.time_since_last_backboard = self
            .stats
            .last_backboard_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_backboard = self
            .stats
            .last_backboard_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &BackboardBounceEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.last_backboard_time = Some(event.time);
        self.stats.last_backboard_frame = Some(event.frame);
        self.stats.time_since_last_backboard = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_backboard =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_backboard_derived_player_stats_match(
    scope: &str,
    actual: &BackboardPlayerStats,
    expected: &BackboardPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} backboard.count");
    assert_eq!(
        actual.is_last_backboard, expected.is_last_backboard,
        "{scope} backboard.is_last_backboard"
    );
    assert_eq!(
        actual.last_backboard_frame, expected.last_backboard_frame,
        "{scope} backboard.last_backboard_frame"
    );
    assert!(
        match (actual.last_backboard_time, expected.last_backboard_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} backboard.last_backboard_time: actual {:?} expected {:?}",
        actual.last_backboard_time,
        expected.last_backboard_time,
    );
    assert_eq!(
        actual.frames_since_last_backboard, expected.frames_since_last_backboard,
        "{scope} backboard.frames_since_last_backboard"
    );
    assert!(
        match (
            actual.time_since_last_backboard,
            expected.time_since_last_backboard,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} backboard.time_since_last_backboard: actual {:?} expected {:?}",
        actual.time_since_last_backboard,
        expected.time_since_last_backboard,
    );
}

fn assert_backboard_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.backboard.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedBackboardPlayerStats> = HashMap::new();
    let mut team_zero = BackboardTeamStats::default();
    let mut team_one = BackboardTeamStats::default();
    let mut last_backboard_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(frame, last_backboard_player.as_ref() == Some(player_id));
        }

        let mut processed_event = false;
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.player.clone())
                .or_default()
                .record_event(event, frame);
            if event.is_team_0 {
                team_zero.count += 1;
            } else {
                team_one.count += 1;
            }
            last_backboard_player = Some(event.player.clone());
            event_index += 1;
            processed_event = true;
        }

        if processed_event {
            for stats in players.values_mut() {
                stats.stats.is_last_backboard = false;
            }
        }

        if let Some(player_id) = last_backboard_player.as_ref() {
            if let Some(stats) = players.get_mut(player_id) {
                stats.stats.is_last_backboard = true;
            }
        }

        assert_eq!(
            frame.team_zero.backboard, team_zero,
            "{replay_path} team_zero backboard frame {}",
            frame.frame_number
        );
        assert_eq!(
            frame.team_one.backboard, team_one,
            "{replay_path} team_one backboard frame {}",
            frame.frame_number
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_backboard_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.backboard,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed backboard events"
    );
}

#[derive(Clone, Default)]
struct DerivedDoubleTapPlayerStats {
    stats: DoubleTapPlayerStats,
}

impl DerivedDoubleTapPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_double_tap_player: bool) {
        self.stats.is_last_double_tap = is_last_double_tap_player;
        self.stats.time_since_last_double_tap = self
            .stats
            .last_double_tap_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_double_tap = self
            .stats
            .last_double_tap_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &DoubleTapEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.last_double_tap_time = Some(event.time);
        self.stats.last_double_tap_frame = Some(event.frame);
        self.stats.time_since_last_double_tap = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_double_tap =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_double_tap_derived_player_stats_match(
    scope: &str,
    actual: &DoubleTapPlayerStats,
    expected: &DoubleTapPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} double_tap.count");
    assert_eq!(
        actual.is_last_double_tap, expected.is_last_double_tap,
        "{scope} double_tap.is_last_double_tap"
    );
    assert_eq!(
        actual.last_double_tap_frame, expected.last_double_tap_frame,
        "{scope} double_tap.last_double_tap_frame"
    );
    assert!(
        match (actual.last_double_tap_time, expected.last_double_tap_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} double_tap.last_double_tap_time: actual {:?} expected {:?}",
        actual.last_double_tap_time,
        expected.last_double_tap_time,
    );
    assert_eq!(
        actual.frames_since_last_double_tap, expected.frames_since_last_double_tap,
        "{scope} double_tap.frames_since_last_double_tap"
    );
    assert!(
        match (
            actual.time_since_last_double_tap,
            expected.time_since_last_double_tap,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} double_tap.time_since_last_double_tap: actual {:?} expected {:?}",
        actual.time_since_last_double_tap,
        expected.time_since_last_double_tap,
    );
}

fn assert_double_tap_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.double_tap.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedDoubleTapPlayerStats> = HashMap::new();
    let mut team_zero = DoubleTapTeamStats::default();
    let mut team_one = DoubleTapTeamStats::default();
    let mut last_double_tap_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(frame, last_double_tap_player.as_ref() == Some(player_id));
        }

        let mut processed_event = false;
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.player.clone())
                .or_default()
                .record_event(event, frame);
            if event.is_team_0 {
                team_zero.count += 1;
            } else {
                team_one.count += 1;
            }
            last_double_tap_player = Some(event.player.clone());
            event_index += 1;
            processed_event = true;
        }

        if processed_event {
            for stats in players.values_mut() {
                stats.stats.is_last_double_tap = false;
            }
        }

        if let Some(player_id) = last_double_tap_player.as_ref() {
            if let Some(stats) = players.get_mut(player_id) {
                stats.stats.is_last_double_tap = true;
            }
        }

        assert_eq!(
            frame.team_zero.double_tap, team_zero,
            "{replay_path} team_zero double_tap frame {}",
            frame.frame_number
        );
        assert_eq!(
            frame.team_one.double_tap, team_one,
            "{replay_path} team_one double_tap frame {}",
            frame.frame_number
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_double_tap_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.double_tap,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed double-tap events"
    );
}

#[derive(Clone, Default)]
struct DerivedPassPlayerStats {
    stats: PassPlayerStats,
}

impl DerivedPassPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_completed_pass_player: bool) {
        self.stats.is_last_completed_pass = is_last_completed_pass_player;
        self.stats.time_since_last_completed_pass = self
            .stats
            .last_completed_pass_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_completed_pass = self
            .stats
            .last_completed_pass_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_completed_pass(&mut self, event: &PassEvent, frame: &ReplayStatsFrame) {
        self.stats.completed_pass_count += 1;
        self.stats.total_pass_distance += event.ball_travel_distance;
        self.stats.total_pass_advance += event.ball_advance_distance;
        self.stats.longest_pass_distance = self
            .stats
            .longest_pass_distance
            .max(event.ball_travel_distance);
        self.stats.last_completed_pass_time = Some(event.time);
        self.stats.last_completed_pass_frame = Some(event.frame);
        self.stats.time_since_last_completed_pass = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_completed_pass =
            Some(frame.frame_number.saturating_sub(event.frame));
    }

    fn record_received_pass(&mut self) {
        self.stats.received_pass_count += 1;
    }
}

fn assert_pass_derived_player_stats_match(
    scope: &str,
    actual: &PassPlayerStats,
    expected: &PassPlayerStats,
) {
    assert_eq!(
        actual.completed_pass_count, expected.completed_pass_count,
        "{scope} pass.completed_pass_count"
    );
    assert_eq!(
        actual.received_pass_count, expected.received_pass_count,
        "{scope} pass.received_pass_count"
    );
    assert!(
        (actual.total_pass_distance - expected.total_pass_distance).abs() < 0.001,
        "{scope} pass.total_pass_distance: actual {:.3} expected {:.3}",
        actual.total_pass_distance,
        expected.total_pass_distance,
    );
    assert!(
        (actual.total_pass_advance - expected.total_pass_advance).abs() < 0.001,
        "{scope} pass.total_pass_advance: actual {:.3} expected {:.3}",
        actual.total_pass_advance,
        expected.total_pass_advance,
    );
    assert!(
        (actual.longest_pass_distance - expected.longest_pass_distance).abs() < 0.001,
        "{scope} pass.longest_pass_distance: actual {:.3} expected {:.3}",
        actual.longest_pass_distance,
        expected.longest_pass_distance,
    );
    assert_eq!(
        actual.is_last_completed_pass, expected.is_last_completed_pass,
        "{scope} pass.is_last_completed_pass"
    );
    assert_eq!(
        actual.last_completed_pass_frame, expected.last_completed_pass_frame,
        "{scope} pass.last_completed_pass_frame"
    );
    assert!(
        match (
            actual.last_completed_pass_time,
            expected.last_completed_pass_time,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} pass.last_completed_pass_time: actual {:?} expected {:?}",
        actual.last_completed_pass_time,
        expected.last_completed_pass_time,
    );
    assert_eq!(
        actual.frames_since_last_completed_pass, expected.frames_since_last_completed_pass,
        "{scope} pass.frames_since_last_completed_pass"
    );
    assert!(
        match (
            actual.time_since_last_completed_pass,
            expected.time_since_last_completed_pass,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} pass.time_since_last_completed_pass: actual {:?} expected {:?}",
        actual.time_since_last_completed_pass,
        expected.time_since_last_completed_pass,
    );
}

fn assert_pass_team_stats_match(scope: &str, actual: &PassTeamStats, expected: &PassTeamStats) {
    assert_eq!(
        actual.completed_pass_count, expected.completed_pass_count,
        "{scope} pass.completed_pass_count"
    );
    assert!(
        (actual.total_pass_distance - expected.total_pass_distance).abs() < 0.001,
        "{scope} pass.total_pass_distance: actual {:.3} expected {:.3}",
        actual.total_pass_distance,
        expected.total_pass_distance,
    );
    assert!(
        (actual.total_pass_advance - expected.total_pass_advance).abs() < 0.001,
        "{scope} pass.total_pass_advance: actual {:.3} expected {:.3}",
        actual.total_pass_advance,
        expected.total_pass_advance,
    );
    assert!(
        (actual.longest_pass_distance - expected.longest_pass_distance).abs() < 0.001,
        "{scope} pass.longest_pass_distance: actual {:.3} expected {:.3}",
        actual.longest_pass_distance,
        expected.longest_pass_distance,
    );
}

fn assert_pass_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.pass.clone();
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
    });
    let mut last_completed_events = timeline.events.pass_last_completed.clone();
    last_completed_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let has_last_completed_events = !last_completed_events.is_empty();

    let mut event_index = 0;
    let mut last_completed_event_index = 0;
    let mut players: HashMap<PlayerId, DerivedPassPlayerStats> = HashMap::new();
    let mut team_zero = PassTeamStats::default();
    let mut team_one = PassTeamStats::default();
    let mut last_completed_pass_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(
                frame,
                frame.is_live_play && last_completed_pass_player.as_ref() == Some(player_id),
            );
        }

        if !frame.is_live_play {
            last_completed_pass_player = None;
        } else {
            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                players
                    .entry(event.passer.clone())
                    .or_default()
                    .record_completed_pass(event, frame);
                players
                    .entry(event.receiver.clone())
                    .or_default()
                    .record_received_pass();

                let team_stats = if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                };
                team_stats.completed_pass_count += 1;
                team_stats.total_pass_distance += event.ball_travel_distance;
                team_stats.total_pass_advance += event.ball_advance_distance;
                team_stats.longest_pass_distance = team_stats
                    .longest_pass_distance
                    .max(event.ball_travel_distance);

                last_completed_pass_player = Some(event.passer.clone());
                event_index += 1;
                processed_event = true;
            }

            if !has_last_completed_events && processed_event {
                for stats in players.values_mut() {
                    stats.stats.is_last_completed_pass = false;
                }
            }

            if !has_last_completed_events {
                if let Some(player_id) = last_completed_pass_player.as_ref() {
                    if let Some(stats) = players.get_mut(player_id) {
                        stats.stats.is_last_completed_pass = true;
                    }
                }
            }
        }

        let mut processed_last_completed_event = false;
        while last_completed_event_index < last_completed_events.len()
            && last_completed_events[last_completed_event_index].frame <= frame.frame_number
        {
            last_completed_pass_player = last_completed_events[last_completed_event_index]
                .player
                .clone();
            last_completed_event_index += 1;
            processed_last_completed_event = true;
        }
        if processed_last_completed_event {
            for stats in players.values_mut() {
                stats.stats.is_last_completed_pass = false;
            }
            if let Some(player_id) = last_completed_pass_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.stats.is_last_completed_pass = true;
                }
            }
        }

        assert_pass_team_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.pass,
            &team_zero,
        );
        assert_pass_team_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.pass,
            &team_one,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_pass_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.pass,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed pass events"
    );
    assert_eq!(
        last_completed_event_index,
        last_completed_events.len(),
        "{replay_path} unprocessed pass-last-completed events"
    );
}

fn apply_rush_event(stats: &mut RushTeamStats, event: &RushEvent) {
    stats.count += 1;
    match (event.attackers, event.defenders) {
        (2, 1) => stats.two_v_one_count += 1,
        (2, 2) => stats.two_v_two_count += 1,
        (2, 3) => stats.two_v_three_count += 1,
        (3, 1) => stats.three_v_one_count += 1,
        (3, 2) => stats.three_v_two_count += 1,
        (3, 3) => stats.three_v_three_count += 1,
        _ => {}
    }
}

fn assert_rush_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.rush.clone();
    events.sort_by(|left, right| {
        left.start_frame
            .cmp(&right.start_frame)
            .then_with(|| left.start_time.total_cmp(&right.start_time))
            .then_with(|| left.end_frame.cmp(&right.end_frame))
    });

    let mut event_index = 0;
    let mut team_zero = RushTeamStats::default();
    let mut team_one = RushTeamStats::default();
    let min_retained_seconds = timeline.config.rush_min_possession_retained_seconds;

    for frame in &timeline.frames {
        while event_index < events.len()
            && frame.frame_number >= events[event_index].start_frame
            && frame.time - events[event_index].start_time >= min_retained_seconds
        {
            let event = &events[event_index];
            apply_rush_event(
                if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.rush, team_zero,
            "{replay_path} team_zero rush frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.rush, team_one,
            "{replay_path} team_one rush frame {}",
            frame.frame_number,
        );
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed rush events"
    );
}

#[derive(Clone, Default)]
struct DerivedBumpPlayerStats {
    stats: BumpPlayerStats,
}

impl DerivedBumpPlayerStats {
    fn record_inflicted(&mut self, event: &BumpEvent) {
        self.stats.bumps_inflicted += 1;
        if event.is_team_bump {
            self.stats.team_bumps_inflicted += 1;
        }
        self.stats.last_bump_time = Some(event.time);
        self.stats.last_bump_frame = Some(event.frame);
        self.stats.last_bump_strength = Some(event.strength);
        self.stats.max_bump_strength = self.stats.max_bump_strength.max(event.strength);
        self.stats.cumulative_bump_strength += event.strength;
    }

    fn record_taken(&mut self, event: &BumpEvent) {
        self.stats.bumps_taken += 1;
        if event.is_team_bump {
            self.stats.team_bumps_taken += 1;
        }
    }
}

fn apply_bump_team_event(stats: &mut BumpTeamStats, event: &BumpEvent) {
    stats.bumps_inflicted += 1;
    if event.is_team_bump {
        stats.team_bumps_inflicted += 1;
    }
}

fn assert_bump_player_stats_match(
    scope: &str,
    actual: &BumpPlayerStats,
    expected: &BumpPlayerStats,
) {
    assert_eq!(
        actual.bumps_inflicted, expected.bumps_inflicted,
        "{scope} bump.bumps_inflicted"
    );
    assert_eq!(
        actual.bumps_taken, expected.bumps_taken,
        "{scope} bump.bumps_taken"
    );
    assert_eq!(
        actual.team_bumps_inflicted, expected.team_bumps_inflicted,
        "{scope} bump.team_bumps_inflicted"
    );
    assert_eq!(
        actual.team_bumps_taken, expected.team_bumps_taken,
        "{scope} bump.team_bumps_taken"
    );
    assert_eq!(
        actual.last_bump_frame, expected.last_bump_frame,
        "{scope} bump.last_bump_frame"
    );
    assert!(
        match (actual.last_bump_time, expected.last_bump_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} bump.last_bump_time: actual {:?} expected {:?}",
        actual.last_bump_time,
        expected.last_bump_time,
    );
    assert!(
        match (actual.last_bump_strength, expected.last_bump_strength) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} bump.last_bump_strength: actual {:?} expected {:?}",
        actual.last_bump_strength,
        expected.last_bump_strength,
    );
    assert!(
        (actual.max_bump_strength - expected.max_bump_strength).abs() < 0.001,
        "{scope} bump.max_bump_strength: actual {:.3} expected {:.3}",
        actual.max_bump_strength,
        expected.max_bump_strength,
    );
    assert!(
        (actual.cumulative_bump_strength - expected.cumulative_bump_strength).abs() < 0.001,
        "{scope} bump.cumulative_bump_strength: actual {:.3} expected {:.3}",
        actual.cumulative_bump_strength,
        expected.cumulative_bump_strength,
    );
}

fn assert_bump_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.bump.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedBumpPlayerStats> = HashMap::new();
    let mut team_zero = BumpTeamStats::default();
    let mut team_one = BumpTeamStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.initiator.clone())
                .or_default()
                .record_inflicted(event);
            players
                .entry(event.victim.clone())
                .or_default()
                .record_taken(event);
            apply_bump_team_event(
                if event.initiator_is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.bump, team_zero,
            "{replay_path} team_zero bump frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.bump, team_one,
            "{replay_path} team_one bump frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_bump_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.bump,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed bump events"
    );
}

fn assert_demo_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events: Vec<_> = timeline
        .events
        .timeline
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                TimelineEventKind::Kill | TimelineEventKind::Death
            )
        })
        .cloned()
        .collect();
    events.sort_by(|left, right| left.time.total_cmp(&right.time));

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DemoPlayerStats> = HashMap::new();
    let mut team_zero = DemoTeamStats::default();
    let mut team_one = DemoTeamStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].time <= frame.time {
            let event = &events[event_index];
            if let Some(player_id) = event.player_id.as_ref() {
                match event.kind {
                    TimelineEventKind::Kill => {
                        players
                            .entry(player_id.clone())
                            .or_default()
                            .demos_inflicted += 1;
                        match event.is_team_0 {
                            Some(true) => team_zero.demos_inflicted += 1,
                            Some(false) => team_one.demos_inflicted += 1,
                            None => {}
                        }
                    }
                    TimelineEventKind::Death => {
                        players.entry(player_id.clone()).or_default().demos_taken += 1;
                    }
                    _ => {}
                }
            }
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.demo, team_zero,
            "{replay_path} team_zero demo frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.demo, team_one,
            "{replay_path} team_one demo frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.demo, expected,
                "{replay_path} player {} demo frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed demo events"
    );
}

fn ball_carry_kind_label_for_derivation(kind: BallCarryKind) -> StatLabel {
    match kind {
        BallCarryKind::Carry => StatLabel::new("kind", "carry"),
        BallCarryKind::AirDribble => StatLabel::new("kind", "air_dribble"),
    }
}

fn air_dribble_origin_label_for_derivation(origin: AirDribbleOrigin) -> StatLabel {
    StatLabel::new("origin", origin.as_label_value())
}

fn apply_ball_carry_event(stats: &mut BallCarryStats, event: &BallCarryEvent) {
    stats
        .labeled_event_counts
        .increment([ball_carry_kind_label_for_derivation(event.kind)]);
    stats.carry_count = stats.labeled_event_counts.total();
    stats.total_carry_time += event.duration;
    stats.total_straight_line_distance += event.straight_line_distance;
    stats.total_path_distance += event.path_distance;
    stats.longest_carry_time = stats.longest_carry_time.max(event.duration);
    stats.furthest_carry_distance = stats
        .furthest_carry_distance
        .max(event.straight_line_distance);
    stats.fastest_carry_speed = stats.fastest_carry_speed.max(event.average_speed);
    stats.carry_speed_sum += event.average_speed;
    stats.average_horizontal_gap_sum += event.average_horizontal_gap;
    stats.average_vertical_gap_sum += event.average_vertical_gap;
}

fn apply_air_dribble_event(stats: &mut AirDribbleStats, event: &BallCarryEvent) {
    if let Some(origin) = event.air_dribble_origin {
        stats
            .labeled_event_counts
            .increment([air_dribble_origin_label_for_derivation(origin)]);
        match origin {
            AirDribbleOrigin::GroundToAir => stats.ground_to_air_count += 1,
            AirDribbleOrigin::WallToAir => stats.wall_to_air_count += 1,
        }
    }
    stats.count = stats.labeled_event_counts.total();
    stats.total_time += event.duration;
    stats.total_straight_line_distance += event.straight_line_distance;
    stats.total_path_distance += event.path_distance;
    stats.longest_time = stats.longest_time.max(event.duration);
    stats.furthest_distance = stats.furthest_distance.max(event.straight_line_distance);
    stats.fastest_speed = stats.fastest_speed.max(event.average_speed);
    stats.speed_sum += event.average_speed;
    stats.average_horizontal_gap_sum += event.average_horizontal_gap;
    stats.average_vertical_gap_sum += event.average_vertical_gap;
    stats.total_touch_count += event.touch_count;
    stats.max_touch_count = stats.max_touch_count.max(event.touch_count);
}

fn assert_ball_carry_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.ball_carry.clone();
    events.sort_by(|left, right| {
        left.end_frame
            .cmp(&right.end_frame)
            .then_with(|| left.end_time.total_cmp(&right.end_time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, BallCarryStats> = HashMap::new();
    let mut player_air_dribbles: HashMap<PlayerId, AirDribbleStats> = HashMap::new();
    let mut team_zero = BallCarryStats::default();
    let mut team_one = BallCarryStats::default();
    let mut team_zero_air_dribble = AirDribbleStats::default();
    let mut team_one_air_dribble = AirDribbleStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].end_frame < frame.frame_number {
            let event = &events[event_index];
            match event.kind {
                BallCarryKind::Carry => {
                    apply_ball_carry_event(
                        players.entry(event.player_id.clone()).or_default(),
                        event,
                    );
                    apply_ball_carry_event(
                        if event.is_team_0 {
                            &mut team_zero
                        } else {
                            &mut team_one
                        },
                        event,
                    );
                }
                BallCarryKind::AirDribble => {
                    apply_air_dribble_event(
                        player_air_dribbles
                            .entry(event.player_id.clone())
                            .or_default(),
                        event,
                    );
                    apply_air_dribble_event(
                        if event.is_team_0 {
                            &mut team_zero_air_dribble
                        } else {
                            &mut team_one_air_dribble
                        },
                        event,
                    );
                }
            }
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.ball_carry, team_zero,
            "{replay_path} team_zero ball_carry frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.ball_carry, team_one,
            "{replay_path} team_one ball_carry frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_zero.air_dribble, team_zero_air_dribble,
            "{replay_path} team_zero air_dribble frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.air_dribble, team_one_air_dribble,
            "{replay_path} team_one air_dribble frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            assert_eq!(
                player.ball_carry,
                players.get(&player.player_id).cloned().unwrap_or_default(),
                "{replay_path} player {} ball_carry frame {}",
                player.name,
                frame.frame_number,
            );
            assert_eq!(
                player.air_dribble,
                player_air_dribbles
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default(),
                "{replay_path} player {} air_dribble frame {}",
                player.name,
                frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed ball-carry events"
    );
}

fn apply_wall_aerial_event(
    stats: &mut WallAerialStats,
    event: &WallAerialEvent,
    frame: &ReplayStatsFrame,
) {
    const WALL_AERIAL_HIGH_CONFIDENCE: f32 = 0.78;

    stats.count += 1;
    if event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE {
        stats.high_confidence_count += 1;
    }
    stats.is_last_wall_aerial = true;
    stats.last_wall_aerial_time = Some(event.time);
    stats.last_wall_aerial_frame = Some(event.frame);
    stats.time_since_last_wall_aerial = Some((frame.time - event.time).max(0.0));
    stats.frames_since_last_wall_aerial = Some(frame.frame_number.saturating_sub(event.frame));
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
    stats.cumulative_setup_duration += event.setup_duration;
    stats.cumulative_takeoff_to_touch_time += event.time_since_takeoff;
    stats.cumulative_touch_height += event.player_position[2];
}

fn advance_wall_aerial_stats(
    stats: &mut WallAerialStats,
    frame: &ReplayStatsFrame,
    is_last_wall_aerial_player: bool,
) {
    stats.is_last_wall_aerial = is_last_wall_aerial_player;
    stats.time_since_last_wall_aerial = stats
        .last_wall_aerial_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_wall_aerial = stats
        .last_wall_aerial_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_wall_aerial_stats_match(
    scope: &str,
    actual: &WallAerialStats,
    expected: &WallAerialStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} wall_aerial.count");
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} wall_aerial.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_wall_aerial, expected.is_last_wall_aerial,
        "{scope} wall_aerial.is_last_wall_aerial"
    );
    assert_eq!(
        actual.last_wall_aerial_frame, expected.last_wall_aerial_frame,
        "{scope} wall_aerial.last_wall_aerial_frame"
    );
    assert!(
        match (actual.last_wall_aerial_time, expected.last_wall_aerial_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial.last_wall_aerial_time: actual {:?} expected {:?}",
        actual.last_wall_aerial_time,
        expected.last_wall_aerial_time
    );
    assert_eq!(
        actual.frames_since_last_wall_aerial, expected.frames_since_last_wall_aerial,
        "{scope} wall_aerial.frames_since_last_wall_aerial"
    );
    assert!(
        match (
            actual.time_since_last_wall_aerial,
            expected.time_since_last_wall_aerial,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial.time_since_last_wall_aerial: actual {:?} expected {:?}",
        actual.time_since_last_wall_aerial,
        expected.time_since_last_wall_aerial
    );
    assert!(
        match (actual.last_confidence, expected.last_confidence) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial.last_confidence: actual {:?} expected {:?}",
        actual.last_confidence,
        expected.last_confidence
    );
    assert!(
        (actual.best_confidence - expected.best_confidence).abs() < 0.001,
        "{scope} wall_aerial.best_confidence: actual {:.3} expected {:.3}",
        actual.best_confidence,
        expected.best_confidence
    );
    assert!(
        (actual.cumulative_confidence - expected.cumulative_confidence).abs() < 0.001,
        "{scope} wall_aerial.cumulative_confidence: actual {:.3} expected {:.3}",
        actual.cumulative_confidence,
        expected.cumulative_confidence
    );
    assert!(
        (actual.cumulative_setup_duration - expected.cumulative_setup_duration).abs() < 0.001,
        "{scope} wall_aerial.cumulative_setup_duration: actual {:.3} expected {:.3}",
        actual.cumulative_setup_duration,
        expected.cumulative_setup_duration
    );
    assert!(
        (actual.cumulative_takeoff_to_touch_time - expected.cumulative_takeoff_to_touch_time).abs()
            < 0.001,
        "{scope} wall_aerial.cumulative_takeoff_to_touch_time: actual {:.3} expected {:.3}",
        actual.cumulative_takeoff_to_touch_time,
        expected.cumulative_takeoff_to_touch_time
    );
    assert!(
        (actual.cumulative_touch_height - expected.cumulative_touch_height).abs() < 0.001,
        "{scope} wall_aerial.cumulative_touch_height: actual {:.3} expected {:.3}",
        actual.cumulative_touch_height,
        expected.cumulative_touch_height
    );
}

fn assert_wall_aerial_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.wall_aerial.clone();
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, WallAerialStats> = HashMap::new();
    let mut last_wall_aerial_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            advance_wall_aerial_stats(
                stats,
                frame,
                frame.is_live_play && last_wall_aerial_player.as_ref() == Some(player_id),
            );
        }

        if frame.is_live_play {
            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_wall_aerial_event(stats, event, frame);
                last_wall_aerial_player = Some(event.player.clone());
                processed_event = true;
                event_index += 1;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.is_last_wall_aerial = false;
                }
            }
            if let Some(player_id) = last_wall_aerial_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_wall_aerial = true;
                }
            }
        } else {
            last_wall_aerial_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_wall_aerial_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.wall_aerial,
                &expected,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed wall-aerial events"
    );
}

fn apply_wall_aerial_shot_event(stats: &mut WallAerialShotStats, event: &WallAerialShotEvent) {
    const WALL_AERIAL_HIGH_CONFIDENCE: f32 = 0.78;

    stats.count += 1;
    if event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE {
        stats.high_confidence_count += 1;
    }
    stats.is_last_wall_aerial_shot = true;
    stats.last_wall_aerial_shot_time = Some(event.time);
    stats.last_wall_aerial_shot_frame = Some(event.frame);
    stats.time_since_last_wall_aerial_shot = Some(0.0);
    stats.frames_since_last_wall_aerial_shot = Some(0);
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
    stats.cumulative_takeoff_to_shot_time += event.time_since_takeoff;
    stats.cumulative_shot_height += event.player_position[2];
}

fn advance_wall_aerial_shot_stats(
    stats: &mut WallAerialShotStats,
    frame: &ReplayStatsFrame,
    is_last_wall_aerial_shot_player: bool,
) {
    stats.is_last_wall_aerial_shot = is_last_wall_aerial_shot_player;
    stats.time_since_last_wall_aerial_shot = stats
        .last_wall_aerial_shot_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_wall_aerial_shot = stats
        .last_wall_aerial_shot_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_wall_aerial_shot_stats_match(
    scope: &str,
    actual: &WallAerialShotStats,
    expected: &WallAerialShotStats,
) {
    assert_eq!(
        actual.count, expected.count,
        "{scope} wall_aerial_shot.count"
    );
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} wall_aerial_shot.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_wall_aerial_shot, expected.is_last_wall_aerial_shot,
        "{scope} wall_aerial_shot.is_last_wall_aerial_shot"
    );
    assert_eq!(
        actual.last_wall_aerial_shot_frame, expected.last_wall_aerial_shot_frame,
        "{scope} wall_aerial_shot.last_wall_aerial_shot_frame"
    );
    assert!(
        match (
            actual.last_wall_aerial_shot_time,
            expected.last_wall_aerial_shot_time,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial_shot.last_wall_aerial_shot_time: actual {:?} expected {:?}",
        actual.last_wall_aerial_shot_time,
        expected.last_wall_aerial_shot_time
    );
    assert_eq!(
        actual.frames_since_last_wall_aerial_shot, expected.frames_since_last_wall_aerial_shot,
        "{scope} wall_aerial_shot.frames_since_last_wall_aerial_shot"
    );
    assert!(
        match (
            actual.time_since_last_wall_aerial_shot,
            expected.time_since_last_wall_aerial_shot,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial_shot.time_since_last_wall_aerial_shot: actual {:?} expected {:?}",
        actual.time_since_last_wall_aerial_shot,
        expected.time_since_last_wall_aerial_shot
    );
    assert!(
        match (actual.last_confidence, expected.last_confidence) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial_shot.last_confidence: actual {:?} expected {:?}",
        actual.last_confidence,
        expected.last_confidence
    );
    assert!(
        (actual.best_confidence - expected.best_confidence).abs() < 0.001,
        "{scope} wall_aerial_shot.best_confidence: actual {:.3} expected {:.3}",
        actual.best_confidence,
        expected.best_confidence
    );
    assert!(
        (actual.cumulative_confidence - expected.cumulative_confidence).abs() < 0.001,
        "{scope} wall_aerial_shot.cumulative_confidence: actual {:.3} expected {:.3}",
        actual.cumulative_confidence,
        expected.cumulative_confidence
    );
    assert!(
        (actual.cumulative_takeoff_to_shot_time - expected.cumulative_takeoff_to_shot_time).abs()
            < 0.001,
        "{scope} wall_aerial_shot.cumulative_takeoff_to_shot_time: actual {:.3} expected {:.3}",
        actual.cumulative_takeoff_to_shot_time,
        expected.cumulative_takeoff_to_shot_time
    );
    assert!(
        (actual.cumulative_shot_height - expected.cumulative_shot_height).abs() < 0.001,
        "{scope} wall_aerial_shot.cumulative_shot_height: actual {:.3} expected {:.3}",
        actual.cumulative_shot_height,
        expected.cumulative_shot_height
    );
}

fn assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.wall_aerial_shot.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, WallAerialShotStats> = HashMap::new();
    let mut last_wall_aerial_shot_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            advance_wall_aerial_shot_stats(
                stats,
                frame,
                frame.is_live_play && last_wall_aerial_shot_player.as_ref() == Some(player_id),
            );
        }

        if frame.is_live_play {
            let mut processed_event = false;
            while event_index < events.len() && events[event_index].frame <= frame.frame_number {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_wall_aerial_shot_event(stats, event);
                last_wall_aerial_shot_player = Some(event.player.clone());
                processed_event = true;
                event_index += 1;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.is_last_wall_aerial_shot = false;
                }
            }
            if let Some(player_id) = last_wall_aerial_shot_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_wall_aerial_shot = true;
                }
            }
        } else {
            last_wall_aerial_shot_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_wall_aerial_shot_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.wall_aerial_shot,
                &expected,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed wall-aerial-shot events"
    );
}

fn apply_ceiling_shot_event(stats: &mut CeilingShotStats, event: &CeilingShotEvent) {
    const CEILING_SHOT_HIGH_CONFIDENCE: f32 = 0.78;

    stats
        .labeled_event_counts
        .increment([confidence_band_label_for_derivation(
            event.confidence >= CEILING_SHOT_HIGH_CONFIDENCE,
        )]);
    stats.count = stats.labeled_event_counts.total();
    stats.high_confidence_count = stats
        .labeled_event_counts
        .count_matching(&[confidence_band_label_for_derivation(true)]);
    stats.is_last_ceiling_shot = true;
    stats.last_ceiling_shot_time = Some(event.time);
    stats.last_ceiling_shot_frame = Some(event.frame);
    stats.time_since_last_ceiling_shot = Some(0.0);
    stats.frames_since_last_ceiling_shot = Some(0);
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
}

fn advance_ceiling_shot_stats(
    stats: &mut CeilingShotStats,
    frame: &ReplayStatsFrame,
    is_last_ceiling_shot_player: bool,
) {
    stats.is_last_ceiling_shot = is_last_ceiling_shot_player;
    stats.time_since_last_ceiling_shot = stats
        .last_ceiling_shot_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_ceiling_shot = stats
        .last_ceiling_shot_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_ceiling_shot_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.ceiling_shot.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, CeilingShotStats> = HashMap::new();
    let mut last_ceiling_shot_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for (player_id, stats) in players.iter_mut() {
                advance_ceiling_shot_stats(
                    stats,
                    frame,
                    last_ceiling_shot_player.as_ref() == Some(player_id),
                );
            }

            while event_index < events.len() && events[event_index].frame <= frame.frame_number {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_ceiling_shot_event(stats, event);
                last_ceiling_shot_player = Some(event.player.clone());
                event_index += 1;
            }
        } else {
            last_ceiling_shot_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.ceiling_shot, expected,
                "{replay_path} player {} ceiling_shot frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed ceiling-shot events"
    );
}

fn confidence_band_label_for_derivation(high_confidence: bool) -> StatLabel {
    if high_confidence {
        StatLabel::new("confidence_band", "high")
    } else {
        StatLabel::new("confidence_band", "standard")
    }
}

fn vertical_state_label_for_derivation(aerial: bool) -> StatLabel {
    if aerial {
        StatLabel::new("vertical_state", "aerial")
    } else {
        StatLabel::new("vertical_state", "grounded")
    }
}

fn apply_flick_event(stats: &mut FlickStats, event: &FlickEvent) {
    const FLICK_HIGH_CONFIDENCE: f32 = 0.80;

    stats
        .labeled_event_counts
        .increment([confidence_band_label_for_derivation(
            event.confidence >= FLICK_HIGH_CONFIDENCE,
        )]);
    stats.count = stats.labeled_event_counts.total();
    stats.high_confidence_count = stats
        .labeled_event_counts
        .count_matching(&[confidence_band_label_for_derivation(true)]);
    stats.is_last_flick = true;
    stats.last_flick_time = Some(event.time);
    stats.last_flick_frame = Some(event.frame);
    stats.time_since_last_flick = Some(0.0);
    stats.frames_since_last_flick = Some(0);
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
    stats.cumulative_setup_duration += event.setup_duration;
    stats.cumulative_ball_speed_change += event.ball_speed_change;
}

fn advance_flick_stats(
    stats: &mut FlickStats,
    frame: &ReplayStatsFrame,
    is_last_flick_player: bool,
) {
    stats.is_last_flick = is_last_flick_player;
    stats.time_since_last_flick = stats
        .last_flick_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_flick = stats
        .last_flick_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_flick_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.flick.clone();
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, FlickStats> = HashMap::new();
    let mut last_flick_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for (player_id, stats) in players.iter_mut() {
                advance_flick_stats(stats, frame, last_flick_player.as_ref() == Some(player_id));
            }

            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_flick_event(stats, event);
                last_flick_player = Some(event.player.clone());
                event_index += 1;
            }

            if let Some(player_id) = last_flick_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_flick = true;
                }
            }
        } else {
            last_flick_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.flick, expected,
                "{replay_path} player {} flick frame {} live={} phase={:?}",
                player.name, frame.frame_number, frame.is_live_play, frame.gameplay_phase,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed flick events"
    );
}

fn apply_musty_flick_event(stats: &mut MustyFlickStats, event: &MustyFlickEvent) {
    const MUSTY_HIGH_CONFIDENCE: f32 = 0.80;

    stats.labeled_event_counts.increment([
        vertical_state_label_for_derivation(event.aerial),
        confidence_band_label_for_derivation(event.confidence >= MUSTY_HIGH_CONFIDENCE),
    ]);
    stats.count = stats.labeled_event_counts.total();
    stats.aerial_count = stats
        .labeled_event_counts
        .count_matching(&[vertical_state_label_for_derivation(true)]);
    stats.high_confidence_count = stats
        .labeled_event_counts
        .count_matching(&[confidence_band_label_for_derivation(true)]);
    stats.is_last_musty = true;
    stats.last_musty_time = Some(event.time);
    stats.last_musty_frame = Some(event.frame);
    stats.time_since_last_musty = Some(0.0);
    stats.frames_since_last_musty = Some(0);
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
}

fn advance_musty_flick_stats(
    stats: &mut MustyFlickStats,
    frame: &ReplayStatsFrame,
    is_last_musty_player: bool,
) {
    stats.is_last_musty = is_last_musty_player;
    stats.time_since_last_musty = stats
        .last_musty_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_musty = stats
        .last_musty_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_musty_flick_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.musty_flick.clone();
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, MustyFlickStats> = HashMap::new();
    let mut last_musty_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for (player_id, stats) in players.iter_mut() {
                advance_musty_flick_stats(
                    stats,
                    frame,
                    last_musty_player.as_ref() == Some(player_id),
                );
            }

            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_musty_flick_event(stats, event);
                last_musty_player = Some(event.player.clone());
                event_index += 1;
                processed_event = true;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.is_last_musty = false;
                }
            }

            if let Some(player_id) = last_musty_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_musty = true;
                }
            }
        } else {
            last_musty_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.musty_flick, expected,
                "{replay_path} player {} musty_flick frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed musty-flick events"
    );
}

fn assert_dodge_reset_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.dodge_reset.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DodgeResetStats> = HashMap::new();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            let stats = players.entry(event.player.clone()).or_default();
            stats.count += 1;
            if event.on_ball {
                stats.on_ball_count += 1;
            }
            event_index += 1;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.dodge_reset, expected,
                "{replay_path} player {} dodge_reset frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed dodge-reset events"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DerivedPowerslideState {
    active: bool,
    is_team_0: bool,
}

fn powerslide_frame_counts_toward_motion(frame: &ReplayStatsFrame) -> bool {
    matches!(
        frame.gameplay_phase,
        GameplayPhase::ActivePlay | GameplayPhase::KickoffWaitingForTouch
    )
}

fn assert_powerslide_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.powerslide.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut active_states: HashMap<PlayerId, DerivedPowerslideState> = HashMap::new();
    let mut players: HashMap<PlayerId, PowerslideStats> = HashMap::new();
    let mut team_zero = PowerslideStats::default();
    let mut team_one = PowerslideStats::default();

    for frame in &timeline.frames {
        let counts_toward_motion = powerslide_frame_counts_toward_motion(frame);

        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            let previous_active = active_states
                .get(&event.player)
                .is_some_and(|state| state.active);

            active_states.insert(
                event.player.clone(),
                DerivedPowerslideState {
                    active: event.active,
                    is_team_0: event.is_team_0,
                },
            );

            if counts_toward_motion && event.active && !previous_active {
                players.entry(event.player.clone()).or_default().press_count += 1;
                if event.is_team_0 {
                    team_zero.press_count += 1;
                } else {
                    team_one.press_count += 1;
                }
            }

            event_index += 1;
        }

        if counts_toward_motion {
            for player in &frame.players {
                if active_states
                    .get(&player.player_id)
                    .is_some_and(|state| state.active)
                {
                    players
                        .entry(player.player_id.clone())
                        .or_default()
                        .total_duration += frame.dt;
                    if player.is_team_0 {
                        team_zero.total_duration += frame.dt;
                    } else {
                        team_one.total_duration += frame.dt;
                    }
                }
            }
        }

        assert!(
            (frame.team_zero.powerslide.total_duration - team_zero.total_duration).abs() < 0.001,
            "{replay_path} team_zero powerslide.total_duration frame {} actual {:.3} expected {:.3}",
            frame.frame_number,
            frame.team_zero.powerslide.total_duration,
            team_zero.total_duration
        );
        assert_eq!(
            frame.team_zero.powerslide.press_count, team_zero.press_count,
            "{replay_path} team_zero powerslide.press_count frame {}",
            frame.frame_number
        );
        assert!(
            (frame.team_one.powerslide.total_duration - team_one.total_duration).abs() < 0.001,
            "{replay_path} team_one powerslide.total_duration frame {} actual {:.3} expected {:.3}",
            frame.frame_number,
            frame.team_one.powerslide.total_duration,
            team_one.total_duration
        );
        assert_eq!(
            frame.team_one.powerslide.press_count, team_one.press_count,
            "{replay_path} team_one powerslide.press_count frame {}",
            frame.frame_number
        );

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert!(
                (player.powerslide.total_duration - expected.total_duration).abs() < 0.001,
                "{replay_path} player {} powerslide.total_duration frame {} actual {:.3} expected {:.3}",
                player.name,
                frame.frame_number,
                player.powerslide.total_duration,
                expected.total_duration
            );
            assert_eq!(
                player.powerslide.press_count, expected.press_count,
                "{replay_path} player {} powerslide.press_count frame {}",
                player.name, frame.frame_number
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed powerslide events"
    );
}

fn touch_label_for_derivation(key: &'static str, value: &str) -> StatLabel {
    match (key, value) {
        ("kind", "control") => StatLabel::new("kind", "control"),
        ("kind", "medium_hit") => StatLabel::new("kind", "medium_hit"),
        ("kind", "hard_hit") => StatLabel::new("kind", "hard_hit"),
        ("height_band", "ground") => StatLabel::new("height_band", "ground"),
        ("height_band", "low_air") => StatLabel::new("height_band", "low_air"),
        ("height_band", "high_air") => StatLabel::new("height_band", "high_air"),
        ("surface", "ground") => StatLabel::new("surface", "ground"),
        ("surface", "air") => StatLabel::new("surface", "air"),
        ("surface", "wall") => StatLabel::new("surface", "wall"),
        ("dodge_state", "no_dodge") => StatLabel::new("dodge_state", "no_dodge"),
        ("dodge_state", "dodge") => StatLabel::new("dodge_state", "dodge"),
        _ => panic!("unexpected touch label {key}={value}"),
    }
}

fn apply_touch_stats_event_for_derivation(
    stats: &mut TouchStats,
    event: &TouchStatsEvent,
    frame: &ReplayStatsFrame,
) {
    stats.touch_count += 1;
    match event.kind.as_str() {
        "control" => stats.control_touch_count += 1,
        "medium_hit" => stats.medium_hit_count += 1,
        "hard_hit" => stats.hard_hit_count += 1,
        value => panic!("unexpected touch kind {value}"),
    }
    match event.height_band.as_str() {
        "ground" => {}
        "low_air" => stats.aerial_touch_count += 1,
        "high_air" => {
            stats.aerial_touch_count += 1;
            stats.high_aerial_touch_count += 1;
        }
        value => panic!("unexpected touch height band {value}"),
    }
    match event.surface.as_str() {
        "wall" => stats.wall_touch_count += 1,
        "ground" | "air" => {}
        value => panic!("unexpected touch surface {value}"),
    }
    stats.labeled_touch_counts.increment([
        touch_label_for_derivation("kind", &event.kind),
        touch_label_for_derivation("height_band", &event.height_band),
        touch_label_for_derivation("surface", &event.surface),
        touch_label_for_derivation("dodge_state", &event.dodge_state),
    ]);
    stats.last_touch_time = Some(event.time);
    stats.last_touch_frame = Some(event.frame);
    stats.time_since_last_touch = Some((frame.time - event.time).max(0.0));
    stats.frames_since_last_touch = Some(frame.frame_number.saturating_sub(event.frame));
    stats.last_ball_speed_change = Some(event.ball_speed_change);
    stats.max_ball_speed_change = stats.max_ball_speed_change.max(event.ball_speed_change);
    stats.cumulative_ball_speed_change += event.ball_speed_change;
}

fn assert_touch_stats_close(
    replay_path: &str,
    player_name: &str,
    frame_number: usize,
    actual: &TouchStats,
    expected: &TouchStats,
) {
    assert_eq!(
        actual.touch_count, expected.touch_count,
        "{replay_path} player {player_name} touch.touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.control_touch_count, expected.control_touch_count,
        "{replay_path} player {player_name} touch.control_touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.medium_hit_count, expected.medium_hit_count,
        "{replay_path} player {player_name} touch.medium_hit_count frame {frame_number}"
    );
    assert_eq!(
        actual.hard_hit_count, expected.hard_hit_count,
        "{replay_path} player {player_name} touch.hard_hit_count frame {frame_number}"
    );
    assert_eq!(
        actual.aerial_touch_count, expected.aerial_touch_count,
        "{replay_path} player {player_name} touch.aerial_touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.high_aerial_touch_count, expected.high_aerial_touch_count,
        "{replay_path} player {player_name} touch.high_aerial_touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.wall_touch_count, expected.wall_touch_count,
        "{replay_path} player {player_name} touch.wall_touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.is_last_touch, expected.is_last_touch,
        "{replay_path} player {player_name} touch.is_last_touch frame {frame_number}"
    );
    assert_eq!(
        actual.last_touch_time, expected.last_touch_time,
        "{replay_path} player {player_name} touch.last_touch_time frame {frame_number}"
    );
    assert_eq!(
        actual.last_touch_frame, expected.last_touch_frame,
        "{replay_path} player {player_name} touch.last_touch_frame frame {frame_number}"
    );
    assert_eq!(
        actual.time_since_last_touch, expected.time_since_last_touch,
        "{replay_path} player {player_name} touch.time_since_last_touch frame {frame_number}"
    );
    assert_eq!(
        actual.frames_since_last_touch, expected.frames_since_last_touch,
        "{replay_path} player {player_name} touch.frames_since_last_touch frame {frame_number}"
    );
    assert_eq!(
        actual.last_ball_speed_change, expected.last_ball_speed_change,
        "{replay_path} player {player_name} touch.last_ball_speed_change frame {frame_number}"
    );
    assert!(
        (actual.max_ball_speed_change - expected.max_ball_speed_change).abs() < 0.001,
        "{replay_path} player {player_name} touch.max_ball_speed_change frame {frame_number} actual {:.3} expected {:.3}",
        actual.max_ball_speed_change,
        expected.max_ball_speed_change
    );
    assert!(
        (actual.cumulative_ball_speed_change - expected.cumulative_ball_speed_change).abs() < 0.001,
        "{replay_path} player {player_name} touch.cumulative_ball_speed_change frame {frame_number} actual {:.3} expected {:.3}",
        actual.cumulative_ball_speed_change,
        expected.cumulative_ball_speed_change
    );
    assert!(
        (actual.total_ball_travel_distance - expected.total_ball_travel_distance).abs() < 0.001,
        "{replay_path} player {player_name} touch.total_ball_travel_distance frame {frame_number} actual {:.3} expected {:.3}",
        actual.total_ball_travel_distance,
        expected.total_ball_travel_distance
    );
    assert!(
        (actual.total_ball_advance_distance - expected.total_ball_advance_distance).abs() < 0.001,
        "{replay_path} player {player_name} touch.total_ball_advance_distance frame {frame_number} actual {:.3} expected {:.3}",
        actual.total_ball_advance_distance,
        expected.total_ball_advance_distance
    );
    assert!(
        (actual.total_ball_retreat_distance - expected.total_ball_retreat_distance).abs() < 0.001,
        "{replay_path} player {player_name} touch.total_ball_retreat_distance frame {frame_number} actual {:.3} expected {:.3}",
        actual.total_ball_retreat_distance,
        expected.total_ball_retreat_distance
    );
    assert_eq!(
        actual.labeled_touch_counts, expected.labeled_touch_counts,
        "{replay_path} player {player_name} touch.labeled_touch_counts frame {frame_number}"
    );
}

fn assert_touch_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut touch_events = timeline.events.touch.clone();
    touch_events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut movement_events = timeline.events.touch_ball_movement.clone();
    movement_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut last_touch_events = timeline.events.touch_last_touch.clone();
    last_touch_events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut touch_event_index = 0;
    let mut movement_event_index = 0;
    let mut last_touch_event_index = 0;
    let mut current_last_touch_player: Option<PlayerId> = None;
    let mut players: HashMap<PlayerId, TouchStats> = HashMap::new();

    for frame in &timeline.frames {
        if !frame.is_live_play {
            current_last_touch_player = None;
        } else {
            for stats in players.values_mut() {
                stats.is_last_touch = false;
                if let Some(last_touch_time) = stats.last_touch_time {
                    stats.time_since_last_touch = Some((frame.time - last_touch_time).max(0.0));
                }
                if let Some(last_touch_frame) = stats.last_touch_frame {
                    stats.frames_since_last_touch =
                        Some(frame.frame_number.saturating_sub(last_touch_frame));
                }
            }

            while touch_event_index < touch_events.len()
                && touch_events[touch_event_index].sample_frame <= frame.frame_number
            {
                let event = &touch_events[touch_event_index];
                apply_touch_stats_event_for_derivation(
                    players.entry(event.player.clone()).or_default(),
                    event,
                    frame,
                );
                touch_event_index += 1;
            }

            while last_touch_event_index < last_touch_events.len()
                && last_touch_events[last_touch_event_index].sample_frame <= frame.frame_number
            {
                current_last_touch_player =
                    last_touch_events[last_touch_event_index].player.clone();
                last_touch_event_index += 1;
            }

            if let Some(player_id) = current_last_touch_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_touch = true;
                }
            }
        }

        while movement_event_index < movement_events.len()
            && movement_events[movement_event_index].frame <= frame.frame_number
        {
            let event = &movement_events[movement_event_index];
            let stats = players.entry(event.player.clone()).or_default();
            stats.total_ball_travel_distance += event.travel_distance;
            stats.total_ball_advance_distance += event.advance_distance;
            stats.total_ball_retreat_distance += event.retreat_distance;
            movement_event_index += 1;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_touch_stats_close(
                replay_path,
                &player.name,
                frame.frame_number,
                &player.touch,
                &expected,
            );
        }
    }

    assert_eq!(
        touch_event_index,
        touch_events.len(),
        "{replay_path} unprocessed touch events"
    );
    assert_eq!(
        movement_event_index,
        movement_events.len(),
        "{replay_path} unprocessed touch ball movement events"
    );
    assert_eq!(
        last_touch_event_index,
        last_touch_events.len(),
        "{replay_path} unprocessed touch last-touch events"
    );
}

fn assert_core_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut player_events = timeline.events.core_player.clone();
    player_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut team_events = timeline.events.core_team.clone();
    team_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut player_event_index = 0;
    let mut team_event_index = 0;
    let mut players: HashMap<PlayerId, CorePlayerStats> = HashMap::new();
    let mut team_zero = CoreTeamStats::default();
    let mut team_one = CoreTeamStats::default();

    for frame in &timeline.frames {
        while player_event_index < player_events.len()
            && player_events[player_event_index].frame <= frame.frame_number
        {
            let event = &player_events[player_event_index];
            apply_core_player_delta(
                players.entry(event.player.clone()).or_default(),
                &event.delta,
            );
            player_event_index += 1;
        }

        while team_event_index < team_events.len()
            && team_events[team_event_index].frame <= frame.frame_number
        {
            let event = &team_events[team_event_index];
            if event.is_team_0 {
                apply_core_team_delta(&mut team_zero, &event.delta);
            } else {
                apply_core_team_delta(&mut team_one, &event.delta);
            }
            team_event_index += 1;
        }

        assert_eq!(
            frame.team_zero.core, team_zero,
            "{replay_path} team_zero core frame {}",
            frame.frame_number
        );
        assert_eq!(
            frame.team_one.core, team_one,
            "{replay_path} team_one core frame {}",
            frame.frame_number
        );

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.core, expected,
                "{replay_path} player {} core frame {}",
                player.name, frame.frame_number
            );
        }
    }

    assert_eq!(
        player_event_index,
        player_events.len(),
        "{replay_path} unprocessed core player events"
    );
    assert_eq!(
        team_event_index,
        team_events.len(),
        "{replay_path} unprocessed core team events"
    );
}

fn apply_goal_after_kickoff_delta(
    stats: &mut GoalAfterKickoffStats,
    delta: &GoalAfterKickoffStats,
) {
    if delta.goal_times().is_empty() {
        stats.kickoff_goal_count += delta.kickoff_goal_count;
        stats.short_goal_count += delta.short_goal_count;
        stats.medium_goal_count += delta.medium_goal_count;
        stats.long_goal_count += delta.long_goal_count;
    } else {
        for time in delta.goal_times() {
            stats.record_goal(*time);
        }
    }
}

fn apply_goal_buildup_delta(stats: &mut GoalBuildupStats, delta: &GoalBuildupStats) {
    stats.counter_attack_goal_count += delta.counter_attack_goal_count;
    stats.sustained_pressure_goal_count += delta.sustained_pressure_goal_count;
    stats.other_buildup_goal_count += delta.other_buildup_goal_count;
}

fn apply_goal_ball_air_time_delta(stats: &mut GoalBallAirTimeStats, delta: &GoalBallAirTimeStats) {
    if delta.goal_ball_air_times().is_empty() {
        stats.goal_ball_air_time_sample_count += delta.goal_ball_air_time_sample_count;
        stats.cumulative_goal_ball_air_time += delta.cumulative_goal_ball_air_time;
        if delta.last_goal_ball_air_time.is_some() {
            stats.last_goal_ball_air_time = delta.last_goal_ball_air_time;
        }
    } else {
        let previous_last_goal_ball_air_time = stats.last_goal_ball_air_time;
        for time in delta.goal_ball_air_times() {
            stats.record_goal(*time);
        }
        stats.last_goal_ball_air_time = delta
            .last_goal_ball_air_time
            .or(previous_last_goal_ball_air_time);
    }
}

fn apply_core_team_delta(stats: &mut CoreTeamStats, delta: &CoreTeamStats) {
    stats.score += delta.score;
    stats.goals += delta.goals;
    stats.assists += delta.assists;
    stats.saves += delta.saves;
    stats.shots += delta.shots;
    apply_goal_after_kickoff_delta(
        &mut stats.scoring_context.goal_after_kickoff,
        &delta.scoring_context.goal_after_kickoff,
    );
    apply_goal_buildup_delta(
        &mut stats.scoring_context.goal_buildup,
        &delta.scoring_context.goal_buildup,
    );
    apply_goal_ball_air_time_delta(
        &mut stats.scoring_context.goal_ball_air_time,
        &delta.scoring_context.goal_ball_air_time,
    );
}

fn apply_core_player_delta(stats: &mut CorePlayerStats, delta: &CorePlayerStats) {
    stats.score += delta.score;
    stats.goals += delta.goals;
    stats.assists += delta.assists;
    stats.saves += delta.saves;
    stats.shots += delta.shots;
    stats.scoring_context.goals_conceded_while_last_defender +=
        delta.scoring_context.goals_conceded_while_last_defender;
    stats.scoring_context.goals_for_while_most_back +=
        delta.scoring_context.goals_for_while_most_back;
    stats.scoring_context.goals_against_while_most_back +=
        delta.scoring_context.goals_against_while_most_back;
    stats.scoring_context.goal_against_boost_sample_count +=
        delta.scoring_context.goal_against_boost_sample_count;
    stats.scoring_context.cumulative_boost_on_goals_against +=
        delta.scoring_context.cumulative_boost_on_goals_against;
    if delta.scoring_context.last_boost_on_goal_against.is_some() {
        stats.scoring_context.last_boost_on_goal_against =
            delta.scoring_context.last_boost_on_goal_against;
    }
    stats.scoring_context.goal_against_boost_leadup_sample_count +=
        delta.scoring_context.goal_against_boost_leadup_sample_count;
    stats
        .scoring_context
        .cumulative_average_boost_in_goal_against_leadup += delta
        .scoring_context
        .cumulative_average_boost_in_goal_against_leadup;
    stats
        .scoring_context
        .cumulative_min_boost_in_goal_against_leadup += delta
        .scoring_context
        .cumulative_min_boost_in_goal_against_leadup;
    if delta
        .scoring_context
        .last_average_boost_in_goal_against_leadup
        .is_some()
    {
        stats
            .scoring_context
            .last_average_boost_in_goal_against_leadup = delta
            .scoring_context
            .last_average_boost_in_goal_against_leadup;
    }
    if delta
        .scoring_context
        .last_min_boost_in_goal_against_leadup
        .is_some()
    {
        stats.scoring_context.last_min_boost_in_goal_against_leadup =
            delta.scoring_context.last_min_boost_in_goal_against_leadup;
    }
    stats.scoring_context.goal_against_position_sample_count +=
        delta.scoring_context.goal_against_position_sample_count;
    stats.scoring_context.cumulative_goal_against_position_x +=
        delta.scoring_context.cumulative_goal_against_position_x;
    stats.scoring_context.cumulative_goal_against_position_y +=
        delta.scoring_context.cumulative_goal_against_position_y;
    stats.scoring_context.cumulative_goal_against_position_z +=
        delta.scoring_context.cumulative_goal_against_position_z;
    if delta.scoring_context.last_goal_against_position.is_some() {
        stats.scoring_context.last_goal_against_position =
            delta.scoring_context.last_goal_against_position;
    }
    stats
        .scoring_context
        .scoring_goal_last_touch_position_sample_count += delta
        .scoring_context
        .scoring_goal_last_touch_position_sample_count;
    stats
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_x += delta
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_x;
    stats
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_y += delta
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_y;
    stats
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_z += delta
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_z;
    if delta
        .scoring_context
        .last_scoring_goal_last_touch_position
        .is_some()
    {
        stats.scoring_context.last_scoring_goal_last_touch_position =
            delta.scoring_context.last_scoring_goal_last_touch_position;
    }
    apply_goal_after_kickoff_delta(
        &mut stats.scoring_context.goal_after_kickoff,
        &delta.scoring_context.goal_after_kickoff,
    );
    apply_goal_buildup_delta(
        &mut stats.scoring_context.goal_buildup,
        &delta.scoring_context.goal_buildup,
    );
    apply_goal_ball_air_time_delta(
        &mut stats.scoring_context.goal_ball_air_time,
        &delta.scoring_context.goal_ball_air_time,
    );
}

fn possession_label_for_derivation(key: &'static str, value: &str) -> StatLabel {
    match (key, value) {
        ("possession_state", "team_zero") => StatLabel::new("possession_state", "team_zero"),
        ("possession_state", "team_one") => StatLabel::new("possession_state", "team_one"),
        ("possession_state", "neutral") => StatLabel::new("possession_state", "neutral"),
        ("field_third", "team_zero_third") => StatLabel::new("field_third", "team_zero_third"),
        ("field_third", "neutral_third") => StatLabel::new("field_third", "neutral_third"),
        ("field_third", "team_one_third") => StatLabel::new("field_third", "team_one_third"),
        _ => panic!("unexpected possession label {key}={value}"),
    }
}

#[derive(Debug, Clone, Default)]
struct PossessionDerivationState {
    active: bool,
    possession_state: String,
    field_third: Option<String>,
}

fn apply_possession_event_for_derivation(
    state: &mut PossessionDerivationState,
    event: &PossessionEvent,
) {
    state.active = event.active;
    state.possession_state = event.possession_state.clone();
    state.field_third = event.field_third.clone();
}

fn accumulate_possession_frame_for_derivation(
    stats: &mut PossessionStats,
    state: &PossessionDerivationState,
    frame: &ReplayStatsFrame,
) {
    if !state.active {
        return;
    }

    stats.tracked_time += frame.dt;
    match state.possession_state.as_str() {
        "team_zero" => stats.team_zero_time += frame.dt,
        "team_one" => stats.team_one_time += frame.dt,
        "neutral" => stats.neutral_time += frame.dt,
        value => panic!("unexpected possession state {value}"),
    }

    let state_label = possession_label_for_derivation("possession_state", &state.possession_state);
    if let Some(field_third) = state.field_third.as_deref() {
        stats.labeled_time.add(
            [
                state_label,
                possession_label_for_derivation("field_third", field_third),
            ],
            frame.dt,
        );
    } else {
        stats.labeled_time.add([state_label], frame.dt);
    }
}

fn assert_labeled_float_sums_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &LabeledFloatSums,
    expected: &LabeledFloatSums,
) {
    assert_eq!(
        actual.entries.len(),
        expected.entries.len(),
        "{replay_path} {label}.labeled_time entry count frame {frame_number}"
    );
    for (actual_entry, expected_entry) in actual.entries.iter().zip(&expected.entries) {
        assert_eq!(
            actual_entry.labels, expected_entry.labels,
            "{replay_path} {label}.labeled_time labels frame {frame_number}"
        );
        assert!(
            (actual_entry.value - expected_entry.value).abs() < 0.001,
            "{replay_path} {label}.labeled_time {:?} frame {frame_number} actual {:.3} expected {:.3}",
            actual_entry.labels,
            actual_entry.value,
            expected_entry.value
        );
    }
}

fn assert_possession_team_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &PossessionTeamStats,
    expected: &PossessionTeamStats,
) {
    assert!(
        (actual.tracked_time - expected.tracked_time).abs() < 0.001,
        "{replay_path} {label}.tracked_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.tracked_time,
        expected.tracked_time
    );
    assert!(
        (actual.possession_time - expected.possession_time).abs() < 0.001,
        "{replay_path} {label}.possession_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.possession_time,
        expected.possession_time
    );
    assert!(
        (actual.opponent_possession_time - expected.opponent_possession_time).abs() < 0.001,
        "{replay_path} {label}.opponent_possession_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.opponent_possession_time,
        expected.opponent_possession_time
    );
    assert!(
        (actual.neutral_time - expected.neutral_time).abs() < 0.001,
        "{replay_path} {label}.neutral_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.neutral_time,
        expected.neutral_time
    );
    assert_labeled_float_sums_close(
        replay_path,
        label,
        frame_number,
        &actual.labeled_time,
        &expected.labeled_time,
    );
}

fn assert_possession_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.possession.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut stats = PossessionStats::default();
    let mut state = PossessionDerivationState {
        active: false,
        possession_state: "neutral".to_owned(),
        field_third: None,
    };

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            apply_possession_event_for_derivation(&mut state, &events[event_index]);
            event_index += 1;
        }

        accumulate_possession_frame_for_derivation(&mut stats, &state, frame);
        assert_possession_team_stats_close(
            replay_path,
            "team_zero.possession",
            frame.frame_number,
            &frame.team_zero.possession,
            &stats.for_team(true),
        );
        assert_possession_team_stats_close(
            replay_path,
            "team_one.possession",
            frame.frame_number,
            &frame.team_one.possession,
            &stats.for_team(false),
        );
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed possession events"
    );
}

fn pressure_label_for_derivation(value: &str) -> StatLabel {
    match value {
        "team_zero_side" => StatLabel::new("field_half", "team_zero_side"),
        "team_one_side" => StatLabel::new("field_half", "team_one_side"),
        "neutral" => StatLabel::new("field_half", "neutral"),
        _ => panic!("unexpected pressure field_half={value}"),
    }
}

#[derive(Debug, Clone)]
struct PressureDerivationState {
    active: bool,
    field_half: String,
}

impl Default for PressureDerivationState {
    fn default() -> Self {
        Self {
            active: false,
            field_half: "neutral".to_owned(),
        }
    }
}

fn apply_pressure_event_for_derivation(state: &mut PressureDerivationState, event: &PressureEvent) {
    state.active = event.active;
    state.field_half = event.field_half.clone();
}

fn accumulate_pressure_frame_for_derivation(
    stats: &mut PressureStats,
    state: &PressureDerivationState,
    frame: &ReplayStatsFrame,
) {
    if !state.active {
        return;
    }

    stats.tracked_time += frame.dt;
    match state.field_half.as_str() {
        "team_zero_side" => stats.team_zero_side_time += frame.dt,
        "team_one_side" => stats.team_one_side_time += frame.dt,
        "neutral" => stats.neutral_time += frame.dt,
        value => panic!("unexpected pressure field half {value}"),
    }
    stats
        .labeled_time
        .add([pressure_label_for_derivation(&state.field_half)], frame.dt);
}

fn assert_pressure_team_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &PressureTeamStats,
    expected: &PressureTeamStats,
) {
    assert!(
        (actual.tracked_time - expected.tracked_time).abs() < 0.001,
        "{replay_path} {label}.tracked_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.tracked_time,
        expected.tracked_time
    );
    assert!(
        (actual.defensive_half_time - expected.defensive_half_time).abs() < 0.001,
        "{replay_path} {label}.defensive_half_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.defensive_half_time,
        expected.defensive_half_time
    );
    assert!(
        (actual.offensive_half_time - expected.offensive_half_time).abs() < 0.001,
        "{replay_path} {label}.offensive_half_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.offensive_half_time,
        expected.offensive_half_time
    );
    assert!(
        (actual.neutral_time - expected.neutral_time).abs() < 0.001,
        "{replay_path} {label}.neutral_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.neutral_time,
        expected.neutral_time
    );
    assert_labeled_float_sums_close(
        replay_path,
        label,
        frame_number,
        &actual.labeled_time,
        &expected.labeled_time,
    );
}

fn assert_pressure_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.pressure.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut stats = PressureStats::default();
    let mut state = PressureDerivationState::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            apply_pressure_event_for_derivation(&mut state, &events[event_index]);
            event_index += 1;
        }

        accumulate_pressure_frame_for_derivation(&mut stats, &state, frame);
        assert_pressure_team_stats_close(
            replay_path,
            "team_zero.pressure",
            frame.frame_number,
            &frame.team_zero.pressure,
            &stats.for_team(true),
        );
        assert_pressure_team_stats_close(
            replay_path,
            "team_one.pressure",
            frame.frame_number,
            &frame.team_one.pressure,
            &stats.for_team(false),
        );
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed pressure events"
    );
}

fn movement_label_for_derivation(key: &'static str, value: &str) -> StatLabel {
    match (key, value) {
        ("speed_band", "slow") => StatLabel::new("speed_band", "slow"),
        ("speed_band", "boost") => StatLabel::new("speed_band", "boost"),
        ("speed_band", "supersonic") => StatLabel::new("speed_band", "supersonic"),
        ("height_band", "ground") => StatLabel::new("height_band", "ground"),
        ("height_band", "low_air") => StatLabel::new("height_band", "low_air"),
        ("height_band", "high_air") => StatLabel::new("height_band", "high_air"),
        _ => panic!("unexpected movement label {key}={value}"),
    }
}

fn apply_movement_event_for_derivation(stats: &mut MovementStats, event: &MovementEvent) {
    stats.tracked_time += event.dt;
    stats.total_distance += event.distance;
    stats.speed_integral += event.speed * event.dt;

    match event.speed_band.as_str() {
        "slow" => stats.time_slow_speed += event.dt,
        "boost" => stats.time_boost_speed += event.dt,
        "supersonic" => stats.time_supersonic_speed += event.dt,
        value => panic!("unexpected movement speed band {value}"),
    }

    match event.height_band.as_str() {
        "ground" => stats.time_on_ground += event.dt,
        "low_air" => stats.time_low_air += event.dt,
        "high_air" => stats.time_high_air += event.dt,
        value => panic!("unexpected movement height band {value}"),
    }

    stats.labeled_tracked_time.add(
        [
            movement_label_for_derivation("speed_band", &event.speed_band),
            movement_label_for_derivation("height_band", &event.height_band),
        ],
        event.dt,
    );
}

fn assert_movement_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &MovementStats,
    expected: &MovementStats,
) {
    assert!(
        (actual.tracked_time - expected.tracked_time).abs() < 0.001,
        "{replay_path} {label}.tracked_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.tracked_time,
        expected.tracked_time
    );
    assert!(
        (actual.total_distance - expected.total_distance).abs() < 0.001,
        "{replay_path} {label}.total_distance frame {frame_number} actual {:.3} expected {:.3}",
        actual.total_distance,
        expected.total_distance
    );
    assert!(
        (actual.speed_integral - expected.speed_integral).abs() < 0.001,
        "{replay_path} {label}.speed_integral frame {frame_number} actual {:.3} expected {:.3}",
        actual.speed_integral,
        expected.speed_integral
    );
    assert!(
        (actual.time_slow_speed - expected.time_slow_speed).abs() < 0.001,
        "{replay_path} {label}.time_slow_speed frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_slow_speed,
        expected.time_slow_speed
    );
    assert!(
        (actual.time_boost_speed - expected.time_boost_speed).abs() < 0.001,
        "{replay_path} {label}.time_boost_speed frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_boost_speed,
        expected.time_boost_speed
    );
    assert!(
        (actual.time_supersonic_speed - expected.time_supersonic_speed).abs() < 0.001,
        "{replay_path} {label}.time_supersonic_speed frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_supersonic_speed,
        expected.time_supersonic_speed
    );
    assert!(
        (actual.time_on_ground - expected.time_on_ground).abs() < 0.001,
        "{replay_path} {label}.time_on_ground frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_on_ground,
        expected.time_on_ground
    );
    assert!(
        (actual.time_low_air - expected.time_low_air).abs() < 0.001,
        "{replay_path} {label}.time_low_air frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_low_air,
        expected.time_low_air
    );
    assert!(
        (actual.time_high_air - expected.time_high_air).abs() < 0.001,
        "{replay_path} {label}.time_high_air frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_high_air,
        expected.time_high_air
    );
    assert_labeled_float_sums_close(
        replay_path,
        label,
        frame_number,
        &actual.labeled_tracked_time,
        &expected.labeled_tracked_time,
    );
}

fn assert_movement_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.movement.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, MovementStats> = HashMap::new();
    let mut team_zero = MovementStats::default();
    let mut team_one = MovementStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            apply_movement_event_for_derivation(
                players.entry(event.player.clone()).or_default(),
                event,
            );
            if event.is_team_0 {
                apply_movement_event_for_derivation(&mut team_zero, event);
            } else {
                apply_movement_event_for_derivation(&mut team_one, event);
            }
            event_index += 1;
        }

        assert_movement_stats_close(
            replay_path,
            "team_zero.movement",
            frame.frame_number,
            &frame.team_zero.movement,
            &team_zero,
        );
        assert_movement_stats_close(
            replay_path,
            "team_one.movement",
            frame.frame_number,
            &frame.team_one.movement,
            &team_one,
        );

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_movement_stats_close(
                replay_path,
                &format!("player {} movement", player.name),
                frame.frame_number,
                &player.movement,
                &expected,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed movement events"
    );
}

fn apply_positioning_event_for_derivation(stats: &mut PositioningStats, event: &PositioningEvent) {
    stats.active_game_time += event.active_game_time;
    stats.tracked_time += event.tracked_time;
    stats.sum_distance_to_teammates += event.sum_distance_to_teammates;
    stats.sum_distance_to_ball += event.sum_distance_to_ball;
    stats.sum_distance_to_ball_has_possession += event.sum_distance_to_ball_has_possession;
    stats.time_has_possession += event.time_has_possession;
    stats.sum_distance_to_ball_no_possession += event.sum_distance_to_ball_no_possession;
    stats.time_no_possession += event.time_no_possession;
    stats.time_demolished += event.time_demolished;
    stats.time_no_teammates += event.time_no_teammates;
    stats.time_most_back += event.time_most_back;
    stats.time_most_forward += event.time_most_forward;
    stats.time_mid_role += event.time_mid_role;
    stats.time_other_role += event.time_other_role;
    stats.time_defensive_zone += event.time_defensive_zone;
    stats.time_neutral_zone += event.time_neutral_zone;
    stats.time_offensive_zone += event.time_offensive_zone;
    stats.time_defensive_half += event.time_defensive_half;
    stats.time_offensive_half += event.time_offensive_half;
    stats.time_closest_to_ball += event.time_closest_to_ball;
    stats.time_farthest_from_ball += event.time_farthest_from_ball;
    stats.time_behind_ball += event.time_behind_ball;
    stats.time_level_with_ball += event.time_level_with_ball;
    stats.time_in_front_of_ball += event.time_in_front_of_ball;
    stats.times_caught_ahead_of_play_on_conceded_goals +=
        event.times_caught_ahead_of_play_on_conceded_goals;
}

fn assert_positioning_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &PositioningStats,
    expected: &PositioningStats,
) {
    macro_rules! assert_close_field {
        ($field:ident) => {
            assert!(
                (actual.$field - expected.$field).abs() < 0.001,
                "{replay_path} {label}.{} frame {frame_number} actual {:.3} expected {:.3}",
                stringify!($field),
                actual.$field,
                expected.$field
            );
        };
    }

    assert_close_field!(active_game_time);
    assert_close_field!(tracked_time);
    assert_close_field!(sum_distance_to_teammates);
    assert_close_field!(sum_distance_to_ball);
    assert_close_field!(sum_distance_to_ball_has_possession);
    assert_close_field!(time_has_possession);
    assert_close_field!(sum_distance_to_ball_no_possession);
    assert_close_field!(time_no_possession);
    assert_close_field!(time_demolished);
    assert_close_field!(time_no_teammates);
    assert_close_field!(time_most_back);
    assert_close_field!(time_most_forward);
    assert_close_field!(time_mid_role);
    assert_close_field!(time_other_role);
    assert_close_field!(time_defensive_zone);
    assert_close_field!(time_neutral_zone);
    assert_close_field!(time_offensive_zone);
    assert_close_field!(time_defensive_half);
    assert_close_field!(time_offensive_half);
    assert_close_field!(time_closest_to_ball);
    assert_close_field!(time_farthest_from_ball);
    assert_close_field!(time_behind_ball);
    assert_close_field!(time_level_with_ball);
    assert_close_field!(time_in_front_of_ball);
    assert_eq!(
        actual.times_caught_ahead_of_play_on_conceded_goals,
        expected.times_caught_ahead_of_play_on_conceded_goals,
        "{replay_path} {label}.times_caught_ahead_of_play_on_conceded_goals frame {frame_number}"
    );
}

fn assert_positioning_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.positioning.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, PositioningStats> = HashMap::new();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            apply_positioning_event_for_derivation(
                players.entry(event.player.clone()).or_default(),
                event,
            );
            event_index += 1;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_positioning_stats_close(
                replay_path,
                &format!("player {} positioning", player.name),
                frame.frame_number,
                &player.positioning,
                &expected,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed positioning events"
    );
}

#[derive(Debug, Clone, Default)]
struct RotationPlayerDerivationState {
    active: bool,
    first_man_stint_active: bool,
    current_first_man_stint_time: f32,
    non_first_man_seconds: f32,
    stats: RotationPlayerStats,
}

fn apply_rotation_player_event_for_derivation(
    state: &mut RotationPlayerDerivationState,
    event: &RotationPlayerEvent,
) {
    state.active = event.active;
    if !event.active {
        state.first_man_stint_active = false;
        state.current_first_man_stint_time = 0.0;
        state.non_first_man_seconds = 0.0;
    }
    let stats = &mut state.stats;
    stats.became_first_man_count += event.became_first_man_count;
    stats.lost_first_man_count += event.lost_first_man_count;
    stats.current_role_state = event.current_role_state;
    stats.current_depth_state = event.current_depth_state;
}

fn accumulate_rotation_player_frame_for_derivation(
    state: &mut RotationPlayerDerivationState,
    frame: &ReplayStatsFrame,
    first_man_stint_end_grace_seconds: f32,
) {
    if !state.active {
        return;
    }

    state.stats.active_game_time += frame.dt;
    state.stats.tracked_time += frame.dt;

    match state.stats.current_role_state {
        RoleState::FirstMan => {
            if !state.first_man_stint_active {
                state.first_man_stint_active = true;
                state.current_first_man_stint_time = 0.0;
                state.stats.first_man_stint_count += 1;
            }
            state.current_first_man_stint_time += frame.dt;
            state.stats.longest_first_man_stint_time = state
                .stats
                .longest_first_man_stint_time
                .max(state.current_first_man_stint_time);
            state.non_first_man_seconds = 0.0;
            state.stats.time_first_man += frame.dt;
        }
        RoleState::SecondMan => {
            update_non_first_man_stint_state(state, frame.dt, first_man_stint_end_grace_seconds);
            state.stats.time_second_man += frame.dt;
        }
        RoleState::ThirdMan => {
            update_non_first_man_stint_state(state, frame.dt, first_man_stint_end_grace_seconds);
            state.stats.time_third_man += frame.dt;
        }
        RoleState::Ambiguous => {
            update_non_first_man_stint_state(state, frame.dt, first_man_stint_end_grace_seconds);
            state.stats.time_ambiguous_role += frame.dt;
        }
        RoleState::Unknown => {
            update_non_first_man_stint_state(state, frame.dt, first_man_stint_end_grace_seconds)
        }
    }

    match state.stats.current_depth_state {
        PlayDepthState::BehindPlay => state.stats.time_behind_play += frame.dt,
        PlayDepthState::LevelWithPlay => state.stats.time_level_with_play += frame.dt,
        PlayDepthState::AheadOfPlay => state.stats.time_ahead_of_play += frame.dt,
        PlayDepthState::Unknown => {}
    }
}

fn update_non_first_man_stint_state(
    state: &mut RotationPlayerDerivationState,
    dt: f32,
    first_man_stint_end_grace_seconds: f32,
) {
    if !state.first_man_stint_active {
        return;
    }

    state.non_first_man_seconds += dt;
    if state.non_first_man_seconds > first_man_stint_end_grace_seconds {
        state.first_man_stint_active = false;
        state.current_first_man_stint_time = 0.0;
        state.non_first_man_seconds = 0.0;
    }
}

fn apply_rotation_team_event_for_derivation(
    stats: &mut RotationTeamStats,
    event: &RotationTeamEvent,
) {
    stats.first_man_changes_for_team += event.first_man_changes_for_team;
    stats.rotation_count += event.rotation_count;
}

fn assert_rotation_player_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &RotationPlayerStats,
    expected: &RotationPlayerStats,
) {
    macro_rules! assert_close_field {
        ($field:ident) => {
            assert!(
                (actual.$field - expected.$field).abs() < 0.001,
                "{replay_path} {label}.{} frame {frame_number} actual {:.3} expected {:.3}",
                stringify!($field),
                actual.$field,
                expected.$field
            );
        };
    }

    assert_close_field!(active_game_time);
    assert_close_field!(tracked_time);
    assert_close_field!(time_first_man);
    assert_close_field!(time_second_man);
    assert_close_field!(time_third_man);
    assert_close_field!(time_ambiguous_role);
    assert_close_field!(time_behind_play);
    assert_close_field!(time_level_with_play);
    assert_close_field!(time_ahead_of_play);
    assert_close_field!(longest_first_man_stint_time);
    assert_eq!(
        actual.first_man_stint_count, expected.first_man_stint_count,
        "{replay_path} {label}.first_man_stint_count frame {frame_number}"
    );
    assert_eq!(
        actual.became_first_man_count, expected.became_first_man_count,
        "{replay_path} {label}.became_first_man_count frame {frame_number}"
    );
    assert_eq!(
        actual.lost_first_man_count, expected.lost_first_man_count,
        "{replay_path} {label}.lost_first_man_count frame {frame_number}"
    );
    assert_eq!(
        actual.current_role_state, expected.current_role_state,
        "{replay_path} {label}.current_role_state frame {frame_number}"
    );
    assert_eq!(
        actual.current_depth_state, expected.current_depth_state,
        "{replay_path} {label}.current_depth_state frame {frame_number}"
    );
}

fn assert_rotation_team_stats_equal(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &RotationTeamStats,
    expected: &RotationTeamStats,
) {
    assert_eq!(
        actual.first_man_changes_for_team, expected.first_man_changes_for_team,
        "{replay_path} {label}.first_man_changes_for_team frame {frame_number}"
    );
    assert_eq!(
        actual.rotation_count, expected.rotation_count,
        "{replay_path} {label}.rotation_count frame {frame_number}"
    );
}

fn assert_rotation_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut player_events = timeline.events.rotation_player.clone();
    player_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut team_events = timeline.events.rotation_team.clone();
    team_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut player_event_index = 0;
    let mut team_event_index = 0;
    let mut players: HashMap<PlayerId, RotationPlayerDerivationState> = HashMap::new();
    let mut team_zero = RotationTeamStats::default();
    let mut team_one = RotationTeamStats::default();
    let first_man_stint_end_grace_seconds = timeline.config.rotation_first_man_debounce_seconds;

    for frame in &timeline.frames {
        while player_event_index < player_events.len()
            && player_events[player_event_index].frame <= frame.frame_number
        {
            let event = &player_events[player_event_index];
            apply_rotation_player_event_for_derivation(
                players.entry(event.player.clone()).or_default(),
                event,
            );
            player_event_index += 1;
        }

        while team_event_index < team_events.len()
            && team_events[team_event_index].frame <= frame.frame_number
        {
            let event = &team_events[team_event_index];
            apply_rotation_team_event_for_derivation(
                if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            team_event_index += 1;
        }

        assert_rotation_team_stats_equal(
            replay_path,
            "team_zero.rotation",
            frame.frame_number,
            &frame.team_zero.rotation,
            &team_zero,
        );
        assert_rotation_team_stats_equal(
            replay_path,
            "team_one.rotation",
            frame.frame_number,
            &frame.team_one.rotation,
            &team_one,
        );

        for player in &frame.players {
            if let Some(state) = players.get_mut(&player.player_id) {
                accumulate_rotation_player_frame_for_derivation(
                    state,
                    frame,
                    first_man_stint_end_grace_seconds,
                );
            }
            let expected = players
                .get(&player.player_id)
                .map(|state| state.stats.clone())
                .unwrap_or_default();
            assert_rotation_player_stats_close(
                replay_path,
                &format!("player {} rotation", player.name),
                frame.frame_number,
                &player.rotation,
                &expected,
            );
        }
    }

    assert_eq!(
        player_event_index,
        player_events.len(),
        "{replay_path} unprocessed rotation player events"
    );
    assert_eq!(
        team_event_index,
        team_events.len(),
        "{replay_path} unprocessed rotation team events"
    );
}

fn fifty_fifty_phase_label_for_derivation(is_kickoff: bool) -> StatLabel {
    if is_kickoff {
        StatLabel::new("phase", "kickoff")
    } else {
        StatLabel::new("phase", "open_play")
    }
}

fn fifty_fifty_player_outcome_label_for_derivation(
    player_team_is_team_0: bool,
    winning_team_is_team_0: Option<bool>,
) -> StatLabel {
    match winning_team_is_team_0 {
        Some(winning_team) if winning_team == player_team_is_team_0 => {
            StatLabel::new("outcome", "win")
        }
        Some(_) => StatLabel::new("outcome", "loss"),
        None => StatLabel::new("outcome", "neutral"),
    }
}

fn fifty_fifty_player_possession_label_for_derivation(
    player_team_is_team_0: bool,
    possession_team_is_team_0: Option<bool>,
) -> StatLabel {
    match possession_team_is_team_0 {
        Some(possession_team) if possession_team == player_team_is_team_0 => {
            StatLabel::new("possession_after", "self")
        }
        Some(_) => StatLabel::new("possession_after", "opponent"),
        None => StatLabel::new("possession_after", "neutral"),
    }
}

fn fifty_fifty_player_dodge_state_label_for_derivation(
    player_team_is_team_0: bool,
    event: &FiftyFiftyEvent,
) -> StatLabel {
    let dodge_contact = if player_team_is_team_0 {
        event.team_zero_dodge_contact
    } else {
        event.team_one_dodge_contact
    };
    if dodge_contact {
        StatLabel::new("dodge_state", "dodge")
    } else {
        StatLabel::new("dodge_state", "no_dodge")
    }
}

fn apply_fifty_fifty_team_event(
    stats: &mut FiftyFiftyTeamStats,
    team_is_team_0: bool,
    event: &FiftyFiftyEvent,
) {
    stats.count += 1;
    match event.winning_team_is_team_0 {
        Some(winning_team) if winning_team == team_is_team_0 => stats.wins += 1,
        Some(_) => stats.losses += 1,
        None => stats.neutral_outcomes += 1,
    }
    match event.possession_team_is_team_0 {
        Some(possession_team) if possession_team == team_is_team_0 => {
            stats.possession_after_count += 1;
        }
        Some(_) => stats.opponent_possession_after_count += 1,
        None => stats.neutral_possession_after_count += 1,
    }
    if event.is_kickoff {
        stats.kickoff_count += 1;
        match event.winning_team_is_team_0 {
            Some(winning_team) if winning_team == team_is_team_0 => stats.kickoff_wins += 1,
            Some(_) => stats.kickoff_losses += 1,
            None => stats.kickoff_neutral_outcomes += 1,
        }
        match event.possession_team_is_team_0 {
            Some(possession_team) if possession_team == team_is_team_0 => {
                stats.kickoff_possession_after_count += 1;
            }
            Some(_) => stats.kickoff_opponent_possession_after_count += 1,
            None => stats.kickoff_neutral_possession_after_count += 1,
        }
    }
}

fn apply_fifty_fifty_player_event(
    stats: &mut FiftyFiftyPlayerStats,
    player_team_is_team_0: bool,
    event: &FiftyFiftyEvent,
) {
    stats.labeled_event_counts.increment([
        fifty_fifty_phase_label_for_derivation(event.is_kickoff),
        fifty_fifty_player_outcome_label_for_derivation(
            player_team_is_team_0,
            event.winning_team_is_team_0,
        ),
        fifty_fifty_player_possession_label_for_derivation(
            player_team_is_team_0,
            event.possession_team_is_team_0,
        ),
        fifty_fifty_player_dodge_state_label_for_derivation(player_team_is_team_0, event),
    ]);
    stats.count += 1;
    match event.winning_team_is_team_0 {
        Some(winning_team) if winning_team == player_team_is_team_0 => stats.wins += 1,
        Some(_) => stats.losses += 1,
        None => stats.neutral_outcomes += 1,
    }
    if event.possession_team_is_team_0 == Some(player_team_is_team_0) {
        stats.possession_after_count += 1;
    }
    if event.is_kickoff {
        stats.kickoff_count += 1;
        match event.winning_team_is_team_0 {
            Some(winning_team) if winning_team == player_team_is_team_0 => stats.kickoff_wins += 1,
            Some(_) => stats.kickoff_losses += 1,
            None => stats.kickoff_neutral_outcomes += 1,
        }
        if event.possession_team_is_team_0 == Some(player_team_is_team_0) {
            stats.kickoff_possession_after_count += 1;
        }
    }
}

fn assert_fifty_fifty_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.fifty_fifty.clone();
    events.sort_by(|left, right| {
        left.resolve_frame
            .cmp(&right.resolve_frame)
            .then_with(|| left.resolve_time.total_cmp(&right.resolve_time))
    });

    let mut event_index = 0;
    let mut team_zero = FiftyFiftyTeamStats::default();
    let mut team_one = FiftyFiftyTeamStats::default();
    let mut players: HashMap<PlayerId, FiftyFiftyPlayerStats> = HashMap::new();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].resolve_frame <= frame.frame_number
        {
            let event = &events[event_index];
            apply_fifty_fifty_team_event(&mut team_zero, true, event);
            apply_fifty_fifty_team_event(&mut team_one, false, event);
            if let Some(player_id) = event.team_zero_player.as_ref() {
                apply_fifty_fifty_player_event(
                    players.entry(player_id.clone()).or_default(),
                    true,
                    event,
                );
            }
            if let Some(player_id) = event.team_one_player.as_ref() {
                apply_fifty_fifty_player_event(
                    players.entry(player_id.clone()).or_default(),
                    false,
                    event,
                );
            }
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.fifty_fifty, team_zero,
            "{replay_path} team_zero fifty_fifty frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.fifty_fifty, team_one,
            "{replay_path} team_one fifty_fifty frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.fifty_fifty, expected,
                "{replay_path} player {} fifty_fifty frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed fifty-fifty events"
    );
}

#[derive(Clone, Default)]
struct DerivedOneTimerPlayerStats {
    stats: OneTimerPlayerStats,
}

impl DerivedOneTimerPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_one_timer_player: bool) {
        self.stats.is_last_one_timer = is_last_one_timer_player;
        self.stats.time_since_last_one_timer = self
            .stats
            .last_one_timer_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_one_timer = self
            .stats
            .last_one_timer_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &OneTimerEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.total_ball_speed += event.ball_speed;
        self.stats.fastest_ball_speed = self.stats.fastest_ball_speed.max(event.ball_speed);
        self.stats.total_pass_distance += event.pass_travel_distance;
        self.stats.last_one_timer_time = Some(event.time);
        self.stats.last_one_timer_frame = Some(event.frame);
        self.stats.time_since_last_one_timer = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_one_timer =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_one_timer_derived_player_stats_match(
    scope: &str,
    actual: &OneTimerPlayerStats,
    expected: &OneTimerPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} one_timer.count");
    assert!(
        (actual.total_ball_speed - expected.total_ball_speed).abs() < 0.001,
        "{scope} one_timer.total_ball_speed: actual {:.3} expected {:.3}",
        actual.total_ball_speed,
        expected.total_ball_speed,
    );
    assert!(
        (actual.fastest_ball_speed - expected.fastest_ball_speed).abs() < 0.001,
        "{scope} one_timer.fastest_ball_speed: actual {:.3} expected {:.3}",
        actual.fastest_ball_speed,
        expected.fastest_ball_speed,
    );
    assert!(
        (actual.total_pass_distance - expected.total_pass_distance).abs() < 0.001,
        "{scope} one_timer.total_pass_distance: actual {:.3} expected {:.3}",
        actual.total_pass_distance,
        expected.total_pass_distance,
    );
    assert_eq!(
        actual.is_last_one_timer, expected.is_last_one_timer,
        "{scope} one_timer.is_last_one_timer"
    );
    assert_eq!(
        actual.last_one_timer_frame, expected.last_one_timer_frame,
        "{scope} one_timer.last_one_timer_frame"
    );
    assert!(
        match (actual.last_one_timer_time, expected.last_one_timer_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} one_timer.last_one_timer_time: actual {:?} expected {:?}",
        actual.last_one_timer_time,
        expected.last_one_timer_time,
    );
    assert_eq!(
        actual.frames_since_last_one_timer, expected.frames_since_last_one_timer,
        "{scope} one_timer.frames_since_last_one_timer"
    );
    assert!(
        match (
            actual.time_since_last_one_timer,
            expected.time_since_last_one_timer,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} one_timer.time_since_last_one_timer: actual {:?} expected {:?}",
        actual.time_since_last_one_timer,
        expected.time_since_last_one_timer,
    );
}

fn assert_one_timer_team_stats_match(
    scope: &str,
    actual: &OneTimerTeamStats,
    expected: &OneTimerTeamStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} one_timer.count");
    assert!(
        (actual.total_ball_speed - expected.total_ball_speed).abs() < 0.001,
        "{scope} one_timer.total_ball_speed: actual {:.3} expected {:.3}",
        actual.total_ball_speed,
        expected.total_ball_speed,
    );
    assert!(
        (actual.fastest_ball_speed - expected.fastest_ball_speed).abs() < 0.001,
        "{scope} one_timer.fastest_ball_speed: actual {:.3} expected {:.3}",
        actual.fastest_ball_speed,
        expected.fastest_ball_speed,
    );
}

fn assert_one_timer_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.one_timer.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedOneTimerPlayerStats> = HashMap::new();
    let mut team_zero = OneTimerTeamStats::default();
    let mut team_one = OneTimerTeamStats::default();
    let mut last_one_timer_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(
                frame,
                frame.is_live_play && last_one_timer_player.as_ref() == Some(player_id),
            );
        }

        if !frame.is_live_play {
            last_one_timer_player = None;
        } else {
            let mut processed_event = false;
            while event_index < events.len() && events[event_index].frame <= frame.frame_number {
                let event = &events[event_index];
                players
                    .entry(event.player.clone())
                    .or_default()
                    .record_event(event, frame);
                let team_stats = if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                };
                team_stats.count += 1;
                team_stats.total_ball_speed += event.ball_speed;
                team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);
                last_one_timer_player = Some(event.player.clone());
                event_index += 1;
                processed_event = true;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.stats.is_last_one_timer = false;
                }
            }

            if let Some(player_id) = last_one_timer_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.stats.is_last_one_timer = true;
                }
            }
        }

        assert_one_timer_team_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.one_timer,
            &team_zero,
        );
        assert_one_timer_team_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.one_timer,
            &team_one,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_one_timer_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.one_timer,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed one-timer events"
    );
}

#[derive(Clone, Default)]
struct DerivedHalfVolleyPlayerStats {
    stats: HalfVolleyPlayerStats,
}

impl DerivedHalfVolleyPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_half_volley_player: bool) {
        self.stats.is_last_half_volley = is_last_half_volley_player;
        self.stats.time_since_last_half_volley = self
            .stats
            .last_half_volley_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_half_volley = self
            .stats
            .last_half_volley_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &HalfVolleyEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.total_ball_speed += event.ball_speed;
        self.stats.fastest_ball_speed = self.stats.fastest_ball_speed.max(event.ball_speed);
        self.stats.last_half_volley_time = Some(event.time);
        self.stats.last_half_volley_frame = Some(event.frame);
        self.stats.time_since_last_half_volley = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_half_volley =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_half_volley_derived_player_stats_match(
    scope: &str,
    actual: &HalfVolleyPlayerStats,
    expected: &HalfVolleyPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} half_volley.count");
    assert!(
        (actual.total_ball_speed - expected.total_ball_speed).abs() < 0.001,
        "{scope} half_volley.total_ball_speed: actual {:.3} expected {:.3}",
        actual.total_ball_speed,
        expected.total_ball_speed,
    );
    assert!(
        (actual.fastest_ball_speed - expected.fastest_ball_speed).abs() < 0.001,
        "{scope} half_volley.fastest_ball_speed: actual {:.3} expected {:.3}",
        actual.fastest_ball_speed,
        expected.fastest_ball_speed,
    );
    assert_eq!(
        actual.is_last_half_volley, expected.is_last_half_volley,
        "{scope} half_volley.is_last_half_volley"
    );
    assert_eq!(
        actual.last_half_volley_frame, expected.last_half_volley_frame,
        "{scope} half_volley.last_half_volley_frame"
    );
    assert!(
        match (actual.last_half_volley_time, expected.last_half_volley_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_volley.last_half_volley_time: actual {:?} expected {:?}",
        actual.last_half_volley_time,
        expected.last_half_volley_time,
    );
    assert_eq!(
        actual.frames_since_last_half_volley, expected.frames_since_last_half_volley,
        "{scope} half_volley.frames_since_last_half_volley"
    );
    assert!(
        match (
            actual.time_since_last_half_volley,
            expected.time_since_last_half_volley,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_volley.time_since_last_half_volley: actual {:?} expected {:?}",
        actual.time_since_last_half_volley,
        expected.time_since_last_half_volley,
    );
}

fn assert_half_volley_team_stats_match(
    scope: &str,
    actual: &HalfVolleyTeamStats,
    expected: &HalfVolleyTeamStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} half_volley.count");
    assert!(
        (actual.total_ball_speed - expected.total_ball_speed).abs() < 0.001,
        "{scope} half_volley.total_ball_speed: actual {:.3} expected {:.3}",
        actual.total_ball_speed,
        expected.total_ball_speed,
    );
    assert!(
        (actual.fastest_ball_speed - expected.fastest_ball_speed).abs() < 0.001,
        "{scope} half_volley.fastest_ball_speed: actual {:.3} expected {:.3}",
        actual.fastest_ball_speed,
        expected.fastest_ball_speed,
    );
}

fn assert_half_volley_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.half_volley.clone();
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedHalfVolleyPlayerStats> = HashMap::new();
    let mut team_zero = HalfVolleyTeamStats::default();
    let mut team_one = HalfVolleyTeamStats::default();
    let mut last_half_volley_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(
                frame,
                frame.is_live_play && last_half_volley_player.as_ref() == Some(player_id),
            );
        }

        if !frame.is_live_play {
            last_half_volley_player = None;
        } else {
            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                players
                    .entry(event.player.clone())
                    .or_default()
                    .record_event(event, frame);
                let team_stats = if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                };
                team_stats.count += 1;
                team_stats.total_ball_speed += event.ball_speed;
                team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);
                last_half_volley_player = Some(event.player.clone());
                event_index += 1;
                processed_event = true;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.stats.is_last_half_volley = false;
                }
            }

            if let Some(player_id) = last_half_volley_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.stats.is_last_half_volley = true;
                }
            }
        }

        assert_half_volley_team_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.half_volley,
            &team_zero,
        );
        assert_half_volley_team_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.half_volley,
            &team_one,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_half_volley_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.half_volley,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed half-volley events"
    );
}

#[test]
fn test_stats_timeline_frame_lookup_uses_frame_number() {
    let timeline = ReplayStatsTimeline {
        config: StatsTimelineConfig {
            most_back_forward_threshold_y: PositioningCalculatorConfig::default()
                .most_back_forward_threshold_y,
            level_ball_depth_margin: PositioningCalculatorConfig::default().level_ball_depth_margin,
            pressure_neutral_zone_half_width_y: PressureCalculatorConfig::default()
                .neutral_zone_half_width_y,
            territorial_pressure_neutral_zone_half_width_y:
                TerritorialPressureCalculatorConfig::default().neutral_zone_half_width_y,
            territorial_pressure_min_establish_seconds:
                TerritorialPressureCalculatorConfig::default().min_establish_seconds,
            territorial_pressure_min_establish_third_seconds:
                TerritorialPressureCalculatorConfig::default().min_establish_third_seconds,
            territorial_pressure_relief_grace_seconds:
                TerritorialPressureCalculatorConfig::default().relief_grace_seconds,
            territorial_pressure_confirmed_relief_grace_seconds:
                TerritorialPressureCalculatorConfig::default().confirmed_relief_grace_seconds,
            rotation_role_depth_margin: RotationCalculatorConfig::default().role_depth_margin,
            rotation_first_man_ambiguity_margin: RotationCalculatorConfig::default()
                .first_man_ambiguity_margin,
            rotation_first_man_debounce_seconds: RotationCalculatorConfig::default()
                .first_man_debounce_seconds,
            rush_max_start_y: RushCalculatorConfig::default().max_start_y,
            rush_attack_support_distance_y: RushCalculatorConfig::default()
                .attack_support_distance_y,
            rush_defender_distance_y: RushCalculatorConfig::default().defender_distance_y,
            rush_min_possession_retained_seconds: RushCalculatorConfig::default()
                .min_possession_retained_seconds,
            aerial_goal_min_ball_z: AerialGoalCalculatorConfig::default().min_ball_z,
            high_aerial_goal_min_ball_z: HighAerialGoalCalculatorConfig::default().min_ball_z,
            long_distance_goal_max_attacking_y: LongDistanceGoalCalculatorConfig::default()
                .max_attacking_y,
            own_half_goal_max_attacking_y: OwnHalfGoalCalculatorConfig::default().max_attacking_y,
            empty_net_min_defender_y_margin: EmptyNetGoalCalculatorConfig::default()
                .min_defender_y_margin,
            empty_net_min_defender_distance: EmptyNetGoalCalculatorConfig::default()
                .min_defender_distance,
            empty_net_max_touch_attacking_y: EmptyNetGoalCalculatorConfig::default()
                .max_touch_attacking_y,
            flick_goal_max_event_to_goal_seconds: FlickGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            double_tap_goal_max_event_to_goal_seconds: DoubleTapGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            one_timer_goal_max_event_to_goal_seconds: OneTimerGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            air_dribble_goal_max_end_to_goal_seconds: AirDribbleGoalCalculatorConfig::default()
                .max_end_to_goal_seconds,
            flip_reset_goal_max_event_to_goal_seconds: FlipResetGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            half_volley_max_bounce_to_touch_seconds: HalfVolleyCalculatorConfig::default()
                .max_bounce_to_touch_seconds,
            half_volley_min_ball_speed: HalfVolleyCalculatorConfig::default().min_ball_speed,
            half_volley_goal_max_touch_to_goal_seconds: HalfVolleyGoalCalculatorConfig::default()
                .max_touch_to_goal_seconds,
            half_volley_goal_min_goal_alignment: HalfVolleyGoalCalculatorConfig::default()
                .min_goal_alignment,
        },
        replay_meta: ReplayMeta {
            team_zero: Vec::new(),
            team_one: Vec::new(),
            all_headers: Vec::new(),
        },
        events: ReplayStatsTimelineEvents {
            timeline: Vec::new(),
            core_player: Vec::new(),
            core_team: Vec::new(),
            possession: Vec::new(),
            pressure: Vec::new(),
            territorial_pressure: Vec::new(),
            movement: Vec::new(),
            positioning: Vec::new(),
            rotation_player: Vec::new(),
            rotation_team: Vec::new(),
            mechanics: Vec::new(),
            goal_context: Vec::new(),
            backboard: Vec::new(),
            ceiling_shot: Vec::new(),
            wall_aerial: Vec::new(),
            wall_aerial_shot: Vec::new(),
            center: Vec::new(),
            flick: Vec::new(),
            musty_flick: Vec::new(),
            dodge_reset: Vec::new(),
            double_tap: Vec::new(),
            fifty_fifty: Vec::new(),
            one_timer: Vec::new(),
            pass: Vec::new(),
            pass_last_completed: Vec::new(),
            ball_carry: Vec::new(),
            goal_tags: Vec::new(),
            rush: Vec::new(),
            speed_flip: Vec::new(),
            half_flip: Vec::new(),
            half_volley: Vec::new(),
            wavedash: Vec::new(),
            whiff: Vec::new(),
            powerslide: Vec::new(),
            touch: Vec::new(),
            touch_ball_movement: Vec::new(),
            touch_last_touch: Vec::new(),
            boost_pickups: Vec::new(),
            boost_ledger: Vec::new(),
            boost_state: Vec::new(),
            bump: Vec::new(),
        },
        frames: vec![
            ReplayStatsFrame {
                frame_number: 10,
                time: 0.0,
                dt: 0.0,
                seconds_remaining: None,
                game_state: None,
                ball_has_been_hit: None,
                kickoff_countdown_time: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
                players: Vec::new(),
            },
            ReplayStatsFrame {
                frame_number: 11,
                time: 0.1,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                ball_has_been_hit: None,
                kickoff_countdown_time: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
                players: Vec::new(),
            },
            ReplayStatsFrame {
                frame_number: 15,
                time: 0.2,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                ball_has_been_hit: None,
                kickoff_countdown_time: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
                players: Vec::new(),
            },
        ],
    };

    assert_eq!(timeline.frames[2].frame_number, 15);
    assert_eq!(timeline.frame_by_number(2), None);
    assert_eq!(
        timeline
            .frame_by_number(15)
            .expect("Expected frame lookup by frame number")
            .frame_number,
        15
    );
}

#[test]
fn test_stats_timeline_collector_final_frame_matches_analysis_graph() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");
    let final_frame = timeline.frames.last().expect("Expected at least one frame");

    let graph = stats::analysis_graph::collect_builtin_analysis_graph_for_replay(
        &replay,
        [
            "fifty_fifty",
            "possession",
            "pressure",
            "rush",
            "core",
            "backboard",
            "double_tap",
            "ball_carry",
            "boost",
            "movement",
            "positioning",
            "powerslide",
            "demo",
        ],
    )
    .expect("Expected analysis graph to process replay");

    let possession = graph
        .state::<PossessionCalculator>()
        .expect("missing possession calculator state");
    let fifty_fifty = graph
        .state::<FiftyFiftyCalculator>()
        .expect("missing fifty_fifty calculator state");
    let pressure = graph
        .state::<PressureCalculator>()
        .expect("missing pressure calculator state");
    let rush = graph
        .state::<RushCalculator>()
        .expect("missing rush calculator state");
    let match_stats = graph
        .state::<MatchStatsCalculator>()
        .expect("missing match stats calculator state");
    let backboard = graph
        .state::<BackboardCalculator>()
        .expect("missing backboard calculator state");
    let double_tap = graph
        .state::<DoubleTapCalculator>()
        .expect("missing double tap calculator state");
    let ball_carry = graph
        .state::<BallCarryCalculator>()
        .expect("missing ball carry calculator state");
    let boost = graph
        .state::<BoostCalculator>()
        .expect("missing boost calculator state");
    let movement = graph
        .state::<MovementCalculator>()
        .expect("missing movement calculator state");
    let positioning = graph
        .state::<PositioningCalculator>()
        .expect("missing positioning calculator state");
    let powerslide = graph
        .state::<PowerslideCalculator>()
        .expect("missing powerslide calculator state");
    let demo = graph
        .state::<DemoCalculator>()
        .expect("missing demo calculator state");

    let assert_core_team_stats_match =
        |actual: &CoreTeamStats, expected: &CoreTeamStats, team_label: &str| {
            assert_eq!(actual.score, expected.score, "{team_label} score");
            assert_eq!(actual.goals, expected.goals, "{team_label} goals");
            assert_eq!(actual.assists, expected.assists, "{team_label} assists");
            assert_eq!(actual.saves, expected.saves, "{team_label} saves");
            assert_eq!(actual.shots, expected.shots, "{team_label} shots");
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.kickoff_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .kickoff_goal_count,
                "{team_label} kickoff-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.short_goal_count,
                expected.scoring_context.goal_after_kickoff.short_goal_count,
                "{team_label} short-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.medium_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .medium_goal_count,
                "{team_label} medium-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.long_goal_count,
                expected.scoring_context.goal_after_kickoff.long_goal_count,
                "{team_label} long-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_buildup, expected.scoring_context.goal_buildup,
                "{team_label} goal buildup classifications",
            );
        };

    let assert_core_player_stats_match =
        |actual: &CorePlayerStats, expected: &CorePlayerStats, player_label: &str| {
            assert_eq!(actual.score, expected.score, "{player_label} score");
            assert_eq!(actual.goals, expected.goals, "{player_label} goals");
            assert_eq!(actual.assists, expected.assists, "{player_label} assists");
            assert_eq!(actual.saves, expected.saves, "{player_label} saves");
            assert_eq!(actual.shots, expected.shots, "{player_label} shots");
            assert_eq!(
                actual.scoring_context.goals_conceded_while_last_defender,
                expected.scoring_context.goals_conceded_while_last_defender,
                "{player_label} last-defender concessions",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.kickoff_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .kickoff_goal_count,
                "{player_label} kickoff-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.short_goal_count,
                expected.scoring_context.goal_after_kickoff.short_goal_count,
                "{player_label} short-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.medium_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .medium_goal_count,
                "{player_label} medium-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.long_goal_count,
                expected.scoring_context.goal_after_kickoff.long_goal_count,
                "{player_label} long-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_buildup, expected.scoring_context.goal_buildup,
                "{player_label} goal buildup classifications",
            );
        };

    assert_eq!(
        final_frame.team_zero.fifty_fifty,
        fifty_fifty.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.fifty_fifty,
        fifty_fifty.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.possession,
        possession.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.possession,
        possession.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.pressure,
        pressure.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.pressure,
        pressure.stats().for_team(false)
    );
    assert_eq!(final_frame.team_zero.rush, rush.stats().for_team(true));
    assert_eq!(final_frame.team_one.rush, rush.stats().for_team(false));
    assert_core_team_stats_match(
        &final_frame.team_zero.core,
        &match_stats.team_zero_stats(),
        "team zero",
    );
    assert_core_team_stats_match(
        &final_frame.team_one.core,
        &match_stats.team_one_stats(),
        "team one",
    );
    assert_eq!(
        final_frame.team_zero.ball_carry,
        ball_carry.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.air_dribble,
        ball_carry.team_zero_air_dribble_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.backboard,
        backboard.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.backboard,
        backboard.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.double_tap,
        double_tap.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.double_tap,
        double_tap.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.ball_carry,
        ball_carry.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.air_dribble,
        ball_carry.team_one_air_dribble_stats().clone()
    );
    assert_eq!(final_frame.team_zero.boost, boost.team_zero_stats().clone());
    assert_eq!(final_frame.team_one.boost, boost.team_one_stats().clone());
    assert_eq!(
        final_frame.team_zero.movement,
        movement.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.movement,
        movement.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.powerslide,
        powerslide.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.powerslide,
        powerslide.team_one_stats().clone()
    );
    assert_eq!(final_frame.team_zero.demo, demo.team_zero_stats().clone());
    assert_eq!(final_frame.team_one.demo, demo.team_one_stats().clone());

    assert_eq!(
        final_frame.players.len(),
        timeline.replay_meta.player_count()
    );
    for player in &final_frame.players {
        assert_core_player_stats_match(
            &player.core,
            &match_stats
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default(),
            &player.name,
        );
        assert_eq!(
            player.ball_carry,
            ball_carry
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.air_dribble,
            ball_carry
                .player_air_dribble_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.backboard,
            backboard
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.double_tap,
            double_tap
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.boost,
            boost
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            complete_movement_breakdowns_for_comparison(&player.movement),
            movement
                .player_stats()
                .get(&player.player_id)
                .map(complete_movement_breakdowns_for_comparison)
                .unwrap_or_else(|| {
                    complete_movement_breakdowns_for_comparison(&MovementStats::default())
                })
        );
        assert_eq!(
            player.positioning,
            positioning
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.powerslide,
            powerslide
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.demo,
            demo.player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
    }
    assert_eq!(timeline.events.backboard, backboard.events());
    assert_eq!(timeline.events.double_tap, double_tap.events());
}

fn assert_boost_ledger_reconstructs_serialized_boost_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut ledger_events = timeline.events.boost_ledger.clone();
    ledger_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut state_events = timeline.events.boost_state.clone();
    state_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut ledger_event_index = 0;
    let mut state_event_index = 0;
    let mut players: HashMap<PlayerId, DerivedBoostLedgerStats> = HashMap::new();
    let mut team_zero = DerivedBoostLedgerStats::default();
    let mut team_one = DerivedBoostLedgerStats::default();

    for frame in &timeline.frames {
        let mut state_event_players_this_frame = Vec::new();
        while state_event_index < state_events.len()
            && state_events[state_event_index].frame <= frame.frame_number
        {
            let event = &state_events[state_event_index];
            apply_boost_state_event(players.entry(event.player_id.clone()).or_default(), event);
            if event.frame == frame.frame_number {
                state_event_players_this_frame.push((event.player_id.clone(), event.is_team_0));
            }
            state_event_index += 1;
        }
        while ledger_event_index < ledger_events.len()
            && ledger_events[ledger_event_index].frame <= frame.frame_number
        {
            let event = &ledger_events[ledger_event_index];
            apply_boost_ledger_event(players.entry(event.player_id.clone()).or_default(), event);
            apply_boost_ledger_event(
                if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            ledger_event_index += 1;
        }

        for (player_id, is_team_0) in state_event_players_this_frame {
            let player_stats = players.entry(player_id).or_default();
            let Some((previous_boost_amount, boost_amount)) =
                apply_boost_state_sample(player_stats, frame.dt, frame.frame_number)
            else {
                continue;
            };
            add_boost_state_sample(
                if is_team_0 {
                    &mut team_zero.stats
                } else {
                    &mut team_one.stats
                },
                previous_boost_amount,
                boost_amount,
                frame.dt,
            );
        }

        assert_boost_ledger_derived_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.boost,
            &team_zero.stats,
        );
        assert_boost_ledger_derived_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.boost,
            &team_one.stats,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).map(|stats| &stats.stats);
            let default_stats = BoostStats::default();
            assert_boost_ledger_derived_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.boost,
                expected.unwrap_or(&default_stats),
            );
        }
    }
    assert_eq!(
        ledger_event_index,
        ledger_events.len(),
        "{replay_path} unprocessed boost ledger events"
    );
    assert_eq!(
        state_event_index,
        state_events.len(),
        "{replay_path} unprocessed boost state events"
    );
}

#[test]
fn test_boost_ledger_reconstructs_serialized_boost_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");
    assert!(
        !timeline.events.boost_ledger.is_empty(),
        "expected boost ledger events to be emitted"
    );
    assert!(
        !timeline.events.boost_state.is_empty(),
        "expected boost state events to be emitted"
    );
    assert_boost_ledger_reconstructs_serialized_boost_partial_sums(replay_path, &timeline);
}

#[test]
fn test_mechanic_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
    ];
    let mut saw_half_flip_event = false;
    let mut saw_wavedash_event = false;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));

        if timeline.events.half_flip.is_empty() && timeline.events.wavedash.is_empty() {
            continue;
        }

        assert_quality_mechanic_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
        saw_half_flip_event |= !timeline.events.half_flip.is_empty();
        saw_wavedash_event |= !timeline.events.wavedash.is_empty();

        if saw_half_flip_event && saw_wavedash_event {
            break;
        }
    }

    assert!(
        saw_half_flip_event,
        "expected at least one fixture to contain a half-flip event"
    );
    assert!(
        saw_wavedash_event,
        "expected at least one fixture to contain a wavedash event"
    );
}

#[test]
fn test_speed_flip_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.speed_flip.is_empty(),
        "expected speed-flip fixture to contain speed-flip events"
    );
    assert_speed_flip_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_whiff_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.whiff.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain whiff events");
    assert_whiff_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_backboard_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.backboard.is_empty(),
        "expected backboard fixture to contain backboard events"
    );
    assert_backboard_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_double_tap_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.double_tap.is_empty(),
        "expected double-tap fixture to contain double-tap events"
    );
    assert_double_tap_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_one_timer_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.one_timer.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain one-timer events");
    assert_one_timer_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_pass_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.pass.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain pass events");
    assert_pass_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_rush_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.rush.is_empty(),
        "expected rush fixture to contain rush events"
    );
    assert_rush_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_bump_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
        "assets/post-eac-ranked-standard-2026-04-28.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.bump.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain bump events");
    assert_bump_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_demo_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline.events.timeline.iter().any(|event| matches!(
            event.kind,
            TimelineEventKind::Kill | TimelineEventKind::Death
        )),
        "expected demo fixture to contain kill/death timeline events"
    );
    assert_demo_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_fifty_fifty_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.fifty_fifty.is_empty(),
        "expected fifty-fifty fixture to contain fifty-fifty events"
    );
    assert_fifty_fifty_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_half_volley_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
        "assets/air-dribble-goal-mouth-2026-05-24.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.half_volley.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain half-volley events");
    assert_half_volley_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_ball_carry_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/air-dribble-goal-mouth-2026-05-24.replay",
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.ball_carry.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain ball-carry events");
    assert_ball_carry_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_wall_aerial_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/air-dribble-goal-mouth-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.wall_aerial.is_empty(),
        "expected wall-aerial fixture to contain wall-aerial events"
    );
    assert_wall_aerial_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_wall_aerial_shot_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/air-dribble-goal-mouth-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.wall_aerial_shot.is_empty(),
        "expected wall-aerial fixture to contain wall-aerial-shot events"
    );
    assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_flick_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.flick.is_empty(),
        "expected flick fixture to contain flick events"
    );
    assert_flick_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_musty_flick_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.musty_flick.is_empty(),
        "expected musty-flick fixture to contain musty-flick events"
    );
    assert_musty_flick_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_dodge_reset_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.dodge_reset.is_empty(),
        "expected dodge-reset fixture to contain dodge-reset events"
    );
    assert!(
        timeline
            .events
            .dodge_reset
            .iter()
            .any(|event| event.on_ball),
        "expected dodge-reset fixture to contain on-ball dodge-reset events"
    );
    assert_dodge_reset_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_powerslide_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.powerslide.is_empty(),
        "expected powerslide fixture to contain powerslide events"
    );
    assert_powerslide_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_touch_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.touch.is_empty(),
        "expected touch fixture to contain touch events"
    );
    assert!(
        !timeline.events.touch_ball_movement.is_empty(),
        "expected touch fixture to contain ball movement credit events"
    );
    assert_touch_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_core_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.core_player.is_empty(),
        "expected core fixture to contain player stat events"
    );
    assert!(
        !timeline.events.core_team.is_empty(),
        "expected core fixture to contain team stat events"
    );
    assert_core_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_possession_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.possession.is_empty(),
        "expected possession fixture to contain possession events"
    );
    assert_possession_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_pressure_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.pressure.is_empty(),
        "expected pressure fixture to contain pressure events"
    );
    assert_pressure_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_movement_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.movement.is_empty(),
        "expected movement fixture to contain movement events"
    );
    assert_movement_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_positioning_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.positioning.is_empty(),
        "expected positioning fixture to contain positioning events"
    );
    assert_positioning_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_rotation_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.rotation_player.is_empty(),
        "expected rotation fixture to contain rotation player events"
    );
    assert_rotation_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

fn assert_converted_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    assert_boost_ledger_reconstructs_serialized_boost_partial_sums(replay_path, timeline);
    assert_core_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_possession_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_pressure_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_movement_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_positioning_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_rotation_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_quality_mechanic_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_speed_flip_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_whiff_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_backboard_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_double_tap_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_demo_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_fifty_fifty_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_bump_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_rush_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_pass_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_one_timer_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_ball_carry_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_wall_aerial_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_flick_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_ceiling_shot_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_musty_flick_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_dodge_reset_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_powerslide_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_touch_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_half_volley_events_reconstruct_serialized_partial_sums(replay_path, timeline);
}

fn assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths: Vec<String>) {
    for replay_path in replay_paths {
        eprintln!("checking {replay_path}");
        let replay = parse_replay(&replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        assert_converted_events_reconstruct_serialized_partial_sums(&replay_path, &timeline);
    }
}

#[test]
#[ignore = "wide replay-format parity is slow; run explicitly when changing compact timeline derivation"]
fn replay_format_fixture_events_reconstruct_serialized_partial_sums() {
    let replay_paths = replay_format_fixture_paths();
    assert!(
        !replay_paths.is_empty(),
        "expected replay-format docs to list checked-in fixtures"
    );
    assert!(
        std::env::var("SUBTR_ACTOR_REPLAY_FORMAT_FIXTURE").is_ok() || replay_paths.len() >= 10,
        "expected replay-format docs to list checked-in fixtures"
    );

    assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths);
}

#[test]
#[ignore = "all replay asset event parity is slow; run explicitly before removing transferred partial sums"]
fn all_asset_fixture_events_reconstruct_serialized_partial_sums() {
    let replay_paths = asset_replay_fixture_paths();
    assert!(
        !replay_paths.is_empty(),
        "expected checked-in replay asset fixtures"
    );
    assert!(
        std::env::var("SUBTR_ACTOR_REPLAY_FIXTURE").is_ok() || replay_paths.len() >= 20,
        "expected broad replay fixture coverage"
    );

    assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths);
}

#[test]
fn test_ceiling_shot_events_reconstruct_serialized_partial_sums() {
    let blue_player = PlayerId::Steam(1001);
    let orange_player = PlayerId::Steam(2002);
    let blue_event = CeilingShotEvent {
        time: 2.0,
        frame: 20,
        player: blue_player.clone(),
        is_team_0: true,
        ceiling_contact_time: 1.2,
        ceiling_contact_frame: 12,
        time_since_ceiling_contact: 0.8,
        ceiling_contact_position: [0.0, 0.0, 2040.0],
        touch_position: [500.0, 100.0, 520.0],
        local_ball_position: [120.0, 10.0, 30.0],
        separation_from_ceiling: 460.0,
        roof_alignment: 0.9,
        forward_alignment: 0.8,
        forward_approach_speed: 700.0,
        ball_speed_change: 500.0,
        confidence: 0.82,
    };
    let orange_event = CeilingShotEvent {
        time: 3.0,
        frame: 30,
        player: orange_player.clone(),
        is_team_0: false,
        ceiling_contact_time: 2.4,
        ceiling_contact_frame: 24,
        time_since_ceiling_contact: 0.6,
        ceiling_contact_position: [0.0, 0.0, 2040.0],
        touch_position: [-400.0, -200.0, 480.0],
        local_ball_position: [100.0, -20.0, 20.0],
        separation_from_ceiling: 520.0,
        roof_alignment: 0.85,
        forward_alignment: 0.7,
        forward_approach_speed: 650.0,
        ball_speed_change: 350.0,
        confidence: 0.7,
    };

    let mut blue_after_event = CeilingShotStats::default();
    blue_after_event
        .labeled_event_counts
        .increment([confidence_band_label_for_derivation(true)]);
    blue_after_event.count = 1;
    blue_after_event.high_confidence_count = 1;
    blue_after_event.is_last_ceiling_shot = true;
    blue_after_event.last_ceiling_shot_time = Some(2.0);
    blue_after_event.last_ceiling_shot_frame = Some(20);
    blue_after_event.time_since_last_ceiling_shot = Some(0.0);
    blue_after_event.frames_since_last_ceiling_shot = Some(0);
    blue_after_event.last_confidence = Some(0.82);
    blue_after_event.best_confidence = 0.82;
    blue_after_event.cumulative_confidence = 0.82;

    let mut blue_after_reset = blue_after_event.clone();
    blue_after_reset.is_last_ceiling_shot = false;
    blue_after_reset.time_since_last_ceiling_shot = Some(1.0);
    blue_after_reset.frames_since_last_ceiling_shot = Some(10);

    let mut orange_after_event = CeilingShotStats::default();
    orange_after_event
        .labeled_event_counts
        .increment([confidence_band_label_for_derivation(false)]);
    orange_after_event.count = 1;
    orange_after_event.high_confidence_count = 0;
    orange_after_event.is_last_ceiling_shot = true;
    orange_after_event.last_ceiling_shot_time = Some(3.0);
    orange_after_event.last_ceiling_shot_frame = Some(30);
    orange_after_event.time_since_last_ceiling_shot = Some(0.0);
    orange_after_event.frames_since_last_ceiling_shot = Some(0);
    orange_after_event.last_confidence = Some(0.7);
    orange_after_event.best_confidence = 0.7;
    orange_after_event.cumulative_confidence = 0.7;

    let frame = |frame_number: usize,
                 time: f32,
                 is_live_play: bool,
                 blue_stats: CeilingShotStats,
                 orange_stats: CeilingShotStats| {
        let mut blue = default_player_stats_snapshot(blue_player.clone(), "Blue ceiling", true);
        blue.ceiling_shot = blue_stats;
        let mut orange =
            default_player_stats_snapshot(orange_player.clone(), "Orange ceiling", false);
        orange.ceiling_shot = orange_stats;
        ReplayStatsFrame {
            frame_number,
            time,
            dt: 0.5,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            gameplay_phase: if is_live_play {
                GameplayPhase::ActivePlay
            } else {
                GameplayPhase::PostGoal
            },
            is_live_play,
            team_zero: default_team_stats_snapshot(),
            team_one: default_team_stats_snapshot(),
            players: vec![blue, orange],
        }
    };

    let timeline = ReplayStatsTimeline {
        config: empty_stats_timeline_config(),
        replay_meta: ReplayMeta {
            team_zero: Vec::new(),
            team_one: Vec::new(),
            all_headers: Vec::new(),
        },
        events: ReplayStatsTimelineEvents {
            ceiling_shot: vec![blue_event, orange_event],
            ..Default::default()
        },
        frames: vec![
            frame(
                20,
                2.0,
                true,
                blue_after_event.clone(),
                CeilingShotStats::default(),
            ),
            frame(
                25,
                2.5,
                false,
                blue_after_event.clone(),
                CeilingShotStats::default(),
            ),
            frame(30, 3.0, true, blue_after_reset, orange_after_event),
        ],
    };

    assert_ceiling_shot_events_reconstruct_serialized_partial_sums("synthetic", &timeline);
}

#[test]
fn test_stats_timeline_collector_frames_are_sorted_and_cumulative() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.frames.is_empty(),
        "Expected stats timeline frames"
    );
    assert!(
        timeline
            .frames
            .windows(2)
            .all(|frames| frames[1].time >= frames[0].time),
        "Expected frame times to be nondecreasing"
    );
    assert!(
        timeline
            .frames
            .windows(2)
            .all(|frames| frames[1].team_zero.core.goals >= frames[0].team_zero.core.goals),
        "Expected team-zero goals to accumulate monotonically"
    );
    assert!(
        timeline.frames.windows(2).all(|frames| {
            frames[1].team_zero.boost.amount_collected >= frames[0].team_zero.boost.amount_collected
        }),
        "Expected collected boost to accumulate monotonically"
    );
    assert!(
        timeline
            .events
            .timeline
            .windows(2)
            .all(|events| events[1].time >= events[0].time),
        "Expected emitted timeline events to be time sorted"
    );
    assert_boost_pickup_nominal_amounts_consistent(&timeline);
}

#[test]
fn test_stats_timeline_value_serializes_for_rlcs_replay() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let captured = StatsCollector::new()
        .capture_frames()
        .get_captured_data(&replay)
        .expect("Expected captured stats data");

    captured
        .into_legacy_stats_timeline_value()
        .expect("Expected stats timeline value");
}

#[test]
fn test_stats_timeline_excludes_post_goal_reset_frames_from_cumulative_stats() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Expected replay data");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let first_goal = replay_data
        .goal_events
        .first()
        .expect("Expected at least one goal event");
    let kickoff_countdown_start = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .skip(first_goal.frame + 1)
        .find(|metadata| metadata.replicated_game_state_time_remaining > 0)
        .map(|metadata| metadata.time)
        .expect("Expected a kickoff countdown after the first goal");
    let post_goal_frames: Vec<_> = timeline
        .frames
        .iter()
        .filter(|frame| frame.time >= first_goal.time && frame.time < kickoff_countdown_start)
        .collect();

    assert!(
        post_goal_frames.len() > 1,
        "Expected multiple frames between the goal and the next kickoff countdown"
    );
    assert!(
        post_goal_frames.iter().all(|frame| !frame.is_live_play),
        "Expected all post-goal reset frames to be inactive"
    );

    let first_post_goal_frame = post_goal_frames
        .first()
        .expect("Expected a first post-goal frame");
    let last_post_goal_frame = post_goal_frames
        .last()
        .expect("Expected a last post-goal frame");

    assert_eq!(
        frame_total_goals(first_post_goal_frame),
        frame_total_goals(last_post_goal_frame),
        "Expected the score to stay fixed throughout the post-goal reset"
    );
    assert_eq!(
        last_post_goal_frame.team_zero.possession,
        first_post_goal_frame.team_zero.possession
    );
    assert_eq!(
        last_post_goal_frame.team_one.possession,
        first_post_goal_frame.team_one.possession
    );
    assert_eq!(
        normalized_team_stats_for_live_play_comparison(&last_post_goal_frame.team_zero),
        normalized_team_stats_for_live_play_comparison(&first_post_goal_frame.team_zero),
    );
    assert_eq!(
        normalized_team_stats_for_live_play_comparison(&last_post_goal_frame.team_one),
        normalized_team_stats_for_live_play_comparison(&first_post_goal_frame.team_one),
    );
    let normalized_last_players: Vec<_> = last_post_goal_frame
        .players
        .iter()
        .map(normalized_player_stats_for_live_play_comparison)
        .collect();
    let normalized_first_players: Vec<_> = first_post_goal_frame
        .players
        .iter()
        .map(normalized_player_stats_for_live_play_comparison)
        .collect();
    assert_eq!(normalized_last_players, normalized_first_players);
}

#[test]
fn test_stats_timeline_old_replay_with_substitutions_discovers_late_players() {
    let replay = parse_replay("assets/old-ballchasing-midfield-car.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");
    let final_frame = timeline.frames.last().expect("Expected timeline frames");
    let names = player_names(final_frame);

    for expected_name in [
        "CritRomney",
        "DatLilBabyG",
        "b_corner",
        "Raptor_Attacks_",
        "jboy42069",
        "remrocker29",
        "a093q262",
        "Q-money219",
    ] {
        assert!(
            names.contains(expected_name),
            "Expected final stats timeline frame to include late player {expected_name}, got {names:?}"
        );
    }
}

#[test]
fn test_stats_timeline_boost_monotonic_dodges_replay() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    dump_final_boost_stats(&timeline);

    // Invariant 1: All cumulative boost stats must be monotonically non-decreasing
    type BoostStatGetter = fn(&BoostStats) -> f64;
    let monotonic_checks: &[(&str, BoostStatGetter)] = &[
        ("amount_collected", |b| b.amount_collected as f64),
        ("amount_collected_big", |b| b.amount_collected_big as f64),
        ("amount_collected_small", |b| {
            b.amount_collected_small as f64
        }),
        ("amount_respawned", |b| b.amount_respawned as f64),
        ("amount_used_while_grounded", |b| {
            b.amount_used_while_grounded as f64
        }),
        ("amount_used_while_airborne", |b| {
            b.amount_used_while_airborne as f64
        }),
        ("amount_stolen", |b| b.amount_stolen as f64),
        ("amount_stolen_big", |b| b.amount_stolen_big as f64),
        ("amount_stolen_small", |b| b.amount_stolen_small as f64),
        ("overfill_total", |b| b.overfill_total as f64),
        ("big_pads_collected", |b| b.big_pads_collected as f64),
        ("small_pads_collected", |b| b.small_pads_collected as f64),
        ("big_pads_stolen", |b| b.big_pads_stolen as f64),
        ("small_pads_stolen", |b| b.small_pads_stolen as f64),
        // NOTE: amount_used is NOT monotonic because kickoff resets set
        // current_boost to 85 without it being "used", lowering amount_used.
    ];
    for (name, getter) in monotonic_checks {
        assert_player_boost_field_monotonic(&timeline, name, *getter);
    }

    // Invariant 2: Bucket sums consistent (every frame)
    assert_boost_bucket_sums_consistent(&timeline);

    // Invariant 3: Respawns reasonable
    assert_boost_respawns_reasonable(&timeline, 3000.0);

    // Invariant 4: Pad counts match collected boost plus overfill
    assert_boost_pickup_nominal_amounts_consistent(&timeline);

    // Invariant 5: Boost accounting — implied current boost in [0, 255]
    assert_boost_accounting_consistent(&timeline);

    // Invariant 6: Every player got at least one kickoff respawn
    let last_frame = timeline.frames.last().unwrap();
    for player in &last_frame.players {
        assert!(
            player.boost.amount_respawned >= BOOST_KICKOFF_START_AMOUNT - 1.0,
            "Player {} has amount_respawned={:.1}, expected at least one kickoff ({:.0})",
            player.name,
            player.boost.amount_respawned,
            BOOST_KICKOFF_START_AMOUNT,
        );
    }

    // Diagnostic: count goals to verify kickoff count
    let goal_count = timeline
        .events
        .timeline
        .iter()
        .filter(|e| matches!(e.kind, TimelineEventKind::Goal))
        .count();
    let expected_kickoffs = goal_count + 1; // +1 for initial kickoff
    eprintln!("Goals: {goal_count}, expected kickoffs: {expected_kickoffs}");
    // Each player should have expected_kickoffs * 85 in respawns
    let expected_respawn = expected_kickoffs as f32 * 85.0;
    eprintln!("Expected respawn per player: {expected_respawn:.0}");
    // Check first frame's game state
    if let Some(first) = timeline.frames.first() {
        eprintln!(
            "First frame: number={}, time={:.3}, game_state={:?}, is_live={}",
            first.frame_number, first.time, first.game_state, first.is_live_play
        );
    }
}

#[test]
fn test_stats_timeline_awards_touch_for_on_ball_reset_in_dodges_replay() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let pre_reset_frame = timeline
        .frame_by_number(2114)
        .expect("Expected pre-reset frame in timeline");
    let reset_frame = timeline
        .frame_by_number(2117)
        .expect("Expected dodge-reset frame in timeline");
    let post_reset_window = (2115..=2118)
        .map(|frame_number| {
            timeline
                .frame_by_number(frame_number)
                .unwrap_or_else(|| panic!("Expected frame {frame_number} in timeline"))
        })
        .collect::<Vec<_>>();

    let pre_reset_player = player_snapshot_by_name(pre_reset_frame, "rayman ty");
    let reset_player = player_snapshot_by_name(reset_frame, "rayman ty");
    let max_touch_count_in_window = post_reset_window
        .iter()
        .map(|frame| {
            player_snapshot_by_name(frame, "rayman ty")
                .touch
                .touch_count
        })
        .max()
        .expect("Expected non-empty post-reset window");

    assert_eq!(
        reset_player.dodge_reset.on_ball_count,
        pre_reset_player.dodge_reset.on_ball_count + 1,
        "Expected rayman ty to get an on-ball reset by frame 2117"
    );
    assert!(
        max_touch_count_in_window > pre_reset_player.touch.touch_count,
        "Expected rayman ty's touch count to increase within frames 2115..=2118 after the on-ball reset, but it stayed at {}",
        pre_reset_player.touch.touch_count
    );
}

#[test]
fn test_stats_timeline_first_kickoff_credits_both_players() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let baseline_frame = timeline
        .frame_by_number(170)
        .expect("Expected baseline kickoff frame in timeline");
    let kickoff_resolution_frame = timeline
        .frame_by_number(205)
        .expect("Expected kickoff-resolution frame in timeline");

    let baseline_orange = player_snapshot_by_name(baseline_frame, "tykop");
    let baseline_blue = player_snapshot_by_name(baseline_frame, "mrtyzz.");
    let kickoff_resolution_orange = player_snapshot_by_name(kickoff_resolution_frame, "tykop");
    let kickoff_resolution_blue = player_snapshot_by_name(kickoff_resolution_frame, "mrtyzz.");

    assert!(
        kickoff_resolution_orange.touch.touch_count > baseline_orange.touch.touch_count,
        "Expected tykop to receive a credited touch during the first kickoff sequence by frame 205"
    );
    assert!(
        kickoff_resolution_blue.touch.touch_count > baseline_blue.touch.touch_count,
        "Expected mrtyzz. to receive a credited touch during the first kickoff sequence by frame 205"
    );
}
