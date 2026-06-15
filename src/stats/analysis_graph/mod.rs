use std::collections::HashSet;
use std::sync::OnceLock;

use crate::Collector;
use crate::{SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

pub mod graph;
pub use graph::{
    AnalysisDependency, AnalysisGraph, AnalysisNode, AnalysisNodeDyn, AnalysisStateContext,
    AnalysisStateRef,
};

#[macro_use]
mod node_macros;

mod collector;
mod nodes;

use crate::stats::calculators::FrameInput;
#[allow(unused_imports)]
pub use collector::AnalysisNodeCollector;
#[allow(unused_imports)]
pub use nodes::*;

/// Constructor for a builtin analysis node.
type BuiltinNodeCtor = fn() -> Box<dyn AnalysisNodeDyn>;

/// Construct a builtin node purely from its type. Every builtin node implements
/// [`Default`], so the registry below can be a plain list of node *types*. The
/// name each node reports through [`AnalysisNode::name`] is the single source of
/// truth — there is no parallel string list or `name => constructor` match to
/// keep in sync when adding a node.
fn boxed_node<N>() -> Box<dyn AnalysisNodeDyn>
where
    N: AnalysisNode + Default,
{
    Box::new(N::default())
}

macro_rules! builtin_analysis_nodes {
    ($($node:ty),+ $(,)?) => {
        /// Every builtin analysis node, as a list of types. Adding a node is one
        /// line here plus the node module itself — no name string, no match arm.
        const BUILTIN_ANALYSIS_NODE_CTORS: &[BuiltinNodeCtor] = &[$(boxed_node::<$node>),+];
    };
}

builtin_analysis_nodes! {
    FrameInfoNode,
    GameplayStateNode,
    BallFrameStateNode,
    PlayerFrameStateNode,
    FrameEventsStateNode,
    LivePlayNode,
    MatchStatsNode,
    BackboardNode,
    BackboardBounceStateNode,
    CeilingShotNode,
    CenterNode,
    ControlledPlayNode,
    ContinuousBallControlNode,
    DoubleTapNode,
    FiftyFiftyNode,
    FiftyFiftyStateNode,
    KickoffNode,
    PlayerPossessionNode,
    PossessionNode,
    PossessionStateNode,
    BallHalfNode,
    TerritorialPressureNode,
    RotationNode,
    RushNode,
    TouchNode,
    TouchStateNode,
    WallAerialNode,
    WallAerialShotNode,
    WhiffNode,
    WavedashNode,
    FlipImpulseNode,
    SpeedFlipNode,
    HalfFlipNode,
    HalfVolleyNode,
    FlickNode,
    AerialGoalNode,
    HighAerialGoalNode,
    LongDistanceGoalNode,
    OwnHalfGoalNode,
    EmptyNetGoalNode,
    CounterAttackGoalNode,
    SustainedPressureGoalNode,
    KickoffGoalNode,
    FlickGoalNode,
    CeilingShotGoalNode,
    DoubleTapGoalNode,
    OneTimerGoalNode,
    PassingGoalNode,
    AirDribbleGoalNode,
    FlipResetGoalNode,
    FlipIntoBallGoalNode,
    BumpGoalNode,
    DemoGoalNode,
    HalfVolleyGoalNode,
    MustyFlickNode,
    OneTimerNode,
    PassNode,
    DodgeResetNode,
    BallCarryNode,
    BoostNode,
    BumpNode,
    MovementNode,
    PositioningNode,
    PowerslideNode,
    PlayerVerticalStateNode,
    DemoNode,
    SettingsNode,
    StatsProjectionNode,
    StatsTimelineFrameNode,
    StatsTimelineEventsNode,
}

/// `(name, constructor)` for every builtin node, with the name read once from a
/// throwaway default instance. Built lazily and cached.
fn builtin_analysis_node_registry() -> &'static [(&'static str, BuiltinNodeCtor)] {
    static REGISTRY: OnceLock<Vec<(&'static str, BuiltinNodeCtor)>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        BUILTIN_ANALYSIS_NODE_CTORS
            .iter()
            .map(|&ctor| (ctor().name(), ctor))
            .collect()
    })
}

pub fn builtin_analysis_node_names() -> &'static [&'static str] {
    static NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        builtin_analysis_node_registry()
            .iter()
            .map(|(name, _)| *name)
            .collect()
    })
}

pub(crate) fn boxed_analysis_node_by_name(name: &str) -> Option<Box<dyn AnalysisNodeDyn>> {
    builtin_analysis_node_registry()
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, ctor)| ctor())
}

pub fn graph_with_builtin_analysis_nodes<I, S>(names: I) -> SubtrActorResult<AnalysisGraph>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    let mut seen = HashSet::new();
    for name in names {
        let name = name.as_ref();
        let node = boxed_analysis_node_by_name(name).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::UnknownStatsModuleName(
                name.to_owned(),
            ))
        })?;
        if !seen.insert(node.name()) {
            continue;
        }
        graph.push_boxed_node(node);
    }
    Ok(graph)
}

pub fn collect_analysis_graph_for_replay(
    replay: &boxcars::Replay,
    graph: AnalysisGraph,
) -> SubtrActorResult<AnalysisGraph> {
    let collector = collector::AnalysisNodeCollector::new(graph).process_replay(replay)?;
    Ok(collector.into_graph())
}

pub fn collect_builtin_analysis_graph_for_replay<I, S>(
    replay: &boxcars::Replay,
    names: I,
) -> SubtrActorResult<AnalysisGraph>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    collect_analysis_graph_for_replay(replay, graph_with_builtin_analysis_nodes(names)?)
}

pub fn all_analysis_nodes() -> Vec<Box<dyn AnalysisNodeDyn>> {
    builtin_analysis_node_registry()
        .iter()
        .map(|(_, ctor)| ctor())
        .collect()
}

pub fn graph_with_all_analysis_nodes() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    for node in all_analysis_nodes() {
        graph.push_boxed_node(node);
    }
    graph
}

#[cfg(test)]
#[path = "module_tests.rs"]
mod tests;
