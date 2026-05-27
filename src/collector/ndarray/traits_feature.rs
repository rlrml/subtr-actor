use std::sync::Arc;

use crate::*;

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

/// Heterogeneous collection of frame-level feature adders.
pub type FeatureAdders<F> = Vec<Arc<dyn FeatureAdder<F> + Send + Sync>>;

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
