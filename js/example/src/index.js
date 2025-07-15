import init, {
    validate_replay,
    get_replay_info,
    get_ndarray_with_info,
    get_replay_meta,
    get_column_headers,
    get_replay_frames_data,
} from "rl-replay-subtr-actor";
import wasmUrl from "rl-replay-subtr-actor/rl_replay_subtr_actor_bg.wasm?url";

import { Chart, registerables } from "chart.js";
Chart.register(...registerables);

class ReplayAnalyzer {
    constructor() {
        this.wasmInitialized = false;
        this.currentReplayData = null;
        this.setupEventListeners();
    }

    async initialize() {
        try {
            await init(wasmUrl);
            this.wasmInitialized = true;
            console.log("🚀 WASM module loaded successfully!");
        } catch (error) {
            console.error("Failed to initialize WASM:", error);
            this.showError("Failed to initialize WebAssembly module");
        }
    }

    setupEventListeners() {
        const uploadArea = document.getElementById("uploadArea");
        const fileInput = document.getElementById("fileInput");

        // File input change
        fileInput.addEventListener("change", (e) => {
            if (e.target.files.length > 0) {
                this.processFile(e.target.files[0]);
            }
        });

        // Drag and drop
        uploadArea.addEventListener("dragover", (e) => {
            e.preventDefault();
            uploadArea.classList.add("dragover");
        });

        uploadArea.addEventListener("dragleave", () => {
            uploadArea.classList.remove("dragover");
        });

        uploadArea.addEventListener("drop", (e) => {
            e.preventDefault();
            uploadArea.classList.remove("dragover");
            if (e.dataTransfer.files.length > 0) {
                this.processFile(e.dataTransfer.files[0]);
            }
        });

        // Tab switching
        document.querySelectorAll(".tab").forEach((tab) => {
            tab.addEventListener("click", (e) => {
                this.switchTab(e.target.dataset.tab);
            });
        });
    }

    switchTab(tabName) {
        // Update tab buttons
        document.querySelectorAll(".tab").forEach((tab) => {
            tab.classList.remove("active");
        });
        document
            .querySelector(`[data-tab="${tabName}"]`)
            .classList.add("active");

        // Update tab content
        document.querySelectorAll(".tab-content").forEach((content) => {
            content.classList.remove("active");
        });
        document.getElementById(tabName).classList.add("active");
    }

    // Helper function to convert Map objects to plain objects
    mapToObject(map) {
        if (!map || typeof map.get !== "function") {
            return map; // Return as-is if not a Map
        }

        const obj = {};
        for (const [key, value] of map) {
            if (
                typeof value === "object" &&
                value &&
                typeof value.get === "function"
            ) {
                // It's a Map, recursively convert
                obj[key] = this.mapToObject(value);
            } else if (Array.isArray(value)) {
                // It's an array, convert each element if needed
                obj[key] = value.map((item) =>
                    typeof item === "object" &&
                    item &&
                    typeof item.get === "function"
                        ? this.mapToObject(item)
                        : item,
                );
            } else {
                // It's a primitive value
                obj[key] = value;
            }
        }
        return obj;
    }

    async processFile(file) {
        if (!this.wasmInitialized) {
            this.showError("WASM module not initialized yet");
            return;
        }

        if (!file.name.endsWith(".replay")) {
            this.showError("Please select a .replay file");
            return;
        }

        this.showProgress(true);
        this.hideError();

        try {
            // Read file as array buffer
            const arrayBuffer = await file.arrayBuffer();
            const replayData = new Uint8Array(arrayBuffer);
            this.currentReplayData = replayData;

            this.updateProgress(20, "Validating replay file...");

            // Validate replay
            const validation = validate_replay(replayData);

            // Handle Map object returned by WASM
            const isValid = validation.get
                ? validation.get("valid")
                : validation.valid;
            const errorMsg = validation.get
                ? validation.get("error")
                : validation.error;

            if (!isValid) {
                throw new Error(
                    `Invalid replay file: ${errorMsg || "Unknown error"}`,
                );
            }

            this.updateProgress(40, "Getting replay information...");

            // Get basic info and convert from Map to plain object
            const info = this.mapToObject(get_replay_info(replayData));

            this.updateProgress(60, "Processing numerical data...");

            // Get NDArray data and convert from Map to plain object
            const ndarrayResult = this.mapToObject(
                get_ndarray_with_info(replayData, null, null, 10.0),
            );

            this.updateProgress(80, "Getting metadata...");

            // Get metadata and convert from Map to plain object
            const metadata = this.mapToObject(get_replay_meta(replayData));

            this.updateProgress(100, "Complete!");

            // Display results
            this.displayResults({
                info,
                ndarrayResult,
                metadata,
                fileName: file.name,
                fileSize: file.size,
            });
        } catch (error) {
            console.error("Error processing replay:", error);
            this.showError(`Failed to process replay: ${error.message}`);
        } finally {
            this.showProgress(false);
        }
    }

    displayResults(results) {
        document.getElementById("results").style.display = "block";

        // Hide the stats section at the top
        const statsCard = document.querySelector('.card h2');
        if (statsCard && statsCard.textContent.includes('Replay Statistics')) {
            statsCard.parentElement.style.display = 'none';
        }

        // Hide all the other tabs and show only playback
        document.querySelectorAll('.tab').forEach(tab => {
            if (tab.dataset.tab !== 'playback') {
                tab.style.display = 'none';
            } else {
                tab.classList.add('active');
            }
        });

        document.querySelectorAll('.tab-content').forEach(content => {
            if (content.id !== 'playback') {
                content.style.display = 'none';
            } else {
                content.classList.add('active');
            }
        });

        this.setupPlayback(results);
    }

    displayStats(results) {
        const statsGrid = document.getElementById("statsGrid");
        const { info, metadata, fileName, fileSize } = results;

        const stats = [
            { label: "File Size", value: this.formatFileSize(fileSize) },
            {
                label: "Replay Version",
                value: `${info.major_version}.${info.minor_version}`,
            },
            { label: "Properties", value: info.properties_count },
            {
                label: "Players",
                value: metadata.replay_meta.players?.length || "N/A",
            },
            {
                label: "Duration",
                value: this.formatDuration(
                    metadata.replay_meta.game_length_seconds,
                ),
            },
            { label: "Data Points", value: results.ndarrayResult.shape[0] },
        ];

        statsGrid.innerHTML = stats
            .map(
                (stat) => `
            <div class="stat-card">
                <div class="stat-value">${stat.value}</div>
                <div class="stat-label">${stat.label}</div>
            </div>
        `,
            )
            .join("");
    }

    displayGameInfo(results) {
        const gameInfo = document.getElementById("gameInfo");
        const { metadata } = results;

        gameInfo.innerHTML = `
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-value">${metadata.replay_meta.map_name || "Unknown"}</div>
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
                ${(metadata.replay_meta.players || [])
                    .map(
                        (player) =>
                            `<li><strong>${player.name}</strong> (Team ${player.team})</li>`,
                    )
                    .join("")}
            </ul>
        `;
    }

    displayBallChart(ndarrayResult) {
        const ctx = document.getElementById("ballChart");
        const data = ndarrayResult.array_data;
        const headers = ndarrayResult.metadata.column_headers.global_headers;

        // Find ball position indices (assuming BallRigidBody gives us X, Y, Z coordinates)
        const ballXIndex = headers.findIndex(
            (h) => h.includes("pos_x") || h.includes("location_x"),
        );
        const ballYIndex = headers.findIndex(
            (h) => h.includes("pos_y") || h.includes("location_y"),
        );

        if (ballXIndex === -1 || ballYIndex === -1) {
            ctx.parentElement.innerHTML =
                "<p>Ball position data not available in this replay</p>";
            return;
        }

        // Extract ball positions (sample every 10th point for performance)
        const ballPositions = data
            .filter((_, index) => index % 10 === 0)
            .map((row, index) => ({
                x: row[ballXIndex],
                y: row[ballYIndex],
                time: (index * 10) / 10, // approximate time in seconds
            }));

        new Chart(ctx, {
            type: "scatter",
            data: {
                datasets: [
                    {
                        label: "Ball Position",
                        data: ballPositions,
                        backgroundColor: "rgba(102, 126, 234, 0.6)",
                        borderColor: "rgba(102, 126, 234, 1)",
                        pointRadius: 2,
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    x: {
                        title: {
                            display: true,
                            text: "X Position",
                        },
                    },
                    y: {
                        title: {
                            display: true,
                            text: "Y Position",
                        },
                    },
                },
                plugins: {
                    title: {
                        display: true,
                        text: "Ball Movement on Field",
                    },
                    tooltip: {
                        callbacks: {
                            label: function (context) {
                                return `Position: (${context.parsed.x.toFixed(1)}, ${context.parsed.y.toFixed(1)})`;
                            },
                        },
                    },
                },
            },
        });
    }

    displayPlayerStats(results) {
        const playerStats = document.getElementById("playerStats");
        const { metadata, ndarrayResult } = results;

        if (
            !metadata.replay_meta.players ||
            metadata.replay_meta.players.length === 0
        ) {
            playerStats.innerHTML = "<p>Player data not available</p>";
            return;
        }

        const playerHeaders =
            ndarrayResult.metadata.column_headers.player_headers;
        const data = ndarrayResult.array_data;

        // Calculate basic stats for each player
        const playerStatsHtml = metadata.replay_meta.players
            .map((player, playerIndex) => {
                // Find boost-related columns for this player
                const boostColumnIndex = playerHeaders.findIndex(
                    (h) =>
                        h.includes(`Player${playerIndex}`) &&
                        h.includes("boost"),
                );

                let avgBoost = "N/A";
                if (boostColumnIndex !== -1) {
                    const globalColumnsCount =
                        ndarrayResult.metadata.column_headers.global_headers
                            .length;
                    const actualBoostIndex =
                        globalColumnsCount + boostColumnIndex;
                    const boostValues = data
                        .map((row) => row[actualBoostIndex])
                        .filter((val) => val !== undefined);
                    avgBoost =
                        boostValues.length > 0
                            ? (
                                  boostValues.reduce((a, b) => a + b, 0) /
                                  boostValues.length
                              ).toFixed(1)
                            : "N/A";
                }

                return `
                <div class="stat-card">
                    <h4>${player.name}</h4>
                    <p><strong>Team:</strong> ${player.team}</p>
                    <p><strong>Score:</strong> ${player.score || "N/A"}</p>
                    <p><strong>Avg Boost:</strong> ${avgBoost}%</p>
                </div>
            `;
            })
            .join("");

        playerStats.innerHTML = `
            <div class="stats-grid">
                ${playerStatsHtml}
            </div>
        `;
    }

    displayRawData(ndarrayResult) {
        const table = document.getElementById("rawDataTable");
        const headers = [
            ...ndarrayResult.metadata.column_headers.global_headers,
            ...ndarrayResult.metadata.column_headers.player_headers,
        ];

        const data = ndarrayResult.array_data.slice(0, 10); // First 10 rows

        const headerRow = headers
            .slice(0, 10)
            .map((h) => `<th>${h}</th>`)
            .join(""); // First 10 columns
        const dataRows = data
            .map((row) => {
                const cells = row
                    .slice(0, 10)
                    .map((cell) => `<td>${cell.toFixed(3)}</td>`)
                    .join("");
                return `<tr>${cells}</tr>`;
            })
            .join("");

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
        if (bytes === 0) return "0 Bytes";
        const k = 1024;
        const sizes = ["Bytes", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }

    formatDuration(seconds) {
        if (!seconds) return "N/A";
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, "0")}`;
    }

    showProgress(show) {
        const progressBar = document.getElementById("progressBar");
        progressBar.style.display = show ? "block" : "none";
        if (!show) {
            this.updateProgress(0, "");
        }
    }

    updateProgress(percent, message) {
        const progressFill = document.getElementById("progressFill");
        progressFill.style.width = `${percent}%`;

        if (message) {
            console.log(`Progress: ${percent}% - ${message}`);
        }
    }

    showError(message) {
        const uploadArea = document.getElementById("uploadArea");
        uploadArea.innerHTML = `
            <div class="error">
                <strong>Error:</strong> ${message}
            </div>
            <button class="btn" onclick="location.reload()">Try Again</button>
        `;
    }

    hideError() {
        // Reset upload area if needed
        const uploadArea = document.getElementById("uploadArea");
        if (uploadArea.innerHTML.includes("error")) {
            location.reload();
        }
    }

    setupPlayback(results) {
        const { ndarrayResult, metadata } = results;
        const data = ndarrayResult.array_data;
        const globalHeaders =
            ndarrayResult.metadata.column_headers.global_headers;
        const playerHeaders =
            ndarrayResult.metadata.column_headers.player_headers;

        // Extract position data for ball and players
        this.playbackData = this.extractPositionData(
            data,
            globalHeaders,
            playerHeaders,
            metadata,
        );

        // Initialize playback controls
        this.initializePlaybackControls();

        // Set up player elements in SVG using real team data
        const realPlayers = [];

        // Add team_zero players
        if (metadata.replay_meta.team_zero) {
            metadata.replay_meta.team_zero.forEach((player) => {
                realPlayers.push({
                    name: player.name || `Team 0 Player`,
                    team: 0,
                });
            });
        }

        // Add team_one players
        if (metadata.replay_meta.team_one) {
            metadata.replay_meta.team_one.forEach((player) => {
                realPlayers.push({
                    name: player.name || `Team 1 Player`,
                    team: 1,
                });
            });
        }

        console.log("Real players with teams:", realPlayers);
        this.setupPlayerElements(realPlayers);

        // Set initial frame
        this.currentFrame = 0;
        this.isPlaying = false;
        this.playbackSpeed = 1;
        this.updatePlaybackFrame();
    }

    extractPositionData(data, globalHeaders, playerHeaders, metadata) {
        // Debug: Log the headers to see what we're working with
        console.log("Global headers:", globalHeaders);
        console.log("Player headers:", playerHeaders);
        console.log(
            "Players metadata team_zero:",
            metadata.replay_meta.team_zero,
        );
        console.log(
            "Players metadata team_one:",
            metadata.replay_meta.team_one,
        );

        // Find ball position indices - looking for "Ball - position x" format
        const ballXIndex = globalHeaders.findIndex(
            (h) =>
                h.includes("Ball - position x") ||
                h.includes("Ball position x"),
        );
        const ballYIndex = globalHeaders.findIndex(
            (h) =>
                h.includes("Ball - position y") ||
                h.includes("Ball position y"),
        );
        const ballZIndex = globalHeaders.findIndex(
            (h) =>
                h.includes("Ball - position z") ||
                h.includes("Ball position z"),
        );

        console.log("Ball indices:", { ballXIndex, ballYIndex, ballZIndex });

        // Extract player position indices
        // The player headers show the template for one player
        // But the actual data contains multiple players worth of data

        console.log("Player headers length:", playerHeaders.length);
        console.log(
            "Total data columns expected:",
            globalHeaders.length + playerHeaders.length,
        );

        // Calculate actual number of players from the data row length
        const playerDataSize = playerHeaders.length; // 14 properties per player
        const playerDataColumns = data[0].length - globalHeaders.length; // Total columns minus global columns
        const numPlayers = Math.floor(playerDataColumns / playerDataSize);

        console.log(
            `Calculated ${numPlayers} players from ${playerDataColumns} player data columns (${playerDataSize} properties each)`,
        );

        const playerPositions = [];

        // Create players based on calculated number
        for (let playerIndex = 0; playerIndex < numPlayers; playerIndex++) {
            const baseOffset = playerIndex * playerDataSize;
            const posXIndex = baseOffset + 0; // position x offset within player data
            const posYIndex = baseOffset + 1; // position y offset within player data
            const posZIndex = baseOffset + 2; // position z offset within player data

            playerPositions.push({
                name: `Player ${playerIndex}`,
                team: playerIndex % 2, // Alternate teams
                posXIndex: globalHeaders.length + posXIndex,
                posYIndex: globalHeaders.length + posYIndex,
                posZIndex: globalHeaders.length + posZIndex,
            });
        }

        console.log("Player positions configuration:", playerPositions);

        // Process each frame
        const frames = data.map((row, frameIndex) => {
            const ballData = {
                x: ballXIndex !== -1 ? row[ballXIndex] : 0,
                y: ballYIndex !== -1 ? row[ballYIndex] : 0,
                z: ballZIndex !== -1 ? row[ballZIndex] : 0,
            };

            const playersData = playerPositions.map((playerInfo) => ({
                name: playerInfo.name,
                team: playerInfo.team,
                x: playerInfo.posXIndex !== -1 ? row[playerInfo.posXIndex] : 0,
                y: playerInfo.posYIndex !== -1 ? row[playerInfo.posYIndex] : 0,
                z: playerInfo.posZIndex !== -1 ? row[playerInfo.posZIndex] : 0,
            }));

            // Debug first frame
            if (frameIndex === 0) {
                console.log("First frame ball data:", ballData);
                console.log("First frame players data:", playersData);
                console.log("Row data length:", row.length);
            }

            const frame = {
                ball: ballData,
                players: playersData,
            };
            return frame;
        });

        return frames;
    }

    setupPlayerElements(players) {
        const svg = document.getElementById("playbackSvg");

        // Remove existing player elements
        svg.querySelectorAll(".player-car").forEach((el) => el.remove());

        // Add player elements
        if (players && players.length > 0) {
            players.forEach((player, index) => {
                const playerElement = document.createElementNS(
                    "http://www.w3.org/2000/svg",
                    "circle",
                );
                playerElement.setAttribute("id", `player-${index}`);
                playerElement.setAttribute(
                    "class",
                    `player-car team-${player.team}`,
                );
                playerElement.setAttribute("r", "25");
                playerElement.setAttribute("cx", "4000");
                playerElement.setAttribute("cy", "2000");
                svg.appendChild(playerElement);

                // Add player name label
                const nameLabel = document.createElementNS(
                    "http://www.w3.org/2000/svg",
                    "text",
                );
                nameLabel.setAttribute("id", `player-name-${index}`);
                nameLabel.setAttribute("x", "4000");
                nameLabel.setAttribute("y", "2000");
                nameLabel.setAttribute("text-anchor", "middle");
                nameLabel.setAttribute("font-size", "96");
                nameLabel.setAttribute("fill", "white");
                nameLabel.setAttribute("font-weight", "bold");
                nameLabel.textContent = player.name;
                svg.appendChild(nameLabel);
            });
        }
    }

    initializePlaybackControls() {
        const playPauseBtn = document.getElementById("playPauseBtn");
        const resetBtn = document.getElementById("resetBtn");
        const speedSlider = document.getElementById("speedSlider");
        const speedValue = document.getElementById("speedValue");
        const timelineSlider = document.getElementById("timelineSlider");

        // Set up timeline slider
        if (this.playbackData && this.playbackData.length > 0) {
            timelineSlider.max = this.playbackData.length - 1;
            timelineSlider.value = 0;

            // Update total time display
            const totalTime = document.getElementById("totalTime");
            const durationSeconds = this.playbackData.length / 10; // Assuming 10 FPS
            totalTime.textContent = this.formatTime(durationSeconds);
        } else {
            // No playback data available
            timelineSlider.max = 0;
            timelineSlider.value = 0;
            const totalTime = document.getElementById("totalTime");
            totalTime.textContent = "0:00";
        }

        // Event listeners
        playPauseBtn.addEventListener("click", () => this.togglePlayback());
        resetBtn.addEventListener("click", () => this.resetPlayback());

        speedSlider.addEventListener("input", (e) => {
            this.playbackSpeed = parseFloat(e.target.value);
            speedValue.textContent = `${this.playbackSpeed}x`;
        });

        timelineSlider.addEventListener("input", (e) => {
            this.currentFrame = parseInt(e.target.value);
            this.updatePlaybackFrame();
        });
    }

    togglePlayback() {
        const playPauseBtn = document.getElementById("playPauseBtn");

        if (this.isPlaying) {
            this.isPlaying = false;
            playPauseBtn.textContent = "Play";
            if (this.playbackInterval) {
                clearInterval(this.playbackInterval);
            }
        } else {
            this.isPlaying = true;
            playPauseBtn.textContent = "Pause";
            this.startPlayback();
        }
    }

    startPlayback() {
        const frameRate = 10; // 10 FPS
        const interval = 1000 / frameRate / this.playbackSpeed;

        this.playbackInterval = setInterval(() => {
            this.currentFrame++;

            if (this.currentFrame >= this.playbackData.length) {
                this.currentFrame = this.playbackData.length - 1;
                this.togglePlayback(); // Stop playback at end
                return;
            }

            this.updatePlaybackFrame();
        }, interval);
    }

    resetPlayback() {
        this.currentFrame = 0;
        this.isPlaying = false;
        document.getElementById("playPauseBtn").textContent = "Play";

        if (this.playbackInterval) {
            clearInterval(this.playbackInterval);
        }

        this.updatePlaybackFrame();
    }

    updatePlaybackFrame() {
        if (
            !this.playbackData ||
            this.currentFrame >= this.playbackData.length
        ) {
            return;
        }

        const frame = this.playbackData[this.currentFrame];

        // Update ball position
        const ball = document.getElementById("ball");
        if (ball && frame.ball) {
            // Swap X and Y coordinates - game X becomes SVG Y, game Y becomes SVG X
            const ballSvgX = this.convertYPosition(frame.ball.y);
            const ballSvgY = this.convertXPosition(frame.ball.x);

            // Debug ball position on first few frames
            if (this.currentFrame < 5) {
                console.log(
                    `Frame ${this.currentFrame}: Ball game coords (${frame.ball.x}, ${frame.ball.y}) -> SVG coords (${ballSvgX}, ${ballSvgY})`,
                );
            }

            ball.setAttribute("cx", ballSvgX);
            ball.setAttribute("cy", ballSvgY);
        }

        // Update player positions
        frame.players.forEach((player, index) => {
            const playerElement = document.getElementById(`player-${index}`);
            const nameLabel = document.getElementById(`player-name-${index}`);

            if (playerElement && nameLabel) {
                // Swap X and Y coordinates - game X becomes SVG Y, game Y becomes SVG X
                const playerSvgX = this.convertYPosition(player.y);
                const playerSvgY = this.convertXPosition(player.x);

                // Debug player position on first few frames
                if (this.currentFrame < 5) {
                    console.log(
                        `Frame ${this.currentFrame}: Player ${index} game coords (${player.x}, ${player.y}) -> SVG coords (${playerSvgX}, ${playerSvgY})`,
                    );
                }

                playerElement.setAttribute("cx", playerSvgX);
                playerElement.setAttribute("cy", playerSvgY);
                nameLabel.setAttribute("x", playerSvgX);
                nameLabel.setAttribute("y", playerSvgY - 50); // Position name above player
            }
        });

        // Update timeline
        const timelineSlider = document.getElementById("timelineSlider");
        const currentTime = document.getElementById("currentTime");

        timelineSlider.value = this.currentFrame;
        const currentSeconds = this.currentFrame / 10; // Assuming 10 FPS
        currentTime.textContent = this.formatTime(currentSeconds);
    }

    convertXPosition(gameX) {
        // Convert game coordinates to SVG coordinates
        // In Rocket League, X runs along the length of the field (goals at ends)
        // SVG field is 8000 units wide, so X maps to SVG Y
        // Rocket League field is approximately 10000 units long (-5000 to 5000)
        // SVG field is 4000 units tall (0 to 4000)
        return 2000 + gameX * 0.4; // Center at 2000, scale to fit height
    }

    convertYPosition(gameY) {
        // Convert game coordinates to SVG coordinates
        // In Rocket League, Y runs across the width of the field
        // SVG field is 4000 units tall, so Y maps to SVG X
        // Rocket League field is approximately 8000 units wide (-4000 to 4000)
        // SVG field is 8000 units wide (0 to 8000)
        return 4000 - gameY * 0.8; // Center at 4000, flip and scale
    }

    formatTime(seconds) {
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, "0")}`;
    }
}

// Initialize the analyzer when the page loads
const analyzer = new ReplayAnalyzer();

// Initialize WASM and start the app
analyzer
    .initialize()
    .then(() => {
        console.log("Replay analyzer ready!");
    })
    .catch((error) => {
        console.error("Failed to initialize:", error);
    });
