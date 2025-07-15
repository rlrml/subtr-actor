import init, {
    validate_replay,
    get_replay_info,
    get_ndarray_with_info,
    get_replay_meta,
    get_column_headers,
    get_replay_frames_data
} from '../pkg/subtr_actor_wasm.js';

import { Chart, registerables } from 'chart.js';
Chart.register(...registerables);

class ReplayAnalyzer {
    constructor() {
        this.wasmInitialized = false;
        this.currentReplayData = null;
        this.setupEventListeners();
    }

    async initialize() {
        try {
            await init();
            this.wasmInitialized = true;
            console.log('🚀 WASM module loaded successfully!');
        } catch (error) {
            console.error('Failed to initialize WASM:', error);
            this.showError('Failed to initialize WebAssembly module');
        }
    }

    setupEventListeners() {
        const uploadArea = document.getElementById('uploadArea');
        const fileInput = document.getElementById('fileInput');

        // File input change
        fileInput.addEventListener('change', (e) => {
            if (e.target.files.length > 0) {
                this.processFile(e.target.files[0]);
            }
        });

        // Drag and drop
        uploadArea.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadArea.classList.add('dragover');
        });

        uploadArea.addEventListener('dragleave', () => {
            uploadArea.classList.remove('dragover');
        });

        uploadArea.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadArea.classList.remove('dragover');
            
            if (e.dataTransfer.files.length > 0) {
                this.processFile(e.dataTransfer.files[0]);
            }
        });

        // Tab switching
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', (e) => {
                this.switchTab(e.target.dataset.tab);
            });
        });
    }

    switchTab(tabName) {
        // Update tab buttons
        document.querySelectorAll('.tab').forEach(tab => {
            tab.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

        // Update tab content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });
        document.getElementById(tabName).classList.add('active');
    }

    async processFile(file) {
        if (!this.wasmInitialized) {
            this.showError('WASM module not initialized yet');
            return;
        }

        if (!file.name.endsWith('.replay')) {
            this.showError('Please select a .replay file');
            return;
        }

        this.showProgress(true);
        this.hideError();

        try {
            // Read file as array buffer
            const arrayBuffer = await file.arrayBuffer();
            const replayData = new Uint8Array(arrayBuffer);
            this.currentReplayData = replayData;

            this.updateProgress(20, 'Validating replay file...');
            
            // Validate replay
            const validation = validate_replay(replayData);
            if (!validation.valid) {
                throw new Error(`Invalid replay file: ${validation.error}`);
            }

            this.updateProgress(40, 'Getting replay information...');
            
            // Get basic info
            const info = get_replay_info(replayData);
            
            this.updateProgress(60, 'Processing numerical data...');
            
            // Get NDArray data
            const ndarrayResult = get_ndarray_with_info(replayData, null, null, 10.0);
            
            this.updateProgress(80, 'Getting metadata...');
            
            // Get metadata
            const metadata = get_replay_meta(replayData);
            
            this.updateProgress(100, 'Complete!');

            // Display results
            this.displayResults({
                info,
                ndarrayResult,
                metadata,
                fileName: file.name,
                fileSize: file.size
            });

        } catch (error) {
            console.error('Error processing replay:', error);
            this.showError(`Failed to process replay: ${error.message}`);
        } finally {
            this.showProgress(false);
        }
    }

    displayResults(results) {
        document.getElementById('results').style.display = 'block';
        
        this.displayStats(results);
        this.displayGameInfo(results);
        this.displayBallChart(results.ndarrayResult);
        this.displayPlayerStats(results);
        this.displayRawData(results.ndarrayResult);
    }

    displayStats(results) {
        const statsGrid = document.getElementById('statsGrid');
        const { info, metadata, fileName, fileSize } = results;
        
        const stats = [
            { label: 'File Size', value: this.formatFileSize(fileSize) },
            { label: 'Replay Version', value: `${info.major_version}.${info.minor_version}` },
            { label: 'Properties', value: info.properties_count },
            { label: 'Players', value: metadata.replay_meta.players?.length || 'N/A' },
            { label: 'Duration', value: this.formatDuration(metadata.replay_meta.game_length_seconds) },
            { label: 'Data Points', value: results.ndarrayResult.shape[0] }
        ];

        statsGrid.innerHTML = stats.map(stat => `
            <div class="stat-card">
                <div class="stat-value">${stat.value}</div>
                <div class="stat-label">${stat.label}</div>
            </div>
        `).join('');
    }

    displayGameInfo(results) {
        const gameInfo = document.getElementById('gameInfo');
        const { metadata } = results;
        
        gameInfo.innerHTML = `
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-value">${metadata.replay_meta.map_name || 'Unknown'}</div>
                    <div class="stat-label">Map</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${metadata.replay_meta.team_0_score || 0}</div>
                    <div class="stat-label">Team 0 Score</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${metadata.replay_meta.team_1_score || 0}</div>
                    <div class="stat-label">Team 1 Score</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${metadata.column_headers.global_headers.length + metadata.column_headers.player_headers.length}</div>
                    <div class="stat-label">Data Columns</div>
                </div>
            </div>
            
            <h4>Players:</h4>
            <ul>
                ${(metadata.replay_meta.players || []).map(player => 
                    `<li><strong>${player.name}</strong> (Team ${player.team})</li>`
                ).join('')}
            </ul>
        `;
    }

    displayBallChart(ndarrayResult) {
        const ctx = document.getElementById('ballChart');
        const data = ndarrayResult.array_data;
        const headers = ndarrayResult.metadata.column_headers.global_headers;
        
        // Find ball position indices (assuming BallRigidBody gives us X, Y, Z coordinates)
        const ballXIndex = headers.findIndex(h => h.includes('pos_x') || h.includes('location_x'));
        const ballYIndex = headers.findIndex(h => h.includes('pos_y') || h.includes('location_y'));
        
        if (ballXIndex === -1 || ballYIndex === -1) {
            ctx.parentElement.innerHTML = '<p>Ball position data not available in this replay</p>';
            return;
        }

        // Extract ball positions (sample every 10th point for performance)
        const ballPositions = data.filter((_, index) => index % 10 === 0).map((row, index) => ({
            x: row[ballXIndex],
            y: row[ballYIndex],
            time: index * 10 / 10 // approximate time in seconds
        }));

        new Chart(ctx, {
            type: 'scatter',
            data: {
                datasets: [{
                    label: 'Ball Position',
                    data: ballPositions,
                    backgroundColor: 'rgba(102, 126, 234, 0.6)',
                    borderColor: 'rgba(102, 126, 234, 1)',
                    pointRadius: 2
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    x: {
                        title: {
                            display: true,
                            text: 'X Position'
                        }
                    },
                    y: {
                        title: {
                            display: true,
                            text: 'Y Position'
                        }
                    }
                },
                plugins: {
                    title: {
                        display: true,
                        text: 'Ball Movement on Field'
                    },
                    tooltip: {
                        callbacks: {
                            label: function(context) {
                                return `Position: (${context.parsed.x.toFixed(1)}, ${context.parsed.y.toFixed(1)})`;
                            }
                        }
                    }
                }
            }
        });
    }

    displayPlayerStats(results) {
        const playerStats = document.getElementById('playerStats');
        const { metadata, ndarrayResult } = results;
        
        if (!metadata.replay_meta.players || metadata.replay_meta.players.length === 0) {
            playerStats.innerHTML = '<p>Player data not available</p>';
            return;
        }

        const playerHeaders = ndarrayResult.metadata.column_headers.player_headers;
        const data = ndarrayResult.array_data;
        
        // Calculate basic stats for each player
        const playerStatsHtml = metadata.replay_meta.players.map((player, playerIndex) => {
            // Find boost-related columns for this player
            const boostColumnIndex = playerHeaders.findIndex(h => 
                h.includes(`Player${playerIndex}`) && h.includes('boost')
            );
            
            let avgBoost = 'N/A';
            if (boostColumnIndex !== -1) {
                const globalColumnsCount = ndarrayResult.metadata.column_headers.global_headers.length;
                const actualBoostIndex = globalColumnsCount + boostColumnIndex;
                const boostValues = data.map(row => row[actualBoostIndex]).filter(val => val !== undefined);
                avgBoost = boostValues.length > 0 ? 
                    (boostValues.reduce((a, b) => a + b, 0) / boostValues.length).toFixed(1) : 'N/A';
            }

            return `
                <div class="stat-card">
                    <h4>${player.name}</h4>
                    <p><strong>Team:</strong> ${player.team}</p>
                    <p><strong>Score:</strong> ${player.score || 'N/A'}</p>
                    <p><strong>Avg Boost:</strong> ${avgBoost}%</p>
                </div>
            `;
        }).join('');

        playerStats.innerHTML = `
            <div class="stats-grid">
                ${playerStatsHtml}
            </div>
        `;
    }

    displayRawData(ndarrayResult) {
        const table = document.getElementById('rawDataTable');
        const headers = [
            ...ndarrayResult.metadata.column_headers.global_headers,
            ...ndarrayResult.metadata.column_headers.player_headers
        ];
        
        const data = ndarrayResult.array_data.slice(0, 10); // First 10 rows
        
        const headerRow = headers.slice(0, 10).map(h => `<th>${h}</th>`).join(''); // First 10 columns
        const dataRows = data.map(row => {
            const cells = row.slice(0, 10).map(cell => `<td>${cell.toFixed(3)}</td>`).join('');
            return `<tr>${cells}</tr>`;
        }).join('');
        
        table.innerHTML = `
            <thead>
                <tr>${headerRow}</tr>
            </thead>
            <tbody>
                ${dataRows}
            </tbody>
        `;
    }

    formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    formatDuration(seconds) {
        if (!seconds) return 'N/A';
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }

    showProgress(show) {
        const progressBar = document.getElementById('progressBar');
        progressBar.style.display = show ? 'block' : 'none';
        if (!show) {
            this.updateProgress(0, '');
        }
    }

    updateProgress(percent, message) {
        const progressFill = document.getElementById('progressFill');
        progressFill.style.width = `${percent}%`;
        
        if (message) {
            console.log(`Progress: ${percent}% - ${message}`);
        }
    }

    showError(message) {
        const uploadArea = document.getElementById('uploadArea');
        uploadArea.innerHTML = `
            <div class="error">
                <strong>Error:</strong> ${message}
            </div>
            <button class="btn" onclick="location.reload()">Try Again</button>
        `;
    }

    hideError() {
        // Reset upload area if needed
        const uploadArea = document.getElementById('uploadArea');
        if (uploadArea.innerHTML.includes('error')) {
            location.reload();
        }
    }
}

// Initialize the analyzer when the page loads
const analyzer = new ReplayAnalyzer();

// Initialize WASM and start the app
analyzer.initialize().then(() => {
    console.log('Replay analyzer ready!');
}).catch(error => {
    console.error('Failed to initialize:', error);
});