import { useEffect, useRef, useState, useCallback } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { Loader2, Users, Calendar, ChevronRight, Camera, Settings, Lock, Box, Sparkles, FileCode, Check, Layers, Car } from 'lucide-react';
import { IoMdFootball } from 'react-icons/io';
import { GiMineExplosion, GiShieldImpact } from 'react-icons/gi';
import * as THREE from 'three';
import { Logo } from '@/components/ui/Logo';
import { GameEngine } from '@/game/GameEngine';
import { GameOverlay } from '@/components/GameOverlay';
import { DebugPanel } from '@/components/DebugPanel';
import { useSettings } from '@/components/SettingsPanel';
import { Slider } from '@/components/ui/slider';
import { toast } from 'sonner';

import { DevToolsPanel } from '@/components/DevToolsPanel';
import { FeedbackPopup } from '@/components/FeedbackPopup';
import { PlayerIndicator } from '@/components/PlayerIndicator';
import { Killfeed, KillfeedEntry } from '@/components/Killfeed';
import { EnvironmentSelector } from '@/components/EnvironmentSelector';
import { api } from '@/services/api';
import { userApi } from '@/services/user.api';
import { CollabProvider } from '@/collab/CollabProvider';
import { SessionPanel } from '@/components/collab/SessionPanel';
import { CollabOverlay } from '@/components/collab/CollabOverlay';
import { useCollab } from '@/collab/useCollab';
import { useCollabPlayback } from '@/collab/useCollabPlayback';
import { ChatOverlayMessages, ChatInput } from '@/components/chat';
import { ToolBar } from '@/components/tools/ToolBar';
import { useAuth } from '@/hooks/useAuth';
import type { QualityMetrics } from '@/types/quality';
// Clip system (024-clip-system)
import { ClipEditor } from '@/components/clips/ClipEditor';
import { useClipEditor } from '@/hooks/useClipEditor';
import type { CameraRecording } from '@/api/clips';
// SEO
import { SEOHead, StructuredData, createReplayStructuredData } from '@/components/SEO';
import { useReplaySEO } from '@/hooks/useSEO';

type CameraModeType = 'free' | 'follow' | 'ball' | 'ballOrbit' | 'player';
type ReplayEvent = { time: number; type: string; data?: unknown };

// Inner component that uses collab hooks (must be inside CollabProvider)
function ViewerContent({ replayId }: { replayId: string }) {
  const { user } = useAuth();
  const navigate = useNavigate();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const gameRef = useRef<GameEngine | null>(null);
  const [containerReady, setContainerReady] = useState(false);

  // Loading state
  const [loading, setLoading] = useState(true);
  const [loadingStep, setLoadingStep] = useState<string>('');
  const [loadingMessage, setLoadingMessage] = useState<string>('Loading...');
  const [error, setError] = useState<string>();

  // Drawer state - single drawer with tabs
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<'events' | 'camera' | 'settings' | 'collab'>('events');

  // Settings
  const [settings, updateSettings] = useSettings();

  // Collab state
  const {
    isInSession, isConnected, participantCount, participants, selfId, hostId,
    startCameraUpdates, onCameraUpdate, onParticipantLeft, onFollowStatusChanged,
    followViewer, chatMessages, sendChat, isHost, updateEnvironment,
    transferHost, kickParticipant, banParticipant,
    environment: collabEnvironment,
    // Ping & Drawing
    toolState, placePing, setTool, setDrawColor, setDrawThickness,
    strokes,
    onPingCreated, onPingExpired,
    startStroke, sendStrokePoints, endStroke, undoStroke, eraseStrokes, clearAllDrawings,
    onStrokeStarted, onStrokePoints, onStrokeCompleted, onStrokeRemoved, onDrawingsCleared,
  } = useCollab();

  // Chat overlay state
  const [isChatInputOpen, setIsChatInputOpen] = useState(false);

  // Follow state - track who we're following locally
  const [followingId, setFollowingId] = useState<string | null>(null);
  const followingIdRef = useRef<string | null>(null);

  // Game State
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [maxTime, setMaxTime] = useState(0);
  const [players, setPlayers] = useState<string[]>([]);
  const [playerTeams, setPlayerTeams] = useState<Record<string, number>>({});
  const [events, setEvents] = useState<ReplayEvent[]>([]);
  const [playerBoosts, setPlayerBoosts] = useState<Record<string, number>>({});
  const [playerScores, setPlayerScores] = useState<Record<string, number>>({});
  const [cameraMode, setCameraMode] = useState<CameraModeType>('free');
  const [selectedPlayer, setSelectedPlayer] = useState<string | null>(null);
  const [showFeedbackPopup, setShowFeedbackPopup] = useState(false);
  const hasShownFeedback = useRef(false);
  const [demoEvents, setDemoEvents] = useState<KillfeedEntry[]>([]);
  const demoEventIdRef = useRef(0);
  // Quality indicator (016-replay-quality-indicator)
  const [qualityScore, setQualityScore] = useState<number | null>(null);
  const [qualityMetrics, setQualityMetrics] = useState<QualityMetrics | null>(null);

  // SEO metadata (022-seo-optimization)
  const [replayMeta, setReplayMeta] = useState<{
    title?: string | null;
    team0Score?: number | null;
    team1Score?: number | null;
    mapName?: string | null;
    players?: Array<{ name: string }>;
    durationSeconds?: number | null;
  } | null>(null);
  const seoData = useReplaySEO(replayMeta);

  // Clip editor (024-clip-system, 026-clip-editor-redesign)
  const clipEditor = useClipEditor(replayId, maxTime);

  // Restore draft dialog state (026-clip-editor-redesign T016-T017)
  const [showRestoreDraftDialog, setShowRestoreDraftDialog] = useState(false);
  const pendingOpenRef = useRef<{ time: number; environmentId: string | null } | null>(null);

  // 026-clip-editor-redesign: Animated preview camera state
  const [previewCameraVisible, setPreviewCameraVisible] = useState(false);

  // Auto-stop clip recording when reaching end time
  useEffect(() => {
    if (clipEditor.state === 'recording' && currentTime >= clipEditor.endTime) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game && game.isRecordingClip()) {
        const recordedData = game.stopClipRecording();
        if (recordedData) {
          clipEditor.stopRecording(recordedData as CameraRecording);
        }
        game.pause();
        setIsPlaying(false);
      }
    }
  }, [currentTime, clipEditor.state, clipEditor.endTime, clipEditor]);

  // Auto-stop clip preview when reaching end time
  useEffect(() => {
    if (clipEditor.state === 'preview' && currentTime >= clipEditor.endTime) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game) {
        game.stopClipPlayback();
        game.pause();
        setIsPlaying(false);
        // Show keyframe visualizers again after cinematic preview
        if (clipEditor.cameraMode === 'cinematic') {
          game.showKeyframes?.();
        }
        clipEditor.stopPreview();
      }
    }
  }, [currentTime, clipEditor.state, clipEditor.endTime, clipEditor.cameraMode, clipEditor]);

  // 026-clip-editor-redesign: Sync keyframes to KeyframeVisualizer when they change
  // This is the SINGLE SOURCE OF TRUTH for keyframe visualization
  // All keyframe changes (add, remove, update) go through clipEditor state, then sync here
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (game?.keyframeVisualizer) {
      game.keyframeVisualizer.setKeyframes(clipEditor.keyframes);
    }
  }, [clipEditor.keyframes]);

  // Refs for accessing state values inside callbacks (avoid stale closures)
  const cameraModeRef = useRef<CameraModeType>('free');
  const selectedPlayerRef = useRef<string | null>(null);

  // Keep refs in sync with state for callback access
  useEffect(() => {
    followingIdRef.current = followingId;
  }, [followingId]);

  useEffect(() => {
    cameraModeRef.current = cameraMode;
  }, [cameraMode]);

  useEffect(() => {
    selectedPlayerRef.current = selectedPlayer;
  }, [selectedPlayer]);

  // Show feedback popup when playback ends
  useEffect(() => {
    if (maxTime > 0 && currentTime >= maxTime - 0.1 && !hasShownFeedback.current && !isPlaying) {
      hasShownFeedback.current = true;
      setShowFeedbackPopup(true);
    }
  }, [currentTime, maxTime, isPlaying]);

  const [cameraSettings, setCameraSettings] = useState(() => {
    // Load from localStorage if available
    try {
      const stored = localStorage.getItem('rl-viewer-camera-settings');
      if (stored) {
        return {
          distance: 260,
          height: 90,
          angle: -4,
          stiffness: 0.45,
          swivelSpeed: 4.30,
          transitionSpeed: 1.30,
          fov: 110,
          freeCamSpeed: 2000,
          ...JSON.parse(stored)
        };
      }
    } catch (e) {
      console.warn('[CameraSettings] Failed to load from localStorage:', e);
    }
    return {
      distance: 260,
      height: 90,
      angle: -4,
      stiffness: 0.45,
      swivelSpeed: 4.30,
      transitionSpeed: 1.30,
      fov: 110,
      freeCamSpeed: 2000
    };
  });
  const [gameTimeMap, setGameTimeMap] = useState<Array<{ time: number; gameTime: number }>>([]);
  const [countdownEvents, setCountdownEvents] = useState<Array<{ time: number; text: string }>>([]);
  const [playbackSpeed, setPlaybackSpeed] = useState(1.0);
  const [textOverlays, setTextOverlays] = useState<Array<{ text: string; position: string }>>([]);
  const [actors, setActors] = useState<Record<string, unknown>>({});
  const [ballActorId, setBallActorId] = useState<string | null>(null);

  // Debug state
  const [interpolationEnabled, setInterpolationEnabled] = useState(true);
  const [interpolationMethod, setInterpolationMethod] = useState<string>('lerp');
  const [smoothingWindowSize, setSmoothingWindowSize] = useState(5);
  const [frameInfo, setFrameInfo] = useState<unknown>(null);
  const [ballTimeline, setBallTimeline] = useState<Array<{ time: number; position?: unknown }>>([]);
  const [playerTimelines, setPlayerTimelines] = useState<Record<string, Array<{ time: number; position?: unknown }>>>({});
  // Player stats timelines for real-time stats display (goals, assists, saves, shots, demos, score, ping)
  const [playerStatsTimelines, setPlayerStatsTimelines] = useState<Record<string, Array<{
    time: number;
    frame: number;
    ping: number;
    goals: number;
    assists: number;
    saves: number;
    shots: number;
    score: number;
    demos: number;
  }>>>({});
  // Game event timeline for overtime detection
  const [gameEventTimeline, setGameEventTimeline] = useState<Array<{
    time: number;
    frame: number;
    countdown: number;
    roundNum: number;
    ballHasBeenHit: boolean;
    isOvertime: boolean;
  }>>([]);

  // Advanced stats timelines for real-time stats display (018-stats-compiler)
  const [advancedStats, setAdvancedStats] = useState<{
    playerTimelines: Record<string, Array<{
      time: number;
      frame: number;
      currentSpeed: number;
      currentBoostAmount: number;
      isBoosting: boolean;
      isAirborne: boolean;
      avgSpeedSoFar: number;
      boostConsumedSoFar: number;
      boostPickupsSoFar: number;
      airTimeSecondsSoFar: number;
      offensiveTimeSoFar: number;
      defensiveTimeSoFar: number;
    }>>;
    teamTimelines: Record<number, Array<{
      time: number;
      possessionPercentage: number;
      avgTeamSpeed: number;
      totalBoostPickups: number;
    }>>;
    matchTimeline: Array<{
      time: number;
      ballSpeed: number;
      avgBallSpeedSoFar: number;
    }>;
  } | null>(null);

  // Player car info (carName, hitboxType, stats, platform)
  const [playerCarInfo, setPlayerCarInfo] = useState<Record<string, {
    carName: string;
    hitboxType: string;
    platform: string | null;
    goals: number;
    assists: number;
    saves: number;
    shots: number;
    matchScore: number;
    isBot: boolean;
  }>>({});

  // DevTools manager reference
  const [devToolsManager, setDevToolsManager] = useState<unknown>(null);

  // Custom environment state
  const [customEnvironmentId, setCustomEnvironmentId] = useState<string | null>(null);
  const [isLoadingEnvironment, setIsLoadingEnvironment] = useState(false);
  const userPreferenceLoadedRef = useRef(false);
  const [gameReady, setGameReady] = useState(false); // Game engine ready (replay loaded)

  // Collab playback sync
  const {
    controlsDisabled,
    handlePlayPause: collabPlayPause,
    handleSeek: collabSeek,
    handleSeekCommit: collabSeekCommit,
    handlePlaybackSpeedChange: collabSpeedChange,
  } = useCollabPlayback({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    gameEngine: gameRef.current as any,
    isReady: !loading,
    onPlayStateChange: setIsPlaying,
    onTimeChange: setCurrentTime,
    onSpeedChange: setPlaybackSpeed,
  });

  // Callback ref to detect when container is mounted
  const setContainerRef = useCallback((node: HTMLDivElement | null) => {
    containerRef.current = node;
    if (node) {
      setContainerReady(true);
    }
  }, []);

  // Load replay binary and initialize game
  useEffect(() => {
    if (!containerReady || !containerRef.current || !replayId) return;

    const loadReplay = async () => {
      try {
        setLoading(true);
        setError(undefined);

        // First, check if the replay is up to date
        const replayInfo = await api.get<{
          replay: {
            status: string;
            frameworkVersion?: string;
            qualityScore?: number | null;
            qualityMetrics?: QualityMetrics | null;
            title?: string | null;
            team0Score?: number | null;
            team1Score?: number | null;
            mapName?: string | null;
            durationSeconds?: number | null;
          };
          players?: Array<{ name: string }>;
          currentFrameworkVersion?: string;
          needsRecompilation?: boolean;
        }>(`/replays/${replayId}`);

        // Store quality data for display
        setQualityScore(replayInfo.replay.qualityScore ?? null);
        setQualityMetrics(replayInfo.replay.qualityMetrics ?? null);

        // Store SEO metadata
        setReplayMeta({
          title: replayInfo.replay.title,
          team0Score: replayInfo.replay.team0Score,
          team1Score: replayInfo.replay.team1Score,
          mapName: replayInfo.replay.mapName,
          durationSeconds: replayInfo.replay.durationSeconds,
          players: replayInfo.players,
        });

        // Redirect to detail page if replay needs recompilation
        if (replayInfo.needsRecompilation) {
          navigate(`/replays/${replayId}`, { replace: true });
          return;
        }

        // Also check if replay is not ready
        if (replayInfo.replay.status !== 'ready') {
          navigate(`/replays/${replayId}`, { replace: true });
          return;
        }

        // Record view (fire and forget - don't block loading)
        api.post(`/replays/${replayId}/view`).catch(() => {
          // Silently ignore errors - view tracking is not critical
        });

        // Fetch binary from API
        const binaryUrl = api.getBinaryUrl(replayId);
        const response = await fetch(binaryUrl);

        if (!response.ok) {
          throw new Error('Failed to load replay');
        }

        const arrayBuffer = await response.arrayBuffer();

        // Initialize Game Engine with binary data
        const game = new GameEngine(containerRef.current!, {
          onTimeUpdate: (time: number) => setCurrentTime(time),
          onMaxTimeUpdate: (max: number) => setMaxTime(max),
          onGameTimeInfoUpdate: (info: { gameTimeMap?: Array<{ time: number; gameTime: number }> }) => {
            if (info.gameTimeMap) setGameTimeMap(info.gameTimeMap);
          },
          onPlayStateChange: (playing: boolean) => setIsPlaying(playing),
          onPlayerListUpdate: (playerName: string) => {
            setPlayers(prev => {
              if (prev.includes(playerName)) return prev;
              return [...prev, playerName];
            });
          },
          onPlayerTeamsUpdate: (teams: Record<string, number>) => setPlayerTeams(teams),
          onEventsLoaded: (loadedEvents: ReplayEvent[]) => setEvents(loadedEvents),
          onPlayerSelect: (playerName: string) => setSelectedPlayer(playerName),
          onCountdownEventsUpdate: (events: Array<{ time: number; text: string }>) => setCountdownEvents(events),
          onPlayerBoostUpdate: (boosts: Record<string, number>) => setPlayerBoosts(boosts),
          onPlayerScoresUpdate: (scores: Record<string, number>) => setPlayerScores(scores),
          onTextOverlaysUpdate: (overlays: Array<{ text: string; position: string }>) => setTextOverlays(overlays),
          onActorsUpdate: (actorsData: Record<string, unknown>, ballId: string | null) => {
            setActors(actorsData);
            setBallActorId(ballId);
          },
          onFrameInfoUpdate: (info: unknown) => setFrameInfo(info),
          onTimelinesReady: (ball: Array<{ time: number }>, players: Record<string, Array<{ time: number }>>) => {
            setBallTimeline(ball);
            setPlayerTimelines(players);
          },
          onPlayerStatsTimelinesReady: (statsTimelines: Record<string, Array<{
            time: number;
            frame: number;
            ping: number;
            goals: number;
            assists: number;
            saves: number;
            shots: number;
            score: number;
            demos: number;
          }>>) => {
            setPlayerStatsTimelines(statsTimelines);
          },
          onGameEventTimelineReady: (eventTimeline: Array<{
            time: number;
            frame: number;
            countdown: number;
            roundNum: number;
            ballHasBeenHit: boolean;
            isOvertime: boolean;
          }>) => {
            setGameEventTimeline(eventTimeline);
          },
          // Advanced stats callback (018-stats-compiler)
          onAdvancedStatsReady: (stats: typeof advancedStats) => {
            setAdvancedStats(stats);
          },
          onPlayerCarInfoUpdate: (carInfo: Record<string, {
            carName: string;
            hitboxType: string;
            platform: string | null;
            goals: number;
            assists: number;
            saves: number;
            shots: number;
            matchScore: number;
            isBot: boolean;
          }>) => setPlayerCarInfo(carInfo),
          onDemoEvent: (event: { victim: string; attacker: string; victimTeam: number; attackerTeam: number; time: number }) => {
            demoEventIdRef.current += 1;
            const newEntry: KillfeedEntry = {
              id: `demo-${demoEventIdRef.current}`,
              attacker: event.attacker,
              victim: event.victim,
              attackerTeam: event.attackerTeam,
              victimTeam: event.victimTeam,
              timestamp: event.time,
            };
            setDemoEvents((prev) => [...prev, newEntry]);
          },
          onLoadingProgress: (step: string, message: string) => {
            setLoadingStep(step);
            setLoadingMessage(message);
          },
          onReady: () => {
            // Game engine is ready, now we need to load the environment
            setGameReady(true);
          },
          onError: (err: Error) => {
            setError(err.message);
            setLoading(false);
          },
          binaryData: arrayBuffer,
          // No initial skybox - it will be loaded from custom environment if selected
          initialSkyboxId: undefined,
        });

        gameRef.current = game;
        setDevToolsManager(game.devToolsManager);

        // Apply initial settings from localStorage
        if (typeof settings.showHitboxes === 'boolean') {
          game.setShowHitboxes(settings.showHitboxes);
        }
        if (settings.showBallSpeed !== undefined || settings.showCarSpeed !== undefined) {
          game.setSpeedDisplaySettings(
            settings.showBallSpeed ?? false,
            settings.showCarSpeed ?? false,
            settings.speedUnit ?? 'kmh'
          );
        }
        // Note: setLoading(false) is now called by onReady callback
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load replay');
        setLoading(false);
      }
    };

    loadReplay();

    return () => {
      gameRef.current?.dispose();
    };
  }, [replayId, containerReady, navigate]);

  // Apply settings changes
  // Note: skybox and exposure are now managed by custom environments

  useEffect(() => {
    if (gameRef.current && typeof settings.showHitboxes === 'boolean') {
      gameRef.current.setShowHitboxes(settings.showHitboxes);
    }
  }, [settings.showHitboxes]);

  useEffect(() => {
    if (gameRef.current) {
      gameRef.current.setSpeedDisplaySettings(
        settings.showBallSpeed ?? false,
        settings.showCarSpeed ?? false,
        settings.speedUnit ?? 'kmh'
      );
    }
  }, [settings.showBallSpeed, settings.showCarSpeed, settings.speedUnit]);

  // Setup clip playback end callback
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Set up the callback that will be called when clip playback ends
    game.callbacks.onClipPlaybackEnd = () => {
      console.log('[Viewer] Clip playback ended');
      game.stopClipPlayback();
      setIsPlaying(false);
      // Show keyframe visualizers again after cinematic preview
      if (clipEditor.cameraMode === 'cinematic') {
        game.showKeyframes?.();
        // Also restore preview camera (ghost camera) if we have enough keyframes
        // 026-clip-editor-redesign: Ghost camera was hidden during preview, restore it
        const visualizerKeyframeCount = game.keyframeVisualizer?.keyframes?.length ?? 0;
        if (visualizerKeyframeCount >= 2) {
          game.showPreviewCamera(clipEditor.startTime);
          setPreviewCameraVisible(true);
        }
      }
      clipEditor.stopPreview();
    };

    return () => {
      if (game.callbacks) {
        game.callbacks.onClipPlaybackEnd = null;
      }
    };
  }, [clipEditor, clipEditor.cameraMode, clipEditor.startTime]);

  // Update viewer cameras when participants change (collab) - initial setup
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (game && isInSession && participants && selfId) {
      game.updateViewerCameras(participants, selfId);
    }
  }, [isInSession, participants, selfId, loading]); // Added loading to re-trigger when game is ready

  // Direct camera update callback (bypasses React for better performance)
  useEffect(() => {
    if (!isInSession) return;

    // Register callback for direct camera updates
    // orbitParams is passed separately for ballOrbit mode (contains distance, azimuth, polar)
    const unsubscribe = onCameraUpdate((participantId, camera, orbitParams) => {
      // Read fresh gameRef inside callback to avoid stale closure
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;

      // Check if we're following this participant
      if (followingIdRef.current === participantId && game?.cameraManager) {
        // Apply their camera to our camera
        const currentMode = game.cameraManager.mode || cameraModeRef.current;
        const theirMode = camera.mode;

        if (theirMode === 'free') {
          // For free cam, sync position and rotation via network interpolation
          if (currentMode !== 'free') {
            game.setCameraMode('free');
            setCameraMode('free');
          }
          // Enable network interpolation for free cam
          game.cameraManager.setFollowingViewer(true);
          game.cameraManager.setFreecamState(camera.position, camera.rotation);
        } else if (theirMode === 'ballOrbit') {
          // For ball cam, use orbit params to orbit around LOCAL ball
          // This prevents stuttering because the camera follows the local ball at 60fps
          if (currentMode !== 'ballOrbit') {
            game.setCameraMode('ballOrbit');
            setCameraMode('ballOrbit');
          }
          // Enable network interpolation for ball orbit
          game.cameraManager.setFollowingViewer(true);
          if (orbitParams) {
            // Apply orbit params - camera will orbit around LOCAL ball
            game.cameraManager.setBallOrbitState(orbitParams);
          }
        } else if (theirMode === 'player' && camera.targetPlayer) {
          // For player cam, use local camera following (not network interpolation)
          // The camera follows the player locally, so we just sync the target player
          if (currentMode !== 'player' || selectedPlayerRef.current !== camera.targetPlayer) {
            game.setCameraMode('player');
            setCameraMode('player');
            game.selectPlayer(camera.targetPlayer);
            setSelectedPlayer(camera.targetPlayer);
          }
          // Disable network interpolation - player cam works locally
          game.cameraManager.setFollowingViewer(false);
        }
      }

      // Always update the viewer camera visualization
      if (game?.viewerCameraManager) {
        game.viewerCameraManager.updateViewerCamera(participantId, camera);
      }
    });

    return unsubscribe;
  }, [isInSession, onCameraUpdate]);

  // Start broadcasting camera updates when in a collab session
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!isInSession || !game) return;

    // Start sending camera updates at regular intervals
    // Don't broadcast when following someone (our camera mesh shouldn't be visible to others)
    const cleanup = startCameraUpdates(
      () => {
        if (!game) return { position: { x: 0, y: 0, z: 0 }, rotation: { x: 0, y: 0, z: 0, w: 1 }, mode: 'free' as const };
        return game.getCameraState();
      },
      () => !followingIdRef.current // shouldBroadcast: true when NOT following anyone
    );

    return cleanup;
  }, [isInSession, startCameraUpdates, loading]); // Added loading to re-trigger when game is ready

  // Clean up viewer cameras when participants leave
  useEffect(() => {
    if (!isInSession) return;

    const unsubscribe = onParticipantLeft((participantId) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game?.viewerCameraManager) {
        game.viewerCameraManager.removeViewer(participantId);
      }
      // Stop following if the followed participant left
      if (followingIdRef.current === participantId) {
        setFollowingId(null);
      }
    });

    return unsubscribe;
  }, [isInSession, onParticipantLeft]);

  // Handle follow status changes from other participants (direct callback, bypasses React)
  useEffect(() => {
    if (!isInSession) return;

    const unsubscribe = onFollowStatusChanged((participantId, followingTargetId) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      console.log(`[Viewer] *** FOLLOW STATUS: ${participantId.slice(0,8)} -> ${followingTargetId ? 'following' : 'free'}`);
      if (game?.viewerCameraManager) {
        game.viewerCameraManager.setViewerIsFollowing(participantId, followingTargetId !== null);
      }
    });

    return unsubscribe;
  }, [isInSession, onFollowStatusChanged]);

  // Clean up all viewer cameras when leaving session
  const wasInSessionRef = useRef(false);
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;

    // Track session state changes
    if (isInSession) {
      wasInSessionRef.current = true;
    } else if (wasInSessionRef.current) {
      // We just left a session - clean up all viewer cameras
      console.log('[Viewer] Left session, cleaning up all viewer cameras');
      if (game?.viewerCameraManager) {
        game.viewerCameraManager.reset();
      }
      // Clear following state
      setFollowingId(null);
      wasInSessionRef.current = false;
    }
  }, [isInSession]);

  // Hide/show followed viewer's mesh and set camera follow mode when followingId changes
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game?.viewerCameraManager) return;

    // Hide/show the followed viewer's mesh
    game.viewerCameraManager.setFollowedViewer(followingId);

    // When stopping following, reset to free cam to avoid being stuck in player mode
    if (!followingId && game.cameraManager) {
      game.cameraManager.setFollowingViewer(false);
      // Reset to free cam when stopping follow
      game.setCameraMode('free');
      setCameraMode('free');
      setSelectedPlayer(null);
    }
  }, [followingId]);

  // Hide meshes of participants who are following someone (they shouldn't be visible to anyone)
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game?.viewerCameraManager || !isInSession) return;

    // Check each participant's followingId and mark them as following or not
    Object.values(participants).forEach((participant) => {
      if (participant.id === selfId) return; // Skip self

      const isFollowing = participant.followingId !== null;
      // Use setViewerIsFollowing to properly track and hide/show meshes
      game.viewerCameraManager.setViewerIsFollowing(participant.id, isFollowing);
    });
  }, [participants, isInSession, selfId]);

  // Connect ping callbacks to PingManager
  useEffect(() => {
    if (!isInSession) return;

    const unsubscribePingCreated = onPingCreated((ping) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game?.pingManager) {
        game.pingManager.createPing(ping);
      }
    });

    const unsubscribePingExpired = onPingExpired((pingId) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game?.pingManager) {
        game.pingManager.fadeOutPing(pingId);
      }
    });

    return () => {
      unsubscribePingCreated();
      unsubscribePingExpired();
    };
  }, [isInSession, onPingCreated, onPingExpired]);

  // Sync existing strokes when joining a session (for initial state sync)
  // Track if we've done initial sync to avoid re-syncing on every stroke update
  const initialSyncDoneRef = useRef(false);
  useEffect(() => {
    if (!isInSession || !gameReady) {
      if (!isInSession) {
        initialSyncDoneRef.current = false;
      }
      return;
    }

    // Only sync once when we first join with strokes
    if (initialSyncDoneRef.current) return;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (game?.drawingManager) {
      if (strokes.length > 0) {
        console.log('[Viewer] Initial sync:', strokes.length, 'strokes to DrawingManager');
        game.drawingManager.syncStrokes(strokes);
      }
      initialSyncDoneRef.current = true;
    }
  }, [isInSession, gameReady, strokes]);

  // Connect drawing callbacks to DrawingManager
  useEffect(() => {
    if (!isInSession) return;

    const unsubscribeStrokeStarted = onStrokeStarted((stroke) => {
      console.log('[Viewer] onStrokeStarted callback:', stroke.id, 'by', stroke.authorId);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game?.drawingManager) {
        game.drawingManager.startStroke(stroke.id, stroke.authorId, stroke.color, stroke.thickness, stroke.points[0]);
      } else {
        console.warn('[Viewer] DrawingManager not available for stroke start');
      }
    });

    const unsubscribeStrokePoints = onStrokePoints((strokeId, _authorId, points) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game?.drawingManager) {
        game.drawingManager.addPoints(strokeId, points);
      }
    });

    const unsubscribeStrokeCompleted = onStrokeCompleted((strokeId) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game?.drawingManager) {
        game.drawingManager.completeStroke(strokeId);
      }
    });

    const unsubscribeStrokeRemoved = onStrokeRemoved((strokeId) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game?.drawingManager) {
        game.drawingManager.removeStroke(strokeId);
      }
    });

    const unsubscribeDrawingsCleared = onDrawingsCleared(() => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const game = gameRef.current as any;
      if (game?.drawingManager) {
        game.drawingManager.clearAll();
      }
    });

    return () => {
      unsubscribeStrokeStarted();
      unsubscribeStrokePoints();
      unsubscribeStrokeCompleted();
      unsubscribeStrokeRemoved();
      unsubscribeDrawingsCleared();
    };
  }, [isInSession, onStrokeStarted, onStrokePoints, onStrokeCompleted, onStrokeRemoved, onDrawingsCleared]);

  // Sync tool state with GameEngine
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Update active tool in GameEngine
    game.setActiveTool(toolState.activeTool);

    // Update draw settings
    game.setDrawSettings(toolState.drawColor, toolState.drawThickness);
  }, [toolState.activeTool, toolState.drawColor, toolState.drawThickness]);

  // Setup terrain click/drag callbacks for pings and drawing
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game || !gameReady || !isInSession) return;

    console.log('[Viewer] Setting up terrain callbacks, gameReady:', gameReady, 'isInSession:', isInSession);

    // Track pending draw points for batching
    let pendingPoints: Array<{ x: number; y: number; z: number }> = [];
    let sendPointsTimeout: ReturnType<typeof setTimeout> | null = null;

    const sendPendingPoints = (strokeId: string) => {
      if (pendingPoints.length > 0) {
        sendStrokePoints(strokeId, pendingPoints);
        pendingPoints = [];
      }
    };

    // Terrain click callback
    game.setTerrainClickCallback((
      position: { x: number; y: number; z: number } | null,
      type: string,
      data?: { strokeId?: string; color?: string; thickness?: number; normal?: { x: number; y: number; z: number } }
    ) => {
      if (type === 'ping' && position) {
        // Place ping with surface normal for orientation
        placePing(position, data?.normal);
      } else if (type === 'draw-start' && position && data) {
        // Start stroke
        startStroke(data.strokeId!, data.color!, data.thickness!, position);
      } else if (type === 'draw-end' && data) {
        // Send any pending points first
        if (sendPointsTimeout) {
          clearTimeout(sendPointsTimeout);
          sendPointsTimeout = null;
        }
        sendPendingPoints(data.strokeId!);
        // End stroke
        endStroke(data.strokeId!);
      } else if (type === 'erase' && position) {
        // Find strokes near click position using DrawingManager
        const nearbyStrokes = game.getStrokesNearPoint(position, 50);
        if (nearbyStrokes.length > 0) {
          console.log('[Viewer] Erasing strokes:', nearbyStrokes);
          eraseStrokes(nearbyStrokes);
        }
      }
    });

    // Terrain drag callback (for drawing)
    game.setTerrainDragCallback((
      position: { x: number; y: number; z: number },
      type: string,
      data?: { strokeId?: string }
    ) => {
      if (type === 'draw-point' && data?.strokeId) {
        // Batch points and send periodically (every 50ms)
        pendingPoints.push(position);

        if (!sendPointsTimeout) {
          sendPointsTimeout = setTimeout(() => {
            sendPendingPoints(data.strokeId!);
            sendPointsTimeout = null;
          }, 50);
        }
      }
    });

    return () => {
      game.setTerrainClickCallback(null);
      game.setTerrainDragCallback(null);
      if (sendPointsTimeout) {
        clearTimeout(sendPointsTimeout);
      }
    };
  }, [gameReady, isInSession, placePing, startStroke, sendStrokePoints, endStroke, eraseStrokes]);

  // Handler for following another viewer
  const handleFollowViewer = useCallback(async (targetId: string | null) => {
    // Update local state
    setFollowingId(targetId);
    // Notify server (for potential future use)
    await followViewer(targetId);
  }, [followViewer]);

  // Chat and Tool keyboard handlers (only in collab session)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Skip if chat input is already open
      if (isChatInputOpen) return;

      // Skip if not in collab session
      if (!isInSession) return;

      // Skip if target is an input/textarea (user is typing elsewhere)
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

      // T opens chat (like in Rocket League)
      if (e.key === 't' || e.key === 'T') {
        e.preventDefault();
        setIsChatInputOpen(true);
      }

      // P toggles ping mode
      if (e.key === 'p' || e.key === 'P') {
        e.preventDefault();
        setTool(toolState.activeTool === 'ping' ? 'select' : 'ping');
      }

      // B toggles draw mode (Brush)
      if (e.key === 'b' || e.key === 'B') {
        e.preventDefault();
        setTool(toolState.activeTool === 'draw' ? 'select' : 'draw');
      }

      // X toggles eraser mode
      if (e.key === 'x' || e.key === 'X') {
        e.preventDefault();
        setTool(toolState.activeTool === 'eraser' ? 'select' : 'eraser');
      }

      // Z triggers undo (Ctrl+Z or just Z when in draw/eraser mode)
      if ((e.key === 'z' || e.key === 'Z') && (e.ctrlKey || e.metaKey || toolState.activeTool === 'draw' || toolState.activeTool === 'eraser')) {
        e.preventDefault();
        undoStroke();
      }

      // Escape exits tool mode back to select
      if (e.key === 'Escape' && toolState.activeTool !== 'select') {
        e.preventDefault();
        setTool('select');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isChatInputOpen, isInSession, toolState.activeTool, setTool, undoStroke]);

  // Handler for sending chat message
  const handleSendChatMessage = useCallback((text: string) => {
    if (sendChat) {
      sendChat(text);
    }
  }, [sendChat]);

  // Handlers - use collab-aware versions for playback control
  const handlePlayPause = useCallback(() => {
    if (isInSession) {
      collabPlayPause();
    } else {
      gameRef.current?.togglePlay();
    }
  }, [isInSession, collabPlayPause]);

  const handleSeek = useCallback((time: number) => {
    if (isInSession) {
      collabSeek(time);
    } else {
      gameRef.current?.seek(time);
      setCurrentTime(time);
    }
  }, [isInSession, collabSeek]);

  // Called when user releases the slider (for collab sync)
  const handleSeekCommit = useCallback((time: number) => {
    if (isInSession) {
      collabSeekCommit(time);
    }
    // No-op for non-collab mode (already handled by handleSeek)
  }, [isInSession, collabSeekCommit]);

  const handlePlaybackSpeedChange = useCallback((speed: number) => {
    if (isInSession) {
      collabSpeedChange(speed);
    } else {
      setPlaybackSpeed(speed);
      gameRef.current?.setPlaybackSpeed(speed);
    }
  }, [isInSession, collabSpeedChange]);

  const handleEventClick = (event: ReplayEvent) => gameRef.current?.handleEventClick(event, cameraMode);
  const handleCameraModeChange = (mode: CameraModeType) => {
    setCameraMode(mode);
    gameRef.current?.setCameraMode(mode);

    // Auto-select first player when switching to player mode with no selection
    if (mode === 'player' && !selectedPlayer && players.length > 0) {
      const firstPlayer = players[0];
      setSelectedPlayer(firstPlayer);
      gameRef.current?.selectPlayer(firstPlayer);
    }
  };
  const handlePlayerSelect = (player: string) => {
    setSelectedPlayer(player);
    gameRef.current?.selectPlayer(player);
  };
  const handleCameraSettingsChange = (newSettings: typeof cameraSettings) => {
    setCameraSettings(newSettings);
    gameRef.current?.updateCameraSettings(newSettings);
    try {
      localStorage.setItem('rl-viewer-camera-settings', JSON.stringify(newSettings));
    } catch (e) {
      console.warn('[CameraSettings] Failed to save to localStorage:', e);
    }
  };
  const handleInterpolationToggle = (enabled: boolean) => {
    setInterpolationEnabled(enabled);
    gameRef.current?.setInterpolationEnabled(enabled);
  };

  const handleInterpolationMethodChange = (method: string) => {
    setInterpolationMethod(method);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const actorManager = (gameRef.current as any)?.actorManager;
    actorManager?.setInterpolationMethod(method);
  };

  const handleSmoothingWindowSizeChange = (size: number) => {
    setSmoothingWindowSize(size);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const actorManager = (gameRef.current as any)?.actorManager;
    actorManager?.setSmoothingWindowSize(size);
  };

  // Clip system handlers (024-clip-system, 026-clip-editor-redesign)
  const handleOpenClipEditor = useCallback(() => {
    if (!user) {
      // Require login for clip creation
      navigate('/login', { state: { from: `/viewer/${replayId}` } });
      return;
    }
    // Get current environment ID from EnvironmentManager
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    const currentEnv = game?.environmentManager?.getCurrentEnvironment?.();
    const environmentId = currentEnv?.id ?? null;

    // Check if a draft exists for this replay (T017)
    if (clipEditor.hasDraft) {
      // Store pending open parameters and show restore dialog
      pendingOpenRef.current = { time: currentTime, environmentId };
      setShowRestoreDraftDialog(true);
    } else {
      // No draft, open directly with current playback time and environment ID
      clipEditor.open(currentTime, environmentId);
    }
  }, [user, clipEditor, navigate, replayId, currentTime]);

  // Handle restore draft confirmation (T018)
  const handleRestoreDraft = useCallback(() => {
    const pending = pendingOpenRef.current;
    clipEditor.open(pending?.time ?? 0, pending?.environmentId ?? null);
    clipEditor.restoreDraft();
    setShowRestoreDraftDialog(false);
    pendingOpenRef.current = null;
  }, [clipEditor]);

  // Handle discard draft and start fresh
  const handleDiscardDraftAndOpen = useCallback(() => {
    const pending = pendingOpenRef.current;
    clipEditor.discardDraft();
    clipEditor.open(pending?.time ?? 0, pending?.environmentId ?? null);
    setShowRestoreDraftDialog(false);
    pendingOpenRef.current = null;
  }, [clipEditor]);

  // Handle cancel restore dialog
  const handleCancelRestoreDialog = useCallback(() => {
    setShowRestoreDraftDialog(false);
    pendingOpenRef.current = null;
  }, []);

  const handleStartClipRecording = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Pause playback, seek to clip start time, then start recording
    game.pause();
    game.seek(clipEditor.startTime);
    setCurrentTime(clipEditor.startTime);

    // Start recording in game engine
    game.startClipRecording();

    // Start playback
    game.play();
    setIsPlaying(true);
  }, [clipEditor.startTime]);

  const handleStopClipRecording = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Stop recording and get data
    const recordedData = game.stopClipRecording();
    if (recordedData) {
      clipEditor.stopRecording(recordedData as CameraRecording);
    }

    // Pause playback
    game.pause();
    setIsPlaying(false);
  }, [clipEditor]);

  const handleStartClipPreview = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Get camera data based on mode
    let cameraData = null;
    if (clipEditor.cameraMode === 'capture') {
      cameraData = clipEditor.recordedData;
    } else if (clipEditor.cameraMode === 'cinematic') {
      cameraData = clipEditor.getCinematicData();
      // Hide keyframe visualizers during cinematic preview
      game.hideKeyframes?.();
    }

    if (!cameraData) {
      console.warn('[Viewer] No camera data for preview');
      return;
    }

    console.log('[Viewer] handleStartClipPreview:', {
      cameraMode: clipEditor.cameraMode,
      startTime: clipEditor.startTime,
      cameraDataType: cameraData.type,
      keyframesCount: cameraData.type === 'cinematic' ? cameraData.keyframes?.length : undefined,
      keyframes: cameraData.type === 'cinematic' ? cameraData.keyframes?.map((kf: { t: number; px?: number; py?: number; pz?: number }) => ({
        t: kf.t,
        px: kf.px?.toFixed(2),
        py: kf.py?.toFixed(2),
        pz: kf.pz?.toFixed(2),
      })) : undefined,
    });

    // Hide preview camera during preview (we become the camera)
    if (previewCameraVisible) {
      game.hidePreviewCamera();
      setPreviewCameraVisible(false);
    }

    // startClipPlayback handles seek and play internally
    game.startClipPlayback(cameraData, clipEditor.startTime);
    setCurrentTime(clipEditor.startTime);
    setIsPlaying(true);
  }, [clipEditor.cameraMode, clipEditor.recordedData, clipEditor.getCinematicData, clipEditor.startTime, previewCameraVisible]);

  const handleStopClipPreview = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    game.stopClipPlayback();
    game.pause();
    setIsPlaying(false);

    // Show keyframe visualizers again after cinematic preview
    if (clipEditor.cameraMode === 'cinematic') {
      game.showKeyframes?.();

      // Also restore preview camera (ghost camera) if we have enough keyframes
      // 026-clip-editor-redesign: Ghost camera was hidden during preview, restore it
      const visualizerKeyframeCount = game.keyframeVisualizer?.keyframes?.length ?? 0;
      if (visualizerKeyframeCount >= 2) {
        game.showPreviewCamera(clipEditor.startTime);
        setPreviewCameraVisible(true);
      }
    }
  }, [clipEditor.cameraMode, clipEditor.startTime]);

  const handleCloseClipEditor = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Ensure we stop any recording/playback
    if (game.isRecordingClip()) {
      game.stopClipRecording();
    }
    if (game.isPlayingClip()) {
      game.stopClipPlayback();
    }
    // Clear keyframes from visualizer
    game.clearKeyframes?.();
    game.hideKeyframes?.();

    // Hide preview camera (026-clip-editor-redesign)
    game.hidePreviewCamera?.();
    setPreviewCameraVisible(false);

    game.pause();
    setIsPlaying(false);
  }, []);

  const handleClipSaved = useCallback(() => {
    // Could show a toast notification here
    console.log('[Viewer] Clip saved successfully');
  }, []);

  const handleCaptureFrame = useCallback(async (): Promise<Blob> => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game || !game.captureFrame) {
      throw new Error('Game engine not available');
    }
    return game.captureFrame();
  }, []);

  // Cinematic mode handlers (024-clip-system US2)
  const handleAddKeyframe = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Always use current playback time
    const keyframeTime = currentTime;

    // Check if the keyframe time is within segment bounds before adding
    const keyframeTimeMs = keyframeTime * 1000;
    if (!clipEditor.isTimeInSegment(keyframeTimeMs)) {
      toast.error('Move playhead into the clip zone to add a keyframe', {
        description: `Playhead is at ${Math.floor(keyframeTime / 60)}:${(keyframeTime % 60).toFixed(1).padStart(4, '0')}s, but clip zone is ${Math.floor(clipEditor.startTime / 60)}:${(clipEditor.startTime % 60).toFixed(0).padStart(2, '0')} - ${Math.floor(clipEditor.endTime / 60)}:${(clipEditor.endTime % 60).toFixed(0).padStart(2, '0')}`,
        duration: 4000,
      });
      return;
    }

    // Add keyframe at current camera position using GameEngine method
    const keyframe = game.addKeyframe(keyframeTime);
    if (keyframe) {
      const success = clipEditor.addKeyframe(keyframe);
      if (!success) {
        // Remove from visualizer if it failed to add to editor (e.g., duplicate)
        game.removeKeyframe(keyframe.id);
        toast.error('Could not add keyframe at this position');
      }
    }
  }, [currentTime, clipEditor]);

  const handleRemoveKeyframe = useCallback((id: string) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (game) {
      game.removeKeyframe(id);
    }
  }, []);

  // Clear all keyframes (markers + ghost camera)
  const handleClearAllKeyframes = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (game?.keyframeVisualizer) {
      game.keyframeVisualizer.clearAllKeyframes();
    }
    // Also hide the preview camera if GameEngine has a separate reference
    if (game?.hidePreviewCamera) {
      game.hidePreviewCamera();
    }
    setPreviewCameraVisible(false);
  }, []);

  // 026-clip-editor-redesign: T044 - Set active keyframe (hides its marker for clear view)
  const handleSetActiveKeyframe = useCallback((id: string | null) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (game?.keyframeVisualizer) {
      game.keyframeVisualizer.setActiveKeyframe(id);
    }
  }, []);

  // Update keyframe time in visualizer (called when user drags keyframe on timeline)
  const handleUpdateKeyframeTime = useCallback((id: string, newTimeMs: number) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (game?.keyframeVisualizer) {
      game.keyframeVisualizer.updateKeyframe(id, { t: newTimeMs });
    }
    // Also update in the hook
    clipEditor.updateKeyframeTime(id, newTimeMs);
  }, [clipEditor]);

  // 026-clip-editor-redesign: Toggle all markers AND ghost camera together
  // Ghost camera is always visible when markers are visible (not a separate option)
  const handleToggleAllMarkers = useCallback((visible: boolean) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Reset active keyframe when showing markers (so all markers become visible)
    if (visible && game.keyframeVisualizer) {
      game.keyframeVisualizer.setActiveKeyframe(null);
    }

    // Toggle markers
    if (game.keyframeVisualizer) {
      game.keyframeVisualizer.toggleAllMarkers(visible);
    }

    // Toggle ghost camera together with markers (only if enough keyframes exist)
    // Check keyframes directly from visualizer to avoid stale React state issues
    const visualizerKeyframeCount = game.keyframeVisualizer?.keyframes?.length ?? 0;
    const hasEnoughKeyframes = visualizerKeyframeCount >= 2;

    if (visible && hasEnoughKeyframes) {
      game.showPreviewCamera(clipEditor.startTime);
      setPreviewCameraVisible(true);
    } else {
      game.hidePreviewCamera();
      setPreviewCameraVisible(false);
    }
  }, [clipEditor.startTime]);

  // View keyframe - position camera at keyframe's saved position/rotation
  const handleViewKeyframe = useCallback((keyframe: { px: number; py: number; pz: number; qx: number; qy: number; qz: number; qw: number }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game || !game.cameraManager) return;

    // Set camera to freecam mode if not already
    if (game.cameraMode !== 'free') {
      game.setCameraMode('free');
      setCameraMode('free');
    }

    const cm = game.cameraManager;
    const camera = cm.camera;
    if (!camera) return;

    // Apply keyframe position and rotation to camera
    camera.position.set(keyframe.px, keyframe.py, keyframe.pz);
    camera.quaternion.set(keyframe.qx, keyframe.qy, keyframe.qz, keyframe.qw);

    // IMPORTANT: Update freeCamRotation state to match the quaternion
    // Otherwise updateFreeCam() will override the quaternion on next frame
    if (cm.freeCamRotation) {
      const dir = new THREE.Vector3();
      camera.getWorldDirection(dir);
      cm.freeCamRotation.yaw = Math.atan2(dir.x, dir.z);
      cm.freeCamRotation.pitch = Math.asin(-dir.y);
    }

    // Update camera-controls internal state to match
    if (cm.controls) {
      const lookAt = new THREE.Vector3();
      camera.getWorldDirection(lookAt);
      lookAt.multiplyScalar(100).add(camera.position);
      cm.controls.setLookAt(
        keyframe.px, keyframe.py, keyframe.pz,
        lookAt.x, lookAt.y, lookAt.z,
        false // immediate, no transition
      );
    }

    // Hide ghost camera immediately when viewing a keyframe (user is teleported to same position)
    // Force hide because ghost camera position may not be updated yet at this point
    // The animation loop will show it again when user moves away
    if (game.keyframeVisualizer && game.keyframeVisualizer.previewCamera) {
      game.keyframeVisualizer.previewCamera.visible = false;
    }
  }, []);

  // Update selected keyframe with current camera position
  const handleUpdateKeyframe = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game || !game.cameraManager) return;

    const selectedId = clipEditor.selectedKeyframeId;
    if (!selectedId) return;

    const camera = game.cameraManager.camera;
    if (!camera) return;

    // Update keyframe in editor state
    clipEditor.updateKeyframe(selectedId, {
      px: camera.position.x,
      py: camera.position.y,
      pz: camera.position.z,
      qx: camera.quaternion.x,
      qy: camera.quaternion.y,
      qz: camera.quaternion.z,
      qw: camera.quaternion.w,
    });

    // Update keyframe in visualizer
    game.updateKeyframe?.(selectedId, {
      px: camera.position.x,
      py: camera.position.y,
      pz: camera.position.z,
      qx: camera.quaternion.x,
      qy: camera.quaternion.y,
      qz: camera.quaternion.z,
      qw: camera.quaternion.w,
    });

    console.log('[Viewer] Updated keyframe position:', selectedId);
  }, [clipEditor]);

  // Handle loading a local replay file (for debugging)
  const handleLoadReplayFile = useCallback(async (arrayBuffer: ArrayBuffer, filename: string) => {
    const game = gameRef.current;
    if (!game) {
      console.error('[Viewer] GameEngine not available');
      return;
    }

    console.log(`[Viewer] Loading local replay file: ${filename}`);
    try {
      await game.loadReplayFromBinary(arrayBuffer);
      console.log('[Viewer] Local replay loaded successfully');
    } catch (err) {
      console.error('[Viewer] Failed to load local replay:', err);
      throw err;
    }
  }, []);

  // Handle custom environment loading
  const handleCustomEnvironmentChange = useCallback(async (environmentId: string) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game?.environmentManager) {
      console.warn('[Viewer] EnvironmentManager not available');
      return;
    }

    setIsLoadingEnvironment(true);
    try {
      await game.environmentManager.loadEnvironment(environmentId);
      setCustomEnvironmentId(environmentId);
      console.log('[Viewer] Custom environment loaded:', environmentId);

      // If in collab session as host, sync to other viewers
      if (isInSession && isHost) {
        updateEnvironment({ customEnvironmentId: environmentId });
      }

      // Save user preference if authenticated and not in collab session
      if (user && !isInSession) {
        try {
          await userApi.updatePreferredEnvironment(environmentId);
          console.log('[Viewer] User environment preference saved');
        } catch (error) {
          console.warn('[Viewer] Failed to save environment preference:', error);
        }
      }
    } catch (error) {
      console.error('[Viewer] Failed to load environment:', error);
    } finally {
      setIsLoadingEnvironment(false);
    }
  }, [isInSession, isHost, updateEnvironment, user]);

  // Sync environment from collab session (for viewers when host changes environment or on initial join)
  useEffect(() => {
    if (!isInSession || isHost) return;
    if (!gameReady) return; // Wait for game engine to be ready

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game?.environmentManager) return;

    const targetEnvId = collabEnvironment?.customEnvironmentId;

    // If no environment set in session, load default environment
    if (!targetEnvId) {
      // Only load default if we haven't loaded any environment yet
      if (customEnvironmentId) return;

      console.log('[Viewer] No custom environment in session, loading default');
      const loadDefault = async () => {
        try {
          const { environmentApi } = await import('@/services/environment.api');
          const defaultEnv = await environmentApi.getDefault();
          if (defaultEnv) {
            await game.environmentManager.loadEnvironment(defaultEnv.id);
            setCustomEnvironmentId(defaultEnv.id);
            console.log('[Viewer] Default environment loaded for viewer');
          }
        } catch (err) {
          console.warn('[Viewer] Failed to load default environment:', err);
        } finally {
          // Clear loading state after environment loads
          setLoading(false);
        }
      };
      loadDefault();
      return;
    }

    // Already loaded this environment
    if (targetEnvId === customEnvironmentId) return;

    console.log('[Viewer] Syncing environment from host:', targetEnvId);
    setIsLoadingEnvironment(true);

    game.environmentManager
      .loadEnvironment(targetEnvId)
      .then(() => {
        setCustomEnvironmentId(targetEnvId);
        console.log('[Viewer] Environment synced from host');
      })
      .catch((error: Error) => {
        console.error('[Viewer] Failed to sync environment from host:', error);
      })
      .finally(() => {
        setIsLoadingEnvironment(false);
        // Clear loading state after environment loads
        setLoading(false);
      });
  }, [isInSession, isHost, gameReady, collabEnvironment?.customEnvironmentId, customEnvironmentId]);

  // When host creates/joins a session, sync their current environment to the session
  useEffect(() => {
    if (!isInSession || !isHost) return;
    if (!gameReady) return; // Wait for game engine to be ready
    if (!customEnvironmentId) return;

    // If session already has this environment, skip
    if (collabEnvironment?.customEnvironmentId === customEnvironmentId) return;

    console.log('[Viewer] Host syncing environment to session:', customEnvironmentId);
    updateEnvironment({ customEnvironmentId });
  }, [isInSession, isHost, gameReady, customEnvironmentId, collabEnvironment?.customEnvironmentId, updateEnvironment]);

  // Load user's preferred environment on initial load (once game engine is ready)
  useEffect(() => {
    // Only load once, when game engine is ready
    if (!gameReady || userPreferenceLoadedRef.current) return;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game?.environmentManager) return;

    userPreferenceLoadedRef.current = true;

    // Check if there's a pending session join (viewer joining via /join/:sessionId)
    const hasPendingSession = sessionStorage.getItem('collab-session') !== null;

    // In collab session or about to join one, environment will be synced from host
    // Keep loading state active - it will be cleared when environment loads
    if (isInSession || hasPendingSession) {
      console.log('[Viewer] In/joining collab session, waiting for environment sync from host');
      setLoadingStep('environment');
      setLoadingMessage('Syncing environment...');
      // Don't set loading to false here - wait for environment sync
      return;
    }

    setLoadingStep('environment');
    setLoadingMessage('Loading environment...');

    const loadUserPreference = async () => {
      try {
        // If user is authenticated, load their preference (or default)
        if (user) {
          const prefs = await userApi.getPreferences();
          const envId = prefs.effectiveEnvironmentId;

          if (envId) {
            console.log('[Viewer] Loading user preferred environment:', envId);
            await game.environmentManager.loadEnvironment(envId);
            setCustomEnvironmentId(envId);
            console.log('[Viewer] User preferred environment loaded');
          }
        } else {
          // For non-authenticated users, try to load the default environment
          try {
            const { environmentApi } = await import('@/services/environment.api');
            const defaultEnv = await environmentApi.getDefault();
            if (defaultEnv) {
              console.log('[Viewer] Loading default environment for guest:', defaultEnv.id);
              await game.environmentManager.loadEnvironment(defaultEnv.id);
              setCustomEnvironmentId(defaultEnv.id);
              console.log('[Viewer] Default environment loaded');
            }
          } catch (err) {
            console.warn('[Viewer] No default environment available:', err);
          }
        }
      } catch (error) {
        console.warn('[Viewer] Failed to load environment:', error);
      } finally {
        // Finish loading
        setLoading(false);
      }
    };

    loadUserPreference();
  }, [gameReady, user, isInSession]);

  return (
    <div className="fixed inset-0 pt-16 bg-gray-950">
      {/* SEO (022-seo-optimization) */}
      <SEOHead {...seoData} />
      {replayMeta && (
        <StructuredData data={createReplayStructuredData({
          id: replayId,
          title: seoData.title,
          description: seoData.description,
          team0Score: replayMeta.team0Score ?? undefined,
          team1Score: replayMeta.team1Score ?? undefined,
          mapName: replayMeta.mapName ?? undefined,
          durationSeconds: replayMeta.durationSeconds ?? undefined,
        })} />
      )}

      {/* 3D Container - shrinks from bottom when clip editor is open */}
      <div
        ref={setContainerRef}
        className={`absolute left-0 right-0 top-16 transition-all duration-300 ease-in-out ${
          clipEditor.isOpen ? 'bottom-[280px]' : 'bottom-0'
        }`}
      />

      {/* Loading overlay */}
      {loading && (
        <div className="absolute inset-0 z-50 flex items-center justify-center overflow-hidden">
          {/* Animated gradient background */}
          <div className="absolute inset-0 bg-gray-950" />
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-violet-900/20 via-gray-950 to-gray-950" />
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_right,_var(--tw-gradient-stops))] from-blue-900/15 via-transparent to-transparent" />

          {/* Animated floating orbs */}
          <div className="absolute top-1/4 left-1/4 w-64 h-64 bg-violet-600/10 rounded-full blur-3xl animate-pulse" />
          <div className="absolute bottom-1/3 right-1/4 w-48 h-48 bg-blue-600/10 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '1s' }} />
          <div className="absolute top-1/2 right-1/3 w-32 h-32 bg-cyan-600/10 rounded-full blur-2xl animate-pulse" style={{ animationDelay: '0.5s' }} />

          {/* Main loading card */}
          <div className="relative z-10">
            {/* Glow effect behind card */}
            <div className="absolute inset-0 bg-gradient-to-r from-violet-600 to-blue-600 rounded-2xl blur-2xl opacity-30 scale-105" />

            {/* Card with glassmorphism */}
            <div className="relative bg-gray-900/80 backdrop-blur-xl rounded-2xl border border-gray-700/50 p-8 min-w-[380px] shadow-2xl">
              {/* Logo */}
              <div className="flex justify-center mb-6">
                <Logo size="lg" animated={false} />
              </div>

              {/* Loading steps */}
              <div className="space-y-3 mb-6">
                {[
                  { key: 'arena', label: 'Building Arena', icon: Box },
                  { key: 'replay', label: 'Processing Replay', icon: FileCode },
                  { key: 'models', label: 'Loading Car Models', icon: Car },
                  { key: 'shaders', label: 'Compiling Shaders', icon: Sparkles },
                  { key: 'environment', label: 'Applying Environment', icon: Layers },
                ].map((step) => {
                  // Map loading steps to their order (including hidden intermediate steps)
                  const stepOrder = ['arena', 'replay', 'models', 'shaders', 'ready', 'environment'];
                  const currentIndex = stepOrder.indexOf(loadingStep);
                  const stepIndex = stepOrder.indexOf(step.key);
                  const isCompleted = currentIndex > stepIndex;
                  const isCurrent = loadingStep === step.key;

                  return (
                    <div
                      key={step.key}
                      className={`flex items-center gap-3 p-3 rounded-lg transition-all duration-300 ${
                        isCurrent ? 'bg-violet-500/10 border border-violet-500/30' :
                        isCompleted ? 'bg-green-500/5 border border-transparent' :
                        'bg-gray-800/30 border border-transparent'
                      }`}
                    >
                      {/* Status indicator */}
                      <div className={`flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-all duration-300 ${
                        isCompleted ? 'bg-green-500/20' :
                        isCurrent ? 'bg-violet-500/20' :
                        'bg-gray-700/50'
                      }`}>
                        {isCompleted ? (
                          <Check className="w-4 h-4 text-green-400" />
                        ) : isCurrent ? (
                          <Loader2 className="w-4 h-4 text-violet-400 animate-spin" />
                        ) : (
                          <step.icon className="w-4 h-4 text-gray-500" />
                        )}
                      </div>

                      {/* Label */}
                      <span className={`text-sm font-medium transition-colors duration-300 ${
                        isCompleted ? 'text-green-400' :
                        isCurrent ? 'text-violet-300' :
                        'text-gray-500'
                      }`}>
                        {step.label}
                      </span>

                      {/* Checkmark for completed */}
                      {isCompleted && (
                        <span className="ml-auto text-xs text-green-400/70">Done</span>
                      )}
                    </div>
                  );
                })}
              </div>

              {/* Progress bar */}
              <div className="relative h-1.5 bg-gray-800 rounded-full overflow-hidden">
                <div
                  className="absolute inset-y-0 left-0 bg-gradient-to-r from-violet-500 to-blue-500 rounded-full transition-all duration-500"
                  style={{
                    width: `${
                      !loadingStep ? 5 :
                      loadingStep === 'arena' ? 15 :
                      loadingStep === 'replay' ? 35 :
                      loadingStep === 'models' ? 55 :
                      loadingStep === 'shaders' ? 75 :
                      loadingStep === 'ready' ? 85 :
                      loadingStep === 'environment' ? 95 :
                      100
                    }%`
                  }}
                />
                {/* Shimmer effect */}
                <div className="absolute inset-y-0 w-20 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" />
              </div>

              {/* Current message */}
              <p className="text-center text-gray-400 text-sm mt-4">
                {loadingMessage}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Error overlay */}
      {error && (
        <div className="absolute inset-0 z-50 flex items-center justify-center overflow-hidden">
          {/* Gradient background */}
          <div className="absolute inset-0 bg-gray-950" />
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-red-900/20 via-gray-950 to-gray-950" />

          {/* Error card */}
          <div className="relative z-10">
            <div className="absolute inset-0 bg-gradient-to-r from-red-600 to-orange-600 rounded-2xl blur-2xl opacity-20 scale-105" />
            <div className="relative bg-gray-900/80 backdrop-blur-xl rounded-2xl border border-red-500/30 p-8 max-w-md shadow-2xl">
              {/* Error icon */}
              <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-red-500/20 flex items-center justify-center">
                <svg className="w-8 h-8 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              </div>

              <h3 className="text-xl font-semibold text-white text-center mb-2">Loading Error</h3>
              <p className="text-red-300 text-center mb-6">{error}</p>

              <Link
                to="/replays"
                className="flex items-center justify-center gap-2 w-full py-3 px-4 rounded-lg bg-gradient-to-r from-violet-600 to-blue-600 text-white font-medium hover:from-violet-500 hover:to-blue-500 transition-all"
              >
                Back to replays
              </Link>
            </div>
          </div>
        </div>
      )}

      {/* Following indicator */}
      {followingId && participants[followingId] && (
        <div className="absolute top-20 left-1/2 -translate-x-1/2 z-40 pointer-events-auto">
          <div className="flex items-center gap-3 px-4 py-2 rounded-lg bg-violet-600/90 backdrop-blur-sm border border-violet-400/30 shadow-lg">
            <div className="flex items-center gap-2">
              <div
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: participants[followingId].color }}
              />
              <span className="text-white text-sm">
                Following <span className="font-medium">{participants[followingId].nickname}</span>
              </span>
            </div>
            <button
              onClick={() => handleFollowViewer(null)}
              className="px-2 py-1 text-xs bg-white/20 hover:bg-white/30 text-white rounded transition-colors"
            >
              Stop
            </button>
          </div>
        </div>
      )}

      {/* UI Overlay */}
      <GameOverlay
        isPlaying={isPlaying}
        currentTime={currentTime}
        maxTime={maxTime}
        gameTimeMap={gameTimeMap}
        countdownEvents={countdownEvents}
        onPlayPause={handlePlayPause}
        onSeek={handleSeek}
        onSeekCommit={handleSeekCommit}
        onEventClick={handleEventClick}
        cameraMode={cameraMode}
        onCameraModeChange={handleCameraModeChange}
        players={players}
        playerTeams={playerTeams}
        selectedPlayer={selectedPlayer}
        onPlayerSelect={handlePlayerSelect}
        cameraSettings={cameraSettings}
        onCameraSettingsChange={handleCameraSettingsChange}
        events={events}
        playerBoosts={playerBoosts}
        playerScores={playerScores}
        playbackSpeed={playbackSpeed}
        onPlaybackSpeedChange={handlePlaybackSpeedChange}
        textOverlays={textOverlays}
        playerCarInfo={playerCarInfo}
        controlsDisabled={controlsDisabled}
        isInSession={isInSession}
        participants={participants}
        selfId={selfId}
        hostId={hostId}
        isHost={isHost}
        followingId={followingId}
        onFollowViewer={handleFollowViewer}
        onKickParticipant={kickParticipant}
        onBanParticipant={banParticipant}
        onTransferHost={transferHost}
        onStartSession={() => {
          setActiveTab('collab');
          setDrawerOpen(true);
        }}
        currentEnvironmentId={customEnvironmentId}
        onEnvironmentChange={handleCustomEnvironmentChange}
        isLoadingEnvironment={isLoadingEnvironment}
        playerStatsTimelines={playerStatsTimelines}
        gameEventTimeline={gameEventTimeline}
        advancedStats={advancedStats}
        qualityScore={qualityScore}
        qualityMetrics={qualityMetrics}
        onCreateClip={handleOpenClipEditor}
        isClipEditorOpen={clipEditor.isOpen}
      />

      {/* Player Indicator - Only visible in player camera mode */}
      {cameraMode === 'player' && selectedPlayer && (
        <PlayerIndicator
          currentPlayer={selectedPlayer}
          playerTeam={playerTeams[selectedPlayer] ?? 0}
          players={players}
          playerTeams={playerTeams}
          onNavigate={handlePlayerSelect}
        />
      )}

      {/* Killfeed - demolition notifications */}
      <Killfeed entries={demoEvents} />

      {/* Chat Overlay - positioned on left side */}
      {isInSession && (
        <div className="absolute left-4 top-1/2 z-40 flex flex-col gap-1">
          <ChatOverlayMessages messages={chatMessages} />
          <ChatInput
            isOpen={isChatInputOpen}
            onClose={() => setIsChatInputOpen(false)}
            onSend={handleSendChatMessage}
            isConnected={isConnected}
          />
        </div>
      )}

      {/* Collaborative Tools Toolbar - centered bottom */}
      {isInSession && (
        <div className="absolute bottom-28 left-1/2 -translate-x-1/2 z-40">
          <ToolBar
            activeTool={toolState.activeTool}
            onToolChange={setTool}
            drawColor={toolState.drawColor}
            onColorChange={setDrawColor}
            drawThickness={toolState.drawThickness}
            onThicknessChange={setDrawThickness}
            canUndo={strokes.some(s => s.authorId === selfId)}
            onUndo={undoStroke}
            isHost={isHost}
            onClearAll={clearAllDrawings}
          />
        </div>
      )}

      {/* Dev Tools - bottom left */}
      <div className="absolute bottom-24 left-4 z-30 flex gap-2">
        {/* Debug Panel (admin only) */}
        {user?.isAdmin && (
        <DebugPanel
          actors={actors}
          playerTeams={playerTeams}
          ballActorId={ballActorId}
          currentTime={currentTime}
          frameInfo={frameInfo}
          interpolationEnabled={interpolationEnabled}
          onInterpolationToggle={handleInterpolationToggle}
          playerBoosts={playerBoosts}
          interpolationMethod={interpolationMethod}
          onInterpolationMethodChange={handleInterpolationMethodChange}
          smoothingWindowSize={smoothingWindowSize}
          onSmoothingWindowSizeChange={handleSmoothingWindowSizeChange}
          isPlaying={isPlaying}
          ballTimeline={ballTimeline}
          playerTimelines={playerTimelines}
          playbackSpeed={playbackSpeed}
          onPlaybackSpeedChange={handlePlaybackSpeedChange}
          onSeek={handleSeek}
          onPlayPause={handlePlayPause}
          onLoadReplayFile={handleLoadReplayFile}
        />
        )}

        {/* DevTools (admin only) */}
        {user?.isAdmin && (
          <DevToolsPanel
            devToolsManager={devToolsManager as never}
            onSkyboxChange={async (assetId: string) => {
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              const game = gameRef.current as any;
              if (game?.environmentManager) {
                // Load skybox with current exposure (from renderer)
                const exposure = game.sceneManager?.renderer?.toneMappingExposure ?? 1.0;
                await game.environmentManager.loadSkybox(assetId, exposure);
              }
            }}
          />
        )}
      </div>

      {/* DRAWER WITH ATTACHED TABS - Hidden when clip editor is open */}
      {!clipEditor.isOpen && (
      <div
        className={`absolute top-16 right-0 bottom-0 z-40 flex transition-transform duration-300 ease-in-out ${
          drawerOpen ? 'translate-x-0' : 'translate-x-80'
        }`}
      >
        {/* Tab buttons - attached to drawer left edge */}
        <div className="flex flex-col justify-center gap-1 -ml-10">
          {/* Close button - only visible when drawer is open */}
          <button
            onClick={() => setDrawerOpen(false)}
            className={`relative flex items-center gap-1 px-2 py-3 bg-gray-900/90 backdrop-blur border border-gray-800 border-r-0 rounded-l-lg text-gray-300 hover:text-white hover:bg-gray-800 transition-all ${
              drawerOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
            }`}
          >
            <ChevronRight className="w-4 h-4" />
          </button>

          {/* Events tab button */}
          <button
            onClick={() => {
              if (drawerOpen && activeTab === 'events') {
                setDrawerOpen(false);
              } else {
                setActiveTab('events');
                setDrawerOpen(true);
              }
            }}
            className={`relative flex items-center gap-1 px-2 py-3 bg-gray-900/90 backdrop-blur border border-gray-800 border-r-0 rounded-l-lg text-gray-300 hover:text-white hover:bg-gray-800 transition-all ${
              activeTab === 'events' && drawerOpen ? 'bg-gray-800 text-white border-l-2 border-l-blue-500' : ''
            }`}
            title="Events"
          >
            <Calendar className="w-4 h-4" />
            {events.length > 0 && (
              <span className="absolute -top-1 -left-1 w-5 h-5 bg-blue-500 rounded-full text-xs flex items-center justify-center text-white">
                {events.length}
              </span>
            )}
          </button>

          {/* Camera tab button */}
          <button
            onClick={() => {
              if (drawerOpen && activeTab === 'camera') {
                setDrawerOpen(false);
              } else {
                setActiveTab('camera');
                setDrawerOpen(true);
              }
            }}
            className={`relative flex items-center gap-1 px-2 py-3 bg-gray-900/90 backdrop-blur border border-gray-800 border-r-0 rounded-l-lg text-gray-300 hover:text-white hover:bg-gray-800 transition-all ${
              activeTab === 'camera' && drawerOpen ? 'bg-gray-800 text-white border-l-2 border-l-cyan-500' : ''
            }`}
            title="Camera"
          >
            <Camera className="w-4 h-4" />
          </button>

          {/* Settings tab button */}
          <button
            onClick={() => {
              if (drawerOpen && activeTab === 'settings') {
                setDrawerOpen(false);
              } else {
                setActiveTab('settings');
                setDrawerOpen(true);
              }
            }}
            className={`relative flex items-center gap-1 px-2 py-3 bg-gray-900/90 backdrop-blur border border-gray-800 border-r-0 rounded-l-lg text-gray-300 hover:text-white hover:bg-gray-800 transition-all ${
              activeTab === 'settings' && drawerOpen ? 'bg-gray-800 text-white border-l-2 border-l-amber-500' : ''
            }`}
            title="Settings"
          >
            <Settings className="w-4 h-4" />
          </button>

          {/* Collab tab button */}
          <button
            onClick={() => {
              if (drawerOpen && activeTab === 'collab') {
                setDrawerOpen(false);
              } else {
                setActiveTab('collab');
                setDrawerOpen(true);
              }
            }}
            className={`relative flex items-center gap-1 px-2 py-3 bg-gray-900/90 backdrop-blur border border-gray-800 border-r-0 rounded-l-lg text-gray-300 hover:text-white hover:bg-gray-800 transition-all ${
              activeTab === 'collab' && drawerOpen ? 'bg-gray-800 text-white border-l-2 border-l-violet-500' : ''
            }`}
            title="Watch Together"
          >
            <Users className="w-4 h-4" />
            {isInSession && participantCount > 0 && (
              <span className="absolute -top-1 -left-1 w-5 h-5 bg-violet-500 rounded-full text-xs flex items-center justify-center text-white">
                {participantCount}
              </span>
            )}
          </button>
        </div>

        {/* Drawer content */}
        <div className="w-80 h-full bg-gray-900/95 backdrop-blur border-l border-gray-800 flex flex-col">
          {/* Header with title and close button */}
          <div className="flex items-center justify-between px-4 py-3 border-b border-gray-800">
            <div className="flex items-center gap-2">
              {activeTab === 'events' && (
                <>
                  <Calendar className="w-5 h-5 text-blue-400" />
                  <span className="text-white font-medium">Events</span>
                  {events.length > 0 && (
                    <span className="px-1.5 py-0.5 bg-blue-500 rounded text-xs text-white">
                      {events.length}
                    </span>
                  )}
                </>
              )}
              {activeTab === 'camera' && (
                <>
                  <Camera className="w-5 h-5 text-cyan-400" />
                  <span className="text-white font-medium">Camera</span>
                </>
              )}
              {activeTab === 'settings' && (
                <>
                  <Settings className="w-5 h-5 text-amber-400" />
                  <span className="text-white font-medium">Settings</span>
                </>
              )}
              {activeTab === 'collab' && (
                <>
                  <Users className="w-5 h-5 text-violet-400" />
                  <span className="text-white font-medium">Watch Together</span>
                  {isInSession && participantCount > 0 && (
                    <span className="px-1.5 py-0.5 bg-violet-500 rounded text-xs text-white">
                      {participantCount}
                    </span>
                  )}
                </>
              )}
            </div>
            <button
              onClick={() => setDrawerOpen(false)}
              className="p-1 rounded hover:bg-gray-800 text-gray-400 hover:text-white transition-colors"
            >
              <ChevronRight className="w-5 h-5" />
            </button>
          </div>

          {/* Tab content */}
          <div className="flex-1 overflow-y-auto">
            {/* Events Tab */}
            {activeTab === 'events' && (
              <div className="p-4 space-y-2">
                {events.map((event, idx) => {
                  const isPast = currentTime > event.time;
                  const eventColor = (event as { color?: string }).color || '#888';
                  let Icon = IoMdFootball;
                  if (event.type === 'save') Icon = GiShieldImpact;
                  if (event.type === 'demo') Icon = GiMineExplosion;

                  return (
                    <div
                      key={idx}
                      className={`bg-gray-800/50 p-3 rounded cursor-pointer hover:bg-gray-700 transition-all border-l-4 flex items-start gap-3 ${
                        isPast ? 'opacity-40' : 'opacity-100'
                      }`}
                      style={{ borderLeftColor: eventColor }}
                      onClick={() => handleEventClick(event)}
                    >
                      <Icon size={24} color={eventColor} className="flex-shrink-0 mt-0.5" />
                      <div className="flex-1 min-w-0">
                        <div className="flex justify-between text-gray-400 text-xs mb-1">
                          <span className="font-mono">
                            {Math.floor(event.time / 60)}:{Math.floor(event.time % 60).toString().padStart(2, '0')}
                          </span>
                        </div>
                        <div className="text-white text-sm font-medium">
                          {(event as { description?: string }).description || event.type}
                        </div>
                      </div>
                    </div>
                  );
                })}
                {events.length === 0 && (
                  <div className="text-gray-500 text-center py-8 text-sm">
                    No events found
                  </div>
                )}
              </div>
            )}

            {/* Camera Tab */}
            {activeTab === 'camera' && (
              <div className="p-4 space-y-4">
                {/* Camera Mode Buttons */}
                <div className="space-y-2">
                  <label className="text-sm text-gray-400">Mode</label>
                  <div className="flex flex-wrap gap-2">
                    <button
                      onClick={() => handleCameraModeChange('free')}
                      className={`flex-1 min-w-[70px] px-3 py-2 rounded font-medium transition-colors text-sm ${
                        cameraMode === 'free'
                          ? 'bg-cyan-600 text-white'
                          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                      }`}
                    >
                      Free
                    </button>
                    <button
                      onClick={() => handleCameraModeChange('player')}
                      className={`flex-1 min-w-[70px] px-3 py-2 rounded font-medium transition-colors text-sm ${
                        cameraMode === 'player'
                          ? 'bg-cyan-600 text-white'
                          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                      }`}
                    >
                      Player
                    </button>
                    <button
                      onClick={() => handleCameraModeChange('ballOrbit')}
                      className={`flex-1 min-w-[70px] px-3 py-2 rounded font-medium transition-colors text-sm ${
                        cameraMode === 'ballOrbit'
                          ? 'bg-cyan-600 text-white'
                          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                      }`}
                    >
                      Ball
                    </button>
                  </div>
                </div>

                {/* FOV Slider */}
                <div className="space-y-1">
                  <div className="flex justify-between text-xs text-gray-300">
                    <span>FOV</span>
                    <span>{cameraSettings.fov || 110}°</span>
                  </div>
                  <Slider
                    min={60} max={110} step={1}
                    value={[cameraSettings.fov || 110]}
                    onValueChange={(v) => handleCameraSettingsChange({ ...cameraSettings, fov: v[0] })}
                  />
                </div>

                {/* FreeCam Speed */}
                {cameraMode === 'free' && (
                  <div className="space-y-1">
                    <div className="flex justify-between text-xs text-gray-300">
                      <span>Move Speed</span>
                      <span>{cameraSettings.freeCamSpeed || 2000}</span>
                    </div>
                    <Slider
                      min={100} max={10000} step={100}
                      value={[cameraSettings.freeCamSpeed || 2000]}
                      onValueChange={(v) => handleCameraSettingsChange({ ...cameraSettings, freeCamSpeed: v[0] })}
                    />
                  </div>
                )}

                {/* Player Camera Settings */}
                {cameraMode === 'player' && (
                  <div className="space-y-3">
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs text-gray-300">
                        <span>Distance</span>
                        <span>{cameraSettings.distance?.toFixed(0)}</span>
                      </div>
                      <Slider
                        min={100} max={400} step={10}
                        value={[cameraSettings.distance || 260]}
                        onValueChange={(v) => handleCameraSettingsChange({ ...cameraSettings, distance: v[0] })}
                      />
                    </div>
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs text-gray-300">
                        <span>Height</span>
                        <span>{cameraSettings.height?.toFixed(0)}</span>
                      </div>
                      <Slider
                        min={40} max={200} step={10}
                        value={[cameraSettings.height || 90]}
                        onValueChange={(v) => handleCameraSettingsChange({ ...cameraSettings, height: v[0] })}
                      />
                    </div>
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs text-gray-300">
                        <span>Angle</span>
                        <span>{cameraSettings.angle?.toFixed(1)}°</span>
                      </div>
                      <Slider
                        min={-15} max={0} step={0.5}
                        value={[cameraSettings.angle ?? -4]}
                        onValueChange={(v) => handleCameraSettingsChange({ ...cameraSettings, angle: v[0] })}
                      />
                    </div>
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs text-gray-300">
                        <span>Stiffness</span>
                        <span>{cameraSettings.stiffness?.toFixed(2)}</span>
                      </div>
                      <Slider
                        min={0} max={1} step={0.05}
                        value={[cameraSettings.stiffness ?? 0.45]}
                        onValueChange={(v) => handleCameraSettingsChange({ ...cameraSettings, stiffness: v[0] })}
                      />
                    </div>
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs text-gray-300">
                        <span>Swivel Speed</span>
                        <span>{cameraSettings.swivelSpeed?.toFixed(1)}</span>
                      </div>
                      <Slider
                        min={1} max={10} step={0.1}
                        value={[cameraSettings.swivelSpeed || 4.3]}
                        onValueChange={(v) => handleCameraSettingsChange({ ...cameraSettings, swivelSpeed: v[0] })}
                      />
                    </div>
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs text-gray-300">
                        <span>Transition Speed</span>
                        <span>{cameraSettings.transitionSpeed?.toFixed(2)}</span>
                      </div>
                      <Slider
                        min={1} max={2} step={0.05}
                        value={[cameraSettings.transitionSpeed || 1.30]}
                        onValueChange={(v) => handleCameraSettingsChange({ ...cameraSettings, transitionSpeed: v[0] })}
                      />
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Settings Tab */}
            {activeTab === 'settings' && (
              <div className="p-4 space-y-6">
                {/* Environment Section */}
                <div className="space-y-3">
                  <h3 className="text-white font-semibold text-sm uppercase tracking-wider">
                    Environment
                  </h3>
                  <EnvironmentSelector
                    currentEnvironmentId={customEnvironmentId}
                    onEnvironmentChange={handleCustomEnvironmentChange}
                    disabled={isLoadingEnvironment || (isInSession && !isHost)}
                  />
                  {isLoadingEnvironment && (
                    <div className="flex items-center gap-2 text-xs text-gray-400">
                      <Loader2 size={12} className="animate-spin" />
                      <span>Loading environment...</span>
                    </div>
                  )}
                  {isInSession && !isHost && !isLoadingEnvironment && (
                    <p className="text-xs text-yellow-500/80 flex items-center gap-1">
                      <Lock size={10} />
                      Only the host can change the environment
                    </p>
                  )}
                  {(!isInSession || isHost) && (
                    <p className="text-xs text-gray-500">
                      Select a custom environment with meshes, lights, and skybox
                    </p>
                  )}
                </div>

                {/* Speed Display Section */}
                <div className="space-y-3">
                  <h3 className="text-white font-semibold text-sm uppercase tracking-wider">
                    Speed Display
                  </h3>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-300">Show Ball Speed</span>
                    <button
                      onClick={() => updateSettings({ showBallSpeed: !settings.showBallSpeed })}
                      className={`relative w-10 h-5 rounded-full transition-colors ${
                        settings.showBallSpeed ? 'bg-amber-600' : 'bg-gray-600'
                      }`}
                    >
                      <span
                        className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                          settings.showBallSpeed ? 'translate-x-5' : 'translate-x-0'
                        }`}
                      />
                    </button>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-300">Show Car Speed</span>
                    <button
                      onClick={() => updateSettings({ showCarSpeed: !settings.showCarSpeed })}
                      className={`relative w-10 h-5 rounded-full transition-colors ${
                        settings.showCarSpeed ? 'bg-amber-600' : 'bg-gray-600'
                      }`}
                    >
                      <span
                        className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                          settings.showCarSpeed ? 'translate-x-5' : 'translate-x-0'
                        }`}
                      />
                    </button>
                  </div>
                  <div className="space-y-2">
                    <label className="text-sm text-gray-300">Speed Unit</label>
                    <div className="flex gap-2">
                      <button
                        onClick={() => updateSettings({ speedUnit: 'kmh' })}
                        className={`flex-1 py-2 px-4 rounded text-sm font-medium transition-colors ${
                          settings.speedUnit === 'kmh'
                            ? 'bg-amber-600 text-white'
                            : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                        }`}
                      >
                        km/h
                      </button>
                      <button
                        onClick={() => updateSettings({ speedUnit: 'mph' })}
                        className={`flex-1 py-2 px-4 rounded text-sm font-medium transition-colors ${
                          settings.speedUnit === 'mph'
                            ? 'bg-amber-600 text-white'
                            : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                        }`}
                      >
                        mph
                      </button>
                    </div>
                  </div>
                </div>

                {/* Debug Section */}
                <div className="space-y-3">
                  <h3 className="text-white font-semibold text-sm uppercase tracking-wider">
                    Debug
                  </h3>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-300">Show Car Hitboxes</span>
                    <button
                      onClick={() => updateSettings({ showHitboxes: !settings.showHitboxes })}
                      className={`relative w-10 h-5 rounded-full transition-colors ${
                        settings.showHitboxes ? 'bg-amber-600' : 'bg-gray-600'
                      }`}
                    >
                      <span
                        className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                          settings.showHitboxes ? 'translate-x-5' : 'translate-x-0'
                        }`}
                      />
                    </button>
                  </div>
                </div>
              </div>
            )}

            {/* Collab Tab */}
            {activeTab === 'collab' && (
              <div className="p-4">
                {!isInSession ? (
                  <SessionPanel replayId={replayId} />
                ) : (
                  <CollabOverlay inDrawer />
                )}
              </div>
            )}
          </div>
        </div>
      </div>
      )}

      {/* Feedback Popup - shown when playback ends */}
      <FeedbackPopup
        replayId={replayId}
        isVisible={showFeedbackPopup}
        onClose={() => setShowFeedbackPopup(false)}
      />

      {/* Clip Editor (024-clip-system) */}
      {clipEditor.isOpen && (
        <ClipEditor
          editor={clipEditor}
          currentTime={currentTime}
          maxTime={maxTime}
          isPlaying={isPlaying}
          onSeek={handleSeek}
          onPlayPause={handlePlayPause}
          onStartRecording={handleStartClipRecording}
          onStopRecording={handleStopClipRecording}
          onStartPreview={handleStartClipPreview}
          onStopPreview={handleStopClipPreview}
          onClose={handleCloseClipEditor}
          onClipSaved={handleClipSaved}
          onCaptureFrame={handleCaptureFrame}
          onAddKeyframe={handleAddKeyframe}
          onRemoveKeyframe={handleRemoveKeyframe}
          onViewKeyframe={handleViewKeyframe}
          onUpdateKeyframe={handleUpdateKeyframe}
          onSetActiveKeyframe={handleSetActiveKeyframe}
          onToggleAllMarkers={handleToggleAllMarkers}
          onClearAllKeyframes={handleClearAllKeyframes}
          onUpdateKeyframeTime={handleUpdateKeyframeTime}
        />
      )}

      {/* Restore Draft Dialog (026-clip-editor-redesign T016) */}
      {showRestoreDraftDialog && (
        <RestoreDraftDialog
          onRestore={handleRestoreDraft}
          onDiscard={handleDiscardDraftAndOpen}
          onCancel={handleCancelRestoreDialog}
        />
      )}
    </div>
  );
}

// Restore Draft Dialog Component (026-clip-editor-redesign T016)
interface RestoreDraftDialogProps {
  onRestore: () => void;
  onDiscard: () => void;
  onCancel: () => void;
}

function RestoreDraftDialog({ onRestore, onDiscard, onCancel }: RestoreDraftDialogProps) {
  // Handle Escape to cancel
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onCancel();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onCancel]);

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl p-6 max-w-md w-full mx-4 space-y-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-full bg-blue-500/20 flex items-center justify-center">
            <FileCode className="w-5 h-5 text-blue-400" />
          </div>
          <h4 className="text-lg font-semibold text-white">Draft Found</h4>
        </div>

        <p className="text-zinc-300">
          You have an unsaved clip draft for this replay. Would you like to restore it and continue editing?
        </p>

        <div className="flex items-center justify-end gap-3 pt-2">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-zinc-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={onDiscard}
            className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg font-medium transition-colors"
          >
            Start Fresh
          </button>
          <button
            onClick={onRestore}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors"
          >
            Restore Draft
          </button>
        </div>
      </div>
    </div>
  );
}

// Main Viewer component - provides CollabProvider context
export default function Viewer() {
  const { id } = useParams<{ id: string }>();

  if (!id) {
    return (
      <div className="fixed inset-0 bg-gray-950 flex items-center justify-center">
        <div className="text-center">
          <p className="text-red-400 mb-4">Invalid replay ID</p>
          <Link to="/replays" className="text-violet-400 hover:text-violet-300 underline">
            Back to replays
          </Link>
        </div>
      </div>
    );
  }

  return (
    <CollabProvider>
      <ViewerContent replayId={id} />
    </CollabProvider>
  );
}
