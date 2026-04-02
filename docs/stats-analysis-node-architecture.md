# Stats Analysis Node Architecture

## Goal

This document captured the migration away from the old `StatsSample` /
`StatsReducer` / `StatsModule` architecture toward a simpler model:

- one runtime DAG of stateful analysis nodes
- one projection layer for exporting current node state
- one place where dependency solving and type erasure live

The reducer layer has since been removed. Historical mentions of those types
below are only useful as migration context.

## Problems With The Current Design

The current system has several concepts that overlap in confusing ways:

- `StatsSample` is not really a "sample"; it is a per-frame view derived from
  `ReplayProcessor`.
- `StatsReducer` is the real runtime abstraction, but `StatsModule` is also a
  reducer, so the distinction between "compute state" and "export state" is
  blurry.
- `StatsCollector` owns pipeline state such as incremental event cursors and
  live-play tracking, which makes the collector responsible for more than
  orchestration.
- There is both a stats-module dependency mechanism and a derived-signal DAG,
  but the long-term design only really needs one node DAG for stats analysis.

The result is a system that works, but is harder to reason about and harder to
extend cleanly.

## Target Model

### 1. Analysis Graph

The runtime should be a DAG of analysis nodes.

Each analysis node:

- provides exactly one state type `S`
- may depend on zero or more other state types `T0..Tn`
- updates itself from the current input plus dependency state
- owns its internal state across frames

There can be only one active node providing a given state type.

### 2. Input Is Generic

The graph input should be generic over `I`.

That lets the runtime stay agnostic about whether the input is:

- a normalized frame view
- raw processor-derived state
- a testing input

This also means "processor state" does not need special treatment. If we want a
particular projection of processor state, we can model it as a normal root node
with no node dependencies.

### 3. Exporters Are Separate

Modules should become exporters or projections, not runtime state machines.

An exporter:

- has a stable external name
- declares which node state types it needs
- reads current node state and emits playback JSON, typed snapshots, config, or
  dynamic output

Exporters do not participate in dependency solving. They simply select the set
of node states required for output.

### 4. One Typed API, One Erased Runtime

Nodes should declare dependencies by state type, but the runtime will need
internal type erasure to support dependency solving at runtime.

The right compromise is:

- typed at the node API boundary
- type-erased inside the runtime graph

In practice this means nodes use a typed lookup API:

```rust
let touch = ctx.get::<TouchState>()?;
```

while the runtime uses a `TypeId -> &dyn Any` typemap internally.

The cast exists exactly once in the central context lookup path instead of being
spread throughout node implementations.

## Customization And Default Providers

Nodes often need configurable behavior. The graph should support two ways to
get a dependency:

- explicitly provide a node instance
- let the dependency spec provide a default node factory

That yields a simple override model:

- default behavior: dependency solver instantiates missing providers
- custom behavior: caller passes an explicit node instance for the state type
- duplicate providers for the same state type: error

This allows configurable nodes without adding a separate module DAG or a second
configuration registry.

## Example Shape

```rust
trait AnalysisNode<I> {
    type State: 'static;

    fn name(&self) -> &'static str;
    fn dependencies(&self) -> Vec<AnalysisDependency<I>>;
    fn on_input(&mut self, input: &I, ctx: &AnalysisStateContext<'_>) -> Result<()>;
    fn state(&self) -> &Self::State;
}
```

```rust
struct AnalysisDependency<I> {
    state_type: TypeId,
    default_factory: Option<fn() -> Box<dyn AnalysisNodeDyn<I>>>,
}
```

```rust
impl PossessionNode {
    fn dependencies(&self) -> Vec<AnalysisDependency<FrameView>> {
        vec![
            AnalysisDependency::with_default::<TouchState>(|| {
                Box::new(TouchNode::default())
            }),
        ]
    }
}
```

## Where Existing Concepts Fit

### `StatsSample`

The current `StatsSample` concept should survive only as a frame input/view
layer, likely renamed to something like `FrameView`.

It is still useful to build a normalized per-frame view once instead of making
every node query `ReplayProcessor` independently.

### Derived Signals

The current derived-signal DAG is very close to the long-term analysis-node
model. Long term, these should either:

- become ordinary analysis nodes, or
- share the same runtime substrate as analysis nodes

Either way, we should converge toward one dependency runtime instead of two
parallel concepts.

### Current Collectors

The collector should eventually become a thin orchestrator:

1. build the current input
2. evaluate the node graph
3. ask exporters/sinks for the output snapshot

It should not own fine-grained domain state such as event cursors or live-play
classification directly.

## Migration Plan

### Phase 1: Introduce the new runtime scaffold

- Add a generic analysis graph with:
  - typed dependency declarations
  - runtime dependency solving
  - centralized typed state lookup backed by `Any`
- Keep it isolated from existing collectors at first.

### Phase 2: Migrate derived-signal-style logic

- Move or mirror stateful derived computations into analysis nodes.
- Prove that explicit nodes and default nodes can coexist cleanly.

### Phase 3: Split exporters from computation

- Stop making modules implement reducer/runtime behavior directly.
- Convert modules into exporter/projection objects over current node state.

### Phase 4: Rebuild the stats collector around the graph

- Collector builds the current input.
- Graph evaluates.
- Exporters build playback/timeline outputs.

### Phase 5: Remove the old abstractions

- deprecate `StatsModule: StatsReducer`
- rename `StatsSample`
- collapse overlapping DAG concepts

## Non-Goals For The First Refactor

- Rewriting every existing reducer/node immediately
- Eliminating all type erasure
- Solving arbitrary multi-instance node provisioning

The first refactor only needs to establish a clean dependency model and typed
state lookup. We can migrate the existing runtime incrementally after that.
