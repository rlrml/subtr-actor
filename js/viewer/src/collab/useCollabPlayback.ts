import { useEffect, useRef, useCallback } from 'react';
import { useCollab } from './useCollab';

// GameEngine interface for TypeScript (actual implementation is in JS)
interface GameEngineInterface {
  currentTime: number;
  isPlaying: boolean;
  seek(time: number): void;
  togglePlay(): void;
  pause?(): void;
  play?(): void;
  setPlaybackSpeed(speed: number): void;
}

interface UseCollabPlaybackOptions {
  gameEngine: GameEngineInterface | null;
  /** Whether the game is fully loaded and ready for playback sync */
  isReady?: boolean;
  onPlayStateChange?: (isPlaying: boolean) => void;
  onTimeChange?: (time: number) => void;
  onSpeedChange?: (speed: number) => void;
}

/**
 * Hook to sync playback state between local GameEngine and collaborative session
 * Host controls playback, viewers receive updates
 */
export function useCollabPlayback({
  gameEngine,
  isReady = true,
  onPlayStateChange,
  onTimeChange,
  onSpeedChange,
}: UseCollabPlaybackOptions) {
  const { isInSession, isHost, playback, updatePlayback } = useCollab();
  const isApplyingSyncRef = useRef(false);
  const hasInitialSyncRef = useRef(false);

  // Apply playback sync from host (for viewers)
  useEffect(() => {
    if (!isInSession || isHost || !gameEngine || !playback) return;

    // Wait until game is ready before applying sync
    if (!isReady) {
      console.log('[useCollabPlayback] Waiting for game to be ready before sync');
      return;
    }

    // Prevent recursive updates
    if (isApplyingSyncRef.current) return;
    isApplyingSyncRef.current = true;

    try {
      const isSeeking = playback.seeking === true;
      const isInitialSync = !hasInitialSyncRef.current;

      // Calculate target time
      let targetTime = playback.timestamp;

      // Only apply latency compensation when NOT seeking and playing
      // During seeking, we want exact position without any compensation
      if (!isSeeking && !playback.paused) {
        const now = Date.now();
        const latency = now - playback.lastSyncAt;
        if (latency > 0 && latency < 5000) {
          targetTime += (latency / 1000) * playback.speed;
        }
      }

      const timeDiff = Math.abs(gameEngine.currentTime - targetTime);

      // Always apply seek when host is actively seeking (dragging timeline)
      // Otherwise only seek if difference is significant (> 0.5s)
      if (isInitialSync || isSeeking || timeDiff > 0.5) {
        // Seek if this is initial sync, host is seeking, or difference is significant
        if (!isSeeking) {
          console.log(`[useCollabPlayback] Seeking to ${targetTime.toFixed(2)}s (initial: ${isInitialSync}, diff: ${timeDiff.toFixed(2)}s)`);
        }
        gameEngine.seek(targetTime);
        onTimeChange?.(targetTime);
      }

      // Apply play/pause state using togglePlay since pause/play may not exist
      // When host is seeking, pause the viewer's playback to prevent drift
      const isCurrentlyPlaying = gameEngine.isPlaying;
      const shouldBePaused = playback.paused || isSeeking;

      if (shouldBePaused && isCurrentlyPlaying) {
        gameEngine.togglePlay(); // Toggle to pause
        onPlayStateChange?.(false);
      } else if (!shouldBePaused && !isCurrentlyPlaying) {
        gameEngine.togglePlay(); // Toggle to play
        onPlayStateChange?.(true);
      }

      // Apply speed
      gameEngine.setPlaybackSpeed(playback.speed);
      onSpeedChange?.(playback.speed);

      // Mark initial sync as done
      hasInitialSyncRef.current = true;
    } finally {
      isApplyingSyncRef.current = false;
    }
  }, [isInSession, isHost, playback, gameEngine, isReady, onPlayStateChange, onTimeChange, onSpeedChange]);

  // Reset initial sync flag when leaving session
  useEffect(() => {
    if (!isInSession) {
      hasInitialSyncRef.current = false;
    }
  }, [isInSession]);

  // Periodic sync for viewers (every 10 seconds) to correct any drift
  useEffect(() => {
    if (!isInSession || isHost || !gameEngine || !playback || !isReady) return;

    const SYNC_INTERVAL_MS = 10000; // 10 seconds

    const intervalId = setInterval(() => {
      if (isApplyingSyncRef.current || playback.paused) return;

      // Calculate expected time based on playback state
      const now = Date.now();
      const elapsed = (now - playback.lastSyncAt) / 1000;
      const expectedTime = playback.timestamp + elapsed * playback.speed;
      const actualTime = gameEngine.currentTime;
      const drift = Math.abs(actualTime - expectedTime);

      // If drift > 1 second, resync
      if (drift > 1.0) {
        console.log(`[useCollabPlayback] Periodic sync: drift ${drift.toFixed(2)}s, seeking to ${expectedTime.toFixed(2)}s`);
        isApplyingSyncRef.current = true;
        gameEngine.seek(expectedTime);
        onTimeChange?.(expectedTime);
        isApplyingSyncRef.current = false;
      }
    }, SYNC_INTERVAL_MS);

    return () => clearInterval(intervalId);
  }, [isInSession, isHost, gameEngine, playback, isReady, onTimeChange]);

  // Host: Periodic heartbeat to broadcast current position (for new joiners)
  useEffect(() => {
    if (!isInSession || !isHost || !gameEngine || !isReady) return;

    const HEARTBEAT_INTERVAL_MS = 5000; // 5 seconds

    const intervalId = setInterval(() => {
      // Only broadcast if playing (to keep viewers in sync)
      if (gameEngine.isPlaying) {
        updatePlayback({
          timestamp: gameEngine.currentTime,
          paused: false,
        });
      }
    }, HEARTBEAT_INTERVAL_MS);

    return () => clearInterval(intervalId);
  }, [isInSession, isHost, gameEngine, isReady, updatePlayback]);

  // Host: Send playback updates when local state changes
  const sendPlayPause = useCallback(
    async (paused: boolean) => {
      if (!isInSession || !isHost || !gameEngine) return;
      await updatePlayback({
        timestamp: gameEngine.currentTime,
        paused,
      });
    },
    [isInSession, isHost, gameEngine, updatePlayback]
  );

  const sendSpeedChange = useCallback(
    async (speed: number) => {
      if (!isInSession || !isHost || !gameEngine) return;
      await updatePlayback({
        timestamp: gameEngine.currentTime,
        speed,
      });
    },
    [isInSession, isHost, gameEngine, updatePlayback]
  );

  // Wrapped handlers that sync with collab
  const handlePlayPause = useCallback(() => {
    if (!gameEngine) return;

    // Non-collab mode or host: toggle locally
    if (!isInSession || isHost) {
      gameEngine.togglePlay();
      if (isInSession && isHost) {
        sendPlayPause(!gameEngine.isPlaying);
      }
    }
    // Viewers in collab mode: don't do anything locally
  }, [gameEngine, isInSession, isHost, sendPlayPause]);

  // Throttle ref for dragging updates
  const lastDragSyncRef = useRef<number>(0);
  const DRAG_SYNC_THROTTLE_MS = 50; // Send updates every 50ms during drag for smoother viewer experience

  // Send seek with seeking flag
  const sendSeekWithFlag = useCallback(
    async (timestamp: number, seeking: boolean) => {
      if (!isInSession || !isHost) return;
      await updatePlayback({ timestamp, seeking });
    },
    [isInSession, isHost, updatePlayback]
  );

  const handleSeek = useCallback(
    (time: number) => {
      if (!gameEngine) return;

      // Non-collab mode or host: seek locally
      if (!isInSession || isHost) {
        gameEngine.seek(time);
        onTimeChange?.(time);

        // During drag, send throttled updates with seeking=true so viewers apply immediately
        if (isInSession && isHost) {
          const now = Date.now();
          if (now - lastDragSyncRef.current >= DRAG_SYNC_THROTTLE_MS) {
            lastDragSyncRef.current = now;
            sendSeekWithFlag(time, true);
          }
        }
      }
      // Viewers in collab mode: don't seek locally
    },
    [gameEngine, isInSession, isHost, sendSeekWithFlag, onTimeChange]
  );

  // Called when user releases the seek slider - sends immediate collab update with seeking=false
  const handleSeekCommit = useCallback(
    (time: number) => {
      if (!gameEngine) return;

      // Non-collab mode or host: send final position
      if (!isInSession || isHost) {
        // Ensure final position is applied
        gameEngine.seek(time);
        onTimeChange?.(time);

        // Send immediate collab update on release with seeking=false
        if (isInSession && isHost) {
          sendSeekWithFlag(time, false);
        }
      }
    },
    [gameEngine, isInSession, isHost, sendSeekWithFlag, onTimeChange]
  );

  const handlePlaybackSpeedChange = useCallback(
    (speed: number) => {
      if (!gameEngine) return;

      // Non-collab mode or host: change speed locally
      if (!isInSession || isHost) {
        gameEngine.setPlaybackSpeed(speed);
        onSpeedChange?.(speed);
        if (isInSession && isHost) {
          sendSpeedChange(speed);
        }
      }
      // Viewers in collab mode: don't change speed locally
    },
    [gameEngine, isInSession, isHost, sendSpeedChange, onSpeedChange]
  );

  return {
    // Whether controls should be disabled for viewer
    controlsDisabled: isInSession && !isHost,
    handlePlayPause,
    handleSeek,
    handleSeekCommit,
    handlePlaybackSpeedChange,
  };
}
