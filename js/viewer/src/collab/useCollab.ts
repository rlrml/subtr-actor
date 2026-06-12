import { useMemo, useCallback, useRef, useEffect } from 'react';
import { useCollabContext, dispatchCameraUpdate } from './CollabProvider';
import { emitBinary } from './socket';
import { encodeCameraUpdate, type OrbitParams } from './proto';
import type {
  CameraMode,
  Vector3,
  Quaternion,
} from './types';
import { COLLAB_CONFIG } from './types';

/**
 * Hook for accessing collab state and actions
 */
export function useCollab() {
  const { state, actions } = useCollabContext();
  const cameraUpdateIntervalRef = useRef<number | null>(null);
  const lastCameraStateRef = useRef<{
    position: Vector3;
    rotation: Quaternion;
    mode: CameraMode;
  } | null>(null);

  // Computed values
  const isHost = useMemo(
    () => state.selfId !== null && state.selfId === state.hostId,
    [state.selfId, state.hostId]
  );

  const selfParticipant = useMemo(
    () => (state.selfId ? state.participants[state.selfId] : null),
    [state.selfId, state.participants]
  );

  const otherParticipants = useMemo(
    () =>
      Object.values(state.participants).filter(
        (p) => p.id !== state.selfId
      ),
    [state.participants, state.selfId]
  );

  const participantCount = useMemo(
    () => Object.keys(state.participants).length,
    [state.participants]
  );

  const hostParticipant = useMemo(
    () =>
      state.hostId
        ? state.participants[state.hostId] || null
        : null,
    [state.hostId, state.participants]
  );

  // Camera update function (called frequently)
  const sendCameraUpdate = useCallback(
    (position: Vector3, rotation: Quaternion, mode: CameraMode, targetPlayer?: string, orbitParams?: OrbitParams | null) => {
      if (!state.isInSession) return;

      // Encode to binary (includes orbit params for ballOrbit mode)
      const data = encodeCameraUpdate(position, rotation, mode, targetPlayer, orbitParams);
      emitBinary('camera-update', data);

      // Store for reference
      lastCameraStateRef.current = { position, rotation, mode };

      // Update local participant state (server doesn't send broadcast back to sender)
      dispatchCameraUpdate({
        position,
        rotation,
        mode,
        targetPlayer: targetPlayer || null,
        timestamp: Date.now(),
      });

      // Log occasionally (every 100th update to avoid spam)
      if (Math.random() < 0.01) {
        console.log('[useCollab] Sending camera update:', { position, mode, orbitParams });
      }
    },
    [state.isInSession]
  );

  // Start sending camera updates at regular interval
  const startCameraUpdates = useCallback(
    (
      getCameraState: () => { position: Vector3; rotation: Quaternion; mode: CameraMode; targetPlayer?: string; orbitParams?: OrbitParams | null },
      shouldBroadcast?: () => boolean
    ) => {
      if (cameraUpdateIntervalRef.current) {
        clearInterval(cameraUpdateIntervalRef.current);
      }

      cameraUpdateIntervalRef.current = window.setInterval(() => {
        if (!state.isInSession) return;
        // Skip broadcasting if shouldBroadcast returns false (e.g., when following another viewer)
        if (shouldBroadcast && !shouldBroadcast()) return;
        const { position, rotation, mode, targetPlayer, orbitParams } = getCameraState();
        sendCameraUpdate(position, rotation, mode, targetPlayer, orbitParams);
      }, COLLAB_CONFIG.CAMERA_UPDATE_INTERVAL_MS);

      return () => {
        if (cameraUpdateIntervalRef.current) {
          clearInterval(cameraUpdateIntervalRef.current);
          cameraUpdateIntervalRef.current = null;
        }
      };
    },
    [state.isInSession, sendCameraUpdate]
  );

  // Cleanup interval on unmount
  useEffect(() => {
    return () => {
      if (cameraUpdateIntervalRef.current) {
        clearInterval(cameraUpdateIntervalRef.current);
      }
    };
  }, []);

  // Generate share URL
  const getShareUrl = useCallback(
    (sessionId: string) => {
      const baseUrl = window.location.origin;
      return `${baseUrl}/join/${sessionId}`;
    },
    []
  );

  return {
    // State
    isConnected: state.isConnected,
    isInSession: state.isInSession,
    isLoading: state.isLoading,
    error: state.error,
    sessionId: state.sessionId,
    replayId: state.replayId,
    selfId: state.selfId,
    hostId: state.hostId,
    isHost,
    selfParticipant,
    otherParticipants,
    participantCount,
    hostParticipant,
    participants: state.participants,
    chatMessages: state.chatMessages,
    playback: state.playback,
    environment: state.environment,
    // Ping & Drawing state
    activePings: state.activePings,
    strokes: state.strokes,
    toolState: state.toolState,

    // Actions
    createSession: actions.createSession,
    joinSession: actions.joinSession,
    leaveSession: actions.leaveSession,
    updatePlayback: actions.updatePlayback,
    sendChat: actions.sendChat,
    followViewer: actions.followViewer,
    transferHost: actions.transferHost,
    kickParticipant: actions.kickParticipant,
    banParticipant: actions.banParticipant,
    updateEnvironment: actions.updateEnvironment,
    clearError: actions.clearError,
    onCameraUpdate: actions.onCameraUpdate,
    onParticipantLeft: actions.onParticipantLeft,
    onFollowStatusChanged: actions.onFollowStatusChanged,
    // Ping & Drawing actions
    placePing: actions.placePing,
    setTool: actions.setTool,
    setDrawColor: actions.setDrawColor,
    setDrawThickness: actions.setDrawThickness,
    startStroke: actions.startStroke,
    sendStrokePoints: actions.sendStrokePoints,
    endStroke: actions.endStroke,
    undoStroke: actions.undoStroke,
    eraseStrokes: actions.eraseStrokes,
    clearAllDrawings: actions.clearAllDrawings,
    // Ping & Drawing callbacks
    onPingCreated: actions.onPingCreated,
    onPingExpired: actions.onPingExpired,
    onStrokeStarted: actions.onStrokeStarted,
    onStrokePoints: actions.onStrokePoints,
    onStrokeCompleted: actions.onStrokeCompleted,
    onStrokeRemoved: actions.onStrokeRemoved,
    onDrawingsCleared: actions.onDrawingsCleared,

    // Camera
    sendCameraUpdate,
    startCameraUpdates,

    // Utilities
    getShareUrl,
  };
}

// Type exports for consumers
export type UseCollabReturn = ReturnType<typeof useCollab>;
