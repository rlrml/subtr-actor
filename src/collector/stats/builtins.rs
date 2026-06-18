use std::collections::HashMap;

use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::stats::analysis_graph::{
    AnalysisGraph, StatsProjectionState, StatsTimelineEventsState, StatsTimelineFrameState,
    builtin_analysis_node_names,
};
use crate::*;
use boxcars::{Quaternion, RigidBody, Vector3f};

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
struct TeamPlayerStatsWithCollectedEventsExport<'a, Team, Player, Event> {
    team_zero: &'a Team,
    team_one: &'a Team,
    player_stats: Vec<PlayerStatsEntry<'a, Player>>,
    events: Vec<&'a Event>,
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
    player_events: &'a [CorePlayerScoreboardEvent],
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
    demolitions: &'a [DemolitionEvent],
}

pub fn builtin_stats_module_names() -> &'static [&'static str] {
    &[
        "core",
        "backboard",
        "ceiling_shot",
        "wall_aerial",
        "wall_aerial_shot",
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
        "sustained_pressure_goal",
        "kickoff_goal",
        "flick_goal",
        "ceiling_shot_goal",
        "double_tap_goal",
        "one_timer_goal",
        "passing_goal",
        "air_dribble_goal",
        "flip_reset_goal",
        "flip_into_ball_goal",
        "bump_goal",
        "demo_goal",
        "half_volley_goal",
        "fifty_fifty",
        "kickoff",
        "player_possession",
        "possession",
        "ball_half",
        "ball_third",
        "territorial_pressure",
        "rotation",
        "rush",
        "dodge",
        "touch",
        "whiff",
        "wavedash",
        "speed_flip",
        "half_flip",
        "flick",
        "musty_flick",
        "dodge_reset",
        "ball_carry",
        "controlled_play",
        "air_dribble",
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

fn projected_stats<'a>(
    graph: &'a AnalysisGraph,
    module_name: &str,
) -> SubtrActorResult<&'a StatsProjectionState> {
    graph_state::<StatsProjectionState>(graph, module_name)
}

pub(crate) fn builtin_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    match module_name {
        "core" => {
            let calculator = graph_state::<MatchStatsCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&CoreStatsExport {
                team_zero: projection.core.team_zero_stats(),
                team_one: projection.core.team_one_stats(),
                player_stats: player_stats_entries(projection.core.player_stats()),
                timeline: calculator.timeline(),
                goal_context: calculator.goal_context_events(),
                player_events: calculator.core_player_events(),
            })
        }
        "backboard" => {
            let calculator = graph_state::<BackboardCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.backboard.team_zero_stats(),
                team_one: projection.backboard.team_one_stats(),
                player_stats: player_stats_entries(projection.backboard.player_stats()),
                events: calculator.events(),
            })
        }
        "ceiling_shot" => {
            let calculator = graph_state::<CeilingShotCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.ceiling_shot.player_stats()),
                events: calculator.events(),
            })
        }
        "wall_aerial" => {
            let calculator = graph_state::<WallAerialCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.wall_aerial.player_stats()),
                events: calculator.events(),
            })
        }
        "wall_aerial_shot" => {
            let calculator = graph_state::<WallAerialShotCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.wall_aerial_shot.player_stats()),
                events: calculator.events(),
            })
        }
        "center" => {
            let calculator = graph_state::<CenterCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.center.team_zero_stats(),
                team_one: projection.center.team_one_stats(),
                player_stats: player_stats_entries(projection.center.player_stats()),
                events: calculator.events(),
            })
        }
        "double_tap" => {
            let calculator = graph_state::<DoubleTapCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.double_tap.team_zero_stats(),
                team_one: projection.double_tap.team_one_stats(),
                player_stats: player_stats_entries(projection.double_tap.player_stats()),
                events: calculator.events(),
            })
        }
        "one_timer" => {
            let calculator = graph_state::<OneTimerCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.one_timer.team_zero_stats(),
                team_one: projection.one_timer.team_one_stats(),
                player_stats: player_stats_entries(projection.one_timer.player_stats()),
                events: calculator.events(),
            })
        }
        "half_volley" => {
            let calculator = graph_state::<HalfVolleyCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.half_volley.team_zero_stats(),
                team_one: projection.half_volley.team_one_stats(),
                player_stats: player_stats_entries(projection.half_volley.player_stats()),
                events: calculator.events(),
            })
        }
        "pass" => {
            let calculator = graph_state::<PassCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.pass.team_zero_stats(),
                team_one: projection.pass.team_one_stats(),
                player_stats: player_stats_entries(projection.pass.player_stats()),
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
        "sustained_pressure_goal" => {
            let calculator = graph_state::<SustainedPressureGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "kickoff_goal" => {
            let calculator = graph_state::<KickoffGoalCalculator>(graph, module_name)?;
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
        "ceiling_shot_goal" => {
            let calculator = graph_state::<CeilingShotGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "double_tap_goal" => {
            let calculator = graph_state::<DoubleTapGoalCalculator>(graph, module_name)?;
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
        "passing_goal" => {
            let calculator = graph_state::<PassingGoalCalculator>(graph, module_name)?;
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
        "flip_into_ball_goal" => {
            let calculator = graph_state::<FlipIntoBallGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "bump_goal" => {
            let calculator = graph_state::<BumpGoalCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "demo_goal" => {
            let calculator = graph_state::<DemoGoalCalculator>(graph, module_name)?;
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
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerEventsExport {
                stats: projection.fifty_fifty.stats(),
                player_stats: player_stats_entries(projection.fifty_fifty.player_stats()),
                events: calculator.events(),
            })
        }
        "kickoff" => {
            let calculator = graph_state::<KickoffCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerEventsExport {
                stats: projection.kickoff.stats(),
                player_stats: player_stats_entries(projection.kickoff.player_stats()),
                events: calculator.events(),
            })
        }
        "possession" => {
            let calculator = graph_state::<PossessionCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsWithEventsExport {
                stats: projection.possession.stats(),
                events: calculator.events(),
            })
        }
        "player_possession" => {
            let calculator = graph_state::<PlayerPossessionCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "ball_half" => {
            let calculator = graph_state::<BallHalfCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsWithEventsExport {
                stats: projection.ball_half.stats(),
                events: calculator.events(),
            })
        }
        "ball_third" => {
            let calculator = graph_state::<BallThirdCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsWithEventsExport {
                stats: projection.ball_third.stats(),
                events: calculator.events(),
            })
        }
        "territorial_pressure" => {
            let calculator = graph_state::<TerritorialPressureCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "stats": projection.territorial_pressure.stats(),
                "events": calculator.events(),
            }))
        }
        "rotation" => {
            let calculator = graph_state::<RotationCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "team_zero": projection.rotation.team_zero_stats(),
                "team_one": projection.rotation.team_one_stats(),
                "player_stats": player_stats_entries(projection.rotation.player_stats()),
                "role_events": calculator.role_events(),
                "first_man_change_events": calculator.first_man_change_events(),
            }))
        }
        "rush" => {
            let calculator = graph_state::<RushCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsWithEventsExport {
                stats: projection.rush.stats(),
                events: calculator.events(),
            })
        }
        "dodge" | "flip_impulse" => {
            let calculator = graph_state::<FlipImpulseCalculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
        }
        "touch" => {
            let calculator = graph_state::<TouchCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "player_stats": player_stats_entries(projection.touch.player_stats()),
                "events": calculator.events(),
            }))
        }
        "whiff" => {
            let calculator = graph_state::<WhiffCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.whiff.player_stats()),
                events: calculator.events(),
            })
        }
        "wavedash" => {
            let calculator = graph_state::<WavedashCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.wavedash.player_stats()),
                events: calculator.events(),
            })
        }
        "speed_flip" => {
            let calculator = graph_state::<SpeedFlipCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.speed_flip.player_stats()),
                events: calculator.events(),
            })
        }
        "half_flip" => {
            let calculator = graph_state::<HalfFlipCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.half_flip.player_stats()),
                events: calculator.events(),
            })
        }
        "flick" => {
            let calculator = graph_state::<FlickCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.flick.player_stats()),
                events: calculator.events(),
            })
        }
        "musty_flick" => {
            let calculator = graph_state::<MustyFlickCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(projection.musty_flick.player_stats()),
                events: calculator.events(),
            })
        }
        "dodge_reset" => {
            let calculator = graph_state::<DodgeResetCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "player_stats": player_stats_entries(projection.dodge_reset.player_stats()),
                "events": calculator.events(),
            }))
        }
        "ball_carry" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.ball_carry.team_zero_stats(),
                team_one: projection.ball_carry.team_one_stats(),
                player_stats: player_stats_entries(projection.ball_carry.player_stats()),
                events: calculator.carry_events(),
            })
        }
        "controlled_play" => {
            let calculator = graph_state::<ControlledPlayCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.controlled_play.team_zero_stats(),
                team_one: projection.controlled_play.team_one_stats(),
                player_stats: player_stats_entries(projection.controlled_play.player_stats()),
                events: calculator.events(),
            })
        }
        "air_dribble" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            let events = calculator
                .carry_events()
                .iter()
                .filter(|event| event.kind == BallCarryKind::AirDribble)
                .collect::<Vec<_>>();
            serialize_to_json_value(&TeamPlayerStatsWithCollectedEventsExport {
                team_zero: projection.ball_carry.team_zero_air_dribble_stats(),
                team_one: projection.ball_carry.team_one_air_dribble_stats(),
                player_stats: player_stats_entries(
                    projection.ball_carry.player_air_dribble_stats(),
                ),
                events,
            })
        }
        "boost" => {
            let calculator = graph_state::<BoostCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "team_zero": projection.boost.team_zero_stats(),
                "team_one": projection.boost.team_one_stats(),
                "player_stats": player_stats_entries(projection.boost.player_stats()),
                "pickup_events": calculator.pickup_events(),
                "respawn_events": calculator.respawn_events(),
                "accumulation_tracks": calculator.accumulation_tracks(),
            }))
        }
        "bump" => {
            let calculator = graph_state::<BumpCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.bump.team_zero_stats(),
                team_one: projection.bump.team_one_stats(),
                player_stats: player_stats_entries(projection.bump.player_stats()),
                events: calculator.events(),
            })
        }
        "movement" => {
            let calculator = graph_state::<MovementCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.movement.team_zero_stats(),
                team_one: projection.movement.team_one_stats(),
                player_stats: player_stats_entries(projection.movement.player_stats()),
                events: calculator.events(),
            })
        }
        "positioning" => {
            let calculator = graph_state::<PositioningCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "team_zero": projection.positioning.team_zero_stats(),
                "team_one": projection.positioning.team_one_stats(),
                "player_stats": player_stats_entries(projection.positioning.player_stats()),
                "activity_events": calculator.activity_events(),
                "field_third_events": calculator.field_third_events(),
                "field_half_events": calculator.field_half_events(),
                "ball_depth_events": calculator.ball_depth_events(),
                "depth_role_events": calculator.depth_role_events(),
                "ball_proximity_events": calculator.ball_proximity_events(),
            }))
        }
        "powerslide" => {
            let calculator = graph_state::<PowerslideCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: projection.powerslide.team_zero_stats(),
                team_one: projection.powerslide.team_one_stats(),
                player_stats: player_stats_entries(projection.powerslide.player_stats()),
                events: calculator.events(),
            })
        }
        "demo" => {
            let calculator = graph_state::<DemoCalculator>(graph, module_name)?;
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&DemoStatsExport {
                team_zero: projection.demo.team_zero_stats(),
                team_one: projection.demo.team_one_stats(),
                player_stats: player_stats_entries(projection.demo.player_stats()),
                demolitions: calculator.events(),
            })
        }
        _ => SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
            module_name.to_owned(),
        )),
    }
}

pub fn builtin_stats_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    builtin_module_json(module_name, graph)
}

pub fn builtin_stats_module_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    replay_meta: &ReplayMeta,
) -> SubtrActorResult<Value> {
    Ok(builtin_snapshot_frame_json(module_name, graph, replay_meta)?.unwrap_or(Value::Null))
}

pub fn builtin_stats_module_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    Ok(builtin_snapshot_config_json(module_name, graph)?.unwrap_or(Value::Null))
}

fn vec3_json(value: &Vector3f) -> Value {
    json!({
        "x": value.x,
        "y": value.y,
        "z": value.z,
    })
}

fn quat_json(value: &Quaternion) -> Value {
    json!({
        "x": value.x,
        "y": value.y,
        "z": value.z,
        "w": value.w,
    })
}

fn rigid_body_json(value: &RigidBody) -> Value {
    json!({
        "location": vec3_json(&value.location),
        "rotation": quat_json(&value.rotation),
        "sleeping": value.sleeping,
        "linear_velocity": value.linear_velocity.as_ref().map(vec3_json),
        "angular_velocity": value.angular_velocity.as_ref().map(vec3_json),
    })
}

fn ball_frame_state_json(state: &BallFrameState) -> Value {
    match state {
        BallFrameState::Missing => json!({
            "kind": "Missing",
            "ball": Value::Null,
        }),
        BallFrameState::Present(ball) => json!({
            "kind": "Present",
            "ball": ball_sample_json(ball),
        }),
    }
}

fn ball_sample_json(sample: &BallSample) -> Value {
    json!({
        "rigid_body": rigid_body_json(&sample.rigid_body),
    })
}

fn player_sample_json(sample: &PlayerSample) -> Value {
    json!({
        "player_id": sample.player_id,
        "is_team_0": sample.is_team_0,
        "rigid_body": sample.rigid_body.as_ref().map(rigid_body_json),
        "boost_amount": sample.boost_amount,
        "last_boost_amount": sample.last_boost_amount,
        "boost_active": sample.boost_active,
        "dodge_active": sample.dodge_active,
        "powerslide_active": sample.powerslide_active,
        "match_goals": sample.match_goals,
        "match_assists": sample.match_assists,
        "match_saves": sample.match_saves,
        "match_shots": sample.match_shots,
        "match_score": sample.match_score,
    })
}

fn demo_event_sample_json(sample: &DemoEventSample) -> Value {
    json!({
        "attacker": sample.attacker,
        "victim": sample.victim,
    })
}

fn vertical_band_label(band: PlayerVerticalBand) -> &'static str {
    match band {
        PlayerVerticalBand::Ground => "ground",
        PlayerVerticalBand::LowAir => "low_air",
        PlayerVerticalBand::HighAir => "high_air",
    }
}

fn player_vertical_state_json(state: &PlayerVerticalState) -> Value {
    let mut players = state
        .players
        .iter()
        .map(|(player_id, sample)| {
            json!({
                "player_id": player_id,
                "height": sample.height,
                "band": vertical_band_label(sample.band),
            })
        })
        .collect::<Vec<_>>();
    players.sort_by_key(|value| value["player_id"].to_string());
    json!({ "players": players })
}

fn settings_json(calculator: &SettingsCalculator) -> Value {
    let mut player_settings = calculator
        .player_settings()
        .iter()
        .map(|(player_id, settings)| {
            json!({
                "player_id": player_id,
                "settings": {
                    "steering_sensitivity": settings.steering_sensitivity,
                    "camera_fov": settings.camera_fov,
                    "camera_height": settings.camera_height,
                    "camera_pitch": settings.camera_pitch,
                    "camera_distance": settings.camera_distance,
                    "camera_stiffness": settings.camera_stiffness,
                    "camera_swivel_speed": settings.camera_swivel_speed,
                    "camera_transition_speed": settings.camera_transition_speed,
                },
            })
        })
        .collect::<Vec<_>>();
    player_settings.sort_by_key(|value| value["player_id"].to_string());
    json!({ "player_settings": player_settings })
}

pub fn builtin_analysis_node_json(
    node_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    let value = match node_name {
        "core" | "match_stats" => builtin_module_json("core", graph)?,
        "stats_timeline_events" => serialize_to_json_value(
            &graph_state::<StatsTimelineEventsState>(graph, node_name)?.events,
        )?,
        "stats_timeline_frame" => graph_state::<StatsTimelineFrameState>(graph, node_name)?
            .frame
            .as_ref()
            .map(serialize_to_json_value)
            .transpose()?
            .unwrap_or(Value::Null),
        "stats_projection" => {
            let _ = graph_state::<StatsProjectionState>(graph, node_name)?;
            json!({
                "projected_stats_module_names": builtin_stats_module_names(),
            })
        }
        "frame_info" => {
            let state = graph_state::<FrameInfo>(graph, node_name)?;
            json!({
                "frame_number": state.frame_number,
                "time": state.time,
                "dt": state.dt,
                "seconds_remaining": state.seconds_remaining,
            })
        }
        "gameplay_state" => {
            let state = graph_state::<GameplayState>(graph, node_name)?;
            json!({
                "game_state": state.game_state,
                "ball_has_been_hit": state.ball_has_been_hit,
                "kickoff_countdown_time": state.kickoff_countdown_time,
                "team_zero_score": state.team_zero_score,
                "team_one_score": state.team_one_score,
                "possession_team_is_team_0": state.possession_team_is_team_0,
                "scored_on_team_is_team_0": state.scored_on_team_is_team_0,
                "current_in_game_team_player_counts": state.current_in_game_team_player_counts,
                "is_live_play": state.is_live_play(),
                "kickoff_phase_active": state.kickoff_phase_active(),
            })
        }
        "ball_frame_state" => {
            ball_frame_state_json(graph_state::<BallFrameState>(graph, node_name)?)
        }
        "player_frame_state" => {
            let state = graph_state::<PlayerFrameState>(graph, node_name)?;
            json!({
                "players": state.players.iter().map(player_sample_json).collect::<Vec<_>>(),
            })
        }
        "frame_events_state" => {
            let state = graph_state::<FrameEventsState>(graph, node_name)?;
            json!({
                "active_demos": state.active_demos.iter().map(demo_event_sample_json).collect::<Vec<_>>(),
                "demo_events": state.demo_events,
                "boost_pad_events": state.boost_pad_events,
                "touch_events": state.touch_events,
                "dodge_refreshed_events": state.dodge_refreshed_events,
                "player_stat_events": state.player_stat_events,
                "goal_events": state.goal_events,
            })
        }
        "live_play" => serialize_to_json_value(graph_state::<LivePlayState>(graph, node_name)?)?,
        "touch_state" => {
            let state = graph_state::<TouchState>(graph, node_name)?;
            json!({
                "touch_events": state.touch_events,
                "last_touch": state.last_touch,
                "last_touch_player": state.last_touch_player,
                "last_touch_team_is_team_0": state.last_touch_team_is_team_0,
            })
        }
        "possession_state" => {
            let state = graph_state::<PossessionState>(graph, node_name)?;
            json!({
                "active_team_before_sample": state.active_team_before_sample,
                "current_team_is_team_0": state.current_team_is_team_0,
                "active_player_before_sample": state.active_player_before_sample,
                "current_player": state.current_player,
            })
        }
        "backboard_bounce_state" => {
            let state = graph_state::<BackboardBounceState>(graph, node_name)?;
            json!({
                "bounce_events": state.bounce_events,
                "last_bounce_event": state.last_bounce_event,
            })
        }
        "continuous_ball_control" => {
            let state = graph_state::<ContinuousBallControlState>(graph, node_name)?;
            json!({
                "completed_sequences": state.completed_sequences.iter().map(|sequence| {
                    json!({
                        "player_id": sequence.player_id,
                        "is_team_0": sequence.is_team_0,
                        "kind": sequence.kind,
                        "start_frame": sequence.start_frame,
                        "end_frame": sequence.end_frame,
                        "start_time": sequence.start_time,
                        "end_time": sequence.end_time,
                        "duration": sequence.duration,
                        "straight_line_distance": sequence.straight_line_distance,
                        "path_distance": sequence.path_distance,
                        "average_horizontal_gap": sequence.average_horizontal_gap,
                        "average_vertical_gap": sequence.average_vertical_gap,
                        "average_speed": sequence.average_speed,
                        "start_position": {
                            "x": sequence.start_position.x,
                            "y": sequence.start_position.y,
                            "z": sequence.start_position.z,
                        },
                        "end_position": {
                            "x": sequence.end_position.x,
                            "y": sequence.end_position.y,
                            "z": sequence.end_position.z,
                        },
                        "touch_count": sequence.touch_count,
                        "air_touch_count": sequence.air_touch_count,
                    })
                }).collect::<Vec<_>>(),
            })
        }
        "fifty_fifty_state" => {
            let state = graph_state::<FiftyFiftyState>(graph, node_name)?;
            json!({
                "active_event": state.active_event.as_ref().map(|event| {
                    json!({
                        "start_time": event.start_time,
                        "start_frame": event.start_frame,
                        "last_touch_time": event.last_touch_time,
                        "last_touch_frame": event.last_touch_frame,
                        "is_kickoff": event.is_kickoff,
                        "team_zero_player": event.team_zero_player,
                        "team_one_player": event.team_one_player,
                        "team_zero_position": event.team_zero_position,
                        "team_one_position": event.team_one_position,
                        "midpoint": event.midpoint,
                        "plane_normal": event.plane_normal,
                    })
                }),
                "resolved_events": state.resolved_events,
                "last_resolved_event": state.last_resolved_event,
            })
        }
        "player_vertical_state" => {
            player_vertical_state_json(graph_state::<PlayerVerticalState>(graph, node_name)?)
        }
        "settings" => settings_json(graph_state::<SettingsCalculator>(graph, node_name)?),
        module_name if builtin_stats_module_names().contains(&module_name) => {
            builtin_module_json(module_name, graph)?
        }
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                node_name.to_owned(),
            ));
        }
    };
    Ok(value)
}

pub fn builtin_analysis_nodes_json(graph: &AnalysisGraph) -> SubtrActorResult<Value> {
    let mut values = Map::new();
    for node_name in builtin_analysis_node_names() {
        values.insert(
            (*node_name).to_owned(),
            builtin_analysis_node_json(node_name, graph)?,
        );
    }
    Ok(Value::Object(values))
}

pub(crate) fn builtin_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    let value = match module_name {
        "core" => {
            let projection = projected_stats(graph, module_name)?;
            let mut player_stats: Vec<_> = projection
                .core
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
                team_zero: projection.core.team_zero_stats().into(),
                team_one: projection.core.team_one_stats().into(),
                player_stats,
            })?
        }
        "backboard" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.backboard.team_zero_stats(),
                team_one: projection.backboard.team_one_stats(),
                player_stats: player_stats_entries(projection.backboard.player_stats()),
            })?
        }
        "ceiling_shot" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.ceiling_shot.player_stats()),
            })?
        }
        "wall_aerial" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.wall_aerial.player_stats()),
            })?
        }
        "wall_aerial_shot" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.wall_aerial_shot.player_stats()),
            })?
        }
        "center" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.center.team_zero_stats(),
                team_one: projection.center.team_one_stats(),
                player_stats: player_stats_entries(projection.center.player_stats()),
            })?
        }
        "double_tap" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.double_tap.team_zero_stats(),
                team_one: projection.double_tap.team_one_stats(),
                player_stats: player_stats_entries(projection.double_tap.player_stats()),
            })?
        }
        "one_timer" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.one_timer.team_zero_stats(),
                team_one: projection.one_timer.team_one_stats(),
                player_stats: player_stats_entries(projection.one_timer.player_stats()),
            })?
        }
        "half_volley" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.half_volley.team_zero_stats(),
                team_one: projection.half_volley.team_one_stats(),
                player_stats: player_stats_entries(projection.half_volley.player_stats()),
            })?
        }
        "pass" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.pass.team_zero_stats(),
                team_one: projection.pass.team_one_stats(),
                player_stats: player_stats_entries(projection.pass.player_stats()),
            })?
        }
        "aerial_goal"
        | "high_aerial_goal"
        | "long_distance_goal"
        | "own_half_goal"
        | "empty_net_goal"
        | "counter_attack_goal"
        | "sustained_pressure_goal"
        | "kickoff_goal"
        | "flick_goal"
        | "ceiling_shot_goal"
        | "double_tap_goal"
        | "one_timer_goal"
        | "passing_goal"
        | "air_dribble_goal"
        | "flip_reset_goal"
        | "flip_into_ball_goal"
        | "bump_goal"
        | "demo_goal"
        | "half_volley_goal" => serialize_to_json_value(&serde_json::json!({}))?,
        "fifty_fifty" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerStatsExport {
                stats: projection.fifty_fifty.stats(),
                player_stats: player_stats_entries(projection.fifty_fifty.player_stats()),
            })?
        }
        "kickoff" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerStatsExport {
                stats: projection.kickoff.stats(),
                player_stats: player_stats_entries(projection.kickoff.player_stats()),
            })?
        }
        "possession" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: projection.possession.stats(),
            })?
        }
        "ball_half" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: projection.ball_half.stats(),
            })?
        }
        "ball_third" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: projection.ball_third.stats(),
            })?
        }
        "territorial_pressure" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: projection.territorial_pressure.stats(),
            })?
        }
        "rotation" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.rotation.team_zero_stats(),
                team_one: projection.rotation.team_one_stats(),
                player_stats: player_stats_entries(projection.rotation.player_stats()),
            })?
        }
        "rush" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&StatsExport {
                stats: projection.rush.stats(),
            })?
        }
        "dodge" | "flip_impulse" | "player_possession" => {
            serialize_to_json_value(&serde_json::json!({}))?
        }
        "touch" => {
            let projection = projected_stats(graph, module_name)?;
            let player_stats = projection
                .touch
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
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.whiff.player_stats()),
            })?
        }
        "wavedash" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.wavedash.player_stats()),
            })?
        }
        "speed_flip" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.speed_flip.player_stats()),
            })?
        }
        "half_flip" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.half_flip.player_stats()),
            })?
        }
        "flick" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.flick.player_stats()),
            })?
        }
        "musty_flick" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.musty_flick.player_stats()),
            })?
        }
        "dodge_reset" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(projection.dodge_reset.player_stats()),
            })?
        }
        "ball_carry" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.ball_carry.team_zero_stats(),
                team_one: projection.ball_carry.team_one_stats(),
                player_stats: player_stats_entries(projection.ball_carry.player_stats()),
            })?
        }
        "controlled_play" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.controlled_play.team_zero_stats(),
                team_one: projection.controlled_play.team_one_stats(),
                player_stats: player_stats_entries(projection.controlled_play.player_stats()),
            })?
        }
        "air_dribble" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.ball_carry.team_zero_air_dribble_stats(),
                team_one: projection.ball_carry.team_one_air_dribble_stats(),
                player_stats: player_stats_entries(
                    projection.ball_carry.player_air_dribble_stats(),
                ),
            })?
        }
        "boost" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.boost.team_zero_stats(),
                team_one: projection.boost.team_one_stats(),
                player_stats: player_stats_entries(projection.boost.player_stats()),
            })?
        }
        "bump" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.bump.team_zero_stats(),
                team_one: projection.bump.team_one_stats(),
                player_stats: player_stats_entries(projection.bump.player_stats()),
            })?
        }
        "movement" => {
            let projection = projected_stats(graph, module_name)?;
            let player_stats = projection
                .movement
                .player_stats()
                .iter()
                .map(|(player_id, stats)| OwnedPlayerStatsEntry {
                    player_id: player_id.clone(),
                    stats: stats.clone().with_complete_labeled_tracked_time(),
                })
                .collect();
            serialize_to_json_value(&TeamOwnedPlayerStatsExport {
                team_zero: projection.movement.team_zero_stats(),
                team_one: projection.movement.team_one_stats(),
                player_stats,
            })?
        }
        "positioning" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.positioning.team_zero_stats(),
                team_one: projection.positioning.team_one_stats(),
                player_stats: player_stats_entries(projection.positioning.player_stats()),
            })?
        }
        "powerslide" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.powerslide.team_zero_stats(),
                team_one: projection.powerslide.team_one_stats(),
                player_stats: player_stats_entries(projection.powerslide.player_stats()),
            })?
        }
        "demo" => {
            let projection = projected_stats(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: projection.demo.team_zero_stats(),
                team_one: projection.demo.team_one_stats(),
                player_stats: player_stats_entries(projection.demo.player_stats()),
            })?
        }
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                module_name.to_owned(),
            ));
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
                "closest_to_ball_switch_margin": calculator.config().closest_to_ball_switch_margin,
                "closest_to_ball_switch_min_seconds": calculator.config().closest_to_ball_switch_min_seconds,
            }))?)
        }
        "ball_half" => {
            let calculator = graph_state::<BallHalfCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "ball_half_neutral_zone_half_width_y": calculator.config().neutral_zone_half_width_y,
            }))?)
        }
        "ball_third" => {
            let calculator = graph_state::<BallThirdCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "ball_third_boundary_y": calculator.config().boundary_y,
            }))?)
        }
        "territorial_pressure" => {
            let calculator = graph_state::<TerritorialPressureCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "territorial_pressure_neutral_zone_half_width_y": calculator.config().neutral_zone_half_width_y,
                "territorial_pressure_min_establish_seconds": calculator.config().min_establish_seconds,
                "territorial_pressure_min_establish_third_seconds": calculator.config().min_establish_third_seconds,
                "territorial_pressure_relief_grace_seconds": calculator.config().relief_grace_seconds,
                "territorial_pressure_confirmed_relief_grace_seconds": calculator.config().confirmed_relief_grace_seconds,
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
        "ceiling_shot_goal" => {
            let calculator = graph_state::<CeilingShotGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "ceiling_shot_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "double_tap_goal" => {
            let calculator = graph_state::<DoubleTapGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "double_tap_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "one_timer_goal" => {
            let calculator = graph_state::<OneTimerGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "one_timer_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "passing_goal" => {
            let calculator = graph_state::<PassingGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "passing_goal_max_pass_to_goal_seconds": calculator.config().max_pass_to_goal_seconds,
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
        "flip_into_ball_goal" => {
            let calculator = graph_state::<FlipIntoBallGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "flip_into_ball_goal_max_touch_to_goal_seconds": calculator.config().max_touch_to_goal_seconds,
            }))?)
        }
        "bump_goal" => {
            let calculator = graph_state::<BumpGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "bump_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            }))?)
        }
        "demo_goal" => {
            let calculator = graph_state::<DemoGoalCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "demo_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
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
        "core"
        | "backboard"
        | "ceiling_shot"
        | "wall_aerial"
        | "wall_aerial_shot"
        | "center"
        | "double_tap"
        | "one_timer"
        | "pass"
        | "fifty_fifty"
        | "kickoff"
        | "player_possession"
        | "possession"
        | "dodge"
        | "flip_impulse"
        | "touch"
        | "whiff"
        | "wavedash"
        | "speed_flip"
        | "half_flip"
        | "flick"
        | "musty_flick"
        | "dodge_reset"
        | "ball_carry"
        | "controlled_play"
        | "air_dribble"
        | "counter_attack_goal"
        | "sustained_pressure_goal"
        | "kickoff_goal"
        | "boost"
        | "bump"
        | "movement"
        | "powerslide"
        | "demo" => None,
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                module_name.to_owned(),
            ));
        }
    };
    Ok(value)
}
