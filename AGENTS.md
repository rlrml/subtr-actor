# subtr-actor

`subtr-actor` is a Rocket League replay processing monorepo. The Rust crate turns
raw `boxcars` replay data into higher-level game state, frame-by-frame structured
data, and configurable numeric feature matrices for analytics and ML workflows.
The same core pipeline is exposed through Python and JavaScript bindings.

## Major Components

- `src/`: Rust core library. The main replay-processing flow lives under
  `src/processor/`, with bootstrap, queries, and updater modules operating over
  modeled actor state before collectors consume the resulting processor state.
- `src/collector/`: Output modes built on the processing pipeline.
  `replay_data.rs` emits structured frame data, `ndarray/` builds numeric
  feature matrices, and `stats_timeline.rs` produces cumulative stat snapshots.
- `src/stats/`: Higher-level stat extraction modules for exported replay
  statistics. `reducers/` contains the per-frame stat reducers, `export/`
  defines exported stat fields and module wiring, and `comparison/` holds
  stat-comparison tooling.
- `js/player/`: Reusable replay player library package. It handles replay
  loading, normalization, scene playback, camera APIs, timeline overlays, and
  plugin-based viewer extensions on top of the core WASM bindings.
- `js/stat-evaluation-player/`: Stats-focused replay viewer package built on top
  of `js/player/` plus the stats timeline bindings. It is the main home for UI
  that visualizes stat accumulation, overlays, and per-module stat panels
  during playback.

## Working Notes

- Treat the Rust crate as the source of truth. Binding changes in `python/` and
  `js/` usually mirror behavior already defined in `src/`.
- For the current stats DAG layout, see
  [`docs/calculators-and-analysis-nodes.md`](./docs/calculators-and-analysis-nodes.md).
- Most feature extraction work lands either in `src/collector/ndarray/`,
  `src/collector/replay_data.rs`, `src/collector/stats_timeline.rs`, or
  `src/stats/export/`, depending on whether the output is numeric, structured,
  cumulative-over-time, or report-oriented.
- Replay-player infrastructure work usually belongs in `js/player/`. Stats UI
  and stat-timeline visualization work usually belongs in
  `js/stat-evaluation-player/`.

## Agent Workspace

- Codex reads this `AGENTS.md` file as the canonical repo instructions.
- Shared agent workflows, reusable rules, and local agent helper docs belong
  under `.agents/`.
- Legacy Claude-only settings may remain under `.claude/`, but they are not a
  Codex configuration surface. If a Claude setting contains durable project
  guidance, translate it into this file or `.agents/`.

## Common Commands

- Rust formatting and checks generally use `cargo fmt` and `cargo test`.
- Rust build and maintenance commands should use `cargo ...`; `cargo clean`
  is acceptable when stale build artifacts are the issue.
- JavaScript package work under `js/` commonly uses `npm install`,
  `npm run build`, `npm run dev`, `npm run pack`, `npm pack`, and
  `npm publish`.
- WASM binding builds commonly use `wasm-pack build`.
- Prefer `rg` for text search. Use `grep` only when matching an existing
  script or when `rg` is unavailable.
