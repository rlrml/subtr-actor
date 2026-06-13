use crate::stats::analysis_graph::{AnalysisDependency, AnalysisGraph};
use crate::*;
/// Re-export of `derive_new` used by the public ndarray feature macros.
pub use ::derive_new;
/// Re-export of `paste` used by the public ndarray feature macros.
pub use ::paste;
use boxcars;
use std::any::type_name;
use std::marker::PhantomData;
use std::sync::Arc;

/// Typed, read-only view of analysis state available to analysis-backed features.
pub struct AnalysisFeatureContext<'a> {
    graph: &'a AnalysisGraph,
}

impl<'a> AnalysisFeatureContext<'a> {
    pub(crate) fn new(graph: &'a AnalysisGraph) -> Self {
        Self { graph }
    }

    pub fn maybe_state<T: 'static>(&self) -> Option<&'a T> {
        self.graph.state::<T>()
    }

    pub fn state<T: 'static>(&self) -> SubtrActorResult<&'a T> {
        self.maybe_state::<T>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
                "missing analysis state {}",
                type_name::<T>(),
            )))
        })
    }
}

/// Object-safe interface for frame-level feature extraction.
pub trait FeatureAdder<F> {
    fn features_added(&self) -> usize {
        self.get_column_headers().len()
    }

    fn get_column_headers(&self) -> &[&str];

    fn add_features(
        &self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()>;
}

/// Object-safe interface for frame-level features backed by the analysis graph.
pub trait AnalysisFeatureAdder<F> {
    fn features_added(&self) -> usize {
        self.get_column_headers().len()
    }

    fn get_column_headers(&self) -> &[&str];

    fn analysis_dependencies(&self) -> Vec<AnalysisDependency>;

    fn add_features(
        &self,
        context: &AnalysisFeatureContext<'_>,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()>;
}

/// Fixed-width analysis-backed feature extractor with compile-time column count validation.
pub trait LengthCheckedAnalysisFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn analysis_dependencies(&self) -> Vec<AnalysisDependency>;

    fn get_features(
        &self,
        context: &AnalysisFeatureContext<'_>,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<[F; N]>;
}

/// Implements [`AnalysisFeatureAdder`] for a fixed-width analysis-backed type.
#[macro_export]
macro_rules! impl_analysis_feature_adder {
    ($struct_name:ident) => {
        impl<F: TryFrom<f32>> AnalysisFeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn add_features(
                &self,
                context: &AnalysisFeatureContext<'_>,
                processor: &dyn ProcessorView,
                frame: &boxcars::Frame,
                frame_count: usize,
                current_time: f32,
                vector: &mut Vec<F>,
            ) -> SubtrActorResult<()> {
                Ok(vector.extend(self.get_features(
                    context,
                    processor,
                    frame,
                    frame_count,
                    current_time,
                )?))
            }

            fn get_column_headers(&self) -> &[&str] {
                self.get_column_headers_array()
            }

            fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
                LengthCheckedAnalysisFeatureAdder::analysis_dependencies(self)
            }
        }
    };
}

/// Fixed-width feature extractor with compile-time column count validation.
pub trait LengthCheckedFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn get_features(
        &self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<[F; N]>;
}

/// Implements [`FeatureAdder`] for a type that already satisfies [`LengthCheckedFeatureAdder`].
#[macro_export]
macro_rules! impl_feature_adder {
    ($struct_name:ident) => {
        impl<F: TryFrom<f32>> FeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn add_features(
                &self,
                processor: &dyn ProcessorView,
                frame: &boxcars::Frame,
                frame_count: usize,
                current_time: f32,
                vector: &mut Vec<F>,
            ) -> SubtrActorResult<()> {
                Ok(
                    vector.extend(self.get_features(
                        processor,
                        frame,
                        frame_count,
                        current_time,
                    )?),
                )
            }

            fn get_column_headers(&self) -> &[&str] {
                self.get_column_headers_array()
            }
        }
    };
}

/// Object-safe interface for per-player feature extraction.
pub trait PlayerFeatureAdder<F> {
    fn features_added(&self) -> usize {
        self.get_column_headers().len()
    }

    fn get_column_headers(&self) -> &[&str];

    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()>;
}

/// Arguments supplied to object-safe per-player analysis feature adders.
#[derive(Clone, Copy)]
pub struct AnalysisPlayerFeatureInput<'a, 'ctx> {
    pub context: &'a AnalysisFeatureContext<'ctx>,
    pub player_id: &'a PlayerId,
    pub processor: &'a dyn ProcessorView,
    pub frame: &'a boxcars::Frame,
    pub frame_count: usize,
    pub current_time: f32,
}

/// Object-safe interface for per-player features backed by the analysis graph.
pub trait AnalysisPlayerFeatureAdder<F> {
    fn features_added(&self) -> usize {
        self.get_column_headers().len()
    }

    fn get_column_headers(&self) -> &[&str];

    fn analysis_dependencies(&self) -> Vec<AnalysisDependency>;

    fn add_features(
        &self,
        input: AnalysisPlayerFeatureInput<'_, '_>,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()>;
}

/// Fixed-width per-player analysis-backed feature extractor.
pub trait LengthCheckedAnalysisPlayerFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn analysis_dependencies(&self) -> Vec<AnalysisDependency>;

    fn get_features(
        &self,
        context: &AnalysisFeatureContext<'_>,
        player_id: &PlayerId,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<[F; N]>;
}

/// Implements [`AnalysisPlayerFeatureAdder`] for a fixed-width analysis-backed type.
#[macro_export]
macro_rules! impl_analysis_player_feature_adder {
    ($struct_name:ident) => {
        impl<F: TryFrom<f32>> AnalysisPlayerFeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn add_features(
                &self,
                input: AnalysisPlayerFeatureInput<'_, '_>,
                vector: &mut Vec<F>,
            ) -> SubtrActorResult<()> {
                Ok(vector.extend(self.get_features(
                    input.context,
                    input.player_id,
                    input.processor,
                    input.frame,
                    input.frame_count,
                    input.current_time,
                )?))
            }

            fn get_column_headers(&self) -> &[&str] {
                self.get_column_headers_array()
            }

            fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
                LengthCheckedAnalysisPlayerFeatureAdder::analysis_dependencies(self)
            }
        }
    };
}

pub struct DynamicAnalysisFeatureAdder<F, G, const N: usize> {
    get_features: G,
    column_headers: &'static [&'static str; N],
    dependencies: Vec<AnalysisDependency>,
    _marker: PhantomData<F>,
}

impl<F, G, const N: usize> DynamicAnalysisFeatureAdder<F, G, N> {
    pub fn new(
        column_headers: &'static [&'static str; N],
        dependencies: Vec<AnalysisDependency>,
        get_features: G,
    ) -> Self {
        Self {
            get_features,
            column_headers,
            dependencies,
            _marker: PhantomData,
        }
    }
}

impl<F, G, const N: usize> AnalysisFeatureAdder<F> for DynamicAnalysisFeatureAdder<F, G, N>
where
    F: Send + Sync + 'static,
    G: Fn(
            &AnalysisFeatureContext<'_>,
            &dyn ProcessorView,
            &boxcars::Frame,
            usize,
            f32,
        ) -> SubtrActorResult<[F; N]>
        + Send
        + Sync
        + 'static,
{
    fn get_column_headers(&self) -> &[&str] {
        self.column_headers.as_slice()
    }

    fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
        self.dependencies.clone()
    }

    fn add_features(
        &self,
        context: &AnalysisFeatureContext<'_>,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()> {
        vector.extend((self.get_features)(
            context,
            processor,
            frame,
            frame_count,
            current_time,
        )?);
        Ok(())
    }
}

pub struct DynamicAnalysisPlayerFeatureAdder<F, G, const N: usize> {
    get_features: G,
    column_headers: &'static [&'static str; N],
    dependencies: Vec<AnalysisDependency>,
    _marker: PhantomData<F>,
}

impl<F, G, const N: usize> DynamicAnalysisPlayerFeatureAdder<F, G, N> {
    pub fn new(
        column_headers: &'static [&'static str; N],
        dependencies: Vec<AnalysisDependency>,
        get_features: G,
    ) -> Self {
        Self {
            get_features,
            column_headers,
            dependencies,
            _marker: PhantomData,
        }
    }
}

impl<F, G, const N: usize> AnalysisPlayerFeatureAdder<F>
    for DynamicAnalysisPlayerFeatureAdder<F, G, N>
where
    F: Send + Sync + 'static,
    G: Fn(
            &AnalysisFeatureContext<'_>,
            &PlayerId,
            &dyn ProcessorView,
            &boxcars::Frame,
            usize,
            f32,
        ) -> SubtrActorResult<[F; N]>
        + Send
        + Sync
        + 'static,
{
    fn get_column_headers(&self) -> &[&str] {
        self.column_headers.as_slice()
    }

    fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
        self.dependencies.clone()
    }

    fn add_features(
        &self,
        input: AnalysisPlayerFeatureInput<'_, '_>,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()> {
        vector.extend((self.get_features)(
            input.context,
            input.player_id,
            input.processor,
            input.frame,
            input.frame_count,
            input.current_time,
        )?);
        Ok(())
    }
}

pub fn dynamic_analysis_feature_adder<F, G, const N: usize>(
    column_headers: &'static [&'static str; N],
    dependencies: Vec<AnalysisDependency>,
    get_features: G,
) -> Arc<dyn AnalysisFeatureAdder<F> + Send + Sync + 'static>
where
    F: Send + Sync + 'static,
    G: Fn(
            &AnalysisFeatureContext<'_>,
            &dyn ProcessorView,
            &boxcars::Frame,
            usize,
            f32,
        ) -> SubtrActorResult<[F; N]>
        + Send
        + Sync
        + 'static,
{
    Arc::new(DynamicAnalysisFeatureAdder::new(
        column_headers,
        dependencies,
        get_features,
    ))
}

pub fn dynamic_analysis_player_feature_adder<F, G, const N: usize>(
    column_headers: &'static [&'static str; N],
    dependencies: Vec<AnalysisDependency>,
    get_features: G,
) -> Arc<dyn AnalysisPlayerFeatureAdder<F> + Send + Sync + 'static>
where
    F: Send + Sync + 'static,
    G: Fn(
            &AnalysisFeatureContext<'_>,
            &PlayerId,
            &dyn ProcessorView,
            &boxcars::Frame,
            usize,
            f32,
        ) -> SubtrActorResult<[F; N]>
        + Send
        + Sync
        + 'static,
{
    Arc::new(DynamicAnalysisPlayerFeatureAdder::new(
        column_headers,
        dependencies,
        get_features,
    ))
}

#[derive(Clone)]
pub enum NDArrayFeatureAdder<F> {
    Plain(Arc<dyn FeatureAdder<F> + Send + Sync>),
    Analysis(Arc<dyn AnalysisFeatureAdder<F> + Send + Sync>),
}

impl<F> NDArrayFeatureAdder<F> {
    pub fn plain(adder: Arc<dyn FeatureAdder<F> + Send + Sync>) -> Self {
        Self::Plain(adder)
    }

    pub fn analysis(adder: Arc<dyn AnalysisFeatureAdder<F> + Send + Sync>) -> Self {
        Self::Analysis(adder)
    }

    pub fn features_added(&self) -> usize {
        match self {
            Self::Plain(adder) => adder.features_added(),
            Self::Analysis(adder) => adder.features_added(),
        }
    }

    pub fn get_column_headers(&self) -> &[&str] {
        match self {
            Self::Plain(adder) => adder.get_column_headers(),
            Self::Analysis(adder) => adder.get_column_headers(),
        }
    }

    pub fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
        match self {
            Self::Plain(_) => Vec::new(),
            Self::Analysis(adder) => adder.analysis_dependencies(),
        }
    }

    pub fn is_analysis_backed(&self) -> bool {
        matches!(self, Self::Analysis(_))
    }
}

impl<F> From<Arc<dyn FeatureAdder<F> + Send + Sync>> for NDArrayFeatureAdder<F> {
    fn from(adder: Arc<dyn FeatureAdder<F> + Send + Sync>) -> Self {
        Self::plain(adder)
    }
}

impl<F> From<Arc<dyn AnalysisFeatureAdder<F> + Send + Sync>> for NDArrayFeatureAdder<F> {
    fn from(adder: Arc<dyn AnalysisFeatureAdder<F> + Send + Sync>) -> Self {
        Self::analysis(adder)
    }
}

pub type NDArrayFeatureAdders<F> = Vec<NDArrayFeatureAdder<F>>;

#[derive(Clone)]
pub enum NDArrayPlayerFeatureAdder<F> {
    Plain(Arc<dyn PlayerFeatureAdder<F> + Send + Sync>),
    Analysis(Arc<dyn AnalysisPlayerFeatureAdder<F> + Send + Sync>),
}

impl<F> NDArrayPlayerFeatureAdder<F> {
    pub fn plain(adder: Arc<dyn PlayerFeatureAdder<F> + Send + Sync>) -> Self {
        Self::Plain(adder)
    }

    pub fn analysis(adder: Arc<dyn AnalysisPlayerFeatureAdder<F> + Send + Sync>) -> Self {
        Self::Analysis(adder)
    }

    pub fn features_added(&self) -> usize {
        match self {
            Self::Plain(adder) => adder.features_added(),
            Self::Analysis(adder) => adder.features_added(),
        }
    }

    pub fn get_column_headers(&self) -> &[&str] {
        match self {
            Self::Plain(adder) => adder.get_column_headers(),
            Self::Analysis(adder) => adder.get_column_headers(),
        }
    }

    pub fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
        match self {
            Self::Plain(_) => Vec::new(),
            Self::Analysis(adder) => adder.analysis_dependencies(),
        }
    }

    pub fn is_analysis_backed(&self) -> bool {
        matches!(self, Self::Analysis(_))
    }
}

impl<F> From<Arc<dyn PlayerFeatureAdder<F> + Send + Sync>> for NDArrayPlayerFeatureAdder<F> {
    fn from(adder: Arc<dyn PlayerFeatureAdder<F> + Send + Sync>) -> Self {
        Self::plain(adder)
    }
}

impl<F> From<Arc<dyn AnalysisPlayerFeatureAdder<F> + Send + Sync>>
    for NDArrayPlayerFeatureAdder<F>
{
    fn from(adder: Arc<dyn AnalysisPlayerFeatureAdder<F> + Send + Sync>) -> Self {
        Self::analysis(adder)
    }
}

pub type NDArrayPlayerFeatureAdders<F> = Vec<NDArrayPlayerFeatureAdder<F>>;

/// Fixed-width per-player feature extractor with compile-time column count validation.
pub trait LengthCheckedPlayerFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn get_features(
        &self,
        player_id: &PlayerId,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<[F; N]>;
}

/// Implements [`PlayerFeatureAdder`] for a type that satisfies [`LengthCheckedPlayerFeatureAdder`].
#[macro_export]
macro_rules! impl_player_feature_adder {
    ($struct_name:ident) => {
        impl<F: TryFrom<f32>> PlayerFeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn add_features(
                &self,
                player_id: &PlayerId,
                processor: &dyn ProcessorView,
                frame: &boxcars::Frame,
                frame_count: usize,
                current_time: f32,
                vector: &mut Vec<F>,
            ) -> SubtrActorResult<()> {
                Ok(vector.extend(self.get_features(
                    player_id,
                    processor,
                    frame,
                    frame_count,
                    current_time,
                )?))
            }

            fn get_column_headers(&self) -> &[&str] {
                self.get_column_headers_array()
            }
        }
    };
}

impl<G, F, const N: usize> FeatureAdder<F> for (G, &[&str; N])
where
    G: Fn(&dyn ProcessorView, &boxcars::Frame, usize, f32) -> SubtrActorResult<[F; N]>,
{
    fn add_features(
        &self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()> {
        vector.extend(self.0(processor, frame, frame_count, current_time)?);
        Ok(())
    }

    fn get_column_headers(&self) -> &[&str] {
        self.1.as_slice()
    }
}

impl<G, F, const N: usize> PlayerFeatureAdder<F> for (G, &[&str; N])
where
    G: Fn(&PlayerId, &dyn ProcessorView, &boxcars::Frame, usize, f32) -> SubtrActorResult<[F; N]>,
{
    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()> {
        vector.extend(self.0(
            player_id,
            processor,
            frame,
            frame_count,
            current_time,
        )?);
        Ok(())
    }

    fn get_column_headers(&self) -> &[&str] {
        self.1.as_slice()
    }
}

/// Declares a new global feature-adder type and wires it into the ndarray traits.
#[macro_export]
macro_rules! build_global_feature_adder {
    ($struct_name:ident, $prop_getter:expr_2021, $( $column_names:expr_2021 ),* $(,)?) => {

        #[derive(derive_new::new)]
        pub struct $struct_name<F> {
            _zero: std::marker::PhantomData<F>,
        }

        impl<F: Sync + Send + TryFrom<f32> + 'static> $struct_name<F> where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            pub fn arc_new() -> std::sync::Arc<dyn FeatureAdder<F> + Send + Sync + 'static> {
                std::sync::Arc::new(Self::new())
            }
        }

        global_feature_adder!(
            $struct_name,
            $prop_getter,
            $( $column_names ),*
        );
    }
}

/// Implements the ndarray feature-adder traits for an existing global feature type.
#[macro_export]
macro_rules! global_feature_adder {
    ($struct_name:ident, $prop_getter:expr_2021, $( $column_names:expr_2021 ),* $(,)?) => {
        macro_rules! _global_feature_adder {
            ($count:ident) => {
                impl<F: TryFrom<f32>> LengthCheckedFeatureAdder<F, $count> for $struct_name<F>
                where
                    <F as TryFrom<f32>>::Error: std::fmt::Debug,
                {
                    fn get_column_headers_array(&self) -> &[&str; $count] {
                        &[$( $column_names ),*]
                    }

                    fn get_features(
                        &self,
                        processor: &dyn ProcessorView,
                        frame: &boxcars::Frame,
                        frame_count: usize,
                        current_time: f32,
                    ) -> SubtrActorResult<[F; $count]> {
                        $prop_getter(self, processor, frame, frame_count, current_time)
                    }
                }

                impl_feature_adder!($struct_name);
            };
        }
        paste::paste! {
            const [<$struct_name:snake:upper _LENGTH>]: usize = [$($column_names),*].len();
            _global_feature_adder!([<$struct_name:snake:upper _LENGTH>]);
        }
    }
}

/// Declares a new analysis-backed global feature-adder type.
#[macro_export]
macro_rules! build_analysis_global_feature_adder {
    ($struct_name:ident, $dependency_getter:expr_2021, $prop_getter:expr_2021, $( $column_names:expr_2021 ),* $(,)?) => {

        #[derive(derive_new::new)]
        pub struct $struct_name<F> {
            _zero: std::marker::PhantomData<F>,
        }

        impl<F: Sync + Send + TryFrom<f32> + 'static> $struct_name<F> where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            pub fn arc_new() -> std::sync::Arc<dyn AnalysisFeatureAdder<F> + Send + Sync + 'static> {
                std::sync::Arc::new(Self::new())
            }
        }

        analysis_global_feature_adder!(
            $struct_name,
            $dependency_getter,
            $prop_getter,
            $( $column_names ),*
        );
    }
}

/// Implements the ndarray traits for an existing analysis-backed global feature type.
#[macro_export]
macro_rules! analysis_global_feature_adder {
    ($struct_name:ident, $dependency_getter:expr_2021, $prop_getter:expr_2021, $( $column_names:expr_2021 ),* $(,)?) => {
        macro_rules! _analysis_global_feature_adder {
            ($count:ident) => {
                impl<F: TryFrom<f32>> LengthCheckedAnalysisFeatureAdder<F, $count> for $struct_name<F>
                where
                    <F as TryFrom<f32>>::Error: std::fmt::Debug,
                {
                    fn get_column_headers_array(&self) -> &[&str; $count] {
                        &[$( $column_names ),*]
                    }

                    fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
                        $dependency_getter(self)
                    }

                    fn get_features(
                        &self,
                        context: &AnalysisFeatureContext<'_>,
                        processor: &dyn ProcessorView,
                        frame: &boxcars::Frame,
                        frame_count: usize,
                        current_time: f32,
                    ) -> SubtrActorResult<[F; $count]> {
                        $prop_getter(self, context, processor, frame, frame_count, current_time)
                    }
                }

                impl_analysis_feature_adder!($struct_name);
            };
        }
        paste::paste! {
            const [<$struct_name:snake:upper _LENGTH>]: usize = [$($column_names),*].len();
            _analysis_global_feature_adder!([<$struct_name:snake:upper _LENGTH>]);
        }
    }
}

/// Declares a new per-player feature-adder type and wires it into the ndarray traits.
#[macro_export]
macro_rules! build_player_feature_adder {
    ($struct_name:ident, $prop_getter:expr_2021, $( $column_names:expr_2021 ),* $(,)?) => {
        #[derive(derive_new::new)]
        pub struct $struct_name<F> {
            _zero: std::marker::PhantomData<F>,
        }

        impl<F: Sync + Send + TryFrom<f32> + 'static> $struct_name<F> where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            pub fn arc_new() -> std::sync::Arc<dyn PlayerFeatureAdder<F> + Send + Sync + 'static> {
                std::sync::Arc::new(Self::new())
            }
        }

        player_feature_adder!(
            $struct_name,
            $prop_getter,
            $( $column_names ),*
        );
    }
}

/// Implements the ndarray feature-adder traits for an existing per-player feature type.
#[macro_export]
macro_rules! player_feature_adder {
    ($struct_name:ident, $prop_getter:expr_2021, $( $column_names:expr_2021 ),* $(,)?) => {
        macro_rules! _player_feature_adder {
            ($count:ident) => {
                impl<F: TryFrom<f32>> LengthCheckedPlayerFeatureAdder<F, $count> for $struct_name<F>
                where
                    <F as TryFrom<f32>>::Error: std::fmt::Debug,
                {
                    fn get_column_headers_array(&self) -> &[&str; $count] {
                        &[$( $column_names ),*]
                    }

                    fn get_features(
                        &self,
                        player_id: &PlayerId,
                        processor: &dyn ProcessorView,
                        frame: &boxcars::Frame,
                        frame_count: usize,
                        current_time: f32,
                    ) -> SubtrActorResult<[F; $count]> {
                        $prop_getter(self, player_id, processor, frame, frame_count, current_time)
                    }
                }

                impl_player_feature_adder!($struct_name);
            };
        }
        paste::paste! {
            const [<$struct_name:snake:upper _LENGTH>]: usize = [$($column_names),*].len();
            _player_feature_adder!([<$struct_name:snake:upper _LENGTH>]);
        }
    }
}

/// Declares a new analysis-backed per-player feature-adder type.
#[macro_export]
macro_rules! build_analysis_player_feature_adder {
    ($struct_name:ident, $dependency_getter:expr_2021, $prop_getter:expr_2021, $( $column_names:expr_2021 ),* $(,)?) => {
        #[derive(derive_new::new)]
        pub struct $struct_name<F> {
            _zero: std::marker::PhantomData<F>,
        }

        impl<F: Sync + Send + TryFrom<f32> + 'static> $struct_name<F> where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            pub fn arc_new() -> std::sync::Arc<dyn AnalysisPlayerFeatureAdder<F> + Send + Sync + 'static> {
                std::sync::Arc::new(Self::new())
            }
        }

        analysis_player_feature_adder!(
            $struct_name,
            $dependency_getter,
            $prop_getter,
            $( $column_names ),*
        );
    }
}

/// Implements the ndarray traits for an existing analysis-backed per-player feature type.
#[macro_export]
macro_rules! analysis_player_feature_adder {
    ($struct_name:ident, $dependency_getter:expr_2021, $prop_getter:expr_2021, $( $column_names:expr_2021 ),* $(,)?) => {
        macro_rules! _analysis_player_feature_adder {
            ($count:ident) => {
                impl<F: TryFrom<f32>> LengthCheckedAnalysisPlayerFeatureAdder<F, $count> for $struct_name<F>
                where
                    <F as TryFrom<f32>>::Error: std::fmt::Debug,
                {
                    fn get_column_headers_array(&self) -> &[&str; $count] {
                        &[$( $column_names ),*]
                    }

                    fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
                        $dependency_getter(self)
                    }

                    fn get_features(
                        &self,
                        context: &AnalysisFeatureContext<'_>,
                        player_id: &PlayerId,
                        processor: &dyn ProcessorView,
                        frame: &boxcars::Frame,
                        frame_count: usize,
                        current_time: f32,
                    ) -> SubtrActorResult<[F; $count]> {
                        $prop_getter(self, context, player_id, processor, frame, frame_count, current_time)
                    }
                }

                impl_analysis_player_feature_adder!($struct_name);
            };
        }
        paste::paste! {
            const [<$struct_name:snake:upper _LENGTH>]: usize = [$($column_names),*].len();
            _analysis_player_feature_adder!([<$struct_name:snake:upper _LENGTH>]);
        }
    }
}

/// Maps arbitrary conversion failures into a generic float-conversion error.
pub fn convert_float_conversion_error<T>(_: T) -> SubtrActorError {
    SubtrActorError::new(SubtrActorErrorVariant::FloatConversionError)
}

/// Converts a fixed list of values with a caller-supplied error mapper.
#[macro_export]
macro_rules! convert_all {
    ($err:expr_2021, $( $item:expr_2021 ),* $(,)?) => {{
		Ok([
			$( $item.try_into().map_err($err)? ),*
		])
	}};
}

/// Converts a fixed list of float-like values using [`convert_float_conversion_error`].
#[macro_export]
macro_rules! convert_all_floats {
    ($( $item:expr_2021 ),* $(,)?) => {{
        convert_all!(convert_float_conversion_error, $( $item ),*)
    }};
}
