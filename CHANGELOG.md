# Changelog

This is a rough changelog derived from git tags and commit history. It focuses on
notable user-visible or maintenance-relevant changes rather than every formatting,
README, or refactor-only commit.

## v0.10.0 - 2026-05-26

- Add a BakkesMod plugin pipeline that feeds live Rocket League state through
  the shared subtr-actor analysis graph and exposes timeline mechanics in-game.
- Add replay-backed annotation playback for Rocket League replay viewing through
  the normal replay processing path.
- Add Linux-based BakkesMod DLL builds, artifact verification, and CI coverage
  for the plugin and Rust C ABI.
- Introduce the shared processor view surface used by both replay processing and
  live BakkesMod samples.
- Refresh Rust, Python, and JavaScript release metadata to `0.10.0`.

## v0.9.2 - 2026-05-26

- Remove the broken Linux aarch64 cross-wheel job from Python releases so PyPI
  publishing is not blocked after moving away from the third-party action wrapper.
- Keep the GitHub release workflow green when GitHub rejects the default Actions
  token with the transient account-suspended response.
- Refresh Rust, Python, and JavaScript release metadata to `0.9.2`.

## v0.9.1 - 2026-05-26

- Treat demolition timeline events as authoritative in the replay player so
  demoed cars, boost trails, boost meters, and attached cameras disappear during
  the respawn window even when replay frames still contain stale player samples.
- Keep boost removed by demolitions from counting as boost usage while still
  accounting for the later demo respawn boost grant.
- Serve mechanics review playlists through the stats player and route the
  GitHub Pages `/review/` build to that shared app.
- Use unauthenticated public git fetches for GitHub Actions checkout steps so
  CI and release jobs are not blocked when the default Actions token cannot
  fetch the public repository.
- Remove Rust cache action setup from CI to keep jobs moving when third-party
  action archive downloads fail.
- Build Python wheels and publish artifacts with direct `maturin` and `uv`
  commands instead of release helper actions.
- Refresh Rust, Python, and JavaScript release metadata to `0.9.1`.

## v0.8.16 - 2026-05-26

- Treat demolition timeline events as authoritative in the replay player so
  demoed cars, boost trails, boost meters, and attached cameras disappear during
  the respawn window even when replay frames still contain stale player samples.
- Keep boost removed by demolitions from counting as boost usage while still
  accounting for the later demo respawn boost grant.
- Include passing-goal tags in legacy stats timeline conversion so sampled
  timeline collectors agree with the direct event timeline.
- Serve mechanics review playlists through the stats player and route the
  GitHub Pages `/review/` build to that shared app.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.16`.

## v0.8.15 - 2026-05-26

- Convert stats timeline output to the event-backed transfer path.
- Project rush, whiff, and labeled stat observations through the timeline
  snapshot derivation pipeline.
- Keep lazy stat derivation out of replay load progress reporting.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.15`.

## v0.8.14 - 2026-05-26

- Consolidate mechanic timeline sources so event windows include all configured
  mechanics while avoiding duplicate timeline spans.
- Ignore center detections that later resolve as shots or goals.
- Require initiator slowdown before crediting inferred bump events.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.14`.

## v0.8.13 - 2026-05-26

- Smoothly interpolate replay ball and car rotations between frames.
- Resolve mechanics-review absolute replay paths against remote manifest origins
  when hosted review bundles point at non-local assets.
- Throttle stats-player snapshot UI updates during playback.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.13`.

## v0.8.12 - 2026-05-26

- Add a dedicated replay-loading window to the mechanics review player.
- Show active, pending, loaded, and failed replay preload state outside the main
  mechanics review playlist.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.12`.

## v0.8.11 - 2026-05-26

- Improve wall-to-air setup detection for air-based mechanics.
- Refresh dependency locks for the current Rust and JavaScript dependency set.
- Format the timeline marker test so the player package CI style check passes.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.11`.

## v0.8.10 - 2026-05-25

- Suppress bump stats around contested fifty-fifties to reduce false positive
  bump attribution.
- Require dodge setup evidence for half-volley classification.
- Refine stats-player timeline event markers.
- Deduplicate replay fixtures and update tests/docs to use the canonical
  replay-format assets.
- Format the replay URL test so the player package CI style check passes.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.10`.

## v0.8.9 - 2026-05-25

- Remove the manually refreshed GitHub Pages npm dependency hash by deriving
  Pages build dependencies from the stats-player `package-lock.json`.
- Read the Nix JavaScript package versions from the Cargo workspace version so
  release bumps do not leave Pages package metadata stale.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.9`.

## v0.8.8 - 2026-05-25

- Move the compact stats evaluation scoreboard into the same top HUD row as the
  player boost chips, centered between blue and orange.
- Reserve center spacing in the player HUD so the score does not overlap the
  player chips.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.8`.

## v0.8.7 - 2026-05-25

- Simplify the stats evaluation player scoreboard into a compact game score
  strip with team player names on either side.
- Remove the per-player stat columns from the scoreboard overlay to avoid
  overflow and keep detailed player stats in the existing stats windows.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.7`.

## v0.8.6 - 2026-05-25

- Refresh release metadata after the playlist clip seek clamping and Rust
  clippy fixes landed on `master`.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.6`.

## v0.8.5 - 2026-05-25

- Clamp direct replay seeks and state updates against raw replay duration instead
  of skip-filtered playback end time, so clipped playlist items can seek to late
  replay ranges even when skip ranges exist.
- Keep the dedicated mechanic review player from inheriting kickoff skipping.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.5`.

## v0.8.4 - 2026-05-25

- Add the stats evaluation scoreboard to the review UI.
- Add replay binding output for the stats player so replay review flows can use
  the generated package artifacts.
- Distinguish beaten-to-ball whiff attempts in stat event output.
- Tighten mechanic classification by requiring forward dodge acceleration for
  speed flips and presenting dodge refreshes separately from flip resets.
- Hide verbose goal-tag evidence in the stats report by default.
- Fix the GitHub Pages Nix build by refreshing the stats-player npm dependency
  hash after the generated stats-player bindings update.
- Keep the stats-player package smoke install fixture in sync with new timeline
  config fields and wall aerial event collections.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.4`.

## v0.8.3 - 2026-05-25

- Keep mechanics review clips from inheriting kickoff skipping, so kickoff-adjacent
  clip preroll does not jump past the event or stop at the clip boundary.
- Add player-package coverage for playlist clip time bounds, frame bounds, and
  invalid backwards clips.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.3`.

## v0.8.2 - 2026-05-25

- Make mechanics review clip selection start playback immediately after loading
  and avoid re-enforcing the end boundary once the clip is already paused.
- Show clip timing, event timing, preroll, and postroll details in the mechanics
  review panel.
- Add lead-in behavior when cueing timeline events and default noisy event
  playlist sources off.
- Add counter-attack, wall-aerial, and double-tap goal event coverage.
- Fix bot-player handling in replay data exports.
- Fix stats report scrolling inside the review page shell.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.2`.

## v0.8.1 - 2026-05-25

- Fix the GitHub Pages Nix build by refreshing the stats-player Pages package
  metadata and npm dependency hash after the `0.8.0` release.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.1`.

## v0.8.0 - 2026-05-25

- Add a reusable replay review shell to the stats player package with full-page
  Stats and Viewer modes, shared replay bundle loading, and provider-based data
  injection for hosted review flows.
- Move the GitHub Pages stats report into `subtr-actor-stats-player` as a
  reusable `mountStatsReport` surface and make the Pages app a thin review-shell
  host.
- Make goal `Watch` actions switch into the in-page viewer when the report is
  mounted inside the review shell, while preserving standalone report links.
- Refresh Rust, Python, and JavaScript release metadata to `0.8.0`.

## v0.7.13 - 2026-05-25

- Add a stats evaluation player event playlist window with filter controls,
  automatic timeline following, and per-player event colors.
- Expand the stats player events window with lane-separated event sources,
  timeline lane label tooltips, and pass-origin classification.
- Expand touch stats with wall-touch counts and surface-labeled touch
  breakdowns across Rust exports and the stats evaluation player.
- Track dodge state for touch counts.
- Tighten air-dribble detection by separating air-dribble policy from ball carry
  handling and lowering the minimum detected duration.
- Move tools CLI parsing to `clap` and speed up CI replay fixture checks.
- Add touch surface confidence documentation and keep player entrypoint
  formatting lint-clean.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.13`.

## v0.7.12 - 2026-05-25

- Add replay preload state tracking to playlist replay caches and expose queued,
  loading, loaded, and failed replay sources to JavaScript player consumers.
- Show mechanics-review replay preload details in both the standalone review
  player and the stats evaluation player's mechanics review window.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.12`.

## v0.7.11 - 2026-05-24

- Add optional playlist manifest pagination metadata and preserve it through
  the JavaScript replay-player manifest parser and stats review player.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.11`.

## v0.7.10 - 2026-05-24

- Emit consecutive same-player touch candidates without the previous short
  cooldown, preserving rapid touch-state changes for downstream stats.
- Fix own-half goal tagging so team orientation is applied from the scoring
  team perspective.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.10`.

## v0.7.9 - 2026-05-24

- Improve speed-flip detection.
- Update touch-state behavior for downstream mechanics and touch stats.
- Fix the GitHub Pages Nix metadata carried forward from the `0.7.8` release.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.9`.

## v0.7.8 - 2026-05-24

- Collapse the separate dribble-touch bucket into `touch.control_touch_count`
  so controlled ground carries and airborne control touches share one metric.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.8`.

## v0.7.7 - 2026-05-24

- Add inferred bump, rotation, half-volley, goal-tag, goal air-time, and air
  dribble stat coverage, including richer goal watch actions and mechanic
  review decisions in the stats views.
- Add shot metadata to shot stat timeline events and preserve the matching
  JavaScript stats-player normalization.
- Improve stats-player analysis graph rendering and color player charts by
  team.
- Restore CI release metadata and stats timeline fixtures, and refresh the
  GitHub Pages Nix npm dependency hash.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.7`.

## v0.7.6 - 2026-05-23

- Keep demoed players understandable in the replay player by hiding absent car
  meshes while showing a short-lived demo respawn indicator at the victim
  location.
- Mark carried player frame gaps as not present during JavaScript replay
  normalization and include demo victim locations in timeline events.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.6`.

## v0.7.5 - 2026-05-23

- Use the stats `TouchState` attribution stream for goal context, pass,
  backboard-bounce, and double-tap analysis instead of older raw team-touch
  events, fixing stale scorer-touch context on post-EAC replays.
- Prevent stale credited scorer touches from producing own-half goal tags.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.5`.

## v0.7.4 - 2026-05-22

- Add a Goals tab to the stats report page that summarizes goal metadata,
  goal-tag counts, scorer/timing context, tag evidence, and lead-up player
  state from the stats timeline.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.4`.

## v0.7.3 - 2026-05-22

- Fix the stats-player packed-package smoke test fixture so it includes the
  current stats timeline config and event buckets.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.3`.

## v0.7.2 - 2026-05-22

- Keep JavaScript release builds green when npm credentials are unavailable by
  still building and smoke-testing packages while skipping publish steps with an
  explicit warning.
- Make the GitHub Pages Nix build tolerate transient binary-cache substitute
  download failures by allowing source fallback.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.2`.

## v0.7.1 - 2026-05-22

- Fix CI on Rust 1.95 by simplifying the whiff-attempt boolean expression that
  clippy now flags as nonminimal.
- Refresh Rust, Python, and JavaScript release metadata to `0.7.1`.

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
- Add `assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay` and a regression test
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
