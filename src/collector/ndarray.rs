use crate::*;
use ::ndarray;
use boxcars;
pub use derive_new;
use lazy_static::lazy_static;
pub use paste;
use serde::Serialize;
use std::sync::Arc;

/// Represents the column headers in the collected data of an [`NDArrayCollector`].
///
/// # Fields
///
/// * `global_headers`: A list of strings that represent the global,
///   player-independent features' column headers.
/// * `player_headers`: A list of strings that represent the player-specific
///   features' column headers.
///
/// Use [`Self::new`] to construct an instance of this struct.
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

/// A struct that contains both the metadata of a replay and the associated
/// column headers.
///
/// # Fields
///
/// * `replay_meta`: Contains metadata about a [`boxcars::Replay`].
/// * `column_headers`: The [`NDArrayColumnHeaders`] associated with the data
///   collected from the replay.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayMetaWithHeaders {
    pub replay_meta: ReplayMeta,
    pub column_headers: NDArrayColumnHeaders,
}

impl ReplayMetaWithHeaders {
    pub fn headers_vec(&self) -> Vec<String> {
        self.headers_vec_from(|_, _info, index| format!("Player {index} - "))
    }

    pub fn headers_vec_from<F>(&self, player_prefix_getter: F) -> Vec<String>
    where
        F: Fn(&Self, &PlayerInfo, usize) -> String,
    {
        self.column_headers
            .global_headers
            .iter()
            .cloned()
            .chain(self.replay_meta.player_order().enumerate().flat_map(
                move |(player_index, info)| {
                    let player_prefix = player_prefix_getter(self, info, player_index);
                    self.column_headers
                        .player_headers
                        .iter()
                        .map(move |header| format!("{player_prefix}{header}"))
                },
            ))
            .collect()
    }
}

/// [`NDArrayCollector`] is a [`Collector`] which transforms frame-based replay
/// data into a 2-dimensional array of type [`ndarray::Array2`], where each
/// element is of a specified floating point type.
///
/// It's initialized with collections of [`FeatureAdder`] instances which
/// extract global, player independent features for each frame, and
/// [`PlayerFeatureAdder`], which add player specific features for each frame.
///
/// It's main entrypoint is [`Self::get_meta_and_ndarray`], which provides
/// [`ndarray::Array2`] along with column headers and replay metadata.
pub struct NDArrayCollector<F> {
    feature_adders: FeatureAdders<F>,
    player_feature_adders: PlayerFeatureAdders<F>,
    data: Vec<F>,
    replay_meta: Option<ReplayMeta>,
    frames_added: usize,
}

impl<F> NDArrayCollector<F> {
    /// Creates a new instance of `NDArrayCollector`.
    ///
    /// # Arguments
    ///
    /// * `feature_adders` - A vector of [`Arc<dyn FeatureAdder<F>>`], each
    ///   implementing the [`FeatureAdder`] trait. These are used to add global
    ///   features to the replay data.
    ///
    /// * `player_feature_adders` - A vector of [`Arc<dyn PlayerFeatureAdder<F>>`],
    ///   each implementing the [`PlayerFeatureAdder`]
    ///   trait. These are used to add player-specific features to the replay
    ///   data.
    ///
    /// # Returns
    ///
    /// A new [`NDArrayCollector`] instance. This instance is initialized with
    /// empty data, no replay metadata and zero frames added.
    pub fn new(
        feature_adders: FeatureAdders<F>,
        player_feature_adders: PlayerFeatureAdders<F>,
    ) -> Self {
        Self {
            feature_adders,
            player_feature_adders,
            data: Vec::new(),
            replay_meta: None,
            frames_added: 0,
        }
    }

    /// Returns the column headers of the 2-dimensional array produced by the
    /// [`NDArrayCollector`].
    ///
    /// # Returns
    ///
    /// An instance of [`NDArrayColumnHeaders`] representing the column headers
    /// in the collected data.
    pub fn get_column_headers(&self) -> NDArrayColumnHeaders {
        let global_headers = self
            .feature_adders
            .iter()
            .flat_map(move |fa| {
                fa.get_column_headers()
                    .iter()
                    .map(move |column_name| column_name.to_string())
            })
            .collect();
        let player_headers = self
            .player_feature_adders
            .iter()
            .flat_map(move |pfa| {
                pfa.get_column_headers()
                    .iter()
                    .map(move |base_name| base_name.to_string())
            })
            .collect();
        NDArrayColumnHeaders::new(global_headers, player_headers)
    }

    /// This function consumes the [`NDArrayCollector`] instance and returns the
    /// data collected as an [`ndarray::Array2`].
    ///
    /// # Returns
    ///
    /// A [`SubtrActorResult`] containing the collected data as an
    /// [`ndarray::Array2`].
    ///
    /// This method is a shorthand for calling [`Self::get_meta_and_ndarray`]
    /// and discarding the replay metadata and headers.
    pub fn get_ndarray(self) -> SubtrActorResult<ndarray::Array2<F>> {
        self.get_meta_and_ndarray().map(|a| a.1)
    }

    /// Consumes the [`NDArrayCollector`] and returns the collected features as a
    /// 2D ndarray, along with replay metadata and headers.
    ///
    /// # Returns
    ///
    /// A [`SubtrActorResult`] containing a tuple:
    /// - [`ReplayMetaWithHeaders`]: The replay metadata along with the headers
    ///   for each column in the ndarray.
    /// - [`ndarray::Array2<F>`]: The collected features as a 2D ndarray.
    pub fn get_meta_and_ndarray(
        self,
    ) -> SubtrActorResult<(ReplayMetaWithHeaders, ndarray::Array2<F>)> {
        let features_per_row = self.try_get_frame_feature_count()?;
        let expected_length = features_per_row * self.frames_added;
        assert!(self.data.len() == expected_length);
        let column_headers = self.get_column_headers();
        Ok((
            ReplayMetaWithHeaders {
                replay_meta: self.replay_meta.ok_or(SubtrActorError::new(
                    SubtrActorErrorVariant::CouldNotBuildReplayMeta,
                ))?,
                column_headers,
            },
            ndarray::Array2::from_shape_vec((self.frames_added, features_per_row), self.data)
                .map_err(SubtrActorErrorVariant::NDArrayShapeError)
                .map_err(SubtrActorError::new)?,
        ))
    }

    /// Processes a [`boxcars::Replay`] and returns its metadata along with column headers.
    ///
    /// This method first processes the replay using a [`ReplayProcessor`]. It
    /// then updates the `replay_meta` field if it's not already set, and
    /// returns a clone of the `replay_meta` field along with column headers of
    /// the data.
    ///
    /// # Arguments
    ///
    /// * `replay`: A reference to the [`boxcars::Replay`] to process.
    ///
    /// # Returns
    ///
    /// A [`SubtrActorResult`] containing a [`ReplayMetaWithHeaders`] that
    /// includes the metadata of the replay and column headers.
    pub fn process_and_get_meta_and_headers(
        &mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayMetaWithHeaders> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process_long_enough_to_get_actor_ids()?;
        self.maybe_set_replay_meta(&processor)?;
        Ok(ReplayMetaWithHeaders {
            replay_meta: self
                .replay_meta
                .as_ref()
                .ok_or(SubtrActorError::new(
                    SubtrActorErrorVariant::CouldNotBuildReplayMeta,
                ))?
                .clone(),
            column_headers: self.get_column_headers(),
        })
    }

    fn try_get_frame_feature_count(&self) -> SubtrActorResult<usize> {
        let player_count = self
            .replay_meta
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::CouldNotBuildReplayMeta,
            ))?
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

    fn maybe_set_replay_meta(&mut self, processor: &ReplayProcessor) -> SubtrActorResult<()> {
        if self.replay_meta.is_none() {
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
        current_time: f32,
    ) -> SubtrActorResult<collector::TimeAdvance> {
        self.maybe_set_replay_meta(processor)?;

        if !processor.ball_rigid_body_exists()? {
            return Ok(collector::TimeAdvance::NextFrame);
        }

        for feature_adder in self.feature_adders.iter() {
            feature_adder.add_features(
                processor,
                frame,
                frame_number,
                current_time,
                &mut self.data,
            )?;
        }

        for player_id in processor.iter_player_ids_in_order() {
            for player_feature_adder in self.player_feature_adders.iter() {
                player_feature_adder.add_features(
                    player_id,
                    processor,
                    frame,
                    frame_number,
                    current_time,
                    &mut self.data,
                )?;
            }
        }

        self.frames_added += 1;

        Ok(collector::TimeAdvance::NextFrame)
    }
}

impl NDArrayCollector<f32> {
    pub fn from_strings(fa_names: &[&str], pfa_names: &[&str]) -> SubtrActorResult<Self> {
        let feature_adders: Vec<Arc<dyn FeatureAdder<f32> + Send + Sync>> = fa_names
            .iter()
            .map(|name| {
                Ok(NAME_TO_GLOBAL_FEATURE_ADDER
                    .get(name)
                    .ok_or_else(|| {
                        SubtrActorError::new(SubtrActorErrorVariant::UnknownFeatureAdderName(
                            name.to_string(),
                        ))
                    })?
                    .clone())
            })
            .collect::<SubtrActorResult<Vec<_>>>()?;
        let player_feature_adders: Vec<Arc<dyn PlayerFeatureAdder<f32> + Send + Sync>> = pfa_names
            .iter()
            .map(|name| {
                Ok(NAME_TO_PLAYER_FEATURE_ADDER
                    .get(name)
                    .ok_or_else(|| {
                        SubtrActorError::new(SubtrActorErrorVariant::UnknownFeatureAdderName(
                            name.to_string(),
                        ))
                    })?
                    .clone())
            })
            .collect::<SubtrActorResult<Vec<_>>>()?;
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

/// This trait acts as an abstraction over a feature adder, and is primarily
/// used to allow for heterogeneous collections of feature adders in the
/// [`NDArrayCollector`]. While it provides methods for adding features and
/// retrieving column headers, it is generally recommended to implement the
/// [`LengthCheckedFeatureAdder`] trait instead, which provides compile-time
/// guarantees about the number of features returned.
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
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()>;
}

pub type FeatureAdders<F> = Vec<Arc<dyn FeatureAdder<F> + Send + Sync>>;

/// This trait is stricter version of the [`FeatureAdder`] trait, enforcing at
/// compile time that the number of features added is equal to the number of
/// column headers provided. Implementations of this trait can be automatically
/// adapted to the [`FeatureAdder`] trait using the [`impl_feature_adder!`]
/// macro.
pub trait LengthCheckedFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn get_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<[F; N]>;
}

/// A macro to provide an automatic implementation of the [`FeatureAdder`] trait
/// for types that implement [`LengthCheckedFeatureAdder`]. This allows you to
/// take advantage of the compile-time guarantees provided by
/// [`LengthCheckedFeatureAdder`], while still being able to use your type in
/// contexts that require a [`FeatureAdder`] object. This macro is used to
/// bridge the gap between the two traits, as Rust's type system does not
/// currently provide a way to prove to the compiler that there will always be
/// exactly one implementation of [`LengthCheckedFeatureAdder`] for each type.
#[macro_export]
macro_rules! impl_feature_adder {
    ($struct_name:ident) => {
        impl<F: TryFrom<f32>> FeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn add_features(
                &self,
                processor: &ReplayProcessor,
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

/// This trait acts as an abstraction over a player-specific feature adder, and
/// is primarily used to allow for heterogeneous collections of player feature
/// adders in the [`NDArrayCollector`]. While it provides methods for adding
/// player-specific features and retrieving column headers, it is generally
/// recommended to implement the [`LengthCheckedPlayerFeatureAdder`] trait
/// instead, which provides compile-time guarantees about the number of features
/// returned.
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
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()>;
}

pub type PlayerFeatureAdders<F> = Vec<Arc<dyn PlayerFeatureAdder<F> + Send + Sync>>;

/// This trait is a more strict version of the [`PlayerFeatureAdder`] trait,
/// enforcing at compile time that the number of player-specific features added
/// is equal to the number of column headers provided. Implementations of this
/// trait can be automatically adapted to the [`PlayerFeatureAdder`] trait using
/// the [`impl_player_feature_adder!`] macro.
pub trait LengthCheckedPlayerFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn get_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<[F; N]>;
}

/// A macro to provide an automatic implementation of the [`PlayerFeatureAdder`]
/// trait for types that implement [`LengthCheckedPlayerFeatureAdder`]. This
/// allows you to take advantage of the compile-time guarantees provided by
/// [`LengthCheckedPlayerFeatureAdder`], while still being able to use your type
/// in contexts that require a [`PlayerFeatureAdder`] object. This macro is used
/// to bridge the gap between the two traits, as Rust's type system does not
/// currently provide a way to prove to the compiler that there will always be
/// exactly one implementation of [`LengthCheckedPlayerFeatureAdder`] for each
/// type.
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
                processor: &ReplayProcessor,
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
    G: Fn(&ReplayProcessor, &boxcars::Frame, usize, f32) -> SubtrActorResult<[F; N]>,
{
    fn add_features(
        &self,
        processor: &ReplayProcessor,
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
    G: Fn(&PlayerId, &ReplayProcessor, &boxcars::Frame, usize, f32) -> SubtrActorResult<[F; N]>,
{
    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
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

/// This macro creates a global [`FeatureAdder`] struct and implements the
/// necessary traits to add the calculated features to the data matrix. The
/// macro exports a struct with the same name as passed in the parameter. The
/// number of column names and the length of the feature array returned by
/// `$prop_getter` are checked at compile time to ensure they match, in line
/// with the [`LengthCheckedFeatureAdder`] trait. The output struct also
/// provides an implementation of the [`FeatureAdder`] trait via the
/// [`impl_feature_adder!`] macro, allowing it to be used in contexts where a
/// [`FeatureAdder`] object is required.
///
/// # Parameters
///
/// * `$struct_name`: The name of the struct to be created.
/// * `$prop_getter`: The function or closure used to calculate the features.
/// * `$( $column_names:expr ),*`: A comma-separated list of column names as strings.
///
/// # Example
///
/// ```
/// use subtr_actor::*;
///
/// build_global_feature_adder!(
///     SecondsRemainingExample,
///     |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
///         convert_all_floats!(processor.get_seconds_remaining()?.clone() as f32)
///     },
///     "seconds remaining"
/// );
/// ```
///
/// This will create a struct named `SecondsRemaining` and implement necessary
/// traits to calculate features using the provided closure. The feature will be
/// added under the column name "seconds remaining". Note, however, that it is
/// possible to add more than one feature with each feature adder
#[macro_export]
macro_rules! build_global_feature_adder {
    ($struct_name:ident, $prop_getter:expr, $( $column_names:expr ),* $(,)?) => {

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

/// This macro is used to implement necessary traits for an existing struct to
/// add the calculated features to the data matrix. This macro is particularly
/// useful when the feature adder needs to be instantiated with specific
/// parameters. The number of column names and the length of the feature array
/// returned by `$prop_getter` are checked at compile time to ensure they match.
///
/// # Parameters
///
/// * `$struct_name`: The name of the existing struct.
/// * `$prop_getter`: The function or closure used to calculate the features.
/// * `$( $column_names:expr ),*`: A comma-separated list of column names as strings.
#[macro_export]
macro_rules! global_feature_adder {
    ($struct_name:ident, $prop_getter:expr, $( $column_names:expr ),* $(,)?) => {
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
                        processor: &ReplayProcessor,
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

/// This macro creates a player feature adder struct and implements the
/// necessary traits to add the calculated player-specific features to the data
/// matrix. The macro exports a struct with the same name as passed in the
/// parameter. The number of column names and the length of the feature array
/// returned by `$prop_getter` are checked at compile time to ensure they match,
/// in line with the [`LengthCheckedPlayerFeatureAdder`] trait. The output
/// struct also provides an implementation of the [`PlayerFeatureAdder`] trait
/// via the [`impl_player_feature_adder!`] macro, allowing it to be used in
/// contexts where a [`PlayerFeatureAdder`] object is required.
///
/// # Parameters
///
/// * `$struct_name`: The name of the struct to be created.
/// * `$prop_getter`: The function or closure used to calculate the features.
/// * `$( $column_names:expr ),*`: A comma-separated list of column names as strings.
///
/// # Example
///
/// ```
/// use subtr_actor::*;
///
/// fn u8_get_f32(v: u8) -> SubtrActorResult<f32> {
///    v.try_into().map_err(convert_float_conversion_error)
/// }
///
/// build_player_feature_adder!(
///     PlayerJump,
///     |_,
///      player_id: &PlayerId,
///      processor: &ReplayProcessor,
///      _frame,
///      _frame_number,
///      _current_time: f32| {
///         convert_all_floats!(
///             processor
///                 .get_dodge_active(player_id)
///                 .and_then(u8_get_f32)
///                 .unwrap_or(0.0),
///             processor
///                 .get_jump_active(player_id)
///                 .and_then(u8_get_f32)
///                 .unwrap_or(0.0),
///             processor
///                 .get_double_jump_active(player_id)
///                 .and_then(u8_get_f32)
///                 .unwrap_or(0.0),
///         )
///     },
///     "dodge active",
///     "jump active",
///     "double jump active"
/// );
/// ```
///
/// This will create a struct named `PlayerJump` and implement necessary
/// traits to calculate features using the provided closure. The player-specific
/// features will be added under the column names "dodge active",
/// "jump active", and "double jump active" respectively.
#[macro_export]
macro_rules! build_player_feature_adder {
    ($struct_name:ident, $prop_getter:expr, $( $column_names:expr ),* $(,)?) => {
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

/// This macro is used to implement necessary traits for an existing struct to
/// add the calculated player-specific features to the data matrix. This macro
/// is particularly useful when the feature adder needs to be instantiated with
/// specific parameters. The number of column names and the length of the
/// feature array returned by `$prop_getter` are checked at compile time to
/// ensure they match.
///
/// # Parameters
///
/// * `$struct_name`: The name of the existing struct.
/// * `$prop_getter`: The function or closure used to calculate the features.
/// * `$( $column_names:expr ),*`: A comma-separated list of column names as strings.
#[macro_export]
macro_rules! player_feature_adder {
    ($struct_name:ident, $prop_getter:expr, $( $column_names:expr ),* $(,)?) => {
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
                        processor: &ReplayProcessor,
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

/// Unconditionally convert any error into a [`SubtrActorError`] of with the
/// [`SubtrActorErrorVariant::FloatConversionError`] variant.
pub fn convert_float_conversion_error<T>(_: T) -> SubtrActorError {
    SubtrActorError::new(SubtrActorErrorVariant::FloatConversionError)
}

/// A macro that tries to convert each provided item into a type. If any of the
/// conversions fail, it short-circuits and returns the error.
///
/// The first argument `$err` is a closure that accepts an error and returns a
/// [`SubtrActorResult`]. It is used to map any conversion errors into a
/// [`SubtrActorResult`].
///
/// Subsequent arguments should be expressions that implement the [`TryInto`]
/// trait, with the type they're being converted into being the one used in the
/// `Ok` variant of the return value.
#[macro_export]
macro_rules! convert_all {
    ($err:expr, $( $item:expr ),* $(,)?) => {{
		Ok([
			$( $item.try_into().map_err($err)? ),*
		])
	}};
}

/// A convenience macro that uses the [`convert_all`] macro with the
/// [`convert_float_conversion_error`] function for error handling.
///
/// Each item provided is attempted to be converted into a floating point
/// number. If any of the conversions fail, it short-circuits and returns the
/// error. This macro must be used in the context of a function that returns a
/// [`Result`] because it uses the ? operator. It is primarily useful for
/// defining function like the one shown in the example below that are generic
/// in some parameter that can implements [`TryFrom`].
///
/// # Example
///
/// ```
/// use subtr_actor::*;
///
/// pub fn some_constant_function<F: TryFrom<f32>>(
///     rigid_body: &boxcars::RigidBody,
/// ) -> SubtrActorResult<[F; 3]> {
///     convert_all_floats!(42.0, 0.0, 1.234)
/// }
/// ```
#[macro_export]
macro_rules! convert_all_floats {
    ($( $item:expr ),* $(,)?) => {{
        convert_all!(convert_float_conversion_error, $( $item ),*)
    }};
}

fn or_zero_boxcars_3f() -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

type RigidBodyArrayResult<F> = SubtrActorResult<[F; 12]>;

/// Extracts the location, rotation, linear velocity and angular velocity from a
/// [`boxcars::RigidBody`] and converts them to a type implementing [`TryFrom<f32>`].
///
/// If any of the components of the rigid body are not set (`None`), they are
/// treated as zero.
///
/// The returned array contains twelve elements in the following order: x, y, z
/// location, x, y, z rotation (as Euler angles), x, y, z linear velocity, x, y,
/// z angular velocity.
pub fn get_rigid_body_properties<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> RigidBodyArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
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
    convert_all_floats!(
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

/// Extracts the location and rotation from a [`boxcars::RigidBody`] and
/// converts them to a type implementing [`TryFrom<f32>`].
///
/// If any of the components of the rigid body are not set (`None`), they are
/// treated as zero.
///
/// The returned array contains seven elements in the following order: x, y, z
/// location, x, y, z, w rotation.
pub fn get_rigid_body_properties_no_velocities<F: TryFrom<f32>>(
    rigid_body: &boxcars::RigidBody,
) -> SubtrActorResult<[F; 7]>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let rotation = rigid_body.rotation;
    let location = rigid_body.location;
    convert_all_floats!(
        location.x, location.y, location.z, rotation.x, rotation.y, rotation.z, rotation.w
    )
}

fn default_rb_state<F: TryFrom<f32>>() -> RigidBodyArrayResult<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all!(
        convert_float_conversion_error,
        // We use huge values for location instead of 0s so that hopefully any
        // model built on this data can understand that the player is not
        // actually on the field.
        0.0,
        0.0,
        0.0,
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

fn default_rb_state_no_velocities<F: TryFrom<f32>>() -> SubtrActorResult<[F; 7]>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    convert_all_floats!(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,)
}

build_global_feature_adder!(
    SecondsRemaining,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        convert_all_floats!(processor.get_seconds_remaining()?.clone() as f32)
    },
    "seconds remaining"
);

build_global_feature_adder!(
    CurrentTime,
    |_, _processor, _frame, _index, current_time: f32| { convert_all_floats!(current_time) },
    "current time"
);

build_global_feature_adder!(
    FrameTime,
    |_, _processor, frame: &boxcars::Frame, _index, _current_time| {
        convert_all_floats!(frame.time)
    },
    "frame time"
);

build_global_feature_adder!(
    BallRigidBody,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        get_rigid_body_properties(processor.get_ball_rigid_body()?)
    },
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

build_global_feature_adder!(
    BallRigidBodyNoVelocities,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        get_rigid_body_properties_no_velocities(processor.get_ball_rigid_body()?)
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - rotation w",
);

// XXX: This approach seems to give some unexpected results with rotation
// changes. There may be a unit mismatch or some other type of issue.
build_global_feature_adder!(
    VelocityAddedBallRigidBodyNoVelocities,
    |_, processor: &ReplayProcessor, _frame, _index, current_time: f32| {
        get_rigid_body_properties_no_velocities(
            &processor.get_velocity_applied_ball_rigid_body(current_time)?,
        )
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - rotation w",
);

#[derive(derive_new::new)]
pub struct InterpolatedBallRigidBodyNoVelocities<F> {
    close_enough_to_frame_time: f32,
    _zero: std::marker::PhantomData<F>,
}

impl<F> InterpolatedBallRigidBodyNoVelocities<F> {
    pub fn arc_new(close_enough_to_frame_time: f32) -> Arc<Self> {
        Arc::new(Self::new(close_enough_to_frame_time))
    }
}

global_feature_adder!(
    InterpolatedBallRigidBodyNoVelocities,
    |s: &InterpolatedBallRigidBodyNoVelocities<F>,
     processor: &ReplayProcessor,
     _frame: &boxcars::Frame,
     _index,
     current_time: f32| {
        processor
            .get_interpolated_ball_rigid_body(current_time, s.close_enough_to_frame_time)
            .map(|v| get_rigid_body_properties_no_velocities(&v))
            .unwrap_or_else(|_| default_rb_state_no_velocities())
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - rotation x",
    "Ball - rotation y",
    "Ball - rotation z",
    "Ball - rotation w",
);

build_player_feature_adder!(
    PlayerRigidBody,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        if let Ok(rb) = processor.get_player_rigid_body(player_id) {
            get_rigid_body_properties(rb)
        } else {
            default_rb_state()
        }
    },
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

build_player_feature_adder!(
    PlayerRigidBodyNoVelocities,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        if let Ok(rb) = processor.get_player_rigid_body(player_id) {
            get_rigid_body_properties_no_velocities(rb)
        } else {
            default_rb_state_no_velocities()
        }
    },
    "position x",
    "position y",
    "position z",
    "rotation x",
    "rotation y",
    "rotation z",
    "rotation w"
);

// XXX: This approach seems to give some unexpected results with rotation
// changes. There may be a unit mismatch or some other type of issue.
build_player_feature_adder!(
    VelocityAddedPlayerRigidBodyNoVelocities,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, current_time: f32| {
        if let Ok(rb) = processor.get_velocity_applied_player_rigid_body(player_id, current_time) {
            get_rigid_body_properties_no_velocities(&rb)
        } else {
            default_rb_state_no_velocities()
        }
    },
    "position x",
    "position y",
    "position z",
    "rotation x",
    "rotation y",
    "rotation z",
    "rotation w"
);

#[derive(derive_new::new)]
pub struct InterpolatedPlayerRigidBodyNoVelocities<F> {
    close_enough_to_frame_time: f32,
    _zero: std::marker::PhantomData<F>,
}

impl<F> InterpolatedPlayerRigidBodyNoVelocities<F> {
    pub fn arc_new(close_enough_to_frame_time: f32) -> Arc<Self> {
        Arc::new(Self::new(close_enough_to_frame_time))
    }
}

player_feature_adder!(
    InterpolatedPlayerRigidBodyNoVelocities,
    |s: &InterpolatedPlayerRigidBodyNoVelocities<F>,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame: &boxcars::Frame,
     _index,
     current_time: f32| {
        processor
            .get_interpolated_player_rigid_body(
                player_id,
                current_time,
                s.close_enough_to_frame_time,
            )
            .map(|v| get_rigid_body_properties_no_velocities(&v))
            .unwrap_or_else(|_| default_rb_state_no_velocities())
    },
    "i position x",
    "i position y",
    "i position z",
    "i rotation x",
    "i rotation y",
    "i rotation z",
    "i rotation w"
);

build_player_feature_adder!(
    PlayerBoost,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        convert_all_floats!(processor.get_player_boost_level(player_id).unwrap_or(0.0))
    },
    "boost level"
);

fn u8_get_f32(v: u8) -> SubtrActorResult<f32> {
    Ok(v.into())
}

build_player_feature_adder!(
    PlayerJump,
    |_,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame,
     _frame_number,
     _current_time: f32| {
        convert_all_floats!(
            processor
                .get_dodge_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
            processor
                .get_jump_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
            processor
                .get_double_jump_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
        )
    },
    "dodge active",
    "jump active",
    "double jump active"
);

build_player_feature_adder!(
    PlayerAnyJump,
    |_,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame,
     _frame_number,
     _current_time: f32| {
        let dodge_is_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2;
        let jump_is_active = processor.get_jump_active(player_id).unwrap_or(0) % 2;
        let double_jump_is_active = processor.get_double_jump_active(player_id).unwrap_or(0) % 2;
        let value: f32 = [dodge_is_active, jump_is_active, double_jump_is_active]
            .into_iter()
            .enumerate()
            .map(|(index, is_active)| (1 << index) * is_active)
            .sum::<u8>() as f32;
        convert_all_floats!(value)
    },
    "any_jump_active"
);

const DEMOLISH_APPEARANCE_FRAME_COUNT: usize = 30;

build_player_feature_adder!(
    PlayerDemolishedBy,
    |_,
     player_id: &PlayerId,
     processor: &ReplayProcessor,
     _frame,
     frame_number,
     _current_time: f32| {
        let demolisher_index = processor
            .demolishes
            .iter()
            .find(|demolish_info| {
                &demolish_info.victim == player_id
                    && frame_number - demolish_info.frame < DEMOLISH_APPEARANCE_FRAME_COUNT
            })
            .map(|demolish_info| {
                processor
                    .iter_player_ids_in_order()
                    .position(|player_id| player_id == &demolish_info.attacker)
                    .unwrap_or_else(|| processor.iter_player_ids_in_order().count())
            })
            .and_then(|v| i32::try_from(v).ok())
            .unwrap_or(-1);
        convert_all_floats!(demolisher_index as f32)
    },
    "player demolished by"
);

build_player_feature_adder!(
    PlayerRigidBodyQuaternions,
    |_, player_id: &PlayerId, processor: &ReplayProcessor, _frame, _index, _current_time: f32| {
        if let Ok(rb) = processor.get_player_rigid_body(player_id) {
            let rotation = rb.rotation;
            let location = rb.location;
            convert_all_floats!(
                location.x, location.y, location.z, rotation.x, rotation.y, rotation.z, rotation.w
            )
        } else {
            convert_all_floats!(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0)
        }
    },
    "position x",
    "position y",
    "position z",
    "quaternion x",
    "quaternion y",
    "quaternion z",
    "quaternion w"
);

build_global_feature_adder!(
    BallRigidBodyQuaternions,
    |_, processor: &ReplayProcessor, _frame, _index, _current_time| {
        let rb = processor.get_ball_rigid_body()?;
        let rotation = rb.rotation;
        let location = rb.location;
        convert_all_floats!(
            location.x, location.y, location.z, rotation.x, rotation.y, rotation.z, rotation.w
        )
    },
    "Ball - position x",
    "Ball - position y",
    "Ball - position z",
    "Ball - quaternion x",
    "Ball - quaternion y",
    "Ball - quaternion z",
    "Ball - quaternion w"
);

lazy_static! {
    static ref NAME_TO_GLOBAL_FEATURE_ADDER: std::collections::HashMap<&'static str, Arc<dyn FeatureAdder<f32> + Send + Sync + 'static>> = {
        let mut m: std::collections::HashMap<
            &'static str,
            Arc<dyn FeatureAdder<f32> + Send + Sync + 'static>,
        > = std::collections::HashMap::new();
        macro_rules! insert_adder {
            ($adder_name:ident, $( $arguments:expr ),*) => {
                m.insert(stringify!($adder_name), $adder_name::<f32>::arc_new($ ( $arguments ),*));
            };
            ($adder_name:ident) => {
                insert_adder!($adder_name,)
            }
        }
        insert_adder!(BallRigidBody);
        insert_adder!(BallRigidBodyNoVelocities);
        insert_adder!(BallRigidBodyQuaternions);
        insert_adder!(VelocityAddedBallRigidBodyNoVelocities);
        insert_adder!(InterpolatedBallRigidBodyNoVelocities, 0.0);
        insert_adder!(SecondsRemaining);
        insert_adder!(CurrentTime);
        insert_adder!(FrameTime);
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
            ($adder_name:ident, $( $arguments:expr ),*) => {
                m.insert(stringify!($adder_name), $adder_name::<f32>::arc_new($ ( $arguments ),*));
            };
            ($adder_name:ident) => {
                insert_adder!($adder_name,)
            };
        }
        insert_adder!(PlayerRigidBody);
        insert_adder!(PlayerRigidBodyNoVelocities);
        insert_adder!(PlayerRigidBodyQuaternions);
        insert_adder!(VelocityAddedPlayerRigidBodyNoVelocities);
        insert_adder!(InterpolatedPlayerRigidBodyNoVelocities, 0.003);
        insert_adder!(PlayerBoost);
        insert_adder!(PlayerJump);
        insert_adder!(PlayerAnyJump);
        insert_adder!(PlayerDemolishedBy);
        m
    };
}
