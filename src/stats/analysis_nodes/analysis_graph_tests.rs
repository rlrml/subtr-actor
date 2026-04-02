use super::*;

#[derive(Debug, Default, PartialEq, Eq)]
struct BaseState(usize);

#[derive(Debug, Default, PartialEq, Eq)]
struct DoubledState(usize);

#[derive(Debug, Default, PartialEq, Eq)]
struct TripledState(usize);

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
