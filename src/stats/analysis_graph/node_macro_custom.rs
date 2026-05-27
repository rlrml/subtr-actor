macro_rules! impl_analysis_node_custom {
    (
        node = $node:ident,
        state = $state:ty,
        name = $name:literal,
        dependencies = [$($dependency:expr),* $(,)?],
        inputs = {$($binding:ident : $binding_ty:ty),* $(,)?},
        $(on_replay_meta = |$meta_self:ident, $meta:ident| $on_replay_meta:block,)?
        evaluate = |$eval_self:ident| $evaluate:block,
        $(finish = |$finish_self:ident| $finish:block,)?
        state_ref = |$state_self:ident| $state_ref:expr $(,)?
    ) => {
        impl Default for $node {
            fn default() -> Self {
                Self::new()
            }
        }

        impl AnalysisNode for $node {
            type State = $state;

            fn name(&self) -> &'static str {
                $name
            }

            fn dependencies(&self) -> Vec<AnalysisDependency> {
                vec![$($dependency),*]
            }

            $(
                fn on_replay_meta(&mut self, $meta: &ReplayMeta) -> SubtrActorResult<()> {
                    let $meta_self = self;
                    $on_replay_meta
                }
            )?

            fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                let _ = ctx;
                $(let $binding = ctx.get::<$binding_ty>()?;)*
                let $eval_self = self;
                $evaluate
            }

            $(
                fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                    let $finish_self = self;
                    $finish
                }
            )?

            fn state(&self) -> &Self::State {
                let $state_self = self;
                $state_ref
            }
        }

        pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
            Box::new($node::new())
        }
    };
}
