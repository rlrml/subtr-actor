use serde::Serialize;

use crate::*;

pub const LEGACY_STAT_VARIANT: &str = "legacy";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatUnit {
    Seconds,
    Percent,
    UnrealUnits,
    UnrealUnitsPerSecond,
    Boost,
    BoostPerMinute,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StatDescriptor {
    pub domain: &'static str,
    pub name: &'static str,
    pub variant: &'static str,
    pub unit: StatUnit,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "value_type", content = "value", rename_all = "snake_case")]
pub enum StatValue {
    Float(f32),
    Unsigned(u32),
    Signed(i32),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportedStat {
    #[serde(flatten)]
    pub descriptor: StatDescriptor,
    pub value: StatValue,
}

impl ExportedStat {
    pub fn float(domain: &'static str, name: &'static str, unit: StatUnit, value: f32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
            },
            value: StatValue::Float(value),
        }
    }

    pub fn unsigned(domain: &'static str, name: &'static str, unit: StatUnit, value: u32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
            },
            value: StatValue::Unsigned(value),
        }
    }

    pub fn signed(domain: &'static str, name: &'static str, unit: StatUnit, value: i32) -> Self {
        Self {
            descriptor: StatDescriptor {
                domain,
                name,
                variant: LEGACY_STAT_VARIANT,
                unit,
            },
            value: StatValue::Signed(value),
        }
    }
}

pub trait StatFieldProvider {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat));

    fn stat_fields(&self) -> Vec<ExportedStat> {
        let mut fields = Vec::new();
        self.visit_stat_fields(&mut |field| fields.push(field));
        fields
    }
}

impl StatFieldProvider for PossessionStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "possession",
            "time",
            StatUnit::Seconds,
            self.tracked_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_zero_time",
            StatUnit::Seconds,
            self.team_zero_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_one_time",
            StatUnit::Seconds,
            self.team_one_time,
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_zero_pct",
            StatUnit::Percent,
            self.team_zero_pct(),
        ));
        visitor(ExportedStat::float(
            "possession",
            "team_one_pct",
            StatUnit::Percent,
            self.team_one_pct(),
        ));
    }
}

impl StatFieldProvider for CorePlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::signed(
            "core",
            "score",
            StatUnit::Count,
            self.score,
        ));
        visitor(ExportedStat::signed(
            "core",
            "goals",
            StatUnit::Count,
            self.goals,
        ));
        visitor(ExportedStat::signed(
            "core",
            "assists",
            StatUnit::Count,
            self.assists,
        ));
        visitor(ExportedStat::signed(
            "core",
            "saves",
            StatUnit::Count,
            self.saves,
        ));
        visitor(ExportedStat::signed(
            "core",
            "shots",
            StatUnit::Count,
            self.shots,
        ));
        visitor(ExportedStat::unsigned(
            "core",
            "goals_conceded_while_last_defender",
            StatUnit::Count,
            self.goals_conceded_while_last_defender,
        ));
        visitor(ExportedStat::float(
            "core",
            "shooting_percentage",
            StatUnit::Percent,
            self.shooting_percentage(),
        ));
    }
}

impl StatFieldProvider for CoreTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::signed(
            "core",
            "score",
            StatUnit::Count,
            self.score,
        ));
        visitor(ExportedStat::signed(
            "core",
            "goals",
            StatUnit::Count,
            self.goals,
        ));
        visitor(ExportedStat::signed(
            "core",
            "assists",
            StatUnit::Count,
            self.assists,
        ));
        visitor(ExportedStat::signed(
            "core",
            "saves",
            StatUnit::Count,
            self.saves,
        ));
        visitor(ExportedStat::signed(
            "core",
            "shots",
            StatUnit::Count,
            self.shots,
        ));
        visitor(ExportedStat::float(
            "core",
            "shooting_percentage",
            StatUnit::Percent,
            self.shooting_percentage(),
        ));
    }
}

impl StatFieldProvider for BoostStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "boost",
            "bpm",
            StatUnit::BoostPerMinute,
            self.bpm(),
        ));
        visitor(ExportedStat::float(
            "boost",
            "avg_amount",
            StatUnit::Boost,
            self.average_boost_amount(),
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_collected",
            StatUnit::Boost,
            self.amount_collected,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_stolen",
            StatUnit::Boost,
            self.amount_stolen,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_collected_big",
            StatUnit::Boost,
            self.amount_collected_big,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_stolen_big",
            StatUnit::Boost,
            self.amount_stolen_big,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_collected_small",
            StatUnit::Boost,
            self.amount_collected_small,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_stolen_small",
            StatUnit::Boost,
            self.amount_stolen_small,
        ));
        visitor(ExportedStat::unsigned(
            "boost",
            "count_collected_big",
            StatUnit::Count,
            self.big_pads_collected,
        ));
        visitor(ExportedStat::unsigned(
            "boost",
            "count_stolen_big",
            StatUnit::Count,
            self.big_pads_stolen,
        ));
        visitor(ExportedStat::unsigned(
            "boost",
            "count_collected_small",
            StatUnit::Count,
            self.small_pads_collected,
        ));
        visitor(ExportedStat::unsigned(
            "boost",
            "count_stolen_small",
            StatUnit::Count,
            self.small_pads_stolen,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_overfill",
            StatUnit::Boost,
            self.overfill_total,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_overfill_stolen",
            StatUnit::Boost,
            self.overfill_from_stolen,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_used_while_supersonic",
            StatUnit::Boost,
            self.amount_used_while_supersonic,
        ));
        visitor(ExportedStat::float(
            "boost",
            "time_zero_boost",
            StatUnit::Seconds,
            self.time_zero_boost,
        ));
        visitor(ExportedStat::float(
            "boost",
            "time_full_boost",
            StatUnit::Seconds,
            self.time_hundred_boost,
        ));
        visitor(ExportedStat::float(
            "boost",
            "time_boost_0_25",
            StatUnit::Seconds,
            self.time_boost_0_25,
        ));
        visitor(ExportedStat::float(
            "boost",
            "time_boost_25_50",
            StatUnit::Seconds,
            self.time_boost_25_50,
        ));
        visitor(ExportedStat::float(
            "boost",
            "time_boost_50_75",
            StatUnit::Seconds,
            self.time_boost_50_75,
        ));
        visitor(ExportedStat::float(
            "boost",
            "time_boost_75_100",
            StatUnit::Seconds,
            self.time_boost_75_100,
        ));
        visitor(ExportedStat::float(
            "boost",
            "percent_zero_boost",
            StatUnit::Percent,
            self.zero_boost_pct(),
        ));
        visitor(ExportedStat::float(
            "boost",
            "percent_full_boost",
            StatUnit::Percent,
            self.hundred_boost_pct(),
        ));
        visitor(ExportedStat::float(
            "boost",
            "percent_boost_0_25",
            StatUnit::Percent,
            self.boost_0_25_pct(),
        ));
        visitor(ExportedStat::float(
            "boost",
            "percent_boost_25_50",
            StatUnit::Percent,
            self.boost_25_50_pct(),
        ));
        visitor(ExportedStat::float(
            "boost",
            "percent_boost_50_75",
            StatUnit::Percent,
            self.boost_50_75_pct(),
        ));
        visitor(ExportedStat::float(
            "boost",
            "percent_boost_75_100",
            StatUnit::Percent,
            self.boost_75_100_pct(),
        ));
    }
}

impl StatFieldProvider for MovementStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "movement",
            "total_distance",
            StatUnit::UnrealUnits,
            self.total_distance,
        ));
        visitor(ExportedStat::float(
            "movement",
            "avg_speed",
            StatUnit::UnrealUnitsPerSecond,
            self.average_speed(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_supersonic_speed",
            StatUnit::Seconds,
            self.time_supersonic_speed,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_boost_speed",
            StatUnit::Seconds,
            self.time_boost_speed,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_slow_speed",
            StatUnit::Seconds,
            self.time_slow_speed,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_ground",
            StatUnit::Seconds,
            self.time_on_ground,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_low_air",
            StatUnit::Seconds,
            self.time_low_air,
        ));
        visitor(ExportedStat::float(
            "movement",
            "time_high_air",
            StatUnit::Seconds,
            self.time_high_air,
        ));
        visitor(ExportedStat::float(
            "movement",
            "avg_speed_percentage",
            StatUnit::Percent,
            self.average_speed_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_slow_speed",
            StatUnit::Percent,
            self.slow_speed_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_boost_speed",
            StatUnit::Percent,
            self.boost_speed_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_supersonic_speed",
            StatUnit::Percent,
            self.supersonic_speed_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_ground",
            StatUnit::Percent,
            self.on_ground_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_low_air",
            StatUnit::Percent,
            self.low_air_pct(),
        ));
        visitor(ExportedStat::float(
            "movement",
            "percent_high_air",
            StatUnit::Percent,
            self.high_air_pct(),
        ));
    }
}

impl StatFieldProvider for PositioningStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "positioning",
            "avg_distance_to_ball",
            StatUnit::UnrealUnits,
            self.average_distance_to_ball(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "avg_distance_to_ball_possession",
            StatUnit::UnrealUnits,
            self.average_distance_to_ball_has_possession(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "avg_distance_to_ball_no_possession",
            StatUnit::UnrealUnits,
            self.average_distance_to_ball_no_possession(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "avg_distance_to_mates",
            StatUnit::UnrealUnits,
            self.average_distance_to_teammates(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_defensive_third",
            StatUnit::Seconds,
            self.time_defensive_third,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_neutral_third",
            StatUnit::Seconds,
            self.time_neutral_third,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_offensive_third",
            StatUnit::Seconds,
            self.time_offensive_third,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_defensive_half",
            StatUnit::Seconds,
            self.time_defensive_half,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_offensive_half",
            StatUnit::Seconds,
            self.time_offensive_half,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_behind_ball",
            StatUnit::Seconds,
            self.time_behind_ball,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_in_front_of_ball",
            StatUnit::Seconds,
            self.time_in_front_of_ball,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_most_back",
            StatUnit::Seconds,
            self.time_most_back,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_most_forward",
            StatUnit::Seconds,
            self.time_most_forward,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_closest_to_ball",
            StatUnit::Seconds,
            self.time_closest_to_ball,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_farthest_from_ball",
            StatUnit::Seconds,
            self.time_farthest_from_ball,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_defensive_third",
            StatUnit::Percent,
            self.defensive_third_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_neutral_third",
            StatUnit::Percent,
            self.neutral_third_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_offensive_third",
            StatUnit::Percent,
            self.offensive_third_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_defensive_half",
            StatUnit::Percent,
            self.defensive_half_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_offensive_half",
            StatUnit::Percent,
            self.offensive_half_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_behind_ball",
            StatUnit::Percent,
            self.behind_ball_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_in_front_of_ball",
            StatUnit::Percent,
            self.in_front_of_ball_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_most_back",
            StatUnit::Percent,
            self.most_back_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_most_forward",
            StatUnit::Percent,
            self.most_forward_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_closest_to_ball",
            StatUnit::Percent,
            self.closest_to_ball_pct(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "percent_farthest_from_ball",
            StatUnit::Percent,
            self.farthest_from_ball_pct(),
        ));
    }
}

impl StatFieldProvider for PowerslideStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "powerslide",
            "time_powerslide",
            StatUnit::Seconds,
            self.total_duration,
        ));
        visitor(ExportedStat::unsigned(
            "powerslide",
            "count_powerslide",
            StatUnit::Count,
            self.press_count,
        ));
        visitor(ExportedStat::float(
            "powerslide",
            "avg_powerslide_duration",
            StatUnit::Seconds,
            self.average_duration(),
        ));
    }
}

impl StatFieldProvider for DemoPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "demo",
            "inflicted",
            StatUnit::Count,
            self.demos_inflicted,
        ));
        visitor(ExportedStat::unsigned(
            "demo",
            "taken",
            StatUnit::Count,
            self.demos_taken,
        ));
    }
}

impl StatFieldProvider for DemoTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "demo",
            "inflicted",
            StatUnit::Count,
            self.demos_inflicted,
        ));
    }
}
