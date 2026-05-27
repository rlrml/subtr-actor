use std::collections::HashMap;

use crate::SubtrActorResult;

use super::{
    analysis_node_graph_error, render_helpers::ensure_external_render_node,
    render_helpers::short_type_name, AnalysisGraph,
};

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
            let dependency_refs = node
                .dependencies()
                .into_iter()
                .map(|dependency| {
                    render_dependency_ref(
                        &providers,
                        &external_node_ids,
                        external_count,
                        dependency.state_type_id(),
                    )
                })
                .collect::<Vec<_>>();

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

fn render_dependency_ref(
    providers: &HashMap<std::any::TypeId, usize>,
    external_node_ids: &HashMap<std::any::TypeId, usize>,
    external_count: usize,
    dependency_type_id: std::any::TypeId,
) -> String {
    let source_id = providers
        .get(&dependency_type_id)
        .map(|provider_index| external_count + *provider_index)
        .or_else(|| external_node_ids.get(&dependency_type_id).copied())
        .expect("render dependency should have a provider or external node");
    format!("[{source_id}]")
}
