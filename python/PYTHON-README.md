# subtr-actor-py

Python bindings for [subtr-actor](https://github.com/rlrml/subtr-actor), a Rocket League replay processing library.

## Installation

```bash
pip install subtr-actor-py
# or, with uv:
uv add subtr-actor-py
# or, with Poetry:
poetry add subtr-actor-py
```

## Usage

```python
import subtr_actor

replay_path = "path/to/replay.replay"

# Parse raw replay bytes into the full replay structure.
with open(replay_path, "rb") as replay_file:
    replay = subtr_actor.parse_replay(replay_file.read())

# Build a numpy ndarray plus metadata.
meta, ndarray = subtr_actor.get_ndarray_with_info_from_replay_filepath(
    replay_path,
    global_feature_adders=["BallRigidBody", "SecondsRemaining"],
    player_feature_adders=["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"],
    fps=10.0,
    dtype="float32",
)

headers = subtr_actor.get_column_headers(
    global_feature_adders=["BallRigidBody", "SecondsRemaining"],
    player_feature_adders=["PlayerRigidBody", "PlayerBoost"],
)

replay_meta = subtr_actor.get_replay_meta(replay_path)
frames_data = subtr_actor.get_replay_frames_data(replay_path)
stats_events = subtr_actor.get_stats_events(replay_path)
summed_stats = subtr_actor.get_summed_stats(
    replay_path,
    module_names=["core", "boost", "movement"],
)
stats_module_names = subtr_actor.get_stats_module_names()
stats_snapshot_data = subtr_actor.get_stats_snapshot_data(
    replay_path,
    module_names=["core", "boost"],
    frame_step_seconds=1.0,
)
stats_timeline = subtr_actor.get_stats_timeline(
    replay_path,
    frame_step_seconds=1.0,
)
legacy_stats_timeline = subtr_actor.get_legacy_stats_timeline(
    replay_path,
    module_names=["core", "boost", "movement"],
    frame_step_seconds=1.0,
)

print(ndarray.shape)
print(headers["player_headers"][:5])
print(replay_meta["map_name"])
print(stats_events["boost_ledger"][-1])
print(summed_stats["modules"]["boost"]["team_zero"]["amount_collected"])
print(stats_module_names)
print(stats_snapshot_data["frames"][-1]["modules"]["boost"]["team_zero"]["amount_collected"])
print(stats_timeline["events"]["boost_ledger"][-1])
print(legacy_stats_timeline["frames"][-1]["team_zero"]["boost"]["amount_collected"])
```

`get_stats_timeline` is the compact event-backed timeline. Its `frames` contain
timing, gameplay state, and player identity scaffolding only; stat deltas live
under `events`. Use `get_legacy_stats_timeline` only for compatibility code that
still needs serialized per-frame team/player snapshots.

## API Surface

### `parse_replay(data: bytes) -> dict`

Parse raw replay bytes and return the full replay structure as Python data.

### `get_ndarray_with_info_from_replay_filepath(filepath, global_feature_adders=None, player_feature_adders=None, fps=None, dtype=None) -> tuple[dict, numpy.ndarray]`

Process a replay file and return metadata plus a `numpy.ndarray`.

Parameters:

- `filepath`: path to the replay file
- `global_feature_adders`: list of global feature names, default `["BallRigidBody"]`
- `player_feature_adders`: list of player feature names, default `["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"]`
- `fps`: target FPS for resampling, default `10.0`
- `dtype`: output dtype string. Supported values are `float16`/`f16`/`half`, `float32`/`f32`, and `float64`/`f64`/`double`

### `get_replay_meta(filepath, global_feature_adders=None, player_feature_adders=None) -> dict`

Get replay metadata and ndarray headers without materializing the full ndarray.

### `get_column_headers(global_feature_adders=None, player_feature_adders=None) -> dict`

Get header information for the configured ndarray layout.

### `get_replay_frames_data(filepath) -> dict`

Get structured frame-by-frame game state data with no FPS resampling.

### `get_stats_events(filepath, frame_step_seconds=None) -> dict`

Get the compact modern stats event streams for a replay.

Parameters:

- `filepath`: path to the replay file
- `frame_step_seconds`: optional positive sampling interval in seconds for the
  accompanying timeline collector. Events are still emitted as compact stat
  change streams.

### `get_summed_stats(filepath, module_names=None) -> dict`

Get aggregate summed stats for the selected builtin modules.

Parameters:

- `filepath`: path to the replay file
- `module_names`: optional list of builtin stats module names. By default all
  builtin modules are included.

### `get_stats_module_names() -> list[str]`

List the builtin stats modules that can be selected in `get_summed_stats`,
`get_stats_snapshot_data`, and `get_legacy_stats_timeline`.

### `get_stats_snapshot_data(filepath, module_names=None, frame_step_seconds=None) -> dict`

Get the raw stats snapshot payload produced by `StatsCollector`, including:

- `config`: module configuration emitted by the selected stats modules
- `modules`: aggregate module outputs
- `frames`: per-sample module snapshots keyed by module name

Parameters:

- `filepath`: path to the replay file
- `module_names`: optional list of builtin stats module names. By default all
  builtin modules are included.
- `frame_step_seconds`: optional positive sampling interval in seconds. By
  default every replay frame is captured.

### `get_stats_timeline(filepath, frame_step_seconds=None) -> dict`

Get the compact event-backed stats timeline for each replay sample.

Frames contain timing, gameplay state, and player identity scaffolding only;
stat state changes are transferred through `events`, and full team/player
snapshots can be derived by clients that need them.

Parameters:

- `filepath`: path to the replay file
- `frame_step_seconds`: optional positive sampling interval in seconds. By
  default every replay frame is captured.

`module_names` filtering is not supported for compact event timelines. Passing
it raises `ValueError`; use `get_legacy_stats_timeline` if filtered full
snapshot timelines are needed.

### `get_legacy_stats_timeline(filepath, module_names=None, frame_step_seconds=None) -> dict`

Get cumulative typed stats snapshots for each replay sample.

This preserves the pre-compact timeline behavior for compatibility and for
explicit parity checks, but it serializes the full team/player partial sums.

Parameters:

- `filepath`: path to the replay file
- `module_names`: optional list of builtin stats module names. By default all
  builtin modules are included.
- `frame_step_seconds`: optional positive sampling interval in seconds. By
  default every replay frame is captured.

## Feature Adders

See the [subtr-actor ndarray docs](https://docs.rs/subtr-actor/latest/subtr_actor/collector/ndarray/index.html) for the full list of feature-adder names.

Common global features:

- `BallRigidBody`
- `CurrentTime`
- `SecondsRemaining`

Common player features:

- `PlayerRigidBody`
- `PlayerBallDistance`
- `PlayerBoost`
- `PlayerAnyJump`
- `PlayerJump`
- `PlayerDodgeRefreshed`
- `PlayerEvent:touch`

`PlayerBoost` is exposed in raw replay units (`0-255`), not `0-100` percent.
Analysis-backed player event indicators use `PlayerEvent:<event_name>` and emit
`1` for a sampled frame when that player has a new event, otherwise `0`.

## Development

Repository-level compile check:

```bash
just build-python
```

For an importable local Python environment, use `maturin develop` from the `python/` directory:

```bash
cd python
uv sync --group dev
uv run maturin develop
uv run pytest
```

If you are using the repo flake, `nix develop` now provides the pinned CPython 3.11 toolchain and Python dev dependencies via `uv2nix`. Create a writable virtual environment from that interpreter, then install the local extension into it:

```bash
nix develop
uv venv /tmp/subtr-actor-venv
source /tmp/subtr-actor-venv/bin/activate
cd python
maturin develop
pytest
```

If you are not using `uv` or Nix, install `maturin`, `pytest`, and `numpy` in a virtual environment and run `maturin develop` directly.

## Publishing Notes

This binding depends on the workspace crate via:

```toml
[dependencies.subtr-actor]
path = ".."
version = "1.2.0"
```

That keeps local development wired to the workspace crate while still pinning the published dependency version. Use `just bump <version>` to update the workspace and binding versions together.

## License

MIT
