# rl-replay-subtr-actor

WebAssembly bindings for [subtr-actor](https://crates.io/crates/subtr-actor), a Rocket League replay processing library.

This package mirrors the Python binding at a JavaScript/TypeScript-friendly boundary: pass replay bytes in, get structured data or ndarray-like output back.

## Installation

```bash
npm install rl-replay-subtr-actor
```

## Runtime Support

The published npm package is the web-target ESM build produced by `wasm-pack`. It is the right fit for browsers and bundlers.

If you need a Node-specific build from the repository, build it yourself with:

```bash
npm --prefix js install
npm --prefix js run build:nodejs
```

That generates `js/pkg-node/`.

## Usage

### Browser / bundler

```javascript
import init, {
  get_column_headers,
  get_ndarray_with_info,
  get_replay_frames_data,
  get_replay_info,
  get_replay_meta,
  validate_replay,
} from "rl-replay-subtr-actor";

await init();

const replayData = new Uint8Array(
  await fetch("/example.replay").then((response) => response.arrayBuffer())
);

const validation = validate_replay(replayData);
if (!validation.valid) {
  throw new Error(validation.error ?? "Replay is not valid");
}

const info = get_replay_info(replayData);
const headers = get_column_headers(
  ["BallRigidBody", "SecondsRemaining"],
  ["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"]
);
const ndarrayResult = get_ndarray_with_info(
  replayData,
  ["BallRigidBody", "SecondsRemaining"],
  ["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"],
  10.0
);
const metadata = get_replay_meta(replayData);
const frameData = get_replay_frames_data(replayData);

console.log(info);
console.log(headers.player_headers.slice(0, 5));
console.log(ndarrayResult.shape);
console.log(metadata.replay_meta.player_stats.length);
console.log(frameData.meta.map_name);
```

## API Surface

### `validate_replay(data: Uint8Array)`

Validate that replay bytes can be parsed successfully.

Return shape:

```javascript
{ valid: true, message: "Replay is valid" }
```

or:

```javascript
{ valid: false, error: "..." }
```

### `get_replay_info(data: Uint8Array)`

Return lightweight replay metadata including version numbers and property counts.

### `parse_replay(data: Uint8Array)`

Parse raw replay bytes and return the full replay structure as plain JS data.

### `get_ndarray_with_info(data, globalFeatureAdders?, playerFeatureAdders?, fps?)`

Return numerical replay data suitable for analysis or ML.

Defaults:

- global features: `["BallRigidBody"]`
- player features: `["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"]`
- FPS: `10.0`

Return shape:

```javascript
{
  metadata: {
    replay_meta: { /* replay metadata */ },
    column_headers: {
      global_headers: string[],
      player_headers: string[],
    },
  },
  array_data: number[][],
  shape: number[],
}
```

### `get_replay_meta(data, globalFeatureAdders?, playerFeatureAdders?)`

Return replay metadata plus column headers without building the full ndarray.

### `get_column_headers(globalFeatureAdders?, playerFeatureAdders?)`

Return only the ndarray header layout for a given feature configuration.

### `get_replay_frames_data(data)`

Return structured frame-by-frame data from `ReplayDataCollector`. This path does not do FPS resampling.

## Common Feature Names

See the [subtr-actor ndarray docs](https://docs.rs/subtr-actor/latest/subtr_actor/collector/ndarray/index.html) for the full list.

Common global features:

- `"BallRigidBody"`
- `"CurrentTime"`
- `"SecondsRemaining"`

Common player features:

- `"PlayerRigidBody"`
- `"PlayerBoost"`
- `"PlayerAnyJump"`
- `"PlayerJump"`
- `"PlayerDodgeRefreshed"`

`"PlayerBoost"` is exposed in raw replay units (`0-255`), not percentage.

## Building from Source

Requirements:

- Rust toolchain
- `wasm-pack`
- `just`
- `npm`

Repository-local bundler build:

```bash
just build-js
```

Publishable web build from `js/package.json`:

```bash
npm --prefix js install
npm --prefix js run build
```

Other package targets:

```bash
npm --prefix js run build:nodejs
npm --prefix js run build:bundler
```

Tests:

```bash
npm --prefix js test
```

The example app under [`js/example`](./example/README.md) uses the web target and expects `js/pkg/` to be built with `wasm-pack build --target web --out-dir pkg`.

## Publishing Notes

This binding depends on the workspace crate via:

```toml
[dependencies.subtr-actor]
path = ".."
version = "0.1.17"
```

That keeps local development wired to the workspace crate while still pinning the published crate version. Use `just bump <version>` to update the versions together.

## License

MIT
