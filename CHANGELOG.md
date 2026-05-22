# Changelog

This is a rough changelog derived from git tags and commit history. It focuses on
notable user-visible or maintenance-relevant changes rather than every formatting,
README, or refactor-only commit.

## v0.7.0 - 2026-05-22

- Add stats report page improvements, including grouped team/player views,
  overview pressure bars, and clearer boost tank colors.
- Add goal lead-up context stats, flick stats, ball-advancement touch stats,
  wavedash stats, air-dribble stats, goal tags, one-timer and pass stats, and
  half-flip detection across Rust exports and the stats player.
- Add mechanic review playlist generation/playback support plus mechanics and
  goal overview windows in the stats player.
- Improve replay loading progress ordering and keep stats snapshot conversion
  aligned with graph-backed air-dribble timeline snapshots.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.0`.

## v0.6.6 - 2026-05-13

- Keep old replay player cars visible through bounded gaps in exported player
  frame data by preserving bootstrapped player/car mappings and sleeping player
  positions, then carrying normalized player samples across short unavailable
  spans.
- Add whiff timeline visualization support in the stats player.
- Refresh Rust, Python, and JavaScript release metadata to `0.6.6`.

## v0.6.5 - 2026-05-13

- Disable `wasm-opt` for JavaScript WASM release builds to avoid CI runtime
  failures when initializing the generated externref table.
- Fix the packed stats-player smoke test fixture to include `whiff` timeline
  events.
- Refresh Rust, Python, and JavaScript release metadata to `0.6.5`.

## v0.6.4 - 2026-05-13

- Fix JavaScript CI and release builds by using the installed `wasm-pack`
  binary and installing Binaryen for `wasm-opt`.
- Refresh the GitHub Pages Nix package metadata and npm dependency hash for
  the current release.
- Refresh Rust, Python, and JavaScript release metadata to `0.6.4`.

## v0.6.3 - 2026-05-13

- Fix older replay player substitution handling so late reservation and
  party-leader mappings update player rosters, stats timeline metadata, and
  Ballchasing comparison output instead of leaving placeholder cars at midfield.
- Add a 2016 Ballchasing replay fixture and regression coverage for old replay
  substitutions and PlayStation-style player identity collisions.
- Document the legacy player-identity compatibility behavior discovered from
  the old replay investigation.
- Refresh Rust, Python, and JavaScript release metadata to `0.6.3`.

## v0.6.2 - 2026-05-10

- Improve stats timeline bundle performance by building typed stats timeline
  data directly in the WASM replay bundle.
- Keep player replay normalization from forcing extra progress-yield work.
- Reduce stats timeline payload size by omitting zero-valued labeled timeline
  buckets while preserving semantic comparisons in Rust coverage.
- Make the web package build repeatable and address CI clippy, Pages build, and
  sparse-breakdown test failures from the release candidate path.
- Refresh Rust, Python, and JavaScript release metadata to `0.6.2`.

## v0.6.1 - 2026-05-10

- Add shareable stats player URL configuration.
- Move boost pickup filters into the stats-player window controls and allow
  stat selection before a replay is loaded.
- Refine player and stats-player controls, including the launcher menu,
  compact timeline controls, visualization controls, and boost pad glow assets.
- Stop emitting stale or ghost boost pickup events.
- Rename positioning role badges to depth and remove Rush timeline event
  markers.
- Require release metadata checks to verify that the current version has a
  matching changelog entry before release automation runs.
- Add the 2025 RLCS Worlds grand final Team Falcons vs NRG game 5 replay and
  Ballchasing metadata as a comparison fixture.
- Refresh Rust, Python, and JavaScript release metadata to `0.6.1`.

## v0.6.0 - 2026-05-09

- Rework the stats evaluation player into a compact movable-window UI with
  cleaner overlay controls, grouped timeline effect toggles, editable stats
  windows during playback, and literal-token stat picker search.
- Add configurable follow camera behavior and a reusable canvas recording
  plugin for replay viewport capture.
- Improve boost pickup analysis and visualization, including corrected counts
  near final goals, filterable boost pickup count animation, and visible
  skipped timeline ranges.
- Report progress while building stats snapshots.
- Refresh Rust, Python, and JavaScript release metadata to `0.6.0`.

## v0.5.3 - 2026-05-08

- Fix the stats evaluation player npm smoke-install fixture so it includes the
  required `boost_pickups` stats event bucket.
- Refresh Rust, Python, and JavaScript release metadata to `0.5.3`.

## v0.5.2 - 2026-05-08

- Fix the Nix-powered GitHub Pages build metadata for the stats evaluation
  player after the package lockfile version refresh.
- Refresh Rust, Python, and JavaScript release metadata to `0.5.2`.

## v0.5.1 - 2026-05-08

- Improve the stats evaluation player replay-loading modal with more granular,
  callback-backed progress phases for stats construction, serialization,
  decoding, and replay-model normalization.
- Refresh Rust, Python, and JavaScript release metadata to `0.5.1`.

## v0.5.0 - 2026-04-29

- Relax Cargo dependency version bounds so downstream builds can pick up newer
  `boxcars` releases with EAC replay-format support.
- Add April 2026 post-EAC Ballchasing replay fixtures across duel, doubles,
  standard, and private matches.
- Add post-EAC regression coverage for parsing, structured replay data, ndarray
  features, aggregate stats, typed and dynamic stats timelines, JSON
  serialization, and motion plausibility.

## v0.4.0 - 2026-04-25

- Fix legacy replay rigid-body normalization across position, velocity, and
  rotation format boundaries, with fixture-backed replay-format documentation.
- Add replay plausibility probes and regression coverage for historical replay
  interpretation.
- Flatten checked-in replay fixture assets and add viewer links for visual
  inspection through the GitHub Pages stats player.
- Add URL preloading to the stats player, including readable `replayUrl=`
  links and compressed `r=` replay URL links.
- Improve stats replay loading progress and fixture handling in the viewer.
- Refresh Rust, Python, and JavaScript release metadata to `0.4.0`.

## v0.3.1 - 2026-03-20

- Add backboard hit and double-tap stats to the stats pipeline and player UI.
- Detect speed flips outside kickoff windows and reset speed-flip reducer state
  outside live play.
- Fix stats-player positioning metrics so they derive from accumulated stats.
- Make JavaScript WASM package preparation idempotent and install Node.js in
  GitHub Actions.
- Refresh Rust, Python, and JavaScript release metadata to `0.3.1`.

## v0.3.0 - 2026-03-19

- Add musty flick, speed flip, Rush event, positioning range, and half-control
  stats and timeline visualizations.
- Unify shared stats summary cards and split large player and stats-player
  modules for maintainability.
- Bundle replay-loading progress with stats data and cache timeline projections
  and stat renders for faster stats-player updates.
- Fix kickoff boost average gating, initial boost state initialization, and
  replay-start timeline control placement.
- Move WASM builds to npm-managed `wasm-pack` and refresh release metadata to
  `0.3.0`.

## v0.2.3 - 2026-03-19

- Fix PyPI wheel builds by keeping the `reqwest` and `ring` dependency chain
  out of the Python binding build unless the Ballchasing comparison CLI is
  explicitly enabled.
- Refresh Rust, Python, and JavaScript release metadata to `0.2.3`.

## v0.2.2 - 2026-03-19

- Rename the npm stats viewer package to `subtr-actor-stats-player`.
- Refresh the Pages build metadata after the stats player lockfile update.
- Refresh release metadata to `0.2.2`.

## v0.2.1 - 2026-03-19

- Refactor the stats player into reusable scaffolding, templates, and module
  builders to make the package easier to extend and maintain.
- Fix time-in-zone colors so player-relative lanes render correctly.
- Refresh release metadata to `0.2.1`.

## v0.2.0 - 2026-03-19

- Package `subtr-actor-stats-player` as a publishable npm library
  with a reusable mount API, package metadata, smoke-install checks, and
  README/license assets.
- Wire the new stats player package into CI, npm release automation, and release
  version checks.
- Refresh release metadata to `0.2.0`.

## v0.1.17 - 2026-03-10

- Fix demolition extraction when same-frame replay cleanup clears a car's
  `Engine.Pawn:PlayerReplicationInfo` link to `ActorId(-1)`.
- Continue collecting demolitions from raw frame updates when same-frame actor
  deletion prevents the demolish attribute from entering modeled actor state.
- Fix the `car_to_player` mapping used during demolish resolution so it remains
  keyed by car actor ID rather than player actor ID.
- Preserve the victim's last known rigid body location for same-frame deletions
  instead of falling back to origin.
- Add `assets/new_demolition_format.replay` and a regression test
  that asserts it yields 10 demolition events.

## v0.1.16 - 2026-03-09

- Switch the WASM build target from `bundler` to `web`.
- Rewrite the README.
- Harden release version checks in CI.

## v0.1.15 - 2026-03-06

- Fix WASM `get_replay_frames_data` alignment to match Python bindings.
- Clarify boost units and add percent conversion helpers.
- Add and then harden the auto-release workflow.

## v0.1.14 - 2026-03-05

- Add a crates.io release workflow.

## v0.1.13 - 2026-03-05

- Add PyPI and npm badges.
- Set explicit interpreters for aarch64 wheel builds.

## v0.1.12 - 2026-03-05

- Add typed ndarray export support while preserving `f32` defaults.
- Support both `DemolishFx` and `DemolishExtended` demolition formats.
- Add `car_to_player` for O(1) car-to-player lookups.
- Add multi-collector replay processing and helper methods for player names.
- Make processor helper methods and macros public.
- Improve debug logging and release workflows for Python and JavaScript.
- Fix `BallHasBeenHit` always returning `1` in `NDArrayCollector`.

## v0.1.11 - 2026-01-31

- Add comprehensive replay processing tests and replay fixtures from `boxcars`.
- Add game state feature adders for kickoff detection.
- Bump `boxcars` to `0.10.10` for `ViralItemActor_TA` support.
- Update Python bindings for `pyo3 0.27`.
- Add monorepo dependency management for bindings.

## v0.1.10 / v0.1.9 - 2025-09-09

- Add Python package build and publish scripts.
- Replace the old publish shell script with a `justfile` workflow.
- Fix Python binding build issues and publishing authentication details.
- Update Python bindings and tests.

## v0.1.8 - 2025-07-15

- Add WASM bindings and a JavaScript package/example flow.
- Import Python bindings into the monorepo.
- Add quaternion feature adders and a frame-rate parameter to replay frame data.
- Improve replay data docstrings, examples, doctests, and binding packaging.

## v0.1.6 - 2025-07-14

- Support the newer `ReplicatedBoost` replay format.
- Allow spectator and non-playing actors by relaxing unconditional player-set checks.
- Add CI workflow support for the newer release/binding flow.

## v0.1.5 - 2023-06-17

- Refresh release metadata for macro fixes in the early exported macro support.

## v0.1.4 / v0.1.3 / v0.1.2 - 2023-06

- Stabilize exported macro support and related derive re-exports.
- Improve doctests, examples, README links, and generated documentation.
- Simplify ndarray macro column counting and clean up early public API details.

## v0.1.1 - 2023-06-12

- Initial documented release of `subtr-actor`.
- Add the `ReplayProcessor`, collector abstractions, frame-rate decoration, and
  interpolation support.
- Build out the initial ndarray and replay-data collection pipeline.
- Add early demolition features, player ordering, replay metadata extraction,
  typed errors, and top-level documentation/README content.
