use std::any::{type_name, Any, TypeId};

use crate::{ReplayMeta, SubtrActorResult};

use super::{AnalysisDependency, AnalysisStateContext};

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

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
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

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()>;

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

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        AnalysisNode::finish(self, ctx)
    }

    fn state_any(&self) -> &dyn Any {
        self.state()
    }
}
