use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

use crate::stats::analysis_graph::AnalysisGraph;
use crate::*;

use super::types::serialize_to_json_value;

fn player_stats_entries<'a, T>(
    player_stats: &'a HashMap<PlayerId, T>,
) -> Vec<PlayerStatsEntry<'a, T>> {
    let mut entries: Vec<_> = player_stats
        .iter()
        .map(|(player_id, stats)| PlayerStatsEntry {
            player_id: player_id.clone(),
            stats,
        })
        .collect();
    entries.sort_by(|left, right| {
        format!("{:?}", left.player_id).cmp(&format!("{:?}", right.player_id))
    });
    entries
}

#[derive(Serialize)]
struct PlayerStatsEntry<'a, T> {
    player_id: PlayerId,
    stats: &'a T,
}

#[derive(Serialize)]
struct OwnedPlayerStatsEntry<T> {
    player_id: PlayerId,
    stats: T,
}

#[derive(Serialize)]
struct PlayerStatsExport<'a, T> {
    player_stats: Vec<PlayerStatsEntry<'a, T>>,
}

#[derive(Serialize)]
struct OwnedPlayerStatsExport<T> {
    player_stats: Vec<OwnedPlayerStatsEntry<T>>,
}

#[derive(Serialize)]
struct PlayerStatsWithEventsExport<'a, T, E> {
    player_stats: Vec<PlayerStatsEntry<'a, T>>,
    events: &'a [E],
}

#[derive(Serialize)]
struct TeamPlayerStatsExport<'a, Team, Player> {
    team_zero: &'a Team,
    team_one: &'a Team,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
}

#[derive(Serialize)]
struct TeamOwnedPlayerStatsExport<'a, Team, Player> {
    team_zero: &'a Team,
    team_one: &'a Team,
    player_stats: Vec<OwnedPlayerStatsEntry<Player>>,
}

#[derive(Serialize)]
struct TeamPlayerStatsWithEventsExport<'a, Team, Player, Event> {
    team_zero: &'a Team,
    team_one: &'a Team,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
    events: &'a [Event],
}

#[derive(Serialize)]
struct StatsExport<'a, T> {
    stats: &'a T,
}

#[derive(Serialize)]
struct StatsWithEventsExport<'a, T, E> {
    stats: &'a T,
    events: &'a [E],
}

#[derive(Serialize)]
struct EventsExport<'a, E> {
    events: &'a [E],
}

#[derive(Serialize)]
struct StatsWithPlayerEventsExport<'a, T, Player, E> {
    stats: &'a T,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
    events: &'a [E],
}

#[derive(Serialize)]
struct StatsWithPlayerStatsExport<'a, T, Player> {
    stats: &'a T,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
}

#[derive(Serialize)]
struct CoreStatsExport<'a> {
    team_zero: CoreTeamStats,
    team_one: CoreTeamStats,
    player_stats: Vec<PlayerStatsEntry<'a, CorePlayerStats>>,
    timeline: &'a [TimelineEvent],
    goal_context: &'a [GoalContextEvent],
}

#[derive(Serialize)]
struct CoreStatsSnapshotExport {
    team_zero: CoreTeamStatsSnapshot,
    team_one: CoreTeamStatsSnapshot,
    player_stats: Vec<OwnedPlayerStatsEntry<CorePlayerStatsSnapshot>>,
}

#[derive(Serialize)]
struct CoreTeamStatsSnapshot {
    score: i32,
    goals: i32,
    assists: i32,
    saves: i32,
    shots: i32,
    kickoff_goal_count: u32,
    short_goal_count: u32,
    medium_goal_count: u32,
    long_goal_count: u32,
    goal_times: Vec<f32>,
    goal_ball_air_time_sample_count: u32,
    cumulative_goal_ball_air_time: f32,
    average_goal_ball_air_time: f32,
    median_goal_ball_air_time: f32,
    last_goal_ball_air_time: Option<f32>,
    goal_ball_air_times: Vec<f32>,
    counter_attack_goal_count: u32,
    sustained_pressure_goal_count: u32,
    other_buildup_goal_count: u32,
}

impl From<CoreTeamStats> for CoreTeamStatsSnapshot {
    fn from(stats: CoreTeamStats) -> Self {
        Self {
            score: stats.score,
            goals: stats.goals,
            assists: stats.assists,
            saves: stats.saves,
            shots: stats.shots,
            kickoff_goal_count: stats.scoring_context.goal_after_kickoff.kickoff_goal_count,
            short_goal_count: stats.scoring_context.goal_after_kickoff.short_goal_count,
            medium_goal_count: stats.scoring_context.goal_after_kickoff.medium_goal_count,
            long_goal_count: stats.scoring_context.goal_after_kickoff.long_goal_count,
            goal_times: stats
                .scoring_context
                .goal_after_kickoff
                .goal_times()
                .to_vec(),
            goal_ball_air_time_sample_count: stats
                .scoring_context
                .goal_ball_air_time
                .goal_ball_air_time_sample_count,
            cumulative_goal_ball_air_time: stats
                .scoring_context
                .goal_ball_air_time
                .cumulative_goal_ball_air_time,
            average_goal_ball_air_time: stats.average_goal_ball_air_time(),
            median_goal_ball_air_time: stats.median_goal_ball_air_time(),
            last_goal_ball_air_time: stats
                .scoring_context
                .goal_ball_air_time
                .last_goal_ball_air_time,
            goal_ball_air_times: stats
                .scoring_context
                .goal_ball_air_time
                .goal_ball_air_times()
                .to_vec(),
            counter_attack_goal_count: stats.scoring_context.goal_buildup.counter_attack_goal_count,
            sustained_pressure_goal_count: stats
                .scoring_context
                .goal_buildup
                .sustained_pressure_goal_count,
            other_buildup_goal_count: stats.scoring_context.goal_buildup.other_buildup_goal_count,
        }
    }
}

#[derive(Serialize)]
struct CorePlayerStatsSnapshot {
    score: i32,
    goals: i32,
    assists: i32,
    saves: i32,
    shots: i32,
    goals_conceded_while_last_defender: u32,
    goals_for_while_most_back: u32,
    goals_against_while_most_back: u32,
    goal_against_boost_sample_count: u32,
    cumulative_boost_on_goals_against: f32,
    average_boost_on_goals_against: f32,
    last_boost_on_goal_against: Option<f32>,
    goal_against_boost_leadup_sample_count: u32,
    cumulative_average_boost_in_goal_against_leadup: f32,
    cumulative_min_boost_in_goal_against_leadup: f32,
    average_boost_in_goal_against_leadup: f32,
    average_min_boost_in_goal_against_leadup: f32,
    last_average_boost_in_goal_against_leadup: Option<f32>,
    last_min_boost_in_goal_against_leadup: Option<f32>,
    goal_against_position_sample_count: u32,
    cumulative_goal_against_position_x: f32,
    cumulative_goal_against_position_y: f32,
    cumulative_goal_against_position_z: f32,
    average_goal_against_position_x: f32,
    average_goal_against_position_y: f32,
    average_goal_against_position_z: f32,
    last_goal_against_position: Option<GoalContextPosition>,
    scoring_goal_last_touch_position_sample_count: u32,
    cumulative_scoring_goal_last_touch_position_x: f32,
    cumulative_scoring_goal_last_touch_position_y: f32,
    cumulative_scoring_goal_last_touch_position_z: f32,
    average_scoring_goal_last_touch_position_x: f32,
    average_scoring_goal_last_touch_position_y: f32,
    average_scoring_goal_last_touch_position_z: f32,
    last_scoring_goal_last_touch_position: Option<GoalContextPosition>,
    kickoff_goal_count: u32,
    short_goal_count: u32,
    medium_goal_count: u32,
    long_goal_count: u32,
    goal_times: Vec<f32>,
    goal_ball_air_time_sample_count: u32,
    cumulative_goal_ball_air_time: f32,
    average_goal_ball_air_time: f32,
    median_goal_ball_air_time: f32,
    last_goal_ball_air_time: Option<f32>,
    goal_ball_air_times: Vec<f32>,
    counter_attack_goal_count: u32,
    sustained_pressure_goal_count: u32,
    other_buildup_goal_count: u32,
}

impl From<&CorePlayerStats> for CorePlayerStatsSnapshot {
    fn from(stats: &CorePlayerStats) -> Self {
        Self {
            score: stats.score,
            goals: stats.goals,
            assists: stats.assists,
            saves: stats.saves,
            shots: stats.shots,
            goals_conceded_while_last_defender: stats
                .scoring_context
                .goals_conceded_while_last_defender,
            goals_for_while_most_back: stats.scoring_context.goals_for_while_most_back,
            goals_against_while_most_back: stats.scoring_context.goals_against_while_most_back,
            goal_against_boost_sample_count: stats.scoring_context.goal_against_boost_sample_count,
            cumulative_boost_on_goals_against: stats
                .scoring_context
                .cumulative_boost_on_goals_against,
            average_boost_on_goals_against: stats.average_boost_on_goals_against(),
            last_boost_on_goal_against: stats.scoring_context.last_boost_on_goal_against,
            goal_against_boost_leadup_sample_count: stats
                .scoring_context
                .goal_against_boost_leadup_sample_count,
            cumulative_average_boost_in_goal_against_leadup: stats
                .scoring_context
                .cumulative_average_boost_in_goal_against_leadup,
            cumulative_min_boost_in_goal_against_leadup: stats
                .scoring_context
                .cumulative_min_boost_in_goal_against_leadup,
            average_boost_in_goal_against_leadup: stats.average_boost_in_goal_against_leadup(),
            average_min_boost_in_goal_against_leadup: stats
                .average_min_boost_in_goal_against_leadup(),
            last_average_boost_in_goal_against_leadup: stats
                .scoring_context
                .last_average_boost_in_goal_against_leadup,
            last_min_boost_in_goal_against_leadup: stats
                .scoring_context
                .last_min_boost_in_goal_against_leadup,
            goal_against_position_sample_count: stats
                .scoring_context
                .goal_against_position_sample_count,
            cumulative_goal_against_position_x: stats
                .scoring_context
                .cumulative_goal_against_position_x,
            cumulative_goal_against_position_y: stats
                .scoring_context
                .cumulative_goal_against_position_y,
            cumulative_goal_against_position_z: stats
                .scoring_context
                .cumulative_goal_against_position_z,
            average_goal_against_position_x: stats.average_goal_against_position_x(),
            average_goal_against_position_y: stats.average_goal_against_position_y(),
            average_goal_against_position_z: stats.average_goal_against_position_z(),
            last_goal_against_position: stats.scoring_context.last_goal_against_position,
            scoring_goal_last_touch_position_sample_count: stats
                .scoring_context
                .scoring_goal_last_touch_position_sample_count,
            cumulative_scoring_goal_last_touch_position_x: stats
                .scoring_context
                .cumulative_scoring_goal_last_touch_position_x,
            cumulative_scoring_goal_last_touch_position_y: stats
                .scoring_context
                .cumulative_scoring_goal_last_touch_position_y,
            cumulative_scoring_goal_last_touch_position_z: stats
                .scoring_context
                .cumulative_scoring_goal_last_touch_position_z,
            average_scoring_goal_last_touch_position_x: stats
                .average_scoring_goal_last_touch_position_x(),
            average_scoring_goal_last_touch_position_y: stats
                .average_scoring_goal_last_touch_position_y(),
            average_scoring_goal_last_touch_position_z: stats
                .average_scoring_goal_last_touch_position_z(),
            last_scoring_goal_last_touch_position: stats
                .scoring_context
                .last_scoring_goal_last_touch_position,
            kickoff_goal_count: stats.scoring_context.goal_after_kickoff.kickoff_goal_count,
            short_goal_count: stats.scoring_context.goal_after_kickoff.short_goal_count,
            medium_goal_count: stats.scoring_context.goal_after_kickoff.medium_goal_count,
            long_goal_count: stats.scoring_context.goal_after_kickoff.long_goal_count,
            goal_times: stats
                .scoring_context
                .goal_after_kickoff
                .goal_times()
                .to_vec(),
            goal_ball_air_time_sample_count: stats
                .scoring_context
                .goal_ball_air_time
                .goal_ball_air_time_sample_count,
            cumulative_goal_ball_air_time: stats
                .scoring_context
                .goal_ball_air_time
                .cumulative_goal_ball_air_time,
            average_goal_ball_air_time: stats.average_goal_ball_air_time(),
            median_goal_ball_air_time: stats.median_goal_ball_air_time(),
            last_goal_ball_air_time: stats
                .scoring_context
                .goal_ball_air_time
                .last_goal_ball_air_time,
            goal_ball_air_times: stats
                .scoring_context
                .goal_ball_air_time
                .goal_ball_air_times()
                .to_vec(),
            counter_attack_goal_count: stats.scoring_context.goal_buildup.counter_attack_goal_count,
            sustained_pressure_goal_count: stats
                .scoring_context
                .goal_buildup
                .sustained_pressure_goal_count,
            other_buildup_goal_count: stats.scoring_context.goal_buildup.other_buildup_goal_count,
        }
    }
}

#[derive(Serialize)]
struct DemoStatsExport<'a> {
    team_zero: &'a DemoTeamStats,
    team_one: &'a DemoTeamStats,
    player_stats: Vec<PlayerStatsEntry<'a, DemoPlayerStats>>,
    timeline: &'a [TimelineEvent],
}

pub fn builtin_stats_module_names() -> &'static [&'static str] {
    &[
        "core",
        "backboard",
        "ceiling_shot",
        "center",
        "double_tap",
        "one_timer",
        "half_volley",
        "pass",
        "aerial_goal",
        "high_aerial_goal",
        "long_distance_goal",
        "own_half_goal",
        "empty_net_goal",
        "counter_attack_goal",
        "flick_goal",
        "one_timer_goal",
        "air_dribble_goal",
        "flip_reset_goal",
        "half_volley_goal",
        "fifty_fifty",
        "possession",
        "pressure",
        "rotation",
        "rush",
        "touch",
        "whiff",
        "wavedash",
        "speed_flip",
        "half_flip",
        "flick",
        "musty_flick",
        "dodge_reset",
        "ball_carry",
        "boost",
        "bump",
        "movement",
        "positioning",
        "powerslide",
        "demo",
    ]
}

fn graph_state<'a, T: 'static>(
    graph: &'a AnalysisGraph,
    module_name: &str,
) -> SubtrActorResult<&'a T> {
    graph.state::<T>().ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "missing analysis-node state for builtin stats module '{module_name}'"
        )))
    })
}

pub(crate) fn builtin_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    match module_name {
        "core" => {
            let calculator = graph_state::<MatchStatsCalculator>(graph, module_name)?;
            serialize_to_json_value(&CoreStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                timeline: calculator.timeline(),
                goal_context: calculator.goal_context_events(),
            })
        }
        "backboard" => {
            let calculator = graph_state::<BackboardCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "ceiling_shot" => {
            let calculator = graph_state::<CeilingShotCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "center" => {
            let calculator = graph_state::<CenterCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "double_tap" => {
            let calculator = graph_state::<DoubleTapCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "one_timer" => {
            let calculator = graph_state::<OneTimerCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "half_volley" => {
            let calculator = graph_state::<HalfVolleyCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "pass" => {
            let calculator = graph_state::<PassCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "aerial_goal" => {
            let calculator = graph_state::<AerialGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "high_aerial_goal" => {
            let calculator = graph_state::<HighAerialGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "long_distance_goal" => {
            let calculator = graph_state::<LongDistanceGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "own_half_goal" => {
            let calculator = graph_state::<OwnHalfGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "empty_net_goal" => {
            let calculator = graph_state::<EmptyNetGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "counter_attack_goal" => {
            let calculator = graph_state::<CounterAttackGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "flick_goal" => {
            let calculator = graph_state::<FlickGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "one_timer_goal" => {
            let calculator = graph_state::<OneTimerGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "air_dribble_goal" => {
            let calculator = graph_state::<AirDribbleGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "flip_reset_goal" => {
            let calculator = graph_state::<FlipResetGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "half_volley_goal" => {
            let calculator = graph_state::<HalfVolleyGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "fifty_fifty" => {
            let calculator = graph_state::<FiftyFiftyCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerEventsExport {
                stats: calculator.stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "possession" => {
            let calculator = graph_state::<PossessionCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })
        }
        "pressure" => {
            let calculator = graph_state::<PressureCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })
        }
        "rotation" => {
            let calculator = graph_state::<RotationCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })
        }
        "rush" => {
            let calculator = graph_state::<RushCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsWithEventsExport {
                stats: calculator.stats(),
                events: calculator.events(),
            })
        }
        "touch" => {
            let calculator = graph_state::<TouchCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })
        }
        "whiff" => {
            let calculator = graph_state::<WhiffCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "wavedash" => {
            let calculator = graph_state::<WavedashCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "speed_flip" => {
            let calculator = graph_state::<SpeedFlipCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "half_flip" => {
            let calculator = graph_state::<HalfFlipCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "flick" => {
            let calculator = graph_state::<FlickCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "musty_flick" => {
            let calculator = graph_state::<MustyFlickCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "dodge_reset" => {
            let calculator = graph_state::<DodgeResetCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.on_ball_events(),
            })
        }
        "ball_carry" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.carry_events(),
            })
        }
        "boost" => {
            let calculator = graph_state::<BoostCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.pickup_comparison_events(),
            })
        }
        "bump" => {
            let calculator = graph_state::<BumpCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
        }
        "movement" => {
            let calculator = graph_state::<MovementCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })
        }
        "positioning" => {
            let calculator = graph_state::<PositioningCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })
        }
        "powerslide" => {
            let calculator = graph_state::<PowerslideCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })
        }
        "demo" => {
            let calculator = graph_state::<DemoCalculator>(graph, module_name)?;
            serialize_to_json_value(&DemoStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                timeline: calculator.timeline(),
            })
        }
        _ => SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
            module_name.to_owned(),
        )),
    }
}

pub(crate) fn builtin_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    let value = match module_name {
        "core" => {
            let calculator = graph_state::<MatchStatsCalculator>(graph, module_name)?;
            let mut player_stats: Vec<_> = calculator
                .player_stats()
                .iter()
                .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                    player_id: player_id.clone(),
                    stats: CorePlayerStatsSnapshot::from(stats),
                })
                .collect();
            player_stats.sort_by(|left, right| {
                format!("{:?}", left.player_id).cmp(&format!("{:?}", right.player_id))
            });
            serialize_to_json_value(&CoreStatsSnapshotExport {
                team_zero: calculator.team_zero_stats().into(),
                team_one: calculator.team_one_stats().into(),
                player_stats,
            })?
        }
        "backboard" => {
            let calculator = graph_state::<BackboardCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "ceiling_shot" => {
            let calculator = graph_state::<CeilingShotCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "center" => {
            let calculator = graph_state::<CenterCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "double_tap" => {
            let calculator = graph_state::<DoubleTapCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "one_timer" => {
            let calculator = graph_state::<OneTimerCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "half_volley" => {
            let calculator = graph_state::<HalfVolleyCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "pass" => {
            let calculator = graph_state::<PassCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "aerial_goal" | "high_aerial_goal" | "long_distance_goal" | "own_half_goal"
        | "empty_net_goal" | "counter_attack_goal" | "flick_goal" | "one_timer_goal" | "air_dribble_goal"
        | "flip_reset_goal" | "half_volley_goal" => {
            serialize_to_json_value(&serde_json::json!({}))?
        }
        "fifty_fifty" => {
            let calculator = graph_state::<FiftyFiftyCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerStatsExport {
                stats: calculator.stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "possession" => {
            let calculator = graph_state::<PossessionCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })?
        }
        "pressure" => {
            let calculator = graph_state::<PressureCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })?
        }
        "rotation" => {
            let calculator = graph_state::<RotationCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "rush" => {
            let calculator = graph_state::<RushCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: calculator.stats(),
            })?
        }
        "touch" => {
            let calculator = graph_state::<TouchCalculator>(graph, module_name)?;
            let player_stats = calculator
                .player_stats()
                .iter()
                .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                    player_id: player_id.clone(),
                    stats: stats.clone().with_complete_labeled_touch_counts(),
                })
                .collect();
            serialize_to_json_value(&OwnedPlayerStatsExport { player_stats })?
        }
        "whiff" => {
            let calculator = graph_state::<WhiffCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "wavedash" => {
            let calculator = graph_state::<WavedashCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "speed_flip" => {
            let calculator = graph_state::<SpeedFlipCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "half_flip" => {
            let calculator = graph_state::<HalfFlipCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "flick" => {
            let calculator = graph_state::<FlickCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "musty_flick" => {
            let calculator = graph_state::<MustyFlickCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "dodge_reset" => {
            let calculator = graph_state::<DodgeResetCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "ball_carry" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "air_dribble" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_air_dribble_stats(),
                team_one: calculator.team_one_air_dribble_stats(),
                player_stats: player_stats_entries(calculator.player_air_dribble_stats()),
            })?
        }
        "boost" => {
            let calculator = graph_state::<BoostCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "bump" => {
            let calculator = graph_state::<BumpCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "movement" => {
            let calculator = graph_state::<MovementCalculator>(graph, module_name)?;
            let player_stats = calculator
                .player_stats()
                .iter()
                .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                    player_id: player_id.clone(),
                    stats: stats.clone().with_complete_labeled_tracked_time(),
                })
                .collect();
            serialize_to_json_value(&TeamOwnedPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats,
            })?
        }
        "positioning" => {
            let calculator = graph_state::<PositioningCalculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "powerslide" => {
            let calculator = graph_state::<PowerslideCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        "demo" => {
            let calculator = graph_state::<DemoCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
        }
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                module_name.to_owned(),
            ))
        }
    };
    Ok(Some(value))
}

pub(crate) fn builtin_snapshot_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    let value = match module_name {
        "positioning" => {
            let calculator = graph_state::<PositioningCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "most_back_forward_threshold_y": calculator.config().most_back_forward_threshold_y,
                "level_ball_depth_margin": calculator.config().level_ball_depth_margin,
            }))?)
        }
        "pressure" => {
            let calculator = graph_state::<PressureCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "pressure_neutral_zone_half_width_y": calculator.config().neutral_zone_half_width_y,
            }))?)
        }
        "rotation" => {
            let calculator = graph_state::<RotationCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "role_depth_margin": calculator.config().role_depth_margin,
                "first_man_ambiguity_margin": calculator.config().first_man_ambiguity_margin,
                "first_man_debounce_seconds": calculator.config().first_man_debounce_seconds,
            }))?)
        }
        "rush" => {
            let calculator = graph_state::<RushCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "rush_max_start_y": calculator.config().max_start_y,
                "rush_attack_support_distance_y": calculator.config().attack_support_distance_y,
                "rush_defender_distance_y": calculator.config().defender_distance_y,
                "rush_min_possession_retained_seconds": calculator.config().min_possession_retained_seconds,
            }))?)
        }
        "aerial_goal" => {
            let calculator = graph_state::<AerialGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "aerial_goal_min_ball_z": calculator.config().min_ball_z,
            }))?)
        }
        "high_aerial_goal" => {
            let calculator = graph_state::<HighAerialGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "high_aerial_goal_min_ball_z": calculator.config().min_ball_z,
            }))?)
        }
        "long_distance_goal" => {
            let calculator = graph_state::<LongDistanceGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "long_distance_goal_max_attacking_y": calculator.config().max_attacking_y,
            }))?)
        }
        "own_half_goal" => {
            let calculator = graph_state::<OwnHalfGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "own_half_goal_max_attacking_y": calculator.config().max_attacking_y,
            }))?)
        }
        "empty_net_goal" => {
            let calculator = graph_state::<EmptyNetGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "empty_net_min_defender_y_margin": calculator.config().min_defender_y_margin,
                "empty_net_min_defender_distance": calculator.config().min_defender_distance,
                "empty_net_max_touch_attacking_y": calculator.config().max_touch_attacking_y,
            }))?)
        }
        "flick_goal" => {
            let calculator = graph_state::<FlickGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "flick_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "one_timer_goal" => {
            let calculator = graph_state::<OneTimerGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "one_timer_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "air_dribble_goal" => {
            let calculator = graph_state::<AirDribbleGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "air_dribble_goal_max_end_to_goal_seconds": calculator.config().max_end_to_goal_seconds,
            }))?)
        }
        "flip_reset_goal" => {
            let calculator = graph_state::<FlipResetGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "flip_reset_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "half_volley_goal" => {
            let calculator = graph_state::<HalfVolleyGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "half_volley_goal_max_touch_to_goal_seconds": calculator.config().max_touch_to_goal_seconds,
                "half_volley_goal_min_goal_alignment": calculator.config().min_goal_alignment,
            }))?)
        }
        "half_volley" => {
            let calculator = graph_state::<HalfVolleyCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "half_volley_max_bounce_to_touch_seconds": calculator.config().max_bounce_to_touch_seconds,
                "half_volley_min_ball_speed": calculator.config().min_ball_speed,
            }))?)
        }
        "core" | "backboard" | "ceiling_shot" | "center" | "double_tap" | "one_timer" | "pass"
        | "fifty_fifty" | "possession" | "touch" | "whiff" | "wavedash" | "speed_flip"
        | "half_flip" | "flick" | "musty_flick" | "dodge_reset" | "ball_carry" | "boost"
        | "bump" | "movement" | "powerslide" | "demo" | "counter_attack_goal" => None,
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                module_name.to_owned(),
            ))
        }
    };
    Ok(value)
}
