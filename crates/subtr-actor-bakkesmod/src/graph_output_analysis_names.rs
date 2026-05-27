use super::*;

pub(crate) fn callable_analysis_node_names_for_graph(graph: &AnalysisGraph) -> Vec<String> {
    let mut names = BTreeSet::new();
    names.extend(graph.node_names().map(str::to_owned));
    names.extend(
        builtin_analysis_node_names()
            .iter()
            .map(|name| (*name).to_owned()),
    );
    names.extend(
        builtin_analysis_node_aliases()
            .iter()
            .map(|alias| alias.alias.to_owned()),
    );
    names.into_iter().collect()
}

pub(crate) fn callable_analysis_node_names(engine: &SaEngine) -> Vec<String> {
    callable_analysis_node_names_for_graph(&engine.graph)
}

pub(crate) fn serialize_analysis_node_names(engine: *const SaEngine) -> Vec<u8> {
    let Some(engine) = (unsafe { engine.as_ref() }) else {
        return Vec::new();
    };
    serde_json::to_vec(&callable_analysis_node_names(engine)).unwrap_or_default()
}
