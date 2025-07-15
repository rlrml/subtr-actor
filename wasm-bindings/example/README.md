# Rocket League Replay Analyzer - Example

This is an example web application demonstrating the subtr-actor WASM bindings for analyzing Rocket League replay files in the browser.

## Features

- 🚀 **WebAssembly Powered** - Fast replay processing using Rust/WASM
- 📊 **Interactive Visualizations** - Charts showing ball movement and game statistics  
- 📁 **Drag & Drop Interface** - Easy file upload with progress tracking
- 🎮 **Comprehensive Analysis** - Player stats, game metadata, and raw data viewing
- 📱 **Responsive Design** - Works on desktop and mobile devices

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or higher)
- [Yarn](https://yarnpkg.com/) package manager
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) for building WASM bindings

### Installation

1. **Clone the repository and navigate to the example:**
   ```bash
   git clone https://github.com/rlrml/subtr-actor.git
   cd subtr-actor/wasm-bindings/example
   ```

2. **Install dependencies:**
   ```bash
   yarn install
   ```

3. **Build the WASM bindings:**
   ```bash
   yarn build-wasm
   ```

4. **Start the development server:**
   ```bash
   yarn dev
   ```

5. **Open your browser:**
   - Navigate to `http://localhost:5173`
   - Upload a `.replay` file to start analyzing!

### Alternative: Build and run in one command

```bash
yarn dev-with-wasm
```

## Usage

1. **Open the web application** in your browser
2. **Drag and drop** a Rocket League `.replay` file onto the upload area, or **click to browse** for a file
3. **Wait for processing** - the app will validate, parse, and analyze the replay
4. **Explore the results** using the different tabs:
   - **Overview**: Game information and player list
   - **Ball Movement**: Interactive chart showing ball position over time
   - **Player Data**: Individual player statistics and performance metrics
   - **Raw Data**: First 10 rows of the processed numerical data

## What the App Demonstrates

### WASM Functions Used

- `validate_replay()` - Ensures the file is a valid replay
- `get_replay_info()` - Extracts basic replay metadata
- `get_ndarray_with_info()` - Processes the replay into numerical arrays for analysis
- `get_replay_meta()` - Gets detailed metadata including player information

### Data Visualization

- **Ball Position Chart**: Shows the ball's movement on the field using Chart.js
- **Statistics Dashboard**: Key metrics like game duration, player count, file size
- **Player Analytics**: Individual player stats including boost usage
- **Raw Data Table**: Direct view of the processed numerical data

### Technical Features

- **Progress Tracking**: Real-time progress updates during processing
- **Error Handling**: User-friendly error messages for invalid files
- **Responsive UI**: Clean, modern interface that works on all devices
- **Performance**: Efficient processing of large replay files

## File Structure

```
example/
├── index.html          # Main HTML page with UI
├── src/
│   └── index.js        # JavaScript application logic
├── package.json        # Dependencies and scripts
├── vite.config.js      # Vite bundler configuration
└── README.md          # This file
```

## Customization

### Adding New Visualizations

You can extend the app by adding new charts or analysis views:

1. Add a new tab in `index.html`
2. Create the visualization logic in `src/index.js`
3. Use additional WASM functions like `get_replay_frames_data()` for more detailed analysis

### Modifying Feature Extractors

The app uses default feature extractors, but you can customize them:

```javascript
// Custom feature configuration
const customResult = get_ndarray_with_info(
    replayData,
    ['BallRigidBody', 'GameTime'],           // global features
    ['PlayerRigidBody', 'PlayerBoost'],      // player features
    30.0                                     // higher FPS for more detail
);
```

### Styling Changes

The CSS is embedded in `index.html` and uses CSS Grid and modern features. You can:
- Modify colors by changing the gradient values
- Adjust the responsive grid layouts
- Add new animations or transitions

## Performance Notes

- **File Size**: The app can handle replay files up to several MB
- **Processing Time**: Depends on replay length and complexity (typically 1-5 seconds)
- **Memory Usage**: Larger replays will use more browser memory
- **Browser Support**: Requires WebAssembly support (all modern browsers)

## Troubleshooting

### Common Issues

**"WASM module not initialized"**
- Make sure `yarn build-wasm` was run successfully
- Check that the `pkg/` directory exists with WASM files

**"Invalid replay file"**
- Ensure you're uploading a `.replay` file from Rocket League
- Some very old or corrupted replays may not be supported

**Charts not displaying**
- Check browser console for JavaScript errors
- Ensure Chart.js loaded correctly

**Development server won't start**
- Make sure you have Node.js 18+ installed
- Try `yarn install` to reinstall dependencies

### Getting Help

- Check the browser developer console for detailed error messages
- Refer to the main [subtr-actor documentation](https://docs.rs/subtr-actor/)
- Open an issue on [GitHub](https://github.com/rlrml/subtr-actor/issues)

## Next Steps

This example provides a foundation for building more sophisticated replay analysis tools. Consider adding:

- **Timeline scrubbing** to view game state at specific moments
- **Heatmaps** showing player positioning over time
- **Advanced statistics** like boost efficiency or ball possession
- **Comparison tools** for analyzing multiple replays
- **Export functionality** to save analysis results

Happy analyzing! 🚗⚽