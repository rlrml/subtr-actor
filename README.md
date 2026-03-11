# subtr-actor

[![Workflow Status](https://github.com/rlrml/subtr-actor/workflows/main/badge.svg)](https://github.com/rlrml/subtr-actor/actions?query=workflow%3A%22main%22) [![](https://docs.rs/subtr-actor/badge.svg)](https://docs.rs/subtr-actor) [![Version](https://img.shields.io/crates/v/subtr-actor.svg?style=flat-square)](https://crates.io/crates/subtr-actor) [![PyPI](https://img.shields.io/pypi/v/subtr-actor-py?style=flat-square)](https://pypi.org/project/subtr-actor-py/) [![npm](https://img.shields.io/npm/v/rl-replay-subtr-actor?style=flat-square)](https://www.npmjs.com/package/rl-replay-subtr-actor) ![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

`subtr-actor` turns Rocket League replay files into higher-level data than the raw actor graph exposed by [`boxcars`](https://docs.rs/boxcars/).

It supports two main workflows:

- structured replay data for inspection, export, and analysis
- dense numeric arrays for ML and other downstream pipelines

The core crate is written in Rust, with bindings for Python and JavaScript.

## Packages

- Rust: [`subtr-actor`](https://crates.io/crates/subtr-actor)
- Python: [`subtr-actor-py`](https://pypi.org/project/subtr-actor-py/)
- JavaScript / WASM: [`rl-replay-subtr-actor`](https://www.npmjs.com/package/rl-replay-subtr-actor)

## What You Get

- A higher-level replay model built from `boxcars`
- Frame-by-frame structured game state via `ReplayDataCollector`
- Configurable numeric feature extraction via `NDArrayCollector`
- Frame-rate resampling with `FrameRateDecorator`
- A similar replay-processing model across Rust, Python, and JS

## Installation

### Rust

```toml
[dependencies]
subtr-actor = "0.1.17"
```

### Python

```bash
pip install subtr-actor-py
```

### JavaScript

```bash
npm install rl-replay-subtr-actor
```

## Quick Start

### Rust: get structured replay data

```rust
use boxcars::ParserBuilder;
use subtr_actor::ReplayDataCollector;

fn main() -> anyhow::Result<()> {
    let data = std::fs::read("example.replay")?;
    let replay = ParserBuilder::new(&data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()?;

    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .map_err(|e| e.variant)?;

    println!("{}", replay_data.as_json()?);
    Ok(())
}
```

### Rust: build an ndarray for ML

```rust
use subtr_actor::*;

fn main() -> anyhow::Result<()> {
    let data = std::fs::read("example.replay")?;
    let replay = boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()?;

    let mut collector = NDArrayCollector::new(
        vec![
            InterpolatedBallRigidBodyNoVelocities::arc_new(0.003),
            CurrentTime::arc_new(),
        ],
        vec![
            InterpolatedPlayerRigidBodyNoVelocities::arc_new(0.003),
            PlayerBoost::arc_new(),
            PlayerAnyJump::arc_new(),
        ],
    );

    FrameRateDecorator::new_from_fps(30.0, &mut collector)
        .process_replay(&replay)
        .map_err(|e| e.variant)?;

    let (meta, array) = collector.get_meta_and_ndarray().map_err(|e| e.variant)?;
    println!("rows={} cols={}", array.nrows(), array.ncols());
    println!("players={}", meta.replay_meta.player_stats.len());
    Ok(())
}
```

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
import init, { get_ndarray_with_info, validate_replay } from "rl-replay-subtr-actor";

await init();

const replayData = new Uint8Array(
  await fetch("example.replay").then((response) => response.arrayBuffer())
);

const validation = validate_replay(replayData);
if (!validation.valid) {
  throw new Error(validation.error ?? "Replay is not valid");
}

const result = get_ndarray_with_info(
  replayData,
  ["BallRigidBody", "SecondsRemaining"],
  ["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"],
  10.0
);

console.log(result.shape);
console.log(result.metadata.column_headers.player_headers.slice(0, 5));
```

## Core Concepts

### `ReplayDataCollector`

Use this when you want a serializable, frame-by-frame representation of the replay without dealing directly with the low-level actor graph.

### `NDArrayCollector`

Use this when you want numeric features in a 2D matrix. In Rust you construct feature adders directly; in the Python and JS bindings you provide feature-adder names as strings.

### `FrameRateDecorator`

Use this to resample replay processing to a fixed FPS before collecting data.

## Common Feature Names

These are useful when working through the Python or JavaScript bindings:

- Global: `BallRigidBody`, `CurrentTime`, `SecondsRemaining`
- Player: `PlayerRigidBody`, `PlayerBoost`, `PlayerAnyJump`, `PlayerDoubleJump`

`PlayerBoost` is exposed in raw replay units (`0-255`), not percentage.

## Documentation

- Rust API docs: <https://docs.rs/subtr-actor>
- Project docs: [docs/README.md](./docs/README.md)
- Changelog: [docs/CHANGELOG.md](./docs/CHANGELOG.md)
- Python package README: [python/README.md](./python/README.md)
- JavaScript package README: [js/README.md](./js/README.md)
- Example app: [js/example/README.md](./js/example/README.md)
- Release notes and process: [docs/RELEASING.md](./docs/RELEASING.md)

## Development

```bash
just build
just test
just fmt
just clippy
```

Bindings:

```bash
just build-python
just build-js
```

`just build-js` builds the repo-local bundler target into `js/pkg`. To build the web-target package that matches `npm publish`, run `npm --prefix js install` once and then `npm --prefix js run build`.

## License

MIT
