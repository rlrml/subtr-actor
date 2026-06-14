# subtr-actor

[![Workflow Status](https://github.com/rlrml/subtr-actor/workflows/main/badge.svg)](https://github.com/rlrml/subtr-actor/actions?query=workflow%3A%22main%22) [![Docs.rs](https://docs.rs/subtr-actor/badge.svg)](https://docs.rs/subtr-actor) [![Unreleased commits](https://img.shields.io/github/commits-since/rlrml/subtr-actor/latest?style=flat-square&label=unreleased)](https://github.com/rlrml/subtr-actor/releases) ![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

[![Rust version](https://img.shields.io/crates/v/subtr-actor.svg?style=flat-square&label=rust)](https://crates.io/crates/subtr-actor) [![Python version](https://img.shields.io/pypi/v/subtr-actor-py?style=flat-square&label=python)](https://pypi.org/project/subtr-actor-py/) [![JS bindings version](https://img.shields.io/npm/v/%40rlrml%2Fsubtr-actor?style=flat-square&label=js%20bindings)](https://www.npmjs.com/package/@rlrml/subtr-actor) [![JS player version](https://img.shields.io/npm/v/%40rlrml%2Fplayer?style=flat-square&label=js%20player)](https://www.npmjs.com/package/@rlrml/player) [![JS stats player version](https://img.shields.io/npm/v/%40rlrml%2Fstats-player?style=flat-square&label=js%20stats%20player)](https://www.npmjs.com/package/@rlrml/stats-player)

> ▶ **[Try the live stats player](https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/problematic-private-duel-2026-03-20.replay)** — watch a real replay play back and stats accumulate frame-by-frame, right in your browser.

<!-- The section below is generated from the crate-level docs in `src/lib.rs`
     by `cargo rdme`. Do not edit it by hand: run `just readme` to regenerate,
     and `just check` verifies it stays in sync. -->
<!-- cargo-rdme start -->

`subtr-actor` turns raw [`boxcars`](https://docs.rs/boxcars) replay data into
higher-level game state, derived replay events, structured frame payloads, and
dense numeric features for analytics and ML workflows.

- **Higher-level game state** modeled from the raw actor graph
- **Frame-by-frame structured data** ready for JSON export and playback UIs
- **Dense numeric feature matrices** for ML, built from a string-addressable
  feature registry
- **Derived events and cumulative stats** — touches, boost pickups, dodge
  refreshes, goals, demolishes, and more
- **One pipeline, three languages** — the same Rust core drives the Python and
  JavaScript/WASM bindings

## Processing model

- `ReplayProcessor` walks the replay's network frames, models actor state,
  and tracks derived replay events such as touches, boost pad pickups,
  dodge refreshes, goals, player stat events, and demolishes.
- `Collector` is the core extension point. Collectors observe the replay
  frame by frame and can either process every frame or control sampling via
  `TimeAdvance`.
- `ReplayProcessor::process_all` lets multiple collectors share a single
  replay pass when you want to build several outputs at once.
- `FrameRateDecorator` and `CallbackCollector` provide lightweight
  utilities for downsampling a collector or attaching side-effectful hooks
  such as progress reporting and debugging.

## Primary output layers

- `ReplayDataCollector` builds a serde-friendly replay payload with frame
  data, replay metadata, and derived event streams suitable for JSON export
  and playback UIs.
- `NDArrayCollector` emits a dense `ndarray::Array2` with replay
  metadata and headers. It supports both explicit feature adders and the
  string-based registry exposed through `NDArrayCollector::from_strings`
  and `NDArrayCollector::from_strings_typed`.
- `StatsCollector` accumulates graph-backed replay statistics as a
  module-keyed dynamic payload suitable for builtin module selection and
  JSON export.
- `StatsTimelineEventCollector` accumulates graph-backed replay statistics
  as event streams plus lightweight frame scaffolding. This is the preferred
  timeline export when callers do not need to serialize full per-frame
  partial sums.
- `StatsTimelineCollector` preserves the legacy full snapshot timeline
  form (`ReplayStatsTimeline`) for parity checks and compatibility.

## Stats and exports

The `stats` module houses analysis calculators, graph nodes, stat
event calculators, and the exported stat-field model built around
`ExportedStat`.

## Examples

### Collect structured replay data

```rust
use boxcars::ParserBuilder;
use subtr_actor::ReplayDataCollector;

let bytes = std::fs::read("replay.replay").unwrap();
let replay = ParserBuilder::new(&bytes)
    .must_parse_network_data()
    .on_error_check_crc()
    .parse()
    .unwrap();

let replay_data = ReplayDataCollector::new().get_replay_data(&replay).unwrap();
println!("frames: {}", replay_data.frame_data.frame_count());
println!("touches: {}", replay_data.touch_events.len());
```

### Build a sampled feature matrix

```rust
use boxcars::ParserBuilder;
use subtr_actor::{Collector, FrameRateDecorator, NDArrayCollector};

let bytes = std::fs::read("replay.replay").unwrap();
let replay = ParserBuilder::new(&bytes)
    .must_parse_network_data()
    .on_error_check_crc()
    .parse()
    .unwrap();

let mut collector = NDArrayCollector::<f32>::from_strings(
    &["BallRigidBody", "CurrentTime"],
    &["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"],
)
.unwrap();

FrameRateDecorator::new_from_fps(30.0, &mut collector)
    .process_replay(&replay)
    .unwrap();

let (meta, features) = collector.get_meta_and_ndarray().unwrap();
println!("players: {}", meta.replay_meta.player_count());
println!("shape: {:?}", features.raw_dim());
```

### Export compact event-backed stats timeline

```rust
use boxcars::ParserBuilder;
use subtr_actor::StatsTimelineEventCollector;

let bytes = std::fs::read("replay.replay").unwrap();
let replay = ParserBuilder::new(&bytes)
    .must_parse_network_data()
    .on_error_check_crc()
    .parse()
    .unwrap();

let timeline = StatsTimelineEventCollector::new()
    .get_replay_stats_timeline_scaffold(&replay)
    .unwrap();

println!("timeline frames: {}", timeline.frames.len());
let rush_events = timeline
    .events
    .events
    .iter()
    .filter(|event| event.meta.stream == "rush")
    .count();
println!("rush events: {rush_events}");
```

<!-- cargo-rdme end -->

## Packages

- Rust: [`subtr-actor`](https://crates.io/crates/subtr-actor)
- Python: [`subtr-actor-py`](https://pypi.org/project/subtr-actor-py/)
- JavaScript / WASM bindings: [`@rlrml/subtr-actor`](https://www.npmjs.com/package/@rlrml/subtr-actor)
- JavaScript replay player: [`@rlrml/player`](https://www.npmjs.com/package/@rlrml/player)
- JavaScript stats player: [`@rlrml/stats-player`](https://www.npmjs.com/package/@rlrml/stats-player) (see the [live demo](https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/problematic-private-duel-2026-03-20.replay) above)

## Installation

### Rust

```bash
cargo add subtr-actor
```

### Python

```bash
pip install subtr-actor-py
# or, with uv:
uv add subtr-actor-py
# or, with Poetry:
poetry add subtr-actor-py
```

### JavaScript

```bash
npm install @rlrml/subtr-actor
```

### JavaScript player

```bash
npm install @rlrml/player three
```

## Using the bindings

The Rust examples above carry over to the bindings: you choose feature adders by
name and get back replay metadata plus a numeric array. `PlayerBoost` is exposed
in raw replay units (`0-255`), not a percentage.

### Python

```python
import subtr_actor

meta, ndarray = subtr_actor.get_ndarray_with_info_from_replay_filepath(
    "example.replay",
    global_feature_adders=["BallRigidBody", "SecondsRemaining"],
    player_feature_adders=["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"],
    fps=10.0,
    dtype="float32",
)

print(ndarray.shape)
print(meta["column_headers"]["player_headers"][:5])
```

### JavaScript

```javascript
import init, {
  get_ndarray_with_info,
  validate_replay,
} from "@rlrml/subtr-actor";

await init();

const replayData = new Uint8Array(
  await fetch("example.replay").then((response) => response.arrayBuffer()),
);

const validation = validate_replay(replayData);
if (!validation.valid) {
  throw new Error(validation.error ?? "Replay is not valid");
}

const result = get_ndarray_with_info(
  replayData,
  ["BallRigidBody", "SecondsRemaining"],
  ["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"],
  10.0,
);

console.log(result.shape);
console.log(result.metadata.column_headers.player_headers.slice(0, 5));
```

### Common feature names

These string identifiers select feature adders through the Python and JavaScript
bindings:

- Global: `BallRigidBody`, `CurrentTime`, `SecondsRemaining`
- Player: `PlayerRigidBody`, `PlayerBoost`, `PlayerAnyJump`, `PlayerJump`, `PlayerEvent:touch`

Analysis-backed player event indicators use `PlayerEvent:<event_name>` and emit
`1` for a sampled frame when that player has a new event, otherwise `0`.

## Documentation

- Rust API docs: <https://docs.rs/subtr-actor>
- Changelog: [CHANGELOG.md](./CHANGELOG.md)
- Python package README: [python/PYTHON-README.md](./python/PYTHON-README.md)
- JavaScript package README: [js/README.md](./js/README.md)
- JavaScript player README: [js/player/README.md](./js/player/README.md)
- Stat definitions: [docs/event-definitions.md](./docs/event-definitions.md)
- Statistic confidence: [docs/stat-confidence.md](./docs/stat-confidence.md)
- Release notes and process: [docs/RELEASING.md](./docs/RELEASING.md)

## Development

```bash
just build
just test
just fmt
just clippy
just check   # fast lint/format/compile gate — run clean before committing
```

These `just` recipes enter the flake dev shell, so they use the Rust toolchain
from `nix develop` instead of any older `cargo`/`rustc` on your ambient `PATH`.

Bindings:

```bash
just build-python
just build-js
```

`just build-js` builds the repo-local bundler target into `js/pkg`. To build the web-target package that matches `npm publish`, run `npm --prefix js install` once and then `npm --prefix js run build`.

The crate-level docs in `src/lib.rs` are the source of truth for the overview
section above. Run `just readme` after editing them to regenerate this file;
`just check` fails if the two drift apart.

## License

MIT
