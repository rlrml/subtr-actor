use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

use crate::stats::analysis_nodes::analysis_graph::AnalysisGraph;
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
    kickoff_goal_count: u32,
    short_goal_count: u32,
    medium_goal_count: u32,
    long_goal_count: u32,
    goal_times: Vec<f32>,
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
            kickoff_goal_count: stats.scoring_context.goal_after_kickoff.kickoff_goal_count,
            short_goal_count: stats.scoring_context.goal_after_kickoff.short_goal_count,
            medium_goal_count: stats.scoring_context.goal_after_kickoff.medium_goal_count,
            long_goal_count: stats.scoring_context.goal_after_kickoff.long_goal_count,
            goal_times: stats
                .scoring_context
                .goal_after_kickoff
                .goal_times()
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
        "double_tap",
        "fifty_fifty",
        "possession",
        "pressure",
        "rush",
        "touch",
        "speed_flip",
        "musty_flick",
        "dodge_reset",
        "ball_carry",
        "boost",
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
        "double_tap" => {
            let calculator = graph_state::<DoubleTapCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
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
        "speed_flip" => {
            let calculator = graph_state::<SpeedFlipCalculator>(graph, module_name)?;
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
            serialize_to_json_value(&PlayerStatsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
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
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
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
        "double_tap" => {
            let calculator = graph_state::<DoubleTapCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })?
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
        "speed_flip" => {
            let calculator = graph_state::<SpeedFlipCalculator>(graph, module_name)?;
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
        "boost" => {
            let calculator = graph_state::<BoostCalculator>(graph, module_name)?;
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
            }))?)
        }
        "pressure" => {
            let calculator = graph_state::<PressureCalculator>(graph, module_name)?;
            Some(serialize_to_json_value(&serde_json::json!({
                "pressure_neutral_zone_half_width_y": calculator.config().neutral_zone_half_width_y,
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
        "core" | "backboard" | "ceiling_shot" | "double_tap" | "fifty_fifty" | "possession"
        | "touch" | "speed_flip" | "musty_flick" | "dodge_reset" | "ball_carry" | "boost"
        | "movement" | "powerslide" | "demo" => None,
        _ => {
            return SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
                module_name.to_owned(),
            ))
        }
    };
    Ok(value)
}
