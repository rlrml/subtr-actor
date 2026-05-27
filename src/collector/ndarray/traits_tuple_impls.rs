use super::{FeatureAdder, PlayerFeatureAdder};
use crate::*;

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
