use std::collections::BTreeSet;

use serde_json::Value;

use crate::*;

use super::comparable_types::{
    ComparableBoostStats, ComparableCoreStats, ComparableDemoStats, ComparableMovementStats,
    ComparablePlayerStats, ComparablePositioningStats, ComparableReplayStats,
};
use super::model::TeamColor;

fn json_number(stats: Option<&Value>, field: &str) -> Option<f64> {
    stats
        .and_then(|stats| stats.get(field))
        .and_then(Value::as_f64)
}

fn comparable_core_from_json(stats: Option<&Value>) -> ComparableCoreStats {
    ComparableCoreStats {
        score: json_number(stats, "score"),
        goals: json_number(stats, "goals"),
        assists: json_number(stats, "assists"),
        saves: json_number(stats, "saves"),
        shots: json_number(stats, "shots"),
        shooting_percentage: json_number(stats, "shooting_percentage"),
    }
}

fn comparable_boost_from_json(stats: Option<&Value>) -> ComparableBoostStats {
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

fn comparable_movement_from_json(stats: Option<&Value>) -> ComparableMovementStats {
    ComparableMovementStats {
        avg_speed: json_number(stats, "avg_speed"),
        total_distance: json_number(stats, "total_distance"),
        time_supersonic_speed: json_number(stats, "time_supersonic_speed"),
        time_boost_speed: json_number(stats, "time_boost_speed"),
        time_slow_speed: json_number(stats, "time_slow_speed"),
        time_ground: json_number(stats, "time_ground"),
        time_low_air: json_number(stats, "time_low_air"),
        time_high_air: json_number(stats, "time_high_air"),
        time_powerslide: json_number(stats, "time_powerslide"),
        count_powerslide: json_number(stats, "count_powerslide"),
        avg_powerslide_duration: json_number(stats, "avg_powerslide_duration"),
        avg_speed_percentage: json_number(stats, "avg_speed_percentage"),
        percent_slow_speed: json_number(stats, "percent_slow_speed"),
        percent_boost_speed: json_number(stats, "percent_boost_speed"),
        percent_supersonic_speed: json_number(stats, "percent_supersonic_speed"),
        percent_ground: json_number(stats, "percent_ground"),
        percent_low_air: json_number(stats, "percent_low_air"),
        percent_high_air: json_number(stats, "percent_high_air"),
    }
}

fn comparable_positioning_from_json(stats: Option<&Value>) -> ComparablePositioningStats {
    ComparablePositioningStats {
        avg_distance_to_ball: json_number(stats, "avg_distance_to_ball"),
        avg_distance_to_ball_possession: json_number(stats, "avg_distance_to_ball_possession"),
        avg_distance_to_ball_no_possession: json_number(
            stats,
            "avg_distance_to_ball_no_possession",
        ),
        avg_distance_to_mates: json_number(stats, "avg_distance_to_mates"),
        time_defensive_third: json_number(stats, "time_defensive_third"),
        time_neutral_third: json_number(stats, "time_neutral_third"),
        time_offensive_third: json_number(stats, "time_offensive_third"),
        time_defensive_half: json_number(stats, "time_defensive_half"),
        time_offensive_half: json_number(stats, "time_offensive_half"),
        time_behind_ball: json_number(stats, "time_behind_ball"),
        time_infront_ball: json_number(stats, "time_infront_ball"),
        time_most_back: json_number(stats, "time_most_back"),
        time_most_forward: json_number(stats, "time_most_forward"),
        time_closest_to_ball: json_number(stats, "time_closest_to_ball"),
        time_farthest_from_ball: json_number(stats, "time_farthest_from_ball"),
        percent_defensive_third: json_number(stats, "percent_defensive_third"),
        percent_neutral_third: json_number(stats, "percent_neutral_third"),
        percent_offensive_third: json_number(stats, "percent_offensive_third"),
        percent_defensive_half: json_number(stats, "percent_defensive_half"),
        percent_offensive_half: json_number(stats, "percent_offensive_half"),
        percent_behind_ball: json_number(stats, "percent_behind_ball"),
        percent_infront_ball: json_number(stats, "percent_infront_ball"),
        percent_most_back: json_number(stats, "percent_most_back"),
        percent_most_forward: json_number(stats, "percent_most_forward"),
        percent_closest_to_ball: json_number(stats, "percent_closest_to_ball"),
        percent_farthest_from_ball: json_number(stats, "percent_farthest_from_ball"),
    }
}

fn comparable_demo_from_json(stats: Option<&Value>) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: json_number(stats, "inflicted"),
        taken: json_number(stats, "taken"),
    }
}

fn comparable_team_demo_from_json(stats: Option<&Value>) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: json_number(stats, "inflicted"),
        taken: None,
    }
}

fn comparable_core_from_player(stats: &CorePlayerStats) -> ComparableCoreStats {
    ComparableCoreStats {
        score: Some(stats.score as f64),
        goals: Some(stats.goals as f64),
        assists: Some(stats.assists as f64),
        saves: Some(stats.saves as f64),
        shots: Some(stats.shots as f64),
        shooting_percentage: Some(stats.shooting_percentage() as f64),
    }
}

fn comparable_core_from_team(stats: &CoreTeamStats) -> ComparableCoreStats {
    ComparableCoreStats {
        score: Some(stats.score as f64),
        goals: Some(stats.goals as f64),
        assists: Some(stats.assists as f64),
        saves: Some(stats.saves as f64),
        shots: Some(stats.shots as f64),
        shooting_percentage: Some(stats.shooting_percentage() as f64),
    }
}

pub(super) fn raw_boost_amount_as_comparable_units(value: f32) -> f64 {
    boost_amount_to_percent(value) as f64
}

fn comparable_boost_from_stats(stats: &BoostStats) -> ComparableBoostStats {
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

fn sum_present(values: impl IntoIterator<Item = Option<f64>>) -> Option<f64> {
    let mut saw_value = false;
    let sum = values.into_iter().fold(0.0, |acc, value| match value {
        Some(value) => {
            saw_value = true;
            acc + value
        }
        None => acc,
    });
    saw_value.then_some(sum)
}

fn comparable_movement_from_stats(
    movement: &MovementStats,
    powerslide: &PowerslideStats,
) -> ComparableMovementStats {
    ComparableMovementStats {
        avg_speed: Some(movement.average_speed() as f64),
        total_distance: Some(movement.total_distance as f64),
        time_supersonic_speed: Some(movement.time_supersonic_speed as f64),
        time_boost_speed: Some(movement.time_boost_speed as f64),
        time_slow_speed: Some(movement.time_slow_speed as f64),
        time_ground: Some(movement.time_on_ground as f64),
        time_low_air: Some(movement.time_low_air as f64),
        time_high_air: Some(movement.time_high_air as f64),
        time_powerslide: Some(powerslide.total_duration as f64),
        count_powerslide: Some(powerslide.press_count as f64),
        avg_powerslide_duration: Some(powerslide.average_duration() as f64),
        avg_speed_percentage: Some(movement.average_speed_pct() as f64),
        percent_slow_speed: Some(movement.slow_speed_pct() as f64),
        percent_boost_speed: Some(movement.boost_speed_pct() as f64),
        percent_supersonic_speed: Some(movement.supersonic_speed_pct() as f64),
        percent_ground: Some(movement.on_ground_pct() as f64),
        percent_low_air: Some(movement.low_air_pct() as f64),
        percent_high_air: Some(movement.high_air_pct() as f64),
    }
}

fn comparable_positioning_from_stats(stats: &PositioningStats) -> ComparablePositioningStats {
    ComparablePositioningStats {
        avg_distance_to_ball: Some(stats.average_distance_to_ball() as f64),
        avg_distance_to_ball_possession: Some(
            stats.average_distance_to_ball_has_possession() as f64
        ),
        avg_distance_to_ball_no_possession: Some(
            stats.average_distance_to_ball_no_possession() as f64
        ),
        avg_distance_to_mates: Some(stats.average_distance_to_teammates() as f64),
        time_defensive_third: Some(stats.time_defensive_zone as f64),
        time_neutral_third: Some(stats.time_neutral_zone as f64),
        time_offensive_third: Some(stats.time_offensive_zone as f64),
        time_defensive_half: Some(stats.time_defensive_half as f64),
        time_offensive_half: Some(stats.time_offensive_half as f64),
        time_behind_ball: Some(stats.time_behind_ball as f64),
        time_infront_ball: Some(stats.time_in_front_of_ball as f64),
        time_most_back: Some(stats.time_most_back as f64),
        time_most_forward: Some(stats.time_most_forward as f64),
        time_closest_to_ball: Some(stats.time_closest_to_ball as f64),
        time_farthest_from_ball: Some(stats.time_farthest_from_ball as f64),
        percent_defensive_third: Some(stats.defensive_zone_pct() as f64),
        percent_neutral_third: Some(stats.neutral_zone_pct() as f64),
        percent_offensive_third: Some(stats.offensive_zone_pct() as f64),
        percent_defensive_half: Some(stats.defensive_half_pct() as f64),
        percent_offensive_half: Some(stats.offensive_half_pct() as f64),
        percent_behind_ball: Some(stats.behind_ball_pct() as f64),
        percent_infront_ball: Some(stats.in_front_of_ball_pct() as f64),
        percent_most_back: Some(stats.most_back_pct() as f64),
        percent_most_forward: Some(stats.most_forward_pct() as f64),
        percent_closest_to_ball: Some(stats.closest_to_ball_pct() as f64),
        percent_farthest_from_ball: Some(stats.farthest_from_ball_pct() as f64),
    }
}

fn comparable_demo_from_player(stats: &DemoPlayerStats) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: Some(stats.demos_inflicted as f64),
        taken: Some(stats.demos_taken as f64),
    }
}

fn comparable_demo_from_team(stats: &DemoTeamStats) -> ComparableDemoStats {
    ComparableDemoStats {
        inflicted: Some(stats.demos_inflicted as f64),
        taken: None,
    }
}

pub(crate) struct ComputedComparableStats {
    pub(super) replay_meta: ReplayMeta,
    pub(super) match_stats: MatchStatsReducer,
    pub(super) boost: BoostReducer,
    pub(super) movement: MovementReducer,
    pub(super) positioning: PositioningReducer,
    pub(super) demo: DemoReducer,
    pub(super) powerslide: PowerslideReducer,
}

struct ComparableStatsCollector {
    match_stats: MatchStatsReducer,
    boost: BoostReducer,
    movement: MovementReducer,
    positioning: PositioningReducer,
    demo: DemoReducer,
    powerslide: PowerslideReducer,
    derived_signals: DerivedSignalGraph,
    replay_meta: Option<ReplayMeta>,
    last_sample_time: Option<f32>,
    last_demolish_count: usize,
    last_boost_pad_event_count: usize,
    last_touch_event_count: usize,
    last_player_stat_event_count: usize,
    last_goal_event_count: usize,
}

impl ComparableStatsCollector {
    fn new() -> Self {
        let match_stats = MatchStatsReducer::new();
        let boost = BoostReducer::new();
        let movement = MovementReducer::new();
        let positioning = PositioningReducer::new();
        let demo = DemoReducer::new();
        let powerslide = PowerslideReducer::new();

        let mut required_signals = BTreeSet::new();
        for signals in [
            match_stats.required_derived_signals(),
            boost.required_derived_signals(),
            movement.required_derived_signals(),
            positioning.required_derived_signals(),
            demo.required_derived_signals(),
            powerslide.required_derived_signals(),
        ] {
            required_signals.extend(signals);
        }

        Self {
            match_stats,
            boost,
            movement,
            positioning,
            demo,
            powerslide,
            derived_signals: derived_signal_graph_for_ids(required_signals.into_iter()),
            replay_meta: None,
            last_sample_time: None,
            last_demolish_count: 0,
            last_boost_pad_event_count: 0,
            last_touch_event_count: 0,
            last_player_stat_event_count: 0,
            last_goal_event_count: 0,
        }
    }

    fn into_stats(self) -> ComputedComparableStats {
        ComputedComparableStats {
            replay_meta: self
                .replay_meta
                .expect("replay metadata should be initialized before building comparable stats"),
            match_stats: self.match_stats,
            boost: self.boost,
            movement: self.movement,
            positioning: self.positioning,
            demo: self.demo,
            powerslide: self.powerslide,
        }
    }
}

impl Collector for ComparableStatsCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if self.replay_meta.is_none() {
            let replay_meta = processor.get_replay_meta()?;
            self.derived_signals.on_replay_meta(&replay_meta)?;
            self.match_stats.on_replay_meta(&replay_meta)?;
            self.boost.on_replay_meta(&replay_meta)?;
            self.movement.on_replay_meta(&replay_meta)?;
            self.positioning.on_replay_meta(&replay_meta)?;
            self.demo.on_replay_meta(&replay_meta)?;
            self.powerslide.on_replay_meta(&replay_meta)?;
            self.replay_meta = Some(replay_meta);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let mut sample = CoreSample::from_processor(processor, frame_number, current_time, dt)?;
        sample.active_demos.clear();
        sample.demo_events = processor.demolishes[self.last_demolish_count..].to_vec();
        sample.boost_pad_events =
            processor.boost_pad_events[self.last_boost_pad_event_count..].to_vec();
        sample.touch_events = processor.touch_events[self.last_touch_event_count..].to_vec();
        sample.player_stat_events =
            processor.player_stat_events[self.last_player_stat_event_count..].to_vec();
        sample.goal_events = processor.goal_events[self.last_goal_event_count..].to_vec();
        let analysis_context = self.derived_signals.evaluate(&sample)?;

        self.match_stats
            .on_sample_with_context(&sample, analysis_context)?;
        self.boost
            .on_sample_with_context(&sample, analysis_context)?;
        self.movement
            .on_sample_with_context(&sample, analysis_context)?;
        self.positioning
            .on_sample_with_context(&sample, analysis_context)?;
        self.demo
            .on_sample_with_context(&sample, analysis_context)?;
        self.powerslide
            .on_sample_with_context(&sample, analysis_context)?;

        self.last_sample_time = Some(current_time);
        self.last_demolish_count = processor.demolishes.len();
        self.last_boost_pad_event_count = processor.boost_pad_events.len();
        self.last_touch_event_count = processor.touch_events.len();
        self.last_player_stat_event_count = processor.player_stat_events.len();
        self.last_goal_event_count = processor.goal_events.len();

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.derived_signals.finish()?;
        self.match_stats.finish()?;
        self.boost.finish()?;
        self.movement.finish()?;
        self.positioning.finish()?;
        self.demo.finish()?;
        self.powerslide.finish()?;
        Ok(())
    }
}

pub(crate) fn compute_comparable_stats(
    replay: &boxcars::Replay,
) -> SubtrActorResult<ComputedComparableStats> {
    let mut collector = ComparableStatsCollector::new();
    let mut processor = ReplayProcessor::new(replay)?;
    processor.process(&mut collector)?;
    Ok(collector.into_stats())
}

pub(crate) fn build_actual_comparable_stats(
    stats: &ComputedComparableStats,
) -> ComparableReplayStats {
    let mut comparable = ComparableReplayStats::default();

    for (team_color, players) in [
        (TeamColor::Blue, &stats.replay_meta.team_zero),
        (TeamColor::Orange, &stats.replay_meta.team_one),
    ] {
        let team_stats = comparable.team_mut(team_color);
        team_stats.core = comparable_core_from_team(&match team_color {
            TeamColor::Blue => stats.match_stats.team_zero_stats(),
            TeamColor::Orange => stats.match_stats.team_one_stats(),
        });
        let mut team_boost = comparable_boost_from_stats(match team_color {
            TeamColor::Blue => stats.boost.team_zero_stats(),
            TeamColor::Orange => stats.boost.team_one_stats(),
        });
        team_stats.movement = comparable_movement_from_stats(
            match team_color {
                TeamColor::Blue => stats.movement.team_zero_stats(),
                TeamColor::Orange => stats.movement.team_one_stats(),
            },
            match team_color {
                TeamColor::Blue => stats.powerslide.team_zero_stats(),
                TeamColor::Orange => stats.powerslide.team_one_stats(),
            },
        );
        team_stats.demo = comparable_demo_from_team(match team_color {
            TeamColor::Blue => stats.demo.team_zero_stats(),
            TeamColor::Orange => stats.demo.team_one_stats(),
        });

        let mut player_boost_stats = Vec::new();
        for player in players {
            let player_boost = comparable_boost_from_stats(
                &stats
                    .boost
                    .player_stats()
                    .get(&player.remote_id)
                    .cloned()
                    .unwrap_or_default(),
            );
            player_boost_stats.push(player_boost.clone());
            let player_stats = ComparablePlayerStats {
                core: comparable_core_from_player(
                    &stats
                        .match_stats
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                boost: player_boost,
                movement: comparable_movement_from_stats(
                    &stats
                        .movement
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    &stats
                        .powerslide
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                positioning: comparable_positioning_from_stats(
                    &stats
                        .positioning
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                demo: comparable_demo_from_player(
                    &stats
                        .demo
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
            };
            team_stats.players.insert(player.name.clone(), player_stats);
        }

        team_boost.avg_amount =
            sum_present(player_boost_stats.iter().map(|stats| stats.avg_amount));
        team_boost.bpm = sum_present(player_boost_stats.iter().map(|stats| stats.bpm));
        team_stats.boost = team_boost;
    }

    comparable
}

pub(crate) fn build_expected_comparable_stats(expected: &Value) -> ComparableReplayStats {
    let mut comparable = ComparableReplayStats::default();

    for team_color in [TeamColor::Blue, TeamColor::Orange] {
        let Some(team) = expected.get(team_color.team_key()) else {
            continue;
        };

        let team_stats = comparable.team_mut(team_color);
        let team_json_stats = team.get("stats");
        team_stats.core =
            comparable_core_from_json(team_json_stats.and_then(|stats| stats.get("core")));
        team_stats.boost =
            comparable_boost_from_json(team_json_stats.and_then(|stats| stats.get("boost")));
        team_stats.movement =
            comparable_movement_from_json(team_json_stats.and_then(|stats| stats.get("movement")));
        team_stats.demo =
            comparable_team_demo_from_json(team_json_stats.and_then(|stats| stats.get("demo")));

        let Some(players) = team.get("players").and_then(Value::as_array) else {
            continue;
        };

        for player in players {
            let Some(name) = player.get("name").and_then(Value::as_str) else {
                continue;
            };
            let stats = player.get("stats");
            team_stats.players.insert(
                name.to_string(),
                ComparablePlayerStats {
                    core: comparable_core_from_json(stats.and_then(|stats| stats.get("core"))),
                    boost: comparable_boost_from_json(stats.and_then(|stats| stats.get("boost"))),
                    movement: comparable_movement_from_json(
                        stats.and_then(|stats| stats.get("movement")),
                    ),
                    positioning: comparable_positioning_from_json(
                        stats.and_then(|stats| stats.get("positioning")),
                    ),
                    demo: comparable_demo_from_json(stats.and_then(|stats| stats.get("demo"))),
                },
            );
        }
    }

    comparable
}

#[cfg(test)]
#[path = "conversion_test.rs"]
mod tests;
