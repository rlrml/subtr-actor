use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;

use crate::SubtrActorResult;

use super::{analysis_node_graph_error, AnalysisNodeDyn};

pub struct AnalysisStateContext<'a> {
    states: HashMap<TypeId, &'a dyn Any>,
}

pub struct AnalysisStateRef<'a> {
    type_id: TypeId,
    type_name: &'static str,
    state: &'a dyn Any,
}

impl<'a> AnalysisStateRef<'a> {
    pub fn of<T: 'static>(state: &'a T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            state,
        }
    }

    pub(super) fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub(super) fn type_name(&self) -> &'static str {
        self.type_name
    }

    fn state(&self) -> &'a dyn Any {
        self.state
    }
}

impl<'a> AnalysisStateContext<'a> {
    pub(super) fn from_parts(
        root_states: &'a HashMap<TypeId, Box<dyn Any>>,
        input_states: &'a [AnalysisStateRef<'a>],
        before: &'a [Box<dyn AnalysisNodeDyn>],
    ) -> Self {
        let mut states =
            HashMap::with_capacity(root_states.len() + input_states.len() + before.len());
        for (type_id, state) in root_states {
            states.insert(*type_id, state.as_ref());
        }
        for input_state in input_states {
            states.insert(input_state.type_id(), input_state.state());
        }
        for node in before {
            states.insert(node.provides_state_type_id(), node.state_any());
        }
        Self { states }
    }

    pub fn get<T: 'static>(&self) -> SubtrActorResult<&'a T> {
        self.maybe_get::<T>().ok_or_else(|| {
            analysis_node_graph_error(format!(
                "Missing state {} in analysis context",
                type_name::<T>()
            ))
        })
    }

    pub fn maybe_get<T: 'static>(&self) -> Option<&'a T> {
        self.states
            .get(&TypeId::of::<T>())
            .and_then(|state| state.downcast_ref::<T>())
    }
}
