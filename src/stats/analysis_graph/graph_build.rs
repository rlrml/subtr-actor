use std::any::{type_name, TypeId};

use super::{AnalysisGraph, AnalysisNode, AnalysisNodeDyn};

impl AnalysisGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_root_state_type<T: 'static>(mut self) -> Self {
        self.register_root_state::<T>();
        self
    }

    pub fn register_root_state<T: 'static>(&mut self) {
        self.declared_root_states
            .insert(TypeId::of::<T>(), type_name::<T>());
    }

    pub fn with_input_state_type<T: 'static>(mut self) -> Self {
        self.register_input_state::<T>();
        self
    }

    pub fn register_input_state<T: 'static>(&mut self) {
        self.declared_input_states
            .insert(TypeId::of::<T>(), type_name::<T>());
    }

    pub fn set_root_state<T: 'static>(&mut self, value: T) {
        self.register_root_state::<T>();
        self.root_states.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn with_node<N>(mut self, node: N) -> Self
    where
        N: AnalysisNode,
    {
        self.push_node(node);
        self
    }

    pub fn with_boxed_node(mut self, node: Box<dyn AnalysisNodeDyn>) -> Self {
        self.push_boxed_node(node);
        self
    }

    pub fn push_node<N>(&mut self, node: N)
    where
        N: AnalysisNode,
    {
        self.push_boxed_node(Box::new(node));
    }

    pub fn push_boxed_node(&mut self, node: Box<dyn AnalysisNodeDyn>) {
        self.nodes.push(node);
        self.resolved = false;
    }

    pub fn state<T: 'static>(&self) -> Option<&T> {
        let target = TypeId::of::<T>();
        self.root_states
            .get(&target)
            .and_then(|state| state.downcast_ref::<T>())
            .or_else(|| {
                self.nodes
                    .iter()
                    .find(|node| node.provides_state_type_id() == target)
                    .and_then(|node| node.state_any().downcast_ref::<T>())
            })
    }

    pub fn node_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.nodes.iter().map(|node| node.name())
    }
}
