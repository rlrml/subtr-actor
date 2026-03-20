import type { Group, Object3D } from "three";
import type { ReplayScene } from "./scene";
import type { ReplayPlayer } from "./player";

export interface RawVec3 {
  x: number;
  y: number;
  z: number;
}

export interface RawRotation {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface RawRigidBody {
  sleeping: boolean;
  location: RawVec3;
  rotation: RawRotation;
  linear_velocity: RawVec3;
  angular_velocity: RawVec3;
}

export interface RawBallFrameData {
  rigid_body: RawRigidBody;
}

export interface RawPlayerFrameData {
  rigid_body: RawRigidBody;
  boost_amount: number;
  boost_active: boolean;
  powerslide_active: boolean;
  jump_active: boolean;
  double_jump_active: boolean;
  dodge_active: boolean;
  player_name?: string;
  team?: number;
  is_team_0?: boolean;
}

export type RawBallFrame = "Empty" | { Data: RawBallFrameData };
export type RawPlayerFrame = "Empty" | { Data: RawPlayerFrameData };

export interface RawPlayerData {
  frames: RawPlayerFrame[];
}

export interface RawBallData {
  frames: RawBallFrame[];
}

export interface RawMetadataFrame {
  time: number;
  seconds_remaining: number;
  replicated_game_state_name: number;
  replicated_game_state_time_remaining: number;
}

export interface RawPlayerInfo {
  remote_id?: Record<string, string>;
  stats?: Map<string, unknown> | Record<string, unknown> | null;
  name: string;
}

export type RawPlayerStatEventKind = "Shot" | "Save" | "Assist";

export interface RawDemolishInfo {
  time: number;
  seconds_remaining: number;
  frame: number;
  attacker: Record<string, string>;
  victim: Record<string, string>;
  attacker_velocity: RawVec3;
  victim_velocity: RawVec3;
  victim_location: RawVec3;
}

export interface RawGoalEvent {
  time: number;
  frame: number;
  scoring_team_is_team_0: boolean;
  player?: Record<string, string> | null;
  team_zero_score?: number | null;
  team_one_score?: number | null;
}

export interface RawPlayerStatEvent {
  time: number;
  frame: number;
  player: Record<string, string>;
  is_team_0: boolean;
  kind: RawPlayerStatEventKind;
}

export interface RawBoostPadEvent {
  time: number;
  frame: number;
  pad_id: string;
  player?: Record<string, string> | null;
  kind: unknown;
}

export interface RawBoostPad {
  index: number;
  pad_id?: string | null;
  size: unknown;
  position: RawVec3;
}

export interface RawReplayFramesData {
  frame_data: {
    ball_data: RawBallData;
    players: Array<[Record<string, string>, RawPlayerData]>;
    metadata_frames: RawMetadataFrame[];
  };
  meta: {
    team_zero: RawPlayerInfo[];
    team_one: RawPlayerInfo[];
    all_headers: unknown[];
  };
  demolish_infos?: RawDemolishInfo[];
  boost_pad_events?: RawBoostPadEvent[];
  boost_pads?: RawBoostPad[];
  goal_events?: RawGoalEvent[];
  touch_events?: unknown[];
  player_stat_events?: RawPlayerStatEvent[];
}

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

export type ReplayCameraViewMode =
  | "free"
  | "follow";

export type ReplayFreeCameraPreset =
  | "overhead"
  | "side";

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

export type ReplayPlayerActiveMetadata =
  | ReplayPlayerKickoffCountdownMetadata;

export interface BallSample {
  position: Vec3 | null;
  linearVelocity: Vec3 | null;
  angularVelocity: Vec3 | null;
  rotation: Quaternion | null;
}

export interface PlayerSample {
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
}

export interface ReplayPlayerTrack {
  id: string;
  name: string;
  isTeamZero: boolean;
  cameraSettings: CameraSettings;
  frames: PlayerSample[];
}

export type ReplayTimelineEventKind =
  | "goal"
  | "shot"
  | "save"
  | "assist"
  | "demo"
  | (string & {});

export interface ReplayTimelineEvent {
  id?: string;
  time: number;
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
  frames: PlaybackFrame[];
  ballFrames: BallSample[];
  boostPads: ReplayBoostPad[];
  players: ReplayPlayerTrack[];
  timelineEvents: ReplayTimelineEvent[];
  teamZeroNames: string[];
  teamOneNames: string[];
}

export interface ReplayLoadResult {
  replay: ReplayModel;
  raw: RawReplayFramesData;
}

export type ReplayLoadStage =
  | "validating"
  | "processing"
  | "normalizing"
  | (string & {});

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

export type PlaybackBound =
  | { kind: "frame"; value: number }
  | { kind: "time"; value: number };

export interface ReplaySource {
  id: string;
  load: () => Promise<LoadedReplay>;
}

export interface PlaylistItem {
  replay: ReplaySource;
  start: PlaybackBound;
  end: PlaybackBound;
  label?: string;
  meta?: Record<string, unknown>;
}

export interface PlaylistManifestReplay {
  id: string;
  path?: string;
  label?: string;
  meta?: Record<string, unknown>;
}

export interface PlaylistManifestItem {
  replay: string;
  start: PlaybackBound;
  end: PlaybackBound;
  label?: string;
  meta?: Record<string, unknown>;
}

export interface PlaylistManifest {
  replays?: PlaylistManifestReplay[];
  items: PlaylistManifestItem[];
  label?: string;
  meta?: Record<string, unknown>;
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

export interface ReplayPreloadContext {
  items: PlaylistItem[];
  currentIndex: number;
  currentItem: PlaylistItem;
}

export type ReplayPreloadPolicy =
  | { kind: "none" }
  | { kind: "all" }
  | { kind: "adjacent"; ahead: number; behind?: number }
  | {
      kind: "custom";
      pick: (
        context: ReplayPreloadContext
      ) => Iterable<string | ReplaySource>;
    };

export interface ReplayPlayerPluginContext {
  player: ReplayPlayer;
  replay: ReplayModel;
  scene: ReplayScene;
  container: HTMLElement;
  options: ReplayPlayerOptions;
}

export interface ReplayPlayerPluginStateContext
  extends ReplayPlayerPluginContext {
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

export interface ReplayPlayerRenderContext
  extends ReplayPlayerPluginStateContext,
    FrameRenderInfo {
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
export type ReplayPlayerPluginDefinition =
  | ReplayPlayerPlugin
  | ReplayPlayerPluginFactory;

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
  initialCameraViewMode?: ReplayCameraViewMode;
  initialAttachedPlayerId?: string | null;
  initialBallCamEnabled?: boolean;
  initialBoostMeterEnabled?: boolean;
  initialPlaybackRate?: number;
  initialSkipPostGoalTransitionsEnabled?: boolean;
  initialSkipKickoffsEnabled?: boolean;
  plugins?: ReplayPlayerPluginDefinition[];
}

export interface ReplayPlaylistPlayerOptions
  extends Omit<ReplayPlayerOptions, "autoplay"> {
  autoplay?: boolean;
  advanceOnEnd?: boolean;
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
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  boostMeterEnabled: boolean;
  skipPostGoalTransitionsEnabled: boolean;
  skipKickoffsEnabled: boolean;
}

export interface ReplayPlaylistPlayerState {
  ready: boolean;
  loading: boolean;
  error: string | null;
  itemIndex: number;
  itemCount: number;
  item: PlaylistItem | null;
  currentTime: number;
  duration: number;
  replayCurrentTime: number;
  replayDuration: number;
  frameIndex: number;
  activeMetadata: ReplayPlayerActiveMetadata | null;
  playing: boolean;
  speed: number;
  cameraDistanceScale: number;
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
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
    | "cameraViewMode"
    | "attachedPlayerId"
    | "ballCamEnabled"
    | "boostMeterEnabled"
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
