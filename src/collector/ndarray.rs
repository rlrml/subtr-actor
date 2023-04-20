use ::ndarray;
use boxcars;
use serde::Serialize;

use crate::*;

macro_rules! string_error {
	($format:expr) => {
		string_error!($format,)
	};
    ($format:expr, $( $arg:expr ),* $(,)?) => {
        |e| format!($format, $( $arg, )* e)
    };
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NDArrayColumnHeaders {
    pub global_headers: Vec<String>,
    pub player_headers: Vec<String>,
    pub player_infos_and_indices: Vec<(PlayerInfo, usize)>,
}

impl NDArrayColumnHeaders {
    fn new(
        global_headers: Vec<String>,
        player_headers: Vec<String>,
        player_infos: Vec<PlayerInfo>,
    ) -> Self {
        let global_header_count = global_headers.len();
        let player_header_count = player_headers.len();
        let player_infos_and_indices = player_infos
            .into_iter()
            .enumerate()
            .map(|(player_index, player_info)| {
                (
                    player_info,
                    player_index * player_header_count + global_header_count,
                )
            })
            .collect();
        Self {
            global_headers,
            player_headers,
            player_infos_and_indices,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayMetaWithHeaders {
    pub replay_meta: ReplayMeta,
    pub column_headers: NDArrayColumnHeaders,
}

impl From<&NDArrayColumnHeaders> for Vec<String> {
    fn from(column_headers: &NDArrayColumnHeaders) -> Self {
        column_headers
            .global_headers
            .iter()
            .cloned()
            .chain(
                column_headers
                    .player_infos_and_indices
                    .iter()
                    .enumerate()
                    .flat_map(move |(player_index, (info, _))| {
                        column_headers.player_headers.iter().map(move |header| {
                            format!("Player {} ({}) - {}", player_index, info.name, header)
                        })
                    }),
            )
            .collect()
    }
}

pub struct NDArrayCollector<F> {
    feature_adders: Vec<Box<dyn FeatureAdder<F>>>,
    player_feature_adders: Vec<Box<dyn PlayerFeatureAdder<F>>>,
    data: Vec<F>,
    replay_meta: Option<ReplayMeta>,
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
            replay_meta: None,
            frames_added: 0,
        }
    }

    fn try_get_frame_feature_count(&self) -> Result<usize, String> {
        let player_count = self
            .replay_meta
            .as_ref()
            .ok_or("Replay meta not yet set")?
            .player_count();
        let global_feature_count: usize = self
            .feature_adders
            .iter()
            .map(|fa| fa.features_added())
            .sum();
        let player_feature_count: usize = self
            .player_feature_adders
            .iter() // iterate
            .map(|pfa| pfa.features_added() * player_count)
            .sum();
        Ok(global_feature_count + player_feature_count)
    }

    pub fn get_column_headers(&self) -> Result<NDArrayColumnHeaders, String> {
        let replay_meta = self.replay_meta.as_ref().ok_or("Replay meta not yet set")?;
        let player_infos = replay_meta.player_order().cloned().collect();
        let global_headers = self
            .feature_adders
            .iter()
            .flat_map(move |fa| {
                fa.get_column_headers()
                    .iter()
                    .map(move |column_name| format!("{}", column_name))
            })
            .collect();
        let player_headers = self
            .player_feature_adders
            .iter()
            .flat_map(move |pfa| {
                pfa.get_column_headers()
                    .iter()
                    .map(move |base_name| format!("{}", base_name))
            })
            .collect();
        Ok(NDArrayColumnHeaders::new(
            global_headers,
            player_headers,
            player_infos,
        ))
    }

    pub fn get_ndarray(self) -> Result<ndarray::Array2<F>, String> {
        self.get_meta_and_ndarray().map(|a| a.1)
    }

    pub fn get_meta_and_ndarray(
        self,
    ) -> Result<(ReplayMetaWithHeaders, ndarray::Array2<F>), String> {
        let features_per_row = self.try_get_frame_feature_count()?;
        let expected_length = features_per_row * self.frames_added;
        let column_headers = self.get_column_headers()?;
        if self.data.len() != expected_length {
            Err(format!(
                "Unexpected vector length: actual: {}, expected: {}, features: {}, rows: {}",
                self.data.len(),
                expected_length,
                features_per_row,
                self.frames_added,
            ))
        } else {
            Ok((
                ReplayMetaWithHeaders {
                    replay_meta: self.replay_meta.ok_or("No replay meta")?,
                    column_headers,
                },
                ndarray::Array2::from_shape_vec((self.frames_added, features_per_row), self.data)
                    .map_err(string_error!("Error building array from vec {:?}",))?,
            ))
        }
    }

    pub fn process_and_get_meta_and_headers(
        &mut self,
        replay: &boxcars::Replay,
    ) -> ReplayProcessorResult<ReplayMetaWithHeaders> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process_long_enough_to_get_actor_ids()?;
        self.maybe_set_replay_meta(&processor)?;
        Ok(ReplayMetaWithHeaders {
            replay_meta: self.replay_meta.as_ref().ok_or("No replay meta")?.clone(),
            column_headers: self.get_column_headers()?,
        })
    }

    fn maybe_set_replay_meta(&mut self, processor: &ReplayProcessor) -> ReplayProcessorResult<()> {
        if let None = self.replay_meta {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        Ok(())
    }
}

impl<F> Collector for NDArrayCollector<F> {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
    ) -> ReplayProcessorResult<()> {
        self.maybe_set_replay_meta(processor)?;
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
            vec![build_ball_rigidbody_feature_adder()],
            vec![
                build_player_rigidbody_feature_adder(),
                build_player_boost_feature_adder(),
                build_player_jump_feature_adder(),
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
            vec![build_ball_rigidbody_feature_adder()],
            vec![
                build_player_rigidbody_feature_adder(),
                build_player_boost_feature_adder(),
            ],
        )
    }
}

pub trait FeatureAdder<F> {
    fn features_added(&self) -> usize {
        self.get_column_headers().len()
    }

    fn get_column_headers(&self) -> &[&str];

    fn add_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        vector: &mut Vec<F>,
    ) -> ReplayProcessorResult<()>;
}

pub trait PlayerFeatureAdder<F> {
    fn features_added(&self) -> usize {
        self.get_column_headers().len()
    }

    fn get_column_headers(&self) -> &[&str];

    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        vector: &mut Vec<F>,
    ) -> ReplayProcessorResult<()>;
}

impl<G, F, const N: usize> FeatureAdder<F> for (G, &[&str; N])
where
    G: Fn(&ReplayProcessor, &boxcars::Frame, usize) -> Result<[F; N], String>,
{
    fn add_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        vector: &mut Vec<F>,
    ) -> ReplayProcessorResult<()> {
        Ok(vector.extend(self.0(processor, frame, frame_count)?))
    }

    fn get_column_headers(&self) -> &[&str] {
        &self.1.as_slice()
    }
}

impl<G, F, const N: usize> PlayerFeatureAdder<F> for (G, &[&str; N])
where
    G: Fn(&PlayerId, &ReplayProcessor, &boxcars::Frame, usize) -> Result<[F; N], String>,
{
    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        vector: &mut Vec<F>,
    ) -> ReplayProcessorResult<()> {
        Ok(vector.extend(self.0(player_id, processor, frame, frame_count)?))
    }

    fn get_column_headers(&self) -> &[&str] {
        &self.1.as_slice()
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
    // important

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

pub static BALL_RIGID_BODY_COLUMN_NAMES: [&str; 12] = [
    "Ball - pos x",
    "Ball - pos y",
    "Ball - pos z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - linear velocity x",
    "Ball - linear velocity y",
    "Ball - linear velocity z",
    "Ball - angular velocity x",
    "Ball - angular velocity y",
    "Ball - angular velocity z",
];

pub fn build_ball_rigidbody_feature_adder<F: TryFrom<f32> + 'static>() -> Box<dyn FeatureAdder<F>>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    Box::new((&get_ball_rb_properties::<F>, &BALL_RIGID_BODY_COLUMN_NAMES))
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

pub static PLAYER_RIGID_BODY_COLUMN_NAMES: [&str; 12] = [
    "pos x",
    "pos y",
    "pos z",
    "rotation x",
    "rotation y",
    "rotation z",
    "linear velocity x",
    "linear velocity y",
    "linear velocity z",
    "angular velocity x",
    "angular velocity y",
    "angular velocity z",
];

pub fn build_player_rigidbody_feature_adder<F: TryFrom<f32> + 'static>(
) -> Box<dyn PlayerFeatureAdder<F>>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    Box::new((
        &get_player_rb_properties::<F>,
        &PLAYER_RIGID_BODY_COLUMN_NAMES,
    ))
}

pub static PLAYER_BOOST_COLUMN_NAMES: [&str; 1] = ["boost level"];

pub fn build_player_boost_feature_adder<F: TryFrom<f32> + 'static>(
) -> Box<dyn PlayerFeatureAdder<F>>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    Box::new((&get_player_boost_level::<F>, &PLAYER_BOOST_COLUMN_NAMES))
}

pub static PLAYER_JUMP_COLUMN_NAMES: [&str; 3] =
    ["dodge active", "jump active", "double jump active"];

pub fn build_player_jump_feature_adder<F: TryFrom<f32> + 'static>() -> Box<dyn PlayerFeatureAdder<F>>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    Box::new((&get_jump_availabilities, &PLAYER_JUMP_COLUMN_NAMES))
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
