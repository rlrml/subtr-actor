/**
 * useClipEditor - Hook for managing clip creation state
 * (024-clip-system)
 * (026-clip-editor-redesign) - Added draft persistence
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import * as clipsApi from '@/api/clips';
import type {
  CameraData,
  CameraRecording,
  CameraKeyframe,
  CameraKeyframes,
  ClipWithDetails,
} from '@/api/clips';
import { useClipDraft } from './useClipDraft';

export type ClipCameraMode = 'capture' | 'cinematic';
export type ClipEditorState = 'idle' | 'selecting' | 'recording' | 'preview' | 'saving';

export interface UseClipEditorResult {
  // State
  isOpen: boolean;
  state: ClipEditorState;
  cameraMode: ClipCameraMode;
  startTime: number;
  endTime: number;
  title: string;
  description: string;
  recordedData: CameraData | null;
  environmentId: string | null;
  thumbnail: string | null;

  // Cinematic mode keyframes
  keyframes: CameraKeyframe[];
  selectedKeyframeId: string | null;

  // Auto-spacing for cinematic mode
  autoSpaceEnabled: boolean;
  autoSpaceInterval: number; // in seconds

  // Draft persistence (026-clip-editor-redesign)
  hasUnsavedChanges: boolean;
  hasDraft: boolean;

  // Actions
  open: (currentTime?: number, environmentId?: string | null) => void;
  close: () => void;
  setSegment: (start: number, end: number) => void;
  setCameraMode: (mode: ClipCameraMode) => void;
  setTitle: (title: string) => void;
  setDescription: (desc: string) => void;
  setEnvironmentId: (id: string | null) => void;
  setThumbnail: (thumbnail: string | null) => void;
  startRecording: () => void;
  stopRecording: (data: CameraRecording) => void;
  setRecordedData: (data: CameraData) => void;
  preview: () => void;
  stopPreview: () => void;
  save: (thumbnail?: string) => Promise<ClipWithDetails | null>;
  reset: () => void;

  // Cinematic mode keyframe actions
  addKeyframe: (keyframe: CameraKeyframe) => boolean; // Returns false if keyframe outside segment
  removeKeyframe: (id: string) => void;
  updateKeyframe: (id: string, updates: Partial<CameraKeyframe>) => void;
  selectKeyframe: (id: string | null) => void;
  setKeyframes: (keyframes: CameraKeyframe[]) => void;
  canPreviewCinematic: () => boolean;
  getCinematicData: () => CameraKeyframes | null;
  isTimeInSegment: (absoluteTimeMs: number) => boolean; // Check if time is within segment

  // Auto-spacing actions
  setAutoSpaceEnabled: (enabled: boolean) => void;
  setAutoSpaceInterval: (interval: number) => void;
  getNextKeyframeTime: () => number; // Returns time in ms for next keyframe

  // Keyframe timing actions (026-clip-editor-redesign T029-T030)
  updateKeyframeTime: (id: string, newTimeMs: number) => void;
  distributeKeyframesEvenly: () => void;

  // Draft persistence actions (026-clip-editor-redesign)
  restoreDraft: () => boolean;
  discardDraft: () => void;
  saveDraftNow: () => boolean; // Force immediate save (no debounce)

  // Saving state
  isSaving: boolean;
  saveError: string | null;

  // Draft save error (026-clip-editor-redesign T046)
  draftSaveError: string | null;
}

export function useClipEditor(replayId: string, maxTime: number = 0): UseClipEditorResult {
  // UI state
  const [isOpen, setIsOpen] = useState(false);
  const [state, setState] = useState<ClipEditorState>('idle');

  // Clip configuration
  const [cameraMode, setCameraMode] = useState<ClipCameraMode>('capture');
  const [startTime, setStartTime] = useState(0);
  const [endTime, setEndTime] = useState(Math.min(30, maxTime));
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');

  // Recorded data
  const [recordedData, setRecordedData] = useState<CameraData | null>(null);

  // Environment ID (captured when clip editor is opened)
  const [environmentId, setEnvironmentId] = useState<string | null>(null);

  // Thumbnail (base64 data URL)
  const [thumbnail, setThumbnail] = useState<string | null>(null);

  // Cinematic mode keyframes
  const [keyframes, setKeyframesState] = useState<CameraKeyframe[]>([]);
  const [selectedKeyframeId, setSelectedKeyframeId] = useState<string | null>(null);

  // Auto-spacing for cinematic mode
  const [autoSpaceEnabled, setAutoSpaceEnabled] = useState(false); // Disabled by default
  const [autoSpaceInterval, setAutoSpaceInterval] = useState(2); // 2 seconds default

  // Save state
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  // Ref to track if preview was started
  const previewActiveRef = useRef(false);

  // ============================================
  // Draft Persistence (026-clip-editor-redesign)
  // ============================================
  const clipDraft = useClipDraft(replayId);

  // Track if there are unsaved changes (T011)
  const hasUnsavedChanges = (
    keyframes.length > 0 ||
    recordedData !== null ||
    title.trim() !== '' ||
    description.trim() !== ''
  );

  // Check if a draft exists for this replay
  const hasDraft = clipDraft.hasDraft();

  // Draft save error state (026-clip-editor-redesign T046)
  const [draftSaveError, setDraftSaveError] = useState<string | null>(null);

  // Auto-save draft when state changes (T010)
  useEffect(() => {
    // Only save if editor is open and there are changes to save
    if (!isOpen || !hasUnsavedChanges) return;

    // Debounce the save to avoid too many writes
    const timeoutId = setTimeout(() => {
      const success = clipDraft.saveDraft({
        replayId,
        segment: { start: startTime, end: endTime },
        mode: cameraMode,
        keyframes: cameraMode === 'cinematic' ? keyframes : undefined,
        captureData: cameraMode === 'capture' ? recordedData ?? undefined : undefined,
        selectedKeyframeId: selectedKeyframeId ?? undefined,
        autoSpaceEnabled,
        autoSpaceInterval,
      });

      if (success) {
        setDraftSaveError(null);
        console.log('[useClipEditor] Draft auto-saved');
      } else {
        // T046: Show user-friendly error message for quota exceeded
        setDraftSaveError('Unable to save draft locally. Storage may be full.');
        console.error('[useClipEditor] Failed to save draft - storage quota exceeded');
      }
    }, 500); // 500ms debounce

    return () => clearTimeout(timeoutId);
  }, [
    isOpen,
    hasUnsavedChanges,
    replayId,
    startTime,
    endTime,
    cameraMode,
    keyframes,
    recordedData,
    selectedKeyframeId,
    autoSpaceEnabled,
    autoSpaceInterval,
    clipDraft,
  ]);

  // Restore draft function (T018)
  const restoreDraft = useCallback((): boolean => {
    const draft = clipDraft.getDraft();
    if (!draft) return false;

    // Restore segment
    setStartTime(draft.segment.start);
    setEndTime(draft.segment.end);

    // Restore mode
    setCameraMode(draft.mode);

    // Restore mode-specific data
    if (draft.mode === 'cinematic' && draft.keyframes) {
      setKeyframesState(draft.keyframes);
    } else if (draft.mode === 'capture' && draft.captureData) {
      setRecordedData(draft.captureData);
    }

    // Restore UI state
    if (draft.selectedKeyframeId) {
      setSelectedKeyframeId(draft.selectedKeyframeId);
    }
    if (draft.autoSpaceEnabled !== undefined) {
      setAutoSpaceEnabled(draft.autoSpaceEnabled);
    }
    if (draft.autoSpaceInterval !== undefined) {
      setAutoSpaceInterval(draft.autoSpaceInterval);
    }

    console.log('[useClipEditor] Draft restored:', draft);
    return true;
  }, [clipDraft]);

  // Discard draft function
  const discardDraft = useCallback(() => {
    clipDraft.deleteDraft();
    console.log('[useClipEditor] Draft discarded');
  }, [clipDraft]);

  // Force immediate draft save (bypasses debounce)
  const saveDraftNow = useCallback((): boolean => {
    if (!hasUnsavedChanges) return true; // Nothing to save

    const success = clipDraft.saveDraft({
      replayId,
      segment: { start: startTime, end: endTime },
      mode: cameraMode,
      keyframes: cameraMode === 'cinematic' ? keyframes : undefined,
      captureData: cameraMode === 'capture' ? recordedData ?? undefined : undefined,
      selectedKeyframeId: selectedKeyframeId ?? undefined,
      autoSpaceEnabled,
      autoSpaceInterval,
    });

    if (success) {
      console.log('[useClipEditor] Draft saved immediately');
    } else {
      console.error('[useClipEditor] Failed to save draft immediately');
    }
    return success;
  }, [
    hasUnsavedChanges,
    clipDraft,
    replayId,
    startTime,
    endTime,
    cameraMode,
    keyframes,
    recordedData,
    selectedKeyframeId,
    autoSpaceEnabled,
    autoSpaceInterval,
  ]);

  // Open the clip editor with optional current time to set initial segment
  const open = useCallback((currentTime?: number, currentEnvironmentId?: string | null) => {
    setIsOpen(true);
    setState('selecting');
    setRecordedData(null);
    setKeyframesState([]);
    setSelectedKeyframeId(null);
    setSaveError(null);
    setEnvironmentId(currentEnvironmentId ?? null);

    // Set initial segment: 10 seconds starting at current time
    const clipDuration = 10;
    const start = currentTime ?? 0;
    // Ensure end doesn't exceed maxTime
    const end = Math.min(start + clipDuration, maxTime);
    // If we're near the end, adjust start to ensure at least 10 seconds (or maxTime)
    const adjustedStart = Math.max(0, end - clipDuration);

    setStartTime(adjustedStart);
    setEndTime(end);
  }, [maxTime]);

  // Close the clip editor
  const close = useCallback(() => {
    setIsOpen(false);
    setState('idle');
    previewActiveRef.current = false;
  }, []);

  // Set segment times
  const setSegment = useCallback((start: number, end: number) => {
    // Validate
    if (start < 0) start = 0;
    if (end > maxTime) end = maxTime;
    if (end <= start) end = start + 1;

    setStartTime(start);
    setEndTime(end);

    // Clear recorded data when segment changes (capture mode)
    setRecordedData(null);

    // Note: Keyframes are kept as-is. They store time relative to clip start,
    // so changing segment bounds will shift which game moments they point to.
    // The timeline zoom feature (026-clip-editor-redesign) provides precision
    // for adjusting keyframe positions within the current segment bounds.
  }, [maxTime]);

  // Start recording
  const startRecording = useCallback(() => {
    setState('recording');
    setRecordedData(null);
  }, []);

  // Stop recording and store data
  const stopRecording = useCallback((data: CameraRecording) => {
    setRecordedData(data);
    setState('selecting'); // Back to selecting, ready to preview or save
  }, []);

  // Start preview - works for both capture and cinematic modes
  const preview = useCallback(() => {
    if (cameraMode === 'capture') {
      if (!recordedData) {
        console.warn('[useClipEditor] No recorded data to preview (capture mode)');
        return;
      }
    } else {
      // Cinematic mode - check for keyframes instead
      if (keyframes.length < 2) {
        console.warn('[useClipEditor] Need at least 2 keyframes to preview (cinematic mode)');
        return;
      }
    }
    setState('preview');
    previewActiveRef.current = true;
  }, [cameraMode, recordedData, keyframes.length]);

  // Stop preview
  const stopPreview = useCallback(() => {
    setState('selecting');
    previewActiveRef.current = false;
  }, []);

  // ============================================
  // ============================================
  // Cinematic Mode Keyframe Methods
  // ============================================
  //
  // ARCHITECTURE NOTE (026-clip-editor-redesign):
  // Keyframes are stored with ABSOLUTE time (ms from replay start).
  // This ensures consistency with:
  //   - GameEngine.currentTime (absolute)
  //   - onSeek() calls (absolute)
  //   - KeyframeVisualizer (expects absolute)
  //   - ClipTimeline display (converts to relative for display only)
  //
  // Conversion to RELATIVE time happens ONLY in buildCinematicData() for API save.
  // ============================================

  // T048: Prevent duplicate timestamps (within 17ms tolerance - one frame at 60fps)
  const DUPLICATE_TOLERANCE_MS = 17;

  // Helper to check if a given time (absolute, in ms) is within segment bounds
  const isTimeInSegment = useCallback((absoluteTimeMs: number): boolean => {
    const clipStartMs = startTime * 1000;
    const clipEndMs = endTime * 1000;
    return absoluteTimeMs >= clipStartMs && absoluteTimeMs <= clipEndMs;
  }, [startTime, endTime]);

  // Add a keyframe - keyframe.t is ABSOLUTE time in ms
  const addKeyframe = useCallback((keyframe: CameraKeyframe): boolean => {
    const clipStartMs = startTime * 1000;
    const clipEndMs = endTime * 1000;

    // REJECT if keyframe is outside segment bounds
    if (keyframe.t < clipStartMs || keyframe.t > clipEndMs) {
      console.warn('[useClipEditor] Cannot add keyframe: time is outside segment bounds', {
        keyframeTime: keyframe.t,
        clipStartMs,
        clipEndMs,
      });
      return false;
    }

    // T048: Check for duplicate timestamps (using absolute time)
    const hasDuplicate = keyframes.some(
      (kf) => Math.abs(kf.t - keyframe.t) < DUPLICATE_TOLERANCE_MS
    );

    if (hasDuplicate) {
      console.warn('[useClipEditor] Keyframe at this timestamp already exists (within 17ms tolerance)');
      return false;
    }

    console.log('[useClipEditor] addKeyframe (ABSOLUTE time):', {
      t: keyframe.t,
      clipStartMs,
      clipEndMs,
    });

    setKeyframesState((prev) => {
      const updated = [...prev, keyframe]; // Keep keyframe as-is (absolute time)
      updated.sort((a, b) => a.t - b.t);
      return updated;
    });

    return true;
  }, [startTime, endTime, keyframes]);

  // Remove a keyframe
  const removeKeyframe = useCallback((id: string) => {
    setKeyframesState((prev) => prev.filter((kf) => kf.id !== id));
    // Clear selection if removed keyframe was selected
    setSelectedKeyframeId((prevId) => (prevId === id ? null : prevId));
  }, []);

  // Update a keyframe
  const updateKeyframe = useCallback((id: string, updates: Partial<CameraKeyframe>) => {
    setKeyframesState((prev) => {
      const updated = prev.map((kf) => (kf.id === id ? { ...kf, ...updates } : kf));
      // Re-sort if time changed
      if (updates.t !== undefined) {
        updated.sort((a, b) => a.t - b.t);
      }
      return updated;
    });
  }, []);

  // Select a keyframe
  const selectKeyframe = useCallback((id: string | null) => {
    setSelectedKeyframeId(id);
  }, []);

  // Set all keyframes (replaces existing)
  const setKeyframes = useCallback((newKeyframes: CameraKeyframe[]) => {
    setKeyframesState(newKeyframes);
    setSelectedKeyframeId(null);
  }, []);

  // Check if cinematic mode can be previewed (needs 2+ keyframes)
  const canPreviewCinematic = useCallback(() => {
    return keyframes.length >= 2;
  }, [keyframes.length]);

  // Get the time for the next keyframe (considers auto-spacing)
  // Returns ABSOLUTE time in ms
  const getNextKeyframeTime = useCallback((): number => {
    const clipStartMs = startTime * 1000;
    const clipEndMs = endTime * 1000;

    if (!autoSpaceEnabled || keyframes.length === 0) {
      return clipStartMs;
    }

    // Keyframes are already in ABSOLUTE time
    const sortedKeyframes = [...keyframes].sort((a, b) => a.t - b.t);
    const lastKeyframe = sortedKeyframes[sortedKeyframes.length - 1];
    const nextTimeAbsolute = lastKeyframe.t + autoSpaceInterval * 1000;

    // Clamp to segment end
    return Math.min(nextTimeAbsolute, clipEndMs);
  }, [autoSpaceEnabled, keyframes, autoSpaceInterval, startTime, endTime]);

  // Update a specific keyframe's time (T029)
  // newTimeMs is ABSOLUTE time in ms (from timeline drag which now uses absolute time)
  const updateKeyframeTime = useCallback((id: string, newTimeMs: number) => {
    // Clamp to clip bounds (absolute time)
    const clipStartMs = startTime * 1000;
    const clipEndMs = endTime * 1000;
    const clampedTime = Math.max(clipStartMs, Math.min(newTimeMs, clipEndMs));
    updateKeyframe(id, { t: clampedTime });
  }, [updateKeyframe, startTime, endTime]);

  // Distribute keyframes evenly across the clip duration (T030)
  // Keyframes are in ABSOLUTE time, so we distribute from clipStart to clipEnd
  const distributeKeyframesEvenly = useCallback(() => {
    if (keyframes.length < 2) return;

    const clipStartMs = startTime * 1000;
    const clipEndMs = endTime * 1000;
    const clipDurationMs = clipEndMs - clipStartMs;
    const interval = clipDurationMs / (keyframes.length - 1);

    // Sort keyframes by current time to maintain order
    const sortedKeyframes = [...keyframes].sort((a, b) => a.t - b.t);

    // Update each keyframe with evenly distributed ABSOLUTE time
    const updatedKeyframes = sortedKeyframes.map((kf, index) => ({
      ...kf,
      t: Math.round(clipStartMs + index * interval),
    }));

    setKeyframesState(updatedKeyframes);
    console.log('[useClipEditor] Keyframes distributed evenly (ABSOLUTE):', updatedKeyframes.map(k => k.t));
  }, [keyframes, startTime, endTime]);

  // Build cinematic camera data from keyframes for API save
  // ARCHITECTURE NOTE: Keyframes are stored in ABSOLUTE time internally.
  // We convert to RELATIVE time (relative to clip start) HERE for the backend API.
  const buildCinematicData = useCallback((): CameraKeyframes | null => {
    if (keyframes.length < 2) return null;

    const clipStartMs = Math.round(startTime * 1000);

    // Sort keyframes by time and convert to RELATIVE time for API
    // Round all time values to integers (backend schema requires int)
    const sortedKeyframes = [...keyframes].sort((a, b) => a.t - b.t);
    const relativeKeyframes = sortedKeyframes.map(kf => ({
      ...kf,
      t: Math.round(kf.t - clipStartMs), // Convert ABSOLUTE → RELATIVE, ensure integer
    }));

    console.log('[useClipEditor] buildCinematicData (converting to RELATIVE for API):', {
      clipStartMs,
      keyframes: relativeKeyframes.map(k => ({ t: k.t, px: k.px?.toFixed(2), py: k.py?.toFixed(2) })),
    });

    return {
      type: 'cinematic',
      interpolation: 'catmullrom',
      tension: 0.5,
      keyframes: relativeKeyframes, // RELATIVE time for backend API
    };
  }, [keyframes, startTime]);

  // Save clip to API
  // Save clip - thumbnailOverride is passed directly to avoid React state timing issues
  // NOTE: replayId is now passed to the hook, not to save()
  const save = useCallback(async (thumbnailOverride?: string): Promise<ClipWithDetails | null> => {
    // Validate based on camera mode
    let cameraData: CameraData | null = null;

    if (cameraMode === 'capture') {
      if (!recordedData) {
        setSaveError('No recorded data');
        return null;
      }
      cameraData = recordedData;
    } else {
      // Cinematic mode
      if (keyframes.length < 2) {
        setSaveError('At least 2 keyframes are required');
        return null;
      }
      cameraData = buildCinematicData();
    }

    if (!cameraData) {
      setSaveError('Invalid camera data');
      return null;
    }

    if (!title.trim()) {
      setSaveError('Title is required');
      return null;
    }

    setIsSaving(true);
    setSaveError(null);
    setState('saving');

    try {
      // Use thumbnailOverride if provided, otherwise fall back to state
      const thumbnailToUse = thumbnailOverride || thumbnail || undefined;

      const result = await clipsApi.createClip({
        replayId,
        title: title.trim(),
        description: description.trim() || undefined,
        startTime,
        endTime,
        cameraMode,
        cameraData,
        environmentId,
        thumbnail: thumbnailToUse,
      });

      // Success - delete draft and close the editor (T012)
      clipDraft.deleteDraft();
      console.log('[useClipEditor] Draft deleted after successful save');
      close();
      return result.clip;
    } catch (error) {
      const apiError = error as { message?: string };
      setSaveError(apiError.message || 'Failed to save clip');
      setState('selecting');
      return null;
    } finally {
      setIsSaving(false);
    }
  }, [replayId, recordedData, keyframes, buildCinematicData, title, description, startTime, endTime, cameraMode, environmentId, thumbnail, clipDraft, close]);

  // Reset all state
  const reset = useCallback(() => {
    setState('idle');
    setStartTime(0);
    setEndTime(Math.min(30, maxTime));
    setTitle('');
    setDescription('');
    setRecordedData(null);
    setEnvironmentId(null);
    setThumbnail(null);
    setKeyframesState([]);
    setSelectedKeyframeId(null);
    setSaveError(null);
    previewActiveRef.current = false;
  }, [maxTime]);

  return {
    // State
    isOpen,
    state,
    cameraMode,
    startTime,
    endTime,
    title,
    description,
    recordedData,
    environmentId,
    thumbnail,

    // Cinematic mode keyframes
    keyframes,
    selectedKeyframeId,

    // Auto-spacing
    autoSpaceEnabled,
    autoSpaceInterval,

    // Draft persistence (026-clip-editor-redesign)
    hasUnsavedChanges,
    hasDraft,

    // Actions
    open,
    close,
    setSegment,
    setCameraMode,
    setTitle,
    setDescription,
    setEnvironmentId,
    setThumbnail,
    startRecording,
    stopRecording,
    setRecordedData,
    preview,
    stopPreview,
    save,
    reset,

    // Cinematic mode keyframe actions
    addKeyframe,
    removeKeyframe,
    updateKeyframe,
    selectKeyframe,
    setKeyframes,
    canPreviewCinematic,
    getCinematicData: buildCinematicData,
    isTimeInSegment,

    // Auto-spacing actions
    setAutoSpaceEnabled,
    setAutoSpaceInterval,
    getNextKeyframeTime,

    // Keyframe timing actions (026-clip-editor-redesign T029-T030)
    updateKeyframeTime,
    distributeKeyframesEvenly,

    // Draft persistence actions (026-clip-editor-redesign)
    restoreDraft,
    discardDraft,
    saveDraftNow,

    // Saving state
    isSaving,
    saveError,

    // Draft save error (026-clip-editor-redesign T046)
    draftSaveError,
  };
}
