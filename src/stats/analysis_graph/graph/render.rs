use std::any::TypeId;
use std::collections::HashMap;

use super::{analysis_node_graph_error, AnalysisGraph};
use crate::*;

impl AnalysisGraph {
    pub fn render_ascii_dag(&mut self) -> SubtrActorResult<String> {
        self.resolve()?;

        let providers = self.provider_index_by_type()?;
        let mut external_labels = Vec::new();
        let mut external_node_ids = HashMap::new();

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
                    dependency_type_id,
                    label,
                );
            }
        }

        if self.nodes.is_empty() && external_labels.is_empty() {
            return Ok("AnalysisGraph\n\\- (empty)".to_owned());
        }

        let external_count = external_labels.len();
        let mut lines = Vec::with_capacity(1 + external_count + self.nodes.len());
        lines.push("AnalysisGraph".to_owned());

        for (display_id, (_, label)) in external_labels.iter().enumerate() {
            lines.push(format!("[{display_id}] {label}"));
        }

        for (index, node) in self.nodes.iter().enumerate() {
            let display_id = external_count + index;
            let mut dependency_refs = Vec::new();
            for dependency in node.dependencies() {
                let dependency_type_id = dependency.state_type_id();
                let source_id = if let Some(provider_index) = providers.get(&dependency_type_id) {
                    external_count + *provider_index
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
                dependency_refs.push(format!("[{source_id}]"));
            }

            if dependency_refs.is_empty() {
                lines.push(format!("[{display_id}] {}", node.name()));
            } else {
                lines.push(format!(
                    "[{display_id}] {} <- {}",
                    node.name(),
                    dependency_refs.join(", "),
                ));
            }
        }

        Ok(lines.join("\n"))
    }
}

fn ensure_external_render_node(
    labels: &mut Vec<(TypeId, Box<str>)>,
    external_node_ids: &mut HashMap<TypeId, usize>,
    dependency_type_id: TypeId,
    label: String,
) -> usize {
    if let Some(node_id) = external_node_ids.get(&dependency_type_id) {
        return *node_id;
    }

    let node_id = labels.len();
    labels.push((dependency_type_id, label.into_boxed_str()));
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
