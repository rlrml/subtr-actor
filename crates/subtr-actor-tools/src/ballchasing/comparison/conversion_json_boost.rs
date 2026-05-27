use serde_json::Value;

use super::super::super::comparable_types::ComparableBoostStats;
use super::json_number;

pub(crate) fn comparable_boost_from_json(stats: Option<&Value>) -> ComparableBoostStats {
    ComparableBoostStats {
        bpm: json_number(stats, "bpm"),
        avg_amount: json_number(stats, "avg_amount"),
        amount_collected: json_number(stats, "amount_collected"),
        amount_stolen: json_number(stats, "amount_stolen"),
        amount_collected_big: json_number(stats, "amount_collected_big"),
        amount_stolen_big: json_number(stats, "amount_stolen_big"),
        amount_collected_small: json_number(stats, "amount_collected_small"),
        amount_stolen_small: json_number(stats, "amount_stolen_small"),
        count_collected_big: json_number(stats, "count_collected_big"),
        count_stolen_big: json_number(stats, "count_stolen_big"),
        count_collected_small: json_number(stats, "count_collected_small"),
        count_stolen_small: json_number(stats, "count_stolen_small"),
        amount_overfill: json_number(stats, "amount_overfill"),
        amount_overfill_stolen: json_number(stats, "amount_overfill_stolen"),
        amount_used_while_supersonic: json_number(stats, "amount_used_while_supersonic"),
        time_zero_boost: json_number(stats, "time_zero_boost"),
        percent_zero_boost: json_number(stats, "percent_zero_boost"),
        time_full_boost: json_number(stats, "time_full_boost"),
        percent_full_boost: json_number(stats, "percent_full_boost"),
        time_boost_0_25: json_number(stats, "time_boost_0_25"),
        time_boost_25_50: json_number(stats, "time_boost_25_50"),
        time_boost_50_75: json_number(stats, "time_boost_50_75"),
        time_boost_75_100: json_number(stats, "time_boost_75_100"),
        percent_boost_0_25: json_number(stats, "percent_boost_0_25"),
        percent_boost_25_50: json_number(stats, "percent_boost_25_50"),
        percent_boost_50_75: json_number(stats, "percent_boost_50_75"),
        percent_boost_75_100: json_number(stats, "percent_boost_75_100"),
    }
}
