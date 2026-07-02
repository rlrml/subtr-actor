//! The analysis-graph runtime: a dependency DAG of [`AnalysisNode`]s that turn
//! raw replay frames into derived state, gameplay events, and stats.
//!
//! # How it works
//!
//! Each node implements [`AnalysisNode`]: it declares the upstream state it
//! needs via [`dependencies`](AnalysisNode::dependencies), reads that state
//! through an [`AnalysisStateContext`] each frame in
//! [`evaluate`](AnalysisNode::evaluate), and exposes its own typed
//! [`state`](AnalysisNode::state) for downstream nodes. Source nodes read the
//! per-frame `FrameInput`; higher-level nodes build
//! on their outputs. The graph topologically resolves dependencies, so adding a
//! node automatically pulls in everything it needs.
//!
//! Most nodes are thin wrappers around a *calculator* (see [`crate::stats`]);
//! the node handles graph plumbing while the calculator holds the detection
//! logic.
//!
//! # Building a graph
//!
//! - [`AnalysisGraph::new`] + [`with_node`](AnalysisGraph::with_node) /
//!   [`push_node`](AnalysisGraph::push_node) to assemble nodes by hand.
//! - [`graph_with_builtin_analysis_nodes`] / [`graph_with_all_analysis_nodes`]
//!   to build from the built-in registry by name.
//! - [`collect_builtin_analysis_graph_for_replay`] to build *and* run a graph
//!   over a replay in one call.
//!
//! The names accepted by the registry are listed in
//! [`BUILTIN_ANALYSIS_NODE_NAMES`] (with aliases in
//! [`BUILTIN_ANALYSIS_NODE_ALIASES`]).
//!
//! # The nodes
//!
//! All node types are re-exported from this module; their first-line summaries
//! appear in the item list below, and the [`AnalysisNode`] *Implementors* list
//! is another way to browse them. By role:
//!
//! - **Per-frame source state** — [`FrameInfoNode`], [`GameplayStateNode`],
//!   [`BallFrameStateNode`], [`PlayerFrameStateNode`], [`FrameEventsStateNode`],
//!   [`LivePlayNode`], [`SettingsNode`].
//! - **Shared derived state** — [`TouchStateNode`], [`PossessionStateNode`],
//!   [`PlayerPossessionNode`], [`PossessionNode`], [`BallHalfNode`],
//!   [`PlayerVerticalStateNode`], [`PositioningNode`], [`RotationNode`],
//!   [`BackboardBounceStateNode`], [`FiftyFiftyStateNode`],
//!   [`ContinuousBallControlNode`].
//! - **Mechanic detection** — [`FlickNode`], [`HalfFlipNode`],
//!   [`SpeedFlipNode`], [`WavedashNode`], [`PowerslideNode`],
//!   [`FlipImpulseNode`], [`DodgeResetNode`], [`WallAerialNode`],
//!   [`WallAerialShotNode`], [`CeilingShotNode`], [`DoubleTapNode`],
//!   [`HalfVolleyNode`], [`OneTimerNode`], [`BallCarryNode`] (carries/air
//!   dribbles).
//! - **Play & contest detection** — [`TouchNode`], [`PassNode`], [`CenterNode`],
//!   [`KickoffNode`], [`BumpNode`], [`DemoNode`], [`RushNode`],
//!   [`ControlledPlayNode`], [`TerritorialPressureNode`], [`WhiffNode`],
//!   [`FiftyFiftyNode`], [`BackboardNode`], [`MovementNode`], [`BoostNode`].
//! - **Match-level & projection** — [`MatchStatsNode`], goal-tag nodes (e.g.
//!   [`HalfVolleyGoalNode`] plus the `*_goal` registry names),
//!   [`StatsProjectionNode`], [`StatsTimelineEventsNode`],
//!   [`StatsTimelineFrameNode`].
//!
//! See the [stats-runtime guide](crate::guides::calculators_and_analysis_nodes)
//! for the full DAG map.

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
    LoosePossessionNode,
    PossessionNode,
    PossessionStateNode,
    BallHalfNode,
    BallThirdNode,
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
    HalfVolleyGoalNode, OneTimerNode,
    PassNode,
    DodgeResetNode,
    BallCarryNode,
    AirDribbleNode,
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
