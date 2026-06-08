# Calculators and Analysis Nodes

This is the short map of the current stats runtime.

## Core split

- `src/stats/calculators/` holds the actual stat logic and state machines.
- `src/stats/analysis_graph/` wraps calculator logic in the DAG runtime used by
  the newer stats timeline/export flow.
- `src/stats/reducers/` is the older reducer pipeline. It still matters, but if
  you are working on the analysis-node graph, treat calculators plus
  `analysis_graph/` as the primary structure.

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
- Per-stat files such as `pressure.rs`, `rush.rs`, `positioning.rs`, and
  `boost.rs` define calculators and the event/state types needed to detect
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

- `analysis_graph.rs` defines the runtime DAG, typed dependency lookup, default
  dependency factories, topological sorting, and graph evaluation.
- `nodes.rs` defines common dependency helpers for shared frame-derived state.
- Files like `positioning.rs`, `pressure.rs`, and `rush.rs` usually contain a
  thin node wrapper around a calculator.
- Files like `frame_info.rs`, `frame_events_state.rs`, `player_frame_state.rs`,
  and `live_play.rs` provide graph state that other nodes depend on.
- `mod.rs` is the registry for built-in node names and graph construction.

Rule of thumb: if the change is about wiring, dependency declarations, graph
inputs, default providers, or exposing calculator state through the graph, it
belongs here.

## Runtime shape

The current flow is:

1. `AnalysisNodeCollector` builds a `FrameInput` from `ReplayProcessor`.
2. `AnalysisGraph` resolves node dependencies and evaluates nodes in dependency
   order.
3. Each node pulls the states it needs from `AnalysisStateContext`.
4. Most stat nodes call into a calculator and then expose the calculator itself
   as the node state.
5. Collectors/export code read calculator state back out of the graph.

That means a node often looks like:

- declare dependencies
- fetch shared frame/intermediate state from the context
- call `calculator.update(...)`
- return `&calculator` as the node state

## Naming pattern

There are two common node shapes:

- Intermediate state nodes: `TouchStateNode -> TouchState`,
  `PossessionStateNode -> PossessionState`, `LivePlayNode -> LivePlayState`
- Stat nodes: `PressureNode -> PressureCalculator`,
  `PositioningNode -> PositioningCalculator`,
  `MatchStatsNode -> MatchStatsCalculator`

For exported stats, the node state is often the calculator itself because later
code wants the calculator's accumulated stats, config, and event lists.

## Adding or changing a stat

- Add or modify the core logic in `src/stats/calculators/<stat>.rs`.
- If the stat participates in the DAG runtime, add or update the matching node
  in `src/stats/analysis_graph/<stat>.rs`.
- Register the node in `src/stats/analysis_graph/mod.rs`.
- If shared frame-derived state is missing, add that as a dedicated dependency
  node instead of recomputing it inside each stat node.
- If output wiring changes, update the relevant collector or exporter
  (`src/stats/timeline/`, `src/collector/stats/`, or
  `src/stats/export/`).

## Legacy note

`src/stats/reducers/analysis.rs` contains the older derived-signal graph. It is
helpful for historical context, but the analysis-node graph is the cleaner
structure to follow for new dependency-driven stat work.
