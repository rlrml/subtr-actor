use super::*;

pub(crate) fn graph_state<'a, T: 'static>(
    graph: &'a AnalysisGraph,
    module_name: &str,
) -> SubtrActorResult<&'a T> {
    graph.state::<T>().ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "missing analysis-node state for builtin stats module '{module_name}'"
        )))
    })
}
