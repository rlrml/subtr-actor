macro_rules! goal_tag_node {
    ($node:ident, $calculator:ident, $name:literal) => {
        pub struct $node {
            calculator: $calculator,
        }

        impl $node {
            pub fn new() -> Self {
                Self {
                    calculator: $calculator::new(),
                }
            }
        }

        impl Default for $node {
            fn default() -> Self {
                Self::new()
            }
        }

        impl AnalysisNode for $node {
            type State = $calculator;

            fn name(&self) -> &'static str {
                $name
            }

            fn dependencies(&self) -> NodeDependencies {
                vec![match_stats_dependency()]
            }

            fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                self.calculator.update(ctx.get::<MatchStatsCalculator>()?)
            }

            fn state(&self) -> &Self::State {
                &self.calculator
            }
        }
    };
}

macro_rules! mechanic_goal_tag_node {
    ($node:ident, $calculator:ident, $name:literal, $dependency:ident, $dependency_type:ty) => {
        pub struct $node {
            calculator: $calculator,
        }

        impl $node {
            pub fn new() -> Self {
                Self {
                    calculator: $calculator::new(),
                }
            }
        }

        impl Default for $node {
            fn default() -> Self {
                Self::new()
            }
        }

        impl AnalysisNode for $node {
            type State = $calculator;

            fn name(&self) -> &'static str {
                $name
            }

            fn dependencies(&self) -> NodeDependencies {
                vec![match_stats_dependency(), $dependency()]
            }

            fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                self.calculator.update(
                    ctx.get::<MatchStatsCalculator>()?,
                    ctx.get::<$dependency_type>()?,
                )
            }

            fn state(&self) -> &Self::State {
                &self.calculator
            }
        }
    };
}
