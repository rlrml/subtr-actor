use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::stats::analysis_graph::{graph_with_builtin_analysis_nodes, AnalysisGraph};
use crate::{
    build_legacy_timeline_graph, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult,
};

use super::super::builtins::{builtin_module_json, builtin_stats_module_names};
use super::super::types::CollectedStatsModule;

use super::BuiltinModuleSelection;

impl BuiltinModuleSelection {
    pub(super) fn all() -> Self {
        Self {
            module_names: builtin_stats_module_names().to_vec(),
        }
    }

    pub(super) fn from_names<I, S>(module_names: I) -> SubtrActorResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut selected = Vec::new();
        let mut seen = HashSet::new();
        for module_name in module_names {
            let module_name = module_name.as_ref();
            let resolved_name = builtin_stats_module_names()
                .iter()
                .copied()
                .find(|candidate| *candidate == module_name)
                .ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::UnknownStatsModuleName(
                        module_name.to_owned(),
                    ))
                })?;
            if seen.insert(resolved_name) {
                selected.push(resolved_name);
            }
        }
        Ok(Self {
            module_names: selected,
        })
    }

    pub(super) fn graph(&self) -> SubtrActorResult<AnalysisGraph> {
        if self.module_names == builtin_stats_module_names() {
            return Ok(build_legacy_timeline_graph());
        }
        graph_with_builtin_analysis_nodes(self.module_names.iter().copied())
    }

    pub(super) fn collected_modules(
        &self,
        graph: &AnalysisGraph,
    ) -> SubtrActorResult<Vec<CollectedStatsModule>> {
        self.module_names
            .iter()
            .copied()
            .map(|module_name| {
                Ok(CollectedStatsModule {
                    name: module_name,
                    value: builtin_module_json(module_name, graph)?,
                })
            })
            .collect()
    }

    pub(super) fn modules_json(
        &self,
        graph: &AnalysisGraph,
    ) -> SubtrActorResult<Map<String, Value>> {
        let mut modules = Map::new();
        for module_name in self.module_names.iter().copied() {
            modules.insert(
                module_name.to_owned(),
                builtin_module_json(module_name, graph)?,
            );
        }
        Ok(modules)
    }
}
