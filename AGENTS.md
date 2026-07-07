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
  statistics. `accumulators/` contains the per-stat accumulators,
  `calculators/` and `analysis_graph/` define the stats DAG, and `labels.rs`
  holds the labeled stat-aggregation types (`LabeledCounts`,
  `LabeledFloatSums`).
- `js/player/`: Reusable replay player library package. It handles replay
  loading, normalization, scene playback, camera APIs, timeline overlays, and
  plugin-based player extensions on top of the core WASM bindings.
- `js/stat-evaluation-player/`: Stats-focused replay player UI built on top
  of `js/player/` plus the stats timeline bindings. It is the main home for UI
  that visualizes stat accumulation, overlays, and per-module stat panels
  during playback.

## Working Notes

- Treat the Rust crate as the source of truth. Binding changes in `python/` and
  `js/` usually mirror behavior already defined in `src/`.
- Keep tests in separate files from production code. For Rust unit tests, prefer
  adjacent `*_tests.rs` files included with `#[cfg(test)] #[path = "..."] mod
  tests;`; for JavaScript/TypeScript, keep tests in `.test.ts` files.
- For the current stats DAG layout, see
  [`docs/calculators-and-analysis-nodes.md`](./docs/calculators-and-analysis-nodes.md).
- Most feature extraction work lands either in `src/collector/ndarray/`,
  `src/collector/replay_data.rs`, `src/collector/stats_timeline.rs`, or
  `src/stats/` (accumulators and the analysis DAG), depending on whether the
  output is numeric, structured, cumulative-over-time, or stat-oriented.
- Replay-player infrastructure work usually belongs in `js/player/`. Stats UI
  and stat-timeline visualization work usually belongs in
  `js/stat-evaluation-player/`.
- A dribble *flick* is the final dodge touch at the end of a run of `control`
  touches, and that single touch's launch impulse is delivered over several
  frames (the car drags the ball through the dodge) — not in the one frame the
  touch is first detected. So flick detection measures the ball's *peak*
  velocity change over a short window after the dodge-touch, and gates on real
  carry evidence (ball moving with the car), rather than a single-frame delta.
  See `FLICK_IMPULSE_WINDOW_SECONDS` / `FLICK_MAX_CARRY_REL_HORIZONTAL_SPEED` in
  `src/stats/calculators/flick.rs`.

## Agent Workspace

- Codex reads this `AGENTS.md` file as the canonical repo instructions.
- Shared agent workflows, reusable rules, and local agent helper docs belong
  under `.agents/`.
- Legacy Claude-only settings may remain under `.claude/`, but they are not a
  Codex configuration surface. If a Claude setting contains durable project
  guidance, translate it into this file or `.agents/`.

## Before Committing (avoid CI failures)

CI fails on lint/format/compile issues far more often than on test logic. To
catch those locally without running the whole suite:

- **Always run `just check` clean before committing.** It is the fast gate that
  mirrors CI's blocking lint/compile checks: `check_release_versions.py`,
  `cargo fmt --all -- --check`, `cargo metadata --locked`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and
  the JS prettier/eslint style check. If it is not clean, do not commit.
- Run `just check-rust` or `just check-style` alone when a change is purely
  Rust or purely JS and you want a faster loop. `just check` runs both.
- Clippy in CI uses `--workspace --all-targets --all-features`, so a warning
  in a test or feature-gated module of *any* workspace member fails CI even
  though a plain `cargo build` passes. Without `--workspace`, cargo only lints
  the root `subtr-actor` package, silently skipping every other member. The
  `just clippy` / `just fmt-check` recipes now use the same flags as CI — bare
  `cargo clippy` / `cargo fmt --check` do not, so prefer the `just` recipes.
- When you touch JS/TS, or any Rust type that is exported via `ts-rs`, also run
  `just check-types` before committing. CI regenerates the TS bindings and
  fails on any drift, including formatting-only diffs, so stale or hand-edited
  generated types under `js/*/src/generated/` are a common failure mode.
  Regenerate them with the curated `npm run generate:raw-types` (player) or
  `npm run generate:stats-types` (stats player), then re-run `just check-types`
  and commit the exact generated output.
- When you change exported event/state structs or stats timeline data that may
  flow through the BakkesMod live-plugin ABI, also run
  `cargo test -p subtr-actor-bakkesmod --no-run` before committing. This catches
  missed mirror updates in `bakkesmod/rust/src/lib_tests/abi_layout.rs` and other
  BakkesMod compile-time fixtures without requiring the full Windows DLL build.
- `just check` deliberately omits the slow CI jobs (`cargo test`, the release
  build, JS bundling, the binding-regen step). Run those targeted at what you
  changed — e.g. `cargo test module_name` — rather than the full suite.

## Common Commands

- Rust formatting generally uses `cargo fmt`.
- For Rust tests, default to targeted commands for the specific behavior under
  investigation, such as `cargo test path::to::test_name` or
  `cargo test module_name`. Do not run the entire local `cargo test` suite by
  default; only run the whole suite when there is a concrete reason, such as a
  broad cross-cutting change or an explicit user request.
- Rust build and maintenance commands should use `cargo ...`; `cargo clean`
  is acceptable when stale build artifacts are the issue.
- JavaScript package work under `js/` commonly uses `npm install`,
  `npm run build`, `npm run dev`, `npm run pack`, `npm pack`, and
  `npm publish`.
- WASM binding builds commonly use `wasm-pack build`.
- Prefer `rg` for text search. Use `grep` only when matching an existing
  script or when `rg` is unavailable.
