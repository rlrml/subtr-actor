use super::*;
use std::collections::BTreeSet;

use crate::stats::analysis_graph::{
    builtin_analysis_node_names, graph_with_builtin_analysis_nodes, nodes::DoubleTapNode,
};
use crate::stats::calculators::{
    ApproachConfidenceLevel, DOUBLE_TAP_EVENT_DEFINITION, EventCategory, FinalizationHorizon,
    FrameInput, StatsEvent, TIMELINE_EVENT_DEFINITION, UNKNOWN_DETECTION_CONFIDENCE,
    produced_event,
};

#[derive(Debug, Default, PartialEq, Eq)]
struct BaseState(usize);

#[derive(Debug, Default, PartialEq, Eq)]
struct DoubledState(usize);

#[derive(Debug, Default, PartialEq, Eq)]
struct TripledState(usize);

#[derive(Debug, Default, PartialEq, Eq)]
struct QuadrupledState(usize);

#[derive(Default)]
struct BaseNode {
    factor: usize,
    state: BaseState,
}

impl AnalysisNode for BaseNode {
    type State = BaseState;

    fn name(&self) -> &'static str {
        "base"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<usize>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let factor = if self.factor == 0 { 1 } else { self.factor };
        self.state.0 = ctx.get::<usize>()? * factor;
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[derive(Default)]
struct DoubledNode {
    state: DoubledState,
}

impl AnalysisNode for DoubledNode {
    type State = DoubledState;

    fn name(&self) -> &'static str {
        "doubled"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::with_default::<BaseState>(|| {
            Box::new(BaseNode::default())
        })]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state.0 = ctx.get::<BaseState>()?.0 * 2;
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[derive(Default)]
struct TripledNode {
    state: TripledState,
}

impl AnalysisNode for TripledNode {
    type State = TripledState;

    fn name(&self) -> &'static str {
        "tripled"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::with_default::<DoubledState>(|| {
            Box::new(DoubledNode::default())
        })]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state.0 = ctx.get::<DoubledState>()?.0 * 3;
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[derive(Default)]
struct QuadrupledNode {
    state: QuadrupledState,
}

impl AnalysisNode for QuadrupledNode {
    type State = QuadrupledState;

    fn name(&self) -> &'static str {
        "quadrupled"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::with_default::<BaseState>(|| {
            Box::new(BaseNode::default())
        })]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state.0 = ctx.get::<BaseState>()?.0 * 4;
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[derive(Default)]
struct AlternateBaseNode {
    state: BaseState,
}

impl AnalysisNode for AlternateBaseNode {
    type State = BaseState;

    fn name(&self) -> &'static str {
        "alternate_base"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<usize>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state.0 = ctx.get::<usize>()? * 10;
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[derive(Default)]
struct CycleAState;

#[derive(Default)]
struct CycleBState;

#[derive(Default)]
struct CycleANode {
    state: CycleAState,
}

impl AnalysisNode for CycleANode {
    type State = CycleAState;

    fn name(&self) -> &'static str {
        "cycle_a"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::with_default::<CycleBState>(|| {
            Box::new(CycleBNode::default())
        })]
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[derive(Default)]
struct CycleBNode {
    state: CycleBState,
}

impl AnalysisNode for CycleBNode {
    type State = CycleBState;

    fn name(&self) -> &'static str {
        "cycle_b"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::with_default::<CycleAState>(|| {
            Box::new(CycleANode::default())
        })]
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[test]
fn resolves_default_dependencies_and_evaluates_in_dependency_order() {
    let mut graph = AnalysisGraph::new()
        .with_root_state_type::<usize>()
        .with_node(TripledNode::default());

    graph.resolve().expect("graph should resolve");
    graph.set_root_state(4usize);
    graph.evaluate().expect("graph should evaluate");

    assert_eq!(graph.state::<BaseState>().unwrap(), &BaseState(4));
    assert_eq!(graph.state::<DoubledState>().unwrap(), &DoubledState(8));
    assert_eq!(graph.state::<TripledState>().unwrap(), &TripledState(24));
}

#[test]
fn explicit_provider_overrides_default_provider() {
    let mut graph = AnalysisGraph::new()
        .with_root_state_type::<usize>()
        .with_node(DoubledNode::default())
        .with_node(AlternateBaseNode::default());

    graph.resolve().expect("graph should resolve");
    graph.set_root_state(3usize);
    graph.evaluate().expect("graph should evaluate");

    assert_eq!(graph.state::<BaseState>().unwrap(), &BaseState(30));
    assert_eq!(graph.state::<DoubledState>().unwrap(), &DoubledState(60));
}

#[test]
fn rejects_duplicate_state_providers() {
    let resolution = AnalysisGraph::new()
        .with_root_state_type::<usize>()
        .with_node(BaseNode::default())
        .with_node(AlternateBaseNode::default())
        .resolve();

    let error = resolution.expect_err("duplicate providers should fail");
    assert!(matches!(
        error.variant,
        SubtrActorErrorVariant::CallbackError(_)
    ));
}

#[test]
fn rejects_dependency_cycles() {
    let resolution = AnalysisGraph::new()
        .with_node(CycleANode::default())
        .resolve();

    let error = resolution.expect_err("cycle should fail");
    assert!(matches!(
        error.variant,
        SubtrActorErrorVariant::CallbackError(_)
    ));
}

#[test]
fn renders_ascii_dag() {
    let rendered = AnalysisGraph::new()
        .with_root_state_type::<usize>()
        .with_node(TripledNode::default())
        .render_ascii_dag()
        .expect("graph should render");

    assert!(rendered.starts_with("AnalysisGraph\n"));
    assert!(rendered.contains("tripled"));
    assert!(rendered.contains("doubled"));
    assert!(rendered.contains("base"));
    assert!(rendered.contains("root:usize"));
}

#[test]
fn renders_shared_dependencies_as_references() {
    let rendered = AnalysisGraph::new()
        .with_root_state_type::<usize>()
        .with_node(TripledNode::default())
        .with_node(QuadrupledNode::default())
        .render_ascii_dag()
        .expect("graph should render");

    assert!(rendered.starts_with("AnalysisGraph\n"));
    assert!(rendered.contains("tripled"));
    assert!(rendered.contains("quadrupled"));
    assert_eq!(rendered.matches("base").count(), 1);
    assert!(rendered.contains("root:usize"));
}

#[test]
fn discovers_static_event_metadata_from_analysis_nodes() {
    let mut graph = AnalysisGraph::new()
        .with_input_state_type::<FrameInput>()
        .with_node(DoubleTapNode::new());
    let emitted_events = graph
        .emitted_events()
        .expect("graph should resolve emitted event metadata");

    let emitted = emitted_events
        .iter()
        .find(|emitted| emitted.event.id == "double_tap")
        .expect("double tap node should emit double tap metadata");
    assert_eq!(emitted.event.id, "double_tap");
    assert_eq!(emitted.event.category, EventCategory::Mechanic);
    assert_eq!(
        emitted.event.confidence.approach,
        ApproachConfidenceLevel::Unknown
    );
    assert_eq!(emitted.event.confidence, UNKNOWN_DETECTION_CONFIDENCE);
    assert_eq!(emitted.producer.node_name, "double_tap");
    assert_eq!(emitted.producer.node_type, "DoubleTapNode");
    assert_eq!(emitted.producer.calculator_type, "DoubleTapCalculator");
    assert_eq!(
        <crate::stats::calculators::DoubleTapEvent as StatsEvent>::DEFINITION,
        DOUBLE_TAP_EVENT_DEFINITION
    );
}

#[test]
fn builtin_event_metadata_contains_emitted_event_payloads() {
    let mut graph =
        graph_with_builtin_analysis_nodes(builtin_analysis_node_names().iter().copied())
            .expect("builtin graph should build");
    let emitted_events = graph
        .emitted_events()
        .expect("builtin graph should resolve emitted event metadata");

    let actual_ids = emitted_events
        .iter()
        .map(|event| event.event.id)
        .collect::<BTreeSet<_>>();
    let expected_ids = BTreeSet::from([
        "backboard_bounce",
        "ball_carry",
        "boost_pickups",
        "boost_respawn",
        "bump",
        "ceiling_shot",
        "center",
        "controlled_play",
        "core_player_scoreboard",
        "demolition",
        "dodge_reset",
        "double_tap",
        "fifty_fifty",
        "flick",
        "flip_reset",
        "dodge",
        "goal_context",
        "half_flip",
        "half_volley",
        "kickoff",
        "movement",
        "one_timer",
        "pass",
        "player_activity",
        "ball_proximity",
        "ball_depth",
        "field_third",
        "field_half",
        "depth_role",
        "shadow_defense",
        "loose_possession",
        "player_possession",
        "possession",
        "powerslide",
        "ball_half",
        "ball_third",
        "rotation_role",
        "first_man_change",
        "rush",
        "speed_flip",
        "territorial_pressure",
        "timeline",
        "touch",
        "wall_aerial",
        "wall_aerial_shot",
        "wavedash",
        "whiff",
    ]);

    assert_eq!(actual_ids, expected_ids);
    assert!(emitted_events.iter().all(|event| {
        event.event.confidence == UNKNOWN_DETECTION_CONFIDENCE
            && event.producer.implementation_notes.is_empty()
    }));
}

/// Stream declaration shared by the toy projecting nodes below: both project
/// the `timeline` stream (the graph rejects projections on undeclared
/// streams). Unique cross-node ownership is a property of the builtin node
/// catalog, asserted over `all_analysis_nodes()` in the module tests — toy
/// nodes may share a stream to exercise the duplicate-id invariant.
const PROJECTING_TEST_EMITTED_EVENTS: &[crate::stats::calculators::EmittedEvent] =
    &[produced_event(
        &TIMELINE_EVENT_DEFINITION,
        "timeline",
        FinalizationHorizon::EndPlus(0.0),
        "projecting_test",
        "ProjectingNode",
        "test-only",
    )];

/// A toy projecting node: publishes a fixed event set through
/// `project_events`, exercising the graph's central store without any
/// calculator machinery.
struct ProjectingNode {
    name: &'static str,
    state: BaseState,
    events: Vec<Event>,
}

fn projected_event(id: &str, lifecycle: EventLifecycle, time: f32) -> Event {
    projected_event_on_stream("timeline", id, lifecycle, time)
}

fn projected_event_on_stream(
    stream: &str,
    id: &str,
    lifecycle: EventLifecycle,
    time: f32,
) -> Event {
    Event {
        meta: EventMeta {
            id: id.to_owned(),
            stream: stream.to_owned(),
            label: stats_timeline_event_label(stream),
            scope: EventScope::Match,
            lifecycle,
            timing: EventTiming::Moment { frame: 10, time },
            primary_player: None,
            secondary_player: None,
            player_position: None,
            ball_position: None,
            team_is_team_0: None,
            confidence: None,
            properties: Vec::new(),
        },
        payload: EventPayload::Timeline(TimelineEvent {
            time,
            frame: Some(10),
            kind: TimelineEventKind::Shot,
            player_id: None,
            player_position: None,
            is_team_0: None,
        }),
    }
}

impl AnalysisNode for ProjectingNode {
    type State = BaseState;

    fn name(&self) -> &'static str {
        self.name
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        PROJECTING_TEST_EMITTED_EVENTS
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        Ok(())
    }

    fn project_events(&self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>> {
        Ok(self.events.clone())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

/// A second projecting node type: the store aggregates sets from all nodes
/// (`State` types must be distinct, so the second node needs its own type).
struct OtherProjectingNode {
    state: DoubledState,
    events: Vec<Event>,
}

impl AnalysisNode for OtherProjectingNode {
    type State = DoubledState;

    fn name(&self) -> &'static str {
        "other_projecting"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        PROJECTING_TEST_EMITTED_EVENTS
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        Ok(())
    }

    fn project_events(&self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>> {
        Ok(self.events.clone())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[test]
fn project_events_now_aggregates_sets_from_all_nodes() {
    let mut graph = AnalysisGraph::new()
        .with_node(ProjectingNode {
            name: "projecting_a",
            state: BaseState::default(),
            events: vec![projected_event("a", EventLifecycle::Confirmed, 1.0)],
        })
        .with_node(OtherProjectingNode {
            state: DoubledState::default(),
            events: vec![projected_event("b", EventLifecycle::Confirmed, 2.0)],
        });

    graph
        .project_events_now()
        .expect("distinct ids from two nodes aggregate cleanly");
    let ids: Vec<&str> = graph
        .event_transaction_log()
        .current_events()
        .iter()
        .map(|event| event.meta.id.as_str())
        .collect();
    assert_eq!(ids, ["a", "b"]);

    // `finish` finalizes the aggregate set from all nodes.
    graph.finish().expect("finish applies the final projection");
    assert!(
        graph
            .event_transaction_log()
            .current_events()
            .iter()
            .all(|event| event.meta.lifecycle == EventLifecycle::Finalized),
        "finish must finalize every aggregated event"
    );
}

#[test]
fn duplicate_ids_across_nodes_violate_stream_ownership() {
    let mut graph = AnalysisGraph::new()
        .with_node(ProjectingNode {
            name: "projecting_a",
            state: BaseState::default(),
            events: vec![projected_event("a", EventLifecycle::Confirmed, 1.0)],
        })
        .with_node(OtherProjectingNode {
            state: DoubledState::default(),
            events: vec![projected_event("a", EventLifecycle::Confirmed, 1.0)],
        });

    // Two nodes projecting the same id means two nodes claim the same
    // stream+anchor — the store's duplicate check makes that loud.
    let error = graph
        .project_events_now()
        .expect_err("duplicate ids across nodes must error");
    assert!(matches!(
        error.variant,
        SubtrActorErrorVariant::TimelineEventInvariantViolation(_)
    ));
}

#[test]
fn projecting_an_undeclared_stream_is_rejected() {
    // `ProjectingNode` declares only the `timeline` stream; an event on any
    // other stream must be rejected before it reaches the store.
    let mut graph = AnalysisGraph::new().with_node(ProjectingNode {
        name: "projecting_a",
        state: BaseState::default(),
        events: vec![projected_event_on_stream(
            "undeclared_stream",
            "undeclared_stream:10:0",
            EventLifecycle::Confirmed,
            1.0,
        )],
    });

    let error = graph
        .project_events_now()
        .expect_err("projecting an undeclared stream must error");
    assert!(matches!(
        error.variant,
        SubtrActorErrorVariant::TimelineEventInvariantViolation(_)
    ));
}
