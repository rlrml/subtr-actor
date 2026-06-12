/**
 * ClipEdit Page
 *
 * Edit an existing clip - modify title/description, re-record camera, or edit keyframes.
 *
 * Feature: 024-clip-system (US4 - Re-record and Edit)
 * Tasks: T077-T081
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import {
  Loader2,
  ArrowLeft,
  Save,
  X,
  RefreshCw,
  Film,
  Video,
  Play,
  Square,
  Plus,
  Trash2,
  AlertCircle,
} from 'lucide-react';
import { GameEngine } from '@/game/GameEngine';
import * as clipsApi from '@/api/clips';
import type { ClipWithDetails, CameraData, CameraKeyframe, CameraKeyframes } from '@/api/clips';
import { useAuth } from '@/hooks/useAuth';
import { cn } from '@/lib/utils';
import { toast } from 'sonner';

type EditState = 'idle' | 'recording' | 'preview';

export default function ClipEdit() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { user, isLoading: authLoading } = useAuth();

  // Clip data
  const [clip, setClip] = useState<ClipWithDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Edit state
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [cameraData, setCameraData] = useState<CameraData | null>(null);
  const [keyframes, setKeyframes] = useState<CameraKeyframe[]>([]);
  const [selectedKeyframeId, setSelectedKeyframeId] = useState<string | null>(null);
  const [editState, setEditState] = useState<EditState>('idle');
  const [saving, setSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  // Game engine
  const containerRef = useRef<HTMLDivElement>(null);
  const gameRef = useRef<GameEngine | null>(null);
  const [gameReady, setGameReady] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [replayBinary, setReplayBinary] = useState<ArrayBuffer | null>(null);

  // Fetch clip data and replay binary
  useEffect(() => {
    async function fetchClipAndReplay() {
      if (!id) return;

      setLoading(true);
      setError(null);

      try {
        // 1. Fetch clip metadata
        const response = await clipsApi.getClip(id);
        setClip(response.clip);
        setTitle(response.clip.title);
        setDescription(response.clip.description || '');
        setCameraData(response.clip.cameraData);

        // Extract keyframes for cinematic mode
        if (response.clip.cameraData.type === 'cinematic') {
          setKeyframes((response.clip.cameraData as CameraKeyframes).keyframes);
        }

        // 2. Fetch replay binary (compiled .rlrf format)
        const replayResponse = await fetch(`/api/replays/${response.clip.replayId}/binary`);
        if (!replayResponse.ok) {
          throw new Error('Failed to fetch replay');
        }
        const arrayBuffer = await replayResponse.arrayBuffer();
        setReplayBinary(arrayBuffer);
      } catch (err) {
        console.error('[ClipEdit] Failed to fetch clip:', err);
        setError('Clip not found or unavailable');
      } finally {
        setLoading(false);
      }
    }

    fetchClipAndReplay();
  }, [id]);

  // Check authorization
  useEffect(() => {
    if (!authLoading && !user) {
      navigate('/login', { state: { from: `/clips/${id}/edit` } });
    } else if (!authLoading && clip && user && user.id !== clip.creator.id) {
      navigate(`/clips/${id}`);
      toast.error('You can only edit your own clips');
    }
  }, [authLoading, user, clip, id, navigate]);

  // Initialize game engine when we have clip and binary data
  useEffect(() => {
    if (!containerRef.current || !clip || !replayBinary) return;

    console.log('[ClipEdit] Initializing game engine with binary data');

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = new GameEngine(containerRef.current, {
      binaryData: replayBinary,
      onTimeUpdate: (time: number) => setCurrentTime(time),
      onPlayStateChange: () => {},
      onReady: () => {
        console.log('[ClipEdit] Game engine ready');
        setGameReady(true);
      },
      onError: (err: Error) => {
        console.error('[ClipEdit] GameEngine error:', err);
        setError(err.message || 'Failed to initialize viewer');
      },
    });

    gameRef.current = game;

    return () => {
      console.log('[ClipEdit] Disposing game engine');
      game.dispose();
      gameRef.current = null;
      setGameReady(false);
    };
  }, [clip, replayBinary]);

  // Load environment and seek to clip start when game is ready
  useEffect(() => {
    if (!gameReady || !clip) return;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    const initEditor = async () => {
      // Load the clip's environment or fall back to default
      if (game.environmentManager) {
        try {
          if (clip.environment?.id) {
            // Load the specific environment saved with this clip
            console.log('[ClipEdit] Loading clip environment:', clip.environment.name);
            await game.environmentManager.loadEnvironment(clip.environment.id);
          } else {
            // Fall back to default environment
            console.log('[ClipEdit] Loading default environment...');
            await game.environmentManager.loadDefaultEnvironment();
          }
          console.log('[ClipEdit] Environment loaded');
        } catch (err) {
          console.warn('[ClipEdit] Failed to load environment:', err);
        }
      }

      // Seek to clip start time
      game.seek(clip.startTime);

      // Show keyframes for cinematic mode
      if (clip.cameraData.type === 'cinematic') {
        game.showKeyframes?.();
        const kfs = (clip.cameraData as CameraKeyframes).keyframes;
        kfs.forEach((kf: CameraKeyframe) => {
          game.keyframeVisualizer?.addKeyframe(kf);
        });
      }
    };

    initEditor();
  }, [gameReady, clip]);

  // Track changes
  useEffect(() => {
    if (!clip) return;

    const titleChanged = title !== clip.title;
    const descChanged = description !== (clip.description || '');
    const cameraChanged = JSON.stringify(cameraData) !== JSON.stringify(clip.cameraData);
    const keyframesChanged =
      clip.cameraMode === 'cinematic' &&
      JSON.stringify(keyframes) !== JSON.stringify((clip.cameraData as CameraKeyframes).keyframes);

    setHasChanges(titleChanged || descChanged || cameraChanged || keyframesChanged);
  }, [clip, title, description, cameraData, keyframes]);

  // Handle re-record (capture mode)
  const handleStartReRecord = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game || !clip) return;

    // Seek to start time and start recording
    game.seek(clip.startTime);
    game.startClipRecording?.();
    game.play();
    setEditState('recording');
  }, [clip]);

  const handleStopReRecord = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const game = gameRef.current as any;
    if (!game) return;

    game.pause();
    const recordedData = game.stopClipRecording?.();
    if (recordedData) {
      setCameraData(recordedData);
    }
    setEditState('idle');
  }, []);

  // Handle preview
  const handlePreview = useCallback(() => {
    const game = gameRef.current;
    if (!game || !clip || !cameraData) return;

    // Start clip playback with current camera data
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (game as any).startClipPlayback?.(cameraData, clip.startTime);
    setEditState('preview');
  }, [clip, cameraData]);

  const handleStopPreview = useCallback(() => {
    const game = gameRef.current;
    if (!game) return;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (game as any).stopClipPlayback?.();
    setEditState('idle');
  }, []);

  // Keyframe editing (cinematic mode)
  const handleAddKeyframe = useCallback(() => {
    const game = gameRef.current;
    if (!game) return;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const keyframe = (game as any).addKeyframe?.(currentTime);
    if (keyframe) {
      setKeyframes((prev) => [...prev, keyframe].sort((a, b) => a.t - b.t));
    }
  }, [currentTime]);

  const handleRemoveKeyframe = useCallback((kfId: string) => {
    const game = gameRef.current;
    if (game) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (game as any).removeKeyframe?.(kfId);
    }
    setKeyframes((prev) => prev.filter((kf) => kf.id !== kfId));
    if (selectedKeyframeId === kfId) {
      setSelectedKeyframeId(null);
    }
  }, [selectedKeyframeId]);

  const handleSelectKeyframe = useCallback((kfId: string | null) => {
    setSelectedKeyframeId(kfId);
    const game = gameRef.current;
    if (game) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (game as any).selectKeyframe?.(kfId);
    }
  }, []);

  const handleUpdateKeyframeFromCamera = useCallback(() => {
    if (!selectedKeyframeId) return;

    const game = gameRef.current;
    if (!game) return;

    // Get current camera state
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const camera = (game as any).sceneManager?.camera;
    if (!camera) return;

    // Update keyframe with current camera position
    setKeyframes((prev) =>
      prev.map((kf) => {
        if (kf.id === selectedKeyframeId) {
          return {
            ...kf,
            px: camera.position.x,
            py: camera.position.y,
            pz: camera.position.z,
            qx: camera.quaternion.x,
            qy: camera.quaternion.y,
            qz: camera.quaternion.z,
            qw: camera.quaternion.w,
          };
        }
        return kf;
      })
    );

    // Update visualizer
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (game as any).keyframeVisualizer?.updateKeyframe(selectedKeyframeId, {
      px: camera.position.x,
      py: camera.position.y,
      pz: camera.position.z,
      qx: camera.quaternion.x,
      qy: camera.quaternion.y,
      qz: camera.quaternion.z,
      qw: camera.quaternion.w,
    });

    toast.success('Keyframe updated');
  }, [selectedKeyframeId]);

  // Save changes
  const handleSave = useCallback(async () => {
    if (!id || !clip) return;

    setSaving(true);
    try {
      let updatedCameraData = cameraData;

      // For cinematic mode, rebuild camera data from keyframes
      if (clip.cameraMode === 'cinematic') {
        updatedCameraData = {
          type: 'cinematic',
          interpolation: 'catmullrom',
          tension: 0.5,
          keyframes,
        } as CameraKeyframes;
      }

      await clipsApi.updateClip(id, {
        title: title.trim(),
        description: description.trim() || null,
        cameraData: updatedCameraData ?? undefined,
      });

      toast.success('Clip saved successfully');
      navigate(`/clips/${id}`);
    } catch (err) {
      console.error('[ClipEdit] Failed to save clip:', err);
      toast.error('Failed to save clip');
    } finally {
      setSaving(false);
    }
  }, [id, clip, title, description, cameraData, keyframes, navigate]);

  // Cancel edit
  const handleCancel = useCallback(() => {
    if (hasChanges) {
      if (!confirm('Discard unsaved changes?')) return;
    }
    navigate(`/clips/${id}`);
  }, [id, hasChanges, navigate]);

  if (authLoading || loading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
      </div>
    );
  }

  if (error || !clip) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] gap-4">
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

  const isCinematic = clip.cameraMode === 'cinematic';

  return (
    <div className="max-w-7xl mx-auto px-4 py-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-4">
            <button
              onClick={handleCancel}
              className="p-2 hover:bg-zinc-700 rounded-lg transition-colors"
            >
              <ArrowLeft className="w-5 h-5 text-zinc-400" />
            </button>
            <div>
              <h1 className="text-xl font-semibold text-white">Edit Clip</h1>
              <div className="flex items-center gap-2 text-sm text-zinc-400">
                {isCinematic ? (
                  <Film className="w-4 h-4" />
                ) : (
                  <Video className="w-4 h-4" />
                )}
                <span className="capitalize">{clip.cameraMode} mode</span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={handleCancel}
              className="flex items-center gap-2 px-4 py-2 text-zinc-400 hover:text-white hover:bg-zinc-700 rounded-lg transition-colors"
            >
              <X className="w-4 h-4" />
              <span>Cancel</span>
            </button>
            <button
              onClick={handleSave}
              disabled={saving || !hasChanges}
              className={cn(
                "flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors",
                hasChanges
                  ? "bg-blue-600 hover:bg-blue-500 text-white"
                  : "bg-zinc-700 text-zinc-500 cursor-not-allowed"
              )}
            >
              {saving ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Save className="w-4 h-4" />
              )}
              <span>{saving ? 'Saving...' : 'Save Changes'}</span>
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Viewer */}
          <div className="lg:col-span-2">
            <div
              ref={containerRef}
              className="aspect-video bg-zinc-900 rounded-xl overflow-hidden"
            />

            {/* Recording/Preview controls */}
            <div className="mt-4 flex items-center justify-center gap-4">
              {isCinematic ? (
                <>
                  {/* Cinematic mode controls */}
                  <button
                    onClick={handleAddKeyframe}
                    className="flex items-center gap-2 px-4 py-2 bg-violet-600 hover:bg-violet-500 text-white rounded-lg font-medium transition-colors"
                  >
                    <Plus className="w-4 h-4" />
                    <span>Add Keyframe</span>
                  </button>

                  {selectedKeyframeId && (
                    <button
                      onClick={handleUpdateKeyframeFromCamera}
                      className="flex items-center gap-2 px-4 py-2 bg-amber-600 hover:bg-amber-500 text-white rounded-lg font-medium transition-colors"
                    >
                      <RefreshCw className="w-4 h-4" />
                      <span>Update Position</span>
                    </button>
                  )}

                  {editState === 'preview' ? (
                    <button
                      onClick={handleStopPreview}
                      className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium transition-colors"
                    >
                      <Square className="w-4 h-4" />
                      <span>Stop Preview</span>
                    </button>
                  ) : (
                    <button
                      onClick={handlePreview}
                      disabled={keyframes.length < 2}
                      className={cn(
                        "flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors",
                        keyframes.length >= 2
                          ? "bg-green-600 hover:bg-green-500 text-white"
                          : "bg-zinc-700 text-zinc-500 cursor-not-allowed"
                      )}
                    >
                      <Play className="w-4 h-4" />
                      <span>Preview</span>
                    </button>
                  )}
                </>
              ) : (
                <>
                  {/* Capture mode controls */}
                  {editState === 'recording' ? (
                    <button
                      onClick={handleStopReRecord}
                      className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium transition-colors animate-pulse"
                    >
                      <Square className="w-4 h-4" />
                      <span>Stop Recording</span>
                    </button>
                  ) : editState === 'preview' ? (
                    <button
                      onClick={handleStopPreview}
                      className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium transition-colors"
                    >
                      <Square className="w-4 h-4" />
                      <span>Stop Preview</span>
                    </button>
                  ) : (
                    <>
                      <button
                        onClick={handleStartReRecord}
                        className="flex items-center gap-2 px-4 py-2 bg-violet-600 hover:bg-violet-500 text-white rounded-lg font-medium transition-colors"
                      >
                        <RefreshCw className="w-4 h-4" />
                        <span>Re-record Camera</span>
                      </button>

                      {cameraData && (
                        <button
                          onClick={handlePreview}
                          className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium transition-colors"
                        >
                          <Play className="w-4 h-4" />
                          <span>Preview</span>
                        </button>
                      )}
                    </>
                  )}
                </>
              )}
            </div>
          </div>

          {/* Edit Panel */}
          <div className="space-y-6">
            {/* Title & Description */}
            <div className="bg-zinc-800/50 rounded-xl p-4 space-y-4">
              <div>
                <label className="block text-sm font-medium text-zinc-400 mb-2">
                  Title
                </label>
                <input
                  type="text"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="Clip title"
                  className="w-full px-3 py-2 bg-zinc-700/50 border border-zinc-600 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  maxLength={100}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-zinc-400 mb-2">
                  Description
                </label>
                <textarea
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Optional description..."
                  rows={3}
                  className="w-full px-3 py-2 bg-zinc-700/50 border border-zinc-600 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                  maxLength={500}
                />
              </div>
            </div>

            {/* Keyframes List (cinematic mode only) */}
            {isCinematic && (
              <div className="bg-zinc-800/50 rounded-xl p-4">
                <h3 className="text-sm font-medium text-zinc-400 mb-3">
                  Keyframes ({keyframes.length})
                </h3>

                {keyframes.length === 0 ? (
                  <p className="text-sm text-zinc-500 text-center py-4">
                    No keyframes yet. Add keyframes using the button above.
                  </p>
                ) : (
                  <div className="space-y-2 max-h-[300px] overflow-y-auto">
                    {keyframes.map((kf, index) => (
                      <div
                        key={kf.id}
                        onClick={() => handleSelectKeyframe(kf.id)}
                        className={cn(
                          "flex items-center justify-between p-2 rounded-lg cursor-pointer transition-colors",
                          selectedKeyframeId === kf.id
                            ? "bg-blue-600/20 border border-blue-500/50"
                            : "bg-zinc-700/30 hover:bg-zinc-700/50"
                        )}
                      >
                        <div className="flex items-center gap-2">
                          <span className="text-xs font-mono text-zinc-400">
                            #{index + 1}
                          </span>
                          <span className="text-sm text-white">
                            {(kf.t / 1000).toFixed(2)}s
                          </span>
                        </div>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleRemoveKeyframe(kf.id);
                          }}
                          className="p-1 text-red-400 hover:text-red-300 hover:bg-red-500/20 rounded transition-colors"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    ))}
                  </div>
                )}

                {keyframes.length > 0 && keyframes.length < 2 && (
                  <p className="text-xs text-amber-400 mt-2">
                    At least 2 keyframes required for preview
                  </p>
                )}
              </div>
            )}

            {/* Clip Info */}
            <div className="bg-zinc-800/50 rounded-xl p-4 space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-zinc-400">Start Time</span>
                <span className="text-white">{clip.startTime.toFixed(2)}s</span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-400">End Time</span>
                <span className="text-white">{clip.endTime.toFixed(2)}s</span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-400">Duration</span>
                <span className="text-white">
                  {(clip.endTime - clip.startTime).toFixed(2)}s
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
  );
}
