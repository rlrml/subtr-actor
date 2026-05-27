use super::*;

#[path = "builtins_snapshot_frame_core.rs"]
mod builtins_snapshot_frame_core;
#[path = "builtins_snapshot_frame_core_player.rs"]
mod builtins_snapshot_frame_core_player;
#[path = "builtins_snapshot_frame_core_player_from.rs"]
mod builtins_snapshot_frame_core_player_from;
#[path = "builtins_snapshot_frame_core_team.rs"]
mod builtins_snapshot_frame_core_team;
#[path = "builtins_snapshot_frame_empty.rs"]
mod builtins_snapshot_frame_empty;
#[path = "builtins_snapshot_frame_owned.rs"]
mod builtins_snapshot_frame_owned;
#[path = "builtins_snapshot_frame_player.rs"]
mod builtins_snapshot_frame_player;
#[path = "builtins_snapshot_frame_stats.rs"]
mod builtins_snapshot_frame_stats;
#[path = "builtins_snapshot_frame_team.rs"]
mod builtins_snapshot_frame_team;
#[path = "builtins_snapshot_frame_values.rs"]
mod builtins_snapshot_frame_values;

use builtins_snapshot_frame_core::core_snapshot_frame_json;
use builtins_snapshot_frame_empty::empty_goal_tag_snapshot_frame_json;
use builtins_snapshot_frame_owned::owned_snapshot_frame_json;
use builtins_snapshot_frame_player::player_snapshot_frame_json;
use builtins_snapshot_frame_stats::stats_snapshot_frame_json;
use builtins_snapshot_frame_team::team_snapshot_frame_json;

pub fn builtin_stats_module_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    replay_meta: &ReplayMeta,
) -> SubtrActorResult<Value> {
    Ok(builtin_snapshot_frame_json(module_name, graph, replay_meta)?.unwrap_or(Value::Null))
}

pub(crate) fn builtin_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    for snapshot in [
        core_snapshot_frame_json,
        team_snapshot_frame_json,
        player_snapshot_frame_json,
        stats_snapshot_frame_json,
        owned_snapshot_frame_json,
        empty_goal_tag_snapshot_frame_json,
    ] {
        if let Some(value) = snapshot(module_name, graph, replay_meta)? {
            return Ok(Some(value));
        }
    }
    SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
        module_name.to_owned(),
    ))
}
