use std::any::TypeId;
use std::collections::HashMap;

use ascii_dag::graph::{Graph as AsciiGraph, RenderMode};

use super::{analysis_node_graph_error, AnalysisGraph};
use crate::*;

impl AnalysisGraph {
    pub fn render_ascii_dag(&mut self) -> SubtrActorResult<String> {
        self.resolve()?;

        if self.nodes.is_empty() {
            return Ok("AnalysisGraph\n\\- (empty)".to_owned());
        }

        let providers = self.provider_index_by_type()?;
        let node_labels: Vec<Box<str>> = self
            .nodes
            .iter()
            .map(|node| node.name().to_owned().into_boxed_str())
            .collect();
        let mut external_labels = Vec::new();
        let mut external_node_ids = HashMap::new();
        let mut next_node_id = self.nodes.len();

        for node in &self.nodes {
            for dependency in node.dependencies() {
                let dependency_type_id = dependency.state_type_id();
                if providers.contains_key(&dependency_type_id) {
                    continue;
                }

                let label = if self.declared_root_states.contains_key(&dependency_type_id) {
                    format!("root:{}", short_type_name(dependency.state_type_name()))
                } else if self.declared_input_states.contains_key(&dependency_type_id) {
                    format!("input:{}", short_type_name(dependency.state_type_name()))
                } else {
                    return Err(analysis_node_graph_error(format!(
                        "Node '{}' depends on missing state {}",
                        node.name(),
                        dependency.state_type_name(),
                    )));
                };
                ensure_external_render_node(
                    &mut external_labels,
                    &mut external_node_ids,
                    &mut next_node_id,
                    dependency_type_id,
                    label,
                );
            }
        }

        let mut dag = AsciiGraph::new().with_render_mode(RenderMode::Vertical);

        for (index, label) in node_labels.iter().enumerate() {
            dag.add_node(index, label);
        }

        for (_, node_id, label) in &external_labels {
            dag.add_node(*node_id, label);
        }

        for (index, node) in self.nodes.iter().enumerate() {
            for dependency in node.dependencies() {
                let dependency_type_id = dependency.state_type_id();
                let source_id = if let Some(provider_index) = providers.get(&dependency_type_id) {
                    *provider_index
                } else if self.declared_root_states.contains_key(&dependency_type_id) {
                    *external_node_ids
                        .get(&dependency_type_id)
                        .expect("root node should have been prepared")
                } else if self.declared_input_states.contains_key(&dependency_type_id) {
                    *external_node_ids
                        .get(&dependency_type_id)
                        .expect("input node should have been prepared")
                } else {
                    return Err(analysis_node_graph_error(format!(
                        "Node '{}' depends on missing state {}",
                        node.name(),
                        dependency.state_type_name(),
                    )));
                };
                dag.add_edge(source_id, index, None);
            }
        }

        Ok(format!("AnalysisGraph\n{}", dag.render()))
    }
}

fn ensure_external_render_node(
    labels: &mut Vec<(TypeId, usize, Box<str>)>,
    external_node_ids: &mut HashMap<TypeId, usize>,
    next_node_id: &mut usize,
    dependency_type_id: TypeId,
    label: String,
) -> usize {
    if let Some(node_id) = external_node_ids.get(&dependency_type_id) {
        return *node_id;
    }

    let node_id = *next_node_id;
    *next_node_id += 1;
    labels.push((dependency_type_id, node_id, label.into_boxed_str()));
    external_node_ids.insert(dependency_type_id, node_id);
    node_id
}

fn short_type_name(type_name: &str) -> String {
    let mut shortened = String::with_capacity(type_name.len());
    let mut token = String::new();

    for character in type_name.chars() {
        if character.is_alphanumeric() || matches!(character, '_' | ':') {
            token.push(character);
            continue;
        }

        if !token.is_empty() {
            shortened.push_str(token.rsplit("::").next().unwrap_or(&token));
            token.clear();
        }
        shortened.push(character);
    }

    if !token.is_empty() {
        shortened.push_str(token.rsplit("::").next().unwrap_or(&token));
    }

    shortened
}
