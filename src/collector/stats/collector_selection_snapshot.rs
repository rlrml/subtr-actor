use serde_json::{Map, Value};

use crate::stats::analysis_graph::AnalysisGraph;
use crate::{
    FrameInfo, GameplayState, LivePlayState, ReplayMeta, StatsSnapshotFrame, SubtrActorError,
    SubtrActorErrorVariant, SubtrActorResult,
};

use super::super::builtins::{builtin_snapshot_config_json, builtin_snapshot_frame_json};
use super::BuiltinModuleSelection;

impl BuiltinModuleSelection {
    pub(super) fn frame_modules_json(
        &self,
        graph: &AnalysisGraph,
        replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<Map<String, Value>> {
        let mut modules = Map::new();
        for module_name in self.module_names.iter().copied() {
            if let Some(snapshot) = builtin_snapshot_frame_json(module_name, graph, replay_meta)? {
                modules.insert(module_name.to_owned(), snapshot);
            }
            if module_name == "ball_carry" {
                if let Some(snapshot) =
                    builtin_snapshot_frame_json("air_dribble", graph, replay_meta)?
                {
                    modules.insert("air_dribble".to_owned(), snapshot);
                }
            }
        }
        Ok(modules)
    }

    pub(super) fn snapshot_config_json(
        &self,
        graph: &AnalysisGraph,
    ) -> SubtrActorResult<Map<String, Value>> {
        let mut config = Map::new();
        for module_name in self.module_names.iter().copied() {
            if let Some(module_config) = builtin_snapshot_config_json(module_name, graph)? {
                config.insert(module_name.to_owned(), module_config);
            }
        }
        Ok(config)
    }

    pub(super) fn snapshot_frame(
        &self,
        graph: &AnalysisGraph,
        replay_meta: &ReplayMeta,
    ) -> SubtrActorResult<StatsSnapshotFrame> {
        let frame = graph.state::<FrameInfo>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing FrameInfo state while snapshotting stats frame".to_owned(),
            ))
        })?;
        let gameplay = graph.state::<GameplayState>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing GameplayState state while snapshotting stats frame".to_owned(),
            ))
        })?;
        let live_play_state = graph.state::<LivePlayState>().cloned().unwrap_or_default();
        Ok(StatsSnapshotFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: gameplay.game_state,
            ball_has_been_hit: gameplay.ball_has_been_hit,
            kickoff_countdown_time: gameplay.kickoff_countdown_time,
            gameplay_phase: live_play_state.gameplay_phase,
            is_live_play: live_play_state.is_live_play,
            modules: self.frame_modules_json(graph, replay_meta)?,
        })
    }
}
