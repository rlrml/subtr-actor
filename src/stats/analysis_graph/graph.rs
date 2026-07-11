use std::any::{Any, TypeId, type_name};
use std::collections::{HashMap, HashSet};

use crate::stats::calculators::EmittedEvent;
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

/// A node in the [`AnalysisGraph`]: consumes upstream state, runs once per
/// frame, and exposes its own typed state for downstream nodes.
///
/// Implementors are the catalog of analysis nodes (see the
/// [`nodes`](crate::stats::analysis_graph) module and the *Implementors* list
/// below). A node declares what it reads via
/// [`dependencies`](AnalysisNode::dependencies), reads it from the
/// [`AnalysisStateContext`] in [`evaluate`](AnalysisNode::evaluate), and
/// publishes [`State`](AnalysisNode::State) via [`state`](AnalysisNode::state).
/// The blanket [`AnalysisNodeDyn`] impl makes every `AnalysisNode` usable as a
/// boxed graph node.
pub trait AnalysisNode: 'static {
    /// The typed state this node publishes to downstream nodes.
    type State: 'static;

    /// Stable identifier for this node, used for dependency wiring and the
    /// built-in node registry.
    fn name(&self) -> &'static str;

    /// Static catalog of the events this node emits, if any.
    ///
    /// The node is the source of truth for what it produces: a graph's emitted
    /// events come from walking its actual nodes (see
    /// [`AnalysisGraph::emitted_events`]), so there is no name-keyed side
    /// registry that can drift out of sync with the nodes themselves.
    fn emitted_events(&self) -> &'static [EmittedEvent] {
        &[]
    }

    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        Ok(())
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        Vec::new()
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()>;

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        Ok(())
    }

    /// The node's *current* full timeline-event set: a pure projection of the
    /// state accumulated so far, keyed by cadence-invariant `meta.id`s and
    /// lifecycle-annotated per event (see [`crate::EventLifecycle`]).
    ///
    /// Nodes never diff against prior emissions — they simply re-project on
    /// every call, and the graph's central store
    /// ([`AnalysisGraph::project_events_now`]) turns successive projections
    /// into upsert/retract transactions and enforces the lifecycle
    /// invariants. Each event stream must be projected by exactly one node
    /// (ids embed the stream name, so distinct streams can never collide
    /// across nodes; the store rejects duplicate ids).
    ///
    /// The default projects nothing, which is right for every node that only
    /// publishes derived state.
    fn project_events(&self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>> {
        Ok(Vec::new())
    }

    fn state(&self) -> &Self::State;
}

pub trait AnalysisNodeDyn: 'static {
    fn name(&self) -> &'static str;

    fn emitted_events(&self) -> &'static [EmittedEvent];

    fn provides_state_type_id(&self) -> TypeId;

    fn provides_state_type_name(&self) -> &'static str;

    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()>;

    fn dependencies(&self) -> Vec<AnalysisDependency>;

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()>;

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()>;

    fn project_events(&self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>>;

    fn state_any(&self) -> &dyn Any;
}

impl<N> AnalysisNodeDyn for N
where
    N: AnalysisNode,
{
    fn name(&self) -> &'static str {
        AnalysisNode::name(self)
    }

    fn emitted_events(&self) -> &'static [EmittedEvent] {
        AnalysisNode::emitted_events(self)
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

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        AnalysisNode::finish(self, ctx)
    }

    fn project_events(&self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>> {
        AnalysisNode::project_events(self, ctx)
    }

    fn state_any(&self) -> &dyn Any {
        self.state()
    }
}

/// A resolved, ordered collection of [`AnalysisNode`]s evaluated together over a
/// replay.
///
/// Add nodes with [`with_node`](AnalysisGraph::with_node) /
/// [`push_node`](AnalysisGraph::push_node) (or by name via the module-level
/// `graph_with_*` helpers); the graph topologically orders them by their
/// dependencies. Drive it frame by frame, then read any node's published state
/// with [`state`](AnalysisGraph::state).
#[derive(Default)]
pub struct AnalysisGraph {
    nodes: Vec<Box<dyn AnalysisNodeDyn>>,
    evaluation_order: Vec<usize>,
    declared_root_states: HashMap<TypeId, &'static str>,
    declared_input_states: HashMap<TypeId, &'static str>,
    root_states: HashMap<TypeId, Box<dyn Any>>,
    resolved: bool,
    /// Central differential store over the nodes' event projections: each
    /// [`project_events_now`](AnalysisGraph::project_events_now) (and the
    /// final projection inside [`finish`](AnalysisGraph::finish)) diffs the
    /// aggregated node projections against this log's previous view,
    /// appending [`EventTransaction`]s and enforcing the lifecycle
    /// invariants.
    event_log: TimelineTransactionLog,
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

    pub fn ensure_dependency(&mut self, dependency: AnalysisDependency) -> SubtrActorResult<()> {
        let providers = self.provider_index_by_type()?;
        if providers.contains_key(&dependency.state_type_id())
            || self
                .declared_root_states
                .contains_key(&dependency.state_type_id())
            || self
                .declared_input_states
                .contains_key(&dependency.state_type_id())
        {
            return Ok(());
        }
        if dependency.is_external() {
            return Err(analysis_node_graph_error(format!(
                "Required state {} has no provider",
                dependency.state_type_name(),
            )));
        }

        self.push_boxed_node((dependency.default_factory())());
        Ok(())
    }

    pub fn ensure_dependencies<I>(&mut self, dependencies: I) -> SubtrActorResult<()>
    where
        I: IntoIterator<Item = AnalysisDependency>,
    {
        for dependency in dependencies {
            self.ensure_dependency(dependency)?;
        }
        Ok(())
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
        self.resolve()?;
        for node_index in self.evaluation_order.clone() {
            let (before, current_and_after) = self.nodes.split_at_mut(node_index);
            let (current, _) = current_and_after
                .split_first_mut()
                .expect("evaluation order should contain valid indexes");
            let ctx = AnalysisStateContext::from_parts(&self.root_states, &[], before);
            current.finish(&ctx)?;
        }
        // One final projection in finalize-everything mode: after the node
        // finishes above, no future evidence exists by definition, so every
        // projected event is upgraded to `Finalized` before the store diffs it
        // against the last interim projection — which is exactly where the
        // lifecycle invariants (finalized content never changed, nothing
        // vanished) are asserted for the whole run. Finish-only consumers get
        // a single projection here and never see an interim lifecycle.
        let mut projection = self.collect_projected_events()?;
        for event in &mut projection {
            event.meta.lifecycle = EventLifecycle::Finalized;
        }
        self.event_log.apply_projection(&projection)
    }

    /// Projects every node's current event set into the graph's central
    /// transaction log (see [`AnalysisNode::project_events`]).
    ///
    /// Cadence is owned by the caller — typically a live driver invoking this
    /// on a game-time interval (each projection re-scans all committed
    /// calculator events, so a ~1s cadence keeps the amortized cost
    /// negligible; projecting every frame would be quadratic over a match).
    /// Event ids are cadence-invariant, so the cadence only decides *when* an
    /// event becomes observable, never which id it gets. Batch consumers can
    /// skip this entirely: [`finish`](AnalysisGraph::finish) always performs
    /// one final, finalize-everything projection.
    pub fn project_events_now(&mut self) -> SubtrActorResult<()> {
        self.resolve()?;
        let projection = self.collect_projected_events()?;
        self.event_log.apply_projection(&projection)
    }

    /// The central event transaction log fed by
    /// [`project_events_now`](AnalysisGraph::project_events_now) and
    /// [`finish`](AnalysisGraph::finish). Incremental consumers keep their own
    /// cursor over it (e.g. `TimelineTransactionLog::transactions_since`);
    /// reading never mutates the log.
    pub fn event_transaction_log(&self) -> &TimelineTransactionLog {
        &self.event_log
    }

    /// Aggregates every node's current projection, in evaluation order.
    ///
    /// The order is deterministic once the graph is resolved (topological
    /// order over the same node set), and each node's own projection order is
    /// deterministic by the id-disambiguator contract, so the aggregate is a
    /// pure function of calculator state. Ids embed their stream name and
    /// each stream is owned by exactly one node, so projections from
    /// different nodes cannot collide (the store's duplicate-id check turns
    /// any violation of that ownership assumption into a loud error).
    fn collect_projected_events(&self) -> SubtrActorResult<Vec<Event>> {
        let mut projection = Vec::new();
        for &node_index in &self.evaluation_order {
            let (before, current_and_after) = self.nodes.split_at(node_index);
            let current = current_and_after
                .first()
                .expect("evaluation order should contain valid indexes");
            let ctx = AnalysisStateContext::from_parts(&self.root_states, &[], before);
            let node_projection = current.project_events(&ctx)?;
            verify_projected_streams_are_declared(current.as_ref(), &node_projection)?;
            projection.extend(node_projection);
        }
        Ok(projection)
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

    pub fn emitted_events(&mut self) -> SubtrActorResult<Vec<EmittedEvent>> {
        self.resolve()?;
        Ok(self
            .nodes
            .iter()
            .flat_map(|node| node.emitted_events().iter().copied())
            .collect())
    }

    fn provider_index_by_type(&self) -> SubtrActorResult<HashMap<TypeId, usize>> {
        let mut providers = HashMap::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if self
                .declared_root_states
                .contains_key(&node.provides_state_type_id())
            {
                return SubtrActorError::new_result(SubtrActorErrorVariant::CallbackError(
                    format!(
                        "analysis node graph error: Duplicate providers for root state {}: root and '{}'",
                        node.provides_state_type_name(),
                        node.name(),
                    ),
                ));
            }
            if self
                .declared_input_states
                .contains_key(&node.provides_state_type_id())
            {
                return SubtrActorError::new_result(SubtrActorErrorVariant::CallbackError(
                    format!(
                        "analysis node graph error: Duplicate providers for input state {}: input and '{}'",
                        node.provides_state_type_name(),
                        node.name(),
                    ),
                ));
            }
            if let Some(existing) = providers.insert(node.provides_state_type_id(), index) {
                return SubtrActorError::new_result(SubtrActorErrorVariant::CallbackError(
                    format!(
                        "analysis node graph error: Duplicate providers for state {}: '{}' and '{}'",
                        node.provides_state_type_name(),
                        self.nodes[existing].name(),
                        node.name(),
                    ),
                ));
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
            let mut dependency_refs = Vec::new();
            for dependency in node.dependencies() {
                let dependency_type_id = dependency.state_type_id();
                let source_id = if let Some(provider_index) = providers.get(&dependency_type_id) {
                    external_count + *provider_index
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
                dependency_refs.push(format!("[{source_id}]"));
            }

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

fn ensure_external_render_node(
    labels: &mut Vec<(TypeId, Box<str>)>,
    external_node_ids: &mut HashMap<TypeId, usize>,
    dependency_type_id: TypeId,
    label: String,
) -> usize {
    if let Some(node_id) = external_node_ids.get(&dependency_type_id) {
        return *node_id;
    }

    let node_id = labels.len();
    labels.push((dependency_type_id, label.into_boxed_str()));
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

fn analysis_node_graph_error(message: String) -> SubtrActorError {
    SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
        "analysis node graph error: {message}"
    )))
}

/// Rejects a node projection containing an event on a stream the node does
/// not declare via [`AnalysisNode::emitted_events`].
///
/// Stream ownership is a static, per-node declaration
/// ([`EmittedEvent::projected`]); this check keeps every projection site
/// honest about it, so the declared catalog (and the finalization horizons it
/// carries) can be trusted to describe what actually reaches the timeline.
fn verify_projected_streams_are_declared(
    node: &dyn AnalysisNodeDyn,
    projection: &[Event],
) -> SubtrActorResult<()> {
    for event in projection {
        let declared = node.emitted_events().iter().any(|emitted| {
            emitted
                .projected
                .is_some_and(|projected| projected.stream == event.meta.stream)
        });
        if !declared {
            return SubtrActorError::new_result(
                SubtrActorErrorVariant::TimelineEventInvariantViolation(format!(
                    "node {:?} projected event {:?} on undeclared stream {:?}",
                    node.name(),
                    event.meta.id,
                    event.meta.stream,
                )),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
