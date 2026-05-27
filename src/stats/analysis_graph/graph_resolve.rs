use std::collections::HashSet;

use crate::SubtrActorResult;

use super::{analysis_node_graph_error, AnalysisGraph, AnalysisNodeDyn};

impl AnalysisGraph {
    pub fn resolve(&mut self) -> SubtrActorResult<()> {
        if self.resolved {
            return Ok(());
        }

        loop {
            let providers = self.provider_index_by_type()?;
            let mut additions = Vec::new();
            let mut queued_types = HashSet::new();

            for node in &self.nodes {
                for dependency in node.dependencies() {
                    if providers.contains_key(&dependency.state_type_id())
                        || self
                            .declared_root_states
                            .contains_key(&dependency.state_type_id())
                        || self
                            .declared_input_states
                            .contains_key(&dependency.state_type_id())
                    {
                        continue;
                    }
                    if dependency.is_external() {
                        return Err(analysis_node_graph_error(format!(
                            "Node '{}' requires state {} with no provider",
                            node.name(),
                            dependency.state_type_name(),
                        )));
                    }
                    let default_factory = dependency.default_factory();
                    if queued_types.insert(dependency.state_type_id()) {
                        additions.push(default_factory());
                    }
                }
            }

            if additions.is_empty() {
                break;
            }

            self.nodes.extend(additions);
        }

        let providers = self.provider_index_by_type()?;
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut order = Vec::with_capacity(self.nodes.len());

        for index in 0..self.nodes.len() {
            self.visit_node(
                index,
                &providers,
                &mut visiting,
                &mut visited,
                &mut order,
                &mut Vec::new(),
            )?;
        }

        self.reorder_nodes(order);
        self.evaluation_order = (0..self.nodes.len()).collect();
        self.resolved = true;
        Ok(())
    }

    fn reorder_nodes(&mut self, order: Vec<usize>) {
        let mut ordered_nodes = Vec::with_capacity(self.nodes.len());
        let mut original_nodes: Vec<Option<Box<dyn AnalysisNodeDyn>>> =
            std::mem::take(&mut self.nodes)
                .into_iter()
                .map(Some)
                .collect();
        for index in order {
            ordered_nodes.push(
                original_nodes[index]
                    .take()
                    .expect("topological order should only reference each node once"),
            );
        }
        self.nodes = ordered_nodes;
    }
}
