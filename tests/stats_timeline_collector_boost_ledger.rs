use std::collections::HashMap;

use subtr_actor::*;

const TEST_BOOST_ZERO_BAND_RAW: f32 = 1.0;
const TEST_BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;

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

pub fn assert_boost_ledger_reconstructs_serialized_boost_partial_sums(
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
