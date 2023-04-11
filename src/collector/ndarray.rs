use ::ndarray;
use boxcars;

use crate::*;

macro_rules! string_error {
	($format:expr) => {
		string_error!($format,)
	};
    ($format:expr, $( $arg:expr ),* $(,)?) => {
        |e| format!($format, $( $arg, )* e)
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
    data: Vec<F>,
    player_count: Option<usize>,
    frames_added: usize,
}

impl<F> NDArrayCollector<F> {
    pub fn new(
        feature_adders: Vec<Box<dyn FeatureAdder<F>>>,
        player_feature_adders: Vec<Box<dyn PlayerFeatureAdder<F>>>,
    ) -> Self {
        Self {
            feature_adders,
            player_feature_adders,
            data: Vec::new(),
            player_count: None,
            frames_added: 0,
        }
    }

    fn try_get_frame_feature_count(&self) -> Result<usize, String> {
        let player_count = self.player_count.ok_or("Player count not yet set")?;
        let global_feature_count: usize = self
            .feature_adders
            .iter()
            .map(|fa| fa.features_added())
            .sum();
        let player_feature_count: usize = self
            .player_feature_adders
            .iter()
            .map(|pfa| pfa.features_added() * player_count)
            .sum();
        Ok(global_feature_count + player_feature_count)
    }

    pub fn get_ndarray(self) -> Result<ndarray::Array2<F>, String> {
        let features_per_row = self.try_get_frame_feature_count()?;
        let expected_length = features_per_row * self.frames_added;
        if self.data.len() != expected_length {
            Err(format!(
                "Unexpected vector length: actual: {}, expected: {}, features: {}, rows: {}",
                self.data.len(),
                expected_length,
                features_per_row,
                self.frames_added,
            ))
        } else {
            Ok(
                ndarray::Array2::from_shape_vec((self.frames_added, features_per_row), self.data)
                    .map_err(string_error!("Error building array from vec {:?}",))?,
            )
        }
    }
}

impl<F> Collector for NDArrayCollector<F> {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
    ) -> Result<(), String> {
        if let None = self.player_count {
            self.player_count = Some(processor.player_count());
        }
        if !require_ball_rigid_body_exists(processor, frame, frame_number)? {
            return Ok(());
        }
        for feature_adder in self.feature_adders.iter() {
            feature_adder.add_features(processor, frame, frame_number, &mut self.data)?;
        }
        for player_id in processor.iter_player_ids_in_order() {
            for player_feature_adder in self.player_feature_adders.iter() {
                player_feature_adder.add_features(
                    player_id,
                    processor,
                    frame,
                    frame_number,
                    &mut self.data,
                )?;
            }
        }
        self.frames_added += 1;
        Ok(())
    }
}

impl<F: TryFrom<f32> + 'static> NDArrayCollector<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    pub fn with_jump_availabilities() -> Self {
        NDArrayCollector::new(
            vec![Box::new(&get_ball_rb_properties)],
            vec![
                Box::new(&get_player_rb_properties),
                Box::new(&get_player_boost_level),
                Box::new(&get_jump_availabilities),
            ],
        )
    }
}

impl<F: TryFrom<f32> + 'static> Default for NDArrayCollector<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    fn default() -> Self {
        NDArrayCollector::new(
            vec![Box::new(&get_ball_rb_properties)],
            vec![
                Box::new(&get_player_rb_properties),
                Box::new(&get_player_boost_level),
            ],
        )
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
		Ok([
			$( $item.try_into().map_err($err)? ),*
		])
	}};
}

fn or_zero_boxcars_3f() -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

type RigidBodyArrayResult<F> = Result<[F; 12], String>;

pub fn get_rigid_body_properties<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> RigidBodyArrayResult<F>
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
    convert_all!(
        convert,
        location.x,
        location.y,
        location.z,
        rx,
        ry,
        rz,
        linear_velocity.x,
        linear_velocity.y,
        linear_velocity.z,
        angular_velocity.x,
        angular_velocity.y,
        angular_velocity.z,
    )
}

fn default_rb_state<F: TryFrom<f32>>() -> RigidBodyArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all!(
        string_error!("{:?}"),
        // We use huge values for location instead of 0s so that hopefully any
        // model built on this data can understand that the player is not
        // actually on the field.
        200000.0,
        200000.0,
        200000.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    )
}

pub fn get_ball_rb_properties<F: TryFrom<f32>>(
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _: usize,
) -> RigidBodyArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    get_rigid_body_properties(processor.get_ball_rigid_body()?)
}

pub fn get_player_rb_properties<F: TryFrom<f32>>(
    player_id: &PlayerId,
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _frame_number: usize,
) -> RigidBodyArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    if let Ok(rb) = processor.get_player_rigid_body(player_id) {
        get_rigid_body_properties(rb)
    } else {
        default_rb_state()
    }
}

pub fn get_player_boost_level<F: TryFrom<f32>>(
    player_id: &PlayerId,
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _frame_number: usize,
) -> Result<[F; 1], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all!(
        string_error!("{:?}"),
        processor
            .get_player_boost_level(player_id)
            .cloned()
            .unwrap_or(0.0)
    )
}

pub fn get_jump_availabilities<F: TryFrom<f32>>(
    player_id: &PlayerId,
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _frame_number: usize,
) -> Result<[F; 3], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let get_f32 =
        |b| -> Result<f32, String> { TryFrom::try_from(b % 2).map_err(string_error!("{:?}")) };
    convert_all!(
        string_error!("{:?}"),
        processor
            .get_dodge_active(player_id)
            .and_then(get_f32)
            .unwrap_or(0.0),
        processor
            .get_jump_active(player_id)
            .and_then(get_f32)
            .unwrap_or(0.0),
        processor
            .get_double_jump_active(player_id)
            .and_then(get_f32)
            .unwrap_or(0.0),
    )
}
