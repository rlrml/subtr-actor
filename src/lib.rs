//! # subtr-actor
//!
//! [`subtr-actor`](crate) is a versatile library designed to facilitate the
//! process of working with and extracting data from Rocket League replays.
//! Utilizing the powerful [`boxcars`] library for parsing, subtr-actor
//! simplifies the underlying actor-based structure of replay files, making them
//! more accessible and easier to manipulate.
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
//! - **[`Collector`]**: This trait outlines the blueprint for data
//! collection from replays. The Collector interfaces with a [`ReplayProcessor`],
//! handling frame data and guiding the pace of replay progression. It is
//! typically invoked repeatedly through the [`ReplayProcessor::process`] method
//! as the replay is processed frame by frame.
//!
//! Notably, subtr-actor includes implementations of the [`Collector`] trait,
//!
//! - **[`NDArrayCollector`]**: This [`Collector`] implementations translates
//! frame-based replay data into a 2 dimensional array in the form of a
//! [`::ndarray::Array2`] instance. The exact data that is recorded in each frame
//! can be configured with the [`FeatureAdder`] and [`PlayerFeatureAdder`]
//! instances that are provided to its constructor ([`NDArrayCollector::new`]).
//! This representation is ideal for use with machine learning libraries like
//! pytorch and tensorflow.
//!
//! - **[`ReplayData`]**: This [`Collector`] implementation provides an easy way
//! to get a serializable to e.g. json (though [`serde::Serialize`])
//! representation of the replay. The representation differs from what you might
//! get from e.g. raw boxcars in that it is not a complicated graph of actor
//! objects, but instead something more natural where the data associated with
//! each entity in the game is grouped together.

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
