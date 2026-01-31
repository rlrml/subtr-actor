import init, {
    validate_replay,
    get_replay_info,
    get_replay_meta,
    get_replay_frames_data,
} from "../../pkg/rl_replay_subtr_actor";
import wasmUrl from "../../pkg/rl_replay_subtr_actor_bg.wasm?url";

class ReplayAnalyzer {
    constructor() {
        this.wasmInitialized = false;
        this.setupEventListeners();
    }

    async initialize() {
        try {
            await init(wasmUrl);
            this.wasmInitialized = true;
            console.log("WASM module loaded successfully!");
        } catch (error) {
            console.error("Failed to initialize WASM:", error);
            this.showError("Failed to initialize WebAssembly module");
        }
    }

    setupEventListeners() {
        const uploadArea = document.getElementById("uploadArea");
        const fileInput = document.getElementById("fileInput");

        fileInput.addEventListener("change", (e) => {
            if (e.target.files.length > 0) {
                this.processFile(e.target.files[0]);
            }
        });

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
    }

    mapToObject(map) {
        if (!map || typeof map.get !== "function") {
            return map;
        }

        const obj = {};
        for (const [key, value] of map) {
            if (typeof value === "object" && value && typeof value.get === "function") {
                obj[key] = this.mapToObject(value);
            } else if (Array.isArray(value)) {
                obj[key] = value.map((item) =>
                    typeof item === "object" && item && typeof item.get === "function"
                        ? this.mapToObject(item)
                        : item
                );
            } else {
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

        try {
            const arrayBuffer = await file.arrayBuffer();
            const replayData = new Uint8Array(arrayBuffer);

            this.updateProgress(20, "Validating replay file...");
            const validation = validate_replay(replayData);
            const isValid = validation.get ? validation.get("valid") : validation.valid;
            const errorMsg = validation.get ? validation.get("error") : validation.error;

            if (!isValid) {
                throw new Error(`Invalid replay file: ${errorMsg || "Unknown error"}`);
            }

            this.updateProgress(40, "Getting replay information...");
            const info = this.mapToObject(get_replay_info(replayData));

            this.updateProgress(60, "Processing frame data...");
            const frameData = this.mapToObject(get_replay_frames_data(replayData, 30.0));

            this.updateProgress(80, "Getting metadata...");
            const metadata = this.mapToObject(get_replay_meta(replayData));

            this.updateProgress(100, "Complete!");

            this.displayResults({ info, frameData, metadata, fileName: file.name, fileSize: file.size });
        } catch (error) {
            console.error("Error processing replay:", error);
            this.showError(`Failed to process replay: ${error.message}`);
        } finally {
            this.showProgress(false);
        }
    }

    displayResults(results) {
        document.getElementById("uploadCard").style.display = "none";
        document.getElementById("results").style.display = "block";
        this.setupPlayback(results);
    }

    setupPlayback(results) {
        const { frameData } = results;
        const adaptedData = this.adaptFrameData(frameData);
        window.replayData = adaptedData;
        this.initializeBallchasingPlayer();
    }

    adaptFrameData(frameData) {
        const { frame_data, meta } = frameData;
        const { ball_data, players, metadata_frames } = frame_data;

        const frameRate = 30;
        const maxTime = metadata_frames.length / frameRate;

        // Build ball data
        const ballData = {
            start: 0,
            end: maxTime,
            times: metadata_frames.map((_, i) => i / frameRate),
            pos: [],
            quat: [],
        };

        ball_data.frames.forEach((ballFrame) => {
            if (ballFrame.Data && ballFrame.Data.rigid_body) {
                const rb = ballFrame.Data.rigid_body;
                ballData.pos.push(rb.location.x, rb.location.y, rb.location.z);
                ballData.quat.push(rb.rotation.x, rb.rotation.y, rb.rotation.z, rb.rotation.w);
            } else {
                ballData.pos.push(0, 0, 0);
                ballData.quat.push(0, 0, 0, 1);
            }
        });

        // Build player data
        const adaptedPlayers = [];
        const allPlayerMeta = [...(meta.team_zero || []), ...(meta.team_one || [])];

        players.forEach(([playerId, playerData], playerIndex) => {
            const playerMeta = allPlayerMeta[playerIndex];
            const playerName = playerMeta?.name || `Player ${playerIndex}`;
            const team = playerMeta?.team === 0 ? "blue" : "orange";

            const times = metadata_frames.map((_, i) => i / frameRate);
            const positions = [];
            const quaternions = [];
            const boostValues = [];

            // Track boost state for trails
            const boostStartTimes = [];
            const boostEndTimes = [];
            let boostWasActive = false;
            let boostStartTime = 0;

            playerData.frames.forEach((playerFrame, frameIndex) => {
                const time = frameIndex / frameRate;

                if (playerFrame.Data && playerFrame.Data.rigid_body) {
                    const rb = playerFrame.Data.rigid_body;
                    positions.push(rb.location.x, rb.location.y, rb.location.z);
                    quaternions.push(rb.rotation.x, rb.rotation.y, rb.rotation.z, rb.rotation.w);

                    // Convert boost from 0-255 range to 0-100 percentage
                    const boostPercent = Math.round(playerFrame.Data.boost_amount / 2.55);
                    boostValues.push(Math.min(100, Math.max(0, boostPercent)));

                    // Track boost activation for trails
                    const boostActive = playerFrame.Data.boost_active;
                    if (boostActive && !boostWasActive) {
                        boostStartTime = time;
                        boostWasActive = true;
                    } else if (!boostActive && boostWasActive) {
                        boostStartTimes.push(boostStartTime);
                        boostEndTimes.push(time);
                        boostWasActive = false;
                    }
                } else {
                    positions.push(0, 0, 0);
                    quaternions.push(0, 0, 0, 1);
                    boostValues.push(0);

                    if (boostWasActive) {
                        boostStartTimes.push(boostStartTime);
                        boostEndTimes.push(time);
                        boostWasActive = false;
                    }
                }
            });

            // Close any open boost period at the end
            if (boostWasActive) {
                boostStartTimes.push(boostStartTime);
                boostEndTimes.push(maxTime);
            }

            adaptedPlayers.push({
                player: playerName,
                team: team,
                color: team === "blue" ? 0x209cee : 0xff9f43,
                cars: [{
                    start: 0,
                    end: maxTime,
                    times: times,
                    pos: positions,
                    quat: quaternions,
                }],
                boost_amount: {
                    times: times,
                    values: boostValues,
                },
                boost_state: {
                    start: boostStartTimes,
                    end: boostEndTimes,
                },
                tracks: {},
                events: {},
            });
        });

        return {
            map: meta.map_name || "unknown",
            map_type: "soccar",
            max_time: maxTime,
            ball_type: "sphere",
            balls: [ballData],
            players: adaptedPlayers,
            countdowns: [],
            rem_seconds: { times: [], rem_seconds: [] },
            blue_score: { times: [], score: [] },
            orange_score: { times: [], score: [] },
            boost_pads: [],
            ticks: [],
        };
    }

    initializeBallchasingPlayer() {
        this.loadPlayerDependencies()
            .then(() => {
                document.getElementById("player-container").style.display = "block";
                document.getElementById("details-watch").style.display = "block";

                const settings = window.Settings({
                    "player.autoplay": false,
                    "player.speed": 3, // Start at 0.5x speed (index 3 in speed array)
                    "cars.colors.simple": false,
                    "cars.name.hide": false,
                    "cars.bam.hide": false,
                    "boost.pads.hide": true,
                    "cars.boost.trail.hide": false,
                    "cars.trails": false,
                    "ball.trail": false,
                    "trail.duration": 1,
                });

                window.ReplayPlayer({ settings: settings, slave: null });
                console.log("Ballchasing.com player initialized successfully!");
            })
            .catch((error) => {
                console.error("Failed to load ballchasing.com player:", error);
            });
    }

    async loadPlayerDependencies() {
        if (!window.THREE) {
            await this.loadScript("https://cdnjs.cloudflare.com/ajax/libs/three.js/r116/three.min.js");
        }

        if (!window.Stats) {
            try {
                await this.loadScript("https://unpkg.com/stats.js@0.17.0/build/stats.min.js");
            } catch (e) {
                window.Stats = function () {
                    return { begin() {}, end() {}, showPanel() {}, dom: document.createElement("div") };
                };
            }
        }

        if (!window.hotkeys) {
            try {
                await this.loadScript("https://unpkg.com/hotkeys-js@3.8.1/dist/hotkeys.min.js");
            } catch (e) {
                window.hotkeys = function (keys, callback) {
                    document.addEventListener("keydown", (e) => {
                        if (keys.includes(e.key.toLowerCase())) {
                            callback(e, { key: e.key });
                        }
                    });
                };
            }
        }

        this.addCookieFunctions();
        this.addMissingDOMElements();
        await this.loadScript("./src/player.js");
    }

    addCookieFunctions() {
        window.readCookie = function (name) {
            const nameEQ = name + "=";
            const ca = document.cookie.split(";");
            for (let i = 0; i < ca.length; i++) {
                let c = ca[i];
                while (c.charAt(0) === " ") c = c.substring(1, c.length);
                if (c.indexOf(nameEQ) === 0) return c.substring(nameEQ.length, c.length);
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

        this.createParticleTexture();
    }

    createParticleTexture() {
        const canvas = document.createElement("canvas");
        canvas.width = 64;
        canvas.height = 64;
        const ctx = canvas.getContext("2d");

        const centerX = canvas.width / 2;
        const centerY = canvas.height / 2;
        const radius = 30;

        const gradient = ctx.createRadialGradient(centerX, centerY, 0, centerX, centerY, radius);
        gradient.addColorStop(0, "rgba(255, 255, 255, 1)");
        gradient.addColorStop(0.5, "rgba(255, 255, 255, 0.5)");
        gradient.addColorStop(1, "rgba(255, 255, 255, 0)");

        ctx.fillStyle = gradient;
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        if (window.THREE && window.THREE.TextureLoader) {
            const texture = new THREE.CanvasTexture(canvas);
            texture.needsUpdate = true;

            const originalLoad = THREE.TextureLoader.prototype.load;
            THREE.TextureLoader.prototype.load = function (url, onLoad, onProgress, onError) {
                if (url === "/static/textures/solid-particle.png" || url === "/static/textures/lensflare0_alpha.png") {
                    if (onLoad) onLoad(texture);
                    return texture;
                }
                return originalLoad.call(this, url, onLoad, onProgress, onError);
            };
        }
    }

    loadScript(src) {
        return new Promise((resolve, reject) => {
            const script = document.createElement("script");
            script.src = src;
            script.onload = () => {
                console.log(`Loaded: ${src}`);
                resolve();
            };
            script.onerror = (error) => {
                console.error(`Failed to load: ${src}`, error);
                reject(error);
            };
            document.head.appendChild(script);
        });
    }

    formatFileSize(bytes) {
        if (bytes === 0) return "0 Bytes";
        const k = 1024;
        const sizes = ["Bytes", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
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
}

const analyzer = new ReplayAnalyzer();
analyzer.initialize()
    .then(() => console.log("Replay analyzer ready!"))
    .catch((error) => console.error("Failed to initialize:", error));
