# Changelog

This is a rough changelog derived from git tags and commit history. It focuses on
notable user-visible or maintenance-relevant changes rather than every formatting,
README, or refactor-only commit.

## Unreleased

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
