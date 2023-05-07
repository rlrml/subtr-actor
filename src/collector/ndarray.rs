use ::ndarray;
use boxcars;
use lazy_static::lazy_static;
use serde::Serialize;
use std::sync::Arc;

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
}

impl NDArrayColumnHeaders {
    pub fn new(global_headers: Vec<String>, player_headers: Vec<String>) -> Self {
        Self {
            global_headers,
            player_headers,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayMetaWithHeaders {
    pub replay_meta: ReplayMeta,
    pub column_headers: NDArrayColumnHeaders,
}

impl ReplayMetaWithHeaders {
    pub fn headers_vec(&self) -> Vec<String> {
        self.column_headers
            .global_headers
            .iter()
            .cloned()
            .chain(self.replay_meta.player_order().enumerate().flat_map(
                move |(player_index, info)| {
                    self.column_headers
                        .player_headers
                        .iter()
                        .map(move |header| {
                            format!("Player {} ({}) - {}", player_index, info.name, header)
                        })
                },
            ))
            .collect()
    }
}

pub struct NDArrayCollector<F> {
    feature_adders: Vec<Arc<dyn FeatureAdder<F> + Send + Sync>>,
    player_feature_adders: Vec<Arc<dyn PlayerFeatureAdder<F> + Send + Sync>>,
    data: Vec<F>,
    replay_meta: Option<ReplayMeta>,
    frames_added: usize,
}

impl<F> NDArrayCollector<F> {
    pub fn new(
        feature_adders: Vec<Arc<dyn FeatureAdder<F> + Send + Sync>>,
        player_feature_adders: Vec<Arc<dyn PlayerFeatureAdder<F> + Send + Sync>>,
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

    pub fn get_column_headers(&self) -> NDArrayColumnHeaders {
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
        NDArrayColumnHeaders::new(global_headers, player_headers)
    }

    pub fn get_ndarray(self) -> Result<ndarray::Array2<F>, String> {
        self.get_meta_and_ndarray().map(|a| a.1)
    }

    pub fn get_meta_and_ndarray(
        self,
    ) -> Result<(ReplayMetaWithHeaders, ndarray::Array2<F>), String> {
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
            let column_headers = self.get_column_headers();
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
            column_headers: self.get_column_headers(),
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

impl NDArrayCollector<f32> {
    pub fn from_strings(fa_names: &[&str], pfa_names: &[&str]) -> Result<Self, String> {
        let feature_adders: Vec<Arc<dyn FeatureAdder<f32> + Send + Sync>> = fa_names
            .iter()
            .map(|name| {
                Ok(NAME_TO_GLOBAL_FEATURE_ADDER
                    .get(name)
                    .ok_or_else(|| format!("{:?} was not a recognized feature adder", name))?
                    .clone())
            })
            .collect::<Result<Vec<_>, String>>()?;
        let player_feature_adders: Vec<Arc<dyn PlayerFeatureAdder<f32> + Send + Sync>> = pfa_names
            .iter()
            .map(|name| {
                Ok(NAME_TO_PLAYER_FEATURE_ADDER
                    .get(name)
                    .ok_or_else(|| format!("{:?} was not a recognized feature adder", name))?
                    .clone())
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self::new(feature_adders, player_feature_adders))
    }
}

impl<F: TryFrom<f32> + Send + Sync + 'static> Default for NDArrayCollector<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    fn default() -> Self {
        NDArrayCollector::new(
            vec![BallRigidBody::arc_new()],
            vec![
                PlayerRigidBody::arc_new(),
                PlayerBoost::arc_new(),
                PlayerAnyJump::arc_new(),
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

pub trait LengthCheckedFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn get_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
    ) -> Result<[F; N], String>;
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

pub trait LengthCheckedPlayerFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn get_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
    ) -> Result<[F; N], String>;
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

pub fn get_rigid_body_properties_no_velocities<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> Result<[F; 6], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let convert = string_error!("Error in rigid body float conversion {:?}");
    let rotation = rigid_body.rotation;
    let location = rigid_body.location;
    let (rx, ry, rz) =
        glam::quat(rotation.x, rotation.y, rotation.z, rotation.w).to_euler(glam::EulerRot::XYZ);
    convert_all!(convert, location.x, location.y, location.z, rx, ry, rz)
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

fn default_rb_state_no_velocities<F: TryFrom<f32>>() -> Result<[F; 6], String>
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
    )
}

macro_rules! count_exprs {
    () => {0usize};
    ($val:expr $(, $vals:expr)*) => {1usize + count_exprs!($($vals),*)};
}

macro_rules! global_feature_adder {
    ($struct_name:ident, $prop_getter:ident, $( $column_names:expr ),* $(,)?) => {
        _global_feature_adder!(
            {count_exprs!($( $column_names ),*)},
            $struct_name,
            $prop_getter,
            $( $column_names ),*
        );
    }
}

macro_rules! _global_feature_adder {
    ($count:expr, $struct_name:ident, $prop_getter:ident, $( $column_names:expr ),* $(,)?) => {
        pub struct $struct_name<F> {
            _zero: std::marker::PhantomData<F>,
        }

        impl<F: Sync + Send + TryFrom<f32> + 'static> $struct_name<F> where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            pub fn new() -> Self {
                Self {
                    _zero: std::marker::PhantomData
                }
            }

            pub fn arc_new() -> Arc<dyn FeatureAdder<F> + Send + Sync + 'static> {
                Arc::new(Self::new())
            }
        }

        paste::paste! {
            pub static [<$struct_name:snake:upper _COLUMN_NAMES>]: [&str; count_exprs!($( $column_names ),*)] = [
                $( $column_names ),*
            ];
        }

        impl<F: TryFrom<f32>> LengthCheckedFeatureAdder<F, $count> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn get_column_headers_array(&self) -> &[&str; $count] {
                &paste::paste!{[<$struct_name:snake:upper _COLUMN_NAMES>]}
            }

            fn get_features(
                &self,
                processor: &ReplayProcessor,
                frame: &boxcars::Frame,
                frame_count: usize,
            ) -> Result<[F; $count], String> {
                $prop_getter(processor, frame, frame_count)
            }
        }

        impl<F: TryFrom<f32>> FeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug, {
            fn add_features(
                &self,
                processor: &ReplayProcessor,
                frame: &boxcars::Frame,
                frame_count: usize,
                vector: &mut Vec<F>,
            ) -> ReplayProcessorResult<()> {
                Ok(vector.extend(self.get_features(processor, frame, frame_count)?))
            }

            fn get_column_headers(&self) -> &[&str] {
                self.get_column_headers_array()
            }
        }
    };
}

macro_rules! player_feature_adder {
    ($struct_name:ident, $prop_getter:ident, $( $column_names:expr ),* $(,)?) => {
        _player_feature_adder!(
            {count_exprs!($( $column_names ),*)},
            $struct_name,
            $prop_getter,
            $( $column_names ),*
        );
    }
}

macro_rules! _player_feature_adder {
    ($count:expr, $struct_name:ident, $prop_getter:ident, $( $column_names:expr ),* $(,)?) => {
        pub struct $struct_name<F> {
            _zero: std::marker::PhantomData<F>,
        }

        impl<F: Sync + Send + TryFrom<f32> + 'static> $struct_name<F> where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            pub fn new() -> Self {
                Self {
                    _zero: std::marker::PhantomData
                }
            }

            pub fn arc_new() -> Arc<dyn PlayerFeatureAdder<F> + Send + Sync + 'static> {
                Arc::new(Self::new())
            }
        }

        paste::paste! {
            pub static  [<$struct_name:snake:upper _COLUMN_NAMES>] : [&str; count_exprs!($( $column_names ),*)] = [
                $( $column_names ),*
            ];
        }

        impl<F: TryFrom<f32>> LengthCheckedPlayerFeatureAdder<F, $count> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn get_column_headers_array(&self) -> &[&str; $count] {
                &paste::paste!{[<$struct_name:snake:upper _COLUMN_NAMES>]}
            }

            fn get_features(
                &self,
                player_id: &PlayerId,
                processor: &ReplayProcessor,
                frame: &boxcars::Frame,
                frame_count: usize,
            ) -> Result<[F; $count], String> {
                $prop_getter(player_id, processor, frame, frame_count)
            }
        }

        impl<F: TryFrom<f32>> PlayerFeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn add_features(
                &self,
                player_id: &PlayerId,
                processor: &ReplayProcessor,
                frame: &boxcars::Frame,
                frame_count: usize,
                vector: &mut Vec<F>,
            ) -> ReplayProcessorResult<()> {
                Ok(vector.extend(self.get_features(player_id, processor, frame, frame_count)?))
            }

            fn get_column_headers(&self) -> &[&str] {
                self.get_column_headers_array()
            }
        }
    };
}

global_feature_adder!(SecondsRemaining, get_seconds_remaining, "seconds remaining");

pub fn get_seconds_remaining<F: TryFrom<f32>>(
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _: usize,
) -> Result<[F; 1], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all!(
        string_error!("{:?}"),
        processor.get_seconds_remaining()?.clone() as f32
    )
}

global_feature_adder!(
    BallRigidBody,
    get_ball_rb_properties,
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - linear velocity x",
    "Ball - linear velocity y",
    "Ball - linear velocity z",
    "Ball - angular velocity x",
    "Ball - angular velocity y",
    "Ball - angular velocity z",
);

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

global_feature_adder!(
    BallRigidBodyNoVelocities,
    get_ball_rb_properties_no_velocities,
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
);

pub fn get_ball_rb_properties_no_velocities<F: TryFrom<f32>>(
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _: usize,
) -> Result<[F; 6], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    get_rigid_body_properties_no_velocities(processor.get_ball_rigid_body()?)
}

player_feature_adder!(
    PlayerRigidBody,
    get_player_rb_properties,
    "position x",
    "position y",
    "position z",
    "rotation x",
    "rotation y",
    "rotation z",
    "linear velocity x",
    "linear velocity y",
    "linear velocity z",
    "angular velocity x",
    "angular velocity y",
    "angular velocity z",
);

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

player_feature_adder!(
    PlayerRigidBodyNoVelocities,
    get_player_rb_properties_no_velocities,
    "position x",
    "position y",
    "position z",
    "rotation x",
    "rotation y",
    "rotation z",
);

pub fn get_player_rb_properties_no_velocities<F: TryFrom<f32>>(
    player_id: &PlayerId,
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _: usize,
) -> Result<[F; 6], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    if let Ok(rb) = processor.get_player_rigid_body(player_id) {
        get_rigid_body_properties_no_velocities(rb)
    } else {
        default_rb_state_no_velocities()
    }
}

player_feature_adder!(PlayerBoost, get_player_boost_level, "boost level");

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

player_feature_adder!(
    PlayerJump,
    get_jump_activities,
    "dodge active",
    "jump active",
    "double jump active"
);

pub fn get_f32(v: u8) -> Result<f32, String> {
    TryFrom::try_from(v % 2).map_err(string_error!("{:?}"))
}

pub fn get_jump_activities<F: TryFrom<f32>>(
    player_id: &PlayerId,
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _frame_number: usize,
) -> Result<[F; 3], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
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

player_feature_adder!(PlayerAnyJump, get_any_jump_active, "any_jump_active");

pub fn get_any_jump_active<F: TryFrom<f32>>(
    player_id: &PlayerId,
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _frame_number: usize,
) -> Result<[F; 1], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let dodge_is_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 0;
    let jump_is_active = processor.get_jump_active(player_id).unwrap_or(0) % 2 == 0;
    let double_jump_is_active = processor.get_double_jump_active(player_id).unwrap_or(0) % 2 == 0;
    let value = if dodge_is_active || jump_is_active || double_jump_is_active {
        1.0
    } else {
        0.0
    };
    convert_all!(string_error!("{:?}"), value)
}

player_feature_adder!(
    PlayerDemolishedBy,
    get_player_demolished_by,
    "player demolished by"
);

pub fn get_player_demolished_by<F: TryFrom<f32>>(
    player_id: &PlayerId,
    processor: &ReplayProcessor,
    _frame: &boxcars::Frame,
    _frame_number: usize,
) -> Result<[F; 1], String>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let car_actor_id = processor.get_car_actor_id(player_id)?;
    let demolisher_index = processor
        .get_active_demolish_fx()?
        .find(|demolish_fx| demolish_fx.victim == car_actor_id)
        .and_then(|demolish_fx| {
            let demolisher_id = processor
                .get_player_id_from_car_id(&demolish_fx.attacker)
                .ok()?;
            processor
                .iter_player_ids_in_order()
                .position(|player_id| player_id == &demolisher_id)
        })
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(-1);
    convert_all!(string_error!("{:?}"), demolisher_index as f32)
}

lazy_static! {
    static ref NAME_TO_GLOBAL_FEATURE_ADDER: std::collections::HashMap<&'static str, Arc<dyn FeatureAdder<f32> + Send + Sync + 'static>> = {
        let mut m: std::collections::HashMap<
            &'static str,
            Arc<dyn FeatureAdder<f32> + Send + Sync + 'static>,
        > = std::collections::HashMap::new();
        macro_rules! insert_adder {
            ($adder_name:ident ) => {
                m.insert(stringify!($adder_name), $adder_name::<f32>::arc_new());
            };
        }
        insert_adder!(BallRigidBody);
        insert_adder!(BallRigidBodyNoVelocities);
        insert_adder!(SecondsRemaining);
        m
    };
    static ref NAME_TO_PLAYER_FEATURE_ADDER: std::collections::HashMap<
        &'static str,
        Arc<dyn PlayerFeatureAdder<f32> + Send + Sync + 'static>,
    > = {
        let mut m: std::collections::HashMap<
            &'static str,
            Arc<dyn PlayerFeatureAdder<f32> + Send + Sync + 'static>,
        > = std::collections::HashMap::new();
        macro_rules! insert_adder {
            ($adder_name:ident ) => {
                m.insert(stringify!($adder_name), $adder_name::<f32>::arc_new());
            };
        }
        insert_adder!(PlayerRigidBody);
        insert_adder!(PlayerRigidBodyNoVelocities);
        insert_adder!(PlayerBoost);
        insert_adder!(PlayerJump);
        insert_adder!(PlayerAnyJump);
        insert_adder!(PlayerDemolishedBy);
        m
    };
}
