/**
 * ClipViewer Page
 *
 * View a shared clip with playback controls, like, and share functionality.
 *
 * Feature: 024-clip-system (US3 - Sharing)
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { useParams, Link } from 'react-router-dom';
import {
  Loader2,
  ArrowLeft,
  Heart,
  Share2,
  Eye,
  User,
  Film,
  Video,
  Clock,
  AlertCircle,
  Pencil,
  Play,
  Pause,
  RotateCcw,
} from 'lucide-react';
import { GameEngine } from '@/game/GameEngine';
import * as clipsApi from '@/api/clips';
import type { ClipWithDetails } from '@/api/clips';
import { useAuth } from '@/hooks/useAuth';
import { cn } from '@/lib/utils';
import { formatClipDuration } from '@/api/clips';
import { SEOHead } from '@/components/SEO/SEOHead';
import { useClipSEO } from '@/hooks/useSEO';

// Format seconds to MM:SS
function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

export default function ClipViewer() {
  const { id } = useParams<{ id: string }>();
  const { user } = useAuth();

  // State
  const [clip, setClip] = useState<ClipWithDetails | null>(null);
  const [replayBinary, setReplayBinary] = useState<ArrayBuffer | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingStatus, setLoadingStatus] = useState('Loading clip...');
  const [error, setError] = useState<string | null>(null);
  const [isLiked, setIsLiked] = useState(false);
  const [likeCount, setLikeCount] = useState(0);
  const [viewRecorded, setViewRecorded] = useState(false);
  const [copied, setCopied] = useState(false);

  // Game engine refs
  const containerRef = useRef<HTMLDivElement>(null);
  const gameRef = useRef<GameEngine | null>(null);
  const [gameReady, setGameReady] = useState(false);

  // SEO
  const seo = useClipSEO(clip);

  // Playback state
  const [isPlaying, setIsPlaying] = useState(false);
  const [hasEnded, setHasEnded] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [clipDuration, setClipDuration] = useState(0);
  const [followedPlayerName, setFollowedPlayerName] = useState<string | null>(null);
  const timeUpdateIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Fetch clip data and replay binary
  useEffect(() => {
    async function fetchData() {
      if (!id) return;

      setLoading(true);
      setError(null);

      try {
        // 1. Fetch clip metadata
        setLoadingStatus('Loading clip data...');
        const response = await clipsApi.getClip(id);
        setClip(response.clip);
        setIsLiked(response.clip.isLiked ?? false);
        setLikeCount(response.clip.likeCount);
        setClipDuration(response.clip.endTime - response.clip.startTime);

        // 2. Fetch replay binary (compiled .rlrf format)
        setLoadingStatus('Loading replay...');
        const apiUrl = import.meta.env.VITE_API_URL || '/api';
        const replayResponse = await fetch(`${apiUrl}/replays/${response.clip.replayId}/binary`);
        if (!replayResponse.ok) {
          if (replayResponse.status === 404) {
            throw new Error('REPLAY_NOT_FOUND');
          }
          throw new Error('Failed to fetch replay');
        }

        const arrayBuffer = await replayResponse.arrayBuffer();
        setReplayBinary(arrayBuffer);
        setLoadingStatus('Initializing viewer...');
      } catch (err) {
        console.error('[ClipViewer] Failed to fetch data:', err);
        const errorMsg = (err as Error).message;
        if (errorMsg === 'REPLAY_NOT_FOUND') {
          setError('The replay associated with this clip is no longer available.');
        } else {
          setError('Clip not found or unavailable');
        }
        setLoading(false);
      }
    }

    fetchData();
  }, [id]);

  // Record view
  useEffect(() => {
    async function recordView() {
      if (!id || viewRecorded) return;

      try {
        await clipsApi.recordView(id);
        setViewRecorded(true);
      } catch (err) {
        console.error('[ClipViewer] Failed to record view:', err);
      }
    }

    // Record view after a short delay
    const timer = setTimeout(recordView, 2000);
    return () => clearTimeout(timer);
  }, [id, viewRecorded]);

  // Handle clip playback end
  const handleClipEnd = useCallback(() => {
    console.log('[ClipViewer] Clip ended');
    setIsPlaying(false);
    setHasEnded(true);
  }, []);

  // Initialize game engine only when we have both clip data and replay binary
  useEffect(() => {
    if (!containerRef.current || !clip || !replayBinary) return;

    console.log('[ClipViewer] Initializing game engine with binary data');

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = new GameEngine(containerRef.current, {
      binaryData: replayBinary,
      onTimeUpdate: () => {},
      onPlayStateChange: (playing: boolean) => {
        setIsPlaying(playing);
      },
      onClipPlaybackEnd: handleClipEnd,
      onReady: () => {
        console.log('[ClipViewer] Game engine ready');
        setGameReady(true);
        setLoading(false);
      },
      onLoadingProgress: (_step: string, message: string) => {
        setLoadingStatus(message);
      },
      onError: (err: Error) => {
        console.error('[ClipViewer] GameEngine error:', err);
        setError(err.message || 'Failed to initialize viewer');
        setLoading(false);
      },
    });

    gameRef.current = game;

    return () => {
      console.log('[ClipViewer] Disposing game engine');
      game.dispose();
      gameRef.current = null;
      setGameReady(false);
    };
  }, [clip, replayBinary, handleClipEnd]);

  // Track current time and followed player during playback
  useEffect(() => {
    if (isPlaying && gameRef.current) {
      // Start interval to update current time and followed player
      timeUpdateIntervalRef.current = setInterval(() => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const game = gameRef.current as any;
        const info = game?.getClipPlaybackInfo?.();
        if (info) {
          setCurrentTime(info.currentTime);
          setFollowedPlayerName(info.followedPlayerName ?? null);
        }
      }, 100);
    } else {
      // Clear interval when not playing
      if (timeUpdateIntervalRef.current) {
        clearInterval(timeUpdateIntervalRef.current);
        timeUpdateIntervalRef.current = null;
      }
    }

    return () => {
      if (timeUpdateIntervalRef.current) {
        clearInterval(timeUpdateIntervalRef.current);
      }
    };
  }, [isPlaying]);

  // Load environment and start clip playback when game is ready
  useEffect(() => {
    if (!gameReady || !clip) return;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    // Load environment first, then start playback
    const initPlayback = async () => {
      // Load the clip's environment or fall back to default
      if (game.environmentManager) {
        try {
          if (clip.environment?.id) {
            // Load the specific environment saved with this clip
            console.log('[ClipViewer] Loading clip environment:', clip.environment.name);
            await game.environmentManager.loadEnvironment(clip.environment.id);
          } else {
            // Fall back to default environment
            console.log('[ClipViewer] Loading default environment...');
            await game.environmentManager.loadDefaultEnvironment();
          }
          console.log('[ClipViewer] Environment loaded');
        } catch (err) {
          console.warn('[ClipViewer] Failed to load environment:', err);
        }
      }

      // Start clip playback
      console.log('[ClipViewer] Starting clip playback at', clip.startTime);
      game.startClipPlayback(clip.cameraData, clip.startTime);
      setIsPlaying(true);
      setHasEnded(false);
    };

    initPlayback();
  }, [gameReady, clip]);

  // Playback controls
  const handlePlayPause = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    if (hasEnded) {
      // If ended, replay from start
      game.replayClip();
      setHasEnded(false);
      setIsPlaying(true);
      setCurrentTime(0);
    } else if (isPlaying) {
      game.pause();
      game.clipPlaybackManager?.pause();
      setIsPlaying(false);
    } else {
      game.resumeClipPlayback();
      setIsPlaying(true);
    }
  }, [isPlaying, hasEnded]);

  const handleReplay = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    game.replayClip();
    setHasEnded(false);
    setIsPlaying(true);
    setCurrentTime(0);
  }, []);

  const handleSeek = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    const newTime = parseFloat(e.target.value);
    game.seekClip(newTime);
    setCurrentTime(newTime);

    // Update followed player name after seek
    const info = game.getClipPlaybackInfo?.();
    if (info) {
      setFollowedPlayerName(info.followedPlayerName ?? null);
    }

    // If was ended and seeking, reset ended state
    if (hasEnded) {
      setHasEnded(false);
    }
  }, [hasEnded]);

  // Handle like
  const handleLike = useCallback(async () => {
    if (!user || !id) return;

    // Optimistic update
    setIsLiked((prev) => !prev);
    setLikeCount((prev) => (isLiked ? prev - 1 : prev + 1));

    try {
      const result = await clipsApi.toggleLike(id);
      setIsLiked(result.liked);
      setLikeCount(result.likeCount);
    } catch (err) {
      // Revert on error
      setIsLiked((prev) => !prev);
      setLikeCount((prev) => (isLiked ? prev + 1 : prev - 1));
      console.error('[ClipViewer] Failed to toggle like:', err);
    }
  }, [user, id, isLiked]);

  // Handle share
  const handleShare = useCallback(() => {
    const url = window.location.href;
    navigator.clipboard.writeText(url).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, []);

  if (loading && !clip) {
    return (
      <div className="h-[calc(100vh-4rem)] bg-zinc-900 flex flex-col items-center justify-center gap-4">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
        <p className="text-zinc-400 text-sm">{loadingStatus}</p>
      </div>
    );
  }

  if (error || !clip) {
    return (
      <div className="h-[calc(100vh-4rem)] bg-zinc-900 flex flex-col items-center justify-center gap-4">
        <AlertCircle className="w-16 h-16 text-red-500" />
        <h2 className="text-xl font-semibold text-white">{error || 'Clip not found'}</h2>
        <Link
          to="/clips"
          className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg transition-colors"
        >
          Back to Clips
        </Link>
      </div>
    );
  }

  const duration = formatClipDuration(clip.startTime, clip.endTime);
  const progress = clipDuration > 0 ? (currentTime / clipDuration) * 100 : 0;

  return (
    <div className="h-[calc(100vh-4rem)] bg-zinc-900 flex flex-col overflow-hidden">
      <SEOHead {...seo} type="video.other" />

      {/* Header */}
      <header className="bg-zinc-800/90 backdrop-blur-sm border-b border-zinc-700 py-3 px-4 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Link
            to="/clips"
            className="p-2 hover:bg-zinc-700 rounded-lg transition-colors"
          >
            <ArrowLeft className="w-5 h-5 text-zinc-400" />
          </Link>
          <div>
            <h1 className="text-lg font-semibold text-white line-clamp-1">{clip.title}</h1>
            <div className="flex items-center gap-3 text-sm text-zinc-400">
              <div className="flex items-center gap-1">
                {clip.creator.avatarUrl ? (
                  <img
                    src={clip.creator.avatarUrl}
                    alt={clip.creator.username}
                    className="w-4 h-4 rounded-full"
                  />
                ) : (
                  <User className="w-4 h-4" />
                )}
                <span>{clip.creator.username}</span>
              </div>
              <span>•</span>
              <div className="flex items-center gap-1">
                {clip.cameraMode === 'capture' ? (
                  <Video className="w-3 h-3" />
                ) : (
                  <Film className="w-3 h-3" />
                )}
                <span className="capitalize">{clip.cameraMode}</span>
              </div>
              <span>•</span>
              <div className="flex items-center gap-1">
                <Clock className="w-3 h-3" />
                <span>{duration}</span>
              </div>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* Edit button (owner only) */}
          {user && user.id === clip.creator.id && (
            <Link
              to={`/clips/${clip.id}/edit`}
              className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm font-medium text-zinc-400 hover:text-white hover:bg-zinc-700 transition-colors"
            >
              <Pencil className="w-4 h-4" />
              <span>Edit</span>
            </Link>
          )}

          {/* View count */}
          <div className="flex items-center gap-1 px-3 py-1.5 text-zinc-400">
            <Eye className="w-4 h-4" />
            <span className="text-sm">{clip.viewCount}</span>
          </div>

          {/* Like button */}
          <button
            onClick={handleLike}
            disabled={!user}
            className={cn(
              "flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors",
              isLiked
                ? "bg-red-500/20 text-red-400 border border-red-500/50"
                : "text-zinc-400 hover:text-white hover:bg-zinc-700",
              !user && "opacity-50 cursor-not-allowed"
            )}
            title={user ? (isLiked ? 'Unlike' : 'Like') : 'Login to like'}
          >
            <Heart className={cn("w-4 h-4", isLiked && "fill-current")} />
            <span>{likeCount}</span>
          </button>

          {/* Share button */}
          <button
            onClick={handleShare}
            className={cn(
              "flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors",
              copied
                ? "bg-green-500/20 text-green-400 border border-green-500/50"
                : "text-zinc-400 hover:text-white hover:bg-zinc-700"
            )}
          >
            <Share2 className="w-4 h-4" />
            <span>{copied ? 'Copied!' : 'Share'}</span>
          </button>
        </div>
      </header>

      {/* Viewer - fills available space, GameEngine handles aspect ratio */}
      <div className="flex-1 relative bg-black">
        <div ref={containerRef} className="absolute inset-0" />

        {!gameReady && (
          <div className="absolute inset-0 flex items-center justify-center bg-zinc-900">
            <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
          </div>
        )}

        {/* Ended overlay with replay button */}
        {hasEnded && gameReady && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/50">
            <button
              onClick={handleReplay}
              className="flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-lg font-medium transition-colors"
            >
              <RotateCcw className="w-5 h-5" />
              Replay
            </button>
          </div>
        )}

        {/* Followed player indicator */}
        {gameReady && followedPlayerName && (
          <div className="absolute bottom-4 left-4 bg-black/70 backdrop-blur-sm px-3 py-1.5 rounded-lg flex items-center gap-2 pointer-events-none">
            <User className="w-4 h-4 text-zinc-400" />
            <span className="text-sm font-medium text-white">{followedPlayerName}</span>
          </div>
        )}
      </div>

      {/* Playback Controls */}
      {gameReady && (
        <div className="bg-zinc-800/90 backdrop-blur-sm border-t border-zinc-700 py-3 px-4">
          <div className="flex items-center gap-4">
            {/* Play/Pause button */}
            <button
              onClick={handlePlayPause}
              className="p-2 hover:bg-zinc-700 rounded-lg transition-colors text-white"
            >
              {isPlaying ? (
                <Pause className="w-5 h-5" />
              ) : hasEnded ? (
                <RotateCcw className="w-5 h-5" />
              ) : (
                <Play className="w-5 h-5" />
              )}
            </button>

            {/* Time display */}
            <span className="text-sm text-zinc-400 font-mono min-w-[80px]">
              {formatTime(currentTime)} / {formatTime(clipDuration)}
            </span>

            {/* Timeline slider */}
            <div className="flex-1 relative">
              <input
                type="range"
                min={0}
                max={clipDuration}
                step={0.1}
                value={currentTime}
                onChange={handleSeek}
                className="w-full h-1 bg-zinc-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
                style={{
                  background: `linear-gradient(to right, #3b82f6 ${progress}%, #3f3f46 ${progress}%)`,
                }}
              />
            </div>

            {/* Replay button */}
            <button
              onClick={handleReplay}
              className="p-2 hover:bg-zinc-700 rounded-lg transition-colors text-zinc-400 hover:text-white"
              title="Replay from start"
            >
              <RotateCcw className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}

      {/* Footer with clip info */}
      {clip.description && (
        <div className="bg-zinc-800/90 border-t border-zinc-700 py-4 px-6">
          <p className="text-zinc-300 whitespace-pre-wrap">{clip.description}</p>
        </div>
      )}
    </div>
  );
}
