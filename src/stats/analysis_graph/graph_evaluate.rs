use std::collections::HashMap;

use crate::{ReplayMeta, SubtrActorResult};

use super::{analysis_node_graph_error, AnalysisGraph, AnalysisStateContext, AnalysisStateRef};

impl AnalysisGraph {
    pub fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.resolve()?;
        for node in &mut self.nodes {
            node.on_replay_meta(meta)?;
        }
        Ok(())
    }

    pub fn evaluate(&mut self) -> SubtrActorResult<()> {
        self.evaluate_with_states(&[])
    }

    pub fn evaluate_with_state<T: 'static>(&mut self, value: &T) -> SubtrActorResult<()> {
        self.evaluate_with_states(&[AnalysisStateRef::of(value)])
    }

    pub fn evaluate_with_states<'a>(
        &mut self,
        input_states: &'a [AnalysisStateRef<'a>],
    ) -> SubtrActorResult<()> {
        self.resolve()?;
        self.validate_root_states()?;
        self.validate_input_states(input_states)?;

        for node_index in self.evaluation_order.clone() {
            let (before, current_and_after) = self.nodes.split_at_mut(node_index);
            let (current, _) = current_and_after
                .split_first_mut()
                .expect("evaluation order should contain valid indexes");
            let ctx = AnalysisStateContext::from_parts(&self.root_states, input_states, before);
            current.evaluate(&ctx)?;
        }

        Ok(())
    }

    pub fn finish(&mut self) -> SubtrActorResult<()> {
        self.resolve()?;
        for node_index in self.evaluation_order.clone() {
            let (before, current_and_after) = self.nodes.split_at_mut(node_index);
            let (current, _) = current_and_after
                .split_first_mut()
                .expect("evaluation order should contain valid indexes");
            let ctx = AnalysisStateContext::from_parts(&self.root_states, &[], before);
            current.finish(&ctx)?;
        }
        Ok(())
    }

    fn validate_root_states(&self) -> SubtrActorResult<()> {
        for (type_id, type_name) in &self.declared_root_states {
            if !self.root_states.contains_key(type_id) {
                return Err(analysis_node_graph_error(format!(
                    "Missing root state {type_name} for evaluation"
                )));
            }
        }
        Ok(())
    }

    fn validate_input_states<'a>(
        &self,
        input_states: &'a [AnalysisStateRef<'a>],
    ) -> SubtrActorResult<()> {
        let mut provided_input_types = HashMap::with_capacity(input_states.len());
        for input_state in input_states {
            if let Some(existing) =
                provided_input_types.insert(input_state.type_id(), input_state.type_name())
            {
                return Err(analysis_node_graph_error(format!(
                    "Duplicate input states for {}: {} and {}",
                    input_state.type_name(),
                    existing,
                    input_state.type_name(),
                )));
            }
        }
        for (type_id, type_name) in self.required_input_states() {
            if !provided_input_types.contains_key(&type_id) {
                return Err(analysis_node_graph_error(format!(
                    "Missing input state {type_name} for evaluation"
                )));
            }
        }
        Ok(())
    }
}
