/**
 * ReplayPlayer — the bare playback core of @rlrml/player.
 *
 * Deliberately minimal, like `@rlrml/player`'s ReplayPlayer: it owns the
 * renderer (scene / arena / actors), a playback clock, and the plugin host —
 * nothing else. Scoreboard, name tags, overlays, effects polish, custom
 * cameras: all plugins (docs/player/EXTENSIBILITY.md).
 *
 * Data flows one way each frame:
 *   adapter.seek(t) → ActorManager interpolates meshes → plugins beforeRender →
 *   renderer.render.
 */
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { SceneManager } from "./managers/SceneManager.js";
import { ArenaManager } from "./managers/ArenaManager.js";
import { ActorManager } from "./managers/ActorManager.js";
import { EffectsManager } from "./managers/EffectsManager.js";
import { HitboxManager } from "./managers/HitboxManager.js";
import type { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
import { createBoostPadsPlugin } from "./plugins/boost-pads.js";
// Timeline projection / skip-window semantics are @rlrml/player's own
// ReplayModel utilities, so both players agree on what gets skipped and how
// replay time maps onto the (skip-aware) timeline.
import {
  computeTimelineSegments,
  getKickoffCountdownMetadata,
  getReplayPlaybackEndTime,
  inferKickoffGameState,
  inferLiveGameState,
  projectReplayTimeToTimeline,
  projectTimelineTimeToReplay,
} from "../player-internals/timeline";
import { getKickoffSkipTargetTime, getPostGoalTransitionSkipTargetTime } from "../player-helpers";
import {
  DEFAULT_ENVIRONMENT_ID,
  resolveEnvironment,
  type PlayerEnvironmentSpec,
} from "./environments.js";
import type {
  ReplayModel,
  ReplayPlayerTimelineProjection,
  ReplayPlayerTimelineSegment,
} from "../types";
import type { ReplayScene } from "../scene";
import type { CameraPlugin } from "./plugins/camera.js";
import type {
  BallRenderState,
  BeforeRenderCallback,
  CameraSettings,
  CarRenderState,
  FrameRenderInfo,
  PlayerCameraViewMode,
  PlayerFreeCameraPreset,
  PlayerOptions,
  PlayerPlugin,
  PlayerPluginContext,
  PlayerPluginDefinition,
  PlayerPluginStateContext,
  PlayerRenderContext,
  PlayerSnapshot,
  PlayerState,
  PlayerStatePatch,
} from "./types.js";

type PlayerListener = (state: PlayerState) => void;
type InstalledPlugin = { definition: PlayerPluginDefinition; plugin: PlayerPlugin };
type FreeCameraTransition = {
  position: THREE.Vector3;
  target: THREE.Vector3;
  up: THREE.Vector3;
  fov: number;
};
type FreeCameraPresetOptions = {
  instant?: boolean;
};

// With `effects: false`, every EffectsManager call from ActorManager is a no-op.
const effectsStub = new Proxy({}, { get: () => () => {} });

/**
 * Drop non-finite fields, matching @rlrml/player's normalizeCustomCameraSettings.
 * (`pitch` passes through — the camera plugin maps it onto `angle` on entry.)
 */
function normalizeCustomCameraSettings(
  settings: CameraSettings | null | undefined,
): CameraSettings | null {
  if (!settings) return null;
  const normalized: CameraSettings = {};
  for (const key of Object.keys(settings) as Array<keyof CameraSettings>) {
    const value = settings[key];
    if (typeof value === "number" && Number.isFinite(value)) {
      normalized[key] = value;
    }
  }
  return normalized;
}

const FREE_CAMERA_FOV = 48;
const FREE_CAMERA_TRANSITION_SMOOTHING = 0.14;
const FREE_CAMERA_POSITION_EPSILON_SQ = 16;
const FREE_CAMERA_TARGET_EPSILON_SQ = 16;
const FREE_CAMERA_UP_EPSILON_RAD = 0.003;
const FREE_CAMERA_FOV_EPSILON = 0.05;
const FREE_CAMERA_FIT_MARGIN = 1.08;
const SOCCAR_HALF_X_UU = 4120;
const SOCCAR_HALF_Y_UU = 5140;
const SOCCAR_CAMERA_FIT_MIN_Y_UU = 0;
const SOCCAR_CAMERA_FIT_MAX_Y_UU = 2200;
const OVERHEAD_TARGET = new THREE.Vector3(0, 700, 0);
const OVERHEAD_UP = new THREE.Vector3(-1, 0, 0);
const OVERHEAD_FORWARD = new THREE.Vector3(0, -1, 0);
const SIDE_TARGET = new THREE.Vector3(0, 900, 0);
const SIDE_UP = new THREE.Vector3(0, 1, 0);
const SIDE_FORWARD = new THREE.Vector3(9600, -5500, 12600).normalize();

function getFreeCameraPreset(preset: PlayerFreeCameraPreset, aspect: number): FreeCameraTransition {
  const fitAspect = Number.isFinite(aspect) && aspect > 0 ? aspect : 16 / 9;
  const target = preset === "overhead" ? OVERHEAD_TARGET.clone() : SIDE_TARGET.clone();
  const up = preset === "overhead" ? OVERHEAD_UP.clone() : SIDE_UP.clone();
  const forward = preset === "overhead" ? OVERHEAD_FORWARD.clone() : SIDE_FORWARD.clone();
  const distance = getFreeCameraFitDistance({
    aspect: fitAspect,
    fov: FREE_CAMERA_FOV,
    forward,
    margin: FREE_CAMERA_FIT_MARGIN,
    target,
    up,
  });

  return {
    position: target.clone().addScaledVector(forward, -distance),
    target,
    up,
    fov: FREE_CAMERA_FOV,
  };
}

function getFreeCameraFitDistance(options: {
  aspect: number;
  fov: number;
  forward: THREE.Vector3;
  margin: number;
  target: THREE.Vector3;
  up: THREE.Vector3;
}): number {
  const { aspect, fov, forward, margin, target, up } = options;
  const cameraForward = forward.clone().normalize();
  const right = new THREE.Vector3().crossVectors(cameraForward, up).normalize();
  const cameraUp = new THREE.Vector3().crossVectors(right, cameraForward).normalize();
  const tanVertical = Math.tan(THREE.MathUtils.degToRad(fov) / 2);
  const tanHorizontal = tanVertical * aspect;
  let requiredDistance = 1;

  for (const x of [-SOCCAR_HALF_X_UU, SOCCAR_HALF_X_UU]) {
    for (const y of [SOCCAR_CAMERA_FIT_MIN_Y_UU, SOCCAR_CAMERA_FIT_MAX_Y_UU]) {
      for (const z of [-SOCCAR_HALF_Y_UU, SOCCAR_HALF_Y_UU]) {
        const relative = new THREE.Vector3(x, y, z).sub(target);
        const horizontal = Math.abs(relative.dot(right));
        const vertical = Math.abs(relative.dot(cameraUp));
        const forwardOffset = relative.dot(cameraForward);
        requiredDistance = Math.max(
          requiredDistance,
          horizontal / tanHorizontal - forwardOffset,
          vertical / tanVertical - forwardOffset,
        );
      }
    }
  }

  return Math.max(1, requiredDistance * margin);
}

/**
 * Build the player's `replayRoot`: a group whose LOCAL space is raw Unreal
 * coordinates (RL Z-up, UU). Its fixed basis is exactly `vec3RlToThree`
 * (adapter/coords.ts): x→x, z→y, y→z — so `replayRoot.add(mesh)` with
 * UE-coordinate positions renders correctly in this Y-up world.
 *
 * This matches @rlrml/player's `replayRoot` convention (there it's a
 * `(-fieldScale, fieldScale, fieldScale)` scale in a Z-up world): in BOTH
 * players, replayRoot-local space = chirality-corrected UE coordinates, which
 * is what makes ReplayScene-consuming overlays portable.
 */
function createReplayRoot(scene: THREE.Scene): THREE.Group {
  const replayRoot = new THREE.Group();
  replayRoot.name = "replayRoot";
  replayRoot.matrixAutoUpdate = false;
  // prettier-ignore
  replayRoot.matrix.set(
    1, 0, 0, 0,
    0, 0, 1, 0,
    0, 1, 0, 0,
    0, 0, 0, 1,
  );
  scene.add(replayRoot);
  return replayRoot;
}

export class ReplayPlayer extends EventTarget {
  readonly container: HTMLElement;
  /** The subtr-actor adapter — the sole data source (timelines + live entities). */
  adapter: SubtrActorPlayer;
  /**
   * @rlrml/player's normalized `ReplayModel` over the same raw WASM output the
   * adapter consumes (docs/player/PLAYER_PARITY.md Phase 2) — the data surface
   * @rlrml/player consumers read. Shares the adapter's time axis (t=0 at the
   * first frame) and player-id format. Null when constructed directly with an
   * adapter only; `createPlayer()` always provides it.
   */
  replay: ReplayModel | null;
  readonly options: PlayerOptions;

  readonly sceneManager: any;
  readonly arenaManager: any;
  readonly actorManager: any;
  readonly effectsManager: any;
  readonly hitboxManager: any;
  readonly controls: OrbitControls;
  /**
   * UE-coordinate mount point: children positioned in raw Unreal coords (RL
   * Z-up, UU) render correctly here — same convention as @rlrml/player's
   * `replayRoot`, see createReplayRoot. The portable seam for 3D overlays.
   */
  readonly replayRoot: THREE.Group;
  /**
   * @rlrml/player's `ReplayScene` surface (docs/player/PLAYER_PARITY.md Phase 3+):
   * what `ReplayPlayer.sceneState`-reading consumers (js/stat-evaluation-player
   * stat modules) use to mount THREE overlays. `scene`/`camera`/`renderer`/
   * `controls`/`replayRoot`/`resize` are real; `ballMesh`/`playerMeshes` are
   * live views onto this renderer's actors; the player-renderer internals
   * (body meshes, hitboxes, boost trails/meters, demo indicators) are empty
   * maps — they have no counterpart here.
   */
  readonly sceneState: ReplayScene;
  private readonly effectsEnabled: boolean;
  /** Resolves when async assets (arena meshes, ball model) are in the scene. */
  ready: Promise<void>;

  private readonly plugins: InstalledPlugin[] = [];
  private readonly beforeRenderCallbacks: BeforeRenderCallback[] = [];
  private resizeObserver: ResizeObserver | null = null;
  private animationFrameId: number | null = null;
  private disposed = false;
  private playing = false;
  private speed: number;
  private loop: boolean;
  private currentTime = 0;
  private lastTickAt: number | null = null;
  private freeCameraTransition: FreeCameraTransition | null = null;

  // ── @rlrml/player-parity state (docs/player/PLAYER_PARITY.md). Camera fields are
  //    delegated to an installed camera plugin (id "camera") when present; the
  //    display toggles are tracked-but-inert until their rendering lands.
  private cameraDistanceScaleValue: number;
  private customCameraSettingsValue: CameraSettings | null;
  private cameraViewModeValue: PlayerCameraViewMode;
  private attachedPlayerIdValue: string | null;
  /** null = never set: follow the camera plugin's recorded-state behavior. */
  private ballCamEnabledValue: boolean | null;
  private boostMeterEnabledValue: boolean;
  private boostPickupAnimationEnabledValue: boolean;
  private hitboxWireframesEnabledValue: boolean;
  private hitboxOnlyModeEnabledValue: boolean;
  /** Lazily built player-name → hitbox-family map (roster is static). */
  private hitboxTypeByName: Map<string, string> | null = null;
  /** True while hitbox wireframes are showing (cheap per-frame early-out). */
  private hitboxesActive = false;
  private skipPostGoalTransitionsEnabledValue: boolean;
  private skipKickoffsEnabledValue: boolean;
  /** True once view-mode/attachment was set through the parity surface. */
  private attachmentTouched = false;

  // ── Timeline projection / skip windows (require a ReplayModel) ──────────────
  private liveGameState: number | null = null;
  private kickoffGameState: number | null = null;
  private timelineSegmentsCacheKey: string | null = null;
  private timelineSegmentsCache: ReplayPlayerTimelineSegment[] = [];

  constructor(
    container: HTMLElement,
    adapter: SubtrActorPlayer,
    options: PlayerOptions = {},
    replay: ReplayModel | null = null,
  ) {
    super();
    this.container = container;
    this.adapter = adapter;
    this.replay = replay;
    this.options = options;
    this.updateReplayGameStates();
    this.speed = Math.max(0.1, options.initialPlaybackRate ?? options.speed ?? 1);
    this.loop = options.loop ?? false;
    this.cameraDistanceScaleValue = Math.max(0.25, options.initialCameraDistanceScale ?? 1);
    this.customCameraSettingsValue = normalizeCustomCameraSettings(
      options.initialCustomCameraSettings,
    );
    this.attachedPlayerIdValue = options.initialAttachedPlayerId ?? null;
    this.cameraViewModeValue =
      options.initialCameraViewMode ?? (this.attachedPlayerIdValue ? "follow" : "free");
    this.ballCamEnabledValue = options.initialBallCamEnabled ?? null;
    this.boostMeterEnabledValue = options.initialBoostMeterEnabled ?? false;
    this.boostPickupAnimationEnabledValue = options.initialBoostPickupAnimationEnabled ?? true;
    this.hitboxWireframesEnabledValue = options.initialHitboxWireframesEnabled ?? false;
    this.hitboxOnlyModeEnabledValue = options.initialHitboxOnlyModeEnabled ?? false;
    this.skipPostGoalTransitionsEnabledValue =
      options.initialSkipPostGoalTransitionsEnabled ?? true;
    this.skipKickoffsEnabledValue = options.initialSkipKickoffsEnabled ?? false;

    this.sceneManager = new SceneManager(container, {
      assetBase: options.assetBase,
      preserveDrawingBuffer: options.preserveDrawingBuffer,
    });
    // Neutral IBL renders instantly so playback starts immediately; the HDR
    // environment (default "space") loads lazily and swaps in once decoded.
    this.sceneManager.initDefaultEnvironment();
    this.applyEnvironmentSpec(options.environment ?? DEFAULT_ENVIRONMENT_ID);
    this.arenaManager = new ArenaManager(this.scene, { assetBase: options.assetBase });
    // Trails (boost / supersonic / ball) plus the team-colored goal explosion,
    // which fires from ActorManager when playback reaches each goal (see the
    // setGoalEvents() feed below).
    this.effectsEnabled = options.effects ?? true;
    this.effectsManager = this.effectsEnabled ? new EffectsManager(this.scene) : effectsStub;
    this.actorManager = new ActorManager(this.scene, this.effectsManager, {
      assetBase: options.assetBase,
    });
    if (options.motionInterpolation) {
      this.setMotionInterpolation(options.motionInterpolation);
    }
    this.actorManager.initFromFramework(adapter);
    this.actorManager.initInterpolants(adapter.getTimelines());
    // Hitbox wireframes (driven by the hitboxWireframesEnabled /
    // hitboxOnlyModeEnabled parity toggles; updated per frame in render()).
    this.hitboxManager = new HitboxManager(this.scene);
    this.syncGoalEvents();
    // setRenderContext() pre-warms the explosion shader pools — a multi-second,
    // main-thread-blocking compile. Deferred behind `ready` (below) so it
    // finishes under the load overlay, well before the first goal, instead of
    // hitching on the first explosion.

    // Default camera: simple orbit. The full follow/ballcam path is a later
    // bring-up; plugins can also drive `camera` directly.
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    // Default zoomSpeed (1) feels glacial at field scale — viewing distances are
    // thousands of UU, so each wheel notch barely moves the camera.
    this.controls.zoomSpeed = 2.5;
    this.camera.position.set(0, 4000, 6000);
    this.controls.target.set(0, 200, 0);
    this.controls.update();

    this.replayRoot = createReplayRoot(this.scene);
    this.sceneState = this.createSceneState();

    this.ready = Promise.all([
      this.arenaManager.loadArenaMeshes().catch((e: unknown) => {
        console.warn("[player] arena load failed", e);
      }),
      this.prepareReplayAssets(),
    ]).then(() => undefined);

    this.installResizeHandling();
    for (const definition of options.plugins ?? []) {
      this.installPlugin(definition, false);
    }
    if (!this.plugins.some((entry) => entry.plugin.id === "boost-pads")) {
      this.installPlugin(createBoostPadsPlugin(), false);
    }
    this.applyInitialCameraOptions();
    // @rlrml/player semantics: don't start inside a skipped window (t=0 is a
    // kickoff, so skip-kickoffs jumps straight to live play).
    this.skipPostGoalTransitionIfNeeded();
    this.skipPastKickoffIfNeeded();
    this.scheduleAnimationFrame();
    this.emitChange();

    if (options.autoplay) {
      this.play();
    }
  }

  get scene(): THREE.Scene {
    return this.sceneManager.scene as THREE.Scene;
  }
  get camera(): THREE.PerspectiveCamera {
    return this.sceneManager.camera as THREE.PerspectiveCamera;
  }
  get renderer(): THREE.WebGLRenderer {
    return this.sceneManager.renderer as THREE.WebGLRenderer;
  }
  get duration(): number {
    return this.adapter.duration;
  }

  /**
   * Replace the replay data feeding this player without replacing the renderer,
   * canvas, arena, camera controls, or player instance. This is the replay-boundary
   * path for playlist/review UIs: replay-specific actors, effects, timelines,
   * and plugin setup are refreshed against the new adapter while the static
   * render shell remains mounted.
   */
  async replaceReplay(
    adapter: SubtrActorPlayer,
    replay: ReplayModel | null,
    options: { currentTime?: number; preservePlayback?: boolean } = {},
  ): Promise<void> {
    if (this.disposed) {
      throw new Error("Cannot replace replay on a disposed ReplayPlayer");
    }

    const shouldResume = options.preservePlayback ?? this.playing;
    if (this.playing) {
      this.setPlayingInternal(false);
    }

    this.teardownPlugins();
    this.effectsManager.reset();
    this.effectsManager.clearEvents?.();
    this.hitboxManager.reset();
    this.actorManager.reset();

    this.adapter = adapter;
    this.replay = replay;
    this.updateReplayGameStates();
    this.timelineSegmentsCacheKey = null;
    this.timelineSegmentsCache = [];
    this.hitboxTypeByName = null;
    this.hitboxesActive = false;
    this.freeCameraTransition = null;

    this.actorManager.initFromFramework(adapter);
    this.actorManager.initInterpolants(adapter.getTimelines());
    this.syncGoalEvents();

    const attachedPlayerId =
      this.attachedPlayerIdValue &&
      this.adapter.playerList.some((player) => player.id === this.attachedPlayerIdValue)
        ? this.attachedPlayerIdValue
        : null;
    if (attachedPlayerId !== this.attachedPlayerIdValue) {
      this.attachedPlayerIdValue = attachedPlayerId;
      if (this.cameraViewModeValue === "follow") {
        this.cameraViewModeValue = "free";
      }
    }

    this.seekInternal(options.currentTime ?? 0);
    this.ready = this.prepareReplayAssets();
    await this.ready;

    this.setupPlugins();
    this.applyInitialCameraOptions();
    this.skipPostGoalTransitionIfNeeded();
    this.skipPastKickoffIfNeeded();
    if (shouldResume) {
      this.setPlayingInternal(true);
    }
    this.render();
    this.emitChange();
  }

  // ── Environment (skybox + IBL) ──────────────────────────────────────────────
  /**
   * Switch the skybox environment at runtime. Accepts a built-in id (e.g.
   * `"space"`), a full `PlayerEnvironment` descriptor, or `false` for the
   * neutral default (no skybox). Non-blocking: the HDR swaps in when decoded.
   */
  setEnvironment(spec: PlayerEnvironmentSpec): void {
    this.applyEnvironmentSpec(spec);
  }

  private applyEnvironmentSpec(spec: PlayerEnvironmentSpec): void {
    const env = resolveEnvironment(spec);
    if (!env) {
      this.sceneManager.setDefaultBackground();
      return;
    }
    // Fire-and-forget: do NOT await or fold into `this.ready`, so playback never
    // waits on the HDR download.
    void this.sceneManager.applyEnvironment(env).catch((e: unknown) => {
      console.warn(`[player] environment "${env.id}" failed to load`, e);
    });
  }

  // ── Playback control ────────────────────────────────────────────────────────
  play(): void {
    if (this.playing) return;
    this.setPlayingInternal(true);
    this.emitChange();
  }

  pause(): void {
    if (!this.playing) return;
    this.setPlayingInternal(false);
    this.emitChange();
  }

  togglePlayback(): void {
    if (this.playing) this.pause();
    else this.play();
  }

  seek(time: number): void {
    this.seekInternal(time);
    if (this.playing) {
      // Never land playback inside a skipped window (@rlrml/player semantics).
      this.skipPostGoalTransitionIfNeeded();
      this.skipPastKickoffIfNeeded();
    }
    this.emitChange();
  }

  setPlaybackRate(speed: number): void {
    this.speed = Math.max(0.1, speed);
    this.emitChange();
  }

  setLoop(loop: boolean): void {
    this.loop = loop;
  }

  /**
   * Switch position interpolation between replay samples (see
   * PlayerOptions.motionInterpolation). Takes effect on the next rendered
   * frame — handy for A/B-ing smoothness live.
   */
  setMotionInterpolation(method: "hermite" | "linear"): void {
    this.actorManager.interpolationMethod = method === "linear" ? "lerp" : "hermite";
  }

  // ── Frame stepping (@rlrml/player parity, off the adapter's frame timeline) ──
  setFrameIndex(frameIndex: number): void {
    const times = this.adapter.frameTimes;
    if (times.length === 0 || !Number.isFinite(frameIndex)) return;
    const clamped = Math.min(Math.max(Math.trunc(frameIndex), 0), times.length - 1);
    // @rlrml/player semantics: landing on an exact frame implies paused playback.
    if (this.playing) this.setPlayingInternal(false);
    this.seekInternal(times[clamped]);
    this.emitChange();
  }

  stepFrames(delta: number): void {
    if (!Number.isFinite(delta)) return;
    this.setFrameIndex(this.adapter.frameIndexAt(this.currentTime) + Math.trunc(delta));
  }

  stepForwardFrame(): void {
    this.stepFrames(1);
  }

  stepBackwardFrame(): void {
    this.stepFrames(-1);
  }

  // ── Camera controls (@rlrml/player parity) ──────────────────────────────────
  // All delegate to an installed camera plugin (id "camera") when present —
  // state is tracked either way, so a plugin added later picks it up.
  setCameraDistanceScale(scale: number): void {
    this.cameraDistanceScaleValue = Math.max(0.25, scale);
    this.getCameraPlugin()?.setDistanceScale(this.cameraDistanceScaleValue);
    this.emitChange();
  }

  setCustomCameraSettings(settings: CameraSettings | null): void {
    this.applyCustomCameraSettings(settings);
    this.emitChange();
  }

  setAttachedPlayer(playerId: string | null): void {
    this.attachedPlayerIdValue = playerId;
    this.cameraViewModeValue = playerId ? "follow" : "free";
    this.attachmentTouched = true;
    this.freeCameraTransition = null;
    this.syncCameraAttachment();
    this.emitChange();
  }

  setCameraViewMode(mode: PlayerCameraViewMode): void {
    this.cameraViewModeValue = mode;
    this.attachmentTouched = true;
    this.freeCameraTransition = null;
    this.syncCameraAttachment();
    this.emitChange();
  }

  setFreeCameraPreset(preset: PlayerFreeCameraPreset, options: FreeCameraPresetOptions = {}): void {
    this.cameraViewModeValue = "free";
    this.attachmentTouched = true;
    this.syncCameraAttachment(); // leave follow mode if we were in it
    const transition = getFreeCameraPreset(preset, this.camera.aspect);
    if (options.instant) {
      this.camera.position.copy(transition.position);
      this.controls.target.copy(transition.target);
      this.camera.up.copy(transition.up).normalize();
      this.camera.fov = transition.fov;
      this.camera.updateProjectionMatrix();
      this.camera.lookAt(transition.target);
      this.controls.enabled = true;
      this.freeCameraTransition = null;
    } else {
      this.freeCameraTransition = transition;
    }
    this.emitChange();
  }

  /**
   * Force ball cam (`true`) or car cam (`false`), or pass `null` to follow the
   * attached player's recorded ball-cam toggle ("player" view — the default).
   */
  setBallCamEnabled(enabled: boolean | null): void {
    this.ballCamEnabledValue = enabled;
    this.getCameraPlugin()?.setBallCam(enabled);
    this.emitChange();
  }

  // ── Display toggles (@rlrml/player parity). Hitbox toggles drive
  //    HitboxManager (see updateHitboxVisualization); the pickup-animation
  //    toggle is read by the bridged plugin; the boost meter is still inert. ──
  setBoostMeterEnabled(enabled: boolean): void {
    this.boostMeterEnabledValue = enabled;
    this.emitChange();
  }

  setBoostPickupAnimationEnabled(enabled: boolean): void {
    this.boostPickupAnimationEnabledValue = enabled;
    this.emitChange();
  }

  setHitboxWireframesEnabled(enabled: boolean): void {
    this.hitboxWireframesEnabledValue = enabled;
    this.emitChange();
  }

  setHitboxOnlyModeEnabled(enabled: boolean): void {
    this.hitboxOnlyModeEnabledValue = enabled;
    this.emitChange();
  }

  // ── Skip windows (@rlrml/player parity; live when a ReplayModel is present) ──
  setSkipPostGoalTransitionsEnabled(enabled: boolean): void {
    this.skipPostGoalTransitionsEnabledValue = enabled;
    if (enabled && this.playing) {
      this.skipPostGoalTransitionIfNeeded();
    }
    this.emitChange();
  }

  setSkipKickoffsEnabled(enabled: boolean): void {
    this.skipKickoffsEnabledValue = enabled;
    if (enabled && this.playing) {
      this.skipPostGoalTransitionIfNeeded();
      this.skipPastKickoffIfNeeded();
    }
    this.emitChange();
  }

  // ── State surface (@rlrml/player parity) ────────────────────────────────────
  setState(patch: PlayerStatePatch): void {
    if (patch.speed !== undefined) {
      this.speed = Math.max(0.1, patch.speed);
    }
    if (patch.cameraDistanceScale !== undefined) {
      this.cameraDistanceScaleValue = Math.max(0.25, patch.cameraDistanceScale);
      this.getCameraPlugin()?.setDistanceScale(this.cameraDistanceScaleValue);
    }
    if (patch.customCameraSettings !== undefined) {
      this.applyCustomCameraSettings(patch.customCameraSettings);
    }
    if (patch.cameraViewMode !== undefined) {
      this.cameraViewModeValue = patch.cameraViewMode;
      this.attachmentTouched = true;
    }
    if (patch.attachedPlayerId !== undefined) {
      this.attachedPlayerIdValue = patch.attachedPlayerId;
      this.attachmentTouched = true;
      if (patch.cameraViewMode === undefined) {
        this.cameraViewModeValue = patch.attachedPlayerId ? "follow" : "free";
      }
    }
    if (patch.cameraViewMode !== undefined || patch.attachedPlayerId !== undefined) {
      this.freeCameraTransition = null;
      this.syncCameraAttachment();
    }
    // "player" view wins when requested (mirrors @rlrml/player: useReplayBallCam
    // overrides the manual ballCamEnabled flag). null = follow recorded state.
    if (patch.useReplayBallCam === true) {
      this.ballCamEnabledValue = null;
      this.getCameraPlugin()?.setBallCam(null);
    } else if (patch.ballCamEnabled !== undefined) {
      this.ballCamEnabledValue = patch.ballCamEnabled;
      this.getCameraPlugin()?.setBallCam(patch.ballCamEnabled);
    }
    if (patch.boostMeterEnabled !== undefined) {
      this.boostMeterEnabledValue = patch.boostMeterEnabled;
    }
    if (patch.boostPickupAnimationEnabled !== undefined) {
      this.boostPickupAnimationEnabledValue = patch.boostPickupAnimationEnabled;
    }
    if (patch.hitboxWireframesEnabled !== undefined) {
      this.hitboxWireframesEnabledValue = patch.hitboxWireframesEnabled;
    }
    if (patch.hitboxOnlyModeEnabled !== undefined) {
      this.hitboxOnlyModeEnabledValue = patch.hitboxOnlyModeEnabled;
    }
    if (patch.skipPostGoalTransitionsEnabled !== undefined) {
      this.skipPostGoalTransitionsEnabledValue = patch.skipPostGoalTransitionsEnabled;
    }
    if (patch.skipKickoffsEnabled !== undefined) {
      this.skipKickoffsEnabledValue = patch.skipKickoffsEnabled;
    }
    if (patch.currentTime !== undefined) {
      this.seekInternal(patch.currentTime);
    }
    if (patch.playing !== undefined && patch.playing !== this.playing) {
      this.setPlayingInternal(patch.playing);
    }
    if (this.playing && (patch.currentTime !== undefined || patch.playing !== undefined)) {
      this.skipPostGoalTransitionIfNeeded();
      this.skipPastKickoffIfNeeded();
    }
    this.emitChange();
  }

  getState(): PlayerState {
    const frameIndex = this.adapter.frameIndexAt(this.currentTime);
    const camera = this.getCameraPlugin();
    let cameraViewMode = this.cameraViewModeValue;
    let attachedPlayerId = this.attachedPlayerIdValue;
    if (camera) {
      // Derive the camera fields from the plugin so state stays truthful even
      // when a consumer drives the plugin handle directly (e.g. the dev UI).
      if (camera.getMode() === "follow") {
        cameraViewMode = "follow";
        const targetName = camera.getTarget();
        const info = targetName
          ? this.adapter.playerList.find((p) => p.name === targetName)
          : undefined;
        attachedPlayerId = info?.id ?? attachedPlayerId;
      } else {
        cameraViewMode = "free";
        attachedPlayerId = null;
      }
    }
    return {
      currentTime: this.currentTime,
      duration: this.duration,
      frameIndex,
      // Kickoff countdowns, like @rlrml/player. The adapter's frame timeline is
      // the ReplayModel's (same metadata frames, same time axis), so its index
      // is valid against the model.
      activeMetadata: this.replay
        ? getKickoffCountdownMetadata(this.replay, frameIndex, this.currentTime)
        : null,
      playing: this.playing,
      speed: this.speed,
      cameraDistanceScale: this.cameraDistanceScaleValue,
      customCameraSettings: this.customCameraSettingsValue,
      cameraViewMode,
      attachedPlayerId,
      ballCamEnabled: camera ? camera.getBallCam() : (this.ballCamEnabledValue ?? false),
      // null override = follow the player's recorded ball-cam toggle.
      useReplayBallCam: this.ballCamEnabledValue === null,
      effectiveBallCamEnabled: camera ? camera.getBallCam() : (this.ballCamEnabledValue ?? false),
      boostMeterEnabled: this.boostMeterEnabledValue,
      boostPickupAnimationEnabled: this.boostPickupAnimationEnabledValue,
      hitboxWireframesEnabled: this.hitboxWireframesEnabledValue,
      hitboxOnlyModeEnabled: this.hitboxOnlyModeEnabledValue,
      skipPostGoalTransitionsEnabled: this.skipPostGoalTransitionsEnabledValue,
      skipKickoffsEnabled: this.skipKickoffsEnabledValue,
    };
  }

  getSnapshot(): PlayerSnapshot {
    return this.getState();
  }

  // ── Timeline projection (@rlrml/player parity) ──────────────────────────────
  // Maps replay time onto the skip-aware timeline (and back). Without a
  // ReplayModel there are no segments, so every projection is the identity.
  getTimelineDuration(): number {
    return this.replay?.duration ?? this.duration;
  }

  getTimelineCurrentTime(): number {
    return this.projectReplayTimeToTimeline(this.currentTime).timelineTime;
  }

  getTimelineSegments(): ReplayPlayerTimelineSegment[] {
    if (!this.replay) return [];
    const cacheKey = `${this.skipPostGoalTransitionsEnabledValue}:${this.skipKickoffsEnabledValue}`;
    if (this.timelineSegmentsCacheKey === cacheKey) {
      return this.timelineSegmentsCache;
    }
    this.timelineSegmentsCacheKey = cacheKey;
    this.timelineSegmentsCache = computeTimelineSegments(
      this.replay,
      this.skipPostGoalTransitionsEnabledValue,
      this.skipKickoffsEnabledValue,
      this.liveGameState,
      this.kickoffGameState,
    );
    return this.timelineSegmentsCache;
  }

  projectReplayTimeToTimeline(replayTime: number): ReplayPlayerTimelineProjection {
    return projectReplayTimeToTimeline(
      this.replay?.duration ?? this.duration,
      this.getTimelineSegments(),
      replayTime,
    );
  }

  projectTimelineTimeToReplay(timelineTime: number): number {
    return projectTimelineTimeToReplay(
      this.replay?.duration ?? this.duration,
      this.getTimelineDuration(),
      this.getTimelineSegments(),
      timelineTime,
    );
  }

  subscribe(listener: PlayerListener): () => void {
    const handleChange = (event: Event): void => {
      listener((event as CustomEvent<PlayerState>).detail);
    };
    this.addEventListener("change", handleChange);
    listener(this.getState());
    return () => {
      this.removeEventListener("change", handleChange);
    };
  }

  /** Per-render frame-timing callback (@rlrml/player parity). Returns a remover. */
  onBeforeRender(callback: BeforeRenderCallback): () => void {
    this.beforeRenderCallbacks.push(callback);
    return () => {
      const index = this.beforeRenderCallbacks.indexOf(callback);
      if (index >= 0) {
        this.beforeRenderCallbacks.splice(index, 1);
      }
    };
  }

  // ── Plugin host (mirrors @rlrml/player) ────────────────────────────────────
  addPlugin(definition: PlayerPluginDefinition): () => void {
    return this.installPlugin(definition, true);
  }

  removePlugin(id: string): boolean {
    const index = this.plugins.findIndex((entry) => entry.plugin.id === id);
    if (index < 0) return false;
    const [entry] = this.plugins.splice(index, 1);
    entry.plugin.teardown?.(this.createPluginContext());
    return true;
  }

  getPlugins(): PlayerPlugin[] {
    return this.plugins.map((entry) => entry.plugin);
  }

  destroy(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.playing = false;
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
    this.resizeObserver?.disconnect();
    this.resizeObserver = null;
    this.beforeRenderCallbacks.length = 0;
    while (this.plugins.length > 0) {
      const entry = this.plugins.pop();
      entry?.plugin.teardown?.(this.createPluginContext());
    }
    this.controls.dispose();
    if (this.effectsEnabled) {
      this.effectsManager.reset();
    }
    this.hitboxManager.dispose();
    this.actorManager.reset();
    this.sceneManager.dispose();
  }

  dispose(): void {
    this.destroy();
  }

  // ── Internals ───────────────────────────────────────────────────────────────
  private setPlayingInternal(playing: boolean): void {
    this.playing = playing;
    this.lastTickAt = null;
    if (playing) {
      this.actorManager.resumeAnimations();
    } else {
      this.actorManager.pauseAnimations();
    }
  }

  private prepareReplayAssets(): Promise<void> {
    return this.actorManager
      .waitForBallModel()
      .catch(() => false)
      .then(() => {
        if (this.effectsEnabled) {
          try {
            this.effectsManager.setRenderContext(this.renderer, this.camera);
          } catch (e) {
            console.warn("[player] explosion warmup failed", e);
          }
        }
      });
  }

  private updateReplayGameStates(): void {
    if (!this.replay) {
      this.liveGameState = null;
      this.kickoffGameState = null;
      return;
    }

    this.liveGameState = inferLiveGameState(this.replay);
    this.kickoffGameState = inferKickoffGameState(this.replay, this.liveGameState);
  }

  private syncGoalEvents(): void {
    if (!this.effectsEnabled) return;
    this.effectsManager.clearEvents?.();
    if (!this.replay) return;
    this.effectsManager.setGoalEvents(
      this.replay.timelineEvents
        .filter((event) => event.kind === "goal")
        .map((event) => ({
          frame: event.frame,
          time: event.time,
          team: event.isTeamZero ? 0 : 1,
          playerName: event.playerName ?? "",
        })),
    );
  }

  private teardownPlugins(): void {
    const context = this.createPluginContext();
    for (const entry of this.plugins) {
      entry.plugin.teardown?.(context);
    }
  }

  private setupPlugins(): void {
    for (const entry of this.plugins) {
      entry.plugin.setup?.(this.createPluginContext());
      if (entry.plugin.id === "camera") {
        this.pushCameraParityState();
      }
      entry.plugin.onStateChange?.(this.createPluginStateContext(this.getState()));
    }
  }

  private seekInternal(time: number): void {
    this.currentTime = THREE.MathUtils.clamp(time, 0, this.duration);
    // Sync the THREE animation system (if active) to the new time, and reset
    // trackers that work off frame-to-frame deltas so they don't see the jump:
    // the ball trail would draw a segment connecting old/new positions and the
    // wheels would spin wildly from the position delta.
    this.actorManager.seekAnimations(this.currentTime);
    this.effectsManager.resetBallTrail();
    this.actorManager.resetWheelTracking();
  }

  /** Where playback stops: the last segment's start if skips run to the end. */
  private getPlaybackEndTime(): number {
    return this.replay
      ? getReplayPlaybackEndTime(this.replay.duration, this.getTimelineSegments())
      : this.duration;
  }

  /**
   * Jump past a kickoff countdown when skip-kickoffs is on (@rlrml/player
   * semantics). A skip is a jump, so it routes through seekInternal — that
   * resets the delta-based trackers (ball trail, wheel spin) which must not
   * see it. Returns true when a skip happened.
   */
  private skipPastKickoffIfNeeded(): boolean {
    if (!this.replay || !this.skipKickoffsEnabledValue) return false;
    const targetTime = getKickoffSkipTargetTime(
      this.replay,
      this.currentTime,
      this.liveGameState,
      this.kickoffGameState,
    );
    if (targetTime === null) return false;
    this.seekInternal(targetTime);
    return true;
  }

  /** Same as skipPastKickoffIfNeeded, for post-goal replay/celebration windows. */
  private skipPostGoalTransitionIfNeeded(): boolean {
    if (!this.replay || !this.skipPostGoalTransitionsEnabledValue) return false;
    const targetTime = getPostGoalTransitionSkipTargetTime(
      this.replay,
      this.currentTime,
      this.liveGameState,
      this.kickoffGameState,
    );
    if (targetTime === null) return false;
    this.seekInternal(targetTime);
    return true;
  }

  /** The installed camera plugin, when one is present (duck-typed by id). */
  private getCameraPlugin(): CameraPlugin | null {
    const plugin = this.plugins.find((entry) => entry.plugin.id === "camera")?.plugin;
    return plugin && typeof (plugin as CameraPlugin).follow === "function"
      ? (plugin as CameraPlugin)
      : null;
  }

  private playerNameForId(id: string): string | null {
    return this.adapter.playerList.find((p) => p.id === id)?.name ?? null;
  }

  /**
   * When ball cam has not been manually overridden (`ballCamEnabledValue ===
   * null`), drive the follow camera from the attached player's replay ball-cam
   * state (resolved per-frame onto `entity.isBallCam` by the adapter), matching
   * the core @rlrml/player behavior. A manual `setBallCamEnabled` takes over.
   */
  private applyReplayBallCam(): void {
    if (this.ballCamEnabledValue !== null) return;
    if (this.cameraViewModeValue !== "follow" || !this.attachedPlayerIdValue) return;
    const name = this.playerNameForId(this.attachedPlayerIdValue);
    if (!name) return;
    const entity = this.adapter.getAllPlayers().find((player) => player.name === name);
    if (!entity) return;
    this.getCameraPlugin()?.setBallCam(entity.isBallCam);
  }

  /** Push the parity view-mode/attachment onto the camera plugin. */
  private syncCameraAttachment(): void {
    const camera = this.getCameraPlugin();
    if (!camera) return;
    if (this.cameraViewModeValue === "follow" && this.attachedPlayerIdValue) {
      const name = this.playerNameForId(this.attachedPlayerIdValue);
      if (!name) {
        console.warn(`[player] no player with id ${JSON.stringify(this.attachedPlayerIdValue)}`);
        return;
      }
      // Follow mode owns the camera; make sure a preset's custom up is undone.
      this.camera.up.set(0, 1, 0);
      camera.follow(name);
      return;
    }
    // "free": only leave follow mode — never stomp the player-native free-fly /
    // ballOrbit modes a consumer may have set on the plugin handle directly.
    if (camera.getMode() === "follow") {
      camera.release();
    }
  }

  private applyCustomCameraSettings(settings: CameraSettings | null | undefined): void {
    this.customCameraSettingsValue = normalizeCustomCameraSettings(settings);
    const camera = this.getCameraPlugin();
    if (camera) {
      // Replace, not merge: @rlrml/player treats customCameraSettings as a
      // whole object, so clear the plugin's overrides before applying.
      camera.setCameraSettings(null);
      if (this.customCameraSettingsValue) {
        camera.setCameraSettings(this.customCameraSettingsValue);
      }
    }
  }

  /** Push explicitly-set parity camera state onto an (newly) installed plugin. */
  private pushCameraParityState(): void {
    const camera = this.getCameraPlugin();
    if (!camera) return;
    if (this.cameraDistanceScaleValue !== 1) {
      camera.setDistanceScale(this.cameraDistanceScaleValue);
    }
    if (this.customCameraSettingsValue) {
      camera.setCameraSettings(this.customCameraSettingsValue);
    }
    if (this.ballCamEnabledValue !== null) {
      camera.setBallCam(this.ballCamEnabledValue);
    }
    if (this.attachmentTouched) {
      this.syncCameraAttachment();
    }
  }

  private applyInitialCameraOptions(): void {
    const o = this.options;
    if (o.initialAttachedPlayerId !== undefined || o.initialCameraViewMode !== undefined) {
      // Only then may the parity state override the plugin's own follow/mode
      // options (e.g. createCameraPlugin({ follow })).
      this.attachmentTouched = true;
    }
    this.pushCameraParityState();
  }

  private computeFrameRenderInfo(): FrameRenderInfo {
    const times = this.adapter.frameTimes;
    const frameIndex = this.adapter.frameIndexAt(this.currentTime);
    const nextFrameIndex = Math.min(frameIndex + 1, Math.max(times.length - 1, 0));
    const t0 = times[frameIndex] ?? 0;
    const t1 = times[nextFrameIndex] ?? t0;
    const alpha = t1 > t0 ? THREE.MathUtils.clamp((this.currentTime - t0) / (t1 - t0), 0, 1) : 0;
    return { frameIndex, nextFrameIndex, alpha, currentTime: this.currentTime };
  }

  private installResizeHandling(): void {
    if (typeof ResizeObserver === "undefined") return; // SceneManager's window listener covers it
    this.resizeObserver = new ResizeObserver(() => this.sceneManager.onWindowResize());
    this.resizeObserver.observe(this.container);
  }

  private scheduleAnimationFrame(): void {
    if (this.animationFrameId !== null || this.disposed) return;
    this.animationFrameId = requestAnimationFrame(this.tick);
  }

  private tick = (now: number): void => {
    this.animationFrameId = null;
    if (this.disposed) return;

    let timeChanged = false;
    let dt = 0;
    if (this.playing) {
      dt = this.lastTickAt === null ? 0 : Math.min(0.1, (now - this.lastTickAt) / 1000);
      this.lastTickAt = now;
      let next = this.currentTime + dt * this.speed;
      // Skip-aware end: when trailing windows are skipped, playback ends at
      // the final segment boundary instead of the raw duration (@rlrml/player).
      const end = this.getPlaybackEndTime();
      if (next >= end) {
        if (this.loop) {
          next = 0;
          // Wrapping is a seek: clear delta-based trackers (see seek()).
          this.actorManager.seekAnimations(0);
          this.effectsManager.resetBallTrail();
          this.actorManager.resetWheelTracking();
        } else {
          next = end;
          this.playing = false;
        }
      }
      timeChanged = next !== this.currentTime || !this.playing;
      this.currentTime = next;
      if (this.playing) {
        timeChanged = this.skipPostGoalTransitionIfNeeded() || timeChanged;
        timeChanged = this.skipPastKickoffIfNeeded() || timeChanged;
      }
    }

    this.render(dt);
    if (timeChanged) {
      this.emitChange();
    }
    this.scheduleAnimationFrame();
  };

  renderFrame(dt = 0): void {
    this.adapter.seek(this.currentTime);
    // Original GameEngine frame order: advance the THREE animation system (when
    // active it owns positions) BEFORE updateFromFramework applies entity state.
    if (this.playing) {
      this.actorManager.updateAnimations(dt * this.speed);
    }
    this.actorManager.updateFromFramework(this.adapter, this.currentTime);
    this.updatePlayerStates();
    this.applyReplayBallCam();
    this.updateHitboxVisualization();
    this.effectsManager.update(dt, this.playing, this.speed);
    if (this.playing) {
      // Wheel spin works off position deltas (not time), steering off userData.steer.
      this.actorManager.updateWheelRotations();
    }
    // Slow skybox drift (no-op unless the active environment enables animation).
    this.sceneManager.updateSkyboxAnimation(this.playing ? dt * this.speed : 0);
    this.controls.update();

    if (this.beforeRenderCallbacks.length > 0) {
      const info = this.computeFrameRenderInfo();
      for (const callback of [...this.beforeRenderCallbacks]) {
        callback(info);
      }
    }
    if (this.plugins.length > 0) {
      const renderContext = this.createRenderContext();
      for (const entry of this.plugins) {
        entry.plugin.beforeRender?.(renderContext);
      }
    }
    this.updateFreeCameraTransition();
    this.renderer.render(this.scene, this.camera);
  }

  private render(dt = 0): void {
    this.renderFrame(dt);
  }

  private updateFreeCameraTransition(): void {
    const transition = this.freeCameraTransition;
    if (!transition) return;

    this.controls.enabled = false;
    this.camera.position.lerp(transition.position, FREE_CAMERA_TRANSITION_SMOOTHING);
    this.controls.target.lerp(transition.target, FREE_CAMERA_TRANSITION_SMOOTHING);
    this.camera.up.lerp(transition.up, FREE_CAMERA_TRANSITION_SMOOTHING).normalize();
    this.camera.fov = THREE.MathUtils.lerp(
      this.camera.fov,
      transition.fov,
      FREE_CAMERA_TRANSITION_SMOOTHING,
    );
    this.camera.updateProjectionMatrix();
    this.camera.lookAt(this.controls.target);

    const reachedPosition =
      this.camera.position.distanceToSquared(transition.position) <=
      FREE_CAMERA_POSITION_EPSILON_SQ;
    const reachedTarget =
      this.controls.target.distanceToSquared(transition.target) <= FREE_CAMERA_TARGET_EPSILON_SQ;
    const reachedUp = this.camera.up.angleTo(transition.up) <= FREE_CAMERA_UP_EPSILON_RAD;
    const reachedFov = Math.abs(this.camera.fov - transition.fov) <= FREE_CAMERA_FOV_EPSILON;
    if (!reachedPosition || !reachedTarget || !reachedUp || !reachedFov) {
      return;
    }

    this.camera.position.copy(transition.position);
    this.controls.target.copy(transition.target);
    this.camera.up.copy(transition.up).normalize();
    this.camera.fov = transition.fov;
    this.camera.updateProjectionMatrix();
    this.camera.lookAt(transition.target);
    this.controls.enabled = true;
    this.freeCameraTransition = null;
  }

  /**
   * Per-player boost / supersonic effect state, ported from the original
   * GameEngine.updateScene(): only update particle emission while playing (so
   * paused frames don't emit at frozen positions), and pass isKickoffReset so
   * the kickoff boost-reset doesn't fire particles. ActorManager resolves the
   * car mesh and forwards to EffectsManager (the stub when `effects: false`).
   */
  private updatePlayerStates(): void {
    if (!this.playing) return;
    // Hitbox-only mode (@rlrml/player parity): bodies are hidden, so suppress
    // boost / supersonic trail emission too (the player hides its trails).
    const suppressTrails = this.hitboxOnlyModeEnabledValue;
    for (const entity of this.adapter.getAllPlayers()) {
      this.actorManager.updateBoostState(
        entity.name,
        entity.isBoosting && !suppressTrails,
        entity.isKickoffReset,
      );
      this.actorManager.updateSupersonicState(
        entity.name,
        entity.isSupersonic && !suppressTrails,
        entity.team,
      );
    }
  }

  /**
   * Drive HitboxManager from the parity display toggles each frame, after
   * ActorManager has applied entity transforms/visibility:
   *
   * - `hitboxWireframesEnabled` — per-car wireframe boxes (color-coded by
   *   family) tracking the live car meshes.
   * - `hitboxOnlyModeEnabled` — wireframes shown AND car bodies hidden
   *   (@rlrml/player semantics). ActorManager re-applies entity visibility on
   *   every updateFromFramework, so simply not hiding next frame recovers.
   */
  private updateHitboxVisualization(): void {
    const enabled = this.hitboxWireframesEnabledValue || this.hitboxOnlyModeEnabledValue;
    if (!enabled && !this.hitboxesActive) return;
    this.hitboxesActive = enabled;
    this.hitboxManager.setEnabled(enabled);
    if (!enabled) return;

    const am = this.actorManager as unknown as {
      actors: Record<string | number, THREE.Object3D | undefined>;
      playerNameToCarActorId: Record<string, string | number | undefined>;
    };
    if (!this.hitboxTypeByName) {
      this.hitboxTypeByName = new Map(
        this.adapter.getAllPlayers().map((entity) => [entity.name, entity.hitboxType]),
      );
    }
    this.hitboxManager.updateHitboxes(
      am.actors,
      am.playerNameToCarActorId,
      (name: string) => this.hitboxTypeByName?.get(name) ?? "Octane",
    );
    if (this.hitboxOnlyModeEnabledValue) {
      for (const carActorId of Object.values(am.playerNameToCarActorId)) {
        const carMesh = carActorId === undefined ? undefined : am.actors[carActorId];
        if (carMesh) {
          carMesh.visible = false;
        }
      }
    }
  }

  private installPlugin(definition: PlayerPluginDefinition, renderAfterSetup: boolean): () => void {
    const plugin = typeof definition === "function" ? definition() : definition;
    if (this.plugins.some((entry) => entry.plugin.id === plugin.id)) {
      throw new Error(`Player plugin "${plugin.id}" is already installed`);
    }

    const entry = { definition, plugin };
    this.plugins.push(entry);
    plugin.setup?.(this.createPluginContext());
    if (plugin.id === "camera") {
      // A camera plugin installed after construction picks up any parity
      // camera state already set through the ReplayPlayer surface.
      this.pushCameraParityState();
    }
    plugin.onStateChange?.(this.createPluginStateContext(this.getState()));
    if (renderAfterSetup) {
      this.render();
    }

    return () => {
      const index = this.plugins.indexOf(entry);
      if (index < 0) return;
      this.plugins.splice(index, 1);
      plugin.teardown?.(this.createPluginContext());
    };
  }

  /**
   * Build the `ReplayScene`-shaped sceneState. `ballMesh`/`playerMeshes` are
   * getters so they track the live actors (GLB model swaps replace the
   * Object3Ds; a snapshot would go stale).
   */
  private createSceneState(): ReplayScene {
    // ActorManager is untyped JS; view the lookup tables we read with types.
    const am = this.actorManager as unknown as {
      ballActorId: string | number | null;
      actors: Record<string | number, THREE.Object3D | undefined>;
      playerNameToCarActorId: Record<string, string | number | undefined>;
    };
    // Capture the instance: the returned object literal's getters need the
    // outer `this`, not their own.
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const player: ReplayPlayer = this;
    // Stand-in until the ball actor spawns (ReplayScene.ballMesh is non-null).
    const fallbackBallMesh = new THREE.Mesh();
    return {
      get scene() {
        return player.scene;
      },
      replayRoot: this.replayRoot,
      get camera() {
        return player.camera;
      },
      get renderer() {
        return player.renderer;
      },
      controls: this.controls,
      resize: () => this.sceneManager.onWindowResize(),
      // Parity with ReplayPlayer.destroy() → sceneState.dispose(); consumers
      // should normally call player.destroy() instead.
      dispose: () => this.destroy(),
      get ballMesh(): THREE.Mesh {
        const ball = am.ballActorId != null ? am.actors[am.ballActorId] : null;
        return (ball as THREE.Mesh) ?? fallbackBallMesh;
      },
      // Car Object3Ds keyed by stable player id, rebuilt per access.
      get playerMeshes(): Map<string, THREE.Object3D> {
        const map = new Map<string, THREE.Object3D>();
        for (const info of player.adapter.playerList) {
          const actorId = am.playerNameToCarActorId[info.name];
          const mesh = actorId != null ? am.actors[actorId] : undefined;
          if (mesh) map.set(info.id, mesh);
        }
        return map;
      },
      // Schematic-player internals with no counterpart in this renderer.
      playerBodyMeshes: new Map(),
      playerHitboxes: new Map(),
      playerBoostTrails: new Map(),
      playerBoostMeters: new Map(),
      playerDemoIndicators: new Map(),
      updateWallVisibility: () => {},
    };
  }

  private createPluginContext(): PlayerPluginContext {
    return {
      player: this,
      replay: this.replay,
      options: this.options,
      scene: this.scene,
      camera: this.camera,
      renderer: this.renderer,
      container: this.container,
    };
  }

  private createPluginStateContext(state: PlayerState): PlayerPluginStateContext {
    return { ...this.createPluginContext(), state };
  }

  private createRenderContext(): PlayerRenderContext {
    // ActorManager is untyped JS; view the two lookup tables we read with types.
    const am = this.actorManager as unknown as {
      ballActorId: string | number | null;
      actors: Record<string | number, THREE.Object3D | undefined>;
      playerNameToCarActorId: Record<string, string | number | undefined>;
    };
    const ball = this.adapter.ball;
    const ballState: BallRenderState = {
      position: ball.position,
      rotation: ball.rotation,
      velocity: ball.velocity,
      visible: ball.visible,
      object3d: am.ballActorId != null ? (am.actors[am.ballActorId] ?? null) : null,
    };
    const cars: CarRenderState[] = this.adapter.getAllPlayers().map((entity) => {
      const carActorId = am.playerNameToCarActorId[entity.name];
      return {
        id: entity.id,
        name: entity.name,
        team: entity.team,
        carName: entity.carName,
        hitboxType: entity.hitboxType,
        position: entity.position,
        rotation: entity.rotation,
        velocity: entity.velocity,
        boost: entity.boost,
        isBoosting: entity.isBoosting,
        visible: entity.isVisible,
        object3d: carActorId != null ? (am.actors[carActorId] ?? null) : null,
      };
    });
    return {
      ...this.createPluginContext(),
      ...this.computeFrameRenderInfo(),
      state: this.getState(),
      time: this.currentTime,
      ball: ballState,
      cars,
    };
  }

  private emitChange(): void {
    const state = this.getState();
    const pluginStateContext = this.createPluginStateContext(state);
    for (const entry of this.plugins) {
      entry.plugin.onStateChange?.(pluginStateContext);
    }
    this.dispatchEvent(new CustomEvent<PlayerState>("change", { detail: state }));
  }
}
