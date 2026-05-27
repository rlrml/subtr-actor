use std::any::TypeId;
use std::collections::HashMap;

use crate::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

use super::AnalysisGraph;

impl AnalysisGraph {
    pub(super) fn provider_index_by_type(&self) -> SubtrActorResult<HashMap<TypeId, usize>> {
        let mut providers = HashMap::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if self
                .declared_root_states
                .contains_key(&node.provides_state_type_id())
            {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::CallbackError(format!(
                        "analysis node graph error: Duplicate providers for root state {}: root and '{}'",
                        node.provides_state_type_name(),
                        node.name(),
                    )),
                );
            }
            if self
                .declared_input_states
                .contains_key(&node.provides_state_type_id())
            {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::CallbackError(format!(
                        "analysis node graph error: Duplicate providers for input state {}: input and '{}'",
                        node.provides_state_type_name(),
                        node.name(),
                    )),
                );
            }
            if let Some(existing) = providers.insert(node.provides_state_type_id(), index) {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::CallbackError(format!(
                        "analysis node graph error: Duplicate providers for state {}: '{}' and '{}'",
                        node.provides_state_type_name(),
                        self.nodes[existing].name(),
                        node.name(),
                    )),
                );
            }
        }
        Ok(providers)
    }

    pub(super) fn required_input_states(&self) -> HashMap<TypeId, &'static str> {
        let mut required = HashMap::new();
        for node in &self.nodes {
            for dependency in node.dependencies() {
                let type_id = dependency.state_type_id();
                if self.declared_input_states.contains_key(&type_id)
                    && !self.root_states.contains_key(&type_id)
                {
                    required.insert(type_id, dependency.state_type_name());
                }
            }
        }
        required
    }
}
