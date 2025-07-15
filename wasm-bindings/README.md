# subtr-actor-wasm

WebAssembly bindings for [subtr-actor](https://crates.io/crates/subtr-actor), a Rocket League replay processing library.

This package provides the same functionality as the Python bindings but for JavaScript/TypeScript environments.

## Installation

```bash
npm install subtr-actor-wasm
```

## Usage

### Web/Browser

```javascript
import init, { 
    parse_replay, 
    get_ndarray_with_info, 
    get_replay_meta, 
    get_column_headers, 
    get_replay_frames_data, 
    validate_replay, 
    get_replay_info 
} from 'subtr-actor-wasm';

async function analyzeReplay() {
    // Initialize the WASM module
    await init();
    
    // Load replay file (e.g., from file input)
    const fileInput = document.getElementById('replay-file');
    const file = fileInput.files[0];
    const arrayBuffer = await file.arrayBuffer();
    const replayData = new Uint8Array(arrayBuffer);
    
    try {
        // Validate the replay first
        const validation = validate_replay(replayData);
        if (!validation.valid) {
            console.error('Invalid replay:', validation.error);
            return;
        }
        
        // Get basic replay information
        const info = get_replay_info(replayData);
        console.log('Replay info:', info);
        
        // Get column headers (useful for understanding data structure)
        const headers = get_column_headers();
        console.log('Column headers:', headers);
        
        // Option 1: Get numerical data as NDArray
        const ndarrayResult = get_ndarray_with_info(replayData, null, null, 10.0);
        console.log('NDArray data:', ndarrayResult.array_data);
        console.log('NDArray shape:', ndarrayResult.shape);
        console.log('Metadata:', ndarrayResult.metadata);
        
        // Option 2: Get structured frame data
        const frameData = get_replay_frames_data(replayData);
        console.log('Frame data:', frameData);
        
        // Option 3: Get just metadata without processing frames
        const metadata = get_replay_meta(replayData);
        console.log('Metadata only:', metadata);
        
    } catch (error) {
        console.error('Error processing replay:', error);
    }
}
```

### Node.js

```javascript
const { 
    parse_replay, 
    get_ndarray_with_info, 
    get_replay_meta, 
    get_column_headers, 
    get_replay_frames_data, 
    validate_replay, 
    get_replay_info 
} = require('subtr-actor-wasm');
const fs = require('fs');

// Read replay file
const replayData = fs.readFileSync('path/to/replay.replay');

try {
    // Validate and get basic info
    const validation = validate_replay(replayData);
    if (!validation.valid) {
        console.error('Invalid replay:', validation.error);
        return;
    }
    
    const info = get_replay_info(replayData);
    console.log('Replay info:', info);
    
    // Get NDArray data with custom feature adders and FPS
    const result = get_ndarray_with_info(
        replayData,
        ['BallRigidBody'],                           // global features
        ['PlayerRigidBody', 'PlayerBoost'],         // player features  
        30.0                                        // FPS
    );
    
    console.log('Array data shape:', result.shape);
    console.log('First few rows:', result.array_data.slice(0, 5));
    
    // Get structured frame data
    const frameData = get_replay_frames_data(replayData);
    console.log('Number of frames:', frameData.frames.length);
    
} catch (error) {
    console.error('Error:', error);
}
```

### Advanced Usage with Custom Feature Adders

```javascript
// Get available column headers for different configurations
const defaultHeaders = get_column_headers();
console.log('Default headers:', defaultHeaders);

const customHeaders = get_column_headers(
    ['BallRigidBody', 'GameTime'],           // global features
    ['PlayerRigidBody', 'PlayerBoost', 'PlayerAnyJump', 'PlayerDoubleJump']  // player features
);
console.log('Custom headers:', customHeaders);

// Process with custom configuration
const customResult = get_ndarray_with_info(
    replayData,
    ['BallRigidBody', 'GameTime'],
    ['PlayerRigidBody', 'PlayerBoost', 'PlayerAnyJump'],
    60.0  // High FPS for detailed analysis
);
```

## API Reference

### Core Functions

#### `validate_replay(data: Uint8Array): {valid: boolean, message?: string, error?: string}`
Validates that the replay file can be parsed successfully.

#### `get_replay_info(data: Uint8Array): object`
Gets basic replay information including version numbers and property counts.

#### `parse_replay(data: Uint8Array): object`
Parses the raw replay data and returns the complete boxcars Replay structure.

### NDArray Functions (Numerical Data)

#### `get_ndarray_with_info(data: Uint8Array, globalFeatures?: string[], playerFeatures?: string[], fps?: number): object`
Returns numerical data suitable for machine learning analysis.

**Parameters:**
- `data` - Replay file data
- `globalFeatures` - Array of global feature names (default: `["BallRigidBody"]`)
- `playerFeatures` - Array of player feature names (default: `["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"]`)
- `fps` - Frames per second for processing (default: 10.0)

**Returns:**
```javascript
{
    metadata: {
        replay_meta: { /* replay metadata */ },
        column_headers: {
            global_headers: string[],
            player_headers: string[]
        }
    },
    array_data: number[][],  // 2D array of numerical data
    shape: number[]          // [rows, columns]
}
```

#### `get_replay_meta(data: Uint8Array, globalFeatures?: string[], playerFeatures?: string[]): object`
Gets only the metadata without processing frame data (faster).

#### `get_column_headers(globalFeatures?: string[], playerFeatures?: string[]): object`
Gets column headers for understanding the data structure.

### Structured Data Functions

#### `get_replay_frames_data(data: Uint8Array): object`
Returns structured frame-by-frame data with full game state information.

**Returns:**
```javascript
{
    frames: [
        {
            time: number,
            actors: { /* actor state data */ },
            // ... other frame data
        }
    ],
    // ... other replay data
}
```

### Feature Adders

Available feature adder strings for customizing data collection:

**Global Features:**
- `"BallRigidBody"` - Ball position, velocity, rotation
- `"GameTime"` - Game time information

**Player Features:**
- `"PlayerRigidBody"` - Player position, velocity, rotation
- `"PlayerBoost"` - Boost amount and usage
- `"PlayerAnyJump"` - Jump state
- `"PlayerDoubleJump"` - Double jump state
- And many more... (see [subtr-actor documentation](https://docs.rs/subtr-actor/latest/subtr_actor/collector/ndarray/index.html))

## Building from Source

Requirements:
- Rust toolchain
- wasm-pack

```bash
# Clone the repository
git clone https://github.com/rlrml/subtr-actor.git
cd subtr-actor/wasm-bindings

# Build for web
npm run build

# Build for Node.js
npm run build:nodejs

# Build for bundlers (webpack, etc.)
npm run build:bundler
```

## TypeScript Support

The package includes TypeScript definitions. All functions return proper JavaScript objects (not strings), making them easy to use with TypeScript.

## License

MIT