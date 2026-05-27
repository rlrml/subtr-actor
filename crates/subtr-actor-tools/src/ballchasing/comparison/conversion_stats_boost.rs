use subtr_actor::BoostStats;

use super::super::super::comparable_types::ComparableBoostStats;
use super::raw_boost_amount_as_comparable_units;

pub(crate) fn comparable_boost_from_stats(stats: &BoostStats) -> ComparableBoostStats {
    ComparableBoostStats {
        bpm: Some(raw_boost_amount_as_comparable_units(stats.bpm())),
        avg_amount: Some(raw_boost_amount_as_comparable_units(
            stats.average_boost_amount(),
        )),
        amount_collected: Some(raw_boost_amount_as_comparable_units(stats.amount_collected)),
        amount_stolen: Some(raw_boost_amount_as_comparable_units(stats.amount_stolen)),
        amount_collected_big: Some(raw_boost_amount_as_comparable_units(
            stats.amount_collected_big,
        )),
        amount_stolen_big: Some(raw_boost_amount_as_comparable_units(
            stats.amount_stolen_big,
        )),
        amount_collected_small: Some(raw_boost_amount_as_comparable_units(
            stats.amount_collected_small,
        )),
        amount_stolen_small: Some(raw_boost_amount_as_comparable_units(
            stats.amount_stolen_small,
        )),
        count_collected_big: Some(stats.big_pads_collected as f64),
        count_stolen_big: Some(stats.big_pads_stolen as f64),
        count_collected_small: Some(stats.small_pads_collected as f64),
        count_stolen_small: Some(stats.small_pads_stolen as f64),
        amount_overfill: Some(raw_boost_amount_as_comparable_units(stats.overfill_total)),
        amount_overfill_stolen: Some(raw_boost_amount_as_comparable_units(
            stats.overfill_from_stolen,
        )),
        amount_used_while_supersonic: Some(raw_boost_amount_as_comparable_units(
            stats.amount_used_while_supersonic,
        )),
        time_zero_boost: Some(stats.time_zero_boost as f64),
        percent_zero_boost: Some(stats.zero_boost_pct() as f64),
        time_full_boost: Some(stats.time_hundred_boost as f64),
        percent_full_boost: Some(stats.hundred_boost_pct() as f64),
        time_boost_0_25: Some(stats.time_boost_0_25 as f64),
        time_boost_25_50: Some(stats.time_boost_25_50 as f64),
        time_boost_50_75: Some(stats.time_boost_50_75 as f64),
        time_boost_75_100: Some(stats.time_boost_75_100 as f64),
        percent_boost_0_25: Some(stats.boost_0_25_pct() as f64),
        percent_boost_25_50: Some(stats.boost_25_50_pct() as f64),
        percent_boost_50_75: Some(stats.boost_50_75_pct() as f64),
        percent_boost_75_100: Some(stats.boost_75_100_pct() as f64),
    }
}
