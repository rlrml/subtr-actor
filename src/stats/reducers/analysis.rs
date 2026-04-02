use std::any::Any;
use std::collections::{BTreeSet, HashMap, HashSet};

pub use crate::stats::calculators::{
    BackboardBounceCalculator, BackboardBounceEvent, BackboardBounceState, FiftyFiftyState,
    FiftyFiftyStateCalculator, PossessionState, PossessionStateCalculator, TouchState,
    TouchStateCalculator,
};
use crate::*;

pub type DerivedSignalId = &'static str;

pub const TOUCH_STATE_SIGNAL_ID: DerivedSignalId = "touch_state";
pub const POSSESSION_STATE_SIGNAL_ID: DerivedSignalId = "possession_state";
pub const BACKBOARD_BOUNCE_STATE_SIGNAL_ID: DerivedSignalId = "backboard_bounce_state";

#[derive(Default)]
pub struct AnalysisContext {
    values: HashMap<DerivedSignalId, Box<dyn Any>>,
}

impl AnalysisContext {
    pub fn get<T: 'static>(&self, id: DerivedSignalId) -> Option<&T> {
        self.values.get(id)?.downcast_ref::<T>()
    }

    pub fn insert<T: 'static>(&mut self, id: DerivedSignalId, value: T) {
        self.insert_box(id, Box::new(value));
    }

    fn insert_box(&mut self, id: DerivedSignalId, value: Box<dyn Any>) {
        self.values.insert(id, value);
    }

    fn clear(&mut self) {
        self.values.clear();
    }
}

pub trait DerivedSignal {
    fn id(&self) -> DerivedSignalId;

    fn dependencies(&self) -> &'static [DerivedSignalId] {
        &[]
    }

    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        Ok(())
    }

    fn evaluate(
        &mut self,
        sample: &FrameState,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>>;

    fn finish(&mut self) -> SubtrActorResult<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DerivedSignalGraph {
    nodes: Vec<Box<dyn DerivedSignal>>,
    evaluation_order: Vec<usize>,
    context: AnalysisContext,
    order_dirty: bool,
}

impl DerivedSignalGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_signal<S: DerivedSignal + 'static>(mut self, signal: S) -> Self {
        self.push(signal);
        self
    }

    pub fn push<S: DerivedSignal + 'static>(&mut self, signal: S) {
        self.nodes.push(Box::new(signal));
        self.order_dirty = true;
    }

    pub fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.rebuild_order_if_needed()?;
        for node in &mut self.nodes {
            node.on_replay_meta(meta)?;
        }
        Ok(())
    }

    pub fn evaluate(&mut self, sample: &FrameState) -> SubtrActorResult<&AnalysisContext> {
        self.rebuild_order_if_needed()?;
        self.context.clear();

        for node_index in &self.evaluation_order {
            let node = &mut self.nodes[*node_index];
            if let Some(value) = node.evaluate(sample, &self.context)? {
                self.context.insert_box(node.id(), value);
            }
        }

        Ok(&self.context)
    }

    pub fn finish(&mut self) -> SubtrActorResult<()> {
        for node in &mut self.nodes {
            node.finish()?;
        }
        Ok(())
    }

    fn rebuild_order_if_needed(&mut self) -> SubtrActorResult<()> {
        if !self.order_dirty {
            return Ok(());
        }

        let id_to_index: HashMap<_, _> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id(), index))
            .collect();
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut order = Vec::with_capacity(self.nodes.len());

        for node in &self.nodes {
            Self::visit_node(
                node.id(),
                &id_to_index,
                &self.nodes,
                &mut visiting,
                &mut visited,
                &mut order,
            )?;
        }

        self.evaluation_order = order.into_iter().map(|id| id_to_index[&id]).collect();
        self.order_dirty = false;
        Ok(())
    }

    fn visit_node(
        node_id: DerivedSignalId,
        id_to_index: &HashMap<DerivedSignalId, usize>,
        nodes: &[Box<dyn DerivedSignal>],
        visiting: &mut HashSet<DerivedSignalId>,
        visited: &mut HashSet<DerivedSignalId>,
        order: &mut Vec<DerivedSignalId>,
    ) -> SubtrActorResult<()> {
        if visited.contains(&node_id) {
            return Ok(());
        }
        if !visiting.insert(node_id) {
            return SubtrActorError::new_result(SubtrActorErrorVariant::DerivedSignalGraphError(
                format!("Cycle detected in derived signal graph at {node_id}"),
            ));
        }

        let node = &nodes[id_to_index[&node_id]];
        for dependency in node.dependencies() {
            if !id_to_index.contains_key(dependency) {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::DerivedSignalGraphError(format!(
                        "Missing derived signal dependency {dependency} for {node_id}"
                    )),
                );
            }
            Self::visit_node(dependency, id_to_index, nodes, visiting, visited, order)?;
        }

        visiting.remove(&node_id);
        visited.insert(node_id);
        order.push(node_id);
        Ok(())
    }
}

#[derive(Default)]
pub struct TouchStateSignal {
    calculator: TouchStateCalculator,
}

impl TouchStateSignal {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DerivedSignal for TouchStateSignal {
    fn id(&self) -> DerivedSignalId {
        TOUCH_STATE_SIGNAL_ID
    }

    fn evaluate(
        &mut self,
        sample: &FrameState,
        _ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>> {
        Ok(Some(Box::new(self.calculator.update(
            &FrameInfo {
                frame_number: sample.frame_number,
                time: sample.time,
                dt: sample.dt,
                seconds_remaining: sample.seconds_remaining,
            },
            &BallFrameState {
                ball: sample.ball.clone(),
            },
            &PlayerFrameState {
                players: sample.players.clone(),
            },
            &FrameEventsState {
                active_demos: sample.active_demos.clone(),
                demo_events: sample.demo_events.clone(),
                boost_pad_events: sample.boost_pad_events.clone(),
                touch_events: sample.touch_events.clone(),
                dodge_refreshed_events: sample.dodge_refreshed_events.clone(),
                player_stat_events: sample.player_stat_events.clone(),
                goal_events: sample.goal_events.clone(),
            },
            sample.is_live_play(),
        ))))
    }
}

#[derive(Default)]
pub struct PossessionStateSignal {
    calculator: PossessionStateCalculator,
}

impl PossessionStateSignal {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DerivedSignal for PossessionStateSignal {
    fn id(&self) -> DerivedSignalId {
        POSSESSION_STATE_SIGNAL_ID
    }

    fn dependencies(&self) -> &'static [DerivedSignalId] {
        &[TOUCH_STATE_SIGNAL_ID]
    }

    fn evaluate(
        &mut self,
        sample: &FrameState,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>> {
        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        Ok(Some(Box::new(self.calculator.update(
            &FrameInfo {
                frame_number: sample.frame_number,
                time: sample.time,
                dt: sample.dt,
                seconds_remaining: sample.seconds_remaining,
            },
            &touch_state,
            sample.is_live_play(),
        ))))
    }
}

#[derive(Default)]
pub struct BackboardBounceStateSignal {
    calculator: BackboardBounceCalculator,
}

impl BackboardBounceStateSignal {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DerivedSignal for BackboardBounceStateSignal {
    fn id(&self) -> DerivedSignalId {
        BACKBOARD_BOUNCE_STATE_SIGNAL_ID
    }

    fn evaluate(
        &mut self,
        sample: &FrameState,
        _ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>> {
        Ok(Some(Box::new(self.calculator.update(
            &FrameInfo {
                frame_number: sample.frame_number,
                time: sample.time,
                dt: sample.dt,
                seconds_remaining: sample.seconds_remaining,
            },
            &BallFrameState {
                ball: sample.ball.clone(),
            },
            &FrameEventsState {
                active_demos: sample.active_demos.clone(),
                demo_events: sample.demo_events.clone(),
                boost_pad_events: sample.boost_pad_events.clone(),
                touch_events: sample.touch_events.clone(),
                dodge_refreshed_events: sample.dodge_refreshed_events.clone(),
                player_stat_events: sample.player_stat_events.clone(),
                goal_events: sample.goal_events.clone(),
            },
            sample.is_live_play(),
        ))))
    }
}

#[derive(Default)]
pub struct FiftyFiftyStateSignal {
    calculator: FiftyFiftyStateCalculator,
}

impl FiftyFiftyStateSignal {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DerivedSignal for FiftyFiftyStateSignal {
    fn id(&self) -> DerivedSignalId {
        FIFTY_FIFTY_STATE_SIGNAL_ID
    }

    fn dependencies(&self) -> &'static [DerivedSignalId] {
        &[TOUCH_STATE_SIGNAL_ID, POSSESSION_STATE_SIGNAL_ID]
    }

    fn evaluate(
        &mut self,
        sample: &FrameState,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<Option<Box<dyn Any>>> {
        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        let possession_state = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();

        Ok(Some(Box::new(self.calculator.update(
            sample,
            &touch_state,
            &possession_state,
        ))))
    }
}

pub fn default_derived_signal_graph() -> DerivedSignalGraph {
    derived_signal_graph_for_ids([
        TOUCH_STATE_SIGNAL_ID,
        POSSESSION_STATE_SIGNAL_ID,
        BACKBOARD_BOUNCE_STATE_SIGNAL_ID,
        FIFTY_FIFTY_STATE_SIGNAL_ID,
    ])
}

fn derived_signal_dependencies(id: DerivedSignalId) -> &'static [DerivedSignalId] {
    match id {
        TOUCH_STATE_SIGNAL_ID => &[],
        POSSESSION_STATE_SIGNAL_ID => &[TOUCH_STATE_SIGNAL_ID],
        BACKBOARD_BOUNCE_STATE_SIGNAL_ID => &[],
        FIFTY_FIFTY_STATE_SIGNAL_ID => &[TOUCH_STATE_SIGNAL_ID, POSSESSION_STATE_SIGNAL_ID],
        _ => &[],
    }
}

fn add_signal_and_dependencies(id: DerivedSignalId, requested: &mut BTreeSet<DerivedSignalId>) {
    if !requested.insert(id) {
        return;
    }

    for dependency in derived_signal_dependencies(id) {
        add_signal_and_dependencies(dependency, requested);
    }
}

pub fn derived_signal_graph_for_ids<I>(ids: I) -> DerivedSignalGraph
where
    I: IntoIterator<Item = DerivedSignalId>,
{
    let mut requested = BTreeSet::new();
    for id in ids {
        add_signal_and_dependencies(id, &mut requested);
    }

    let mut graph = DerivedSignalGraph::new();
    for id in requested {
        match id {
            TOUCH_STATE_SIGNAL_ID => graph.push(TouchStateSignal::new()),
            POSSESSION_STATE_SIGNAL_ID => graph.push(PossessionStateSignal::new()),
            BACKBOARD_BOUNCE_STATE_SIGNAL_ID => graph.push(BackboardBounceStateSignal::new()),
            FIFTY_FIFTY_STATE_SIGNAL_ID => graph.push(FiftyFiftyStateSignal::new()),
            _ => {}
        }
    }

    graph
}

#[cfg(test)]
#[path = "analysis_test.rs"]
mod tests;
