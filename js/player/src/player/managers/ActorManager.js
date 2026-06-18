import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { getCarHitboxInfo } from "../data/hitboxes.js";
import { resolvePlayerAssetUrl } from "../asset-url.js";
import { CarModelLoader } from "./CarModelLoader.js";

// Seconds to hide the ball after a goal so it vanishes inside the explosion
// (covers the goal explosion's ~1.8s lifetime). Recomputed every frame, so a
// skipped/scrubbed-past celebration never leaves the ball stuck hidden.
const GOAL_BALL_HIDE_DURATION = 2.0;

export class ActorManager {
  constructor(scene, effectsManager, options = {}) {
    this.scene = scene;
    this.effectsManager = effectsManager;
    this.assetBase = options.assetBase;
    this.actors = {}; // actorId -> Mesh
    this.ballActorId = null;
    this.ballIndicator = null;
    this.ballVerticalLine = null;

    this.playerNames = new Set();
    this.actorToPlayer = {}; // actorId -> playerName
    this.actorLinks = {}; // actorId -> Set(targetActorIds)
    this.playerNameToCarActorId = {}; // playerName -> currentCarActorId
    this.playerNameToPriActorId = {}; // playerName -> PRI actorId (for camera state)
    this.playerTeams = {}; // playerName -> team (0 or 1)
    this.actorLoadouts = {}; // actorId -> TeamLoadout

    // Car/hitbox family comes from subtr-actor per player (PlayerInfo.car_hitbox_family);
    // getCarHitboxInfo(bodyId) is only a fallback for the legacy body-id path.
    this.carBodyIds = {}; // actorId -> bodyId

    // Car model loader for FBX models
    this.carModelLoader = new CarModelLoader({ assetBase: this.assetBase });
    this.pendingCarReplacements = new Map(); // actorId -> hitboxType (cars waiting for model)

    // Goal-explosion bookkeeping. We fire each goal's explosion the first time
    // forward playback crosses its time, then suppress re-fires until a backward
    // seek/loop. Keyed by goal time so it survives across replays.
    this._lastGoalScanTime = null;
    this._firedGoalTimes = new Set();

    // Reusable vectors for interpolation
    this._p0 = new THREE.Vector3();
    this._p1 = new THREE.Vector3();
    this._v0 = new THREE.Vector3();
    this._v1 = new THREE.Vector3();
    this._nextRot = new THREE.Quaternion();
    this._q0 = new THREE.Quaternion();
    this._q1 = new THREE.Quaternion();
    this._qResult = new THREE.Quaternion();

    // Callback for UI updates (player list)
    this.onPlayerFound = null;

    // Ball trail tracking
    this.lastBallTouchTeam = 0; // 0 = blue, 1 = orange
    this.BALL_TOUCH_DISTANCE = 200; // Distance threshold for ball touch detection (in UU)

    // Timeline data for interpolation
    this.ballTimeline = [];
    this.playerTimelineMap = {};
    this.timelineIndices = { ball: 0, players: {} };
    this.interpolantsInitialized = false;

    // Three.js Animation System for smooth replay playback
    // Using KeyframeTracks handles interpolation automatically and correctly
    this.animationMixer = null;
    this.animationActions = {}; // entityId -> AnimationAction
    this.animationClock = new THREE.Clock(false); // Don't auto-start
    this.replayDuration = 0;
    // DISABLED: Animation system has timing issues with seek/pause
    // Manual LERP interpolation works correctly
    this.useAnimationSystem = false;

    // Position smoothing buffers (moving average filter)
    // Reduces jitter caused by irregular keyframe spacing
    this.SMOOTHING_WINDOW = 5; // Number of frames to average
    this.positionBuffers = {}; // entityId -> { buffer: [], index: 0 }
    this.rotationBuffers = {}; // entityId -> { buffer: [], index: 0 }

    // Debug: Interpolation settings
    this.interpolationEnabled = true;
    // Match Ballcam's production player default. Alternative methods remain
    // useful for debugging/A-B testing, but the public player ships with plain
    // lerp between replay samples.
    this.interpolationMethod = "lerp";
    this.smoothingWindowSize = 12;
    this.lastFrameInfo = null;

    // Low-pass filter state for 'position-smooth' method
    this._lowPassState = new Map(); // entityId -> { x, y, z }
    this._lowPassAlpha = 0.3; // Smoothing factor (0-1): lower = smoother

    // Predict-correct (dead reckoning) state
    this._predictState = new Map(); // entityId -> { lastPos, lastVel, lastTime, correctionStart, correctionTarget }
    this._predictCorrectionTime = 0.1; // Time to blend correction (100ms)

    // Smoothing buffers for lerp-smooth method
    this._smoothingBuffers = new Map(); // entityId -> position buffer

    // Adaptive smooth state - tracks velocity to adjust smoothing dynamically
    this._adaptiveState = new Map(); // entityId -> { buffer: [], lastPos, lastTime, derivedVel }

    // Load ball GLTF model - store promise so we can await it before shader compilation
    this.ballModel = null;
    this._ballModelReplaced = false; // Track if ball model has been replaced
    const gltfLoader = new GLTFLoader();
    this.ballModelReady = new Promise((resolve) => {
      gltfLoader.load(
        resolvePlayerAssetUrl("models/ball/scene.gltf", this.assetBase),
        (gltf) => {
          this.ballModel = gltf.scene;
          console.log("✓ Ball model loaded");
          // Note: Don't replace ball mesh here - let GameEngine control when it's safe
          // This prevents race conditions with shader compilation
          resolve(true);
        },
        undefined,
        (error) => {
          console.error("Failed to load ball model:", error);
          resolve(false); // Resolve even on error to not block shader compilation
        },
      );
    });
  }

  /**
   * Wait for ball model to be loaded, then replace the ball mesh if not already done.
   * This should be called BEFORE shader compilation to ensure all meshes are ready.
   * @returns {Promise<boolean>} - true if loaded successfully, false otherwise
   */
  async waitForBallModel() {
    const success = await this.ballModelReady;
    // Replace ball mesh now that we're explicitly waiting for it
    if (success && !this._ballModelReplaced && this.ballActorId && this.actors[this.ballActorId]) {
      this.replaceBallWithModel(this.ballActorId);
      this._ballModelReplaced = true;
    }
    return success;
  }

  replaceBallWithModel(actorId) {
    const oldMesh = this.actors[actorId];
    if (!oldMesh || !this.ballModel) return;

    // Clone the model
    const newMesh = this.ballModel.clone();

    // Copy userData from old mesh
    newMesh.userData = oldMesh.userData;

    // Copy transform
    newMesh.position.copy(oldMesh.position);
    newMesh.quaternion.copy(oldMesh.quaternion);
    newMesh.scale.copy(oldMesh.scale);

    // Scale to match ball size (92.75 units radius - official Rocket League size)
    // Adjust this scaling factor if needed based on the model's original size
    const ballScale = 92.75;
    newMesh.scale.set(ballScale, ballScale, ballScale);

    // Enable shadow casting and receiving on all meshes
    newMesh.traverse((child) => {
      if (child.isMesh) {
        child.castShadow = true;
        child.receiveShadow = true;
      }
    });

    // Replace in scene
    this.scene.remove(oldMesh);
    this.scene.add(newMesh);

    // Dispose old mesh
    if (oldMesh.geometry) oldMesh.geometry.dispose();
    if (oldMesh.material) oldMesh.material.dispose();

    // Update reference
    this.actors[actorId] = newMesh;

    console.log("✓ Ball replaced with GLTF model");
  }

  reset() {
    // Remove all actors from scene
    Object.values(this.actors).forEach((mesh) => {
      this.scene.remove(mesh);
      if (mesh.geometry) mesh.geometry.dispose();
      if (mesh.material) mesh.material.dispose();
    });
    this.actors = {};
    this.ballActorId = null;
    if (this.ballIndicator) {
      this.scene.remove(this.ballIndicator);
      if (this.ballIndicator.geometry) this.ballIndicator.geometry.dispose();
      if (this.ballIndicator.material) this.ballIndicator.material.dispose();
      this.ballIndicator = null;
    }
    if (this.ballVerticalLine) {
      this.scene.remove(this.ballVerticalLine);
      if (this.ballVerticalLine.geometry) this.ballVerticalLine.geometry.dispose();
      if (this.ballVerticalLine.material) this.ballVerticalLine.material.dispose();
      this.ballVerticalLine = null;
    }
    this.actorToPlayer = {};
    this.actorLinks = {};
    this.playerNameToCarActorId = {};
    this.playerNameToPriActorId = {};
    this.actorLoadouts = {};
    this._lastGoalScanTime = null;
    this._firedGoalTimes.clear();
    // playerTeams are static per replay, usually
  }

  setPlayerTeams(teams) {
    this.playerTeams = teams;
  }

  /**
   * Initialize actors from framework Player API (static mesh creation)
   * This replaces the old processFrame approach - meshes are created once at load time
   * @param {Player} player - Framework Player instance
   */
  initFromFramework(player) {
    console.log("[ActorManager] Initializing actors from framework...");

    // Create ball mesh
    this._createBallMesh();

    // Create car meshes for each player
    const playerList = player.playerList;
    playerList.forEach((playerInfo, index) => {
      this._createCarMesh(
        playerInfo.name,
        playerInfo.team,
        index,
        playerInfo.carName,
        playerInfo.hitboxType,
      );
      //scale car
      const carActorId = this.playerNameToCarActorId[playerInfo.name];
      const carActor = this.actors[carActorId];
    });

    console.log(`[ActorManager] Created ${playerList.length} car meshes + 1 ball`);
  }

  /**
   * Initialize interpolation system with timeline data
   * Uses Three.js AnimationMixer for smooth playback (handles variable frame deltas correctly)
   * @param {Object} timelines - { ballTimeline, playerTimelines } from framework
   */
  initInterpolants(timelines) {
    console.log("[ActorManager] Initializing interpolation system...");

    // Store raw timelines for fallback manual interpolation
    this.ballTimeline = timelines.ballTimeline || [];
    this.playerTimelineMap = timelines.playerTimelines || {};

    // Create CORRECTED timelines for physics-sim mode
    // This fixes time-shifted positions by extrapolating with velocity
    this.ballTimelineCorrected = this._correctTimeShiftedPositions(this.ballTimeline);
    this.playerTimelineMapCorrected = {};
    Object.entries(this.playerTimelineMap).forEach(([name, timeline]) => {
      this.playerTimelineMapCorrected[name] = this._correctTimeShiftedPositions(timeline);
    });

    // Create FILTERED timelines for time-shifted mode (removes bad frames)
    this.ballTimelineFiltered = this._filterBadFrames(this.ballTimeline);
    this.playerTimelineMapFiltered = {};
    Object.entries(this.playerTimelineMap).forEach(([name, timeline]) => {
      this.playerTimelineMapFiltered[name] = this._filterBadFrames(timeline);
    });

    // Create search index cache for faster binary search (fallback)
    this.timelineIndices = {
      ball: 0,
      players: {},
    };
    this.timelineIndicesFiltered = {
      ball: 0,
      players: {},
    };
    this.timelineIndicesCorrected = {
      ball: 0,
      players: {},
    };
    Object.keys(this.playerTimelineMap).forEach((name) => {
      this.timelineIndices.players[name] = 0;
      this.timelineIndicesFiltered.players[name] = 0;
      this.timelineIndicesCorrected.players[name] = 0;
    });

    // Calculate replay duration
    if (this.ballTimeline.length > 0) {
      this.replayDuration = this.ballTimeline[this.ballTimeline.length - 1].time;
    }

    // Initialize Three.js Animation System
    if (this.useAnimationSystem) {
      this._initAnimationSystem();
    }

    this.interpolantsInitialized = true;
    const filteredBallCount = this.ballTimeline.length - this.ballTimelineFiltered.length;
    const correctedBallCount = this.ballTimelineCorrected._correctedCount || 0;
    console.log(
      `  Ball: ${this.ballTimeline.length} keyframes (${correctedBallCount} corrected, ${filteredBallCount} filtered)`,
    );
    Object.entries(this.playerTimelineMap).forEach(([name, timeline]) => {
      const correctedCount = this.playerTimelineMapCorrected[name]?._correctedCount || 0;
      console.log(`  ${name}: ${timeline.length} keyframes (${correctedCount} corrected)`);
    });
    console.log(`  Replay duration: ${this.replayDuration.toFixed(2)}s`);
    console.log("[ActorManager] Animation system ready");
  }

  /**
   * Initialize Three.js AnimationMixer and create KeyframeTracks for all entities
   * This is the recommended approach for smooth replay playback with variable frame deltas
   */
  _initAnimationSystem() {
    console.log("[ActorManager] Building Three.js animation clips...");

    // Create a root object for the mixer (we'll use the scene)
    this.animationMixer = new THREE.AnimationMixer(this.scene);

    // Create animation clip for ball
    const ball = this.actors[this.ballActorId];
    if (ball && this.ballTimeline.length > 0) {
      const clip = this._createAnimationClip("ball", this.ballTimeline, ball);
      if (clip) {
        const action = this.animationMixer.clipAction(clip, ball);
        action.setLoop(THREE.LoopOnce);
        action.clampWhenFinished = true;
        this.animationActions["ball"] = action;
        console.log(`  ✓ Ball animation: ${clip.duration.toFixed(2)}s`);
      }
    }

    // Create animation clips for each player
    Object.entries(this.playerTimelineMap).forEach(([playerName, timeline]) => {
      const carId = this.playerNameToCarActorId[playerName];
      const mesh = this.actors[carId];

      if (mesh && timeline.length > 0) {
        const clip = this._createAnimationClip(playerName, timeline, mesh);
        if (clip) {
          const action = this.animationMixer.clipAction(clip, mesh);
          action.setLoop(THREE.LoopOnce);
          action.clampWhenFinished = true;
          this.animationActions[playerName] = action;
          console.log(`  ✓ ${playerName} animation: ${clip.duration.toFixed(2)}s`);
        }
      }
    });

    console.log("[ActorManager] Animation clips ready");
  }

  /**
   * Create a Three.js AnimationClip from timeline data
   * @param {string} name - Clip name
   * @param {Array} timeline - Array of {time, position, rotation, velocity}
   * @param {THREE.Object3D} target - Target mesh
   * @returns {THREE.AnimationClip|null}
   */
  _createAnimationClip(name, timeline, target) {
    if (!timeline || timeline.length < 2) return null;

    // Extract times and values
    const times = [];
    const positions = [];
    const quaternions = [];

    // If timeline doesn't start at t=0, add a synthetic keyframe at t=0
    // using the first real keyframe's position to prevent interpolation from (0,0,0)
    const firstKeyframe = timeline[0];
    if (firstKeyframe.time > 0) {
      times.push(0);
      if (firstKeyframe.position) {
        positions.push(
          firstKeyframe.position.x,
          firstKeyframe.position.y,
          firstKeyframe.position.z,
        );
      } else {
        positions.push(0, 0, 0);
      }
      if (firstKeyframe.rotation) {
        quaternions.push(
          firstKeyframe.rotation.x,
          firstKeyframe.rotation.y,
          firstKeyframe.rotation.z,
          firstKeyframe.rotation.w,
        );
      } else {
        quaternions.push(0, 0, 0, 1);
      }
    }

    for (const keyframe of timeline) {
      times.push(keyframe.time);

      // Position values (flat array: x, y, z, x, y, z, ...)
      if (keyframe.position) {
        positions.push(keyframe.position.x, keyframe.position.y, keyframe.position.z);
      } else {
        // Use previous position or zero
        const lastIdx = positions.length - 3;
        if (lastIdx >= 0) {
          positions.push(positions[lastIdx], positions[lastIdx + 1], positions[lastIdx + 2]);
        } else {
          positions.push(0, 0, 0);
        }
      }

      // Quaternion values (flat array: x, y, z, w, x, y, z, w, ...)
      if (keyframe.rotation) {
        quaternions.push(
          keyframe.rotation.x,
          keyframe.rotation.y,
          keyframe.rotation.z,
          keyframe.rotation.w,
        );
      } else {
        // Use previous rotation or identity
        const lastIdx = quaternions.length - 4;
        if (lastIdx >= 0) {
          quaternions.push(
            quaternions[lastIdx],
            quaternions[lastIdx + 1],
            quaternions[lastIdx + 2],
            quaternions[lastIdx + 3],
          );
        } else {
          quaternions.push(0, 0, 0, 1);
        }
      }
    }

    // Create KeyframeTracks
    // THREE.InterpolateLinear is the default and handles variable time steps correctly
    const positionTrack = new THREE.VectorKeyframeTrack(
      ".position",
      times,
      positions,
      THREE.InterpolateLinear,
    );

    const rotationTrack = new THREE.QuaternionKeyframeTrack(
      ".quaternion",
      times,
      quaternions,
      // Quaternion tracks use SLERP by default
    );

    // Create the clip
    const duration = times[times.length - 1] - times[0];
    const clip = new THREE.AnimationClip(name, duration, [positionTrack, rotationTrack]);

    return clip;
  }

  /**
   * Start all animations (call when replay starts playing)
   */
  startAnimations() {
    if (!this.animationMixer) return;

    Object.values(this.animationActions).forEach((action) => {
      action.reset();
      action.play();
    });
    this.animationClock.start();
    console.log("[ActorManager] Animations started");
  }

  /**
   * Pause all animations
   */
  pauseAnimations() {
    if (!this.animationMixer) return;
    Object.values(this.animationActions).forEach((action) => {
      action.paused = true;
    });
  }

  /**
   * Resume animations
   */
  resumeAnimations() {
    if (!this.animationMixer) return;
    Object.values(this.animationActions).forEach((action) => {
      action.paused = false;
    });
  }

  /**
   * Seek animations to a specific time
   * @param {number} time - Time in seconds
   */
  seekAnimations(time) {
    if (!this.animationMixer) return;

    Object.values(this.animationActions).forEach((action) => {
      // Set the time directly on the action
      action.time = time;
    });

    // Force update to apply the position at this time
    this.animationMixer.setTime(time);
  }

  /**
   * Update the animation mixer (call every frame)
   * @param {number} delta - Time delta in seconds
   */
  updateAnimations(delta) {
    if (!this.animationMixer || !this.useAnimationSystem) return;
    this.animationMixer.update(delta);
  }

  /**
   * Sub-sample a timeline by taking every 2nd keyframe
   * This reduces the alternating acceleration pattern caused by
   * what appears to be two interleaved update sources in the replay
   * @param {Array} timeline - Original timeline array
   * @returns {Array} Sub-sampled timeline (half the keyframes)
   */
  _subsampleTimeline(timeline) {
    if (!timeline || timeline.length < 4) return timeline;
    // Take every 2nd keyframe (even indices: 0, 2, 4, ...)
    return timeline.filter((_, i) => i % 2 === 0);
  }

  /**
   * Initialize or get a smoothing buffer for an entity
   * @param {string} entityId - Entity identifier (ball or player name)
   * @returns {Object} Buffer object with { positions: [], rotations: [] }
   */
  _getOrCreateSmoothingBuffer(entityId) {
    if (!this.positionBuffers[entityId]) {
      this.positionBuffers[entityId] = [];
      this.rotationBuffers[entityId] = [];
    }
    return {
      positions: this.positionBuffers[entityId],
      rotations: this.rotationBuffers[entityId],
    };
  }

  /**
   * Add a position to the smoothing buffer and return smoothed position
   * Uses moving average over SMOOTHING_WINDOW frames
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Position {x, y, z}
   * @returns {Object} Smoothed position {x, y, z}
   */
  _smoothPosition(entityId, pos) {
    const buffer = this._getOrCreateSmoothingBuffer(entityId).positions;

    // Add new position
    buffer.push({ x: pos.x, y: pos.y, z: pos.z });

    // Keep only last N positions
    while (buffer.length > this.SMOOTHING_WINDOW) {
      buffer.shift();
    }

    // Calculate average
    if (buffer.length === 1) return pos;

    let sumX = 0,
      sumY = 0,
      sumZ = 0;
    for (const p of buffer) {
      sumX += p.x;
      sumY += p.y;
      sumZ += p.z;
    }
    return {
      x: sumX / buffer.length,
      y: sumY / buffer.length,
      z: sumZ / buffer.length,
    };
  }

  /**
   * Add a rotation to the smoothing buffer and return smoothed rotation
   * Uses SLERP-based averaging
   * @param {string} entityId - Entity identifier
   * @param {Object} rot - Rotation quaternion {x, y, z, w}
   * @returns {Object} Smoothed rotation {x, y, z, w}
   */
  _smoothRotation(entityId, rot) {
    const buffer = this._getOrCreateSmoothingBuffer(entityId).rotations;

    // Add new rotation
    buffer.push({ x: rot.x, y: rot.y, z: rot.z, w: rot.w });

    // Keep only last N rotations
    while (buffer.length > this.SMOOTHING_WINDOW) {
      buffer.shift();
    }

    // For rotations, just use the middle one to avoid quaternion averaging complexity
    // This still provides some smoothing without gimbal lock issues
    if (buffer.length < 3) return rot;
    const midIdx = Math.floor(buffer.length / 2);
    return buffer[midIdx];
  }

  /**
   * Reset smoothing buffers (call when seeking)
   */
  resetSmoothingBuffers() {
    this.positionBuffers = {};
    this.rotationBuffers = {};
    this._lowPassState.clear(); // Reset low-pass filter state for position-smooth method
  }

  /**
   * Find the keyframe index for a given time using cached binary search
   * @param {Array} timeline - Timeline array with {time} entries
   * @param {number} time - Current time
   * @param {number} lastIndex - Last known index (for cache)
   * @returns {number} Index of the keyframe just before or at time
   */
  _findKeyframeIndex(timeline, time, lastIndex = 0) {
    if (!timeline || timeline.length === 0) return -1;
    if (time <= timeline[0].time) return 0;
    if (time >= timeline[timeline.length - 1].time) return timeline.length - 2;

    // Start from cached index and search nearby first (temporal coherence)
    let idx = Math.max(0, Math.min(lastIndex, timeline.length - 2));

    // Check if we're still in the same segment
    if (timeline[idx].time <= time && timeline[idx + 1].time > time) {
      return idx;
    }

    // Check next segment (common case: moving forward in time)
    if (
      idx + 2 < timeline.length &&
      timeline[idx + 1].time <= time &&
      timeline[idx + 2].time > time
    ) {
      return idx + 1;
    }

    // Fall back to binary search
    let low = 0;
    let high = timeline.length - 2;
    while (low <= high) {
      const mid = Math.floor((low + high) / 2);
      if (timeline[mid].time <= time && timeline[mid + 1].time > time) {
        return mid;
      } else if (timeline[mid].time > time) {
        high = mid - 1;
      } else {
        low = mid + 1;
      }
    }
    return Math.max(0, Math.min(low, timeline.length - 2));
  }

  /**
   * Apply smoothing filter to a position (moving average)
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Position {x, y, z}
   * @returns {Object} Smoothed position
   */
  _applySmoothing(entityId, pos) {
    if (!this._smoothingBuffers.has(entityId)) {
      this._smoothingBuffers.set(entityId, []);
    }

    const buffer = this._smoothingBuffers.get(entityId);
    buffer.push({ x: pos.x, y: pos.y, z: pos.z });

    // Keep only last N positions
    while (buffer.length > this.smoothingWindowSize) {
      buffer.shift();
    }

    if (buffer.length === 1) return pos;

    // Calculate moving average
    let sumX = 0,
      sumY = 0,
      sumZ = 0;
    for (const p of buffer) {
      sumX += p.x;
      sumY += p.y;
      sumZ += p.z;
    }

    return {
      x: sumX / buffer.length,
      y: sumY / buffer.length,
      z: sumZ / buffer.length,
    };
  }

  /**
   * Apply exponential moving average (EMA) smoothing
   * Less latency than simple moving average, weights recent values more
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Position {x, y, z}
   * @returns {Object} Smoothed position
   */
  _applyEmaSmoothing(entityId, pos) {
    const key = `ema-${entityId}`;
    if (!this._smoothingBuffers.has(key)) {
      this._smoothingBuffers.set(key, { x: pos.x, y: pos.y, z: pos.z });
      return pos;
    }

    const prev = this._smoothingBuffers.get(key);
    // Alpha controls smoothing: lower = smoother but more lag
    // Map windowSize (2-20) to alpha (0.5-0.05)
    const alpha = Math.max(0.05, Math.min(0.5, 1 / this.smoothingWindowSize));

    const smoothed = {
      x: alpha * pos.x + (1 - alpha) * prev.x,
      y: alpha * pos.y + (1 - alpha) * prev.y,
      z: alpha * pos.z + (1 - alpha) * prev.z,
    };

    this._smoothingBuffers.set(key, smoothed);
    return smoothed;
  }

  /**
   * Apply double exponential smoothing (Holt's method)
   * Predicts trend and reduces lag while maintaining smoothness
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Position {x, y, z}
   * @returns {Object} Smoothed position
   */
  _applyDoubleEmaSmoothing(entityId, pos) {
    const key = `dema-${entityId}`;
    if (!this._smoothingBuffers.has(key)) {
      this._smoothingBuffers.set(key, {
        level: { x: pos.x, y: pos.y, z: pos.z },
        trend: { x: 0, y: 0, z: 0 },
      });
      return pos;
    }

    const state = this._smoothingBuffers.get(key);
    // Alpha for level, beta for trend
    const alpha = Math.max(0.1, Math.min(0.6, 2 / this.smoothingWindowSize));
    const beta = alpha * 0.5; // Trend smoothing is slower

    const newLevel = {
      x: alpha * pos.x + (1 - alpha) * (state.level.x + state.trend.x),
      y: alpha * pos.y + (1 - alpha) * (state.level.y + state.trend.y),
      z: alpha * pos.z + (1 - alpha) * (state.level.z + state.trend.z),
    };

    const newTrend = {
      x: beta * (newLevel.x - state.level.x) + (1 - beta) * state.trend.x,
      y: beta * (newLevel.y - state.level.y) + (1 - beta) * state.trend.y,
      z: beta * (newLevel.z - state.level.z) + (1 - beta) * state.trend.z,
    };

    state.level = newLevel;
    state.trend = newTrend;

    // Output includes trend prediction (reduces lag)
    return {
      x: newLevel.x + newTrend.x,
      y: newLevel.y + newTrend.y,
      z: newLevel.z + newTrend.z,
    };
  }

  /**
   * Apply weighted moving average (recent frames count more)
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Position {x, y, z}
   * @returns {Object} Smoothed position
   */
  _applyWeightedSmoothing(entityId, pos) {
    const key = `wma-${entityId}`;
    if (!this._smoothingBuffers.has(key)) {
      this._smoothingBuffers.set(key, []);
    }

    const buffer = this._smoothingBuffers.get(key);
    buffer.push({ x: pos.x, y: pos.y, z: pos.z });

    while (buffer.length > this.smoothingWindowSize) {
      buffer.shift();
    }

    if (buffer.length === 1) return pos;

    // Weighted average: weight = index + 1 (newer = higher weight)
    let sumX = 0,
      sumY = 0,
      sumZ = 0,
      totalWeight = 0;
    for (let i = 0; i < buffer.length; i++) {
      const weight = i + 1;
      sumX += buffer[i].x * weight;
      sumY += buffer[i].y * weight;
      sumZ += buffer[i].z * weight;
      totalWeight += weight;
    }

    return {
      x: sumX / totalWeight,
      y: sumY / totalWeight,
      z: sumZ / totalWeight,
    };
  }

  /**
   * Apply Gaussian-weighted smoothing (bell curve weights)
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Position {x, y, z}
   * @returns {Object} Smoothed position
   */
  _applyGaussianSmoothing(entityId, pos) {
    const key = `gauss-${entityId}`;
    if (!this._smoothingBuffers.has(key)) {
      this._smoothingBuffers.set(key, []);
    }

    const buffer = this._smoothingBuffers.get(key);
    buffer.push({ x: pos.x, y: pos.y, z: pos.z });

    while (buffer.length > this.smoothingWindowSize) {
      buffer.shift();
    }

    if (buffer.length === 1) return pos;

    // Gaussian weights centered on the most recent value
    const sigma = buffer.length / 3;
    let sumX = 0,
      sumY = 0,
      sumZ = 0,
      totalWeight = 0;

    for (let i = 0; i < buffer.length; i++) {
      // Distance from the end (most recent)
      const dist = buffer.length - 1 - i;
      const weight = Math.exp(-(dist * dist) / (2 * sigma * sigma));
      sumX += buffer[i].x * weight;
      sumY += buffer[i].y * weight;
      sumZ += buffer[i].z * weight;
      totalWeight += weight;
    }

    return {
      x: sumX / totalWeight,
      y: sumY / totalWeight,
      z: sumZ / totalWeight,
    };
  }

  /**
   * Adaptive smoothing based on derived velocity from position differences
   * This method adapts the smoothing level based on actual movement:
   * - Slow movement: more smoothing (reduces micro-jitter)
   * - Fast movement: less smoothing (maintains responsiveness)
   * - Direction changes: reduces buffer to avoid lag
   *
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Position {x, y, z}
   * @param {number} time - Current playback time
   * @returns {Object} Smoothed position
   */
  _applyAdaptiveSmoothing(entityId, pos, time) {
    if (!this._adaptiveState.has(entityId)) {
      this._adaptiveState.set(entityId, {
        buffer: [{ x: pos.x, y: pos.y, z: pos.z }],
        lastPos: { x: pos.x, y: pos.y, z: pos.z },
        lastTime: time,
        derivedVel: { x: 0, y: 0, z: 0 },
      });
      return pos;
    }

    const state = this._adaptiveState.get(entityId);
    const dt = time - state.lastTime;

    // Calculate derived velocity from position change (ignore replay velocity)
    if (dt > 0.001) {
      state.derivedVel = {
        x: (pos.x - state.lastPos.x) / dt,
        y: (pos.y - state.lastPos.y) / dt,
        z: (pos.z - state.lastPos.z) / dt,
      };
    }

    // Calculate speed (in UU/s)
    const speed = Math.sqrt(
      state.derivedVel.x ** 2 + state.derivedVel.y ** 2 + state.derivedVel.z ** 2,
    );

    // Adaptive window size based on speed:
    // - Slow (< 500 UU/s): use full window (more smoothing)
    // - Fast (> 2000 UU/s): use minimal window (less smoothing)
    // Speed range in Rocket League: 0-2300 UU/s for cars, ball can go faster
    const minWindow = 2;
    const maxWindow = this.smoothingWindowSize;
    const speedThresholdLow = 300;
    const speedThresholdHigh = 1500;

    let adaptiveWindow;
    if (speed < speedThresholdLow) {
      adaptiveWindow = maxWindow;
    } else if (speed > speedThresholdHigh) {
      adaptiveWindow = minWindow;
    } else {
      // Linear interpolation between thresholds
      const t = (speed - speedThresholdLow) / (speedThresholdHigh - speedThresholdLow);
      adaptiveWindow = Math.round(maxWindow - t * (maxWindow - minWindow));
    }

    // Detect sudden direction change (dot product of velocities)
    if (state.buffer.length >= 2) {
      const prev = state.buffer[state.buffer.length - 1];
      const prevPrev = state.buffer[state.buffer.length - 2];
      const prevVel = {
        x: prev.x - prevPrev.x,
        y: prev.y - prevPrev.y,
        z: prev.z - prevPrev.z,
      };
      const currVel = {
        x: pos.x - prev.x,
        y: pos.y - prev.y,
        z: pos.z - prev.z,
      };

      // Normalize and compute dot product
      const prevMag = Math.sqrt(prevVel.x ** 2 + prevVel.y ** 2 + prevVel.z ** 2);
      const currMag = Math.sqrt(currVel.x ** 2 + currVel.y ** 2 + currVel.z ** 2);

      if (prevMag > 0.1 && currMag > 0.1) {
        const dot =
          (prevVel.x * currVel.x + prevVel.y * currVel.y + prevVel.z * currVel.z) /
          (prevMag * currMag);

        // If direction changed significantly (dot < 0.5 means > 60 degrees)
        if (dot < 0.5) {
          // Clear most of the buffer to reduce lag on direction changes
          while (state.buffer.length > Math.max(2, adaptiveWindow / 2)) {
            state.buffer.shift();
          }
        }
      }
    }

    // Add to buffer
    state.buffer.push({ x: pos.x, y: pos.y, z: pos.z });

    // Trim buffer to adaptive window size
    while (state.buffer.length > adaptiveWindow) {
      state.buffer.shift();
    }

    // Update state
    state.lastPos = { x: pos.x, y: pos.y, z: pos.z };
    state.lastTime = time;

    // Calculate weighted average (more recent = higher weight)
    if (state.buffer.length === 1) return pos;

    let sumX = 0,
      sumY = 0,
      sumZ = 0,
      totalWeight = 0;
    for (let i = 0; i < state.buffer.length; i++) {
      const weight = i + 1; // Linear weights
      sumX += state.buffer[i].x * weight;
      sumY += state.buffer[i].y * weight;
      sumZ += state.buffer[i].z * weight;
      totalWeight += weight;
    }

    return {
      x: sumX / totalWeight,
      y: sumY / totalWeight,
      z: sumZ / totalWeight,
    };
  }

  /**
   * One Euro Filter - adaptive filter that's smooth at low speeds, responsive at high speeds
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Position {x, y, z}
   * @returns {Object} Smoothed position
   */
  _applyOneEuroFilter(entityId, pos) {
    const key = `1euro-${entityId}`;
    if (!this._smoothingBuffers.has(key)) {
      this._smoothingBuffers.set(key, {
        x: pos.x,
        y: pos.y,
        z: pos.z,
        dx: 0,
        dy: 0,
        dz: 0,
        lastTime: performance.now(),
      });
      return pos;
    }

    const state = this._smoothingBuffers.get(key);
    const now = performance.now();
    const dt = Math.max(0.001, (now - state.lastTime) / 1000); // seconds
    state.lastTime = now;

    // Parameters tuned by smoothingWindowSize
    const minCutoff = 0.5 + (20 - this.smoothingWindowSize) * 0.1; // Higher = less smooth
    const beta = 0.01 * this.smoothingWindowSize; // Higher = more responsive to speed
    const dCutoff = 1.0;

    // Calculate derivative (velocity)
    const dx = (pos.x - state.x) / dt;
    const dy = (pos.y - state.y) / dt;
    const dz = (pos.z - state.z) / dt;

    // Filter derivative
    const alphaD = this._oneEuroAlpha(dt, dCutoff);
    const filteredDx = alphaD * dx + (1 - alphaD) * state.dx;
    const filteredDy = alphaD * dy + (1 - alphaD) * state.dy;
    const filteredDz = alphaD * dz + (1 - alphaD) * state.dz;

    // Calculate cutoff based on speed
    const speed = Math.sqrt(
      filteredDx * filteredDx + filteredDy * filteredDy + filteredDz * filteredDz,
    );
    const cutoff = minCutoff + beta * speed;

    // Filter position
    const alpha = this._oneEuroAlpha(dt, cutoff);
    const smoothed = {
      x: alpha * pos.x + (1 - alpha) * state.x,
      y: alpha * pos.y + (1 - alpha) * state.y,
      z: alpha * pos.z + (1 - alpha) * state.z,
    };

    // Update state
    state.x = smoothed.x;
    state.y = smoothed.y;
    state.z = smoothed.z;
    state.dx = filteredDx;
    state.dy = filteredDy;
    state.dz = filteredDz;

    return smoothed;
  }

  /**
   * Helper for One Euro Filter - calculate smoothing factor
   */
  _oneEuroAlpha(dt, cutoff) {
    const tau = 1.0 / (2 * Math.PI * cutoff);
    return 1.0 / (1.0 + tau / dt);
  }

  /**
   * Catmull-Rom spline interpolation (uses 4 keyframes)
   */
  _catmullRomInterpolate(p0, p1, p2, p3, t) {
    const t2 = t * t;
    const t3 = t2 * t;

    // Catmull-Rom basis functions
    const b0 = -0.5 * t3 + t2 - 0.5 * t;
    const b1 = 1.5 * t3 - 2.5 * t2 + 1;
    const b2 = -1.5 * t3 + 2 * t2 + 0.5 * t;
    const b3 = 0.5 * t3 - 0.5 * t2;

    return {
      x: b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x,
      y: b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y,
      z: b0 * p0.z + b1 * p1.z + b2 * p2.z + b3 * p3.z,
    };
  }

  /**
   * Apply low-pass filter to smooth positions
   * This filters out high-frequency noise/jitter from the replay data
   * @param {string} entityId - Entity identifier
   * @param {Object} pos - Raw interpolated position {x, y, z}
   * @returns {Object} Filtered position {x, y, z}
   */
  _applyLowPassFilter(entityId, pos) {
    if (!this._lowPassState.has(entityId)) {
      // Initialize with first position
      this._lowPassState.set(entityId, { x: pos.x, y: pos.y, z: pos.z });
      return pos;
    }

    const prev = this._lowPassState.get(entityId);
    // Map smoothingWindowSize (2-20) to alpha (0.5-0.05)
    // Higher window = lower alpha = smoother
    const alpha = Math.max(0.05, Math.min(0.5, 1 / this.smoothingWindowSize));

    // Low-pass filter: new = alpha * current + (1 - alpha) * previous
    const filtered = {
      x: alpha * pos.x + (1 - alpha) * prev.x,
      y: alpha * pos.y + (1 - alpha) * prev.y,
      z: alpha * pos.z + (1 - alpha) * prev.z,
    };

    this._lowPassState.set(entityId, filtered);
    return filtered;
  }

  /**
   * Calculate derived velocity from position differences
   * This ignores the replay's velocity data and computes velocity from positions
   * @param {Object} p0 - Previous position {x, y, z}
   * @param {Object} p1 - Current position {x, y, z}
   * @param {number} dt - Time delta between positions (seconds)
   * @returns {Object} Derived velocity {x, y, z}
   */
  _deriveVelocity(p0, p1, dt) {
    if (dt <= 0) return { x: 0, y: 0, z: 0 };
    return {
      x: (p1.x - p0.x) / dt,
      y: (p1.y - p0.y) / dt,
      z: (p1.z - p0.z) / dt,
    };
  }

  /**
   * Predict-Correct interpolation (dead reckoning)
   * Instead of interpolating between keyframes, we:
   * 1. Predict position using velocity: P_predicted = P0 + V0 * elapsed
   * 2. When approaching next keyframe, smoothly correct towards it
   *
   * This creates smoother motion because it follows physics instead of
   * interpolating between potentially inconsistent position snapshots.
   *
   * @param {string} entityId - Entity identifier
   * @param {Object} k0 - Start keyframe {position, velocity, time}
   * @param {Object} k1 - End keyframe {position, velocity, time}
   * @param {number} currentTime - Current playback time
   * @returns {Object} Predicted/corrected position {x, y, z}
   */
  _predictCorrectInterpolate(entityId, k0, k1, currentTime) {
    const elapsed = currentTime - k0.time;
    const dt = k1.time - k0.time;
    const t = elapsed / dt; // Normalized time [0, 1]

    // If no velocity data, fall back to lerp
    if (!k0.velocity) {
      return {
        x: k0.position.x + (k1.position.x - k0.position.x) * t,
        y: k0.position.y + (k1.position.y - k0.position.y) * t,
        z: k0.position.z + (k1.position.z - k0.position.z) * t,
      };
    }

    // Step 1: Predict position using velocity
    const predicted = {
      x: k0.position.x + k0.velocity.x * elapsed,
      y: k0.position.y + k0.velocity.y * elapsed,
      z: k0.position.z + k0.velocity.z * elapsed,
    };

    // Step 2: Calculate where prediction SHOULD end up (at t=1)
    const predictedAtEnd = {
      x: k0.position.x + k0.velocity.x * dt,
      y: k0.position.y + k0.velocity.y * dt,
      z: k0.position.z + k0.velocity.z * dt,
    };

    // Step 3: Calculate error between prediction and actual next keyframe
    const error = {
      x: k1.position.x - predictedAtEnd.x,
      y: k1.position.y - predictedAtEnd.y,
      z: k1.position.z - predictedAtEnd.z,
    };

    // Step 4: Smoothly blend in the correction using an ease-in curve
    // This applies more correction as we get closer to the next keyframe
    // Using smoothstep for gradual correction: 3t² - 2t³
    const correctionBlend = t * t * (3 - 2 * t);

    // Step 5: Apply correction to predicted position
    return {
      x: predicted.x + error.x * correctionBlend,
      y: predicted.y + error.y * correctionBlend,
      z: predicted.z + error.z * correctionBlend,
    };
  }

  /**
   * Velocity-based interpolation with clamped correction to avoid jitter
   * This method follows the velocity data but limits how fast corrections are applied
   * to prevent sudden acceleration/deceleration when keyframes are far apart.
   *
   * @param {string} entityId - Entity identifier for state tracking
   * @param {Object} k0 - Start keyframe {position, velocity, time}
   * @param {Object} k1 - End keyframe {position, velocity, time}
   * @param {number} currentTime - Current playback time
   * @param {boolean} isBall - Whether this is the ball (applies gravity)
   * @returns {Object} Interpolated position {x, y, z}
   */
  _velocitySmoothInterpolate(entityId, k0, k1, currentTime, isBall = false) {
    const elapsed = currentTime - k0.time;
    const dt = k1.time - k0.time;
    const t = elapsed / dt; // Normalized time [0, 1]

    // Rocket League physics constants
    const GRAVITY = 650; // units/s² (only for ball, cars have different physics)
    const GROUND_Z = 93; // Ball radius - approximate ground level for ball
    const CAR_GROUND_Z = 17; // Car ground level

    // Maximum correction speed - limits how fast we can "catch up" to the real position
    // This prevents sudden jumps when keyframes are far apart
    const MAX_CORRECTION_SPEED = 500; // units per second

    // If no velocity data, fall back to lerp
    if (!k0.velocity) {
      return {
        x: k0.position.x + (k1.position.x - k0.position.x) * t,
        y: k0.position.y + (k1.position.y - k0.position.y) * t,
        z: k0.position.z + (k1.position.z - k0.position.z) * t,
      };
    }

    // Step 1: Predict position using velocity (with optional gravity for ball)
    let predicted = {
      x: k0.position.x + k0.velocity.x * elapsed,
      y: k0.position.y + k0.velocity.y * elapsed,
      z: k0.position.z + k0.velocity.z * elapsed,
    };

    // Apply gravity for ball if it's in the air
    if (isBall && k0.position.z > GROUND_Z + 10) {
      // z = z0 + vz*t - 0.5*g*t²
      predicted.z = k0.position.z + k0.velocity.z * elapsed - 0.5 * GRAVITY * elapsed * elapsed;
      // Clamp to ground
      if (predicted.z < GROUND_Z) {
        predicted.z = GROUND_Z;
      }
    }

    // Step 2: Calculate where we need to end up (next keyframe position)
    const target = k1.position;

    // Step 3: Calculate the error at current time
    // Where would pure velocity prediction put us at t=1?
    let predictedAtEnd = {
      x: k0.position.x + k0.velocity.x * dt,
      y: k0.position.y + k0.velocity.y * dt,
      z: k0.position.z + k0.velocity.z * dt,
    };

    if (isBall && k0.position.z > GROUND_Z + 10) {
      predictedAtEnd.z = k0.position.z + k0.velocity.z * dt - 0.5 * GRAVITY * dt * dt;
      if (predictedAtEnd.z < GROUND_Z) predictedAtEnd.z = GROUND_Z;
    }

    // Total error to correct
    const totalError = {
      x: target.x - predictedAtEnd.x,
      y: target.y - predictedAtEnd.y,
      z: target.z - predictedAtEnd.z,
    };

    // Step 4: Calculate how much correction we've applied so far
    // Use smooth ease-in-out curve for natural motion
    const easeInOut = t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2;

    // Step 5: Clamp the correction rate
    // Calculate the instantaneous correction velocity
    const errorMagnitude = Math.sqrt(
      totalError.x * totalError.x + totalError.y * totalError.y + totalError.z * totalError.z,
    );

    // If error is large, we need to limit how fast we correct
    let correctionScale = 1.0;
    if (errorMagnitude > 0 && dt > 0) {
      // Maximum correction per frame at 60fps
      const maxCorrectionPerInterval = MAX_CORRECTION_SPEED * dt;
      if (errorMagnitude > maxCorrectionPerInterval * 2) {
        // Large gap - spread the correction more evenly
        correctionScale = (maxCorrectionPerInterval * 2) / errorMagnitude;
      }
    }

    // Apply clamped correction with smooth blending
    const correctionAmount = easeInOut * correctionScale + (1 - correctionScale) * t;

    return {
      x: predicted.x + totalError.x * correctionAmount,
      y: predicted.y + totalError.y * correctionAmount,
      z: predicted.z + totalError.z * correctionAmount,
    };
  }

  /**
   * Physics-tick-aware interpolation
   *
   * The problem: Rocket League replays have positions captured at alternating
   * 5 or 6 physics ticks (120Hz), but timestamps suggest ~46ms intervals.
   * This causes ±10% speed oscillation when interpolating between positions.
   *
   * The solution: Use VELOCITY for movement (which is consistent), not positions.
   * Move at the reported velocity, then smoothly correct to hit the next keyframe.
   *
   * @param {Object} k0 - Start keyframe {position, velocity, time}
   * @param {Object} k1 - End keyframe {position, velocity, time}
   * @param {number} currentTime - Current playback time
   * @returns {Object} Interpolated position {x, y, z}
   */
  _physicsTickInterpolate(k0, k1, currentTime) {
    const rawDt = k1.time - k0.time;
    const elapsed = currentTime - k0.time;
    const t = elapsed / rawDt; // Normalized time [0, 1]

    // If no velocity data, fall back to lerp
    if (!k0.velocity || !k1.velocity) {
      return {
        x: k0.position.x + (k1.position.x - k0.position.x) * t,
        y: k0.position.y + (k1.position.y - k0.position.y) * t,
        z: k0.position.z + (k1.position.z - k0.position.z) * t,
      };
    }

    // CONSTANT VELOCITY APPROACH
    // Use average velocity for the ENTIRE interval - no acceleration within frame
    // This prevents visible speed changes mid-frame
    const avgVelX = (k0.velocity.x + k1.velocity.x) / 2;
    const avgVelY = (k0.velocity.y + k1.velocity.y) / 2;
    const avgVelZ = (k0.velocity.z + k1.velocity.z) / 2;

    // Position based purely on constant velocity
    const velPosX = k0.position.x + avgVelX * elapsed;
    const velPosY = k0.position.y + avgVelY * elapsed;
    const velPosZ = k0.position.z + avgVelZ * elapsed;

    // Calculate error to distribute evenly (LINEAR correction)
    // This distributes the position error uniformly across the interval
    // instead of accumulating it at the end
    const velocityEndX = k0.position.x + avgVelX * rawDt;
    const velocityEndY = k0.position.y + avgVelY * rawDt;
    const velocityEndZ = k0.position.z + avgVelZ * rawDt;

    const errorX = k1.position.x - velocityEndX;
    const errorY = k1.position.y - velocityEndY;
    const errorZ = k1.position.z - velocityEndZ;

    // Linear correction - constant correction rate
    // This means we're moving at: avgVel + error/rawDt (constant!)
    return {
      x: velPosX + errorX * t,
      y: velPosY + errorY * t,
      z: velPosZ + errorZ * t,
    };
  }

  /**
   * Velocity-only interpolation (experimental)
   *
   * Uses ONLY velocity data, completely ignoring position keyframes.
   * This produces the smoothest possible motion but will drift from
   * the actual recorded positions over time.
   *
   * Best for visual quality when exact position accuracy is not critical.
   */
  _velocityOnlyInterpolate(k0, k1, currentTime) {
    const elapsed = currentTime - k0.time;
    const dt = k1.time - k0.time;
    const t = elapsed / dt;

    // If no velocity data, fall back to lerp
    if (!k0.velocity || !k1.velocity) {
      return {
        x: k0.position.x + (k1.position.x - k0.position.x) * t,
        y: k0.position.y + (k1.position.y - k0.position.y) * t,
        z: k0.position.z + (k1.position.z - k0.position.z) * t,
      };
    }

    // For variable velocity, integrate: pos = p0 + integral(v0 + (v1-v0)*s/dt, 0, elapsed)
    // = p0 + v0*elapsed + (v1-v0)*elapsed²/(2*dt)
    const avgVelX = k0.velocity.x + ((k1.velocity.x - k0.velocity.x) * t) / 2;
    const avgVelY = k0.velocity.y + ((k1.velocity.y - k0.velocity.y) * t) / 2;
    const avgVelZ = k0.velocity.z + ((k1.velocity.z - k0.velocity.z) * t) / 2;

    return {
      x: k0.position.x + avgVelX * elapsed,
      y: k0.position.y + avgVelY * elapsed,
      z: k0.position.z + avgVelZ * elapsed,
    };
  }

  /**
   * Smart hybrid interpolation
   *
   * Automatically detects collisions/impacts and switches interpolation method:
   * - COLLISION: Velocity direction or magnitude changes significantly
   *   → Use position lerp (velocity is unreliable mid-interval)
   * - NORMAL: Velocity is consistent
   *   → Use velocity-based interpolation (smoother, more accurate)
   *
   * This addresses the core issue: replay data contains collision frames where
   * the velocity changes during the interval, making velocity-based interpolation
   * produce incorrect results.
   */
  _smartHybridInterpolate(k0, k1, currentTime) {
    const elapsed = currentTime - k0.time;
    const dt = k1.time - k0.time;
    const t = Math.max(0, Math.min(1, elapsed / dt));

    // If no velocity data, fall back to lerp
    if (!k0.velocity || !k1.velocity) {
      return {
        x: k0.position.x + (k1.position.x - k0.position.x) * t,
        y: k0.position.y + (k1.position.y - k0.position.y) * t,
        z: k0.position.z + (k1.position.z - k0.position.z) * t,
      };
    }

    // Calculate velocity magnitudes
    const speed0 = Math.sqrt(k0.velocity.x ** 2 + k0.velocity.y ** 2 + k0.velocity.z ** 2);
    const speed1 = Math.sqrt(k1.velocity.x ** 2 + k1.velocity.y ** 2 + k1.velocity.z ** 2);

    // Detect collision: velocity direction change (dot product)
    let dotProduct = 1;
    if (speed0 > 10 && speed1 > 10) {
      dotProduct =
        (k0.velocity.x * k1.velocity.x +
          k0.velocity.y * k1.velocity.y +
          k0.velocity.z * k1.velocity.z) /
        (speed0 * speed1);
    }

    // Detect collision: significant speed change
    const speedChangePct = speed0 > 10 ? Math.abs(speed1 - speed0) / speed0 : 0;

    // Is this a collision frame?
    // - Direction changed more than ~18 degrees (cos(18°) ≈ 0.95)
    // - OR speed changed by more than 10%
    const isCollision = dotProduct < 0.95 || speedChangePct > 0.1;

    if (isCollision) {
      // COLLISION: Use position lerp with smooth easing
      // Easing helps smooth out the abrupt change
      const smoothT = t * t * (3 - 2 * t); // smoothstep

      return {
        x: k0.position.x + (k1.position.x - k0.position.x) * smoothT,
        y: k0.position.y + (k1.position.y - k0.position.y) * smoothT,
        z: k0.position.z + (k1.position.z - k0.position.z) * smoothT,
      };
    } else {
      // NORMAL: Use velocity-based with linear correction
      // This produces constant speed throughout the interval

      const avgVelX = (k0.velocity.x + k1.velocity.x) / 2;
      const avgVelY = (k0.velocity.y + k1.velocity.y) / 2;
      const avgVelZ = (k0.velocity.z + k1.velocity.z) / 2;

      // Position based on constant velocity
      const velPosX = k0.position.x + avgVelX * elapsed;
      const velPosY = k0.position.y + avgVelY * elapsed;
      const velPosZ = k0.position.z + avgVelZ * elapsed;

      // Calculate position error at end of interval
      const velocityEndX = k0.position.x + avgVelX * dt;
      const velocityEndY = k0.position.y + avgVelY * dt;
      const velocityEndZ = k0.position.z + avgVelZ * dt;

      const errorX = k1.position.x - velocityEndX;
      const errorY = k1.position.y - velocityEndY;
      const errorZ = k1.position.z - velocityEndZ;

      // Distribute error linearly (constant correction rate)
      return {
        x: velPosX + errorX * t,
        y: velPosY + errorY * t,
        z: velPosZ + errorZ * t,
      };
    }
  }

  /**
   * Check if a keyframe transition has a "bad" ratio (time-shifted position)
   * Bad frames have distance ratio ~0.25, 0.50 (position recorded at wrong time)
   *
   * @param {Object} k0 - Start keyframe
   * @param {Object} k1 - End keyframe
   * @returns {boolean} True if this is a bad frame that should be skipped
   */
  _isBadFrame(k0, k1) {
    if (!k0.velocity || !k1.velocity) return false;
    if (!k0.position || !k1.position) return false;

    const dt = k1.time - k0.time;
    if (dt < 0.001) return false;

    const avgVelX = (k0.velocity.x + k1.velocity.x) / 2;
    const avgVelY = (k0.velocity.y + k1.velocity.y) / 2;
    const avgVelZ = (k0.velocity.z + k1.velocity.z) / 2;
    const avgSpeed = Math.sqrt(avgVelX ** 2 + avgVelY ** 2 + avgVelZ ** 2);

    // Don't check slow/stationary objects
    if (avgSpeed < 200) return false;

    const dx = k1.position.x - k0.position.x;
    const dy = k1.position.y - k0.position.y;
    const dz = k1.position.z - k0.position.z;
    const actualDist = Math.sqrt(dx * dx + dy * dy + dz * dz);
    const expectedDist = avgSpeed * dt;
    const ratio = actualDist / expectedDist;

    // Bad if ratio is very far from 1.0
    // Only filter the worst offenders to balance smoothness vs precision
    // 0.6-1.4 keeps ~70% of frames while removing major glitches
    if (ratio < 0.6 || ratio > 1.4) return true;

    return false;
  }

  /**
   * Filter out bad frames from a timeline
   * Bad frames have position recorded at wrong time (ratio ~0.25 or ~0.50)
   *
   * @param {Array} timeline - Array of keyframes
   * @returns {Array} Filtered timeline without bad frames
   */
  _filterBadFrames(timeline) {
    if (!timeline || timeline.length < 2) return timeline;

    const filtered = [timeline[0]]; // Always keep first frame

    for (let i = 1; i < timeline.length; i++) {
      const prevKept = filtered[filtered.length - 1];
      const current = timeline[i];

      // Check if current frame is bad relative to last kept frame
      if (!this._isBadFrame(prevKept, current)) {
        filtered.push(current);
      }
      // If bad, skip it (don't add to filtered)
    }

    return filtered;
  }

  /**
   * Correct time-shifted positions in a timeline
   *
   * Based on Rocket League's 120Hz physics / 30Hz recording:
   * - When ratio ≈ 0.25: position was recorded after 1/4 of the interval
   * - When ratio ≈ 0.50: position was recorded after 1/2 of the interval
   * - When ratio ≈ 0.75: position was recorded after 3/4 of the interval
   *
   * This function extrapolates each time-shifted position to where it
   * SHOULD have been at the actual frame time, using velocity.
   *
   * @param {Array} timeline - Array of keyframes
   * @returns {Array} Timeline with corrected positions
   */
  _correctTimeShiftedPositions(timeline) {
    if (!timeline || timeline.length < 2) {
      const result = timeline ? [...timeline] : [];
      result._correctedCount = 0;
      return result;
    }

    const corrected = [];
    let correctedCount = 0;

    for (let i = 0; i < timeline.length; i++) {
      const frame = timeline[i];

      // Always copy the frame (don't mutate original)
      const newFrame = {
        ...frame,
        position: frame.position ? { ...frame.position } : null,
        velocity: frame.velocity ? { ...frame.velocity } : null,
        rotation: frame.rotation ? { ...frame.rotation } : null,
      };

      // Check if we need to correct this frame
      // We look at the PREVIOUS interval to detect if this frame is shifted
      if (i > 0 && frame.position && frame.velocity) {
        const prev = timeline[i - 1];
        if (prev.position && prev.velocity) {
          const dt = frame.time - prev.time;
          if (dt > 0.001) {
            // Calculate ratio
            const avgVelX = (prev.velocity.x + frame.velocity.x) / 2;
            const avgVelY = (prev.velocity.y + frame.velocity.y) / 2;
            const avgVelZ = (prev.velocity.z + frame.velocity.z) / 2;
            const avgSpeed = Math.sqrt(avgVelX ** 2 + avgVelY ** 2 + avgVelZ ** 2);

            if (avgSpeed > 100) {
              const dx = frame.position.x - prev.position.x;
              const dy = frame.position.y - prev.position.y;
              const dz = frame.position.z - prev.position.z;
              const actualDist = Math.sqrt(dx * dx + dy * dy + dz * dz);
              const expectedDist = avgSpeed * dt;
              const ratio = actualDist / expectedDist;

              // Detect time shift and calculate correction
              let correctionTime = 0;
              if (ratio > 0.15 && ratio < 0.35) {
                // ~25% ratio: position was recorded at 1/4 of the interval
                // Need to extrapolate by 3/4 of dt
                correctionTime = dt * 0.75;
              } else if (ratio > 0.4 && ratio < 0.6) {
                // ~50% ratio: position was recorded at 1/2 of the interval
                correctionTime = dt * 0.5;
              } else if (ratio > 0.65 && ratio < 0.85) {
                // ~75% ratio: position was recorded at 3/4 of the interval
                correctionTime = dt * 0.25;
              }

              if (correctionTime > 0) {
                // Extrapolate position forward using this frame's velocity
                newFrame.position.x += frame.velocity.x * correctionTime;
                newFrame.position.y += frame.velocity.y * correctionTime;
                newFrame.position.z += frame.velocity.z * correctionTime;
                correctedCount++;
              }
            }
          }
        }
      }

      corrected.push(newFrame);
    }

    // Store the count for logging
    corrected._correctedCount = correctedCount;
    return corrected;
  }

  /**
   * Time-Shifted Interpolation
   * Now uses pre-filtered timeline, so this is just lerp
   */
  _timeShiftedInterpolate(k0, k1, currentTime) {
    const elapsed = currentTime - k0.time;
    const dt = k1.time - k0.time;
    const t = Math.max(0, Math.min(1, elapsed / dt));

    return {
      x: k0.position.x + (k1.position.x - k0.position.x) * t,
      y: k0.position.y + (k1.position.y - k0.position.y) * t,
      z: k0.position.z + (k1.position.z - k0.position.z) * t,
    };
  }

  /**
   * Velocity-Anchored Interpolation
   *
   * Uses ONLY velocity for smooth motion, but periodically "anchors" to
   * a known-good position to prevent drift. This gives smooth constant-speed
   * motion while staying accurate over time.
   *
   * The key insight: positions in replay data are unreliable (62% have wrong ratio),
   * but velocities are consistent. We trust velocity for motion, position for anchoring.
   *
   * @param {string} entityId - Unique ID for state tracking
   * @param {Object} k0 - Start keyframe
   * @param {Object} k1 - End keyframe
   * @param {number} currentTime - Current time
   * @param {Array} timeline - Full timeline for anchor lookup
   * @param {number} currentIdx - Current index in timeline
   */
  _velocityAnchoredInterpolate(entityId, k0, k1, currentTime, timeline, currentIdx) {
    if (!k0.velocity || !k1.velocity) {
      // No velocity, fall back to lerp
      const t = (currentTime - k0.time) / (k1.time - k0.time);
      return {
        x: k0.position.x + (k1.position.x - k0.position.x) * t,
        y: k0.position.y + (k1.position.y - k0.position.y) * t,
        z: k0.position.z + (k1.position.z - k0.position.z) * t,
      };
    }

    // Initialize or get state for this entity
    if (!this._velocityAnchorState) {
      this._velocityAnchorState = new Map();
    }

    let state = this._velocityAnchorState.get(entityId);
    const ANCHOR_INTERVAL = 10; // Re-anchor every N keyframes

    // Check if we need to re-anchor (new entity, seek, or interval reached)
    const needsAnchor =
      !state ||
      Math.abs(currentTime - state.lastTime) > 0.5 || // Seek detected
      currentIdx % ANCHOR_INTERVAL === 0; // Periodic re-anchor

    if (needsAnchor) {
      // Anchor to current keyframe position
      state = {
        anchorPos: { ...k0.position },
        anchorTime: k0.time,
        anchorIdx: currentIdx,
        lastTime: currentTime,
      };
      this._velocityAnchorState.set(entityId, state);
    }

    // Calculate position using velocity from anchor point
    const timeSinceAnchor = currentTime - state.anchorTime;

    // Integrate velocity from anchor to current time
    // Use keyframes between anchor and current for accurate velocity
    let posX = state.anchorPos.x;
    let posY = state.anchorPos.y;
    let posZ = state.anchorPos.z;

    // Simple approach: use average of k0 and k1 velocity for this segment
    const avgVelX = (k0.velocity.x + k1.velocity.x) / 2;
    const avgVelY = (k0.velocity.y + k1.velocity.y) / 2;
    const avgVelZ = (k0.velocity.z + k1.velocity.z) / 2;

    // Time within current keyframe interval
    const elapsed = currentTime - k0.time;

    // If anchor is k0, just use velocity
    if (state.anchorIdx === currentIdx) {
      posX = state.anchorPos.x + avgVelX * elapsed;
      posY = state.anchorPos.y + avgVelY * elapsed;
      posZ = state.anchorPos.z + avgVelZ * elapsed;
    } else {
      // Anchor is older, accumulate from anchor through keyframes
      // For simplicity, use k0 position + velocity for current segment
      posX = k0.position.x + avgVelX * elapsed;
      posY = k0.position.y + avgVelY * elapsed;
      posZ = k0.position.z + avgVelZ * elapsed;
    }

    state.lastTime = currentTime;

    return { x: posX, y: posY, z: posZ };
  }

  /**
   * Hermite Spline interpolation
   *
   * Uses cubic Hermite splines which are C1-continuous (smooth in both
   * position and velocity). This creates a curve that:
   * - Passes exactly through keyframe positions
   * - Has tangents matching the reported velocities
   * - Creates natural, smooth motion without visible jitter
   *
   * The Hermite basis functions:
   * h00(t) = 2t³ - 3t² + 1     (start position weight)
   * h10(t) = t³ - 2t² + t      (start tangent weight)
   * h01(t) = -2t³ + 3t²        (end position weight)
   * h11(t) = t³ - t²           (end tangent weight)
   *
   * Position = h00*p0 + h10*m0 + h01*p1 + h11*m1
   * where m0, m1 are the tangents (velocity * dt)
   *
   * @param {Object} k0 - Start keyframe {position, velocity, time}
   * @param {Object} k1 - End keyframe {position, velocity, time}
   * @param {number} currentTime - Current playback time
   * @returns {Object} Interpolated position {x, y, z}
   */
  _hermiteInterpolate(k0, k1, currentTime) {
    const dt = k1.time - k0.time;
    const elapsed = currentTime - k0.time;
    const t = Math.max(0, Math.min(1, elapsed / dt)); // Clamp to [0, 1]

    const linear = {
      x: k0.position.x + (k1.position.x - k0.position.x) * t,
      y: k0.position.y + (k1.position.y - k0.position.y) * t,
      z: k0.position.z + (k1.position.z - k0.position.z) * t,
    };

    // If no velocity data, fall back to lerp
    if (!k0.velocity || !k1.velocity) {
      return linear;
    }

    // Hermite basis functions
    const t2 = t * t;
    const t3 = t2 * t;

    const h00 = 2 * t3 - 3 * t2 + 1; // Weight for p0
    const h10 = t3 - 2 * t2 + t; // Weight for m0 (tangent at p0)
    const h01 = -2 * t3 + 3 * t2; // Weight for p1
    const h11 = t3 - t2; // Weight for m1 (tangent at p1)

    // Tangents are velocity * dt (the expected displacement)
    // This ensures the curve has the correct derivative at endpoints
    const m0 = {
      x: k0.velocity.x * dt,
      y: k0.velocity.y * dt,
      z: k0.velocity.z * dt,
    };
    const m1 = {
      x: k1.velocity.x * dt,
      y: k1.velocity.y * dt,
      z: k1.velocity.z * dt,
    };

    const pos = {
      x: h00 * k0.position.x + h10 * m0.x + h01 * k1.position.x + h11 * m1.x,
      y: h00 * k0.position.y + h10 * m0.y + h01 * k1.position.y + h11 * m1.y,
      z: h00 * k0.position.z + h10 * m0.z + h01 * k1.position.z + h11 * m1.z,
    };

    // Plausibility guard (mirrors @rlrml/player's interpolatePositionHermite):
    // if the velocity-implied curve swings farther from the straight-line path
    // than the segment is long (stale tangents around bounces/demos, units
    // mismatch), fall back to lerp so hermite never looks worse than linear.
    const dx = pos.x - linear.x;
    const dy = pos.y - linear.y;
    const dz = pos.z - linear.z;
    const sx = k1.position.x - k0.position.x;
    const sy = k1.position.y - k0.position.y;
    const sz = k1.position.z - k0.position.z;
    if (dx * dx + dy * dy + dz * dz > sx * sx + sy * sy + sz * sz) {
      return linear;
    }
    return pos;
  }

  /**
   * Physics Simulation Interpolation (RocketSim-based)
   *
   * Based on Rocket League's actual physics from RocketSim:
   * - Physics runs at 120Hz (120 ticks per second)
   * - Replays record at 30Hz (4 physics ticks per frame)
   * - Gravity: -650 UU/s² (Z axis in RL coordinates = Y in Three.js)
   * - Ball max speed: 6000 UU/s
   * - Car max speed: 2300 UU/s
   *
   * The key insight: velocities are accurate, positions may be time-shifted.
   * We use Hermite interpolation which respects both position AND velocity
   * constraints, creating a physically plausible trajectory.
   *
   * @param {Object} k0 - Start keyframe {position, velocity, time}
   * @param {Object} k1 - End keyframe {position, velocity, time}
   * @param {number} currentTime - Current playback time
   * @param {boolean} isBall - Whether this is the ball (applies gravity)
   * @returns {Object} Interpolated position {x, y, z}
   */
  _physicsSimInterpolate(k0, k1, currentTime, isBall = false) {
    const dt = k1.time - k0.time;
    const elapsed = currentTime - k0.time;
    const t = Math.max(0, Math.min(1, elapsed / dt));

    // Fall back to lerp if no velocity data
    if (!k0.velocity || !k1.velocity) {
      return {
        x: k0.position.x + (k1.position.x - k0.position.x) * t,
        y: k0.position.y + (k1.position.y - k0.position.y) * t,
        z: k0.position.z + (k1.position.z - k0.position.z) * t,
      };
    }

    // RocketSim constants
    // Note: In Three.js, Y is up (gravity on Y), in RL it's Z
    // But our data is already converted: position.y = RL's Z
    const GRAVITY = -650; // UU/s² on Y axis (Three.js up)

    // Hermite interpolation parameters
    // h00(t) = 2t³ - 3t² + 1
    // h10(t) = t³ - 2t² + t
    // h01(t) = -2t³ + 3t²
    // h11(t) = t³ - t²
    const t2 = t * t;
    const t3 = t2 * t;
    const h00 = 2 * t3 - 3 * t2 + 1;
    const h10 = t3 - 2 * t2 + t;
    const h01 = -2 * t3 + 3 * t2;
    const h11 = t3 - t2;

    // Tangents are velocity * dt (convert velocity to displacement over interval)
    // For ball, we also account for gravity effect on the tangent
    let m0y = k0.velocity.y * dt;
    let m1y = k1.velocity.y * dt;

    if (isBall) {
      // Add gravity contribution to tangents
      // Gravity affects the trajectory shape between keyframes
      // This makes ball arcs more parabolic
      const gravityEffect = 0.5 * GRAVITY * dt * dt;
      // Distribute gravity effect to both tangents
      m0y += gravityEffect * 0.5;
      m1y += gravityEffect * 0.5;
    }

    // Hermite interpolation for each axis
    return {
      x:
        h00 * k0.position.x +
        h10 * (k0.velocity.x * dt) +
        h01 * k1.position.x +
        h11 * (k1.velocity.x * dt),
      y: h00 * k0.position.y + h10 * m0y + h01 * k1.position.y + h11 * m1y,
      z:
        h00 * k0.position.z +
        h10 * (k0.velocity.z * dt) +
        h01 * k1.position.z +
        h11 * (k1.velocity.z * dt),
    };
  }

  /**
   * Get interpolated position for ball at given time
   * Supports multiple interpolation methods:
   * - 'lerp': Linear interpolation (default)
   * - 'lerp-smooth': Linear + moving average smoothing
   * - 'catmull-rom': Catmull-Rom spline (uses 4 keyframes)
   *
   * @param {number} time - Current time
   * @returns {Object|null} Interpolated position or null
   */
  getBallPositionAt(time) {
    if (!this.ballTimeline || this.ballTimeline.length < 2) return null;

    // If time is before the first keyframe, use the first keyframe's position
    const firstKeyframe = this.ballTimeline[0];
    if (time < firstKeyframe.time && firstKeyframe.position) {
      this.lastFrameInfo = {
        currentFrame: 0,
        totalFrames: this.ballTimeline.length,
      };
      return { ...firstKeyframe.position };
    }

    const idx = this._findKeyframeIndex(this.ballTimeline, time, this.timelineIndices.ball);
    this.timelineIndices.ball = idx;

    const k0 = this.ballTimeline[idx];
    const k1 = this.ballTimeline[idx + 1];

    // Update frame info for debug panel
    this.lastFrameInfo = {
      currentFrame: idx,
      totalFrames: this.ballTimeline.length,
    };

    if (!k0 || !k0.position) return null;

    // If interpolation disabled, return raw keyframe position
    if (!this.interpolationEnabled) {
      return { ...k0.position };
    }

    if (!k1 || !k1.position) return k0.position;

    const dt = k1.time - k0.time;
    if (dt <= 0) return k0.position;

    // Detect teleportation (large position jump)
    const dx = k1.position.x - k0.position.x;
    const dy = k1.position.y - k0.position.y;
    const dz = k1.position.z - k0.position.z;
    const dist = Math.sqrt(dx * dx + dy * dy + dz * dz);

    // If distance > 2000 UU, it's a teleport
    if (dist > 2000) {
      if (k0.sleeping) {
        return null; // Ball is destroyed, hide it
      }
      return { ...k0.position };
    }

    // If ball is sleeping, don't interpolate
    if (k0.sleeping) {
      return { ...k0.position };
    }

    const t = (time - k0.time) / dt;
    let pos;
    // Apply selected interpolation method
    switch (this.interpolationMethod) {
      case "catmull-rom": {
        // Get 4 keyframes for Catmull-Rom
        const km1 = this.ballTimeline[Math.max(0, idx - 1)];
        const k2 = this.ballTimeline[Math.min(this.ballTimeline.length - 1, idx + 2)];
        if (km1?.position && k2?.position) {
          pos = this._catmullRomInterpolate(km1.position, k0.position, k1.position, k2.position, t);
        } else {
          // Fallback to lerp
          pos = {
            x: k0.position.x + (k1.position.x - k0.position.x) * t,
            y: k0.position.y + (k1.position.y - k0.position.y) * t,
            z: k0.position.z + (k1.position.z - k0.position.z) * t,
          };
        }
        break;
      }

      case "lerp-smooth": {
        // Linear interpolation + moving average smoothing
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applySmoothing("ball", pos);
        break;
      }

      case "lerp-ema": {
        // Linear interpolation + Exponential Moving Average
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyEmaSmoothing("ball", pos);
        break;
      }

      case "lerp-dema": {
        // Linear interpolation + Double Exponential Moving Average (Holt's method)
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyDoubleEmaSmoothing("ball", pos);
        break;
      }

      case "lerp-wma": {
        // Linear interpolation + Weighted Moving Average
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyWeightedSmoothing("ball", pos);
        break;
      }

      case "lerp-gauss": {
        // Linear interpolation + Gaussian weighted smoothing
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyGaussianSmoothing("ball", pos);
        break;
      }

      case "one-euro": {
        // One Euro Filter - adaptive smoothing (smooth when slow, responsive when fast)
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyOneEuroFilter("ball", pos);
        break;
      }

      case "predict-correct": {
        // Dead reckoning: predict with velocity, correct towards next keyframe
        // This follows physics more naturally than interpolating between snapshots
        pos = this._predictCorrectInterpolate("ball", k0, k1, time);
        break;
      }

      case "velocity-smooth": {
        // Velocity-based with clamped correction + gravity for ball
        // Best for replays with irregular/missing keyframes
        pos = this._velocitySmoothInterpolate("ball", k0, k1, time, true);
        break;
      }

      case "physics-tick": {
        // Physics-tick-aware interpolation
        // Corrects for 5/6 tick alternation pattern in Rocket League replays
        pos = this._physicsTickInterpolate(k0, k1, time);
        break;
      }

      case "hermite": {
        // Hermite spline interpolation
        // Uses positions as anchors and velocities as tangent hints
        // Creates C1-continuous (smooth) curves through keyframes
        pos = this._hermiteInterpolate(k0, k1, time);
        break;
      }

      case "physics-sim": {
        // Physics simulation with RocketSim constants
        // Uses CORRECTED timeline (time-shifted positions fixed) + Hermite with gravity
        // This gives the smoothest motion by:
        // 1. Pre-correcting 30Hz recording offsets
        // 2. Using Hermite splines with RL gravity

        // Use corrected timeline for physics-sim mode
        const correctedTimeline = this.ballTimelineCorrected;
        if (correctedTimeline && correctedTimeline.length >= 2) {
          const correctedIdx = this._findKeyframeIndex(
            correctedTimeline,
            time,
            this.timelineIndicesCorrected.ball,
          );
          this.timelineIndicesCorrected.ball = correctedIdx;

          const ck0 = correctedTimeline[correctedIdx];
          const ck1 = correctedTimeline[correctedIdx + 1];

          if (ck0?.position && ck1?.position) {
            pos = this._physicsSimInterpolate(ck0, ck1, time, true);
            break;
          }
        }
        // Fallback to raw keyframes if corrected timeline unavailable
        pos = this._physicsSimInterpolate(k0, k1, time, true);
        break;
      }

      case "velocity-only": {
        // Pure velocity-based interpolation (experimental)
        // Ignores position keyframes, uses only velocity for smoothest motion
        // Will drift from actual positions but produces silky smooth movement
        pos = this._velocityOnlyInterpolate(k0, k1, time);
        break;
      }

      case "smart-hybrid": {
        // Smart hybrid: auto-detects collisions and switches method
        // - Collision frames (velocity change): uses position lerp
        // - Normal frames: uses velocity-based interpolation
        pos = this._smartHybridInterpolate(k0, k1, time);
        break;
      }

      case "time-shifted": {
        // Time-shifted: Uses PRE-FILTERED timeline (bad frames already removed)
        // This avoids the acc/decel artifacts from time-shifted positions

        // Use the filtered timeline instead of raw timeline
        const filteredTimeline = this.ballTimelineFiltered;
        if (!filteredTimeline || filteredTimeline.length < 2) {
          // Fallback to lerp if no filtered timeline
          pos = {
            x: k0.position.x + (k1.position.x - k0.position.x) * t,
            y: k0.position.y + (k1.position.y - k0.position.y) * t,
            z: k0.position.z + (k1.position.z - k0.position.z) * t,
          };
          break;
        }

        // Find keyframes in FILTERED timeline
        const filteredIdx = this._findKeyframeIndex(
          filteredTimeline,
          time,
          this.timelineIndicesFiltered.ball,
        );
        this.timelineIndicesFiltered.ball = filteredIdx;

        const fk0 = filteredTimeline[filteredIdx];
        const fk1 = filteredTimeline[filteredIdx + 1];

        if (!fk0?.position || !fk1?.position) {
          pos = fk0?.position ? { ...fk0.position } : { ...k0.position };
          break;
        }

        const fdt = fk1.time - fk0.time;
        const ft = fdt > 0 ? Math.max(0, Math.min(1, (time - fk0.time) / fdt)) : 0;

        pos = {
          x: fk0.position.x + (fk1.position.x - fk0.position.x) * ft,
          y: fk0.position.y + (fk1.position.y - fk0.position.y) * ft,
          z: fk0.position.z + (fk1.position.z - fk0.position.z) * ft,
        };
        break;
      }

      case "position-lerp": {
        // Pure position-based linear interpolation (ignores velocity data completely)
        // This avoids the position/velocity inconsistency issue in replay data
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        break;
      }

      case "position-catmull": {
        // Catmull-Rom spline using ONLY positions (no velocity data)
        // Creates smooth curves through keyframe positions
        const km1 = this.ballTimeline[Math.max(0, idx - 1)];
        const k2 = this.ballTimeline[Math.min(this.ballTimeline.length - 1, idx + 2)];
        if (km1?.position && k2?.position) {
          pos = this._catmullRomInterpolate(km1.position, k0.position, k1.position, k2.position, t);
        } else {
          // Fallback to simple lerp if not enough keyframes
          pos = {
            x: k0.position.x + (k1.position.x - k0.position.x) * t,
            y: k0.position.y + (k1.position.y - k0.position.y) * t,
            z: k0.position.z + (k1.position.z - k0.position.z) * t,
          };
        }
        break;
      }

      case "position-smooth": {
        // Position-based lerp + low-pass filter
        // First interpolate between positions, then apply smoothing filter
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyLowPassFilter("ball", pos);
        break;
      }

      case "adaptive-smooth": {
        // Adaptive smoothing: adjusts smoothing level based on derived velocity
        // - Slow movement: more smoothing (reduces micro-jitter)
        // - Fast movement: less smoothing (maintains responsiveness)
        // - Direction changes: clears buffer to avoid lag
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyAdaptiveSmoothing("ball", pos, time);
        break;
      }

      case "lerp":
      default: {
        // Simple linear interpolation
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        break;
      }
    }

    return pos;
  }

  /**
   * Get interpolated rotation for ball at given time
   * Uses angular velocity for physics-based rotation when available
   * @param {number} time - Current time
   * @returns {Object|null} Interpolated rotation quaternion or null
   */
  getBallRotationAt(time) {
    if (!this.ballTimeline || this.ballTimeline.length < 2) return null;

    // Protection: if time is before first keyframe, return first keyframe rotation
    // This prevents interpolation from identity quaternion during prematch phase
    const firstKeyframe = this.ballTimeline[0];
    if (time < firstKeyframe.time && firstKeyframe.rotation) {
      return { ...firstKeyframe.rotation };
    }

    const idx = this.timelineIndices.ball; // Use cached index from position lookup
    const k0 = this.ballTimeline[idx];
    const k1 = this.ballTimeline[idx + 1];

    if (!k0 || !k0.rotation) return null;

    // If interpolation disabled, return raw keyframe rotation
    if (!this.interpolationEnabled) {
      return { ...k0.rotation };
    }

    if (!k1 || !k1.rotation) return k0.rotation;

    const dt = k1.time - k0.time;
    if (dt <= 0) return k0.rotation;

    // Detect teleportation (large position jump) - keep rotation at k0 too
    if (k0.position && k1.position) {
      const dx = k1.position.x - k0.position.x;
      const dy = k1.position.y - k0.position.y;
      const dz = k1.position.z - k0.position.z;
      const dist = Math.sqrt(dx * dx + dy * dy + dz * dz);

      // If distance > 2000 UU, it's a teleport
      if (dist > 2000) {
        return { ...k0.rotation };
      }
    }

    // If ball is sleeping, don't interpolate rotation either
    if (k0.sleeping) {
      return { ...k0.rotation };
    }

    const t = (time - k0.time) / dt;

    // Use standard SLERP for all modes - it naturally produces constant angular velocity
    // between keyframes, which is the smoothest possible rotation interpolation.
    this._q0.set(k0.rotation.x, k0.rotation.y, k0.rotation.z, k0.rotation.w);
    this._q1.set(k1.rotation.x, k1.rotation.y, k1.rotation.z, k1.rotation.w);
    this._qResult.slerpQuaternions(this._q0, this._q1, t);

    return { x: this._qResult.x, y: this._qResult.y, z: this._qResult.z, w: this._qResult.w };
  }

  /**
   * Get interpolated position for player at given time
   * Supports multiple interpolation methods (same as ball)
   *
   * @param {string} playerName - Player name
   * @param {number} time - Current time
   * @returns {Object|null} Interpolated position or null
   */
  getPlayerPositionAt(playerName, time) {
    const timeline = this.playerTimelineMap[playerName];
    if (!timeline || timeline.length < 2) return null;

    // If time is before the first keyframe, use the first keyframe's position
    const firstKeyframe = timeline[0];
    if (time < firstKeyframe.time && firstKeyframe.position) {
      return { ...firstKeyframe.position };
    }

    const idx = this._findKeyframeIndex(
      timeline,
      time,
      this.timelineIndices.players[playerName] || 0,
    );
    this.timelineIndices.players[playerName] = idx;

    const k0 = timeline[idx];
    const k1 = timeline[idx + 1];

    if (!k0 || !k0.position) return null;

    // If interpolation disabled, return raw keyframe position
    if (!this.interpolationEnabled) {
      return { ...k0.position };
    }

    if (!k1 || !k1.position) return k0.position;

    const dt = k1.time - k0.time;
    if (dt <= 0) return k0.position;

    const t = (time - k0.time) / dt;
    let pos;

    // Apply selected interpolation method
    switch (this.interpolationMethod) {
      case "catmull-rom": {
        // Get 4 keyframes for Catmull-Rom
        const km1 = timeline[Math.max(0, idx - 1)];
        const k2 = timeline[Math.min(timeline.length - 1, idx + 2)];
        if (km1?.position && k2?.position) {
          pos = this._catmullRomInterpolate(km1.position, k0.position, k1.position, k2.position, t);
        } else {
          pos = {
            x: k0.position.x + (k1.position.x - k0.position.x) * t,
            y: k0.position.y + (k1.position.y - k0.position.y) * t,
            z: k0.position.z + (k1.position.z - k0.position.z) * t,
          };
        }
        break;
      }

      case "lerp-smooth": {
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applySmoothing(`player-${playerName}`, pos);
        break;
      }

      case "lerp-ema": {
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyEmaSmoothing(`player-${playerName}`, pos);
        break;
      }

      case "lerp-dema": {
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyDoubleEmaSmoothing(`player-${playerName}`, pos);
        break;
      }

      case "lerp-wma": {
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyWeightedSmoothing(`player-${playerName}`, pos);
        break;
      }

      case "lerp-gauss": {
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyGaussianSmoothing(`player-${playerName}`, pos);
        break;
      }

      case "one-euro": {
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyOneEuroFilter(`player-${playerName}`, pos);
        break;
      }

      case "predict-correct": {
        // Dead reckoning: predict with velocity, correct towards next keyframe
        pos = this._predictCorrectInterpolate(`player-${playerName}`, k0, k1, time);
        break;
      }

      case "velocity-smooth": {
        // Velocity-based with clamped correction (no gravity for cars)
        // Best for replays with irregular/missing keyframes
        pos = this._velocitySmoothInterpolate(`player-${playerName}`, k0, k1, time, false);
        break;
      }

      case "physics-tick": {
        // Physics-tick-aware interpolation
        // Corrects for 5/6 tick alternation pattern in Rocket League replays
        pos = this._physicsTickInterpolate(k0, k1, time);
        break;
      }

      case "hermite": {
        // Hermite spline interpolation
        // Uses positions as anchors and velocities as tangent hints
        // Creates C1-continuous (smooth) curves through keyframes
        pos = this._hermiteInterpolate(k0, k1, time);
        break;
      }

      case "physics-sim": {
        // Physics simulation with RocketSim constants
        // Uses CORRECTED timeline (time-shifted positions fixed) + Hermite
        // Produces smooth curves respecting velocity

        // Use corrected timeline for physics-sim mode
        const correctedTimelinePlayer = this.playerTimelineMapCorrected[playerName];
        if (correctedTimelinePlayer && correctedTimelinePlayer.length >= 2) {
          const correctedIdxPlayer = this._findKeyframeIndex(
            correctedTimelinePlayer,
            time,
            this.timelineIndicesCorrected.players[playerName] || 0,
          );
          this.timelineIndicesCorrected.players[playerName] = correctedIdxPlayer;

          const ck0Player = correctedTimelinePlayer[correctedIdxPlayer];
          const ck1Player = correctedTimelinePlayer[correctedIdxPlayer + 1];

          if (ck0Player?.position && ck1Player?.position) {
            pos = this._physicsSimInterpolate(ck0Player, ck1Player, time, false);
            break;
          }
        }
        // Fallback to raw keyframes if corrected timeline unavailable
        pos = this._physicsSimInterpolate(k0, k1, time, false);
        break;
      }

      case "velocity-only": {
        // Pure velocity-based interpolation (experimental)
        // Ignores position keyframes, uses only velocity for smoothest motion
        pos = this._velocityOnlyInterpolate(k0, k1, time);
        break;
      }

      case "smart-hybrid": {
        // Smart hybrid: auto-detects collisions and switches method
        pos = this._smartHybridInterpolate(k0, k1, time);
        break;
      }

      case "time-shifted": {
        // Time-shifted: Uses PRE-FILTERED timeline (bad frames already removed)
        const filteredTimeline = this.playerTimelineMapFiltered[playerName];
        if (!filteredTimeline || filteredTimeline.length < 2) {
          pos = {
            x: k0.position.x + (k1.position.x - k0.position.x) * t,
            y: k0.position.y + (k1.position.y - k0.position.y) * t,
            z: k0.position.z + (k1.position.z - k0.position.z) * t,
          };
          break;
        }

        const filteredIdx = this._findKeyframeIndex(
          filteredTimeline,
          time,
          this.timelineIndicesFiltered.players[playerName] || 0,
        );
        this.timelineIndicesFiltered.players[playerName] = filteredIdx;

        const fk0 = filteredTimeline[filteredIdx];
        const fk1 = filteredTimeline[filteredIdx + 1];

        if (!fk0?.position || !fk1?.position) {
          pos = fk0?.position ? { ...fk0.position } : { ...k0.position };
          break;
        }

        const fdt = fk1.time - fk0.time;
        const ft = fdt > 0 ? Math.max(0, Math.min(1, (time - fk0.time) / fdt)) : 0;

        pos = {
          x: fk0.position.x + (fk1.position.x - fk0.position.x) * ft,
          y: fk0.position.y + (fk1.position.y - fk0.position.y) * ft,
          z: fk0.position.z + (fk1.position.z - fk0.position.z) * ft,
        };
        break;
      }

      case "position-lerp": {
        // Pure position-based linear interpolation (ignores velocity data completely)
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        break;
      }

      case "position-catmull": {
        // Catmull-Rom spline using ONLY positions (no velocity data)
        const km1 = timeline[Math.max(0, idx - 1)];
        const k2 = timeline[Math.min(timeline.length - 1, idx + 2)];
        if (km1?.position && k2?.position) {
          pos = this._catmullRomInterpolate(km1.position, k0.position, k1.position, k2.position, t);
        } else {
          pos = {
            x: k0.position.x + (k1.position.x - k0.position.x) * t,
            y: k0.position.y + (k1.position.y - k0.position.y) * t,
            z: k0.position.z + (k1.position.z - k0.position.z) * t,
          };
        }
        break;
      }

      case "position-smooth": {
        // Position-based lerp + low-pass filter
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyLowPassFilter(`player-${playerName}`, pos);
        break;
      }

      case "adaptive-smooth": {
        // Adaptive smoothing: adjusts smoothing level based on derived velocity
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        pos = this._applyAdaptiveSmoothing(`player-${playerName}`, pos, time);
        break;
      }

      case "lerp":
      default: {
        pos = {
          x: k0.position.x + (k1.position.x - k0.position.x) * t,
          y: k0.position.y + (k1.position.y - k0.position.y) * t,
          z: k0.position.z + (k1.position.z - k0.position.z) * t,
        };
        break;
      }
    }

    return pos;
  }

  /**
   * Get interpolated rotation for player at given time (slerp)
   * @param {string} playerName - Player name
   * @param {number} time - Current time
   * @returns {Object|null} Interpolated rotation quaternion or null
   */
  getPlayerRotationAt(playerName, time) {
    const timeline = this.playerTimelineMap[playerName];
    if (!timeline || timeline.length < 2) return null;

    // Protection: if time is before first keyframe, return first keyframe rotation
    // This prevents interpolation from identity quaternion during prematch phase
    const firstKeyframe = timeline[0];
    if (time < firstKeyframe.time && firstKeyframe.rotation) {
      return { ...firstKeyframe.rotation };
    }

    const idx = this.timelineIndices.players[playerName] || 0; // Use cached index
    const k0 = timeline[idx];
    const k1 = timeline[idx + 1];

    if (!k0 || !k0.rotation) return null;

    // If interpolation disabled, return raw keyframe rotation
    if (!this.interpolationEnabled) {
      return { ...k0.rotation };
    }

    if (!k1 || !k1.rotation) return k0.rotation;

    const dt = k1.time - k0.time;
    if (dt <= 0) return k0.rotation;

    const t = (time - k0.time) / dt;

    // For all modes: use standard SLERP which gives smooth, constant-speed rotation
    // SLERP naturally produces constant angular velocity between keyframes
    this._q0.set(k0.rotation.x, k0.rotation.y, k0.rotation.z, k0.rotation.w);
    this._q1.set(k1.rotation.x, k1.rotation.y, k1.rotation.z, k1.rotation.w);
    this._qResult.slerpQuaternions(this._q0, this._q1, t);

    return { x: this._qResult.x, y: this._qResult.y, z: this._qResult.z, w: this._qResult.w };
  }

  /**
   * Create the ball mesh
   */
  _createBallMesh() {
    const actorId = "ball"; // Use string ID since we don't have real actor IDs

    const geometry = new THREE.SphereGeometry(92.75, 16, 16);
    const material = new THREE.MeshStandardMaterial({ color: 0xffffff });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.castShadow = true;
    mesh.receiveShadow = true;

    mesh.userData = {
      location: new THREE.Vector3(),
      rotation: new THREE.Quaternion(),
      velocity: new THREE.Vector3(),
      angularVelocity: new THREE.Vector3(),
      isCar: false,
      isBall: true,
      playerId: null,
      sleeping: false,
      isHiddenByGoal: false,
    };

    this.scene.add(mesh);
    this.actors[actorId] = mesh;
    this.ballActorId = actorId;

    // Replace with GLTF model if already loaded
    if (this.ballModel && !this._ballModelReplaced) {
      this.replaceBallWithModel(actorId);
      this._ballModelReplaced = true;
    }

    // Create Ball Indicator
    const radius = 92.75;
    const indicatorGeo = new THREE.RingGeometry(radius * 0.95, radius, 32);
    const indicatorMat = new THREE.MeshBasicMaterial({ color: 0xffffff, side: THREE.DoubleSide });
    this.ballIndicator = new THREE.Mesh(indicatorGeo, indicatorMat);
    this.ballIndicator.rotation.x = -Math.PI / 2;
    this.ballIndicator.visible = false;
    this.scene.add(this.ballIndicator);

    // Create Vertical Line from ball to ground indicator
    const lineGeometry = new THREE.BufferGeometry().setFromPoints([
      new THREE.Vector3(0, 0, 0),
      new THREE.Vector3(0, 1, 0),
    ]);
    const lineMaterial = new THREE.LineBasicMaterial({
      color: 0xffffff,
      opacity: 0.5,
      transparent: true,
    });
    this.ballVerticalLine = new THREE.Line(lineGeometry, lineMaterial);
    this.ballVerticalLine.frustumCulled = false;
    this.ballVerticalLine.visible = false;
    this.scene.add(this.ballVerticalLine);
  }

  /**
   * Create a car mesh for a player
   * @param {string} playerName - Player name
   * @param {number} team - Team (0 = blue, 1 = orange)
   * @param {number} index - Player index (used as actor ID)
   * @param {Object} loadout - Player's TeamLoadout (optional)
   */
  _createCarMesh(playerName, team, index, carName, hitboxType) {
    const actorId = `car_${index}`; // Use string ID

    // Placeholder box (swapped for the GLB model below)
    const geometry = new THREE.BoxGeometry(118, 36, 84);
    const color = team === 0 ? 0x3399ff : 0xff6600;
    const material = new THREE.MeshStandardMaterial({ color });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.castShadow = true;
    mesh.receiveShadow = true;

    // Car/hitbox family resolved by subtr-actor (PlayerInfo.car_body_name /
    // car_hitbox_family). Default to Octane when the replay omits them.
    const carNameResolved = carName || "Octane";
    const hitboxResolved = hitboxType || "Octane";

    mesh.userData = {
      location: new THREE.Vector3(),
      rotation: new THREE.Quaternion(),
      velocity: new THREE.Vector3(),
      angularVelocity: new THREE.Vector3(),
      isCar: true,
      isBall: false,
      playerId: playerName,
      team: team,
      sleeping: false,
      steer: 0,
      carName: carNameResolved,
      hitboxType: hitboxResolved,
    };

    this.scene.add(mesh);
    this.actors[actorId] = mesh;
    this.playerNameToCarActorId[playerName] = actorId;
    this.playerNames.add(playerName);

    // Create boost trail for this car
    this.effectsManager.createBoostTrail(mesh, actorId);

    // Notify UI
    if (this.onPlayerFound) {
      this.onPlayerFound(playerName);
    }

    // Swap the placeholder box for the car's GLB model.
    this.replaceCarWithModel(actorId, mesh, carNameResolved, hitboxResolved);

    console.log(
      `[ActorManager] Created car for ${playerName} (team ${team === 0 ? "blue" : "orange"}, ${carNameResolved} / ${hitboxResolved} hitbox)`,
    );
  }

  /**
   * Update all actors from framework state
   * When useAnimationSystem=true, the AnimationMixer handles position/rotation
   * This method still updates userData and visual effects
   * @param {Player} player - Framework Player instance
   * @param {number} currentTime - Current playback time (for interpolation)
   */
  updateFromFramework(player, currentTime) {
    // Update ball
    const ball = this.actors[this.ballActorId];
    if (ball && player.ball) {
      const ballState = player.ball;

      // When using animation system, mixer.update() handles position/rotation
      // Otherwise use manual interpolation
      let ballPositionValid = true;

      if (!this.useAnimationSystem || !this.animationMixer) {
        if (this.interpolantsInitialized && this.ballTimeline && this.ballTimeline.length >= 2) {
          const pos = this.getBallPositionAt(currentTime);
          const rot = this.getBallRotationAt(currentTime);

          if (pos) {
            ball.position.set(pos.x, pos.y, pos.z);
          } else {
            // Ball is hidden (e.g., after goal explosion)
            ballPositionValid = false;
          }
          if (rot) {
            ball.quaternion.set(rot.x, rot.y, rot.z, rot.w);
          }
        } else {
          // Fallback to framework state (already interpolated)
          ball.position.set(ballState.position.x, ballState.position.y, ballState.position.z);
          ball.quaternion.set(
            ballState.rotation.x,
            ballState.rotation.y,
            ballState.rotation.z,
            ballState.rotation.w,
          );
        }
      }
      // When useAnimationSystem=true, position/rotation are set by mixer.update()

      ball.userData.location.copy(ball.position);
      ball.userData.rotation.copy(ball.quaternion);
      ball.userData.velocity.set(ballState.velocity.x, ballState.velocity.y, ballState.velocity.z);
      if (ballState.angularVelocity) {
        ball.userData.angularVelocity.set(
          ballState.angularVelocity.x,
          ballState.angularVelocity.y,
          ballState.angularVelocity.z,
        );
      }
      ball.userData.sleeping = ballState.sleeping;
      // BallEntity uses 'visible', not 'isVisible'
      // Ball should be visible as long as framework says visible=true
      // Hide ball when position is invalid (after goal explosion)
      ball.visible =
        ballPositionValid && ballState.visible !== false && !ball.userData.isHiddenByGoal;

      // Update ball indicator
      if (this.ballIndicator) {
        this.ballIndicator.position.set(ball.position.x, 2, ball.position.z);
        this.ballIndicator.visible = ball.visible;
      }

      // Update vertical line
      if (this.ballVerticalLine) {
        const groundY = 2;
        const linePositions = new Float32Array([
          ball.position.x,
          groundY,
          ball.position.z,
          ball.position.x,
          ball.position.y,
          ball.position.z,
        ]);
        this.ballVerticalLine.geometry.setAttribute(
          "position",
          new THREE.BufferAttribute(linePositions, 3),
        );
        this.ballVerticalLine.geometry.attributes.position.needsUpdate = true;
        this.ballVerticalLine.visible = ball.visible;
      }

      // Update ball trail
      if (ball.userData.velocity && ball.visible) {
        let closestCarTeam = this.lastBallTouchTeam;
        let minDistance = this.BALL_TOUCH_DISTANCE;

        Object.keys(this.actors).forEach((actorId) => {
          const actor = this.actors[actorId];
          if (actor && actor.userData.isCar && actor.userData.playerId) {
            const distance = ball.position.distanceTo(actor.position);
            if (distance < minDistance) {
              minDistance = distance;
              closestCarTeam = actor.userData.team || 0;
            }
          }
        });

        if (minDistance < this.BALL_TOUCH_DISTANCE) {
          this.lastBallTouchTeam = closestCarTeam;
        }

        this.effectsManager.updateBallTrail(
          ball.position,
          ball.userData.velocity,
          this.lastBallTouchTeam,
        );
      }
    }

    // Update cars
    player.getAllPlayers().forEach((playerEntity) => {
      const carId = this.playerNameToCarActorId[playerEntity.name];
      if (!carId) return;

      const mesh = this.actors[carId];
      if (!mesh) return;

      const playerName = playerEntity.name;

      // When using animation system, mixer.update() handles position/rotation
      // Otherwise use manual interpolation
      if (!this.useAnimationSystem || !this.animationMixer) {
        if (this.interpolantsInitialized && this.playerTimelineMap[playerName]) {
          const pos = this.getPlayerPositionAt(playerName, currentTime);
          const rot = this.getPlayerRotationAt(playerName, currentTime);

          if (pos) {
            mesh.position.set(pos.x, pos.y, pos.z);
          }
          if (rot) {
            mesh.quaternion.set(rot.x, rot.y, rot.z, rot.w);
          }
        } else {
          // Fallback to framework state (already interpolated)
          mesh.position.set(
            playerEntity.position.x,
            playerEntity.position.y,
            playerEntity.position.z,
          );
          mesh.quaternion.set(
            playerEntity.rotation.x,
            playerEntity.rotation.y,
            playerEntity.rotation.z,
            playerEntity.rotation.w,
          );
        }
      }
      // When useAnimationSystem=true, position/rotation are set by mixer.update()

      mesh.userData.location.copy(mesh.position);
      mesh.userData.rotation.copy(mesh.quaternion);
      mesh.userData.velocity.set(
        playerEntity.velocity.x,
        playerEntity.velocity.y,
        playerEntity.velocity.z,
      );
      mesh.userData.sleeping = playerEntity.sleeping;
      mesh.userData.steer = playerEntity.steer || 0;

      // Visibility
      const hasValidPosition = mesh.position.length() > 0.1;
      mesh.visible = playerEntity.isVisible && hasValidPosition && !mesh.userData.sleeping;
    });

    // Fire goal explosions as playback reaches them (and hide the ball inside
    // the blast during the celebration window).
    this._updateGoalExplosions(currentTime);
  }

  /**
   * Trigger the team-colored goal explosion the first time forward playback
   * crosses each goal, and hide the ball for the celebration window so it
   * vanishes inside the blast (matching the original Ballcam player). Robust to
   * scrubbing and post-goal skips: keyed on goal time, self-correcting on
   * backward seeks, and the ball-hidden state is recomputed every frame.
   */
  _updateGoalExplosions(currentTime) {
    const effects = this.effectsManager;
    const goalEvents = effects && effects.explosions ? effects.explosions.goalEvents : null;
    if (!(goalEvents instanceof Map) || goalEvents.size === 0) return;

    const prev = this._lastGoalScanTime;
    // Backward seek / loop: clear fire suppression so goals replay next pass.
    if (prev !== null && currentTime < prev - 0.001) {
      this._firedGoalTimes.clear();
    }

    const ball = this.actors[this.ballActorId];
    let ballHidden = false;

    for (const goal of goalEvents.values()) {
      const goalTime = goal.time;
      if (!Number.isFinite(goalTime)) continue;

      // Hide the ball from the goal moment through the explosion lifetime.
      if (currentTime >= goalTime && currentTime <= goalTime + GOAL_BALL_HIDE_DURATION) {
        ballHidden = true;
      }

      // Fire once, the first time we cross the goal moving forward. A baseline
      // scan (prev === null) only seeds state so a load mid-replay never fires
      // a stale burst.
      const crossedForward = prev !== null && prev < goalTime && currentTime >= goalTime;
      if (crossedForward && !this._firedGoalTimes.has(goalTime)) {
        this._firedGoalTimes.add(goalTime);
        const pos = this.getBallPositionAt(goalTime) || (ball && ball.position) || null;
        if (pos) {
          this.effectsManager.triggerGoalExplosion(pos, goal.team);
        }
      }
    }

    if (ball) {
      ball.userData.isHiddenByGoal = ballHidden;
      if (ballHidden) ball.visible = false;
    }

    this._lastGoalScanTime = currentTime;
  }

  /**
   * Process a network frame for mesh lifecycle management
   * @deprecated Use initFromFramework() and updateFromFramework() instead
   * @param {Object} frame - Network frame
   * @param {Function} getObjectName - Function to get object name by ID (objectId => name)
   * @param {number} frameIndex - Frame index
   * @param {boolean} isSeeking - Whether we're seeking (skip some effects)
   */
  processFrame(frame, getObjectName, frameIndex, isSeeking) {
    if (!frame) return;

    // Handle new actors
    if (frame.new_actors) {
      frame.new_actors.forEach((newActor) => {
        if (!this.actors[newActor.actor_id]) {
          const objectName = getObjectName(newActor.object_id);
          const isBall = objectName && objectName.includes("Ball");
          const isCar = objectName && objectName.includes("Car");

          if (isBall || isCar) {
            let geometry;
            if (isBall) {
              geometry = new THREE.SphereGeometry(92.75, 16, 16);
            } else {
              // Default Octane dimensions (will be updated when we get TeamLoadout)
              geometry = new THREE.BoxGeometry(118, 36, 84);
            }

            const material = new THREE.MeshStandardMaterial({
              color: isBall ? 0xffffff : Math.random() * 0xffffff,
            });
            const mesh = new THREE.Mesh(geometry, material);

            // Initialize userData
            mesh.userData = {
              location: new THREE.Vector3(),
              rotation: new THREE.Quaternion(),
              isCar: isCar,
              isBall: isBall,
              playerId: null,
              lastUpdateTime: frame.time,
              bodyId: null, // Will be set from TeamLoadout
              hasReceivedUpdate: false, // Track if actor has received at least one RigidBody update
            };

            this.scene.add(mesh);
            this.actors[newActor.actor_id] = mesh;

            if (isBall) {
              this.ballActorId = newActor.actor_id;

              // Replace with GLTF model if loaded
              if (this.ballModel) {
                this.replaceBallWithModel(newActor.actor_id);
              }

              // Create Ball Indicator
              // RingGeometry(innerRadius, outerRadius, thetaSegments)
              const radius = 92.75;
              const indicatorGeo = new THREE.RingGeometry(radius * 0.95, radius, 32);
              const indicatorMat = new THREE.MeshBasicMaterial({
                color: 0xffffff,
                side: THREE.DoubleSide,
              });
              this.ballIndicator = new THREE.Mesh(indicatorGeo, indicatorMat);
              this.ballIndicator.rotation.x = -Math.PI / 2; // Flat on ground
              this.ballIndicator.visible = false;
              this.scene.add(this.ballIndicator);

              // Create Vertical Line from ball to ground indicator
              const lineGeometry = new THREE.BufferGeometry().setFromPoints([
                new THREE.Vector3(0, 0, 0),
                new THREE.Vector3(0, 1, 0),
              ]);
              const lineMaterial = new THREE.LineBasicMaterial({
                color: 0xffffff,
                opacity: 0.5,
                transparent: true,
              });
              this.ballVerticalLine = new THREE.Line(lineGeometry, lineMaterial);
              this.ballVerticalLine.frustumCulled = false; // Prevent disappearing at certain camera angles
              this.ballVerticalLine.visible = false;
              this.scene.add(this.ballVerticalLine);
            } else if (isCar) {
              // Create boost trail for this car
              this.effectsManager.createBoostTrail(mesh, newActor.actor_id);
            }
          }
        }
      });
    }

    // Handle deleted actors
    if (frame.deleted_actors) {
      frame.deleted_actors.forEach((actorId) => {
        if (this.actors[actorId]) {
          const mesh = this.actors[actorId];

          // Remove boost trail if it's a car
          if (mesh.userData.isCar) {
            this.effectsManager.removeBoostTrail(actorId);
          }

          this.scene.remove(mesh);
          if (mesh.geometry) mesh.geometry.dispose();
          if (mesh.material) mesh.material.dispose();
          delete this.actors[actorId];

          if (this.ballActorId === actorId) {
            this.ballActorId = null;
            if (this.ballIndicator) {
              this.scene.remove(this.ballIndicator);
              if (this.ballIndicator.geometry) this.ballIndicator.geometry.dispose();
              if (this.ballIndicator.material) this.ballIndicator.material.dispose();
              this.ballIndicator = null;
            }
            if (this.ballVerticalLine) {
              this.scene.remove(this.ballVerticalLine);
              if (this.ballVerticalLine.geometry) this.ballVerticalLine.geometry.dispose();
              if (this.ballVerticalLine.material) this.ballVerticalLine.material.dispose();
              this.ballVerticalLine = null;
            }
          }
        }
      });
    }

    // Handle updates
    if (frame.updated_actors) {
      frame.updated_actors.forEach((update) => {
        const mesh = this.actors[update.actor_id];

        // Extract body ID from TeamLoadout
        if (update.attribute.TeamLoadout) {
          // Store loadout for this actor (usually PRI)
          this.actorLoadouts[update.actor_id] = update.attribute.TeamLoadout;

          // If this actor happens to be a car (rare but possible), apply it directly
          if (mesh && mesh.userData.isCar) {
            mesh.userData.teamLoadout = update.attribute.TeamLoadout;
            this.resolveBodyId(mesh, update.actor_id);
          }
        }

        // Track Player Names (PRI)
        // We need to check both the player name AND that this is actually a PRI actor
        // by checking if the object_id corresponds to a PRI-related object
        const objectName = getObjectName(update.object_id);
        const isPRI =
          objectName &&
          (objectName.includes("PRI_TA") || objectName.includes("PlayerReplicationInfo"));

        if (update.attribute.String && this.playerNames.has(update.attribute.String)) {
          const playerName = update.attribute.String;
          this.actorToPlayer[update.actor_id] = playerName;

          // Only map to PRI actor if this is actually a PRI object AND we haven't mapped this player yet
          if (isPRI && !this.playerNameToPriActorId[playerName]) {
            this.playerNameToPriActorId[playerName] = update.actor_id;
            console.log(
              `[ActorManager] Mapped ${playerName} -> PRI Actor ${update.actor_id} (object: ${objectName})`,
            );
          }

          this.checkCarPlayerLink(update.actor_id);
        }
        if (
          update.attribute.Reservation &&
          this.playerNames.has(update.attribute.Reservation.name)
        ) {
          const playerName = update.attribute.Reservation.name;
          this.actorToPlayer[update.actor_id] = playerName;

          if (isPRI && !this.playerNameToPriActorId[playerName]) {
            this.playerNameToPriActorId[playerName] = update.actor_id;
            console.log(
              `[ActorManager] Mapped ${playerName} -> PRI Actor ${update.actor_id} (object: ${objectName})`,
            );
          }

          this.checkCarPlayerLink(update.actor_id);
        }

        // Track Links (Car -> PRI)
        if (update.attribute.ActiveActor) {
          const targetId = update.attribute.ActiveActor.actor;
          if (!this.actorLinks[update.actor_id]) this.actorLinks[update.actor_id] = new Set();
          this.actorLinks[update.actor_id].add(targetId);

          // Check if this link connects a car to a known player
          if (mesh && mesh.userData.isCar) {
            this.checkCarPlayerLink(targetId, update.actor_id);
          }
        }

        // Physics Updates
        if (mesh && update.attribute && update.attribute.RigidBody) {
          const rb = update.attribute.RigidBody;

          if (rb.location) {
            mesh.userData.location.set(rb.location.x, rb.location.z, rb.location.y);
            mesh.userData.lastUpdateTime = frame.time;
            mesh.userData.hasReceivedUpdate = true; // Mark that actor has received a real position
          }

          if (rb.linear_velocity) {
            if (!mesh.userData.velocity) mesh.userData.velocity = new THREE.Vector3();
            mesh.userData.velocity.set(
              rb.linear_velocity.x,
              rb.linear_velocity.z,
              rb.linear_velocity.y,
            );
          }

          if (rb.rotation) {
            mesh.userData.rotation.set(rb.rotation.x, rb.rotation.z, rb.rotation.y, -rb.rotation.w);
          }

          if (rb.angular_velocity) {
            if (!mesh.userData.angularVelocity) mesh.userData.angularVelocity = new THREE.Vector3();
            mesh.userData.angularVelocity.set(
              rb.angular_velocity.x,
              rb.angular_velocity.z,
              rb.angular_velocity.y,
            );
          }

          // Update Sleeping State
          if (rb.sleeping !== undefined) {
            mesh.userData.sleeping = rb.sleeping;
            if (rb.sleeping) {
              if (mesh.userData.velocity) mesh.userData.velocity.set(0, 0, 0);
              if (mesh.userData.angularVelocity) mesh.userData.angularVelocity.set(0, 0, 0);
            }
          }

          // If it's the ball and we got a RigidBody update, check if it's a reset
          if (mesh.userData.isBall && mesh.userData.isHiddenByGoal) {
            // Only unhide if ball is being reset (e.g., kickoff after goal)
            // Ball is reset when it's near center or has moved significantly
            if (rb.location) {
              const ballX = rb.location.x;
              const ballY = rb.location.y;
              const ballZ = rb.location.z;
              const distFromCenter = Math.sqrt(ballX * ballX + ballY * ballY + ballZ * ballZ);

              // If ball is within ~500 units of center (kickoff position), unhide it
              if (distFromCenter < 500) {
                mesh.userData.isHiddenByGoal = false;
              }
            }
          }
        }

        // Demolition detection moved to GameEngine.js for better name resolution
        // (This avoids duplicate detection and explosions)
        // if (update.attribute && update.attribute.DemolishExtended) {
        //     const demo = update.attribute.DemolishExtended;
        //     if (demo.victim && demo.victim.active) {
        //         const victimActorId = demo.victim.actor;
        //         this.effectsManager.addDemoEvent(frameIndex, victimActorId);
        //         console.log(`🔍 Found DemolishExtended at frame ${frameIndex}: victim=${victimActorId}`);
        //     }
        // }
      });
    }

    // Check for goal events at this frame
    if (this.effectsManager.explosions.goalEvents.has(frameIndex)) {
      const goalEvent = this.effectsManager.explosions.goalEvents.get(frameIndex);
      const ball = this.actors[this.ballActorId];
      if (ball) {
        if (!isSeeking) {
          this.effectsManager.triggerGoalExplosion(ball.position, goalEvent.team);
          console.log(
            `🎯 GOAL! Explosion at frame ${frameIndex} for team ${goalEvent.team} by ${goalEvent.playerName}`,
          );
        }
        ball.userData.isHiddenByGoal = true;
      }
    }

    // Check for demo events at this frame
    if (this.effectsManager.explosions.demoEvents.has(frameIndex)) {
      const demoEvent = this.effectsManager.explosions.demoEvents.get(frameIndex);
      const victimCar = this.actors[demoEvent.victimActorId];
      if (victimCar) {
        if (!isSeeking) {
          const playerName = victimCar.userData.playerId;
          const team = playerName && this.playerTeams ? this.playerTeams[playerName] || 0 : 0;
          this.effectsManager.triggerDemoExplosion(victimCar.position, team);
          console.log(
            `💥 DEMO! Explosion at frame ${frameIndex} for actor ${demoEvent.victimActorId}`,
          );
        }
        victimCar.userData.sleeping = true;
      }
    }
  }

  resolveBodyId(mesh, actorId) {
    if (!mesh || !mesh.userData.isCar || !mesh.userData.teamLoadout) return;

    // Default to blue if we don't know the team yet, or if player not found
    // But ideally we wait for player ID.
    // If we have a playerId, we can check the team.
    let team = 0; // Default to Blue
    if (
      mesh.userData.playerId &&
      Object.prototype.hasOwnProperty.call(this.playerTeams, mesh.userData.playerId)
    ) {
      team = this.playerTeams[mesh.userData.playerId];
    }

    const loadout = mesh.userData.teamLoadout;
    // Select body based on team (0=Blue, 1=Orange)
    const bodyId = team === 1 ? loadout.orange?.body : loadout.blue?.body;

    if (bodyId && mesh.userData.bodyId !== bodyId) {
      mesh.userData.bodyId = bodyId;
      this.updateCarHitbox(mesh, bodyId, actorId);
    }
  }

  updateCarHitbox(mesh, bodyId, actorId) {
    const hitboxInfo = getCarHitboxInfo(bodyId);
    const carName = hitboxInfo?.name || "Octane";
    const hitboxType = hitboxInfo?.hitboxType || "Octane";

    // Try to replace with FBX model
    this.replaceCarWithModel(actorId, mesh, carName, hitboxType);
  }

  /**
   * Replace a car's BoxGeometry with a loaded FBX model
   */
  async replaceCarWithModel(actorId, oldMesh, carName, hitboxType) {
    // Check if model is already loaded
    if (this.carModelLoader.isModelReady(carName, hitboxType)) {
      this._doCarReplacement(actorId, oldMesh, carName, hitboxType);
    } else {
      // Queue for replacement when model loads
      this.pendingCarReplacements.set(actorId, { oldMesh, carName, hitboxType });

      // Wait for model to load then replace
      try {
        const modelType = this.carModelLoader.getModelTypeForCar(carName, hitboxType);
        await this.carModelLoader.loadModel(modelType);
        // Check if this car still exists and needs replacement
        const pending = this.pendingCarReplacements.get(actorId);
        if (pending && this.actors[actorId] === pending.oldMesh) {
          this._doCarReplacement(actorId, pending.oldMesh, pending.carName, pending.hitboxType);
        }
        this.pendingCarReplacements.delete(actorId);
      } catch (error) {
        console.warn(`Failed to load model for ${carName} (${hitboxType}):`, error);
        this.pendingCarReplacements.delete(actorId);
        // Show placeholder as fallback if model fails to load
        if (oldMesh) oldMesh.visible = true;
      }
    }
  }

  _doCarReplacement(actorId, oldMesh, carName, hitboxType) {
    // Determine team from player
    let team = 0;
    if (
      oldMesh.userData.playerId &&
      Object.prototype.hasOwnProperty.call(this.playerTeams, oldMesh.userData.playerId)
    ) {
      team = this.playerTeams[oldMesh.userData.playerId];
    } else if (oldMesh.userData.team !== undefined) {
      // Fallback to userData.team (used in live mode)
      team = oldMesh.userData.team;
    }

    // Get the model synchronously (it's cached now)
    const newMesh = this.carModelLoader.getCarMeshSync(carName, hitboxType, team);

    if (!newMesh) {
      console.warn(`Could not get car mesh for ${carName} (${hitboxType})`);
      return;
    }

    // Merge userData: keep wheels from newMesh, copy rest from oldMesh
    const wheels = newMesh.userData.wheels; // Preserve wheels from new FBX model
    newMesh.userData = { ...oldMesh.userData };
    newMesh.userData.isFBXModel = true;
    newMesh.userData.carName = carName;
    newMesh.userData.hitboxType = hitboxType;
    newMesh.userData.wheels = wheels; // Restore wheels

    // Copy current transform
    newMesh.position.copy(oldMesh.position);
    newMesh.quaternion.copy(oldMesh.quaternion);

    // Remove old mesh from scene
    this.scene.remove(oldMesh);

    // Dispose old geometry and material
    if (oldMesh.geometry) oldMesh.geometry.dispose();
    if (oldMesh.material) {
      if (Array.isArray(oldMesh.material)) {
        oldMesh.material.forEach((m) => m.dispose());
      } else {
        oldMesh.material.dispose();
      }
    }

    // Add new mesh to scene
    this.scene.add(newMesh);

    // Update actor reference
    this.actors[actorId] = newMesh;

    // Update boost trail to use new mesh
    this.effectsManager.removeBoostTrail(actorId);
    this.effectsManager.createBoostTrail(newMesh, actorId);

    const modelType = this.carModelLoader.getModelTypeForCar(carName, hitboxType);
    console.log(
      `🚗 Replaced car ${actorId} with ${modelType.toUpperCase()} model (${carName}, ${hitboxType} hitbox, team ${team === 0 ? "blue" : "orange"})`,
    );
  }

  checkCarPlayerLink(priActorId, carActorId) {
    const playerName = this.actorToPlayer[priActorId];
    const loadout = this.actorLoadouts[priActorId];

    if (!playerName && !loadout) return;

    if (carActorId) {
      const mesh = this.actors[carActorId];
      if (mesh && mesh.userData.isCar) {
        if (this.onPlayerFound) this.onPlayerFound(playerName);
        mesh.userData.playerId = playerName;
        this.playerNameToCarActorId[playerName] = carActorId;

        // Team color will be applied when the FBX model replaces the placeholder
        // (handled in _doCarReplacement via CarModelLoader)

        if (loadout) {
          mesh.userData.teamLoadout = loadout;
        }

        // Now that we know the player (and thus the team), resolve the body ID
        this.resolveBodyId(mesh, carActorId);
      }
    } else {
      // Try to find linked car
      if (this.actorLinks[priActorId]) {
        this.actorLinks[priActorId].forEach((linkedCarId) => {
          this.checkCarPlayerLink(priActorId, linkedCarId);
        });
      }
    }
  }

  updateInterpolation(time, frames, targetFrameIndex) {
    const currentFrame = frames[targetFrameIndex];
    if (currentFrame) {
      Object.keys(this.actors).forEach((actorId) => {
        const mesh = this.actors[actorId];
        const currentLoc = mesh.userData.location;
        const currentRot = mesh.userData.rotation;

        if (!currentLoc || !currentRot) return;

        // Skip interpolation if actor hasn't received any RigidBody update yet
        // This prevents interpolation from (0,0,0) to first real position
        if (!mesh.userData.hasReceivedUpdate) return;

        // Find next update for this actor
        let nextUpdate = null;
        let nextFrameTime = 0;

        // Look ahead up to 60 frames (approx 2 seconds) to find the next update
        for (
          let i = targetFrameIndex + 1;
          i < Math.min(frames.length, targetFrameIndex + 60);
          i++
        ) {
          const frame = frames[i];
          if (frame.updated_actors) {
            const update = frame.updated_actors.find(
              (u) => u.actor_id == actorId && u.attribute && u.attribute.RigidBody,
            );
            if (update) {
              nextUpdate = update;
              nextFrameTime = frame.time;
              break;
            }
          }
        }

        if (nextUpdate) {
          const t1 = mesh.userData.lastUpdateTime || currentFrame.time;
          const t2 = nextFrameTime;

          // Avoid division by zero
          if (t2 > t1) {
            const alpha = (time - t1) / (t2 - t1);
            const clampedAlpha = Math.max(0, Math.min(1, alpha));
            const dt = t2 - t1 || 0.033;
            const rb = nextUpdate.attribute.RigidBody;

            // Interpolate Location
            if (rb.location) {
              this._p0.copy(currentLoc);
              this._p1.set(rb.location.x, rb.location.z, rb.location.y);

              if (mesh.userData.sleeping) {
                mesh.position.copy(currentLoc);
              } else {
                // Hermite Spline Interpolation for Position
                const nextRb = nextUpdate.attribute.RigidBody;
                if (mesh.userData.velocity && nextRb.linear_velocity) {
                  const t = clampedAlpha;
                  const alpha2 = t * t;
                  const alpha3 = alpha2 * t;

                  if (dt > 0.5) {
                    mesh.position.lerpVectors(currentLoc, this._p1, clampedAlpha);
                  } else {
                    this._v0.copy(mesh.userData.velocity).multiplyScalar(dt);
                    this._v1
                      .set(
                        nextRb.linear_velocity.x,
                        nextRb.linear_velocity.z,
                        nextRb.linear_velocity.y,
                      )
                      .multiplyScalar(dt);

                    const h00 = 2 * alpha3 - 3 * alpha2 + 1;
                    const h10 = alpha3 - 2 * alpha2 + t;
                    const h01 = -2 * alpha3 + 3 * alpha2;
                    const h11 = alpha3 - alpha2;

                    mesh.position.set(
                      h00 * this._p0.x + h10 * this._v0.x + h01 * this._p1.x + h11 * this._v1.x,
                      h00 * this._p0.y + h10 * this._v0.y + h01 * this._p1.y + h11 * this._v1.y,
                      h00 * this._p0.z + h10 * this._v0.z + h01 * this._p1.z + h11 * this._v1.z,
                    );
                  }
                } else {
                  mesh.position.lerpVectors(currentLoc, this._p1, clampedAlpha);
                }
              }
            } else {
              mesh.position.copy(currentLoc);
            }

            // Interpolate Rotation
            if (rb.rotation) {
              this._nextRot.set(rb.rotation.x, rb.rotation.z, rb.rotation.y, -rb.rotation.w);
              mesh.quaternion.slerpQuaternions(currentRot, this._nextRot, clampedAlpha);
            } else {
              mesh.quaternion.copy(currentRot);
            }
            return; // Done for this actor
          }
        }

        // Fallback if no future update found
        mesh.position.copy(currentLoc);
        mesh.quaternion.copy(currentRot);
      });
    }

    // Visibility Logic
    Object.keys(this.actors).forEach((actorId) => {
      const mesh = this.actors[actorId];
      if (mesh && mesh.userData.isCar) {
        const pos = mesh.position;
        const hasValidPosition = pos.length() > 0.1;
        const isSleeping = mesh.userData.sleeping === true;
        mesh.visible = hasValidPosition && !isSleeping;
      }
    });

    if (this.ballActorId && this.actors[this.ballActorId]) {
      const ball = this.actors[this.ballActorId];
      ball.visible = !ball.userData.isHiddenByGoal;

      if (this.ballIndicator) {
        // Keep it slightly above ground (2 UU) to avoid z-fighting
        this.ballIndicator.position.set(ball.position.x, 2, ball.position.z);
        this.ballIndicator.visible = ball.visible;
      }

      if (this.ballVerticalLine) {
        // Update the vertical line from ground to ball
        const groundY = 2;
        const linePositions = new Float32Array([
          ball.position.x,
          groundY,
          ball.position.z,
          ball.position.x,
          ball.position.y,
          ball.position.z,
        ]);
        this.ballVerticalLine.geometry.setAttribute(
          "position",
          new THREE.BufferAttribute(linePositions, 3),
        );
        this.ballVerticalLine.geometry.attributes.position.needsUpdate = true;
        this.ballVerticalLine.visible = ball.visible;
      }

      // Update ball trail
      if (ball.userData.velocity && ball.visible) {
        // Check for nearby cars to determine team color
        let closestCarTeam = this.lastBallTouchTeam;
        let minDistance = this.BALL_TOUCH_DISTANCE;

        Object.keys(this.actors).forEach((actorId) => {
          const actor = this.actors[actorId];
          if (actor && actor.userData.isCar && actor.userData.playerId) {
            const distance = ball.position.distanceTo(actor.position);
            if (distance < minDistance) {
              minDistance = distance;
              const playerName = actor.userData.playerId;
              if (Object.prototype.hasOwnProperty.call(this.playerTeams, playerName)) {
                closestCarTeam = this.playerTeams[playerName];
              }
            }
          }
        });

        // Update last touch team if a car is close
        if (minDistance < this.BALL_TOUCH_DISTANCE) {
          this.lastBallTouchTeam = closestCarTeam;
        }

        // Update ball trail with position, velocity, and team
        this.effectsManager.updateBallTrail(
          ball.position,
          ball.userData.velocity,
          this.lastBallTouchTeam,
        );
      }
    }
  }
  /**
   * Update boost state for a player
   * @param {string} playerName - Player name
   * @param {boolean} isBoosting - Whether player is actively boosting
   * @param {boolean} isKickoffReset - Whether boost was reset during kickoff (skip particles)
   */
  updateBoostState(playerName, isBoosting, isKickoffReset = false) {
    const carId = this.playerNameToCarActorId[playerName];
    if (!carId) {
      if (isBoosting && !this._warnedNoCarId) {
        console.warn(`⚠️ updateBoostState: No carId for ${playerName}`);
        this._warnedNoCarId = true;
      }
      return;
    }

    const mesh = this.actors[carId];
    if (!mesh || !mesh.userData.isCar) {
      if (isBoosting) {
        console.warn(`⚠️ updateBoostState: No mesh for car ${carId}, player ${playerName}`);
      }
      return;
    }

    // Get velocity for particle emission
    const velocity = mesh.userData.velocity || new THREE.Vector3(0, 0, 0);

    // Skip boost particle emission during kickoff reset
    // (when boost is reset to 33% during kickoff, don't show particles)
    const shouldShowParticles = isBoosting && !isKickoffReset;

    // Update boost trail particles
    this.effectsManager.updateBoostTrail(
      carId,
      shouldShowParticles,
      mesh.position,
      mesh.quaternion,
      velocity,
    );
  }

  /**
   * Update player steering value from framework
   * @param {string} playerName - Player name
   * @param {number} steer - Normalized steering value (-1 to 1)
   */
  updatePlayerSteer(playerName, steer) {
    const carId = this.playerNameToCarActorId[playerName];
    if (!carId) return;

    const mesh = this.actors[carId];
    if (!mesh || !mesh.userData.isCar) return;

    mesh.userData.steer = steer;
  }

  /**
   * Update wheel rotations for all cars based on actual distance traveled
   * This method uses position delta instead of velocity * time to ensure
   * wheel rotation matches visual movement at any playback speed
   * - Wheel_XX_Y: rotates around local Y for rolling (spin)
   * - Wheel_XX_Z: rotates around local Z for steering (front wheels only)
   */
  updateWheelRotations() {
    // Debug
    if (!this._wheelDebugCounter) this._wheelDebugCounter = 0;
    this._wheelDebugCounter++;

    // Wheel radius in Unreal Units (approximately 17 UU)
    const WHEEL_RADIUS = 17;

    // Max steering angle in radians (~30 degrees)
    const MAX_STEER_ANGLE = Math.PI / 6;

    // Initialize storage if needed
    if (!this._previousCarPositions) {
      this._previousCarPositions = new Map();
    }

    Object.keys(this.actors).forEach((actorId) => {
      const mesh = this.actors[actorId];

      // Only process car models with wheels (FBX or GLB with wheel sockets)
      if (!mesh || !mesh.userData.isCar) return;
      if (!mesh.userData.isFBXModel && !mesh.userData.hasWheelSockets) return;
      if (!mesh.userData.wheels || mesh.userData.wheels.length === 0) return;

      // Get current position
      const currentPos = mesh.position;

      // Get previous position (or use current if first frame)
      let prevPos = this._previousCarPositions.get(actorId);
      if (!prevPos) {
        prevPos = currentPos.clone();
        this._previousCarPositions.set(actorId, prevPos);
      }

      // Calculate position delta (distance traveled this frame)
      const deltaPos = new THREE.Vector3().subVectors(currentPos, prevPos);
      const distanceTraveled = deltaPos.length();

      // Store current position for next frame
      this._previousCarPositions.set(actorId, currentPos.clone());

      // Skip if car hasn't moved
      if (distanceTraveled < 0.01) return;

      // Simple approach: wheels always rotate forward
      // In Rocket League, reversing is rare and brief, so this is visually acceptable
      // This eliminates all the direction calculation bugs that occur during turns/drifts
      const direction = 1;

      // Calculate wheel rotation based on distance traveled
      // rotation = distance / wheel_circumference * 2π = distance / radius
      // Cap the per-frame step: at game speeds the true rate (1-2.3 rad/frame
      // at 60fps) is past the Nyquist limit of a spoked rim, so unclamped
      // rotation strobes (wagon-wheel effect) and reads as "wheels don't
      // spin". 0.5 rad/frame stays below one spoke period per frame, so the
      // spin is fast but visually coherent.
      const MAX_WHEEL_STEP = 0.5;
      const rotationIncrement =
        Math.min(distanceTraveled / WHEEL_RADIUS, MAX_WHEEL_STEP) * direction;

      // Use ReplicatedSteer value from replay if available
      let steerAngle = 0;
      if (mesh.userData.steer !== undefined) {
        // Use actual steering value from replay (normalized -1 to 1)
        // Negate because RL coordinate system is opposite to Three.js for steering
        steerAngle = -mesh.userData.steer * MAX_STEER_ANGLE;
      }

      // Apply rotations to each wheel
      mesh.userData.wheels.forEach((wheelData) => {
        if (wheelData.socket) {
          // New GLB system with wheel sockets
          // Inverted direction: left=1, right=-1 (wheels rotate forward when car moves forward)
          const sideMultiplier = wheelData.side === "left" ? 1 : -1;
          wheelData.mesh.rotateZ(sideMultiplier * rotationIncrement);

          // Steering (front wheels only) - rotate socket around Y axis
          // Left wheel: negative steerAngle, Right wheel: positive steerAngle
          if (wheelData.position === "front" && wheelData.steeringPivot) {
            const steerMultiplier = wheelData.side === "left" ? -1 : 1;
            wheelData.steeringPivot.rotation.y = steerMultiplier * steerAngle;
          }
        } else {
          // Old FBX system: rotate the wheel mesh around Y
          const sideMultiplier = wheelData.side === "left" ? -1 : 1;
          wheelData.mesh.rotateY(sideMultiplier * rotationIncrement);

          // Steering (front wheels only)
          if (wheelData.position === "front" && wheelData.steeringPivot) {
            wheelData.steeringPivot.rotation.z = steerAngle;
          }
        }
      });
    });
  }

  /**
   * Reset wheel rotation tracking (call when seeking)
   */
  resetWheelTracking() {
    if (this._previousCarPositions) {
      this._previousCarPositions.clear();
    }
  }

  updateSupersonicState(playerName, isSupersonic, team) {
    const carId = this.playerNameToCarActorId[playerName];
    if (!carId) {
      return;
    }

    const mesh = this.actors[carId];
    if (!mesh || !mesh.userData.isCar) {
      return;
    }

    // Get velocity for trail emission
    const velocity = mesh.userData.velocity || new THREE.Vector3(0, 0, 0);

    // Update supersonic trail
    this.effectsManager.updateSupersonicTrail(
      carId,
      isSupersonic,
      mesh.position,
      mesh.quaternion,
      velocity,
      team,
    );
  }

  /**
   * Enable or disable interpolation (for debugging)
   * When disabled, shows raw frame data without interpolation
   */
  setInterpolationEnabled(enabled) {
    this.interpolationEnabled = enabled;
    console.log(`[ActorManager] Interpolation ${enabled ? "enabled" : "disabled"}`);
  }

  /**
   * Set interpolation method
   * @param {string} method - Interpolation method name
   */
  setInterpolationMethod(method) {
    const validMethods = [
      "lerp",
      "hermite",
      "catmull-rom",
      "predict-correct",
      "velocity-smooth",
      "physics-tick",
      "velocity-only",
      "smart-hybrid",
      "time-shifted",
      "lerp-smooth",
      "lerp-ema",
      "lerp-dema",
      "lerp-wma",
      "lerp-gauss",
      "one-euro",
      "position-lerp",
      "position-catmull",
      "position-smooth",
      "adaptive-smooth",
    ];
    if (!validMethods.includes(method)) {
      console.warn(`[ActorManager] Invalid interpolation method: ${method}`);
      return;
    }
    this.interpolationMethod = method;
    // Clear ALL smoothing buffers when changing method to avoid artifacts
    this._smoothingBuffers.clear();
    this._lowPassState.clear();
    this._adaptiveState.clear();
    this.resetSmoothingBuffers();
    console.log(`[ActorManager] Interpolation method set to: ${method}`);
  }

  /**
   * Set smoothing window size (for lerp-smooth method)
   * @param {number} size - Window size (1-20)
   */
  setSmoothingWindowSize(size) {
    this.smoothingWindowSize = Math.max(1, Math.min(20, size));
    // Clear buffers when changing size
    this._smoothingBuffers.clear();
    console.log(`[ActorManager] Smoothing window size set to: ${this.smoothingWindowSize}`);
  }

  /**
   * Get current interpolation settings
   */
  getInterpolationSettings() {
    return {
      enabled: this.interpolationEnabled,
      method: this.interpolationMethod,
      smoothingWindowSize: this.smoothingWindowSize,
    };
  }

  /**
   * Clear all smoothing buffers (call when seeking or changing settings)
   */
  clearSmoothingBuffers() {
    this._smoothingBuffers.clear();
  }

  /**
   * Get current frame info for debug panel
   */
  getFrameInfo() {
    return this.lastFrameInfo;
  }

  // ============================================
  // LIVE MODE METHODS (027-live-player)
  // ============================================

  /**
   * Create ball mesh for live mode
   * Returns the ball mesh directly instead of storing it
   * @returns {THREE.Mesh} The ball mesh
   */
  createBallMeshForLive() {
    const geometry = new THREE.SphereGeometry(92.75, 16, 16);
    const material = new THREE.MeshStandardMaterial({ color: 0xffffff });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.castShadow = true;
    mesh.receiveShadow = true;

    mesh.userData = {
      location: new THREE.Vector3(),
      rotation: new THREE.Quaternion(),
      velocity: new THREE.Vector3(),
      angularVelocity: new THREE.Vector3(),
      isCar: false,
      isBall: true,
      playerId: null,
      sleeping: false,
      isHiddenByGoal: false,
    };

    this.scene.add(mesh);

    // Replace with GLTF model if loaded
    if (this.ballModel) {
      const ballModel = this.ballModel.clone();
      ballModel.position.copy(mesh.position);
      ballModel.quaternion.copy(mesh.quaternion);
      ballModel.userData = { ...mesh.userData };

      // Scale to match ball size (92.75 units radius - official Rocket League size)
      // Same as replaceBallWithModel
      const ballScale = 92.75;
      ballModel.scale.set(ballScale, ballScale, ballScale);

      // Enable shadow casting and receiving on all meshes
      ballModel.traverse((child) => {
        if (child.isMesh) {
          child.castShadow = true;
          child.receiveShadow = true;
        }
      });

      this.scene.remove(mesh);
      this.scene.add(ballModel);

      // Dispose temp mesh
      if (mesh.geometry) mesh.geometry.dispose();
      if (mesh.material) mesh.material.dispose();

      console.log("✓ Live ball created with GLTF model");
      return ballModel;
    }

    return mesh;
  }

  /**
   * Create car mesh for live mode
   * Returns the car mesh directly
   * @param {number} team - Team (0 = blue, 1 = orange)
   * @param {number} playerIndex - Player index for unique identification
   * @param {string} playerName - Player name
   * @param {number|null} bodyId - Car body ID (e.g., 23=Octane, 403=Fennec)
   * @returns {THREE.Mesh} The car mesh
   */
  createCarMeshForLive(team, playerIndex, playerName, bodyId = null) {
    const actorId = `live_car_${playerIndex}`;

    // Default Octane dimensions (placeholder until FBX loads)
    const geometry = new THREE.BoxGeometry(118, 36, 84);
    const color = team === 0 ? 0x3399ff : 0xff6600;
    const material = new THREE.MeshStandardMaterial({ color });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    mesh.visible = false; // Hide placeholder until FBX model is loaded

    mesh.userData = {
      location: new THREE.Vector3(),
      rotation: new THREE.Quaternion(),
      velocity: new THREE.Vector3(),
      angularVelocity: new THREE.Vector3(),
      isCar: true,
      isBall: false,
      playerId: playerName,
      team: team,
      sleeping: false,
      steer: 0,
      bodyId: bodyId,
      liveActorId: actorId,
    };

    this.scene.add(mesh);

    // Store reference for effects
    this.actors[actorId] = mesh;
    this.playerNameToCarActorId[playerName] = actorId;

    // Create boost trail for this car
    this.effectsManager.createBoostTrail(mesh, actorId);

    // If we have a body ID, try to load the appropriate car model
    // Otherwise, use Octane as default for live mode
    if (bodyId && bodyId > 0) {
      this.updateCarHitbox(mesh, bodyId, actorId);
    } else {
      // No body ID info - use Octane as default
      this.replaceCarWithModel(actorId, mesh, "Octane", "Octane");
    }

    console.log(
      `[ActorManager] Created live car for ${playerName} (team ${team === 0 ? "blue" : "orange"}, bodyId: ${bodyId})`,
    );
    return mesh;
  }

  /**
   * Update boost particles for live mode car
   * IMPORTANT: Checks BOTH isBoosting AND boost > 0
   * (isBoosting is input, need amount > 0 to emit particles)
   * @param {string} actorId - Car actor ID
   * @param {boolean} isBoosting - Input state
   * @param {number} boostAmount - Current boost amount (0-100)
   * @param {THREE.Mesh} mesh - Car mesh
   */
  updateBoostParticlesLive(actorId, isBoosting, boostAmount, mesh) {
    const shouldShowParticles = isBoosting && boostAmount > 0;
    const velocity = mesh.userData.velocity || new THREE.Vector3(0, 0, 0);

    this.effectsManager.updateBoostTrail(
      actorId,
      shouldShowParticles,
      mesh.position,
      mesh.quaternion,
      velocity,
    );
  }

  /**
   * Update supersonic trail for live mode car
   * @param {string} actorId - Car actor ID
   * @param {boolean} isSupersonic - Whether car is supersonic (speed > 2200)
   * @param {number} team - Team number (0 or 1)
   * @param {THREE.Mesh} mesh - Car mesh
   */
  updateSupersonicTrailLive(actorId, isSupersonic, team, mesh) {
    const velocity = mesh.userData.velocity || new THREE.Vector3(0, 0, 0);

    this.effectsManager.updateSupersonicTrail(
      actorId,
      isSupersonic,
      mesh.position,
      mesh.quaternion,
      velocity,
      team,
    );
  }

  /**
   * Remove a live mode car
   * @param {string} actorId - Car actor ID
   */
  removeLiveCar(actorId) {
    const mesh = this.actors[actorId];
    if (mesh) {
      this.scene.remove(mesh);
      if (mesh.geometry) mesh.geometry.dispose();
      if (mesh.material) mesh.material.dispose();
      delete this.actors[actorId];

      // Clean up boost trail
      this.effectsManager.removeBoostTrail(actorId);
    }
  }

  /**
   * Remove live mode ball
   * @param {THREE.Mesh} ballMesh - Ball mesh to remove
   */
  removeLiveBall(ballMesh) {
    if (ballMesh) {
      this.scene.remove(ballMesh);
      if (ballMesh.geometry) ballMesh.geometry.dispose();
      if (ballMesh.material) ballMesh.material.dispose();
    }
  }
}
