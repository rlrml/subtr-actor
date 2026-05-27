#![allow(dead_code)]

pub mod graph;
pub use graph::{
    AnalysisDependency, AnalysisGraph, AnalysisNode, AnalysisNodeDyn, AnalysisStateContext,
    AnalysisStateRef,
};

#[macro_use]
mod node_macros;

mod builtin_aliases;
mod builtin_all_nodes;
mod builtin_graph;
mod builtin_names;
mod builtin_nodes;
mod collector;
mod nodes;

pub use builtin_aliases::{builtin_analysis_node_aliases, BuiltinAnalysisNodeAlias};
pub use builtin_all_nodes::all_analysis_nodes;
pub use builtin_graph::{
    collect_analysis_graph_for_replay, collect_builtin_analysis_graph_for_replay,
    graph_with_all_analysis_nodes, graph_with_builtin_analysis_nodes,
};
pub use builtin_names::builtin_analysis_node_names;
#[allow(unused_imports)]
pub use collector::AnalysisNodeCollector;
#[allow(unused_imports)]
pub use nodes::*;

#[cfg(test)]
#[path = "module_tests.rs"]
mod tests;
