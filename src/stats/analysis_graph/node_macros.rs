macro_rules! impl_analysis_node {
    (
        node = $node:ident,
        state = $state:ty,
        name = $name:literal,
        dependencies = [$($dependency:expr => $dependency_ty:ty),* $(,)?],
        $(on_replay_meta = |$meta_self:ident, $meta:ident| $on_replay_meta:block,)?
        call = $field:ident.$method:ident
        $(, finish = $finish_field:ident.$finish_method:ident)?
        $(,)?
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
                fn on_replay_meta(
                    &mut self,
                    $meta: &ReplayMeta,
                ) -> SubtrActorResult<()> {
                    let $meta_self = self;
                    $on_replay_meta
                }
            )?

            fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                let _ = ctx;
                self.$field.$method($(ctx.get::<$dependency_ty>()?),*)
            }

            $(
                fn finish(&mut self) -> SubtrActorResult<()> {
                    self.$finish_field.$finish_method()
                }
            )?

            fn state(&self) -> &Self::State {
                &self.$field
            }
        }

        pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
            Box::new($node::new())
        }
    };

    (
        node = $node:ident,
        state = $state:ty,
        name = $name:literal,
        dependencies = [$($dependency:expr => $dependency_ty:ty),* $(,)?],
        $(on_replay_meta = |$meta_self:ident, $meta:ident| $on_replay_meta:block,)?
        update_state = $field:ident.$method:ident
        $(, finish = $finish_field:ident.$finish_method:ident)?
        $(,)?
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
                fn on_replay_meta(
                    &mut self,
                    $meta: &ReplayMeta,
                ) -> SubtrActorResult<()> {
                    let $meta_self = self;
                    $on_replay_meta
                }
            )?

            fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                let _ = ctx;
                self.state = self.$field.$method($(ctx.get::<$dependency_ty>()?),*);
                Ok(())
            }

            $(
                fn finish(&mut self) -> SubtrActorResult<()> {
                    self.$finish_field.$finish_method()
                }
            )?

            fn state(&self) -> &Self::State {
                &self.state
            }
        }

        pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
            Box::new($node::new())
        }
    };

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
                fn on_replay_meta(
                    &mut self,
                    $meta: &ReplayMeta,
                ) -> SubtrActorResult<()> {
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
                fn finish(&mut self) -> SubtrActorResult<()> {
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
