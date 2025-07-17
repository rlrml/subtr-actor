# subtr-actor

[![Workflow Status](https://github.com/rlrml/subtr-actor/workflows/main/badge.svg)](https://github.com/rlrml/subtr-actor/actions?query=workflow%3A%22main%22) [![](https://docs.rs/subtr-actor/badge.svg)](https://docs.rs/subtr-actor) [![Version](https://img.shields.io/crates/v/subtr-actor.svg?style=flat-square)](https://crates.io/crates/subtr-actor) ![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

A powerful Rust library for processing and analyzing Rocket League replay files. Built on top of the [boxcars](https://docs.rs/boxcars/) parser, subtr-actor simplifies the complex actor-based structure of replay files into easy-to-use data formats.

## What is subtr-actor?

subtr-actor transforms Rocket League replay files into structured data that's perfect for:

- **Data analysis** - Extract detailed player statistics, ball movement, and game events
- **Machine learning** - Generate training datasets from replay data
- **Research** - Study player behavior and game dynamics
- **Visualization** - Create custom replay viewers and analysis tools

## Key Features

- 🚀 **High-performance processing** - Efficiently handles large replay files
- 📊 **Multiple output formats** - JSON, NumPy arrays, and custom data structures
- 🎯 **Frame-by-frame precision** - Access every detail of gameplay
- 🔌 **Language bindings** - Use from JavaScript, Python, or Rust
- 🧩 **Extensible architecture** - Add custom data extractors and processors

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
subtr-actor = "0.1.8"
```

### Python

```bash
pip install subtr-actor-py
```

### JavaScript/Node.js

```bash
npm install rl-replay-subtr-actor
```

## Quick Start

### Extract JSON data (Rust)

```rust
use subtr_actor::ReplayDataCollector;
use boxcars::ParserBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse replay file
    let data = std::fs::read("my_replay.replay")?;
    let replay = ParserBuilder::new(&data)
        .must_parse_network_data()
        .parse()?;

    // Extract structured data
    let collector = ReplayDataCollector::new();
    let replay_data = collector.get_replay_data(&replay)?;

    // Convert to JSON
    let json = replay_data.as_json()?;
    println!("{}", json);

    Ok(())
}
```

### Generate ML training data (Rust)

```rust
use subtr_actor::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read("my_replay.replay")?;
    let replay = ParserBuilder::new(&data)
        .must_parse_network_data()
        .parse()?;

    // Configure data extraction
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

    // Process at 30 FPS
    FrameRateDecorator::new_from_fps(30.0, &mut collector)
        .process_replay(&replay)?;

    let (meta, array) = collector.get_meta_and_ndarray()?;
    println!("Generated {}×{} training matrix", array.nrows(), array.ncols());

    Ok(())
}
```

### Python Example

```python
import subtr_actor_py

# Load replay and extract data
with open("my_replay.replay", "rb") as f:
    replay_data = f.read()

result = subtr_actor_py.get_replay_data(replay_data)
print(f"Found {len(result['players'])} players")
```

### JavaScript Example

```javascript
import * as subtrActor from 'rl-replay-subtr-actor';

// Load replay file
const replayData = await fetch('my_replay.replay')
    .then(r => r.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

// Extract structured data
const result = subtrActor.get_replay_data(replayData);
console.log(`Replay duration: ${result.duration} seconds`);
```

## Core Concepts

### ReplayProcessor
The heart of subtr-actor's processing pipeline. It handles the complex task of navigating through replay frames and maintaining game state.

### Collectors
Pluggable data extractors that define what information to extract from replays:

- **`ReplayDataCollector`** - Extracts comprehensive replay data as JSON-serializable structures
- **`NDArrayCollector`** - Generates numerical arrays perfect for machine learning
- **Custom collectors** - Implement the `Collector` trait for specialized data extraction

### Feature Adders
Configurable extractors that specify exactly what data to capture:

- **Global features** - Ball position, game time, score
- **Player features** - Position, boost, controls, team info
- **Custom features** - Implement `FeatureAdder` or `PlayerFeatureAdder` traits

## Advanced Usage

### Custom Feature Extraction

```rust
use subtr_actor::*;

// Create custom feature adders
build_global_feature_adder!(
    CustomBallFeature,
    f32,
    3, // Output 3 values
    |processor, _time| {
        // Extract custom ball data
        let ball_data = processor.get_ball_data();
        Ok(vec![ball_data.x, ball_data.y, ball_data.z])
    }
);
```

### Frame Rate Control

```rust
// Process at custom frame rate
let mut collector = ReplayDataCollector::new();
FrameRateDecorator::new_from_fps(60.0, &mut collector)
    .process_replay(&replay)?;
```

### String-based Configuration

Useful for language bindings or configuration files:

```rust
let collector = NDArrayCollector::<f32>::from_strings(
    &["BallRigidBody", "CurrentTime"],
    &["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"]
)?;
```

## Data Structures

### ReplayData
```rust
pub struct ReplayData {
    pub frame_data: FrameData,      // Frame-by-frame game data
    pub meta: ReplayMeta,           // Player info, game settings
    pub demolish_infos: Vec<DemolishInfo>, // Demolition events
}
```

### FrameData
```rust
pub struct FrameData {
    pub ball_data: BallData,        // Ball position/physics over time
    pub players: Vec<(PlayerId, PlayerData)>, // Player data by ID
    pub metadata_frames: Vec<MetadataFrame>,  // Game state metadata
}
```

## Language Bindings

### Python (`subtr-actor-py`)
- Full API access through Python functions
- NumPy integration for data analysis
- Ideal for data science workflows

### JavaScript (`rl-replay-subtr-actor`)
- WebAssembly-based for high performance
- Works in browsers and Node.js
- Perfect for web applications

## Performance Tips

1. **Use appropriate frame rates** - Higher FPS = more data but slower processing
2. **Select only needed features** - Fewer feature adders = faster processing
3. **Batch processing** - Process multiple replays in parallel
4. **Memory management** - Consider streaming for very large datasets

## Contributing

We welcome contributions! Please check our [issues](https://github.com/rlrml/subtr-actor/issues) for ways to help.

### Development Setup

```bash
git clone https://github.com/rlrml/subtr-actor.git
cd subtr-actor
cargo build
cargo test
```

### Building Language Bindings

```bash
# Python
cd python && poetry build

# JavaScript
cd js && npm run build
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Related Projects

- [boxcars](https://github.com/nickbabcock/boxcars) - The underlying replay parser
- [ballchasing.com](https://ballchasing.com/) - Replay analysis platform

## Support

- 📚 [Documentation](https://docs.rs/subtr-actor)
- 🐛 [Issue Tracker](https://github.com/rlrml/subtr-actor/issues)
- 💬 [Discussions](https://github.com/rlrml/subtr-actor/discussions)
