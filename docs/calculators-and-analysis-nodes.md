# Calculators and Analysis Nodes

This is the short map of the current stats runtime.

## Core split

- `src/stats/calculators/` holds the actual stat logic and state machines.
- `src/stats/analysis_graph/` wraps calculator logic in the DAG runtime.
- `src/stats/accumulators/` folds observations into report statistics.
- `src/stats/timeline/` owns timeline collection, event projection helpers, and
  the transaction log shared by batch and live consumers.

In practice: calculators know how to compute; nodes know where a calculator fits
in the dependency graph.

## Observation-first stats

For stats that emit meaningful domain facts, prefer making those facts the
canonical internal record and deriving counters from them. A calculator can
still keep private state for detection, such as active candidates, previous
frame samples, pending touch windows, or boost reconciliation state. The public
stats shape should avoid hand-maintaining independent cartesian counters when
the same information is naturally represented as labels on an observation.

The boundary is not "calculators have no state." Span-based and inferred events
usually require state: a calculator may keep the active candidate, previous
sample, lookback buffer, pending reconciliation, or projected in-progress event
needed to decide when a domain event starts, updates, and ends. That state is
part of detection.

The boundary is "calculators do not own report accumulation." Counts, sums,
averages, maxima, compatibility fields, and labeled projections belong in
accumulators whenever they can be derived from emitted events. If calculator
state would be unchanged by removing the final report fields, it is probably
detection state. If the state exists only to answer "what is the current total,"
it belongs in an accumulator.

Use these observation shapes:

- Discrete events: touches, whiffs, rushes, flicks, goals with goal tags, demos.
- Intervals or episodes: powerslides, ball carries, air dribbles, possession
  spans, pressure spans.
- Quantity ledger entries: boost collected, used, stolen, overfilled, or
  respawned.
- Time-weighted samples: boost amount, speed bands, height bands, possession
  state, and other continuous signals integrated over `dt`.

Subcounts should usually be labeled projections over those observations. For
example, a rush count is one stat with labels such as `team=team_zero`,
`attackers=2`, and `defenders=1`, while legacy fields like
`team_zero_two_v_one_count` can remain compatibility projections.

## What lives where

### `src/stats/calculators/`

This layer owns the domain event logic.

- Shared frame-level inputs such as `FrameInput`, `FrameInfo`,
  `GameplayState`, `BallFrameState`, `PlayerFrameState`, and
  `FrameEventsState` live in the top-level calculator modules.
- Per-stat files such as `territorial_pressure.rs`, `rush.rs`, `positioning.rs`,
  and `boost.rs` define calculators and the event/state types needed to detect
  domain observations.
- Some files expose intermediate state calculators rather than exported stats,
  for example `touch_state.rs`, `possession_state.rs`, and
  `fifty_fifty_state.rs`.

Rule of thumb: if the change is about thresholds, event semantics, event
generation, candidate tracking, or frame-to-frame detection state, it usually
belongs in a calculator. If the change is about counting, summing, averaging,
max tracking, or projecting labeled compatibility fields from those events, it
usually belongs in an accumulator.

### `src/stats/analysis_graph/`

This layer adapts calculator/state logic into a typed dependency graph.

- `graph.rs` defines the runtime DAG, typed dependency lookup, default
  dependency factories, topological sorting, and graph evaluation.
- `nodes/mod.rs` defines common dependency helpers for shared frame-derived state.
- Files under `nodes/`, such as `positioning.rs` and `rush.rs`, usually contain a
  thin node wrapper around a calculator.
- Files under `nodes/`, such as `frame_info.rs`, `frame_events_state.rs`,
  `player_frame_state.rs`, and `live_play.rs` provide graph state that other
  nodes depend on.
- `mod.rs` is the registry for built-in node names and graph construction.

Rule of thumb: if the change is about wiring, dependency declarations, graph
inputs, default providers, or exposing calculator state through the graph, it
belongs here.

## Runtime shape

Replay and live sources converge on the same `FrameInput` and analysis graph:

1. `ReplayProcessor` reconstructs replay actor state; `LiveProcessorView` adapts
   live state. Both implement `ProcessorView`.
2. Replay collectors use `ReplayFrameInputBuilder`; live drivers construct a
   `FrameInput` with the host's live-play state.
3. `AnalysisGraph` resolves dependencies and evaluates source and detector nodes
   in topological order. Each node exposes typed state to downstream nodes.
4. Report consumers can include `StatsProjectionNode`, which feeds Rust
   accumulators, and `StatsTimelineFrameNode`, which creates full snapshots.
5. Event consumers use each detector node's `project_events` method. The graph
   diffs their combined projection into its `TimelineTransactionLog`.

`StatsTimelineEventsNode` is an aggregation root: its dependencies select the
producer nodes. Its state is a marker; event data lives in the graph's log.

### Event lifecycle and collection cadence

A projection returns the full current event set, with stable IDs independent of
projection frequency. `Confirmed` events may be revised; `Finalized` events
must stay unchanged. Neither may disappear under the strict invariant policy.
`project_events_now` turns changes into upsert/retract transactions. `finish`
finishes nodes in dependency order and then finalizes the last projection.

Batch collectors can project only at finish. Live drivers project periodically.
Frame evaluation, event projection, and persistence of output snapshots are
separate cadences: lowering output frequency must not skip detector input.
Full-history projection still revisits historical events, so lowering projection
frequency reduces cost without changing its asymptotic growth.

### Compact playback and compatibility snapshots

`StatsTimelineEventCollector` exports a frame scaffold plus events and auxiliary
tracks. `js/stat-evaluation-player/src/*EventDerivation.ts` materializes report
statistics from those events. `StatsTimelineCollector` retains the Rust
full-snapshot path for compatibility and parity comparisons. These are two
projections of the same domain observations; changes to accumulation semantics
need Rust/TypeScript parity coverage.

`StatsCollector` captures the same canonical graph events in
`CapturedStatsData.events`; its legacy conversions reuse them directly. Module
JSON remains the report/snapshot representation, rather than an event source.
Custom module selections also include events emitted by graph dependencies.
Callers constructing `CapturedStatsData` directly must supply its typed `events`
field; complete replay collection populates and finalizes it automatically.

## Naming pattern

There are two common node shapes:

- Intermediate state nodes: `TouchStateNode -> TouchState`,
  `PossessionStateNode -> PossessionState`, `LivePlayNode -> LivePlayState`
- Stat nodes: `TerritorialPressureNode -> TerritorialPressureCalculator`,
  `PositioningNode -> PositioningCalculator`,
  `MatchStatsNode -> MatchStatsCalculator`

Detector nodes often publish their calculator so downstream nodes can read
events, configuration, and detection state. Report totals belong in
accumulators and are exposed through `StatsProjectionState`.

## Adding or changing a stat

- Add or modify the core logic in `src/stats/calculators/<stat>.rs`.
- If the stat participates in the DAG runtime, add or update the matching node
  in `src/stats/analysis_graph/nodes/<stat>.rs`.
- Register the node in `src/stats/analysis_graph/mod.rs`.
- If shared frame-derived state is missing, add that as a dedicated dependency
  node instead of recomputing it inside each stat node.
- If output wiring changes, update the relevant collector
  (`src/stats/timeline/` or `src/collector/stats/`).

## Compatibility note

The legacy path is the full-snapshot timeline collector and its projection
nodes. There is no `src/stats/reducers/` pipeline in the current tree. Keep the
legacy collector while it serves external callers and verifies compact playback
parity; remove it only with an explicit compatibility migration.
