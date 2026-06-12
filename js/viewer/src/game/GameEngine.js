import * as THREE from 'three';
import { SceneManager } from '../managers/SceneManager.js';
import { ReplayLoader } from '../managers/ReplayLoader.js';
import { ArenaManager } from '../managers/ArenaManager.js';
import { ActorManager } from '../managers/ActorManager.js';
import { EffectsManager, precompileExplosionMaterials, warmupExplosionPool, resetExplosionPools } from '../managers/EffectsManager.js';
import { InputManager } from '../managers/InputManager.js';
import { CameraManager } from '../managers/CameraManager.js';
import { NameTagManager } from '../managers/NameTagManager.js';
import { HitboxManager } from '../managers/HitboxManager.js';
import { SpeedLabelManager } from '../managers/SpeedLabelManager.js';
import { DevToolsManager } from '../managers/DevToolsManager.js';
import { ViewerCameraManager } from '../managers/ViewerCameraManager.js';
import { EnvironmentManager } from '../managers/EnvironmentManager.ts';
import { PingManager } from '../managers/PingManager.js';
import { DrawingManager } from '../managers/DrawingManager.js';
import { OffscreenIndicatorManager } from '../managers/OffscreenIndicatorManager.js';
import { ClipRecordingManager } from '../managers/ClipRecordingManager.js';
import { ClipPlaybackManager } from '../managers/ClipPlaybackManager.js';
import { KeyframeVisualizer } from '../managers/KeyframeVisualizer.js';

export class GameEngine {
  constructor(container, callbacks) {
    this.callbacks = callbacks; // { onTimeUpdate, onPlayerListUpdate, onPlayStateChange, onReady, onLoadingProgress }

    // Initialize Scene Manager with the container element directly
    // We need to modify SceneManager to accept an element, or we handle appending here.
    // Let's modify SceneManager to accept an element or ID.
    // For now, let's assume we pass the ID or element.
    // Since React refs give us the element, we should update SceneManager to accept an element.

    // HACK: SceneManager expects an ID string currently.
    // We will refactor SceneManager to accept an element in a moment.
    // For now, let's pass the container element to SceneManager if we update it.

    this.sceneManager = new SceneManager(container);
    this.effectsManager = new EffectsManager(this.sceneManager.scene);

    // Note: explosion shaders are pre-compiled in init() which is async
    this.actorManager = new ActorManager(
      this.sceneManager.scene,
      this.effectsManager
    );
    this.arenaManager = new ArenaManager(this.sceneManager.scene);
    this.nameTagManager = new NameTagManager(this.sceneManager.scene, this.sceneManager.camera);
    this.hitboxManager = new HitboxManager(this.sceneManager.scene);
    this.speedLabelManager = new SpeedLabelManager(this.sceneManager.scene, this.sceneManager.camera);
    this.inputManager = new InputManager(
      this.sceneManager.camera,
      this.sceneManager.renderer.domElement
    );
    this.replayLoader = new ReplayLoader();

    // New camera manager with smooth transitions
    this.cameraManager = new CameraManager(
      this.sceneManager.camera,
      this.sceneManager.renderer.domElement
    );

    this.player = null; // Framework Player API
    this.maxTime = 0;
    this.gameTimeMap = []; // Game time data (from framework)
    this.countdownEvents = []; // Kickoff countdowns (from framework)
    this.currentTime = 0;
    this.isPlaying = false;

    this.cameraMode = 'free';
    this.selectedPlayerName = null;

    // Clip recording/playback state (024-clip-system)
    this.clipMode = null; // null | 'recording' | 'playback'
    this.clipRecordingManager = new ClipRecordingManager();
    this.clipPlaybackManager = new ClipPlaybackManager(this.sceneManager.camera);
    this.keyframeVisualizer = new KeyframeVisualizer(
      this.sceneManager.scene,
      this.sceneManager.camera,
      this.sceneManager.renderer
    );
    this.keyframeVisualizer.hide(); // Hidden by default

    this.cameraSettings = {
      distance: 260,        // Distance behind car (UU) - RL range: 100-400
      height: 90,           // Height above car (UU) - RL range: 40-200
      angle: -4,            // Pitch angle in degrees (negative = look down) - RL range: -15 to 0
      stiffness: 0.45,      // Camera stiffness (0.0-1.0) - higher = more responsive
      swivelSpeed: 4.30,    // Rotation speed around car (1.0-10.0)
      transitionSpeed: 1.30, // Ball cam transition speed (1.0-2.0)
      fov: 110,             // Field of view (60-110)
    };
    this.playbackSpeed = 1.0;

    this.clock = new THREE.Clock();

    // Bind methods
    this.animate = this.animate.bind(this);

    // Hook into ActorManager to update UI
    this.actorManager.onPlayerFound = (name) => {
      if (this.callbacks.onPlayerListUpdate) {
        this.callbacks.onPlayerListUpdate(name);
      }
    };

    // Initialize camera mode
    this.setCameraMode('free');

    // Set default freecam position (side view of field)
    this.cameraManager.setDefaultFreecamPosition();

    // Setup pointer lock callback for UI components
    if (callbacks.onPointerLockChange) {
      this.cameraManager.setPointerLockCallback(callbacks.onPointerLockChange);
    }

    // Store binary data if provided (for API loading)
    this.binaryData = callbacks.binaryData || null;

    // Initialize DevTools Manager for scene inspection
    this.devToolsManager = new DevToolsManager(
      this.sceneManager.scene,
      this.sceneManager.camera,
      this.sceneManager.renderer,
      this.cameraManager // Pass CameraManager to disable its controls in dev mode
    );

    // Initialize ViewerCameraManager for collaborative viewing
    this.viewerCameraManager = new ViewerCameraManager(
      this.sceneManager.scene,
      this.sceneManager.camera
    );

    // Initialize EnvironmentManager for custom environments
    this.environmentManager = new EnvironmentManager({
      scene: this.sceneManager.scene,
      renderer: this.sceneManager.renderer,
    });

    // Initialize PingManager for collaborative ping markers
    this.pingManager = new PingManager(this.sceneManager.scene);

    // Initialize DrawingManager for collaborative drawing
    this.drawingManager = new DrawingManager(this.sceneManager.scene, this.sceneManager.renderer);

    // Initialize OffscreenIndicatorManager for off-screen ping arrows
    this.offscreenIndicatorManager = new OffscreenIndicatorManager(
      this.sceneManager.camera,
      this.sceneManager.renderer.domElement.parentElement
    );

    // Initialize raycaster for terrain interaction (pings and drawing)
    this.raycaster = new THREE.Raycaster();
    this.groundPlane = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0); // Y-up plane at ground level (fallback)
    this.useArenaRaycast = true; // Use arena meshes for raycasting instead of ground plane
    this.activeTool = 'select'; // Current tool mode: 'select', 'ping', 'draw', 'eraser'
    this.onTerrainClick = null; // Callback for terrain clicks
    this.onTerrainDrag = null; // Callback for terrain drags (drawing)
    this.isDrawing = false; // Tracks if currently drawing
    this.currentStrokeId = null; // Current stroke ID for drawing
    this.drawColor = '#FF5722'; // Default draw color
    this.drawThickness = 3; // Default draw thickness

    // Setup terrain interaction handlers
    this._setupTerrainInteraction();

    // Connect DevToolsManager to EnvironmentManager for skybox/terrain controls
    this.devToolsManager.environmentManager = this.environmentManager;

    // Connect DevToolsManager to ArenaManager for arena decoration controls
    this.devToolsManager.arenaManager = this.arenaManager;

    // Expose globally for debugging
    window.gameEngine = this;
    console.log(
      '[GameEngine] Instance exposed as window.gameEngine for debugging'
    );

    // Skip auto-init if requested (e.g., for live mode which uses initLiveMode instead)
    if (!callbacks?.skipAutoInit) {
      this.init();
    }
  }

  _reportProgress(step, message) {
    if (this.callbacks.onLoadingProgress) {
      this.callbacks.onLoadingProgress(step, message);
    }
  }

  async init() {
    try {
      // Load skybox if one is specified, otherwise use default background
      // If no skybox is provided (null/undefined), skip skybox loading and use a simple background
      const initialSkybox = this.callbacks.initialSkyboxId;
      if (initialSkybox) {
        this._reportProgress('skybox', 'Loading skybox...');
        console.log('[GameEngine] Loading skybox:', initialSkybox);
        await this.sceneManager.loadSkybox(initialSkybox);
      } else {
        console.log('[GameEngine] No skybox specified, using default background');
        // Use a simple dark background when no skybox is loaded
        this.sceneManager.setDefaultBackground();
      }

      // Load arena meshes
      this._reportProgress('arena', 'Loading arena...');
      console.log('[GameEngine] Loading arena...');
      await this.arenaManager.loadArenaMeshes();

      // Load arena decoration (stands, stadium surroundings)
      console.log('[GameEngine] Loading arena decoration...');
      await this.arenaManager.loadArenaDecor(true);

      // Load drawing collider (simplified mesh for drawing raycasting)
      await this.arenaManager.loadDrawingCollider(false);

      // Load replay from binary data if provided, otherwise from file
      this._reportProgress('replay', 'Loading replay...');
      if (this.binaryData) {
        console.log('[GameEngine] Loading replay from binary data...');
        await this.loadReplayFromBinary(this.binaryData);
      } else {
        // Use VITE_REPLAY_FILE env var or default to compiled binary
        const replayFile = import.meta.env.VITE_REPLAY_FILE || 'sample.compiled.bin';
        console.log(`[GameEngine] Loading replay: ${replayFile}`);
        await this.loadReplay(replayFile);
      }

      // Now that replay is loaded, preload only the car models used in this replay
      // This is much more efficient than preloading all 7 car models
      if (this.player) {
        const players = this.player.getAllPlayers().map(p => ({
          carName: p.carName,
          hitboxType: p.hitboxType
        }));
        this.actorManager.carModelLoader.preloadModelsForReplay(players);
      }

      // Pre-compile explosion shaders to avoid freeze on first demolition
      // Old explosion pool (deprecated but kept for compatibility)
      await precompileExplosionMaterials(
        this.sceneManager.renderer,
        this.sceneManager.scene,
        this.sceneManager.camera
      );

      // Reset explosion pools from previous navigation (fixes React Router navigation issue)
      // This clears stale singleton pools that may reference disposed scene/renderer
      resetExplosionPools();

      // New simplified explosion pool - warmup shaders
      warmupExplosionPool(
        this.sceneManager.scene,
        this.sceneManager.renderer,
        this.sceneManager.camera
      );

      // Set render context on effects manager for goal explosions
      this.effectsManager.setRenderContext(
        this.sceneManager.renderer,
        this.sceneManager.camera
      );

      // Wait for all async model loading to complete BEFORE shader compilation
      // This prevents race conditions where models finish loading during compileAsync,
      // causing "Cannot read properties of undefined (reading 'isReady')" errors
      this._reportProgress('models', 'Loading models...');
      console.log('[GameEngine] Waiting for model preloading...');
      await Promise.all([
        this.actorManager.waitForBallModel(),
        this.actorManager.carModelLoader.waitForPreload(),
      ]);
      console.log('[GameEngine] All models preloaded');

      // Guard against disposed renderer (React StrictMode double-mounting)
      if (!this.sceneManager?.renderer) {
        console.warn('[GameEngine] Renderer disposed during model loading, aborting');
        return;
      }

      // Compile all shaders - try async first, fallback to sync
      this._reportProgress('shaders', 'Compiling shaders...');
      console.log('[GameEngine] Compiling shaders...');

      // Let React update the loading message
      await new Promise(resolve => setTimeout(resolve, 50));

      // Try async compilation first (Three.js 158+ with KHR_parallel_shader_compile)
      // Falls back to sync render if async fails or isn't available
      console.time('[GameEngine] Shader compilation');
      let useAsyncCompile = false;

      // Guard against disposed renderer (React StrictMode double-mounting)
      if (!this.sceneManager?.renderer) {
        console.warn('[GameEngine] Renderer disposed during init, aborting');
        return;
      }

      if (this.sceneManager.renderer.compileAsync) {
        try {
          await this.sceneManager.renderer.compileAsync(
            this.sceneManager.scene,
            this.sceneManager.camera
          );
          useAsyncCompile = true;
          console.log('[GameEngine] Async shader compilation complete');
        } catch (err) {
          console.warn('[GameEngine] Async compile failed, falling back to sync:', err.message);
        }
      }

      // Guard again after async operation
      if (!this.sceneManager?.renderer) {
        console.warn('[GameEngine] Renderer disposed during shader compilation, aborting');
        return;
      }

      if (!useAsyncCompile) {
        // Fallback: sync render (blocks main thread but always works)
        console.log('[GameEngine] Using sync render for shader compilation');
        this.sceneManager.render();
      }
      console.timeEnd('[GameEngine] Shader compilation');

      this._reportProgress('ready', 'Ready!');

      // Signal that the engine is fully ready
      if (this.callbacks.onReady) {
        this.callbacks.onReady();
      }

      this.animate();
    } catch (error) {
      console.error('[GameEngine] Init failed:', error);
      if (this.callbacks.onError) {
        this.callbacks.onError(error);
      }
    }
  }

  async loadReplayFromBinary(arrayBuffer) {
    console.log('[GameEngine] Loading replay with framework from binary...');
    this.player = await this.replayLoader.loadFromBinary(arrayBuffer);

    if (!this.player) {
      console.error('[GameEngine] Failed to load player API from binary');
      return;
    }

    this._initializePlayerData();
  }

  async loadReplay(url) {
    console.log('[GameEngine] Loading replay with framework...');
    this.player = await this.replayLoader.load(url);

    if (!this.player) {
      console.error('[GameEngine] Failed to load player API');
      return;
    }

    this._initializePlayerData();
  }

  /**
   * Initialize all player data and UI callbacks after loading a replay
   * Common code for both loadReplay() and loadReplayFromBinary()
   */
  _initializePlayerData() {
    console.log('[GameEngine] Player API loaded successfully');

    // Get duration from framework
    this.maxTime = this.player.duration;

    // Notify UI of max time
    if (this.callbacks.onMaxTimeUpdate) {
      this.callbacks.onMaxTimeUpdate(this.maxTime);
    }

    // Get game time data from framework
    this.gameTimeMap = this.player.getGameTimeMap();
    this.countdownEvents = this.player.getCountdownEvents();

    // Notify UI of game time map
    if (this.callbacks.onGameTimeInfoUpdate) {
      this.callbacks.onGameTimeInfoUpdate({
        gameTimeMap: this.gameTimeMap,
      });
    }

    // Notify UI of countdown events
    if (this.callbacks.onCountdownEventsUpdate) {
      this.callbacks.onCountdownEventsUpdate(this.countdownEvents);
    }

    // Get player teams from framework
    const playerTeams = this.player.getPlayerTeams();
    this.actorManager.setPlayerTeams(playerTeams);
    this.nameTagManager.setPlayerTeams(playerTeams);

    // Initialize actors from framework (creates ball + car meshes)
    this.actorManager.initFromFramework(this.player);

    // Initialize animation system for smooth motion
    const timelines = this.player.getTimelines();
    this.actorManager.initInterpolants(timelines);

    // Notify UI of timelines for debug visualization
    if (this.callbacks.onTimelinesReady) {
      this.callbacks.onTimelinesReady(
        timelines.ballTimeline || [],
        timelines.playerTimelines || {}
      );
    }

    // Notify UI of player stats timelines for real-time stats display
    if (this.callbacks.onPlayerStatsTimelinesReady) {
      const playerStatsTimelines = this.player.getPlayerStatsTimelines();
      this.callbacks.onPlayerStatsTimelinesReady(playerStatsTimelines || {});
    }

    // Notify UI of game event timeline for overtime detection
    if (this.callbacks.onGameEventTimelineReady) {
      const gameEventTimeline = this.player.getGameEventTimeline();
      this.callbacks.onGameEventTimelineReady(gameEventTimeline || []);
    }

    // Notify UI of advanced stats (018-stats-compiler)
    if (this.callbacks.onAdvancedStatsReady) {
      const advancedStats = this.player.getAdvancedStats();
      this.callbacks.onAdvancedStatsReady(advancedStats || null);
    }

    // Start animations (they will be paused until play() is called)
    if (this.actorManager.animationMixer) {
      this.actorManager.startAnimations();
      this.actorManager.seekAnimations(this.currentTime);
      this.actorManager.pauseAnimations(); // Start paused
    }

    // Notify UI of players
    this.player.playerList.forEach((playerInfo) => {
      if (this.callbacks.onPlayerListUpdate) {
        this.callbacks.onPlayerListUpdate(playerInfo.name);
      }
    });

    // Notify UI of player teams
    if (this.callbacks.onPlayerTeamsUpdate) {
      this.callbacks.onPlayerTeamsUpdate(playerTeams);
    }

    // Notify UI of player car info (for info popup)
    if (this.callbacks.onPlayerCarInfoUpdate) {
      const carInfo = {};
      this.player.getAllPlayers().forEach((playerEntity) => {
        // Find full player info from playerList
        const playerInfo = this.player.playerList.find(p => p.name === playerEntity.name);
        carInfo[playerEntity.name] = {
          carName: playerEntity.carName,
          hitboxType: playerEntity.hitboxType,
          // Stats from playerInfo
          platform: playerInfo?.platform || null,
          goals: playerInfo?.goals || 0,
          assists: playerInfo?.assists || 0,
          saves: playerInfo?.saves || 0,
          shots: playerInfo?.shots || 0,
          matchScore: playerInfo?.score || 0,
          isBot: playerInfo?.isBot || false,
        };
      });
      this.callbacks.onPlayerCarInfoUpdate(carInfo);
    }

    // Get events from framework (goals, demos, etc.)
    const events = this.player.getEvents();
    console.log(`[GameEngine] Loaded ${events.length} events from framework`);
    console.log('[GameEngine] Events:', events.map(e => ({ type: e.type, time: e.time?.toFixed(2), player: e.player, team: e.team })));

    // Notify UI of events
    if (this.callbacks.onEventsLoaded) {
      this.callbacks.onEventsLoaded(events);
    }

    // Setup framework event listeners
    console.log('[GameEngine] Setting up framework event listeners...');
    this.setupFrameworkEvents();
    console.log('[GameEngine] Framework setup complete');

  }

  setupFrameworkEvents() {
    if (!this.player) return;

    // Create visual boost pads from framework data
    this.createBoostPads();

    // Game events - trigger visual effects (framework handles deduplication)
    this.player.on('goal', (playerName, team, time) => {
      console.log(`⚽ [Framework] GOAL by ${playerName} (Team ${team}) at ${time.toFixed(2)}s`);
      // Trigger goal explosion at ball position
      const ball = this.player.getBall();
      if (ball && this.isPlaying) {
        const pos = ball.position;
        const ballPos = new THREE.Vector3(pos.x, pos.y, pos.z);
        this.effectsManager.triggerGoalExplosion(ballPos, team);
      }
    });

    this.player.on('demo', (victim, attacker, time) => {
      console.log(`💥 [Framework] DEMO: ${attacker} destroyed ${victim} at ${time.toFixed(2)}s`);
      // Trigger demo explosion at victim car position with car rotation
      if (this.isPlaying) {
        const victimPlayer = this.player.getPlayer(victim);
        const attackerPlayer = this.player.getPlayer(attacker);
        if (victimPlayer) {
          const victimTeam = victimPlayer.team;
          const attackerTeam = attackerPlayer?.team ?? (victimTeam === 0 ? 1 : 0);
          const pos = victimPlayer.position;
          const rot = victimPlayer.rotation;
          const victimPos = new THREE.Vector3(pos.x, pos.y, pos.z);
          const victimRot = new THREE.Quaternion(rot.x, rot.y, rot.z, rot.w);
          this.effectsManager.triggerDemoExplosion(victimPos, victimRot, victimTeam);

          // Notify UI about demo event for killfeed
          if (this.callbacks.onDemoEvent) {
            this.callbacks.onDemoEvent({
              victim,
              attacker,
              victimTeam,
              attackerTeam,
              time,
            });
          }
        }
      }
    });

    this.player.on('match-start', () => {
      console.log('🏁 [Framework] Match started!');
    });

    this.player.on('match-end', () => {
      console.log('🏁 [Framework] Match ended!');
    });

    this.player.on('overtime-start', () => {
      console.log('⏰ [Framework] OVERTIME!');
    });

    // Player-specific events
    this.player.getAllPlayers().forEach((playerEntity) => {

      playerEntity.on('demo', () => {
        console.log(`💥 [Framework] ${playerEntity.name} was demolished!`);
      });

      playerEntity.on('respawn', () => {
        console.log(`✨ [Framework] ${playerEntity.name} respawned`);
      });
    });

  }

  createBoostPads() {
    if (!this.player || !this.player.boostPads) return;

    console.log(
      `[GameEngine] Creating ${this.player.boostPads.size} boost pads...`
    );

    this.boostPadMeshes = new Map();

    this.player.boostPads.forEach((pad, padId) => {
      const isBig = pad.isBig;

      // Debug first few pads
      if (padId < 3) {
        console.log(
          `Pad ${padId} (${isBig ? 'BIG' : 'small'}):`,
          `Position: (${pad.position.x.toFixed(2)}, ${pad.position.y.toFixed(2)}, ${pad.position.z.toFixed(2)})`
        );
      }

      let geometry, material, mesh;

      if (isBig) {
        // Big pads: Glowing sphere (dimensions in Unreal Units)
        // Big boost pads in RL are ~144 UU diameter, we use a smaller visual sphere
        const radius = 50; // Visual sphere radius in UU
        geometry = new THREE.SphereGeometry(radius, 16, 16);
        material = new THREE.MeshStandardMaterial({
          color: 0xffdd44, // Bright yellow/orange core
          emissive: 0xffaa00,
          emissiveIntensity: 1.0,
          metalness: 0.3,
          roughness: 0.2,
          transparent: true, // CRITICAL: Required for opacity changes
          opacity: 1.0,
          depthWrite: false, // Fix transparency sorting with arena walls
        });
        mesh = new THREE.Mesh(geometry, material);
        mesh.renderOrder = 100; // Render after arena walls

        // Add glow effect - larger transparent sphere with additive blending
        const glowGeometry = new THREE.SphereGeometry(radius * 2.0, 16, 16);
        const glowMaterial = new THREE.MeshBasicMaterial({
          color: 0xffaa00,
          transparent: true,
          opacity: 0.3,
          blending: THREE.AdditiveBlending,
          side: THREE.BackSide, // Render inside of sphere for halo effect
          depthWrite: false,
        });
        const glowMesh = new THREE.Mesh(glowGeometry, glowMaterial);
        glowMesh.renderOrder = 99;
        mesh.add(glowMesh);
        mesh.userData.glowMesh = glowMesh;

        // Add second smaller glow layer for more intensity
        const innerGlowGeometry = new THREE.SphereGeometry(radius * 1.4, 16, 16);
        const innerGlowMaterial = new THREE.MeshBasicMaterial({
          color: 0xffcc00,
          transparent: true,
          opacity: 0.4,
          blending: THREE.AdditiveBlending,
          side: THREE.BackSide,
          depthWrite: false,
        });
        const innerGlowMesh = new THREE.Mesh(innerGlowGeometry, innerGlowMaterial);
        innerGlowMesh.renderOrder = 99;
        mesh.add(innerGlowMesh);
        mesh.userData.innerGlowMesh = innerGlowMesh;

        mesh.userData.needsLight = true;
      } else {
        // Small pads: Flat cylinder at ground level (dimensions in Unreal Units)
        // Small boost pads in RL are ~64 UU diameter
        const radius = 40; // Visual radius in UU
        const height = 5; // Flat disk height in UU
        geometry = new THREE.CylinderGeometry(radius, radius, height, 16);
        material = new THREE.MeshStandardMaterial({
          color: 0xffcc00, // Yellow/orange
          emissive: 0xff9900,
          emissiveIntensity: 0.4,
          metalness: 0.2,
          roughness: 0.8,
          transparent: true, // CRITICAL: Required for opacity changes
          opacity: 1.0,
          depthWrite: false, // Fix transparency sorting with arena walls
        });
        mesh = new THREE.Mesh(geometry, material);
        mesh.renderOrder = 100; // Render after arena walls
      }

      // Position the boost pad (framework provides positions in UU)
      // BoostPadCompiler uses Unreal coordinates: x, y (length), z (height)
      // Three.js expects: x, y (height), z (depth)
      // So we need to swap Y and Z here (unlike physics which does it in the compiler)
      const groundLevel = 10; // Just above ground to avoid z-fighting (in UU)
      const floatHeight = isBig ? 150 : groundLevel; // Big pads at 150 UU, small pads near ground

      mesh.position.set(
        pad.position.x,       // X stays the same
        floatHeight,          // Y = height (use our custom float height)
        pad.position.y        // Z = Unreal Y (position along the field length)
      );

      // Store metadata (preserve existing userData like light reference)
      mesh.userData.padId = padId;
      mesh.userData.isBig = isBig;
      mesh.userData.isAvailable = true;

      this.sceneManager.scene.add(mesh);
      this.boostPadMeshes.set(padId, mesh);

      // Add point light for big pads directly to scene
      if (mesh.userData.needsLight) {
        const light = new THREE.PointLight(0xffaa00, 1.0, 600);
        light.decay = 0; // No decay, but limited by distance
        light.position.set(
          pad.position.x,
          floatHeight - 50,
          pad.position.y
        );
        this.sceneManager.scene.add(light);
        mesh.userData.light = light;
      }
    });

    console.log(`✓ Created ${this.boostPadMeshes.size} boost pad meshes`);
  }

  updateBoostPads() {
    if (!this.player || !this.boostPadMeshes) return;

    // Debug log once per second to check state changes
    if (!this._lastBoostDebugTime) this._lastBoostDebugTime = 0;
    const now = Date.now();
    const shouldDebug = now - this._lastBoostDebugTime > 1000;

    let unavailableCount = 0;

    this.player.boostPads.forEach((pad, padId) => {
      const mesh = this.boostPadMeshes.get(padId);
      if (!mesh) return;

      // Update visibility and color based on availability
      const isAvailable = pad.isAvailable;
      const wasAvailable = mesh.userData.isAvailable;

      mesh.userData.isAvailable = isAvailable;

      if (!isAvailable) unavailableCount++;

      if (isAvailable) {
        // Available - orange/yellow (normal color)
        mesh.material.color.setHex(pad.isBig ? 0xffdd44 : 0xffcc00);
        mesh.material.emissive.setHex(pad.isBig ? 0xffaa00 : 0xff9900);
        mesh.material.emissiveIntensity = pad.isBig ? 1.0 : 0.4;
        mesh.material.opacity = 1.0;
        mesh.visible = true;
        // Turn on light and glow for big pads
        if (mesh.userData.light) {
          mesh.userData.light.intensity = 1.0;
        }
        if (mesh.userData.glowMesh) {
          mesh.userData.glowMesh.visible = true;
        }
        if (mesh.userData.innerGlowMesh) {
          mesh.userData.innerGlowMesh.visible = true;
        }
      } else {
        // Picked up - transparent (faded out)
        mesh.material.color.setHex(pad.isBig ? 0xffaa00 : 0xffcc00);
        mesh.material.emissive.setHex(0x000000); // No emission when inactive
        mesh.material.emissiveIntensity = 0.0;
        mesh.material.opacity = 0.2; // Very transparent
        mesh.visible = true; // Still visible but faded
        // Turn off light and glow for big pads
        if (mesh.userData.light) {
          mesh.userData.light.intensity = 0;
        }
        if (mesh.userData.glowMesh) {
          mesh.userData.glowMesh.visible = false;
        }
        if (mesh.userData.innerGlowMesh) {
          mesh.userData.innerGlowMesh.visible = false;
        }
      }
    });

  }

  play() {
    this.isPlaying = true;
    // Start/resume animation system
    if (this.actorManager.animationMixer) {
      this.actorManager.resumeAnimations();
    }
    if (this.callbacks.onPlayStateChange)
      this.callbacks.onPlayStateChange(true);
  }

  pause() {
    this.isPlaying = false;
    // Pause animation system
    if (this.actorManager.animationMixer) {
      this.actorManager.pauseAnimations();
    }
    if (this.callbacks.onPlayStateChange)
      this.callbacks.onPlayStateChange(false);
  }

  togglePlay() {
    if (this.isPlaying) this.pause();
    else this.play();
  }

  seek(time) {
    this.currentTime = time;
    // Sync animation system to new time
    if (this.actorManager.animationMixer) {
      this.actorManager.seekAnimations(time);
    }
    // Reset ball trail to avoid stale segments connecting old/new positions
    if (this.effectsManager) {
      this.effectsManager.resetBallTrail();
    }
    // Reset wheel position tracking to avoid jumps after seek
    if (this.actorManager) {
      this.actorManager.resetWheelTracking();
    }
  }

  setCameraMode(mode) {
    this.cameraMode = mode;

    if (mode === 'free') {
      this.inputManager.enabled = true;
      this.cameraManager.setMode('free');
    } else if (mode === 'ballOrbit') {
      this.inputManager.enabled = false;
      // Set ball as target for orbit mode
      const ball = this.actorManager.ballMesh;
      if (ball) {
        this.cameraManager.setTargetBall(ball);
      }
      this.cameraManager.setMode('ballOrbit');
    } else {
      this.inputManager.enabled = false;
      this.cameraManager.setMode('player');
    }
  }

  selectPlayer(playerName) {
    this.selectedPlayerName = playerName;
  }

  updateCameraSettings(settings) {
    this.cameraSettings = settings;
  }

  setPlaybackSpeed(speed) {
    this.playbackSpeed = speed;
  }

  setInterpolationEnabled(enabled) {
    this.actorManager.setInterpolationEnabled(enabled);
  }

  resize() {
    this.sceneManager.onWindowResize();
    // Update LineMaterial resolution for fat lines
    if (this.drawingManager) {
      this.drawingManager.updateResolution();
    }
  }

  setExposure(value) {
    this.sceneManager.setExposure(value);
  }

  setSkybox(skyboxId) {
    this.sceneManager.setSkybox(skyboxId);
  }

  setShowHitboxes(enabled) {
    this.hitboxManager.setEnabled(enabled);
  }

  setSpeedDisplaySettings(showBallSpeed, showCarSpeed, speedUnit) {
    this.speedLabelManager.setSettings(showBallSpeed, showCarSpeed, speedUnit);
  }

  /**
   * Update viewer cameras for collaborative viewing
   * @param {Object} participants - Map of participantId -> participant data
   * @param {string} selfId - Local viewer's ID
   */
  updateViewerCameras(participants, selfId) {
    if (this.viewerCameraManager) {
      this.viewerCameraManager.updateFromParticipants(participants, selfId);
    }
  }

  /**
   * Update a single viewer's camera position/rotation
   * @param {string} participantId
   * @param {object} cameraState - { position, rotation, mode, targetPlayer }
   */
  updateViewerCamera(participantId, cameraState) {
    if (this.viewerCameraManager) {
      this.viewerCameraManager.updateViewerCamera(participantId, cameraState);
    }
  }

  /**
   * Get the current camera state for broadcasting
   * @returns {object} { position, rotation, mode, targetPlayer, orbitParams }
   */
  getCameraState() {
    const camera = this.sceneManager.camera;

    // Extract orbit parameters from CameraControls when in ballOrbit mode
    // These are the only values needed to replicate the camera position relative to the ball
    let orbitParams = null;
    if (this.cameraMode === 'ballOrbit' && this.cameraManager?.controls) {
      orbitParams = {
        distance: this.cameraManager.controls.distance,
        azimuth: this.cameraManager.controls.azimuthAngle,
        polar: this.cameraManager.controls.polarAngle,
      };
    }

    return {
      position: {
        x: camera.position.x,
        y: camera.position.y,
        z: camera.position.z,
      },
      rotation: {
        x: camera.quaternion.x,
        y: camera.quaternion.y,
        z: camera.quaternion.z,
        w: camera.quaternion.w,
      },
      mode: this.cameraMode,
      targetPlayer: this.selectedPlayerName,
      orbitParams,
    };
  }

  // ============================================
  // Clip Recording/Playback Methods (024-clip-system)
  // ============================================

  /**
   * Set clip mode
   * @param {'recording' | 'playback' | null} mode - The clip mode
   * @param {Object} [options] - Options for the mode
   * @param {Object} [options.cameraData] - Camera data for playback mode
   * @param {number} [options.startTime] - Start time for playback mode
   */
  setClipMode(mode, options = {}) {
    // Stop current mode
    if (this.clipMode === 'recording') {
      this.clipRecordingManager.stop();
    } else if (this.clipMode === 'playback') {
      this.clipPlaybackManager.stop();
    }

    this.clipMode = mode;

    if (mode === 'playback' && options.cameraData) {
      this.clipPlaybackManager.load(options.cameraData, options.startTime || 0);
      this.clipPlaybackManager.play();
    }

    console.log('[GameEngine] Clip mode set to:', mode);
  }

  /**
   * Start recording camera movements
   */
  startClipRecording() {
    this.clipMode = 'recording';
    // Set player list for name to index conversion during recording
    if (this.player?.playerList) {
      this.clipRecordingManager.setPlayerList(this.player.playerList);
    }
    this.clipRecordingManager.start(this.currentTime);
    console.log('[GameEngine] Clip recording started at:', this.currentTime);
  }

  /**
   * Stop recording and get recorded data
   * @returns {Object} Camera recording data
   */
  stopClipRecording() {
    if (this.clipMode !== 'recording') {
      console.warn('[GameEngine] Not in recording mode');
      return null;
    }

    const data = this.clipRecordingManager.stop();
    this.clipMode = null;
    console.log('[GameEngine] Clip recording stopped. Frames:', data.frames.length);
    return data;
  }

  /**
   * Get recorded data without stopping
   * @returns {Object} Camera recording data
   */
  getRecordedData() {
    return this.clipRecordingManager.getData();
  }

  /**
   * Start clip playback
   * @param {Object} cameraData - Camera data (CameraRecording or CameraKeyframes)
   * @param {number} startTime - Game time when clip starts
   */
  startClipPlayback(cameraData, startTime) {
    this.clipMode = 'playback';
    this._clipPlaybackDebugLogged = false; // Reset debug flag
    this._clipFallbackLogged = false; // Reset fallback debug flag
    this.clipPlaybackManager.load(cameraData, startTime);
    this.clipPlaybackManager.play();

    // Seek to clip start time
    this.seek(startTime);
    this.play();

    console.log('[GameEngine] Clip playback started at:', startTime);
  }

  /**
   * Stop clip playback
   */
  stopClipPlayback() {
    if (this.clipMode !== 'playback') {
      console.warn('[GameEngine] Not in playback mode');
      return;
    }

    this.clipPlaybackManager.stop();
    this.clipMode = null;
    console.log('[GameEngine] Clip playback stopped');
  }

  /**
   * Check if clip mode is active
   * @returns {'recording' | 'playback' | null}
   */
  getClipMode() {
    return this.clipMode;
  }

  /**
   * Check if currently recording
   * @returns {boolean}
   */
  isRecordingClip() {
    return this.clipMode === 'recording' && this.clipRecordingManager.isActive();
  }

  /**
   * Check if currently playing back a clip
   * @returns {boolean}
   */
  isPlayingClip() {
    return this.clipMode === 'playback' && this.clipPlaybackManager.isActive();
  }

  /**
   * Resume clip playback (after pause or end)
   */
  resumeClipPlayback() {
    if (this.clipMode !== 'playback') {
      console.warn('[GameEngine] Not in playback mode');
      return;
    }

    this.clipPlaybackManager.play();
    this.play();
    console.log('[GameEngine] Clip playback resumed');
  }

  /**
   * Replay clip from beginning
   */
  replayClip() {
    if (this.clipMode !== 'playback' || !this.clipPlaybackManager.isLoaded()) {
      console.warn('[GameEngine] No clip loaded');
      return;
    }

    const clipStartTime = this.clipPlaybackManager.clipStartTime;
    this.seek(clipStartTime);
    this.clipPlaybackManager.seek(0);
    this.clipPlaybackManager.play();
    this.play();
    console.log('[GameEngine] Clip replay from start');
  }

  /**
   * Seek within the clip
   * @param {number} clipTime - Time in seconds from clip start
   */
  seekClip(clipTime) {
    if (this.clipMode !== 'playback' || !this.clipPlaybackManager.isLoaded()) {
      console.warn('[GameEngine] No clip loaded');
      return;
    }

    const clipStartTime = this.clipPlaybackManager.clipStartTime;
    const gameTime = clipStartTime + clipTime;
    this.seek(gameTime);
    // Use seekAndApply to update camera even when paused (scrubbing)
    this.clipPlaybackManager.seekAndApply(clipTime * 1000); // Convert to ms
  }

  /**
   * Get clip playback info
   * @returns {{ currentTime: number, duration: number, progress: number } | null}
   */
  getClipPlaybackInfo() {
    if (this.clipMode !== 'playback' || !this.clipPlaybackManager.isLoaded()) {
      return null;
    }

    const duration = this.clipPlaybackManager.getDuration() / 1000; // Convert to seconds
    const currentTime = this.clipPlaybackManager.currentTime / 1000; // Convert to seconds
    const progress = duration > 0 ? currentTime / duration : 0;

    // Get camera state (mode and followed player)
    const cameraState = this.clipPlaybackManager.getCameraState();
    let followedPlayerName = null;

    if (cameraState.mode === 'p' && cameraState.targetPlayerIndex !== null) {
      const playerInfo = this.player?.playerList?.[cameraState.targetPlayerIndex];
      if (playerInfo) {
        followedPlayerName = playerInfo.name;
      }
    }

    return {
      currentTime: Math.max(0, Math.min(currentTime, duration)),
      duration,
      progress: Math.max(0, Math.min(progress, 1)),
      cameraMode: cameraState.mode, // 'f' = freecam, 'b' = ballcam, 'p' = playercam
      followedPlayerName,
    };
  }

  /**
   * Capture the current frame as a JPEG blob for thumbnail
   * Fixed 16:9 aspect ratio (640x360) with center crop
   * @param {number} quality - JPEG quality 0-1 (default 0.85)
   * @returns {Promise<Blob>} JPEG blob
   */
  async captureFrame(quality = 0.85) {
    const renderer = this.sceneManager?.renderer;
    if (!renderer) {
      throw new Error('Renderer not available');
    }

    // Force a render to ensure we have the latest frame
    renderer.render(this.sceneManager.scene, this.sceneManager.camera);

    const canvas = renderer.domElement;

    // Fixed output dimensions (16:9 aspect ratio)
    const targetWidth = 640;
    const targetHeight = 360;
    const targetAspect = targetWidth / targetHeight;

    // Source dimensions
    const srcWidth = canvas.width;
    const srcHeight = canvas.height;
    const srcAspect = srcWidth / srcHeight;

    // Calculate crop region (center crop to match target aspect ratio)
    let cropX = 0;
    let cropY = 0;
    let cropWidth = srcWidth;
    let cropHeight = srcHeight;

    if (srcAspect > targetAspect) {
      // Source is wider - crop left and right
      cropWidth = srcHeight * targetAspect;
      cropX = (srcWidth - cropWidth) / 2;
    } else if (srcAspect < targetAspect) {
      // Source is taller - crop top and bottom
      cropHeight = srcWidth / targetAspect;
      cropY = (srcHeight - cropHeight) / 2;
    }

    // Create a temporary canvas for the output
    const tempCanvas = document.createElement('canvas');
    tempCanvas.width = targetWidth;
    tempCanvas.height = targetHeight;

    const ctx = tempCanvas.getContext('2d');
    // Draw cropped and scaled image
    ctx.drawImage(
      canvas,
      cropX, cropY, cropWidth, cropHeight, // Source rectangle
      0, 0, targetWidth, targetHeight       // Destination rectangle
    );

    // Convert to blob
    return new Promise((resolve, reject) => {
      tempCanvas.toBlob(
        (blob) => {
          if (blob) {
            resolve(blob);
          } else {
            reject(new Error('Failed to capture frame'));
          }
        },
        'image/jpeg',
        quality
      );
    });
  }

  // ============================================
  // Cinematic Mode Keyframe Methods (024-clip-system US2)
  // ============================================

  /**
   * Add a keyframe at the current camera position
   * 026-clip-editor-redesign: Only creates keyframe data, does NOT add to visualizer
   * The useEffect in Viewer.tsx handles syncing clipEditor.keyframes to the visualizer
   * @param {number} time - Time in seconds within the clip
   * @returns {Object} The created keyframe
   */
  addKeyframe(time) {
    const camera = this.sceneManager.camera;
    const keyframe = {
      id: `kf_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      t: time * 1000, // Convert to ms
      px: camera.position.x,
      py: camera.position.y,
      pz: camera.position.z,
      qx: camera.quaternion.x,
      qy: camera.quaternion.y,
      qz: camera.quaternion.z,
      qw: camera.quaternion.w,
      fov: camera.fov || 110,
      easing: 'ease-in-out',
    };

    // Note: Don't add to visualizer here - the useEffect sync handles it
    return keyframe;
  }

  /**
   * Remove a keyframe by ID
   * 026-clip-editor-redesign: No longer removes from visualizer directly
   * The useEffect in Viewer.tsx handles syncing clipEditor.keyframes to the visualizer
   * @param {string} id - Keyframe ID
   */
  removeKeyframe(id) {
    // Note: Don't remove from visualizer here - the useEffect sync handles it
    // This method is kept for API compatibility but is now a no-op
  }

  /**
   * Update a keyframe
   * 026-clip-editor-redesign: No longer updates visualizer directly
   * The useEffect in Viewer.tsx handles syncing clipEditor.keyframes to the visualizer
   * @param {string} id - Keyframe ID
   * @param {Object} updates - Partial keyframe data
   */
  updateKeyframe(id, updates) {
    // Note: Don't update visualizer here - the useEffect sync handles it
    // This method is kept for API compatibility but is now a no-op
  }

  /**
   * Get all keyframes
   * @returns {Array} Array of keyframes
   */
  getKeyframes() {
    return this.keyframeVisualizer.getKeyframes();
  }

  /**
   * Set all keyframes
   * @param {Array} keyframes - Array of keyframes
   */
  setKeyframes(keyframes) {
    this.keyframeVisualizer.setKeyframes(keyframes);
  }

  /**
   * Select a keyframe
   * @param {string} id - Keyframe ID or null to deselect
   */
  selectKeyframe(id) {
    this.keyframeVisualizer.selectKeyframe(id);
  }

  /**
   * Show keyframe visualizer
   */
  showKeyframes() {
    this.keyframeVisualizer.show();
  }

  /**
   * Hide keyframe visualizer
   */
  hideKeyframes() {
    this.keyframeVisualizer.hide();
  }

  /**
   * Show the animated preview camera that follows the trajectory
   * 026-clip-editor-redesign: Animated preview camera
   * @param {number} clipStartTime - Start time of the clip in seconds (game time)
   */
  showPreviewCamera(clipStartTime) {
    this._previewCameraClipStart = clipStartTime;
    this.keyframeVisualizer.showPreviewCamera();
  }

  /**
   * Hide the animated preview camera
   */
  hidePreviewCamera() {
    this.keyframeVisualizer.hidePreviewCamera();
    this._previewCameraClipStart = null;
  }

  /**
   * Check if preview camera is visible
   * @returns {boolean}
   */
  isPreviewCameraVisible() {
    return this.keyframeVisualizer.previewCameraVisible;
  }

  /**
   * Clear all keyframes
   */
  clearKeyframes() {
    this.keyframeVisualizer.setKeyframes([]);
  }

  /**
   * Check for keyframe click
   * @param {number} x - Normalized device X (-1 to 1)
   * @param {number} y - Normalized device Y (-1 to 1)
   * @returns {Object|null} Clicked keyframe or null
   */
  checkKeyframeClick(x, y) {
    return this.keyframeVisualizer.handleClick(x, y);
  }

  /**
   * Get cinematic camera data from current keyframes
   * @returns {Object} CameraKeyframes object
   */
  getCinematicCameraData() {
    const keyframes = this.keyframeVisualizer.getKeyframes();
    return {
      type: 'cinematic',
      interpolation: 'catmullrom',
      tension: 0.5,
      keyframes,
    };
  }

  dispose() {
    // Cleanup Three.js resources
    this.isPlaying = false;
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
    }

    // Cleanup boost pad meshes
    if (this.boostPadMeshes) {
      this.boostPadMeshes.forEach((mesh) => {
        this.sceneManager.scene.remove(mesh);
        mesh.geometry.dispose();
        mesh.material.dispose();
      });
      this.boostPadMeshes.clear();
    }

    // Cleanup camera manager
    if (this.cameraManager) {
      this.cameraManager.dispose();
    }

    // Cleanup clip managers
    if (this.clipPlaybackManager) {
      this.clipPlaybackManager.dispose();
    }
    if (this.clipRecordingManager) {
      this.clipRecordingManager.clear();
    }
    if (this.keyframeVisualizer) {
      this.keyframeVisualizer.dispose();
    }

    // Cleanup name tag manager
    if (this.nameTagManager) {
      this.nameTagManager.dispose();
    }

    // Cleanup hitbox manager
    if (this.hitboxManager) {
      this.hitboxManager.dispose();
    }

    // Cleanup devtools manager
    if (this.devToolsManager) {
      this.devToolsManager.dispose();
    }

    // Cleanup viewer camera manager
    if (this.viewerCameraManager) {
      this.viewerCameraManager.dispose();
    }

    // Cleanup ping manager
    if (this.pingManager) {
      this.pingManager.dispose();
    }

    // Cleanup drawing manager
    if (this.drawingManager) {
      this.drawingManager.dispose();
    }

    // Cleanup offscreen indicator manager
    if (this.offscreenIndicatorManager) {
      this.offscreenIndicatorManager.dispose();
    }

    // Cleanup environment manager
    if (this.environmentManager) {
      this.environmentManager.dispose();
    }

    if (this.sceneManager) {
      this.sceneManager.dispose();
    }
  }

  animate() {
    if (!this.sceneManager.renderer) return; // Safety check if disposed

    this.animationId = requestAnimationFrame(this.animate);

    const delta = this.clock.getDelta();

    if (this.isPlaying) {
      this.currentTime += delta * this.playbackSpeed;
      if (this.currentTime > this.maxTime) {
        this.currentTime = this.maxTime;
        this.pause();
      }

      // Notify UI of time update (throttle this in production, but for now frame-by-frame is ok or maybe every frames)
      if (this.callbacks.onTimeUpdate) {
        this.callbacks.onTimeUpdate(this.currentTime);
      }
    }

    // Update framework Player API if available
    if (this.player) {
      // Always sync player time with our current time via seek()
      // This ensures entities (including boost pads) are updated every frame
      // Note: We don't use player.update() because GameEngine manages its own playback
      this.player.seek(this.currentTime);

      // Update boost pads visual state
      this.updateBoostPads();

      // Update text overlays (countdown, goal text)
      if (this.callbacks.onTextOverlaysUpdate) {
        const overlays = this.player.getTextOverlaysAt(this.currentTime);
        this.callbacks.onTextOverlaysUpdate(overlays);
      }

      // Update frame info for debug panel
      if (this.callbacks.onFrameInfoUpdate) {
        const frameInfo = this.actorManager.getFrameInfo();
        this.callbacks.onFrameInfoUpdate(frameInfo);
      }
    }

    // Update Three.js animation system (handles smooth interpolation)
    // This must be called BEFORE updateScene so positions are updated
    if (this.isPlaying && this.actorManager.animationMixer) {
      this.actorManager.updateAnimations(delta * this.playbackSpeed);
    }

    this.updateScene();

    // Update effects - pass isPlaying and playbackSpeed so effects scale correctly
    const t0 = performance.now();
    this.effectsManager.update(delta, this.isPlaying, this.playbackSpeed);

    // Update skybox animation (Dev Mode Lot 3 - US2)
    if (this.environmentManager) {
      this.environmentManager.updateSkyboxAnimation(delta);
    }
    const t1 = performance.now();
    if (t1 - t0 > 50) {
      console.warn(`[animate] effectsManager.update took ${(t1-t0).toFixed(1)}ms`);
    }

    // Update dev tools camera (if in dev mode, this handles camera and skips normal camera controls)
    if (this.devToolsManager.isDevModeActive()) {
      this.devToolsManager.update(delta);
    } else if (this.clipMode === 'playback' && this.clipPlaybackManager.isActive()) {
      // Clip playback mode - delegate camera to clip playback manager
      // Debug log first frame of clip playback
      if (!this._clipPlaybackDebugLogged) {
        this._clipPlaybackDebugLogged = true;
        console.log('[GameEngine] Entering clip playback camera update:', {
          clipMode: this.clipMode,
          isActive: this.clipPlaybackManager.isActive(),
          currentTime: this.currentTime,
        });
      }
      const stillPlaying = this.clipPlaybackManager.updateCamera(this.currentTime);
      if (!stillPlaying) {
        // Clip ended - pause playback and notify
        this.pause();
        this.clipPlaybackManager.pause();
        if (this.callbacks.onClipPlaybackEnd) {
          this.callbacks.onClipPlaybackEnd();
        }
      }
    } else {
      // Normal camera mode
      // Debug: if clipMode is playback but we're here, something is wrong
      if (this.clipMode === 'playback' && !this._clipFallbackLogged) {
        this._clipFallbackLogged = true;
        console.warn('[GameEngine] In playback mode but fell into normal camera branch!', {
          clipMode: this.clipMode,
          isActive: this.clipPlaybackManager?.isActive(),
          isPlaying: this.clipPlaybackManager?.isPlaying,
          hasData: !!this.clipPlaybackManager?.clipData,
        });
      }
      this.inputManager.update(delta);
      this.updateCamera(delta);

      // Capture frame if recording
      if (this.clipMode === 'recording' && this.clipRecordingManager.isActive()) {
        const cameraState = this.getCameraState();
        this.clipRecordingManager.captureFrame(delta, cameraState, this.currentTime);
      }

      // Update animated preview camera position (026-clip-editor-redesign)
      // Only when NOT in clip playback mode (because then we ARE the camera)
      // Pass ABSOLUTE time since keyframes store absolute time (t: time * 1000)
      if (this.keyframeVisualizer.previewCameraVisible && this._previewCameraClipStart !== null) {
        const absoluteTimeMs = this.currentTime * 1000;
        this.keyframeVisualizer.updatePreviewCamera(absoluteTimeMs);
      }

      // Always check ghost camera visibility when it's shown (even in edit mode)
      // Hide ghost camera if user is too close (to avoid being inside the mesh)
      if (this.keyframeVisualizer.previewCameraVisible && this.keyframeVisualizer.previewCamera && this.cameraManager?.camera) {
        this.keyframeVisualizer.updatePreviewCameraVisibility(this.cameraManager.camera);
      }
    }

    // Update wheel rotations for all cars (only when playing)
    // Note: updateWheelRotations uses position delta, not time delta
    if (this.isPlaying) {
      this.actorManager.updateWheelRotations();
    }

    // Update debug panel (send data to React)
    if (this.callbacks.onActorsUpdate) {
      this.callbacks.onActorsUpdate(
        this.actorManager.actors,
        this.actorManager.ballActorId
      );
    }

    const renderStart = performance.now();
    this.sceneManager.render();
    const renderEnd = performance.now();
    if (renderEnd - renderStart > 50) {
      console.warn(`[animate] sceneManager.render took ${(renderEnd-renderStart).toFixed(1)}ms`);
    }
  }

  updateScene() {
    if (!this.player) return;

    // Update all actor positions from framework (uses cubic interpolation if available)
    this.actorManager.updateFromFramework(this.player, this.currentTime);

    // Update Boost State for all players from framework
    const playerBoosts = {};
    this.player.getAllPlayers().forEach((playerEntity) => {
      playerBoosts[playerEntity.name] = playerEntity.boost;

      // Only update boost effects (particles) when playing to avoid infinite emission at frozen positions
      // Pass isKickoffReset to skip particles when boost is reset during kickoff
      if (this.isPlaying) {
        this.actorManager.updateBoostState(
          playerEntity.name,
          playerEntity.isBoosting,
          playerEntity.isKickoffReset
        );
      }
    });

    // Update Supersonic State for all players from framework
    if (this.isPlaying) {
      this.player.getAllPlayers().forEach((playerEntity) => {
        this.actorManager.updateSupersonicState(
          playerEntity.name,
          playerEntity.isSupersonic,
          playerEntity.team
        );
      });
    }

    // Notify UI of boost amounts
    if (this.callbacks.onPlayerBoostUpdate) {
      this.callbacks.onPlayerBoostUpdate(playerBoosts);
    }

    // Notify UI of player scores
    const playerScores = {};
    this.player.getAllPlayers().forEach((playerEntity) => {
      playerScores[playerEntity.name] = playerEntity.score;
    });
    if (this.callbacks.onPlayerScoresUpdate) {
      this.callbacks.onPlayerScoresUpdate(playerScores);
    }

    // Update name tags above cars
    // Hide name tag for followed player in player cam mode (or clip playback player mode)
    let followedPlayer = this.cameraMode === 'player' ? this.selectedPlayerName : null;

    // In clip playback mode, check if camera is following a player
    if (this.clipMode === 'playback' && this.clipPlaybackManager?.isLoaded()) {
      const cameraState = this.clipPlaybackManager.getCameraState();
      if (cameraState.mode === 'p' && cameraState.targetPlayerIndex !== null) {
        // Find player name by index
        const playerInfo = this.player?.playerList?.[cameraState.targetPlayerIndex];
        if (playerInfo) {
          followedPlayer = playerInfo.name;
        }
      }
    }

    this.nameTagManager.update(
      this.actorManager.actors,
      playerBoosts,
      this.actorManager.playerNameToCarActorId,
      followedPlayer
    );

    // Update ping markers
    if (this.pingManager) {
      this.pingManager.update(0);
    }

    // Update drawing strokes
    if (this.drawingManager) {
      this.drawingManager.update(0);
    }

    // Update offscreen ping indicators
    if (this.offscreenIndicatorManager && this.pingManager) {
      const activePings = this.pingManager.getActivePings();
      this.offscreenIndicatorManager.update(activePings);
    }

    // Update viewer camera labels (for hybrid sizing)
    if (this.viewerCameraManager) {
      this.viewerCameraManager.update();
    }

    // Update hitbox wireframes
    const player = this.player;
    this.hitboxManager.updateHitboxes(
      this.actorManager.actors,
      this.actorManager.playerNameToCarActorId,
      (playerName) => {
        // Get hitbox type from framework player data
        const playerEntity = player?.getPlayer(playerName);
        return playerEntity?.hitboxType || 'Octane';
      }
    );

    // Update speed labels
    this.speedLabelManager.setPlayerTeams(this.actorManager.playerTeams);
    const ballActor = this.actorManager.actors['ball'];
    const ballEntity = this.player?.getBall();
    const ballVelocity = ballEntity?.velocity
      ? new THREE.Vector3(ballEntity.velocity.x, ballEntity.velocity.y, ballEntity.velocity.z)
      : null;

    // Collect player velocities
    const playerVelocities = {};
    Object.keys(this.actorManager.playerNameToCarActorId).forEach((playerName) => {
      const playerEntity = this.player?.getPlayer(playerName);
      if (playerEntity?.velocity) {
        playerVelocities[playerName] = new THREE.Vector3(
          playerEntity.velocity.x,
          playerEntity.velocity.y,
          playerEntity.velocity.z
        );
      }
    });

    this.speedLabelManager.update(
      this.actorManager.actors,
      ballActor,
      ballVelocity,
      this.actorManager.playerNameToCarActorId,
      playerVelocities
    );
  }

  resetScene() {
    this.actorManager.reset();
    this.effectsManager.reset();
    this.nameTagManager.reset();
    this.hitboxManager.reset();
    this.speedLabelManager.reset();
    // UI reset handled by React state usually, but we might need to signal reset?
  }

  updateCamera(delta) {

    // Update FOV (applies to all camera modes)
    // Rocket League uses HORIZONTAL FOV, Three.js uses VERTICAL FOV
    // Convert: FOV_vertical = 2 * atan(tan(FOV_horizontal / 2) / aspectRatio)
    //
    // NOTE: On ultra-wide screens (32:9), pure conversion would give a very small vertical FOV
    // (e.g., 110° horizontal -> ~44° vertical), which cuts off the car.
    // Rocket League appears to use a minimum vertical FOV to prevent this.
    // We use the 16:9 baseline as reference (what the user expects from their FOV setting).
    if (this.cameraSettings.fov) {
      const horizontalFovRad = (this.cameraSettings.fov * Math.PI) / 180;
      const aspectRatio = this.sceneManager.camera.aspect;

      // Calculate what the vertical FOV would be at 16:9 (baseline aspect ratio)
      // This is what players expect when they set their FOV
      const baselineAspect = 16 / 9;
      const baselineVerticalFovRad = 2 * Math.atan(Math.tan(horizontalFovRad / 2) / baselineAspect);

      // Calculate vertical FOV for current aspect ratio
      const calculatedVerticalFovRad = 2 * Math.atan(Math.tan(horizontalFovRad / 2) / aspectRatio);

      // Use the larger of the two - this ensures ultra-wide screens don't cut off content
      // On 16:9, both values are equal. On wider screens, baseline wins. On taller screens, calculated wins.
      const verticalFovRad = Math.max(baselineVerticalFovRad, calculatedVerticalFovRad);
      const verticalFovDeg = (verticalFovRad * 180) / Math.PI;

      if (Math.abs(this.sceneManager.camera.fov - verticalFovDeg) > 0.1) {
        this.sceneManager.camera.fov = verticalFovDeg;
        this.sceneManager.camera.updateProjectionMatrix();
      }
    }

    if (this.cameraMode === 'free') {
      // Free camera handled by CameraManager
      // Update freeCam speed from settings
      if (this.cameraSettings.freeCamSpeed) {
        this.cameraManager.freeCamSpeed = this.cameraSettings.freeCamSpeed;
      }
      this.cameraManager.update(delta);
    } else if (this.cameraMode === 'ballOrbit') {
      // Ball orbit mode - keep ball target updated and update camera
      const ball = this.actorManager.actors[this.actorManager.ballActorId];
      if (ball) {
        this.cameraManager.setTargetBall(ball);
      }
      this.cameraManager.update(delta);
    } else if (this.cameraMode === 'player') {
      // Live mode: use selectedPlayerIndex
      if (this.isLiveMode && this.selectedPlayerIndex !== undefined) {
        const carKey = `live_car_${this.selectedPlayerIndex}`;
        const car = this.actorManager.actors[carKey];
        const ball = this.actorManager.actors['live-ball'];

        if (car) {
          this.cameraManager.setTargetCar(car);
          if (ball) {
            this.cameraManager.setTargetBall(ball);
          }
          this.cameraManager.setFollowSettings(this.cameraSettings);
          // Use isBallCam from car's userData (set in updateCarsLive)
          const isBallCam = car.userData?.isBallCam ?? true;
          this.cameraManager.update(delta, isBallCam);
        } else {
          this.cameraManager.update(delta);
        }
      }
      // Replay mode: use selectedPlayerName
      else if (this.selectedPlayerName) {
        // Find car by searching all actors with matching playerId
        let car = null;
        for (const actor of Object.values(this.actorManager.actors)) {
          if (
            actor.userData &&
            actor.userData.playerId === this.selectedPlayerName
          ) {
            const loc = actor.userData.location;
            const hasValidPosition =
              loc && (loc.x !== 0 || loc.y !== 0 || loc.z !== 0);
            if (hasValidPosition) {
              car = actor;
              break;
            } else if (!car) {
              car = actor;
            }
          }
        }
        const ball = this.actorManager.actors[this.actorManager.ballActorId];

        if (car) {
          // Set targets for camera manager
          this.cameraManager.setTargetCar(car);
          this.cameraManager.setTargetBall(ball);

          // Update follow settings from cameraSettings (matching RL camera options)
          this.cameraManager.setFollowSettings(this.cameraSettings);

          // Get ball cam state from framework's PlayerEntity
          const playerEntity = this.player.getPlayer(this.selectedPlayerName);
          const isBallCam = playerEntity?.isBallCam ?? true;

          // Debug logging on camera change
          if (!this._lastCameraState) this._lastCameraState = {};
          const stateKey = this.selectedPlayerName;
          if (this._lastCameraState[stateKey] !== isBallCam) {
            console.log(
              `[Camera CHANGE] t=${this.currentTime.toFixed(2)}s, ${
                this.selectedPlayerName
              }: ${isBallCam ? '🎯 BALL CAM' : '🚗 CAR CAM'}`
            );
            this._lastCameraState[stateKey] = isBallCam;
          }

          // Update camera with smooth transitions
          this.cameraManager.update(delta, isBallCam);
        } else {
          // No car found, just update camera
          this.cameraManager.update(delta);
        }
      }
    }
  }

  handleEventClick(event, cameraMode) {
    // Seek 3 seconds before event to show build-up (consistent for goals, saves, and demos)
    const seekTime = Math.max(0, event.time - 3);
    this.seek(seekTime);

    let playerName = null;
    if (event.type === 'goal' || event.type === 'demo') {
      playerName = event.player;
    } else if (event.type === 'save') {
      // For saves, we only have the team. Find the player from that team.
      if (event.team !== undefined && this.actorManager.playerTeams) {
        for (const [name, team] of Object.entries(
          this.actorManager.playerTeams
        )) {
          if (team === event.team) {
            playerName = name;
            break;
          }
        }
      }
    }

    console.log(
      `Event: ${event.type}, player = ${playerName}, mode = ${cameraMode} `
    );

    if (
      cameraMode === 'player' &&
      playerName &&
      this.actorManager.playerNames.has(playerName)
    ) {
      // Wait for car to have valid position before selecting
      const trySelectPlayer = (attempt = 1) => {
        // Find car with matching playerId AND valid position
        // (there may be multiple cars with same playerId due to ID changes)
        let carActorId = null;
        let bestCar = null;

        for (const [id, actor] of Object.entries(this.actorManager.actors)) {
          if (actor.userData && actor.userData.playerId === playerName) {
            const loc = actor.userData.location;
            const hasValidPosition =
              loc && (loc.x !== 0 || loc.y !== 0 || loc.z !== 0);

            if (hasValidPosition) {
              carActorId = id;
              bestCar = actor;
              break; // Found a car with valid position, use it!
            } else if (!bestCar) {
              // Keep track of car without valid position as fallback
              carActorId = id;
              bestCar = actor;
            }
          }
        }

        if (!carActorId) {
          if (attempt < 10) {
            setTimeout(() => trySelectPlayer(attempt + 1), attempt * 100);
          } else {
            console.warn(
              `❌ No car actor found for ${playerName} after ${attempt} attempts`
            );
          }
          return;
        }

        const loc = bestCar.userData.location;
        const hasValidPosition =
          loc && (loc.x !== 0 || loc.y !== 0 || loc.z !== 0);

        if (hasValidPosition) {
          console.log(
            `✅ Selected ${playerName} (ID: ${carActorId}) at position: `,
            loc
          );
          this.selectPlayer(playerName);
          if (this.callbacks.onPlayerSelect)
            this.callbacks.onPlayerSelect(playerName);
        } else if (attempt < 10) {
          console.log(
            `⏳ Waiting for ${playerName} position(attempt ${attempt} / 10)...`
          );
          setTimeout(() => trySelectPlayer(attempt + 1), attempt * 100);
        } else {
          console.warn(
            `❌ Car for ${playerName} never got valid position after ${attempt} attempts`
          );
        }
      };
      setTimeout(() => trySelectPlayer(1), 100);
    }
    // For Free Cam, we just seek 3 seconds before but don't change camera position
  }

  /**
   * Setup terrain interaction for pings and drawing
   * @private
   */
  _setupTerrainInteraction() {
    const canvas = this.sceneManager.renderer.domElement;
    const mouse = new THREE.Vector2();

    // Helper to get world position and normal from mouse event
    // Returns { position: {x,y,z}, normal: {x,y,z} } or null
    const getTerrainPosition = (event) => {
      const rect = canvas.getBoundingClientRect();
      mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      this.raycaster.setFromCamera(mouse, this.sceneManager.camera);

      // Try raycasting against simplified drawing collider first (performant)
      if (this.useArenaRaycast && this.arenaManager) {
        const colliderMeshes = this.arenaManager.getDrawingColliderMeshes();
        if (colliderMeshes && colliderMeshes.length > 0) {
          const intersects = this.raycaster.intersectObjects(colliderMeshes, false);
          if (intersects.length > 0) {
            const point = intersects[0].point.clone();
            // Get world normal from face
            const faceNormal = intersects[0].face?.normal;
            let worldNormal = { x: 0, y: 1, z: 0 }; // Default up
            if (faceNormal) {
              const wn = faceNormal.clone().transformDirection(intersects[0].object.matrixWorld).normalize();
              worldNormal = { x: wn.x, y: wn.y, z: wn.z };
              // Offset point slightly along normal to prevent z-fighting
              point.add(wn.clone().multiplyScalar(5));
            }
            return {
              position: { x: point.x, y: point.y, z: point.z },
              normal: worldNormal,
            };
          }
        }
      }

      // Fallback: intersect with ground plane
      const intersection = new THREE.Vector3();
      const ray = this.raycaster.ray;
      const hit = ray.intersectPlane(this.groundPlane, intersection);

      if (hit) {
        // Clamp to reasonable arena bounds (in Unreal units)
        const ARENA_X = 4100; // Half-width
        const ARENA_Z = 6000; // Half-length (goal to goal)
        intersection.x = Math.max(-ARENA_X, Math.min(ARENA_X, intersection.x));
        intersection.z = Math.max(-ARENA_Z, Math.min(ARENA_Z, intersection.z));
        return {
          position: { x: intersection.x, y: 0, z: intersection.z },
          normal: { x: 0, y: 1, z: 0 }, // Up for ground plane
        };
      }
      return null;
    };

    // Mouse down handler
    const onMouseDown = (event) => {
      // Ignore if not left click
      if (event.button !== 0) return;

      // Ignore if in freecam with pointer lock
      if (document.pointerLockElement === canvas) return;

      // Ignore if tool is select
      console.log('[GameEngine] Mouse down, activeTool:', this.activeTool, 'onTerrainClick:', !!this.onTerrainClick);
      if (this.activeTool === 'select') return;

      const terrainHit = getTerrainPosition(event);
      if (!terrainHit) {
        console.log('[GameEngine] No terrain position found');
        return;
      }
      const { position, normal } = terrainHit;
      console.log('[GameEngine] Terrain position:', position, 'normal:', normal);

      if (this.activeTool === 'ping') {
        // Trigger ping callback with position and normal
        if (this.onTerrainClick) {
          this.onTerrainClick(position, 'ping', { normal });
        }
      } else if (this.activeTool === 'draw') {
        // Start drawing
        this.isDrawing = true;
        this.currentStrokeId = crypto.randomUUID();
        if (this.onTerrainClick) {
          this.onTerrainClick(position, 'draw-start', {
            strokeId: this.currentStrokeId,
            color: this.drawColor,
            thickness: this.drawThickness,
          });
        }
        // Also start local stroke for immediate feedback
        if (this.drawingManager) {
          this.drawingManager.startStroke(
            this.currentStrokeId,
            '__self__', // Will be replaced by actual ID
            this.drawColor,
            this.drawThickness,
            position
          );
        }
      } else if (this.activeTool === 'eraser') {
        // Check for stroke intersection
        if (this.onTerrainClick) {
          this.onTerrainClick(position, 'erase');
        }
      }
    };

    // Mouse move handler (for drawing)
    const onMouseMove = (event) => {
      if (!this.isDrawing || this.activeTool !== 'draw') return;

      const terrainHit = getTerrainPosition(event);
      if (!terrainHit) return;

      const { position } = terrainHit;

      // Add point to local stroke
      if (this.drawingManager && this.currentStrokeId) {
        this.drawingManager.addPoints(this.currentStrokeId, [position]);
      }

      // Trigger drag callback
      if (this.onTerrainDrag) {
        this.onTerrainDrag(position, 'draw-point', {
          strokeId: this.currentStrokeId,
        });
      }
    };

    // Mouse up handler
    const onMouseUp = (event) => {
      if (!this.isDrawing) return;

      if (this.activeTool === 'draw' && this.currentStrokeId) {
        // End drawing
        if (this.onTerrainClick) {
          this.onTerrainClick(null, 'draw-end', {
            strokeId: this.currentStrokeId,
          });
        }
        // Complete local stroke
        if (this.drawingManager) {
          this.drawingManager.completeStroke(this.currentStrokeId);
        }
      }

      this.isDrawing = false;
      this.currentStrokeId = null;
    };

    // Add event listeners
    canvas.addEventListener('mousedown', onMouseDown);
    canvas.addEventListener('mousemove', onMouseMove);
    canvas.addEventListener('mouseup', onMouseUp);
    canvas.addEventListener('mouseleave', onMouseUp);

    // Store for cleanup
    this._terrainInteractionHandlers = { onMouseDown, onMouseMove, onMouseUp };
  }

  /**
   * Set the active tool mode
   * @param {'select' | 'ping' | 'draw' | 'eraser'} tool
   */
  setActiveTool(tool) {
    this.activeTool = tool;
    console.log(`[GameEngine] Active tool set to: ${tool}`);
  }

  /**
   * Set draw color and thickness
   * @param {string} color - Hex color string (#RRGGBB)
   * @param {number} thickness - Line thickness (1-10)
   */
  setDrawSettings(color, thickness) {
    this.drawColor = color;
    this.drawThickness = thickness;
  }

  /**
   * Set callback for terrain clicks
   * @param {Function} callback - (position, type, data) => void
   */
  setTerrainClickCallback(callback) {
    this.onTerrainClick = callback;
  }

  /**
   * Set callback for terrain drags (drawing)
   * @param {Function} callback - (position, type, data) => void
   */
  setTerrainDragCallback(callback) {
    this.onTerrainDrag = callback;
  }

  /**
   * Get stroke IDs near a point (for eraser tool)
   * @param {{ x: number, y: number, z: number }} point - Position to check
   * @param {number} radius - Detection radius
   * @returns {string[]} Array of stroke IDs
   */
  getStrokesNearPoint(point, radius = 50) {
    if (!this.drawingManager) return [];
    const vec3 = new THREE.Vector3(point.x, point.y, point.z);
    return this.drawingManager.getStrokesNearPoint(vec3, radius);
  }

  /**
   * DEBUG: Trigger a test explosion at the center of the arena
   * Call from console: gameEngine.testExplosion()
   */
  testExplosion() {
    const position = new THREE.Vector3(0, 1, 0);
    const rotation = new THREE.Quaternion();
    console.log('[GameEngine] Triggering test explosion at', position);
    this.effectsManager.triggerDemoExplosion(position, rotation, 1);
  }

  // ============================================
  // LIVE MODE METHODS (027-live-viewer)
  // ============================================

  /**
   * Initialize GameEngine for live streaming mode
   * This skips replay loading and prepares for direct state updates
   */
  async initLiveMode() {
    console.log('[GameEngine] Initializing live mode...');

    try {
      this.isLiveMode = true;
      this.isPlaying = true; // Live is always "playing"

      // Pre-compile explosion materials
      precompileExplosionMaterials(this.sceneManager.scene, this.sceneManager.renderer);
      warmupExplosionPool(this.sceneManager.scene, this.sceneManager.renderer, this.sceneManager.camera);

      // Load arena meshes (field, walls, goals)
      console.log('[GameEngine] Loading arena for live mode...');
      await this.arenaManager.loadArenaMeshes();
      console.log('[GameEngine] Arena meshes loaded');

      // Load arena decoration (stands, stadium surroundings)
      console.log('[GameEngine] Loading arena decoration...');
      await this.arenaManager.loadArenaDecor(true);
      console.log('[GameEngine] Arena decoration loaded');

      // Preload models for live mode (ball + default Octane car)
      console.log('[GameEngine] Preloading models for live mode...');
      await Promise.all([
        this.actorManager.waitForBallModel(),
        this.actorManager.carModelLoader.loadModel('octane'), // Preload Octane as default car
      ]);
      console.log('[GameEngine] Models preloaded for live mode');

      // Initialize boost pad meshes storage (will be created on first snapshot with boost pad data)
      this.boostPadMeshes = new Map();

      // Initialize live mode car storage
      this.liveCars = new Map(); // index -> { mesh, carState }
      this.liveBall = null;
      this.lastLiveCarStates = []; // For demo detection

      // Initialize live mode player data (for nametags)
      this.livePlayerBoosts = {}; // playerName -> boost amount
      this.livePlayerTeams = {}; // playerName -> team (0 or 1)

      // Set camera to freecam by default
      console.log('[GameEngine] Setting up camera...');
      this.setCameraMode('free');
      this.cameraManager.setDefaultFreecamPosition();

      console.log('[GameEngine] Live mode initialized successfully');

      if (this.callbacks.onReady) {
        this.callbacks.onReady();
      }
    } catch (error) {
      console.error('[GameEngine] Error in initLiveMode:', error);
      throw error;
    }
  }

  /**
   * Update ball state for live mode
   * @param {Object} ballState - { position, velocity, rotation, angularVelocity, lastTouchTeam }
   */
  updateBallLive(ballState) {
    if (!ballState) {
      console.log('[GameEngine] updateBallLive: ballState is null/undefined');
      return;
    }

    // Create ball mesh if it doesn't exist
    if (!this.actorManager.actors['live-ball']) {
      console.log('[GameEngine] Creating live ball mesh');
      const ballMesh = this.actorManager.createBallMeshForLive();
      this.actorManager.ballActorId = 'live-ball';
      this.actorManager.actors['live-ball'] = ballMesh;
      console.log('[GameEngine] Live ball created:', ballMesh);
    }

    // Always get ball from actorManager.actors
    const ball = this.actorManager.actors['live-ball'];
    if (!ball) return;

    // Update position
    ball.position.set(ballState.position.x, ballState.position.y, ballState.position.z);
    ball.quaternion.set(
      ballState.rotation.x,
      ballState.rotation.y,
      ballState.rotation.z,
      ballState.rotation.w
    );

    // Update userData
    ball.userData.location.copy(ball.position);
    ball.userData.rotation.copy(ball.quaternion);
    ball.userData.velocity.set(ballState.velocity.x, ballState.velocity.y, ballState.velocity.z);
    if (ballState.angularVelocity) {
      ball.userData.angularVelocity.set(
        ballState.angularVelocity.x,
        ballState.angularVelocity.y,
        ballState.angularVelocity.z
      );
    }
    ball.visible = true;

    // Update ball trail with lastTouchTeam
    const team = ballState.lastTouchTeam ?? this.actorManager.lastBallTouchTeam ?? 0;
    if (ballState.lastTouchTeam !== null && ballState.lastTouchTeam !== undefined) {
      this.actorManager.lastBallTouchTeam = ballState.lastTouchTeam;
    }

    this.effectsManager.updateBallTrail(ball.position, ball.userData.velocity, team);

    // Update ball indicator
    if (this.actorManager.ballIndicator) {
      this.actorManager.ballIndicator.position.set(ball.position.x, 2, ball.position.z);
      this.actorManager.ballIndicator.visible = true;
    }
  }

  /**
   * Update all cars for live mode
   * @param {Array} carStates - Array of car states with position, rotation, boost, etc.
   */
  updateCarsLive(carStates) {
    if (!carStates || !Array.isArray(carStates)) return;

    const currentCarIds = new Set();

    carStates.forEach((carState, index) => {
      // v11: Use playerId as stable key instead of index (order in array may change)
      const playerId = carState.playerId ?? index;
      const carKey = `live_car_${playerId}`;
      currentCarIds.add(carKey);

      let carData = this.liveCars.get(playerId);

      // Create car if it doesn't exist
      if (!carData) {
        // Use ActorManager's live mode method which handles all setup
        // Pass bodyId directly for correct car model loading (e.g., 23=Octane, 403=Fennec)
        // v11: Use playerId as stable identifier instead of index
        const initialMesh = this.actorManager.createCarMeshForLive(
          carState.team,
          playerId,
          carState.name,
          carState.bodyId || null
        );
        initialMesh.userData.playerId = playerId;

        carData = { lastState: null };
        this.liveCars.set(playerId, carData);

        console.log(`[GameEngine] Created live car playerId=${playerId}: ${carState.name} (Team ${carState.team}, bodyId=${carState.bodyId})`);
      }

      // Always get mesh from actorManager.actors - it may have been replaced by FBX model
      const mesh = this.actorManager.actors[carKey];
      if (!mesh) return;

      // Update position and rotation
      mesh.position.set(carState.position.x, carState.position.y, carState.position.z);
      mesh.quaternion.set(
        carState.rotation.x,
        carState.rotation.y,
        carState.rotation.z,
        carState.rotation.w
      );

      // Update userData
      mesh.userData.location.copy(mesh.position);
      mesh.userData.rotation.copy(mesh.quaternion);
      mesh.userData.velocity.set(carState.velocity.x, carState.velocity.y, carState.velocity.z);
      mesh.userData.team = carState.team;
      mesh.userData.isBallCam = carState.isBallCam ?? true; // Default to ball cam if not provided
      mesh.userData.steer = carState.steer ?? 0; // Steering input for wheel animation

      // Update boost particles - IMPORTANT: check both isBoosting AND boost > 0
      const shouldShowBoostParticles = carState.isBoosting && carState.boost > 0;
      this.effectsManager.updateBoostTrail(
        carKey,
        shouldShowBoostParticles,
        mesh.position,
        mesh.quaternion,
        mesh.userData.velocity
      );

      // Update supersonic trail
      this.effectsManager.updateSupersonicTrail(
        carKey,
        carState.isSupersonic,
        mesh.position,
        mesh.quaternion,
        mesh.userData.velocity,
        carState.team
      );

      // Store last state for demo detection
      carData.lastState = { ...carState };

      // Store boost and team for nametags
      this.livePlayerBoosts[carState.name] = carState.boost ?? 0;
      this.livePlayerTeams[carState.name] = carState.team;
    });

    // Update nameTagManager with player teams
    if (this.nameTagManager && Object.keys(this.livePlayerTeams).length > 0) {
      this.nameTagManager.setPlayerTeams(this.livePlayerTeams);
    }

    // Clean up cars that are no longer in the snapshot
    for (const [playerId, carData] of this.liveCars.entries()) {
      const carKey = `live_car_${playerId}`;
      if (!currentCarIds.has(carKey)) {
        // Car is no longer in snapshot - remove it
        const mesh = this.actorManager.actors[carKey];
        if (mesh) {
          // Stop any active effects for this car
          this.effectsManager.updateBoostTrail(carKey, false, mesh.position, mesh.quaternion, mesh.userData?.velocity);
          this.effectsManager.updateSupersonicTrail(carKey, false, mesh.position, mesh.quaternion, mesh.userData?.velocity, mesh.userData?.team);

          // Remove from scene
          this.sceneManager.scene.remove(mesh);

          // Dispose geometry and materials
          if (mesh.geometry) mesh.geometry.dispose();
          if (mesh.material) {
            if (Array.isArray(mesh.material)) {
              mesh.material.forEach(m => m.dispose());
            } else {
              mesh.material.dispose();
            }
          }

          // Remove from actorManager
          delete this.actorManager.actors[carKey];

          // Remove from nameTagManager mapping
          if (carData.lastState?.name) {
            delete this.actorManager.playerNameToCarActorId[carData.lastState.name];
            delete this.livePlayerBoosts[carData.lastState.name];
            delete this.livePlayerTeams[carData.lastState.name];
          }
        }

        // Remove from liveCars map
        this.liveCars.delete(playerId);
        console.log(`[GameEngine] Removed live car playerId=${playerId} (no longer in snapshot)`);
      }
    }

    // Store for demo detection comparison
    this.lastLiveCarStates = carStates;
  }

  /**
   * Update boost pad states for live mode
   * Creates meshes on first call if they don't exist (using v7+ position data)
   * @param {Array} boostPadStates - Array of { id, position, isBig, isAvailable, respawnTimer }
   */
  updateBoostPadsLive(boostPadStates) {
    if (!boostPadStates || !this.boostPadMeshes) return;

    // Debug log once
    if (!this._boostPadDebugLogged && boostPadStates.length > 0) {
      console.log(`[GameEngine] updateBoostPadsLive called with ${boostPadStates.length} pads`);
      console.log(`[GameEngine] First pad:`, boostPadStates[0]);
      this._boostPadDebugLogged = true;
    }

    boostPadStates.forEach(pad => {
      let mesh = this.boostPadMeshes.get(pad.id);

      // Create mesh if it doesn't exist (first snapshot with boost pad data)
      if (!mesh && pad.position) {
        console.log(`[GameEngine] Creating boost pad mesh for id=${pad.id}, isBig=${pad.isBig}, pos=(${pad.position.x}, ${pad.position.y}, ${pad.position.z})`);
        mesh = this._createBoostPadMeshLive(pad);
        this.boostPadMeshes.set(pad.id, mesh);
      }

      if (!mesh) return;

      // Update visibility based on availability
      const isAvailable = pad.isAvailable;

      // Update main pad visibility
      mesh.visible = isAvailable;

      // Update glow effect if exists
      if (mesh.userData.glowMesh) {
        mesh.userData.glowMesh.visible = isAvailable;
      }

      // Update indicator if exists
      if (mesh.userData.indicator) {
        mesh.userData.indicator.visible = isAvailable;
      }
    });
  }

  /**
   * Create a boost pad mesh for live mode
   * @param {Object} pad - { id, position, isBig, isAvailable }
   * @returns {THREE.Mesh} The boost pad mesh
   */
  _createBoostPadMeshLive(pad) {
    const isBig = pad.isBig;
    let geometry, material, mesh;

    if (isBig) {
      // Big pads: Glowing sphere
      const radius = 50;
      geometry = new THREE.SphereGeometry(radius, 16, 16);
      material = new THREE.MeshStandardMaterial({
        color: 0xffdd44,
        emissive: 0xffaa00,
        emissiveIntensity: 1.0,
        metalness: 0.3,
        roughness: 0.2,
        transparent: true,
        opacity: 1.0,
        depthWrite: false,
      });
      mesh = new THREE.Mesh(geometry, material);
      mesh.renderOrder = 100;

      // Add glow effect
      const glowGeometry = new THREE.SphereGeometry(radius * 2.0, 16, 16);
      const glowMaterial = new THREE.MeshBasicMaterial({
        color: 0xffaa00,
        transparent: true,
        opacity: 0.3,
        blending: THREE.AdditiveBlending,
        side: THREE.BackSide,
        depthWrite: false,
      });
      const glowMesh = new THREE.Mesh(glowGeometry, glowMaterial);
      glowMesh.renderOrder = 99;
      mesh.add(glowMesh);
      mesh.userData.glowMesh = glowMesh;

      // Add inner glow
      const innerGlowGeometry = new THREE.SphereGeometry(radius * 1.4, 16, 16);
      const innerGlowMaterial = new THREE.MeshBasicMaterial({
        color: 0xffcc00,
        transparent: true,
        opacity: 0.4,
        blending: THREE.AdditiveBlending,
        side: THREE.BackSide,
        depthWrite: false,
      });
      const innerGlowMesh = new THREE.Mesh(innerGlowGeometry, innerGlowMaterial);
      innerGlowMesh.renderOrder = 99;
      mesh.add(innerGlowMesh);
      mesh.userData.innerGlowMesh = innerGlowMesh;

      mesh.userData.needsLight = true;
    } else {
      // Small pads: Flat cylinder
      const radius = 40;
      const height = 5;
      geometry = new THREE.CylinderGeometry(radius, radius, height, 16);
      material = new THREE.MeshStandardMaterial({
        color: 0xffcc00,
        emissive: 0xff9900,
        emissiveIntensity: 0.4,
        metalness: 0.2,
        roughness: 0.8,
        transparent: true,
        opacity: 1.0,
        depthWrite: false,
      });
      mesh = new THREE.Mesh(geometry, material);
      mesh.renderOrder = 100;
    }

    // Position the boost pad
    // Live mode sends position in UE coordinates, convert to Three.js (swap Y/Z)
    const groundLevel = 10;
    const floatHeight = isBig ? 150 : groundLevel;

    mesh.position.set(
      pad.position.x,
      floatHeight,
      pad.position.z  // position.z is already converted from UE Y in proto.ts
    );

    // Store metadata
    mesh.userData.padId = pad.id;
    mesh.userData.isBig = isBig;
    mesh.userData.isAvailable = pad.isAvailable;

    this.sceneManager.scene.add(mesh);

    // Add point light for big pads
    if (mesh.userData.needsLight) {
      const light = new THREE.PointLight(0xffaa00, 1.0, 600);
      light.decay = 0;
      light.position.set(pad.position.x, floatHeight - 50, pad.position.z);
      this.sceneManager.scene.add(light);
      mesh.userData.light = light;
    }

    return mesh;
  }

  /**
   * Trigger goal explosion at ball position
   * @param {Object} position - { x, y, z } position of the explosion
   * @param {number} team - 0 for blue, 1 for orange
   */
  triggerGoalExplosion(position, team = 0) {
    if (!position) return;

    const pos = new THREE.Vector3(position.x, position.y, position.z);
    console.log(`[GameEngine] Goal explosion at`, pos, `team: ${team}`);

    // Use the same explosion as replay mode
    this.effectsManager.triggerGoalExplosion(pos, team);

    // Temporarily hide ball
    if (this.liveBall) {
      this.liveBall.visible = false;
      setTimeout(() => {
        if (this.liveBall) this.liveBall.visible = true;
      }, 1500);
    }
  }

  /**
   * Trigger demolition explosion at car position
   * @param {Object} position - { x, y, z }
   * @param {Object} rotation - { x, y, z, w } quaternion
   * @param {number} attackerTeam - Team of the attacker (0 or 1)
   */
  triggerDemoExplosion(position, rotation, attackerTeam = 0) {
    if (!position) return;

    const pos = new THREE.Vector3(position.x, position.y, position.z);
    const rot = new THREE.Quaternion(rotation.x, rotation.y, rotation.z, rotation.w);

    console.log(`[GameEngine] Demo explosion at`, pos);
    this.effectsManager.triggerDemoExplosion(pos, rot, attackerTeam);
  }

  /**
   * Set camera target for live mode
   * @param {string} mode - 'free', 'ball', or 'player'
   * @param {number} playerIndex - Player index for player cam
   */
  setCameraTarget(mode, playerId = 0) {
    if (!this.isLiveMode) {
      // For replay mode, use existing behavior
      return;
    }

    switch (mode) {
      case 'free':
        this.setCameraMode('free');
        this.selectedPlayerIndex = undefined;
        break;
      case 'ball':
        // Use ballOrbit mode for ball camera (same as replay mode)
        this.setCameraMode('ballOrbit');
        this.selectedPlayerIndex = undefined;
        break;
      case 'player':
        // Use player mode (which is 'player' in GameEngine, 'car' in CameraManager)
        this.setCameraMode('player');
        // v11: Store playerId instead of index (actor keys are now live_car_${playerId})
        this.selectedPlayerIndex = playerId;
        // Set initial targets - updateCamera will keep them updated every frame
        const carKey = `live_car_${playerId}`;
        const carMesh = this.actorManager.actors[carKey];
        const ball = this.actorManager.actors['live-ball'];
        if (carMesh) {
          this.cameraManager.setTargetCar(carMesh);
        }
        if (ball) {
          this.cameraManager.setTargetBall(ball);
        }
        break;
    }
  }

  /**
   * Render frame for live mode
   * Should be called every frame after updating state
   */
  renderLive() {
    if (!this.isLiveMode) return;

    const delta = this.clock.getDelta();

    // Update effects (particles, trails, etc.)
    this.effectsManager.update(delta);

    // Update camera
    this.inputManager.update(delta);
    this.updateCamera(delta);

    // Update wheel rotations
    this.actorManager.updateWheelRotations();

    // Update nametags (player labels above cars)
    if (this.nameTagManager && this.livePlayerBoosts) {
      // Determine followed player (to hide their nametag in player cam)
      let followedPlayer = null;
      if (this.cameraMode === 'player' && this.selectedPlayerIndex !== undefined) {
        // v11: Find player by playerId (selectedPlayerIndex is now playerId, not array index)
        const playerState = this.lastLiveCarStates?.find(car => car.playerId === this.selectedPlayerIndex);
        if (playerState) {
          followedPlayer = playerState.name;
        }
      }

      this.nameTagManager.update(
        this.actorManager.actors,
        this.livePlayerBoosts,
        this.actorManager.playerNameToCarActorId,
        followedPlayer
      );
    }

    // Render scene
    this.sceneManager.render();
  }

  /**
   * Get car mesh by player index (for live mode)
   * @param {number} index - Player index
   * @returns {THREE.Object3D|null}
   */
  getLiveCarMesh(index) {
    const carData = this.liveCars?.get(index);
    return carData?.mesh || null;
  }

  /**
   * Get ball mesh for live mode
   * @returns {THREE.Object3D|null}
   */
  getLiveBallMesh() {
    return this.liveBall;
  }
}
