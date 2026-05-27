use std::sync::Arc;

use crate::*;

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

/// Heterogeneous collection of per-player feature adders.
pub type PlayerFeatureAdders<F> = Vec<Arc<dyn PlayerFeatureAdder<F> + Send + Sync>>;

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
