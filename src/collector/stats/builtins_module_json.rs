use super::*;

#[path = "builtins_module_json_ball_carry.rs"]
mod builtins_module_json_ball_carry;
#[path = "builtins_module_json_core.rs"]
mod builtins_module_json_core;
#[path = "builtins_module_json_events.rs"]
mod builtins_module_json_events;
#[path = "builtins_module_json_game_state.rs"]
mod builtins_module_json_game_state;
#[path = "builtins_module_json_player.rs"]
mod builtins_module_json_player;
#[path = "builtins_module_json_team_player.rs"]
mod builtins_module_json_team_player;

use builtins_module_json_ball_carry::ball_carry_module_json;
use builtins_module_json_core::core_module_json;
use builtins_module_json_events::events_module_json;
use builtins_module_json_game_state::game_state_module_json;
use builtins_module_json_player::player_module_json;
use builtins_module_json_team_player::team_player_module_json;

pub(crate) fn builtin_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    if let Some(value) = core_module_json(module_name, graph)? {
        return Ok(value);
    }
    if let Some(value) = team_player_module_json(module_name, graph)? {
        return Ok(value);
    }
    if let Some(value) = events_module_json(module_name, graph)? {
        return Ok(value);
    }
    if let Some(value) = game_state_module_json(module_name, graph)? {
        return Ok(value);
    }
    if let Some(value) = player_module_json(module_name, graph)? {
        return Ok(value);
    }
    if let Some(value) = ball_carry_module_json(module_name, graph)? {
        return Ok(value);
    }

    SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
        module_name.to_owned(),
    ))
}

pub fn builtin_stats_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    builtin_module_json(module_name, graph)
}
