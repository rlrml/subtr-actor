import type { Group, Object3D } from "three";
import type { RawReplayFramesData, RawShotEventMetadata } from "./raw-types";
import type { ReplayScene } from "./scene";
import type { PlaybackBound } from "./generated/PlaybackBound";
import type { PlaylistAdvanceMode } from "./generated/PlaylistAdvanceMode";
import type { PlaylistEndMode } from "./generated/PlaylistEndMode";
import type { ReplayHitboxSpec } from "./hitboxes";
import type { PlayerOptions } from "./player/types";

export type { PlaybackBound } from "./generated/PlaybackBound";
export type { PlaylistAdvanceMode } from "./generated/PlaylistAdvanceMode";
export type { PlaylistEndMode } from "./generated/PlaylistEndMode";
export type { PlaylistManifest } from "./generated/PlaylistManifest";
export type { PlaylistManifestItem } from "./generated/PlaylistManifestItem";
export type { PlaylistManifestPage } from "./generated/PlaylistManifestPage";
export type { PlaylistManifestReplay } from "./generated/PlaylistManifestReplay";
export type { PlaylistManifestReplayLocator } from "./generated/PlaylistManifestReplayLocator";
export type { PlaylistPlaybackOptions as PlaylistManifestPlaybackOptions } from "./generated/PlaylistPlaybackOptions";

export type {
  RawBallData,
  RawBallFrame,
  RawBallFrameData,
  RawBoostPad,
  RawBoostPadEvent,
  RawDemolishInfo,
  RawGoalEvent,
  RawMetadataFrame,
  RawPlayerData,
  RawPlayerFrame,
  RawPlayerFrameData,
  RawPlayerId,
  RawPlayerInfo,
  RawPlayerStatEvent,
  RawPlayerStatEventKind,
  RawReplayGameType,
  RawReplayGameTypeDetails,
  RawReplayTickMark,
  RawReplayFramesData,
  RawRigidBody,
  RawRotation,
  RawShotEventMetadata,
  RawVec3,
} from "./raw-types";

export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export interface Quaternion {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface CameraSettings {
  fov?: number;
  height?: number;
  pitch?: number;
  distance?: number;
  stiffness?: number;
  swivelSpeed?: number;
  transitionSpeed?: number;
}

export type ReplayCameraViewMode = "free" | "follow";

export type ReplayFreeCameraPreset = "overhead" | "side";

export interface PlaybackFrame {
  time: number;
  secondsRemaining: number;
  gameState: number;
  kickoffCountdown: number;
}

export interface ReplayPlayerKickoffCountdownMetadata {
  kind: "kickoff-countdown";
  countdown: number;
  secondsRemaining: number;
  endsAt: number;
}

export type ReplayPlayerActiveMetadata = ReplayPlayerKickoffCountdownMetadata;

export interface BallSample {
  position: Vec3 | null;
  linearVelocity: Vec3 | null;
  angularVelocity: Vec3 | null;
  rotation: Quaternion | null;
}

export interface PlayerSample {
  isPresent?: boolean;
  position: Vec3 | null;
  linearVelocity: Vec3 | null;
  angularVelocity: Vec3 | null;
  rotation: Quaternion | null;
  forward: Vec3 | null;
  up: Vec3 | null;
  boostAmount: number;
  boostFraction: number;
  boostActive: boolean;
  powerslideActive: boolean;
  jumpActive: boolean;
  doubleJumpActive: boolean;
  dodgeActive: boolean;
  /**
   * Replay-driven continuous camera look angles, in radians, derived from the
   * replicated bytes. `null`/absent when the replay does not carry them (older
   * replays, or before the player's camera-settings actor appears). Discrete
   * camera toggles (ball cam, behind-view) live in {@link ReplayPlayerTrack.cameraEvents}.
   */
  cameraPitch?: number | null;
  cameraYaw?: number | null;
  /** Throttle input, normalized to -1 (full reverse) .. 1 (full forward). */
  throttle?: number | null;
  /** Steer input, normalized to -1 (full left) .. 1 (full right). */
  steer?: number | null;
  /** Impulse vector of the most recent dodge (raw replay units). */
  dodgeImpulse?: Vec3 | null;
  /** Torque vector of the most recent dodge (raw replay units). */
  dodgeTorque?: Vec3 | null;
}

/**
 * A coalesced change in a player's discrete camera/vehicle toggles, mirroring
 * the Rust `PlayerCameraStateChange`. Resolve the state at a frame by taking the
 * last change at or before that frame. `null` fields were never replicated.
 */
export interface CameraStateChange {
  frame: number;
  ballCamActive: boolean | null;
  behindViewActive: boolean | null;
  driving: boolean | null;
}

export interface ReplayPlayerTrack {
  id: string;
  name: string;
  isTeamZero: boolean;
  cameraSettings: CameraSettings;
  hitbox: ReplayHitboxSpec;
  frames: PlayerSample[];
  /**
   * Coalesced ball-cam / behind-view / driving changes for this player, sorted
   * by frame. Empty/absent when the replay carries no camera events for the
   * player.
   */
  cameraEvents?: CameraStateChange[];
}

export type ReplayTimelineEventKind = "goal" | "shot" | "save" | "assist" | "demo" | (string & {});

export interface ReplayTickMark {
  id?: string;
  description: string;
  frame: number | null;
  time: number;
}

export interface ReplayTimelineEvent {
  id?: string;
  time: number;
  seekTime?: number;
  frame?: number;
  kind: ReplayTimelineEventKind;
  label?: string;
  shortLabel?: string;
  iconText?: string;
  iconName?: string;
  playerId?: string | null;
  playerName?: string | null;
  secondaryPlayerId?: string | null;
  secondaryPlayerName?: string | null;
  location?: Vec3 | null;
  shot?: RawShotEventMetadata | null;
  isTeamZero?: boolean | null;
  color?: string;
}

export interface ReplayTimelineRange {
  id?: string;
  startTime: number;
  endTime: number;
  lane?: string;
  laneLabel?: string;
  label?: string;
  shortLabel?: string;
  isTeamZero?: boolean | null;
  color?: string;
  className?: string;
}

export interface ReplayPlayerTimelineSegment {
  startTime: number;
  endTime: number;
}

export interface ReplayPlayerTimelineProjection {
  replayTime: number;
  timelineTime: number;
  seekTime: number;
  hiddenBySkip: boolean;
}

export type ReplayBoostPadSize = "big" | "small";

export interface ReplayBoostPadEvent {
  time: number;
  frame: number;
  available: boolean;
  playerId?: string | null;
  playerName?: string | null;
}

export interface ReplayBoostPad {
  index: number;
  padId: string | null;
  size: ReplayBoostPadSize;
  position: Vec3;
  events: ReplayBoostPadEvent[];
}

export interface ReplayModel {
  frameCount: number;
  duration: number;
  rawStartTime: number;
  frames: PlaybackFrame[];
  ballFrames: BallSample[];
  boostPads: ReplayBoostPad[];
  players: ReplayPlayerTrack[];
  tickMarks: ReplayTickMark[];
  timelineEvents: ReplayTimelineEvent[];
  teamZeroNames: string[];
  teamOneNames: string[];
}

export interface ReplayLoadResult {
  replay: ReplayModel;
  raw: RawReplayFramesData;
}

export type ReplayLoadStage = "validating" | "processing" | "normalizing" | (string & {});

export interface ReplayLoadProgress {
  stage: ReplayLoadStage;
  processedFrames?: number;
  totalFrames?: number;
  progress?: number;
}

export interface ReplayLoadOptions {
  onProgress?: (progress: ReplayLoadProgress) => void;
  reportEveryNFrames?: number;
  useWorker?: boolean;
  /**
   * Apply Ballcam-style velocity-based correction while normalizing replay
   * samples. Defaults to true; set false to inspect exact raw frame positions.
   */
  motionSmoothing?: boolean;
  /** Blend toward the measured replay sample during velocity correction. */
  smoothingBlendFactor?: number;
  /** Every N corrected samples, use a stronger measured-sample anchor. */
  smoothingAnchorInterval?: number;
}

export interface ReplayLoadOverlayOptions {
  title?: string;
  formatProgress?: (progress: ReplayLoadProgress) => string;
}

export interface ReplayLoadOverlayController {
  update(progress: ReplayLoadProgress): void;
  complete(message?: string): void;
  fail(message: string): void;
  destroy(): void;
}

export interface LoadedReplay {
  replay: ReplayModel;
  raw?: RawReplayFramesData;
}

export type PlaylistSourceLoadStatus = "idle" | "loading" | "loaded" | "error";

export interface PlaylistSourceLoadProgress {
  stage?: string;
  message?: string;
  progress?: number;
  processedBytes?: number;
  totalBytes?: number;
  processedFrames?: number;
  totalFrames?: number;
}

export interface PlaylistSourceLoadContext {
  sourceId: string;
  updateProgress: (progress: PlaylistSourceLoadProgress) => void;
}

export interface PlaylistSourceLoadState {
  sourceId: string;
  status: PlaylistSourceLoadStatus;
  progress: PlaylistSourceLoadProgress | null;
  error: string | null;
  startedAt: number | null;
  updatedAt: number | null;
  completedAt: number | null;
}

export interface PlaylistLoadSource<TLoaded> {
  id: string;
  load: (context?: PlaylistSourceLoadContext) => Promise<TLoaded>;
}

export interface ReplaySource extends PlaylistLoadSource<LoadedReplay> {}

export interface PlaylistItem<TSource extends PlaylistLoadSource<unknown> = ReplaySource> {
  replay: TSource;
  start: PlaybackBound;
  end: PlaybackBound;
  label?: string;
  meta?: Record<string, unknown>;
}

export interface PlaylistPlaybackOptions {
  /**
   * Controls what happens when the active playlist item reaches its end bound.
   *
   * - "auto" advances to the next item.
   * - "manual" pauses at the item end until the caller chooses another item.
   */
  advanceMode?: PlaylistAdvanceMode;
  /**
   * Controls what happens when automatic advancement reaches the end of the
   * playlist.
   *
   * - "stop" pauses at the final item end.
   * - "loop" continues playback from the first item.
   */
  endMode?: PlaylistEndMode;
  /**
   * @deprecated Use advanceMode instead. true maps to "auto", false maps to
   * "manual".
   */
  advanceOnEnd?: boolean;
}

export interface ResolvedPlaybackBound {
  frameIndex: number;
  time: number;
}

export interface ResolvedPlaylistItem {
  source: PlaylistItem;
  replay: LoadedReplay;
  start: ResolvedPlaybackBound;
  end: ResolvedPlaybackBound;
  duration: number;
}

export interface PlaylistPreloadContext<
  TSource extends PlaylistLoadSource<unknown> = ReplaySource,
  TItem extends PlaylistItem<TSource> = PlaylistItem<TSource>,
> {
  items: TItem[];
  currentIndex: number;
  currentItem: TItem;
}

export type PlaylistPreloadPolicy<
  TSource extends PlaylistLoadSource<unknown> = ReplaySource,
  TItem extends PlaylistItem<TSource> = PlaylistItem<TSource>,
> =
  | { kind: "none" }
  | { kind: "all" }
  | { kind: "adjacent"; ahead: number; behind?: number }
  | {
      kind: "custom";
      pick: (context: PlaylistPreloadContext<TSource, TItem>) => Iterable<string | TSource>;
    };

export type ReplayPreloadContext = PlaylistPreloadContext<ReplaySource, PlaylistItem>;

export type ReplayPreloadPolicy = PlaylistPreloadPolicy<ReplaySource, PlaylistItem>;

export interface ReplayPlayerPluginContext {
  player: any;
  replay: ReplayModel;
  scene: ReplayScene;
  container: HTMLElement;
  options: ReplayPlayerOptions;
}

export interface ReplayPlayerPluginStateContext extends ReplayPlayerPluginContext {
  state: ReplayPlayerState;
}

export interface ReplayPlayerRenderTrackContext {
  track: ReplayPlayerTrack;
  mesh: Object3D | null;
  boostTrail: Group | null;
  frame: PlayerSample | null;
  nextFrame: PlayerSample | null;
  interpolatedPosition: Vec3 | null;
  boostFraction: number;
}

export interface ReplayPlayerRenderContext extends ReplayPlayerPluginStateContext, FrameRenderInfo {
  frame: PlaybackFrame | null;
  nextFrame: PlaybackFrame | null;
  ballFrame: BallSample | null;
  nextBallFrame: BallSample | null;
  ballPosition: Vec3 | null;
  players: ReplayPlayerRenderTrackContext[];
}

export interface ReplayPlayerPlugin {
  id: string;
  setup?(context: ReplayPlayerPluginContext): void;
  onStateChange?(context: ReplayPlayerPluginStateContext): void;
  beforeRender?(context: ReplayPlayerRenderContext): void;
  teardown?(context: ReplayPlayerPluginContext): void;
}

export type ReplayPlayerPluginFactory = () => ReplayPlayerPlugin;
export type ReplayPlayerPluginDefinition = ReplayPlayerPlugin | ReplayPlayerPluginFactory;

export type ReplayTimelineEventSource =
  | ReplayTimelineEvent[]
  | ((context: ReplayPlayerPluginContext) => ReplayTimelineEvent[]);

export type ReplayTimelineRangeSource =
  | ReplayTimelineRange[]
  | ((context: ReplayPlayerPluginContext) => ReplayTimelineRange[]);

export interface ReplayPlayerOptions {
  autoplay?: boolean;
  fieldScale?: number;
  initialCameraDistanceScale?: number;
  initialCustomCameraSettings?: CameraSettings | null;
  initialCameraViewMode?: ReplayCameraViewMode;
  initialAttachedPlayerId?: string | null;
  initialBallCamEnabled?: boolean;
  /**
   * When following a player, drive ball cam from that player's replicated
   * ball-cam toggle instead of the manual `ballCamEnabled` flag. Defaults to
   * `true`. Calling `setBallCamEnabled` turns this off (manual override).
   */
  initialUseReplayBallCam?: boolean;
  /**
   * Opt-in (default false): rotate the follow camera by the player's replicated
   * camera yaw/pitch (free-look swivel) instead of locking to car orientation.
   * Needs per-replay sign tuning; off by default to preserve the synthetic feel.
   */
  initialUseReplayCameraLook?: boolean;
  initialBoostMeterEnabled?: boolean;
  initialBoostPickupAnimationEnabled?: boolean;
  initialHitboxWireframesEnabled?: boolean;
  initialHitboxOnlyModeEnabled?: boolean;
  initialPlaybackRate?: number;
  initialSkipPostGoalTransitionsEnabled?: boolean;
  initialSkipKickoffsEnabled?: boolean;
  plugins?: ReplayPlayerPluginDefinition[];
}

export interface ReplayPlaylistPlayerOptions
  extends Omit<PlayerOptions, "autoplay">, PlaylistPlaybackOptions {
  autoplay?: boolean;
  initialItemIndex?: number;
  preloadPolicy?: ReplayPreloadPolicy;
  preloadRadius?: number;
}

export interface ReplayPlayerState {
  currentTime: number;
  duration: number;
  frameIndex: number;
  activeMetadata: ReplayPlayerActiveMetadata | null;
  playing: boolean;
  speed: number;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  /**
   * Whether the followed player's replicated ball-cam toggle drives the camera.
   * Optional so the structurally-compatible `PlayerState` (which has no
   * such notion) stays assignable to `ReplayPlayerState`.
   */
  useReplayBallCam?: boolean;
  /** Effective ball-cam state actually applied this frame (replay or manual). */
  effectiveBallCamEnabled?: boolean;
  boostMeterEnabled: boolean;
  boostPickupAnimationEnabled: boolean;
  hitboxWireframesEnabled: boolean;
  hitboxOnlyModeEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
}

export interface ReplayPlaylistPlayerState {
  ready: boolean;
  loading: boolean;
  error: string | null;
  replayLoadStates: PlaylistSourceLoadState[];
  itemIndex: number;
  itemCount: number;
  item: PlaylistItem | null;
  advanceMode: PlaylistAdvanceMode;
  endMode: PlaylistEndMode;
  itemEnded: boolean;
  playlistEnded: boolean;
  currentTime: number;
  duration: number;
  replayCurrentTime: number;
  replayDuration: number;
  frameIndex: number;
  activeMetadata: ReplayPlayerActiveMetadata | null;
  playing: boolean;
  speed: number;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  boostPickupAnimationEnabled: boolean;
  hitboxWireframesEnabled: boolean;
  hitboxOnlyModeEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
}

export type ReplayPlayerSnapshot = ReplayPlayerState;
export type ReplayPlaylistPlayerSnapshot = ReplayPlaylistPlayerState;

export type ReplayPlayerStatePatch = Partial<
  Pick<
    ReplayPlayerState,
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

export interface FrameRenderInfo {
  frameIndex: number;
  nextFrameIndex: number;
  alpha: number;
  currentTime: number;
}

export type BeforeRenderCallback = (info: FrameRenderInfo) => void;
