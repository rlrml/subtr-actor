# subtr-actor

`subtr-actor` is a Rocket League replay processing monorepo. The Rust crate turns
raw `boxcars` replay data into higher-level game state, frame-by-frame structured
data, and configurable numeric feature matrices for analytics and ML workflows.
The same core pipeline is exposed through Python and JavaScript bindings.

## Major Components

- `src/`: Rust core library. The main flow is `ReplayProcessor` over modeled
  actor state, with collectors layered on top.
- `src/collector/`: Output modes built on the processing pipeline.
  `replay_data.rs` emits structured frame data, `ndarray.rs` builds numeric
  feature matrices, and `stats_timeline.rs` produces cumulative stat snapshots.
- `src/stats_export/`: Higher-level stat extraction modules for exported replay
  statistics such as movement, boost, possession, positioning, and demos.
- `python/`: PyO3 bindings that expose the Rust pipeline to Python and return
  `numpy` arrays plus metadata.
- `js/`: `wasm-bindgen` bindings and npm packaging for the web/JS API.
  `js/example/` is a small consumer app, and `js/stats-timeline-viewer/` is a
  UI for inspecting timeline-style outputs.
- `assets/`: Replay fixtures and downloaded ballchasing fixtures used for
  testing and investigation.
- `scripts/` and `.github/workflows/`: Release/version checks and CI/publishing
  automation for the Rust crate, PyPI package, and npm package.

## Working Notes

- Treat the Rust crate as the source of truth. Binding changes in `python/` and
  `js/` usually mirror behavior already defined in `src/`.
- Most feature extraction work lands either in `src/collector/ndarray.rs`,
  `src/collector/replay_data.rs`, or `src/stats_export/`, depending on whether
  the output is numeric, structured, or report-oriented.
- Fixtures under `assets/` are important for replay-format regressions, so keep
  them aligned with any parsing or stat extraction changes.

## More Detail

- General project docs live in [docs/README.md](./docs/README.md).
- Release process: [docs/RELEASING.md](./docs/RELEASING.md)
- Changelog: [docs/CHANGELOG.md](./docs/CHANGELOG.md)
- Package-specific usage details: [python/README.md](./python/README.md) and
  [js/README.md](./js/README.md)
