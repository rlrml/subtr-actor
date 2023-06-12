# subtr-actor

## subtr-actor

[`subtr-actor`](crate) is a versatile library designed to facilitate the process of
working with and extracting data from Rocket League replays. Utilizing the
powerful [`boxcars`] library for parsing, subtr-actor simplifies the
underlying complex actor-based structure of replay files, making them more
accessible and easier to manipulate.

### Overview of Key Components

- **[`ReplayProcessor`]**: This struct is at the heart of subtr-actor's
replay processing capabilities. The [`ReplayProcessor`] traverses the actor
graph, pushing frames through an [`ActorStateModeler`] to capture the state of
all actors at any given moment. It provides a suite of helper methods to
assist in the navigation of the actor graph and the retrieval of
information about the game as it progresses.

- **[`Collector`]**: This trait outlines the blueprint for data
collection from replays. The Collector interfaces with a ReplayProcessor,
handling frame data and guiding the pace of replay progression. It is
typically invoked repeatedly through the [`ReplayProcessor::process`] method
as the replay is processed frame by frame.

Notably, subtr-actor includes implementations of the [`Collector`] trait,

- **[`NDArrayCollector`]**: This [`Collector`] implementations translates
frame-based replay data into a 2 dimensional array in the form of a
[`::ndarray::Array2`] instance. The exact data that is recorded in each frame
can be configured with the [`FeatureAdder`] and [`PlayerFeatureAdder`]
instances that are provided to its constructor ([`NDArrayCollector::new`]).
This representation is ideal for use with machine learning libraries like
pytorch and tensorflow.

- **[`ReplayData`]**: This [`Collector`] implementation provides an easy way to
get a serializable to e.g. json (though [`serde::Serialize`]) representation
of the replay. The representation differs from what you might get from e.g.
raw boxcars in that it is not a complicated graph of actor objects and is
instead something more akin to the way a human might think of the data
contained in a replay.
