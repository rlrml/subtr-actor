//! # subtr-actor
//!
//! [`subtr-actor`](crate) is a versatile library designed to facilitate the
//! processes of working with and extracting data from Rocket League replays.
//! Utilizing the powerful [`boxcars`] library for parsing, subtr-actor
//! simplifies (or 'subtracts', as hinted by its name) the underlying
//! actor-based structure of replay files, making them more accessible and
//! easier to manipulate.
//!
//! ## Overview of Key Components
//!
//! - **[`ReplayProcessor`]**: This struct is at the heart of subtr-actor's
//! replay processing capabilities. In its main entry point,
//! [`ReplayProcessor::process`], it pushes network frames from the
//! [`boxcars::Replay`] that it is initialized with though an
//! [`ActorStateModeler`] instance, calling the [`Collector`] instance that is
//! provided as an argument as it does so. The [`Collector`] is provided with a
//! reference to the [`ReplayProcessor`] each time the it is invoked, which
//! allows it to use the suite of helper methods which greatly assist in the
//! navigation of the actor graph and the retrieval of information about the
//! current game state.
//!
//! - **[`Collector`]**: This trait outlines the blueprint for data collection
//! from replays. The [`Collector`] interfaces with a [`ReplayProcessor`],
//! handling frame data and guiding the pace of replay progression with
//! [`TimeAdvance`]. It is typically invoked repeatedly through the
//! [`ReplayProcessor::process`] method as the replay is processed frame by
//! frame.
//!
//! - **[`FrameRateDecorator`]**: This struct decorates a [`Collector`]
//! implementation with a target frame duration, controlling the frame rate of
//! the replay processing.
//!
//! ### Collector implementations
//!
//! [`subtr-actor`](crate) also includes implementations of the [`Collector`] trait:
//!
//! - **[`NDArrayCollector`]**: This [`Collector`] implementations translates
//! frame-based replay data into a 2 dimensional array in the form of a
//! [`::ndarray::Array2`] instance. The exact data that is recorded in each
//! frame can be configured with the [`FeatureAdder`] and [`PlayerFeatureAdder`]
//! instances that are provided to its constructor ([`NDArrayCollector::new`]).
//! Extending the exact behavior of [`NDArrayCollector`] is thus possible with
//! user defined [`FeatureAdder`] and [`PlayerFeatureAdder`], which is made easy
//! with the [`build_global_feature_adder!`] and [`build_player_feature_adder!`]
//! macros. The [`::ndarray::Array2`] produced by [`NDArrayCollector`] is ideal
//! for use with machine learning libraries like pytorch and tensorflow.
//!
//! - **[`ReplayData`]**: This [`Collector`] implementation provides an easy way
//! to get a serializable to e.g. json (though [`serde::Serialize`])
//! representation of the replay. The representation differs from what you might
//! get from e.g. raw [`boxcars`] in that it is not a complicated graph of actor
//! objects, but instead something more natural where the data associated with
//! each entity in the game is grouped together.
//!
//! ## Example
//!
//! In the following example, we demonstrate how to use [`boxcars`],
//! [`NDArrayCollector`] and [`FrameRateDecorator`] to write a function that
//! takes a replay filepath and collections of features adders and returns a
//! [`ReplayMetaWithHeaders`] along with a [`::ndarray::Array2`] . The resulting
//! [`::ndarray::Array2`] would be appropriate for use in a machine learning
//! context. Note that [`ReplayProcessor`] is also used implicitly here in the
//! [`Collector::process_replay`]
//!
//! ```
//! use subtr_actor::*;
//!
//! fn get_ndarray_with_info_from_replay_filepath(
//!     filepath: std::path::PathBuf,
//!     feature_adders: FeatureAdders<f32>,
//!     player_feature_adders: PlayerFeatureAdders<f32>,
//!     fps: Option<f32>,
//! ) -> anyhow::Result<(ReplayMetaWithHeaders, ::ndarray::Array2<f32>)> {
//!     let data = std::fs::read(filepath.as_path())?;
//!     let replay = boxcars::ParserBuilder::new(&data)
//!         .must_parse_network_data()
//!         .on_error_check_crc()
//!         .parse()?;
//!
//!     let mut collector = NDArrayCollector::new(feature_adders, player_feature_adders);
//!
//!     FrameRateDecorator::new_from_fps(fps.unwrap_or(10.0), &mut collector)
//!         .process_replay(&replay)
//!         .map_err(|e| e.variant)?;
//!
//!     Ok(collector.get_meta_and_ndarray().map_err(|e| e.variant)?)
//! }
//! ```

pub mod actor_state;
pub mod collector;
pub mod constants;
pub mod error;
pub mod processor;
pub mod util;

#[cfg(test)]
mod util_test;

pub use crate::actor_state::*;
pub use crate::collector::*;
pub use crate::constants::*;
pub use crate::error::*;
pub use crate::processor::*;
pub use crate::util::*;

#[macro_use]
extern crate derive_new;
