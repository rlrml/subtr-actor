// Participant colors palette (12 highly distinguishable colors)
export const PARTICIPANT_COLORS = [
  '#FF0000', // Red
  '#0066FF', // Blue
  '#00CC00', // Green
  '#FFDD00', // Yellow
  '#FF69B4', // Pink
  '#FF8800', // Orange
  '#00DDDD', // Cyan
  '#9933FF', // Violet
  '#FF4488', // Magenta
  '#88FF00', // Lime
  '#FFFFFF', // White
  '#AAAAAA', // Gray
] as const;

// Camera modes
export type CameraMode = 'free' | 'ballOrbit' | 'player';

// Speed units
export type SpeedUnit = 'kmh' | 'mph';

// Participant role
export type ParticipantRole = 'host' | 'viewer';

// Vector3 interface
export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

// Quaternion interface
export interface Quaternion {
  x: number;
  y: number;
  z: number;
  w: number;
}

// Camera state
export interface CameraState {
  position: Vector3;
  rotation: Quaternion;
  mode: CameraMode;
  targetPlayer: string | null;
  timestamp: number;
}

// Playback state (controlled by host)
export interface PlaybackState {
  timestamp: number;
  speed: number;
  paused: boolean;
  lastSyncAt: number;
  seeking?: boolean; // True when host is actively dragging the timeline
}

// Environment settings
export interface EnvironmentSettings {
  skyboxId: string;
  exposure: number;
  showHitboxes: boolean;
  showBallSpeed: boolean;
  showCarSpeed: boolean;
  speedUnit: SpeedUnit;
  customEnvironmentId?: string | null; // Custom environment from database
}

// Default environment settings
export const DEFAULT_ENVIRONMENT: EnvironmentSettings = {
  skyboxId: 'HighFantasy4k',
  exposure: 1.0,
  showHitboxes: false,
  showBallSpeed: false,
  showCarSpeed: false,
  speedUnit: 'kmh',
  customEnvironmentId: null,
};

// Participant
export interface Participant {
  id: string;
  nickname: string;
  color: string;
  role: ParticipantRole;
  joinedAt: number;
  camera: CameraState;
  followingId: string | null;
}

// Chat message
export interface ChatMessage {
  id: string;
  authorId: string;
  authorNickname: string;
  authorColor: string;
  text: string;
  timestamp: number;
}

// Session
export interface Session {
  id: string;
  replayId: string;
  hostId: string;
  createdAt: number;
  playback: PlaybackState;
  environment: EnvironmentSettings;
  participants: Record<string, Participant>;
  chatHistory: ChatMessage[];
  drawingState?: DrawingState;
}

// Client-to-server events
export interface CreateSessionPayload {
  replayId: string;
  nickname: string;
}

export interface JoinSessionPayload {
  sessionId: string;
  nickname: string;
}

export interface PlaybackUpdatePayload {
  timestamp?: number;
  speed?: number;
  paused?: boolean;
  seeking?: boolean;
}

export interface ChatSendPayload {
  text: string;
}

export interface FollowViewerPayload {
  targetId: string | null;
}

export interface TransferHostPayload {
  targetId: string;
}

export interface KickParticipantPayload {
  targetId: string;
}

export interface BanParticipantPayload {
  targetId: string;
}

export interface EnvironmentUpdatePayload {
  skyboxId?: string;
  exposure?: number;
  showHitboxes?: boolean;
  showBallSpeed?: boolean;
  showCarSpeed?: boolean;
  speedUnit?: SpeedUnit;
  customEnvironmentId?: string | null;
}

// Server-to-client events
export interface SessionStateEvent {
  session: Session;
  selfId: string;
}

export interface ParticipantJoinedEvent {
  participant: Participant;
}

export interface ParticipantLeftEvent {
  participantId: string;
  nickname: string;
}

export interface PlaybackSyncEvent {
  timestamp: number;
  speed: number;
  paused: boolean;
  serverTime: number;
  seeking?: boolean;
}

export interface CameraBroadcastEvent {
  participantId: string;
  camera: CameraState;
}

export interface ChatMessageEvent {
  message: ChatMessage;
}

export interface HostChangedEvent {
  newHostId: string;
  newHostNickname: string;
  previousHostId: string;
  reason: 'transfer' | 'disconnect';
}

export interface EnvironmentSyncEvent {
  environment: EnvironmentSettings;
}

export interface FollowStatusChangedEvent {
  participantId: string;
  followingId: string | null;
}

export interface ErrorEvent {
  code: string;
  message: string;
}

// Error codes
export type ErrorCode =
  | 'SESSION_NOT_FOUND'
  | 'SESSION_FULL'
  | 'NICKNAME_TAKEN'
  | 'INVALID_NICKNAME'
  | 'INVALID_REPLAY'
  | 'NOT_HOST'
  | 'PARTICIPANT_NOT_FOUND'
  | 'RATE_LIMITED'
  | 'INVALID_MESSAGE'
  | 'INVALID_SETTINGS'
  // Ping & Drawing error codes
  | 'INVALID_POSITION'
  | 'INVALID_COLOR'
  | 'INVALID_THICKNESS'
  | 'INVALID_STROKE_ID'
  | 'INVALID_START_POINT'
  | 'STROKE_NOT_FOUND'
  | 'NOTHING_TO_UNDO'
  | 'INVALID_STROKE_IDS'
  | 'TOO_MANY_STROKES';

// Response types
export interface CreateSessionResponse {
  success: boolean;
  sessionId?: string;
  shareUrl?: string;
  error?: {
    code: ErrorCode;
    message: string;
  };
}

export interface JoinSessionResponse {
  success: boolean;
  replayId?: string;
  error?: {
    code: ErrorCode;
    message: string;
  };
}

export interface GenericResponse {
  success: boolean;
  error?: {
    code: ErrorCode;
    message: string;
  };
}

// Collab configuration
export const COLLAB_CONFIG = {
  NAMESPACE: '/collab',
  MAX_PARTICIPANTS: 30,
  SESSION_TIMEOUT_MS: 30 * 60 * 1000, // 30 minutes
  RECONNECT_WINDOW_MS: 60 * 1000, // 1 minute
  CAMERA_UPDATE_INTERVAL_MS: 50, // 20 Hz
  PING_DURATION_MS: 5000, // 5 seconds
  MAX_STROKES: 100,
  MAX_UNDO_STACK: 50,
  DRAWING_BATCH_INTERVAL_MS: 50, // 20 Hz for drawing points
} as const;

// Tool types for collaborative drawing
export type ToolType = 'select' | 'ping' | 'draw' | 'eraser';

// Tool state (local, not synced)
export interface ToolState {
  activeTool: ToolType;
  drawColor: string;
  drawThickness: number;
}

// Default tool state
export const DEFAULT_TOOL_STATE: ToolState = {
  activeTool: 'select',
  drawColor: '#FF6B6B',
  drawThickness: 3,
};

// Ping marker on the terrain
export interface Ping {
  id: string;
  authorId: string;
  authorNickname: string;
  authorColor: string;
  position: Vector3;
  normal?: Vector3; // Surface normal for orientation
  createdAt: number;
  expiresAt: number;
}

// Drawing stroke on the terrain
export interface DrawingStroke {
  id: string;
  authorId: string;
  color: string;
  thickness: number;
  points: Vector3[];
  createdAt: number;
}

// Drawing state for session
export interface DrawingState {
  strokes: DrawingStroke[];
  activePings: Ping[];
}

// Client-to-server: Ping events
export interface PingPlacePayload {
  position: Vector3;
}

// Server-to-client: Ping events
export interface PingCreatedEvent {
  ping: Ping;
}

export interface PingExpiredEvent {
  pingId: string;
  authorId: string;
}

// Client-to-server: Drawing events
export interface DrawStrokeStartPayload {
  strokeId: string;
  color: string;
  thickness: number;
  startPoint: Vector3;
}

export interface DrawStrokeEndPayload {
  strokeId: string;
}

export interface DrawErasePayload {
  strokeIds: string[];
}

// Server-to-client: Drawing events
export interface DrawStrokeStartedEvent {
  stroke: DrawingStroke;
}

export interface DrawStrokeCompletedEvent {
  strokeId: string;
  authorId: string;
}

export interface DrawStrokeRemovedEvent {
  strokeId: string;
  reason: 'undo' | 'erase' | 'limit';
}

export interface DrawClearedEvent {
  clearedBy: string;
  clearedByNickname: string;
}

export interface DrawingStateSyncEvent {
  strokes: DrawingStroke[];
  activePings: Ping[];
}

// Drawing error codes
export type DrawErrorCode =
  | 'STROKE_NOT_FOUND'
  | 'INVALID_STROKE_ID'
  | 'INVALID_COLOR'
  | 'INVALID_THICKNESS'
  | 'POSITION_OUT_OF_BOUNDS'
  | 'NOT_HOST'
  | 'RATE_LIMITED'
  | 'MAX_STROKES_REACHED';

export interface DrawErrorEvent {
  code: DrawErrorCode;
  message: string;
  details?: Record<string, unknown>;
}
