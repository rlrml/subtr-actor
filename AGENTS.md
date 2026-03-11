# subtr-actor

`subtr-actor` is a Rocket League replay processing monorepo. The Rust crate turns
raw `boxcars` replay data into higher-level game state, frame-by-frame structured
data, and configurable numeric feature matrices for analytics and ML workflows.
The same core pipeline is exposed through Python and JavaScript bindings.

## Major Components

- `src/`: Rust core library. The main flow is in `src/processor.rs`  over modeled actor state, with collectors layered on top.
- `src/collector/`: Output modes built on the processing pipeline.
  `replay_data.rs` emits structured frame data, `ndarray.rs` builds numeric
  feature matrices, and `stats_timeline.rs` produces cumulative stat snapshots.
- `src/stats_export/`: Higher-level stat extraction modules for exported replay
  statistics such as movement, boost, possession, positioning, and demos.

## Working Notes

- Treat the Rust crate as the source of truth. Binding changes in `python/` and
  `js/` usually mirror behavior already defined in `src/`.
- Most feature extraction work lands either in `src/collector/ndarray.rs`,
  `src/collector/replay_data.rs`, or `src/stats_export/`, depending on whether
  the output is numeric, structured, or report-oriented.
