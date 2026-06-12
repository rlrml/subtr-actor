import {
  createContext,
  useContext,
  useReducer,
  useEffect,
  useCallback,
  useRef,
  type ReactNode,
} from 'react';
import { toast } from 'sonner';
import { getCollabSocket, connectCollab, emitWithAck } from './socket';
import { decodeCameraBroadcast, decodeDrawStrokePointsBroadcast, isBinaryData, toUint8Array } from './proto';
import type {
  Participant,
  ChatMessage,
  PlaybackState,
  EnvironmentSettings,
  CameraState,
  CreateSessionPayload,
  CreateSessionResponse,
  JoinSessionPayload,
  JoinSessionResponse,
  GenericResponse,
  PlaybackUpdatePayload,
  ChatSendPayload,
  FollowViewerPayload,
  TransferHostPayload,
  KickParticipantPayload,
  BanParticipantPayload,
  EnvironmentUpdatePayload,
  SessionStateEvent,
  ParticipantJoinedEvent,
  ParticipantLeftEvent,
  PlaybackSyncEvent,
  ChatMessageEvent,
  HostChangedEvent,
  EnvironmentSyncEvent,
  ErrorEvent,
  Ping,
  DrawingStroke,
  PingCreatedEvent,
  PingExpiredEvent,
  DrawStrokeStartedEvent,
  DrawStrokeCompletedEvent,
  DrawStrokeRemovedEvent,
  DrawClearedEvent,
  DrawingStateSyncEvent,
  ToolType,
  ToolState,
} from './types';
import { DEFAULT_ENVIRONMENT, DEFAULT_TOOL_STATE } from './types';

// State interface
interface CollabState {
  isConnected: boolean;
  isInSession: boolean;
  sessionId: string | null;
  selfId: string | null;
  replayId: string | null;
  hostId: string | null;
  participants: Record<string, Participant>;
  chatMessages: ChatMessage[];
  playback: PlaybackState | null;
  environment: EnvironmentSettings;
  error: string | null;
  isLoading: boolean;
  // Ping & Drawing state
  activePings: Ping[];
  strokes: DrawingStroke[];
  toolState: ToolState;
}

// Action types
type CollabAction =
  | { type: 'SET_CONNECTED'; payload: boolean }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SESSION_JOINED'; payload: SessionStateEvent }
  | { type: 'SESSION_LEFT' }
  | { type: 'PARTICIPANT_JOINED'; payload: ParticipantJoinedEvent }
  | { type: 'PARTICIPANT_LEFT'; payload: ParticipantLeftEvent }
  | { type: 'PLAYBACK_SYNC'; payload: PlaybackSyncEvent }
  | { type: 'CAMERA_UPDATE'; payload: { participantId: string; camera: CameraState } }
  | { type: 'CHAT_MESSAGE'; payload: ChatMessageEvent }
  | { type: 'HOST_CHANGED'; payload: HostChangedEvent }
  | { type: 'ENVIRONMENT_SYNC'; payload: EnvironmentSyncEvent }
  | { type: 'FOLLOW_STATUS_CHANGED'; payload: { participantId: string; followingId: string | null } }
  // Ping & Drawing actions
  | { type: 'PING_CREATED'; payload: PingCreatedEvent }
  | { type: 'PING_EXPIRED'; payload: PingExpiredEvent }
  | { type: 'DRAWING_STATE_SYNC'; payload: DrawingStateSyncEvent }
  | { type: 'STROKE_STARTED'; payload: DrawStrokeStartedEvent }
  | { type: 'STROKE_COMPLETED'; payload: DrawStrokeCompletedEvent }
  | { type: 'STROKE_REMOVED'; payload: DrawStrokeRemovedEvent }
  | { type: 'DRAWINGS_CLEARED'; payload: DrawClearedEvent }
  | { type: 'SET_TOOL'; payload: ToolType }
  | { type: 'SET_DRAW_COLOR'; payload: string }
  | { type: 'SET_DRAW_THICKNESS'; payload: number };

// Initial state
const initialState: CollabState = {
  isConnected: false,
  isInSession: false,
  sessionId: null,
  selfId: null,
  replayId: null,
  hostId: null,
  participants: {},
  chatMessages: [],
  playback: null,
  environment: DEFAULT_ENVIRONMENT,
  error: null,
  isLoading: false,
  // Ping & Drawing initial state
  activePings: [],
  strokes: [],
  toolState: DEFAULT_TOOL_STATE,
};

// Reducer
function collabReducer(state: CollabState, action: CollabAction): CollabState {
  switch (action.type) {
    case 'SET_CONNECTED':
      return { ...state, isConnected: action.payload };

    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };

    case 'SET_ERROR':
      return { ...state, error: action.payload, isLoading: false };

    case 'SESSION_JOINED': {
      const { session, selfId } = action.payload;
      // Convert participants array to record if needed
      const participants: Record<string, Participant> =
        Array.isArray(session.participants)
          ? session.participants.reduce((acc, p) => ({ ...acc, [p.id]: p }), {})
          : session.participants;

      // Extract strokes and pings from drawingState if present
      const strokes = session.drawingState?.strokes || [];
      const activePings = session.drawingState?.activePings || [];

      return {
        ...state,
        isInSession: true,
        sessionId: session.id,
        selfId,
        replayId: session.replayId,
        hostId: session.hostId,
        participants,
        chatMessages: session.chatHistory || [],
        playback: session.playback,
        environment: session.environment || DEFAULT_ENVIRONMENT,
        error: null,
        isLoading: false,
        // Sync drawing state from session
        strokes,
        activePings,
      };
    }

    case 'SESSION_LEFT':
      return {
        ...initialState,
        isConnected: state.isConnected,
      };

    case 'PARTICIPANT_JOINED':
      return {
        ...state,
        participants: {
          ...state.participants,
          [action.payload.participant.id]: action.payload.participant,
        },
      };

    case 'PARTICIPANT_LEFT': {
      const { [action.payload.participantId]: _, ...remaining } = state.participants;
      return {
        ...state,
        participants: remaining,
      };
    }

    case 'PLAYBACK_SYNC':
      return {
        ...state,
        playback: {
          timestamp: action.payload.timestamp,
          speed: action.payload.speed,
          paused: action.payload.paused,
          lastSyncAt: action.payload.serverTime,
        },
      };

    case 'CAMERA_UPDATE': {
      const { participantId, camera } = action.payload;
      if (!state.participants[participantId]) return state;
      return {
        ...state,
        participants: {
          ...state.participants,
          [participantId]: {
            ...state.participants[participantId],
            camera,
          },
        },
      };
    }

    case 'CHAT_MESSAGE':
      return {
        ...state,
        chatMessages: [...state.chatMessages, action.payload.message].slice(-100),
      };

    case 'HOST_CHANGED':
      return {
        ...state,
        hostId: action.payload.newHostId,
        participants: Object.fromEntries(
          Object.entries(state.participants).map(([id, p]) => [
            id,
            {
              ...p,
              role: id === action.payload.newHostId ? 'host' : 'viewer',
            } as Participant,
          ])
        ),
      };

    case 'ENVIRONMENT_SYNC':
      return {
        ...state,
        environment: action.payload.environment,
      };

    case 'FOLLOW_STATUS_CHANGED': {
      const { participantId, followingId } = action.payload;
      const participant = state.participants[participantId];
      if (!participant) return state;
      return {
        ...state,
        participants: {
          ...state.participants,
          [participantId]: {
            ...participant,
            followingId,
          },
        },
      };
    }

    // Ping actions
    case 'PING_CREATED': {
      const { ping } = action.payload;
      // Replace existing ping from same author, keep max 20 pings
      const filtered = state.activePings.filter((p) => p.authorId !== ping.authorId);
      return {
        ...state,
        activePings: [...filtered, ping].slice(-20),
      };
    }

    case 'PING_EXPIRED': {
      const { pingId } = action.payload;
      return {
        ...state,
        activePings: state.activePings.filter((p) => p.id !== pingId),
      };
    }

    // Drawing actions
    case 'DRAWING_STATE_SYNC': {
      const { strokes } = action.payload;
      return {
        ...state,
        strokes,
      };
    }

    case 'STROKE_STARTED': {
      const { stroke } = action.payload;
      // Add new stroke (replace if same ID exists)
      const filtered = state.strokes.filter((s) => s.id !== stroke.id);
      return {
        ...state,
        strokes: [...filtered, stroke].slice(-100), // FIFO limit
      };
    }

    case 'STROKE_COMPLETED': {
      const { strokeId } = action.payload;
      // Mark stroke as completed (update in place)
      return {
        ...state,
        strokes: state.strokes.map((s) =>
          s.id === strokeId ? { ...s, completed: true } : s
        ),
      };
    }

    case 'STROKE_REMOVED': {
      const { strokeId } = action.payload;
      return {
        ...state,
        strokes: state.strokes.filter((s) => s.id !== strokeId),
      };
    }

    case 'DRAWINGS_CLEARED':
      return {
        ...state,
        strokes: [],
      };

    // Tool state actions
    case 'SET_TOOL':
      return {
        ...state,
        toolState: { ...state.toolState, activeTool: action.payload },
      };

    case 'SET_DRAW_COLOR':
      return {
        ...state,
        toolState: { ...state.toolState, drawColor: action.payload },
      };

    case 'SET_DRAW_THICKNESS':
      return {
        ...state,
        toolState: { ...state.toolState, drawThickness: action.payload },
      };

    default:
      return state;
  }
}

// Orbit params type (from proto.ts)
interface OrbitParams {
  distance: number;
  azimuth: number;
  polar: number;
}

// Camera update callback type - includes optional orbitParams for ballOrbit mode
type CameraUpdateCallback = (participantId: string, camera: CameraState, orbitParams?: OrbitParams | null) => void;

// Participant left callback type
type ParticipantLeftCallback = (participantId: string) => void;

// Follow status changed callback type
type FollowStatusChangedCallback = (participantId: string, followingId: string | null) => void;

// Ping callback types
type PingCreatedCallback = (ping: Ping) => void;
type PingExpiredCallback = (pingId: string) => void;

// Drawing callback types
type StrokeStartedCallback = (stroke: DrawingStroke) => void;
type StrokePointsCallback = (strokeId: string, authorId: string, points: Array<{ x: number; y: number; z: number }>) => void;
type StrokeCompletedCallback = (strokeId: string, authorId: string) => void;
type StrokeRemovedCallback = (strokeId: string, reason: string) => void;
type DrawingsClearedCallback = (clearedBy: string, clearedByNickname: string) => void;

// Context types
interface CollabContextValue {
  state: CollabState;
  actions: {
    createSession: (replayId: string, nickname: string) => Promise<CreateSessionResponse>;
    joinSession: (sessionId: string, nickname: string) => Promise<JoinSessionResponse>;
    leaveSession: () => void;
    updatePlayback: (payload: PlaybackUpdatePayload) => Promise<GenericResponse>;
    sendChat: (text: string) => Promise<GenericResponse>;
    followViewer: (targetId: string | null) => Promise<GenericResponse>;
    transferHost: (targetId: string) => Promise<GenericResponse>;
    kickParticipant: (targetId: string) => Promise<GenericResponse>;
    banParticipant: (targetId: string) => Promise<GenericResponse>;
    updateEnvironment: (payload: EnvironmentUpdatePayload) => Promise<GenericResponse>;
    clearError: () => void;
    onCameraUpdate: (callback: CameraUpdateCallback) => () => void;
    onParticipantLeft: (callback: ParticipantLeftCallback) => () => void;
    onFollowStatusChanged: (callback: FollowStatusChangedCallback) => () => void;
    // Ping & Drawing actions
    placePing: (position: { x: number; y: number; z: number }, normal?: { x: number; y: number; z: number }) => Promise<GenericResponse>;
    setTool: (tool: ToolType) => void;
    setDrawColor: (color: string) => void;
    setDrawThickness: (thickness: number) => void;
    startStroke: (strokeId: string, color: string, thickness: number, startPoint: { x: number; y: number; z: number }) => Promise<GenericResponse>;
    sendStrokePoints: (strokeId: string, points: Array<{ x: number; y: number; z: number }>) => void;
    endStroke: (strokeId: string) => Promise<GenericResponse>;
    undoStroke: () => Promise<GenericResponse>;
    eraseStrokes: (strokeIds: string[]) => Promise<GenericResponse>;
    clearAllDrawings: () => Promise<GenericResponse>;
    // Ping & Drawing callbacks
    onPingCreated: (callback: PingCreatedCallback) => () => void;
    onPingExpired: (callback: PingExpiredCallback) => () => void;
    onStrokeStarted: (callback: StrokeStartedCallback) => () => void;
    onStrokePoints: (callback: StrokePointsCallback) => () => void;
    onStrokeCompleted: (callback: StrokeCompletedCallback) => () => void;
    onStrokeRemoved: (callback: StrokeRemovedCallback) => () => void;
    onDrawingsCleared: (callback: DrawingsClearedCallback) => () => void;
  };
}

// Create context
const CollabContext = createContext<CollabContextValue | null>(null);

// Provider component
interface CollabProviderProps {
  children: ReactNode;
}

// Key for pending session data in sessionStorage
const PENDING_SESSION_KEY = 'collab-session';

export function CollabProvider({ children }: CollabProviderProps) {
  const [state, dispatch] = useReducer(collabReducer, initialState);
  const socketRef = useRef<ReturnType<typeof getCollabSocket> | null>(null);
  const pendingJoinHandled = useRef(false);
  const cameraUpdateCallbacksRef = useRef<Set<CameraUpdateCallback>>(new Set());
  const participantLeftCallbacksRef = useRef<Set<ParticipantLeftCallback>>(new Set());
  const followStatusChangedCallbacksRef = useRef<Set<FollowStatusChangedCallback>>(new Set());
  // Ping & Drawing callback refs
  const pingCreatedCallbacksRef = useRef<Set<PingCreatedCallback>>(new Set());
  const pingExpiredCallbacksRef = useRef<Set<PingExpiredCallback>>(new Set());
  const strokeStartedCallbacksRef = useRef<Set<StrokeStartedCallback>>(new Set());
  const strokePointsCallbacksRef = useRef<Set<StrokePointsCallback>>(new Set());
  const strokeCompletedCallbacksRef = useRef<Set<StrokeCompletedCallback>>(new Set());
  const strokeRemovedCallbacksRef = useRef<Set<StrokeRemovedCallback>>(new Set());
  const drawingsClearedCallbacksRef = useRef<Set<DrawingsClearedCallback>>(new Set());
  // Keep participants in a ref for access in socket handlers (bypasses stale closure)
  const participantsRef = useRef<Record<string, Participant>>({});
  // Keep selfId in a ref for access in socket handlers
  const selfIdRef = useRef<string | null>(null);

  // Setup socket event listeners
  useEffect(() => {
    const socket = getCollabSocket();
    socketRef.current = socket;

    // Connection events
    socket.on('connect', () => {
      console.log('[CollabProvider] Socket connected');
      dispatch({ type: 'SET_CONNECTED', payload: true });
    });

    socket.on('disconnect', () => {
      console.log('[CollabProvider] Socket disconnected');
      dispatch({ type: 'SET_CONNECTED', payload: false });
    });

    // Check if already connected (in case connect event fired before listener attached)
    if (socket.connected) {
      console.log('[CollabProvider] Socket already connected');
      dispatch({ type: 'SET_CONNECTED', payload: true });
    }

    // Session events
    socket.on('session-state', (data: SessionStateEvent) => {
      console.log('[CollabProvider] session-state received, drawingState:', data.session.drawingState);
      dispatch({ type: 'SESSION_JOINED', payload: data });
    });

    socket.on('participant-joined', (data: ParticipantJoinedEvent) => {
      dispatch({ type: 'PARTICIPANT_JOINED', payload: data });
      // Show toast notification with unique ID to prevent duplicates
      const { participant } = data;
      toast(
        <div className="flex items-center gap-2">
          <div
            className="w-3 h-3 rounded-full flex-shrink-0"
            style={{ backgroundColor: participant.color }}
          />
          <span>
            <span className="font-semibold" style={{ color: participant.color }}>
              {participant.nickname}
            </span>
            {' '}joined the session
          </span>
        </div>,
        {
          id: `join-${participant.id}`,
          duration: 3000,
        }
      );
    });

    socket.on('participant-left', (data: ParticipantLeftEvent) => {
      // Get participant color before removing from state (for toast)
      // We access participantsRef which is kept in sync via useEffect
      const participantColor = participantsRef.current[data.participantId]?.color || '#9CA3AF';

      dispatch({ type: 'PARTICIPANT_LEFT', payload: data });

      // Show toast notification with unique ID to prevent duplicates
      toast(
        <div className="flex items-center gap-2">
          <div
            className="w-3 h-3 rounded-full flex-shrink-0"
            style={{ backgroundColor: participantColor }}
          />
          <span>
            <span className="font-semibold" style={{ color: participantColor }}>
              {data.nickname}
            </span>
            {' '}left the session
          </span>
        </div>,
        {
          id: `leave-${data.participantId}`,
          duration: 3000,
        }
      );
      // Call direct callbacks for immediate cleanup (bypasses React)
      console.log(`[CollabProvider] Participant left: ${data.participantId}, calling ${participantLeftCallbacksRef.current.size} callbacks`);
      participantLeftCallbacksRef.current.forEach((callback) => {
        try {
          callback(data.participantId);
        } catch (e) {
          console.error('[CollabProvider] Participant left callback error:', e);
        }
      });
    });

    // Playback events
    socket.on('playback-sync', (data: PlaybackSyncEvent) => {
      dispatch({ type: 'PLAYBACK_SYNC', payload: data });
    });

    // Camera events (binary)
    socket.on('camera-broadcast', (data: unknown) => {
      if (isBinaryData(data)) {
        const result = decodeCameraBroadcast(toUint8Array(data));
        if (result) {
          dispatch({ type: 'CAMERA_UPDATE', payload: { participantId: result.participantId, camera: result.camera } });
          // Also call direct callbacks for immediate updates (bypasses React)
          // Pass orbitParams separately for ballOrbit mode
          const { participantId, camera, orbitParams } = result;
          cameraUpdateCallbacksRef.current.forEach((callback) => {
            try {
              callback(participantId, camera, orbitParams);
            } catch (e) {
              console.error('[CollabProvider] Camera update callback error:', e);
            }
          });
        }
      }
    });

    // Chat events
    socket.on('chat-message', (data: ChatMessageEvent) => {
      dispatch({ type: 'CHAT_MESSAGE', payload: data });
    });

    // Host events
    socket.on('host-changed', (data: HostChangedEvent) => {
      dispatch({ type: 'HOST_CHANGED', payload: data });

      // Show toast notification for host change
      const isNewHostSelf = data.newHostId === selfIdRef.current;

      if (isNewHostSelf) {
        // You are the new host
        toast(
          <div className="flex items-center gap-2">
            <span className="text-yellow-400">👑</span>
            <span>
              You are now the host
              {data.reason === 'disconnect' && ' (previous host left)'}
            </span>
          </div>,
          {
            id: `host-changed-${data.newHostId}`,
            duration: 4000,
          }
        );
      } else {
        // Someone else is the new host
        const newHostColor = participantsRef.current[data.newHostId]?.color || '#9CA3AF';
        toast(
          <div className="flex items-center gap-2">
            <span className="text-yellow-400">👑</span>
            <span>
              <span className="font-semibold" style={{ color: newHostColor }}>
                {data.newHostNickname}
              </span>
              {' '}is now the host
            </span>
          </div>,
          {
            id: `host-changed-${data.newHostId}`,
            duration: 4000,
          }
        );
      }
    });

    // Environment events
    socket.on('environment-sync', (data: EnvironmentSyncEvent) => {
      dispatch({ type: 'ENVIRONMENT_SYNC', payload: data });
    });

    // Follow status events
    socket.on('follow-status-changed', (data: { participantId: string; followingId: string | null }) => {
      console.log(`[CollabProvider] *** FOLLOW EVENT: ${data.participantId.slice(0,8)} -> ${data.followingId ? 'following' : 'free'}`);
      dispatch({ type: 'FOLLOW_STATUS_CHANGED', payload: data });
      // Call direct callbacks for immediate updates (bypasses React)
      const { participantId, followingId } = data;
      followStatusChangedCallbacksRef.current.forEach((callback) => {
        try {
          callback(participantId, followingId);
        } catch (e) {
          console.error('[CollabProvider] Follow status callback error:', e);
        }
      });
    });

    // Ping events
    socket.on('ping-created', (data: PingCreatedEvent) => {
      console.log('[CollabProvider] Ping created:', data.ping.id);
      dispatch({ type: 'PING_CREATED', payload: data });
      // Call direct callbacks for immediate 3D updates
      pingCreatedCallbacksRef.current.forEach((callback) => {
        try {
          callback(data.ping);
        } catch (e) {
          console.error('[CollabProvider] Ping created callback error:', e);
        }
      });
    });

    socket.on('ping-expired', (data: PingExpiredEvent) => {
      console.log('[CollabProvider] Ping expired:', data.pingId);
      dispatch({ type: 'PING_EXPIRED', payload: data });
      // Call direct callbacks for immediate 3D cleanup
      pingExpiredCallbacksRef.current.forEach((callback) => {
        try {
          callback(data.pingId);
        } catch (e) {
          console.error('[CollabProvider] Ping expired callback error:', e);
        }
      });
    });

    // Drawing events
    socket.on('draw-stroke-started', (data: DrawStrokeStartedEvent) => {
      console.log('[CollabProvider] Stroke started:', data.stroke.id);
      dispatch({ type: 'STROKE_STARTED', payload: data });
      // Call direct callbacks for immediate 3D rendering
      strokeStartedCallbacksRef.current.forEach((callback) => {
        try {
          callback(data.stroke);
        } catch (e) {
          console.error('[CollabProvider] Stroke started callback error:', e);
        }
      });
    });

    // Drawing points (binary)
    socket.on('draw-stroke-points', (data: unknown) => {
      if (isBinaryData(data)) {
        const result = decodeDrawStrokePointsBroadcast(toUint8Array(data));
        if (result) {
          // Call direct callbacks for immediate 3D rendering (no state update needed for points)
          strokePointsCallbacksRef.current.forEach((callback) => {
            try {
              callback(result.strokeId, result.authorId, result.points);
            } catch (e) {
              console.error('[CollabProvider] Stroke points callback error:', e);
            }
          });
        }
      }
    });

    socket.on('draw-stroke-completed', (data: DrawStrokeCompletedEvent) => {
      console.log('[CollabProvider] Stroke completed:', data.strokeId);
      dispatch({ type: 'STROKE_COMPLETED', payload: data });
      // Call direct callbacks
      strokeCompletedCallbacksRef.current.forEach((callback) => {
        try {
          callback(data.strokeId, data.authorId);
        } catch (e) {
          console.error('[CollabProvider] Stroke completed callback error:', e);
        }
      });
    });

    socket.on('draw-stroke-removed', (data: DrawStrokeRemovedEvent) => {
      console.log('[CollabProvider] Stroke removed:', data.strokeId, data.reason);
      dispatch({ type: 'STROKE_REMOVED', payload: data });
      // Call direct callbacks for immediate 3D cleanup
      strokeRemovedCallbacksRef.current.forEach((callback) => {
        try {
          callback(data.strokeId, data.reason);
        } catch (e) {
          console.error('[CollabProvider] Stroke removed callback error:', e);
        }
      });
    });

    socket.on('draw-cleared', (data: DrawClearedEvent) => {
      console.log('[CollabProvider] Drawings cleared by:', data.clearedByNickname);
      dispatch({ type: 'DRAWINGS_CLEARED', payload: data });
      // Call direct callbacks for immediate 3D cleanup
      drawingsClearedCallbacksRef.current.forEach((callback) => {
        try {
          callback(data.clearedBy, data.clearedByNickname);
        } catch (e) {
          console.error('[CollabProvider] Drawings cleared callback error:', e);
        }
      });
      // Show toast
      toast(`${data.clearedByNickname} cleared all drawings`, { duration: 3000 });
    });

    // Error events
    socket.on('error', (data: ErrorEvent) => {
      dispatch({ type: 'SET_ERROR', payload: data.message });
    });

    // Kicked event
    socket.on('kicked', (data: { reason: string }) => {
      console.log('[CollabProvider] Kicked from session:', data.reason);
      dispatch({ type: 'SESSION_LEFT' });
      toast.error(data.reason || 'You have been kicked from the session', { duration: 5000 });
    });

    // Banned event
    socket.on('banned', (data: { reason: string }) => {
      console.log('[CollabProvider] Banned from session:', data.reason);
      dispatch({ type: 'SESSION_LEFT' });
      toast.error(data.reason || 'You have been banned from the session', { duration: 5000 });
    });

    // Connect
    connectCollab();

    // Note: We don't disconnect the socket on cleanup because:
    // 1. React StrictMode remounts components, causing unwanted disconnects
    // 2. The socket should persist across navigation
    // 3. The socket will be cleaned up when the tab/window closes
    return () => {
      socket.off('connect');
      socket.off('disconnect');
      socket.off('session-state');
      socket.off('participant-joined');
      socket.off('participant-left');
      socket.off('playback-sync');
      socket.off('camera-broadcast');
      socket.off('chat-message');
      socket.off('host-changed');
      socket.off('environment-sync');
      socket.off('follow-status-changed');
      socket.off('ping-created');
      socket.off('ping-expired');
      socket.off('draw-stroke-started');
      socket.off('draw-stroke-points');
      socket.off('draw-stroke-completed');
      socket.off('draw-stroke-removed');
      socket.off('draw-cleared');
      socket.off('error');
      socket.off('kicked');
      socket.off('banned');
      // Don't call disconnectCollab() here - let socket persist
    };
  }, []);

  // Auto-join pending session from sessionStorage (set by JoinSessionPage)
  useEffect(() => {
    // Only try once and only when connected
    if (!state.isConnected || pendingJoinHandled.current || state.isInSession) {
      return;
    }

    try {
      const pendingData = sessionStorage.getItem(PENDING_SESSION_KEY);
      if (!pendingData) return;

      const { sessionId, nickname } = JSON.parse(pendingData) as {
        sessionId: string;
        nickname: string;
        replayId: string;
      };

      if (sessionId && nickname) {
        console.log('[CollabProvider] Found pending session, joining:', sessionId);
        pendingJoinHandled.current = true;

        // Clear the pending data
        sessionStorage.removeItem(PENDING_SESSION_KEY);

        // Emit join-session
        dispatch({ type: 'SET_LOADING', payload: true });
        const payload: JoinSessionPayload = { sessionId, nickname };

        socketRef.current?.emit('join-session', payload, (response: JoinSessionResponse) => {
          if (!response.success) {
            dispatch({ type: 'SET_ERROR', payload: response.error?.message || 'Failed to join session' });
          }
          // If success, the server will send session-state event which will update our state
        });
      }
    } catch (e) {
      console.warn('[CollabProvider] Failed to parse pending session data:', e);
      sessionStorage.removeItem(PENDING_SESSION_KEY);
    }
  }, [state.isConnected, state.isInSession]);

  // Keep participantsRef in sync with state for socket handlers
  useEffect(() => {
    participantsRef.current = state.participants;
  }, [state.participants]);

  // Keep selfIdRef in sync with state for socket handlers
  useEffect(() => {
    selfIdRef.current = state.selfId;
  }, [state.selfId]);

  // Keep isInSession in a ref for cleanup (to access current value in useEffect cleanup)
  const isInSessionRef = useRef(false);
  useEffect(() => {
    isInSessionRef.current = state.isInSession;
  }, [state.isInSession]);

  // Leave session automatically when component unmounts (e.g., navigating away)
  // Also handle page close/refresh via beforeunload
  useEffect(() => {
    const handleBeforeUnload = () => {
      if (isInSessionRef.current && socketRef.current) {
        console.log('[CollabProvider] Page unloading while in session, sending leave-session');
        socketRef.current.emit('leave-session');
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);

    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
      if (isInSessionRef.current && socketRef.current) {
        console.log('[CollabProvider] Unmounting while in session, sending leave-session');
        socketRef.current.emit('leave-session');
      }
    };
  }, []);

  // Set dispatch ref for external camera updates (used by useCollab)
  useEffect(() => {
    setDispatchRef(dispatch, () => state.selfId);
  }, [state.selfId]);

  // Actions
  const createSession = useCallback(
    async (replayId: string, nickname: string): Promise<CreateSessionResponse> => {
      dispatch({ type: 'SET_LOADING', payload: true });
      try {
        const payload: CreateSessionPayload = { replayId, nickname };
        const response = await emitWithAck<CreateSessionResponse>('create-session', payload);
        if (!response.success) {
          dispatch({ type: 'SET_ERROR', payload: response.error?.message || 'Failed to create session' });
        }
        return response;
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to create session';
        dispatch({ type: 'SET_ERROR', payload: message });
        return { success: false, error: { code: 'SESSION_NOT_FOUND', message } };
      }
    },
    []
  );

  const joinSession = useCallback(
    async (sessionId: string, nickname: string): Promise<JoinSessionResponse> => {
      dispatch({ type: 'SET_LOADING', payload: true });
      try {
        const payload: JoinSessionPayload = { sessionId, nickname };
        const response = await emitWithAck<JoinSessionResponse>('join-session', payload);
        if (!response.success) {
          dispatch({ type: 'SET_ERROR', payload: response.error?.message || 'Failed to join session' });
        }
        return response;
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to join session';
        dispatch({ type: 'SET_ERROR', payload: message });
        return { success: false, error: { code: 'SESSION_NOT_FOUND', message } };
      }
    },
    []
  );

  const leaveSession = useCallback(() => {
    socketRef.current?.emit('leave-session');
    dispatch({ type: 'SESSION_LEFT' });
  }, []);

  const updatePlayback = useCallback(
    async (payload: PlaybackUpdatePayload): Promise<GenericResponse> => {
      try {
        return await emitWithAck<GenericResponse>('playback-update', payload);
      } catch (error) {
        return { success: false, error: { code: 'NOT_HOST', message: 'Failed to update playback' } };
      }
    },
    []
  );

  const sendChat = useCallback(async (text: string): Promise<GenericResponse> => {
    try {
      const payload: ChatSendPayload = { text };
      return await emitWithAck<GenericResponse>('chat-send', payload);
    } catch (error) {
      return { success: false, error: { code: 'RATE_LIMITED', message: 'Failed to send message' } };
    }
  }, []);

  const followViewer = useCallback(
    async (targetId: string | null): Promise<GenericResponse> => {
      console.log('[CollabProvider] *** followViewer called with targetId:', targetId);
      try {
        const payload: FollowViewerPayload = { targetId };
        const response = await emitWithAck<GenericResponse>('follow-viewer', payload);
        console.log('[CollabProvider] *** followViewer response:', response);
        return response;
      } catch (error) {
        console.error('[CollabProvider] *** followViewer error:', error);
        return { success: false, error: { code: 'PARTICIPANT_NOT_FOUND', message: 'Failed to follow' } };
      }
    },
    []
  );

  const transferHost = useCallback(
    async (targetId: string): Promise<GenericResponse> => {
      try {
        const payload: TransferHostPayload = { targetId };
        return await emitWithAck<GenericResponse>('transfer-host', payload);
      } catch (error) {
        return { success: false, error: { code: 'NOT_HOST', message: 'Failed to transfer host' } };
      }
    },
    []
  );

  const kickParticipant = useCallback(
    async (targetId: string): Promise<GenericResponse> => {
      try {
        const payload: KickParticipantPayload = { targetId };
        return await emitWithAck<GenericResponse>('kick-participant', payload);
      } catch (error) {
        return { success: false, error: { code: 'NOT_HOST', message: 'Failed to kick participant' } };
      }
    },
    []
  );

  const banParticipant = useCallback(
    async (targetId: string): Promise<GenericResponse> => {
      try {
        const payload: BanParticipantPayload = { targetId };
        return await emitWithAck<GenericResponse>('ban-participant', payload);
      } catch (error) {
        return { success: false, error: { code: 'NOT_HOST', message: 'Failed to ban participant' } };
      }
    },
    []
  );

  const updateEnvironment = useCallback(
    async (payload: EnvironmentUpdatePayload): Promise<GenericResponse> => {
      try {
        return await emitWithAck<GenericResponse>('environment-update', payload);
      } catch (error) {
        return { success: false, error: { code: 'NOT_HOST', message: 'Failed to update environment' } };
      }
    },
    []
  );

  const clearError = useCallback(() => {
    dispatch({ type: 'SET_ERROR', payload: null });
  }, []);

  // Register a callback for camera updates (bypasses React for performance)
  const onCameraUpdate = useCallback((callback: CameraUpdateCallback) => {
    cameraUpdateCallbacksRef.current.add(callback);
    return () => {
      cameraUpdateCallbacksRef.current.delete(callback);
    };
  }, []);

  // Register a callback for participant left events (bypasses React for immediate cleanup)
  const onParticipantLeft = useCallback((callback: ParticipantLeftCallback) => {
    participantLeftCallbacksRef.current.add(callback);
    return () => {
      participantLeftCallbacksRef.current.delete(callback);
    };
  }, []);

  // Register a callback for follow status changed events (bypasses React for immediate visibility updates)
  const onFollowStatusChanged = useCallback((callback: FollowStatusChangedCallback) => {
    followStatusChangedCallbacksRef.current.add(callback);
    return () => {
      followStatusChangedCallbacksRef.current.delete(callback);
    };
  }, []);

  // Ping action
  const placePing = useCallback(
    async (position: { x: number; y: number; z: number }, normal?: { x: number; y: number; z: number }): Promise<GenericResponse> => {
      try {
        return await emitWithAck<GenericResponse>('ping-place', { position, normal });
      } catch (error) {
        return { success: false, error: { code: 'RATE_LIMITED', message: 'Failed to place ping' } };
      }
    },
    []
  );

  // Tool state actions
  const setTool = useCallback((tool: ToolType) => {
    dispatch({ type: 'SET_TOOL', payload: tool });
  }, []);

  const setDrawColor = useCallback((color: string) => {
    dispatch({ type: 'SET_DRAW_COLOR', payload: color });
  }, []);

  const setDrawThickness = useCallback((thickness: number) => {
    dispatch({ type: 'SET_DRAW_THICKNESS', payload: thickness });
  }, []);

  // Drawing actions
  const startStroke = useCallback(
    async (
      strokeId: string,
      color: string,
      thickness: number,
      startPoint: { x: number; y: number; z: number }
    ): Promise<GenericResponse> => {
      console.log('[CollabProvider] startStroke called:', strokeId, 'socket connected:', socketRef.current?.connected);
      try {
        const result = await emitWithAck<GenericResponse>('draw-stroke-start', {
          strokeId,
          color,
          thickness,
          startPoint,
        });
        console.log('[CollabProvider] startStroke result:', result);

        // Add stroke to local state on success (server only broadcasts to others)
        if (result.success && state.selfId) {
          const stroke: DrawingStroke = {
            id: strokeId,
            authorId: state.selfId,
            color,
            thickness,
            points: [startPoint],
            createdAt: Date.now(),
          };
          dispatch({ type: 'STROKE_STARTED', payload: { stroke } });
        }

        return result;
      } catch (error) {
        console.error('[CollabProvider] startStroke error:', error);
        return { success: false, error: { code: 'RATE_LIMITED', message: 'Failed to start stroke' } };
      }
    },
    [state.selfId]
  );

  const sendStrokePoints = useCallback(
    (strokeId: string, points: Array<{ x: number; y: number; z: number }>) => {
      // Send binary encoded points for efficiency
      // Import encodeDrawStrokePoints from proto and use it
      import('./proto').then(({ encodeDrawStrokePoints }) => {
        const encoded = encodeDrawStrokePoints(strokeId, points);
        socketRef.current?.emit('draw-stroke-points', encoded);
      });
    },
    []
  );

  const endStroke = useCallback(
    async (strokeId: string): Promise<GenericResponse> => {
      try {
        return await emitWithAck<GenericResponse>('draw-stroke-end', { strokeId });
      } catch (error) {
        return { success: false, error: { code: 'STROKE_NOT_FOUND', message: 'Failed to end stroke' } };
      }
    },
    []
  );

  const undoStroke = useCallback(async (): Promise<GenericResponse> => {
    try {
      return await emitWithAck<GenericResponse>('draw-undo', {});
    } catch (error) {
      return { success: false, error: { code: 'NOTHING_TO_UNDO', message: 'Nothing to undo' } };
    }
  }, []);

  const eraseStrokes = useCallback(
    async (strokeIds: string[]): Promise<GenericResponse> => {
      try {
        return await emitWithAck<GenericResponse>('draw-erase', { strokeIds });
      } catch (error) {
        return { success: false, error: { code: 'INVALID_STROKE_IDS', message: 'Failed to erase strokes' } };
      }
    },
    []
  );

  const clearAllDrawings = useCallback(async (): Promise<GenericResponse> => {
    try {
      return await emitWithAck<GenericResponse>('draw-clear-all', {});
    } catch (error) {
      return { success: false, error: { code: 'NOT_HOST', message: 'Only host can clear all drawings' } };
    }
  }, []);

  // Ping callback registrations
  const onPingCreated = useCallback((callback: PingCreatedCallback) => {
    pingCreatedCallbacksRef.current.add(callback);
    return () => {
      pingCreatedCallbacksRef.current.delete(callback);
    };
  }, []);

  const onPingExpired = useCallback((callback: PingExpiredCallback) => {
    pingExpiredCallbacksRef.current.add(callback);
    return () => {
      pingExpiredCallbacksRef.current.delete(callback);
    };
  }, []);

  // Drawing callback registrations
  const onStrokeStarted = useCallback((callback: StrokeStartedCallback) => {
    strokeStartedCallbacksRef.current.add(callback);
    return () => {
      strokeStartedCallbacksRef.current.delete(callback);
    };
  }, []);

  const onStrokePoints = useCallback((callback: StrokePointsCallback) => {
    strokePointsCallbacksRef.current.add(callback);
    return () => {
      strokePointsCallbacksRef.current.delete(callback);
    };
  }, []);

  const onStrokeCompleted = useCallback((callback: StrokeCompletedCallback) => {
    strokeCompletedCallbacksRef.current.add(callback);
    return () => {
      strokeCompletedCallbacksRef.current.delete(callback);
    };
  }, []);

  const onStrokeRemoved = useCallback((callback: StrokeRemovedCallback) => {
    strokeRemovedCallbacksRef.current.add(callback);
    return () => {
      strokeRemovedCallbacksRef.current.delete(callback);
    };
  }, []);

  const onDrawingsCleared = useCallback((callback: DrawingsClearedCallback) => {
    drawingsClearedCallbacksRef.current.add(callback);
    return () => {
      drawingsClearedCallbacksRef.current.delete(callback);
    };
  }, []);

  const value: CollabContextValue = {
    state,
    actions: {
      createSession,
      joinSession,
      leaveSession,
      updatePlayback,
      sendChat,
      followViewer,
      transferHost,
      kickParticipant,
      banParticipant,
      updateEnvironment,
      clearError,
      onCameraUpdate,
      onParticipantLeft,
      onFollowStatusChanged,
      // Ping & Drawing actions
      placePing,
      setTool,
      setDrawColor,
      setDrawThickness,
      startStroke,
      sendStrokePoints,
      endStroke,
      undoStroke,
      eraseStrokes,
      clearAllDrawings,
      // Ping & Drawing callbacks
      onPingCreated,
      onPingExpired,
      onStrokeStarted,
      onStrokePoints,
      onStrokeCompleted,
      onStrokeRemoved,
      onDrawingsCleared,
    },
  };

  return <CollabContext.Provider value={value}>{children}</CollabContext.Provider>;
}

// Hook to use collab context
export function useCollabContext(): CollabContextValue {
  const context = useContext(CollabContext);
  if (!context) {
    throw new Error('useCollabContext must be used within a CollabProvider');
  }
  return context;
}

// Dispatch helper for local camera updates (exported for useCollab to update self camera)
// This is a workaround because server broadcasts exclude the sender
let dispatchRef: React.Dispatch<CollabAction> | null = null;
let selfIdGetter: (() => string | null) | null = null;

export function setDispatchRef(dispatch: React.Dispatch<CollabAction>, getSelfId: () => string | null) {
  dispatchRef = dispatch;
  selfIdGetter = getSelfId;
}

export function dispatchCameraUpdate(camera: CameraState) {
  if (dispatchRef && selfIdGetter) {
    const selfId = selfIdGetter();
    if (selfId) {
      dispatchRef({ type: 'CAMERA_UPDATE', payload: { participantId: selfId, camera } });
    }
  }
}
