/**
 * Public types for @rlrml/player — most importantly the `PlayerPlugin` contract.
 *
 * The plugin model mirrors `@rlrml/player` (js/player/src/types.ts): a bare core
 * (`ReplayPlayer`) holds an ordered list of installed plugins and dispatches
 * lifecycle hooks. Everything above raw playback (scoreboard, name tags,
 * overlays, …) is a plugin — see docs/player/EXTENSIBILITY.md.
 *
 * The state / options / stepping types deliberately match `@rlrml/player`'s
 * `ReplayPlayerState` / `ReplayPlayerOptions` field-for-field so consumers
 * written against `ReplayPlayer` (notably js/stat-evaluation-player) can run on
 * `ReplayPlayer` unchanged — see docs/player/PLAYER_PARITY.md for the matrix.
 */
import type * as THREE from "three";
import type { ReplayModel, ReplayPlayerActiveMetadata } from "../types";
import type { Vec3, Quat } from "./adapter/coords.js";
import type { ReplayPlayer } from "./ReplayPlayer.js";
import type { PlayerEnvironmentSpec } from "./environments.js";

export type { Vec3, Quat };

/**
 * RL-style camera settings (original GameEngine defaults).
 *
 * `@rlrml/player` calls the pitch angle `pitch`; this package's original name
 * is `angle`. Both are accepted everywhere and mean the same thing — when both
 * are set, `angle` wins (it is the native field).
 */
export interface CameraSettings {
  /** Distance behind car in UU (RL range 100-400). */
  distance?: number;
  /** Height above car in UU (RL range 40-200). */
  height?: number;
  /** Pitch angle in degrees, negative = look down (RL range -15 to 0). */
  angle?: number;
  /** Alias for `angle` — the `@rlrml/player` field name. */
  pitch?: number;
  /** Camera stiffness (RL range 0.0-1.0; higher = more responsive). */
  stiffness?: number;
  /** Swivel speed around the car (RL range 1.0-10.0). */
  swivelSpeed?: number;
  /** Ball cam transition speed (RL range 1.0-2.0). */
  transitionSpeed?: number;
  /** HORIZONTAL field of view in degrees (RL range 60-110). */
  fov?: number;
  /** Free-cam fly speed in UU/s (player extension; no @rlrml/player analog). */
  freeCamSpeed?: number;
}

/** Parity with @rlrml/player: "free" = unattached camera, "follow" = car cam. */
export type PlayerCameraViewMode = "free" | "follow";

/** Canned free-camera poses, matching @rlrml/player's presets. */
export type PlayerFreeCameraPreset = "overhead" | "side";

/**
 * Snapshot of playback state, emitted on every "change" event. Shape-compatible
 * with `@rlrml/player`'s `ReplayPlayerState`.
 *
 * The hitbox toggles drive HitboxManager (wireframes; only-mode also hides
 * car bodies and trail emission). The boost-meter toggle is tracked-but-inert
 * for now: the setter updates state and notifies subscribers, but no rendering
 * is wired to it yet (docs/player/PLAYER_PARITY.md). The skip toggles are live when
 * the player has a `ReplayModel` (always, via `createPlayer`).
 */
export interface PlayerState {
  currentTime: number;
  duration: number;
  frameIndex: number;
  /** Kickoff countdown metadata (@rlrml/player semantics); null outside kickoffs. */
  activeMetadata: ReplayPlayerActiveMetadata | null;
  playing: boolean;
  speed: number;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  cameraViewMode: PlayerCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  /**
   * True when the camera follows the attached player's replicated ball-cam
   * toggle ("player" view) instead of a forced ball/car cam. Mirrors
   * `@rlrml/player`'s `ReplayPlayerState.useReplayBallCam`; optional so the
   * shape stays compatible. When true, `ballCamEnabled` reflects whatever the
   * recorded toggle currently resolves to.
   */
  useReplayBallCam?: boolean;
  /** Effective ball-cam state actually applied this frame (recorded or forced). */
  effectiveBallCamEnabled?: boolean;
  boostMeterEnabled: boolean;
  boostPickupAnimationEnabled: boolean;
  hitboxWireframesEnabled: boolean;
  hitboxOnlyModeEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
}

export type PlayerSnapshot = PlayerState;

/** Batch state patch for `setState()` — same keys as @rlrml/player's. */
export type PlayerStatePatch = Partial<
  Pick<
    PlayerState,
    | "currentTime"
    | "playing"
    | "speed"
    | "cameraDistanceScale"
    | "customCameraSettings"
    | "cameraViewMode"
    | "attachedPlayerId"
    | "ballCamEnabled"
    | "useReplayBallCam"
    | "boostMeterEnabled"
    | "boostPickupAnimationEnabled"
    | "hitboxWireframesEnabled"
    | "hitboxOnlyModeEnabled"
    | "skipPostGoalTransitionsEnabled"
    | "skipKickoffsEnabled"
  >
>;

/** Per-render frame timing info handed to `onBeforeRender` callbacks. */
export interface FrameRenderInfo {
  frameIndex: number;
  nextFrameIndex: number;
  /** Interpolation fraction between frameIndex and nextFrameIndex (0-1). */
  alpha: number;
  currentTime: number;
}

export type BeforeRenderCallback = (info: FrameRenderInfo) => void;

/** Per-frame resolved ball state handed to `beforeRender`. */
export interface BallRenderState {
  position: Vec3;
  rotation: Quat;
  velocity: Vec3;
  visible: boolean;
  /** The ball's THREE object (post-interpolation transform), if spawned. */
  object3d: THREE.Object3D | null;
}

/** Per-frame resolved car state handed to `beforeRender`. */
export interface CarRenderState {
  /** Stable player id (from the replay's remote id) — matches `playerList[].id`. */
  id: string;
  name: string;
  team: number;
  carName: string;
  hitboxType: string;
  position: Vec3;
  rotation: Quat;
  velocity: Vec3;
  /** 0-100 */
  boost: number;
  isBoosting: boolean;
  visible: boolean;
  /** The car's THREE object (post-interpolation transform), if spawned. */
  object3d: THREE.Object3D | null;
}

export interface PlayerPluginContext {
  /** The core; exposes playback control, state, and the subtr-actor adapter. */
  player: ReplayPlayer;
  /**
   * @rlrml/player's normalized ReplayModel (`player.replay`) — the shared data
   * layer plugins written against `ReplayPlayerPluginContext` read. Null only
   * when the ReplayPlayer was constructed directly without one.
   */
  replay: ReplayModel | null;
  /** The constructor options the player was created with. */
  options: PlayerOptions;
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  /** For plugins that add DOM overlays (HUD, scoreboard, indicators). */
  container: HTMLElement;
}

export interface PlayerPluginStateContext extends PlayerPluginContext {
  state: PlayerState;
}

export interface PlayerRenderContext extends PlayerPluginStateContext, FrameRenderInfo {
  /** Current playback time (s) — same value as `currentTime` (player-native name). */
  time: number;
  ball: BallRenderState;
  cars: CarRenderState[];
}

export interface PlayerPlugin {
  id: string;
  /** Install: attach meshes/DOM, subscribe to events. */
  setup?(ctx: PlayerPluginContext): void;
  /** Play/pause/seek/speed changes (and once at install). */
  onStateChange?(ctx: PlayerPluginStateContext): void;
  /** Per frame, after ball/car transforms resolve, before renderer.render. */
  beforeRender?(ctx: PlayerRenderContext): void;
  /** Uninstall: dispose everything created in setup. */
  teardown?(ctx: PlayerPluginContext): void;
}

export type PlayerPluginFactory = () => PlayerPlugin;
export type PlayerPluginDefinition = PlayerPlugin | PlayerPluginFactory;

export interface PlayerOptions {
  plugins?: PlayerPluginDefinition[];
  autoplay?: boolean;
  /**
   * Base URL for bundled player assets (`models/`, `draco/`, optional
   * `skyboxes/`). Defaults to Vite's `BASE_URL`, which keeps GitHub Pages
   * subpath deployments working when assets are copied beside the app bundle.
   */
  assetBase?: string | URL;
  /** Initial playback rate (default 1). Player-native alias of initialPlaybackRate. */
  speed?: number;
  /** Wrap to t=0 at the end instead of pausing (default false). */
  loop?: boolean;
  /** Boost/supersonic/ball trail effects (default true). */
  effects?: boolean;
  /**
   * Keep the WebGL drawing buffer readable after rendering. Off by default for
   * normal playback; static image capture enables it before calling
   * canvas.toBlob()/toDataURL().
   */
  preserveDrawingBuffer?: boolean;
  /**
   * Skybox environment driving the background + image-based lighting
   * (reflections/ambient on cars, arena, ball). A built-in id (default
   * `"space"`), a full `PlayerEnvironment` descriptor, or `false` for neutral
   * default lighting (no skybox). Loaded lazily — playback starts on the neutral
   * fallback and the HDR swaps in once decoded. See `./environments.ts`.
   */
  environment?: PlayerEnvironmentSpec;
  /**
   * Position interpolation between ~30Hz replay samples (default "linear",
   * matching Ballcam's production player). "linear" is plain lerp;
   * "hermite" uses per-sample linear velocities as cubic tangents with a lerp
   * fallback when velocity is missing or implausible.
   */
  motionInterpolation?: "hermite" | "linear";
  /**
   * Preprocess ball/car timelines with Ballcam-style velocity correction before
   * render-time interpolation (default true). Set false to inspect raw samples.
   */
  motionSmoothing?: boolean;
  /** Velocity-correction blend toward measured samples (default 0.15). */
  smoothingBlendFactor?: number;
  /** Every N corrected samples, use a stronger measured-sample anchor (default 10). */
  smoothingAnchorInterval?: number;
  /**
   * Remove pre-kickoff idle time and post-goal replay gaps from motion playback,
   * matching Ballcam's compiled .rlrf time axis (default false; changes
   * currentTime semantics relative to `player.replay`).
   */
  timelineCompaction?: boolean;
  /** Disable velocity/position consistency filtering after smoothing (default false). */
  disableFrameFiltering?: boolean;

  // ── @rlrml/player-compatible initial settings (docs/player/PLAYER_PARITY.md). ──────
  /** Initial playback rate; wins over `speed` when both are set. */
  initialPlaybackRate?: number;
  initialCameraDistanceScale?: number;
  initialCustomCameraSettings?: CameraSettings | null;
  initialCameraViewMode?: PlayerCameraViewMode;
  initialAttachedPlayerId?: string | null;
  initialBallCamEnabled?: boolean;
  /** Tracked-but-inert (no boost meter rendering yet). */
  initialBoostMeterEnabled?: boolean;
  /** Read by the bridged boost-pickup-animation plugin when installed. */
  initialBoostPickupAnimationEnabled?: boolean;
  /** Per-car hitbox wireframes (HitboxManager). */
  initialHitboxWireframesEnabled?: boolean;
  /** Hitbox-only mode: wireframes shown, car bodies + trails hidden. */
  initialHitboxOnlyModeEnabled?: boolean;
  /** Live when a ReplayModel is present (@rlrml/player default: true). */
  initialSkipPostGoalTransitionsEnabled?: boolean;
  /** Live when a ReplayModel is present (@rlrml/player default: false). */
  initialSkipKickoffsEnabled?: boolean;
}
