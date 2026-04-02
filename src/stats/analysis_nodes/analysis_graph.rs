#![allow(dead_code)]

use std::any::{type_name, Any, TypeId};
use std::collections::{HashMap, HashSet};

use ascii_dag::graph::{Graph as AsciiGraph, RenderMode};

use crate::*;

#[derive(Clone, Copy)]
pub struct AnalysisDependency {
    state_type_id: TypeId,
    state_type_name: &'static str,
    source: AnalysisDependencySource,
}

#[derive(Clone, Copy)]
enum AnalysisDependencySource {
    DefaultFactory(fn() -> Box<dyn AnalysisNodeDyn>),
    External,
}

impl AnalysisDependency {
    pub fn required<T: 'static>() -> Self {
        Self {
            state_type_id: TypeId::of::<T>(),
            state_type_name: type_name::<T>(),
            source: AnalysisDependencySource::External,
        }
    }

    pub fn with_default<T: 'static>(default_factory: fn() -> Box<dyn AnalysisNodeDyn>) -> Self {
        Self {
            state_type_id: TypeId::of::<T>(),
            state_type_name: type_name::<T>(),
            source: AnalysisDependencySource::DefaultFactory(default_factory),
        }
    }

    pub fn state_type_id(&self) -> TypeId {
        self.state_type_id
    }

    pub fn state_type_name(&self) -> &'static str {
        self.state_type_name
    }

    fn default_factory(&self) -> fn() -> Box<dyn AnalysisNodeDyn> {
        match self.source {
            AnalysisDependencySource::DefaultFactory(default_factory) => default_factory,
            AnalysisDependencySource::External => panic!(
                "analysis dependency for {} has no default factory",
                self.state_type_name
            ),
        }
    }

    fn is_external(&self) -> bool {
        matches!(self.source, AnalysisDependencySource::External)
    }
}

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

    fn type_id(&self) -> TypeId {
        self.type_id
    }

    fn type_name(&self) -> &'static str {
        self.type_name
    }

    fn state(&self) -> &'a dyn Any {
        self.state
    }
}

impl<'a> AnalysisStateContext<'a> {
    fn from_parts(
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

pub trait AnalysisNode: 'static {
    type State: 'static;

    fn name(&self) -> &'static str;

    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        Ok(())
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        Vec::new()
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()>;

    fn finish(&mut self) -> SubtrActorResult<()> {
        Ok(())
    }

    fn state(&self) -> &Self::State;
}

pub trait AnalysisNodeDyn: 'static {
    fn name(&self) -> &'static str;

    fn provides_state_type_id(&self) -> TypeId;

    fn provides_state_type_name(&self) -> &'static str;

    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()>;

    fn dependencies(&self) -> Vec<AnalysisDependency>;

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()>;

    fn finish(&mut self) -> SubtrActorResult<()>;

    fn state_any(&self) -> &dyn Any;
}

impl<N> AnalysisNodeDyn for N
where
    N: AnalysisNode,
{
    fn name(&self) -> &'static str {
        AnalysisNode::name(self)
    }

    fn provides_state_type_id(&self) -> TypeId {
        TypeId::of::<N::State>()
    }

    fn provides_state_type_name(&self) -> &'static str {
        type_name::<N::State>()
    }

    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        AnalysisNode::on_replay_meta(self, meta)
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        AnalysisNode::dependencies(self)
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        AnalysisNode::evaluate(self, ctx)
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        AnalysisNode::finish(self)
    }

    fn state_any(&self) -> &dyn Any {
        self.state()
    }
}

#[derive(Default)]
pub struct AnalysisGraph {
    nodes: Vec<Box<dyn AnalysisNodeDyn>>,
    evaluation_order: Vec<usize>,
    declared_root_states: HashMap<TypeId, &'static str>,
    declared_input_states: HashMap<TypeId, &'static str>,
    root_states: HashMap<TypeId, Box<dyn Any>>,
    resolved: bool,
}

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

    pub fn resolve(&mut self) -> SubtrActorResult<()> {
        if self.resolved {
            return Ok(());
        }

        loop {
            let providers = self.provider_index_by_type()?;
            let mut additions = Vec::new();
            let mut queued_types = HashSet::new();

            for node in &self.nodes {
                for dependency in node.dependencies() {
                    if providers.contains_key(&dependency.state_type_id())
                        || self
                            .declared_root_states
                            .contains_key(&dependency.state_type_id())
                        || self
                            .declared_input_states
                            .contains_key(&dependency.state_type_id())
                    {
                        continue;
                    }
                    if dependency.is_external() {
                        return Err(analysis_node_graph_error(format!(
                            "Node '{}' requires state {} with no provider",
                            node.name(),
                            dependency.state_type_name(),
                        )));
                    }
                    let default_factory = dependency.default_factory();
                    if queued_types.insert(dependency.state_type_id()) {
                        additions.push(default_factory());
                    }
                }
            }

            if additions.is_empty() {
                break;
            }

            self.nodes.extend(additions);
        }

        let providers = self.provider_index_by_type()?;
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut order = Vec::with_capacity(self.nodes.len());

        for index in 0..self.nodes.len() {
            self.visit_node(
                index,
                &providers,
                &mut visiting,
                &mut visited,
                &mut order,
                &mut Vec::new(),
            )?;
        }

        let mut ordered_nodes = Vec::with_capacity(self.nodes.len());
        let mut original_nodes: Vec<Option<Box<dyn AnalysisNodeDyn>>> =
            std::mem::take(&mut self.nodes)
                .into_iter()
                .map(Some)
                .collect();
        for index in order {
            ordered_nodes.push(
                original_nodes[index]
                    .take()
                    .expect("topological order should only reference each node once"),
            );
        }

        self.nodes = ordered_nodes;
        self.evaluation_order = (0..self.nodes.len()).collect();
        self.resolved = true;
        Ok(())
    }

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

        for (type_id, type_name) in &self.declared_root_states {
            if !self.root_states.contains_key(type_id) {
                return Err(analysis_node_graph_error(format!(
                    "Missing root state {type_name} for evaluation"
                )));
            }
        }

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
        for node in &mut self.nodes {
            node.finish()?;
        }
        Ok(())
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

    pub fn render_ascii_dag(&mut self) -> SubtrActorResult<String> {
        self.resolve()?;

        if self.nodes.is_empty() {
            return Ok("AnalysisGraph\n\\- (empty)".to_owned());
        }

        let providers = self.provider_index_by_type()?;
        let node_labels: Vec<Box<str>> = self
            .nodes
            .iter()
            .map(|node| node.name().to_owned().into_boxed_str())
            .collect();
        let mut external_labels = Vec::new();
        let mut external_node_ids = HashMap::new();
        let mut next_node_id = self.nodes.len();

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
                    &mut next_node_id,
                    dependency_type_id,
                    label,
                );
            }
        }

        let mut dag = AsciiGraph::new().with_render_mode(RenderMode::Vertical);

        for (index, label) in node_labels.iter().enumerate() {
            dag.add_node(index, label);
        }

        for (_, node_id, label) in &external_labels {
            dag.add_node(*node_id, label);
        }

        for (index, node) in self.nodes.iter().enumerate() {
            for dependency in node.dependencies() {
                let dependency_type_id = dependency.state_type_id();
                let source_id = if let Some(provider_index) = providers.get(&dependency_type_id) {
                    *provider_index
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
                dag.add_edge(source_id, index, None);
            }
        }

        Ok(format!("AnalysisGraph\n{}", dag.render()))
    }

    fn provider_index_by_type(&self) -> SubtrActorResult<HashMap<TypeId, usize>> {
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

    fn required_input_states(&self) -> HashMap<TypeId, &'static str> {
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

    fn visit_node(
        &self,
        index: usize,
        providers: &HashMap<TypeId, usize>,
        visiting: &mut HashSet<usize>,
        visited: &mut HashSet<usize>,
        order: &mut Vec<usize>,
        stack: &mut Vec<&'static str>,
    ) -> SubtrActorResult<()> {
        if visited.contains(&index) {
            return Ok(());
        }
        if !visiting.insert(index) {
            stack.push(self.nodes[index].name());
            let cycle = stack.join(" -> ");
            stack.pop();
            return Err(analysis_node_graph_error(format!(
                "Cycle detected in analysis node graph: {cycle}"
            )));
        }

        stack.push(self.nodes[index].name());
        for dependency in self.nodes[index].dependencies() {
            if self
                .declared_root_states
                .contains_key(&dependency.state_type_id())
                || self
                    .declared_input_states
                    .contains_key(&dependency.state_type_id())
            {
                continue;
            }

            let Some(dependency_index) = providers.get(&dependency.state_type_id()).copied() else {
                stack.pop();
                return Err(analysis_node_graph_error(format!(
                    "Node '{}' depends on missing state {}",
                    self.nodes[index].name(),
                    dependency.state_type_name(),
                )));
            };
            self.visit_node(dependency_index, providers, visiting, visited, order, stack)?;
        }
        stack.pop();

        visiting.remove(&index);
        visited.insert(index);
        order.push(index);
        Ok(())
    }
}

fn analysis_node_graph_error(message: String) -> SubtrActorError {
    SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
        "analysis node graph error: {message}"
    )))
}

fn ensure_external_render_node(
    labels: &mut Vec<(TypeId, usize, Box<str>)>,
    external_node_ids: &mut HashMap<TypeId, usize>,
    next_node_id: &mut usize,
    dependency_type_id: TypeId,
    label: String,
) -> usize {
    if let Some(node_id) = external_node_ids.get(&dependency_type_id) {
        return *node_id;
    }

    let node_id = *next_node_id;
    *next_node_id += 1;
    labels.push((dependency_type_id, node_id, label.into_boxed_str()));
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

#[cfg(test)]
#[path = "analysis_graph_tests.rs"]
mod tests;
