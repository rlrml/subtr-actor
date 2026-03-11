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
}

export interface RawPlayerInfo {
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
  demolish_infos: unknown[];
}

export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export interface PlaybackFrame {
  time: number;
  secondsRemaining: number;
  gameState: number;
}

export interface BallSample {
  position: Vec3 | null;
}

export interface PlayerSample {
  position: Vec3 | null;
  velocity: Vec3 | null;
  boostAmount: number;
  boostActive: boolean;
  jumpActive: boolean;
  dodgeActive: boolean;
}

export interface ReplayPlayerTrack {
  id: string;
  name: string;
  isTeamZero: boolean;
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

export interface ReplayPlayerOptions {
  autoplay?: boolean;
  fieldScale?: number;
  initialCameraMode?: CameraMode;
  initialTrackedPlayerId?: string;
}

export type CameraMode = "overview" | "attached" | "third-person";

export interface ReplayPlayerSnapshot {
  currentTime: number;
  duration: number;
  frameIndex: number;
  playing: boolean;
  speed: number;
  cameraMode: CameraMode;
  trackedPlayerId: string | null;
}
