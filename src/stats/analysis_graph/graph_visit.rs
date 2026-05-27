use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use crate::SubtrActorResult;

use super::{analysis_node_graph_error, AnalysisGraph};

impl AnalysisGraph {
    pub(super) fn visit_node(
        &self,
        index: usize,
        providers: &HashMap<TypeId, usize>,
        visiting: &mut HashSet<usize>,
        visited: &mut HashSet<usize>,
        order: &mut Vec<usize>,
        stack: &mut Vec<&'static str>,
    ) -> SubtrActorResult<()> {
        if visited.contains(&index) {
            return Ok(());
        }
        if !visiting.insert(index) {
            stack.push(self.nodes[index].name());
            let cycle = stack.join(" -> ");
            stack.pop();
            return Err(analysis_node_graph_error(format!(
                "Cycle detected in analysis node graph: {cycle}"
            )));
        }

        stack.push(self.nodes[index].name());
        for dependency in self.nodes[index].dependencies() {
            if self
                .declared_root_states
                .contains_key(&dependency.state_type_id())
                || self
                    .declared_input_states
                    .contains_key(&dependency.state_type_id())
            {
                continue;
            }

            let Some(dependency_index) = providers.get(&dependency.state_type_id()).copied() else {
                stack.pop();
                return Err(analysis_node_graph_error(format!(
                    "Node '{}' depends on missing state {}",
                    self.nodes[index].name(),
                    dependency.state_type_name(),
                )));
            };
            self.visit_node(dependency_index, providers, visiting, visited, order, stack)?;
        }
        stack.pop();

        visiting.remove(&index);
        visited.insert(index);
        order.push(index);
        Ok(())
    }
}
