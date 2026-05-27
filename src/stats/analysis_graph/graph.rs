#![allow(dead_code)]

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::{SubtrActorError, SubtrActorErrorVariant};

#[path = "graph_build.rs"]
mod build;
#[path = "graph_context.rs"]
mod context;
#[path = "graph_dependency.rs"]
mod dependency;
#[path = "graph_evaluate.rs"]
mod evaluate;
#[path = "graph_node.rs"]
mod node;
#[path = "graph_provider.rs"]
mod provider;
#[path = "graph_render.rs"]
mod render;
#[path = "graph_render_helpers.rs"]
mod render_helpers;
#[path = "graph_resolve.rs"]
mod resolve;
#[path = "graph_visit.rs"]
mod visit;

pub use context::{AnalysisStateContext, AnalysisStateRef};
pub use dependency::AnalysisDependency;
pub use node::{AnalysisNode, AnalysisNodeDyn};

#[derive(Default)]
pub struct AnalysisGraph {
    nodes: Vec<Box<dyn AnalysisNodeDyn>>,
    evaluation_order: Vec<usize>,
    declared_root_states: HashMap<TypeId, &'static str>,
    declared_input_states: HashMap<TypeId, &'static str>,
    root_states: HashMap<TypeId, Box<dyn Any>>,
    resolved: bool,
}

pub(super) fn analysis_node_graph_error(message: String) -> SubtrActorError {
    SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
        "analysis node graph error: {message}"
    )))
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
