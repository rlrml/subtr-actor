import init, {
    validate_replay,
    get_replay_info,
    get_ndarray_with_info,
    get_replay_meta,
    get_column_headers,
    get_replay_frames_data,
} from "../../pkg/rl_replay_subtr_actor";
import wasmUrl from "../../pkg/rl_replay_subtr_actor_bg.wasm?url";

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

            this.updateProgress(60, "Processing frame data...");

            // Get structured frame data using ReplayDataCollector at 60 FPS
            const frameData = this.mapToObject(get_replay_frames_data(replayData, 60.0));

            this.updateProgress(80, "Getting metadata...");

            // Get metadata and convert from Map to plain object
            const metadata = this.mapToObject(get_replay_meta(replayData));

            this.updateProgress(100, "Complete!");

            // Display results
            this.displayResults({
                info,
                frameData,
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
        const statsCard = document.querySelector(".card h2");
        if (statsCard && statsCard.textContent.includes("Replay Statistics")) {
            statsCard.parentElement.style.display = "none";
        }

        // Hide all the other tabs and show only playback
        document.querySelectorAll(".tab").forEach((tab) => {
            if (tab.dataset.tab !== "playback") {
                tab.style.display = "none";
            } else {
                tab.classList.add("active");
            }
        });

        document.querySelectorAll(".tab-content").forEach((content) => {
            if (content.id !== "playback") {
                content.style.display = "none";
            } else {
                content.classList.add("active");
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
            { label: "Data Points", value: results.frameData.frame_data.metadata_frames.length },
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
                    <div class="stat-value">${results.frameData.frame_data.metadata_frames.length}</div>
                    <div class="stat-label">Frame Count</div>
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
        const { frameData, metadata } = results;

        // Transform subtr-actor data to ballchasing.com player format
        const adaptedData = this.adaptFrameData(frameData);

        // Store the adapted data globally for the player
        window.replayData = adaptedData;

        // Initialize the ballchasing.com player
        this.initializeBallchasingPlayer();
    }

    adaptFrameData(frameData) {
        const { frame_data, meta } = frameData;
        const { ball_data, players, metadata_frames } = frame_data;

        // Calculate timing based on metadata frames
        const frameRate = 60; // Using 60 FPS for smooth playback
        const maxTime = metadata_frames.length / frameRate;

        // Extract ball data from frame data
        const ballData = {
            start: 0,
            end: maxTime,
            times: metadata_frames.map((_, i) => i / frameRate),
            pos: [],
            quat: [],
        };

        // Fill ball position and quaternion data from ball_data.frames
        ball_data.frames.forEach((ballFrame) => {
            if (ballFrame.Data && ballFrame.Data.rigid_body) {
                const rb = ballFrame.Data.rigid_body;
                ballData.pos.push(rb.location.x, rb.location.y, rb.location.z);
                ballData.quat.push(rb.rotation.x, rb.rotation.y, rb.rotation.z, rb.rotation.w);
            } else {
                // Empty frame
                ballData.pos.push(0, 0, 0);
                ballData.quat.push(0, 0, 0, 1);
            }
        });

        // Extract player data
        const adaptedPlayers = [];
        
        players.forEach(([playerId, playerData], playerIndex) => {
            // Get player name and team from metadata
            let playerName = `Player ${playerIndex}`;
            let team = playerIndex % 2 === 0 ? "blue" : "orange";

            // Try to get real player info from meta
            const allPlayers = [
                ...(meta.team_zero || []),
                ...(meta.team_one || []),
            ];

            if (allPlayers[playerIndex]) {
                playerName = allPlayers[playerIndex].name || playerName;
                team = allPlayers[playerIndex].team === 0 ? "blue" : "orange";
            }

            const adaptedPlayer = {
                player: playerName,
                team: team,
                color: team === "blue" ? 0x209cee : 0xff9f43,
                cars: [
                    {
                        start: 0,
                        end: maxTime,
                        times: metadata_frames.map((_, i) => i / frameRate),
                        pos: [],
                        quat: [],
                    },
                ],
                boost_amount: {
                    times: metadata_frames.map((_, i) => i / frameRate),
                    values: [],
                },
                boost_state: {
                    start: [],
                    end: [],
                },
                tracks: {},
                events: {},
            };

            // Fill player position, quaternion, and boost data
            playerData.frames.forEach((playerFrame) => {
                if (playerFrame.Data && playerFrame.Data.rigid_body) {
                    const rb = playerFrame.Data.rigid_body;
                    adaptedPlayer.cars[0].pos.push(rb.location.x, rb.location.y, rb.location.z);
                    adaptedPlayer.cars[0].quat.push(rb.rotation.x, rb.rotation.y, rb.rotation.z, rb.rotation.w);
                    adaptedPlayer.boost_amount.values.push(Math.floor(playerFrame.Data.boost_amount * 100));
                } else {
                    // Empty frame
                    adaptedPlayer.cars[0].pos.push(0, 0, 0);
                    adaptedPlayer.cars[0].quat.push(0, 0, 0, 1);
                    adaptedPlayer.boost_amount.values.push(0);
                }
            });

            adaptedPlayers.push(adaptedPlayer);
        });

        // Create the adapted data structure
        const adaptedData = {
            map: meta.map_name || "unknown",
            map_type: "soccar", // Default to soccar for now
            max_time: maxTime,
            ball_type: "sphere",
            balls: [ballData],
            players: adaptedPlayers,
            countdowns: [],
            rem_seconds: {
                times: [],
                rem_seconds: [],
            },
            blue_score: {
                times: [],
                score: [],
            },
            orange_score: {
                times: [],
                score: [],
            },
            boost_pads: [],
            ticks: [],
        };

        // Debug: Final verification of adapted data structure
        console.log("Final adapted data structure:", {
            playersCount: adaptedData.players.length,
            firstPlayerBoost: adaptedData.players[0]?.boost_amount,
            ballDataLength: adaptedData.balls[0]?.times.length,
        });

        return adaptedData;
    }

    initializeBallchasingPlayer() {
        // Load required dependencies for the ballchasing.com player
        this.loadPlayerDependencies()
            .then(() => {
                // Show the player container and hide the simple playback
                document.getElementById("player-container").style.display =
                    "block";
                document.getElementById("simple-playback").style.display =
                    "none";
                document.getElementById("details-watch").style.display =
                    "block";

                // Initialize the settings and player
                const settings = window.Settings({
                    "player.autoplay": false,
                    "cars.colors.simple": false,
                    "cars.name.hide": false,
                    "cars.bam.hide": true,
                    "boost.pads.hide": true,
                    "cars.boost.trail.hide": true,
                    "cars.trails": false,
                    "ball.trail": false,
                    "trail.duration": 1,
                });

                // Create the player
                try {
                    const player = window.ReplayPlayer({
                        settings: settings,
                        slave: null,
                    });

                    console.log("Ballchasing.com player initialized successfully!");
                } catch (error) {
                    console.error("Error initializing ballchasing.com player:", error);
                    console.error("Stack trace:", error.stack);
                    throw error; // Re-throw to trigger the catch block
                }
            })
            .catch((error) => {
                console.error("Failed to load ballchasing.com player:", error);
                // Keep the simple playback as fallback
                console.log("Using simple playback as fallback");
            });
    }

    async loadThreeJS() {
        // Load Three.js
        if (!window.THREE) {
            await this.loadScript(
                "https://cdnjs.cloudflare.com/ajax/libs/three.js/r116/three.min.js",
            );
        }
    }

    initializeCustom3DPlayer() {
        const container = document.getElementById("player");
        const adaptedData = window.replayData;

        // Create the 3D scene
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x2a5234); // Field green

        // Create camera
        const camera = new THREE.PerspectiveCamera(
            75,
            container.clientWidth / container.clientHeight,
            0.1,
            10000,
        );
        camera.position.set(0, -3000, 2000);
        camera.lookAt(0, 0, 0);

        // Create renderer
        const renderer = new THREE.WebGLRenderer();
        renderer.setSize(container.clientWidth, container.clientHeight);
        renderer.shadowMap.enabled = true;
        renderer.shadowMap.type = THREE.PCFSoftShadowMap;
        container.appendChild(renderer.domElement);

        // Create field
        this.createField(scene);

        // Create ball
        const ballGeometry = new THREE.SphereGeometry(93, 16, 16);
        const ballMaterial = new THREE.MeshLambertMaterial({ color: 0xffffff });
        const ball = new THREE.Mesh(ballGeometry, ballMaterial);
        ball.castShadow = true;
        scene.add(ball);

        // Create players
        const players = [];
        adaptedData.players.forEach((playerData, index) => {
            const playerGeometry = new THREE.BoxGeometry(200, 400, 80);
            const playerMaterial = new THREE.MeshLambertMaterial({
                color: playerData.team === "blue" ? 0x209cee : 0xff9f43,
            });
            const player = new THREE.Mesh(playerGeometry, playerMaterial);
            player.castShadow = true;
            scene.add(player);
            players.push(player);
        });

        // Add lighting
        const ambientLight = new THREE.AmbientLight(0x404040, 0.6);
        scene.add(ambientLight);

        const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
        directionalLight.position.set(0, 0, 1000);
        directionalLight.castShadow = true;
        scene.add(directionalLight);

        // Animation variables
        let currentFrame = 0;
        let isPlaying = false;
        let animationId;

        // Animation function
        const animate = () => {
            if (isPlaying && adaptedData.balls.length > 0) {
                const ballData = adaptedData.balls[0];
                const frameIndex = Math.floor(currentFrame);

                if (frameIndex < ballData.times.length) {
                    // Update ball position
                    const ballPosIndex = frameIndex * 3;
                    ball.position.set(
                        ballData.pos[ballPosIndex] || 0,
                        ballData.pos[ballPosIndex + 1] || 0,
                        ballData.pos[ballPosIndex + 2] || 0,
                    );

                    // Update player positions
                    adaptedData.players.forEach((playerData, index) => {
                        if (
                            playerData.cars &&
                            playerData.cars.length > 0 &&
                            players[index]
                        ) {
                            const carData = playerData.cars[0];
                            const playerPosIndex = frameIndex * 3;
                            if (playerPosIndex < carData.pos.length) {
                                players[index].position.set(
                                    carData.pos[playerPosIndex] || 0,
                                    carData.pos[playerPosIndex + 1] || 0,
                                    carData.pos[playerPosIndex + 2] || 0,
                                );
                            }
                        }
                    });

                    currentFrame += 0.1; // Slow down playback
                    if (currentFrame >= ballData.times.length) {
                        isPlaying = false;
                    }
                }
            }

            renderer.render(scene, camera);
            animationId = requestAnimationFrame(animate);
        };

        // Start animation
        animate();

        // Set up controls
        const playPauseBtn = document.getElementById("play-pause");
        const seekBar = document.getElementById("seekbar");

        playPauseBtn.addEventListener("click", () => {
            isPlaying = !isPlaying;
            playPauseBtn.querySelector("i").className = isPlaying
                ? "fa fa-pause"
                : "fa fa-play";
        });

        seekBar.addEventListener("input", (e) => {
            const progress = parseFloat(e.target.value) / 1000;
            if (adaptedData.balls.length > 0) {
                currentFrame = progress * adaptedData.balls[0].times.length;
            }
        });

        // Handle window resize
        window.addEventListener("resize", () => {
            camera.aspect = container.clientWidth / container.clientHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(container.clientWidth, container.clientHeight);
        });

        console.log("Custom 3D player setup complete");
    }

    createField(scene) {
        // Create field geometry
        const fieldGeometry = new THREE.PlaneGeometry(8240, 10280);
        const fieldMaterial = new THREE.MeshLambertMaterial({
            color: 0x2a5234,
        });
        const field = new THREE.Mesh(fieldGeometry, fieldMaterial);
        field.rotation.x = -Math.PI / 2;
        field.receiveShadow = true;
        scene.add(field);

        // Create field lines
        const lineMaterial = new THREE.LineBasicMaterial({ color: 0xffffff });

        // Center line
        const centerLineGeometry = new THREE.BufferGeometry().setFromPoints([
            new THREE.Vector3(0, -5140, 1),
            new THREE.Vector3(0, 5140, 1),
        ]);
        const centerLine = new THREE.Line(centerLineGeometry, lineMaterial);
        scene.add(centerLine);

        // Center circle
        const centerCircleGeometry = new THREE.RingGeometry(900, 920, 64);
        const centerCircleMaterial = new THREE.MeshBasicMaterial({
            color: 0xffffff,
        });
        const centerCircle = new THREE.Mesh(
            centerCircleGeometry,
            centerCircleMaterial,
        );
        centerCircle.rotation.x = -Math.PI / 2;
        centerCircle.position.z = 1;
        scene.add(centerCircle);

        // Goals
        const goalGeometry = new THREE.BoxGeometry(1900, 20, 800);
        const goalMaterial = new THREE.MeshLambertMaterial({ color: 0xffffff });

        const goal1 = new THREE.Mesh(goalGeometry, goalMaterial);
        goal1.position.set(0, 5140, 400);
        scene.add(goal1);

        const goal2 = new THREE.Mesh(goalGeometry, goalMaterial);
        goal2.position.set(0, -5140, 400);
        scene.add(goal2);
    }

    addCookieFunctions() {
        // Add cookie functions that the ballchasing.com player expects
        window.readCookie = function (name) {
            const nameEQ = name + "=";
            const ca = document.cookie.split(";");
            for (let i = 0; i < ca.length; i++) {
                let c = ca[i];
                while (c.charAt(0) === " ") c = c.substring(1, c.length);
                if (c.indexOf(nameEQ) === 0)
                    return c.substring(nameEQ.length, c.length);
            }
            return null;
        };

        window.createCookie = function (name, value, days) {
            let expires = "";
            if (days) {
                const date = new Date();
                date.setTime(date.getTime() + days * 24 * 60 * 60 * 1000);
                expires = "; expires=" + date.toUTCString();
            }
            document.cookie = name + "=" + value + expires + "; path=/";
        };
    }

    addMissingDOMElements() {
        // Add shader elements that the player expects - must be added BEFORE the player script loads
        if (!document.getElementById("particle-vertex-shader")) {
            const vertexShader = document.createElement("script");
            vertexShader.id = "particle-vertex-shader";
            vertexShader.type = "x-shader/x-vertex";
            vertexShader.textContent = `
                attribute float size;
                attribute float age;
                varying float vAge;
                void main() {
                    vAge = age;
                    vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
                    gl_PointSize = size * (300.0 / -mvPosition.z);
                    gl_Position = projectionMatrix * mvPosition;
                }
            `;
            document.head.appendChild(vertexShader);
        }

        if (!document.getElementById("particle-fragment-shader")) {
            const fragmentShader = document.createElement("script");
            fragmentShader.id = "particle-fragment-shader";
            fragmentShader.type = "x-shader/x-fragment";
            fragmentShader.textContent = `
                uniform sampler2D texture;
                varying float vAge;
                void main() {
                    gl_FragColor = vec4(color, vAge) * texture2D(texture, gl_PointCoord);
                }
            `;
            document.head.appendChild(fragmentShader);
        }

        // Create the missing texture by creating a simple white circle
        this.createParticleTexture();
    }

    createParticleTexture() {
        // Create a canvas to generate the particle texture
        const canvas = document.createElement("canvas");
        canvas.width = 64;
        canvas.height = 64;
        const ctx = canvas.getContext("2d");

        // Create a white circle with alpha falloff
        const centerX = canvas.width / 2;
        const centerY = canvas.height / 2;
        const radius = 30;

        const gradient = ctx.createRadialGradient(
            centerX,
            centerY,
            0,
            centerX,
            centerY,
            radius,
        );
        gradient.addColorStop(0, "rgba(255, 255, 255, 1)");
        gradient.addColorStop(0.5, "rgba(255, 255, 255, 0.5)");
        gradient.addColorStop(1, "rgba(255, 255, 255, 0)");

        ctx.fillStyle = gradient;
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        // Create a texture from the canvas
        if (window.THREE && window.THREE.TextureLoader) {
            const texture = new THREE.CanvasTexture(canvas);
            texture.needsUpdate = true;

            // Mock the texture paths that the player expects
            const originalLoad = THREE.TextureLoader.prototype.load;
            THREE.TextureLoader.prototype.load = function (
                url,
                onLoad,
                onProgress,
                onError,
            ) {
                if (
                    url === "/static/textures/solid-particle.png" ||
                    url === "/static/textures/lensflare0_alpha.png"
                ) {
                    if (onLoad) onLoad(texture);
                    return texture;
                }
                return originalLoad.call(
                    this,
                    url,
                    onLoad,
                    onProgress,
                    onError,
                );
            };
        }
    }

    async loadPlayerDependencies() {
        // Load Three.js (use r116 which is more compatible with the ballchasing.com player)
        if (!window.THREE) {
            await this.loadScript(
                "https://cdnjs.cloudflare.com/ajax/libs/three.js/r116/three.min.js",
            );
        }

        // Create a simple Stats mock if Stats.js fails to load
        if (!window.Stats) {
            try {
                await this.loadScript(
                    "https://unpkg.com/stats.js@0.17.0/build/stats.min.js",
                );
            } catch (e) {
                console.warn("Stats.js failed to load, creating mock");
                window.Stats = function () {
                    return {
                        begin: function () {},
                        end: function () {},
                        showPanel: function () {},
                        dom: document.createElement("div"),
                    };
                };
            }
        }

        // Create a simple hotkeys mock if hotkeys.js fails to load
        if (!window.hotkeys) {
            try {
                await this.loadScript(
                    "https://unpkg.com/hotkeys-js@3.8.1/dist/hotkeys.min.js",
                );
            } catch (e) {
                console.warn("Hotkeys.js failed to load, creating mock");
                window.hotkeys = function (keys, callback) {
                    document.addEventListener("keydown", function (e) {
                        if (keys.includes(e.key.toLowerCase())) {
                            callback(e, { key: e.key });
                        }
                    });
                };
            }
        }

        // CRITICAL: Add DOM elements and mock functions BEFORE loading the player script
        this.addCookieFunctions();
        this.addMissingDOMElements();

        // Load the player script
        await this.loadScript("./src/player.js");
    }

    loadScript(src) {
        return new Promise((resolve, reject) => {
            const script = document.createElement("script");
            script.src = src;
            script.onload = () => {
                console.log(`Successfully loaded: ${src}`);
                resolve();
            };
            script.onerror = (error) => {
                console.error(`Failed to load: ${src}`, error);
                reject(error);
            };
            document.head.appendChild(script);
        });
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
