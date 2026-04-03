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

## What lives where

### `src/stats/calculators/`

This layer owns the domain logic.

- Shared frame-level inputs such as `FrameInput`, `FrameInfo`,
  `GameplayState`, `BallFrameState`, `PlayerFrameState`, and
  `FrameEventsState` live in the top-level calculator modules.
- Per-stat files such as `pressure.rs`, `rush.rs`, `positioning.rs`, and
  `boost.rs` define the calculators and their stat/event/state types.
- Some files expose intermediate state calculators rather than exported stats,
  for example `touch_state.rs`, `possession_state.rs`, and
  `fifty_fifty_state.rs`.

Rule of thumb: if the change is about stat semantics, thresholds, event
generation, counters, or owned state, it usually belongs in a calculator.

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
  (`src/collector/stats_timeline.rs`, `src/collector/stats/`, or
  `src/stats/export/`).

## Legacy note

`src/stats/reducers/analysis.rs` contains the older derived-signal graph. It is
helpful for historical context, but the analysis-node graph is the cleaner
structure to follow for new dependency-driven stat work.
