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
  demolish_infos?: unknown[];
  boost_pad_events?: unknown[];
  goal_events?: unknown[];
  touch_events?: unknown[];
  player_stat_events?: unknown[];
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

export interface ReplayModel {
  frameCount: number;
  duration: number;
  frames: PlaybackFrame[];
  ballFrames: BallSample[];
  players: ReplayPlayerTrack[];
  teamZeroNames: string[];
  teamOneNames: string[];
}

export interface ReplayLoadResult {
  replay: ReplayModel;
  raw: RawReplayFramesData;
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

export interface ReplayPlayerOptions {
  autoplay?: boolean;
  fieldScale?: number;
  initialCameraDistanceScale?: number;
  initialAttachedPlayerId?: string | null;
  initialBallCamEnabled?: boolean;
  initialPlaybackRate?: number;
  initialSkipPostGoalTransitionsEnabled?: boolean;
  initialSkipKickoffsEnabled?: boolean;
}

export interface ReplayPlaylistPlayerOptions
  extends Omit<ReplayPlayerOptions, "autoplay"> {
  autoplay?: boolean;
  advanceOnEnd?: boolean;
  initialItemIndex?: number;
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
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
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
  playing: boolean;
  speed: number;
  cameraDistanceScale: number;
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
    | "attachedPlayerId"
    | "ballCamEnabled"
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
