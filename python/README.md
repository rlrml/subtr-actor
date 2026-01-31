# subtr-actor-py

Python bindings for [subtr-actor](https://github.com/rlrml/subtr-actor), a Rocket League replay processing library.

## Installation

```bash
pip install subtr-actor-py
```

## Usage

```python
import subtr_actor

# Parse a replay file to get raw replay data
replay = subtr_actor.parse_replay(open("path/to/replay.replay", "rb").read())

# Get numerical data as numpy ndarray (useful for ML)
meta, ndarray = subtr_actor.get_ndarray_with_info_from_replay_filepath(
    "path/to/replay.replay",
    global_feature_adders=["BallRigidBody"],  # optional
    player_feature_adders=["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"],  # optional
    fps=10.0  # optional, default is 10.0
)

# Get column headers to understand the ndarray structure
headers = subtr_actor.get_column_headers(
    global_feature_adders=["BallRigidBody"],
    player_feature_adders=["PlayerRigidBody", "PlayerBoost"]
)

# Get replay metadata without processing all frames (faster)
meta = subtr_actor.get_replay_meta("path/to/replay.replay")

# Get structured frame-by-frame data
frames_data = subtr_actor.get_replay_frames_data("path/to/replay.replay")
```

## API Reference

### `parse_replay(data: bytes) -> dict`
Parse raw replay bytes and return the complete replay structure.

### `get_ndarray_with_info_from_replay_filepath(filepath, global_feature_adders=None, player_feature_adders=None, fps=None) -> tuple[dict, numpy.ndarray]`
Process a replay file and return metadata plus a numpy ndarray of features.

**Parameters:**
- `filepath` - Path to the replay file
- `global_feature_adders` - List of global feature names (default: `["BallRigidBody"]`)
- `player_feature_adders` - List of player feature names (default: `["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"]`)
- `fps` - Frames per second for processing (default: 10.0)

### `get_replay_meta(filepath, global_feature_adders=None, player_feature_adders=None) -> dict`
Get replay metadata without processing frame data (faster than full processing).

### `get_column_headers(global_feature_adders=None, player_feature_adders=None) -> dict`
Get column header information for understanding ndarray structure.

### `get_replay_frames_data(filepath) -> dict`
Get structured frame-by-frame game state data.

## Feature Adders

See the [subtr-actor documentation](https://docs.rs/subtr-actor/latest/subtr_actor/collector/ndarray/index.html) for available feature adder names.

**Common global features:** `BallRigidBody`, `GameTime`

**Common player features:** `PlayerRigidBody`, `PlayerBoost`, `PlayerAnyJump`, `PlayerDoubleJump`

## Building from Source

Requirements:
- Rust toolchain
- maturin
- just (command runner)

```bash
# Clone the repository
git clone https://github.com/rlrml/subtr-actor.git
cd subtr-actor

# Build the Python package
just build-python

# Or for development (editable install)
cd python && maturin develop
```

### Monorepo Dependency Management

This package is part of the [subtr-actor](https://github.com/rlrml/subtr-actor) monorepo. The Cargo.toml uses a dual dependency specification:

```toml
[dependencies.subtr-actor]
path = ".."
version = "0.1.10"
```

This allows:
- **Local development**: Cargo uses the `path` dependency, so changes to the main `subtr-actor` crate are immediately available for testing
- **Publishing**: crates.io/PyPI strips the `path` and uses the `version`, ensuring the published package depends on the published crate

Use `just bump <version>` to update all versions in sync (workspace version and dependency versions).

### Publishing

To publish all packages in the correct order:

```bash
just publish-all  # Publishes: Rust crate -> Python bindings -> JS bindings
```

Or publish individually:

```bash
just publish-rust    # Publish main Rust crate first
just publish-python  # Then publish Python bindings
```

**Important**: The main `subtr-actor` Rust crate must be published to crates.io before publishing the bindings, as the published bindings depend on the published crate version.

## License

MIT
