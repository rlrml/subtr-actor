use boxcars;
use ndarray;

use crate::processor::*;

macro_rules! string_error {
    ($format:expr) => {
        |e| format!($format, e)
    };
}

trait ArrayLen {
    const SIZE: usize;
}

impl<'a, T, const N: usize> ArrayLen for &'a [T; N] {
    const SIZE: usize = N;
}

pub struct NDArrayCollector<F> {
    feature_adders: Vec<Box<dyn FeatureAdder<F>>>,
    player_feature_adders: Vec<Box<dyn PlayerFeatureAdder<F>>>,
}

impl<F> NDArrayCollector<F> {
    pub fn new(
        feature_adders: Vec<Box<dyn FeatureAdder<F>>>,
        player_feature_adders: Vec<Box<dyn PlayerFeatureAdder<F>>>,
    ) -> Self {
        Self {
            feature_adders,
            player_feature_adders,
        }
    }

    fn get_frame_feature_count(&self, processor: &ReplayProcessor) -> usize {
        let global_feature_count: usize = self
            .feature_adders
            .iter()
            .map(|fa| fa.features_added())
            .sum();
        let player_count = processor.player_count();
        let player_feature_count: usize = self
            .player_feature_adders
            .iter()
            .map(|pfa| pfa.features_added() * player_count)
            .sum();
        global_feature_count + player_feature_count
    }

    pub fn build_ndarray(&self, replay: &boxcars::Replay) -> Result<ndarray::Array2<F>, String> {
        let mut processor = ReplayProcessor::new(replay);
        self.build_ndarray_with_processor(&mut processor)
    }

    pub fn build_ndarray_with_processor(
        &self,
        processor: &mut ReplayProcessor,
    ) -> Result<ndarray::Array2<F>, String> {
        let mut vector = Vec::new();
        let features_per_row = self.get_frame_feature_count(processor);
        let mut frames_added = 0;
        processor.process(&mut |p, f, n| {
            self.extend_vector_for_frame(p, f, n, &mut vector)?;
            frames_added += 1;
            Ok(())
        })?;
        let expected_length = features_per_row * frames_added;
        if vector.len() != expected_length {
            Err(format!(
                "Unexpected vector length: actual: {}, expected: {}, features: {}, rows: {}",
                vector.len(),
                expected_length,
                features_per_row,
                frames_added,
            ))
        } else {
            Ok(
                ndarray::Array2::from_shape_vec((frames_added, features_per_row), vector)
                    .map_err(string_error!("Error building array from vec {:?}"))?,
            )
        }
    }

    fn extend_vector_for_frame(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        vector: &mut Vec<F>,
    ) -> Result<(), String> {
        for feature_adder in self.feature_adders.iter() {
            feature_adder.add_features(processor, frame, frame_number, vector)?;
        }
        for player_id in processor.iter_player_ids_in_order() {
            for player_feature_adder in self.player_feature_adders.iter() {
                player_feature_adder.add_features(
                    player_id,
                    processor,
                    frame,
                    frame_number,
                    vector,
                )?;
            }
        }
        Ok(())
    }
}

impl<F: TryFrom<f32> + 'static> NDArrayCollector<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    pub fn rb_properties_only() -> Self {
        NDArrayCollector {
            feature_adders: vec![Box::new(&get_ball_rb_properties)],
            player_feature_adders: vec![Box::new(&get_player_rb_properties)],
        }
    }
}

pub trait FeatureAdder<F> {
    fn features_added(&self) -> usize;
    fn add_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        vector: &mut Vec<F>,
    ) -> Result<(), String>;
}

pub trait PlayerFeatureAdder<F> {
    fn features_added(&self) -> usize;
    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        vector: &mut Vec<F>,
    ) -> Result<(), String>;
}

impl<G, F, const N: usize> FeatureAdder<F> for G
where
    G: Fn(&ReplayProcessor, &boxcars::Frame, usize) -> Result<[F; N], String>,
{
    fn features_added(&self) -> usize {
        N
    }

    fn add_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        vector: &mut Vec<F>,
    ) -> Result<(), String> {
        Ok(vector.extend(self(processor, frame, frame_count)?))
    }
}

impl<G, F, const N: usize> PlayerFeatureAdder<F> for G
where
    G: Fn(&PlayerId, &ReplayProcessor, &boxcars::Frame, usize) -> Result<[F; N], String>,
{
    fn features_added(&self) -> usize {
        N
    }

    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        vector: &mut Vec<F>,
    ) -> Result<(), String> {
        Ok(vector.extend(self(player_id, processor, frame, frame_count)?))
    }
}

macro_rules! convert_all {
    ($err:expr, $( $item:expr ),* $(,)?) => {{
		[
			$( $item.try_into().map_err($err)? ),*
		]
	}};
}

// Player feature adders

fn or_zero_boxcars_3f() -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

pub fn get_rigid_body_properties<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> Result<[F; 15], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let convert = string_error!("Error in rigid body float conversion {:?}");
    let linear_velocity = rigid_body
        .linear_velocity
        .unwrap_or_else(or_zero_boxcars_3f);
    let angular_velocity = rigid_body
        .angular_velocity
        .unwrap_or_else(or_zero_boxcars_3f);
    let rotation = rigid_body.rotation;
    let location = rigid_body.location;
    let (rx, ry, rz) =
        glam::quat(rotation.x, rotation.y, rotation.z, rotation.w).to_euler(glam::EulerRot::XYZ);
    Ok(convert_all!(
        convert,
        location.x,
        location.y,
        location.z,
        rotation.x,
        rotation.y,
        rotation.z,
        rx,
        ry,
        rz,
        linear_velocity.x,
        linear_velocity.y,
        linear_velocity.z,
        angular_velocity.x,
        angular_velocity.y,
        angular_velocity.z,
    ))
}

pub fn get_ball_rb_properties<F: TryFrom<f32>>(
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _: usize,
) -> Result<[F; 15], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    get_rigid_body_properties(processor.get_ball_rigid_body()?)
}

pub fn get_player_rb_properties<F: TryFrom<f32>>(
    player_id: &PlayerId,
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _: usize,
) -> Result<[F; 15], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    get_rigid_body_properties(processor.get_player_rigid_body(player_id)?)
}
