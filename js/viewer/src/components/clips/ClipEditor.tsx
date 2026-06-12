/**
 * ClipEditor
 *
 * Main component for creating clips from replays.
 * Integrates ClipTimeline, ClipModeSelector, and control buttons.
 *
 * Feature: 024-clip-system
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { X, Circle, Square, Play, Pause, Save, AlertCircle, Loader2, Plus, Trash2, SkipBack, RefreshCw, Timer, Eye, EyeOff, Film, Diamond } from 'lucide-react';
import { cn } from '@/lib/utils';
import { ClipTimeline } from './ClipTimeline';
import { ClipModeSelector } from './ClipModeSelector';
import type { UseClipEditorResult } from '@/hooks/useClipEditor';
import type { CameraKeyframe } from '@/api/clips';

// Helper to format time as MM:SS
function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

interface ClipEditorProps {
  editor: UseClipEditorResult;
  currentTime: number;
  maxTime: number;
  // NOTE (026-clip-editor-redesign): replayId removed - now passed to useClipEditor hook
  isPlaying: boolean;
  onSeek: (time: number) => void;
  onPlayPause: () => void;
  onStartRecording: () => void;
  onStopRecording: () => void;
  onStartPreview: () => void;
  onStopPreview: () => void;
  onClose: () => void;
  onClipSaved?: () => void;
  onCaptureFrame?: () => Promise<Blob>;
  // Cinematic mode callbacks
  onAddKeyframe?: () => void;
  onRemoveKeyframe?: (id: string) => void;
  onViewKeyframe?: (keyframe: CameraKeyframe) => void;
  onUpdateKeyframe?: () => void;
  // Update keyframe time (when dragging on timeline)
  onUpdateKeyframeTime?: (id: string, newTimeMs: number) => void;
  // 026-clip-editor-redesign: T044-T045 - Active keyframe and markers visibility
  onSetActiveKeyframe?: (id: string | null) => void;
  // 026-clip-editor-redesign: Toggle all markers (now includes ghost camera)
  onToggleAllMarkers?: (visible: boolean) => void;
  // Clear all keyframes (markers + ghost camera)
  onClearAllKeyframes?: () => void;
}

export function ClipEditor({
  editor,
  currentTime,
  maxTime,
  isPlaying,
  onSeek,
  onPlayPause,
  onStartRecording,
  onStopRecording,
  onStartPreview,
  onStopPreview,
  onClose,
  onClipSaved,
  onCaptureFrame,
  onAddKeyframe,
  onRemoveKeyframe,
  onViewKeyframe,
  onUpdateKeyframe,
  onUpdateKeyframeTime,
  onSetActiveKeyframe,
  onToggleAllMarkers,
  onClearAllKeyframes,
}: ClipEditorProps) {
  const [showSaveDialog, setShowSaveDialog] = useState(false);

  // Confirmation dialog state (026-clip-editor-redesign T013-T014)
  const [showConfirmClose, setShowConfirmClose] = useState(false);

  // Markers visibility state (026-clip-editor-redesign T045)
  const [markersVisible, setMarkersVisible] = useState(true);

  // Context menu state for keyframes
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; keyframeId: string } | null>(null);

  // Ref for keyframe list to attach native context menu listener
  const keyframeListRef = useRef<HTMLDivElement>(null);

  // Check if we're in a state where we can record
  const canRecord = editor.state === 'selecting' && editor.cameraMode === 'capture';
  const isRecording = editor.state === 'recording';
  const isPreviewing = editor.state === 'preview';
  const hasRecording = editor.recordedData !== null;

  // Cinematic mode state
  const isCinematicMode = editor.cameraMode === 'cinematic';
  const hasEnoughKeyframes = editor.keyframes.length >= 2;

  // Duration validation (T085)
  const clipDuration = editor.endTime - editor.startTime;
  const isClipTooShort = clipDuration < 1.0;
  const isClipLong = clipDuration > 120; // > 2 minutes

  const canSave = (isCinematicMode ? hasEnoughKeyframes : hasRecording) && !isClipTooShort;

  // Handle record button click
  const handleRecordClick = useCallback(() => {
    if (isRecording) {
      onStopRecording();
    } else {
      // Seek to start of segment before recording
      onSeek(editor.startTime);
      onStartRecording();
      editor.startRecording();
    }
  }, [isRecording, onStartRecording, onStopRecording, onSeek, editor]);

  // Handle preview button click
  const handlePreviewClick = useCallback(() => {
    if (isPreviewing) {
      onStopPreview();
      editor.stopPreview();
    } else {
      // Seek to start before preview
      onSeek(editor.startTime);
      onStartPreview();
      editor.preview();
    }
  }, [isPreviewing, onStartPreview, onStopPreview, onSeek, editor]);

  // Handle add keyframe (cinematic mode)
  // Note: onAddKeyframe handles both GameEngine visualization and editor state
  const handleAddKeyframe = useCallback(() => {
    onAddKeyframe?.();
  }, [onAddKeyframe]);

  // Handle keyframe click (seek to keyframe time and view from that position)
  // 026-clip-editor-redesign: T044 - Also set active keyframe to hide its marker
  const handleKeyframeClick = useCallback((keyframe: CameraKeyframe) => {
    onSeek(keyframe.t / 1000); // Convert from ms to seconds
    editor.selectKeyframe(keyframe.id);
    onSetActiveKeyframe?.(keyframe.id); // Hide the marker for clear view (T044)
    onViewKeyframe?.(keyframe); // Position camera at keyframe's view
  }, [onSeek, editor, onViewKeyframe, onSetActiveKeyframe]);

  // Handle update keyframe position
  const handleUpdateKeyframe = useCallback(() => {
    onUpdateKeyframe?.();
  }, [onUpdateKeyframe]);

  // Handle delete keyframe
  const handleDeleteKeyframe = useCallback((id: string) => {
    editor.removeKeyframe(id);
    onRemoveKeyframe?.(id); // Also remove from GameEngine visualizer
    setContextMenu(null); // Close context menu after delete
  }, [editor, onRemoveKeyframe]);

  // Handle clear all keyframes
  const handleClearAllKeyframes = useCallback(() => {
    // Clear all keyframes from visualizer (markers + ghost camera)
    onClearAllKeyframes?.();
    // Then clear the editor state
    editor.setKeyframes([]);
    // Reset markers visibility state (so toggle starts fresh when new keyframes are added)
    setMarkersVisible(true);
  }, [editor, onClearAllKeyframes]);

  // Native context menu handler for keyframes - attached via useEffect for better control
  // Re-run when isCinematicMode changes because the list element is conditionally rendered
  useEffect(() => {
    const listElement = keyframeListRef.current;
    if (!listElement) {
      return;
    }

    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();

      const target = e.target as HTMLElement;
      const keyframeElement = target.closest('[data-keyframe-id]');

      if (keyframeElement) {
        const keyframeId = keyframeElement.getAttribute('data-keyframe-id');
        if (keyframeId) {
          setContextMenu({ x: e.clientX, y: e.clientY, keyframeId });
        }
      }
    };

    // Use capture phase to intercept before browser
    listElement.addEventListener('contextmenu', handleContextMenu, true);

    return () => {
      listElement.removeEventListener('contextmenu', handleContextMenu, true);
    };
  }, [isCinematicMode]); // Re-attach when mode changes (element is conditionally rendered)

  // Close context menu when clicking elsewhere
  useEffect(() => {
    if (contextMenu) {
      const handleClickOutside = (e: MouseEvent) => {
        const target = e.target as HTMLElement;
        if (target.closest('[data-context-menu]')) return;
        setContextMenu(null);
      };

      const handleContextMenuOutside = (e: MouseEvent) => {
        const target = e.target as HTMLElement;
        // Only close if not clicking on a keyframe (let the other handler open new menu)
        if (!target.closest('[data-keyframe-id]') && !target.closest('[data-context-menu]')) {
          e.preventDefault();
          setContextMenu(null);
        }
      };

      document.addEventListener('click', handleClickOutside, true);
      document.addEventListener('contextmenu', handleContextMenuOutside, true);

      return () => {
        document.removeEventListener('click', handleClickOutside, true);
        document.removeEventListener('contextmenu', handleContextMenuOutside, true);
      };
    }
  }, [contextMenu]);

  // Handle toggle markers visibility (026-clip-editor-redesign T045)
  const handleToggleMarkers = useCallback(() => {
    const newVisible = !markersVisible;
    setMarkersVisible(newVisible);
    onToggleAllMarkers?.(newVisible);
  }, [markersVisible, onToggleAllMarkers]);

  // Show/hide keyframes and ghost camera based on camera mode
  // 026-clip-editor-redesign: Ghost camera is now always visible with markers
  // Also re-evaluate when keyframe count changes (to show ghost camera when reaching 2 keyframes)
  useEffect(() => {
    if (isCinematicMode) {
      // Show both markers and ghost camera when entering cinematic mode
      onToggleAllMarkers?.(markersVisible);
    } else {
      // Hide everything when exiting cinematic mode
      onToggleAllMarkers?.(false);
    }
  }, [isCinematicMode, markersVisible, onToggleAllMarkers, editor.keyframes.length]);

  // Handle save
  const handleSaveClick = useCallback(() => {
    setShowSaveDialog(true);
  }, []);

  const handleSaveConfirm = useCallback(async () => {
    // Capture thumbnail before saving if callback is provided
    let thumbnailDataUrl: string | undefined;

    if (onCaptureFrame) {
      try {
        const blob = await onCaptureFrame();
        // Convert blob to base64 data URL
        const reader = new FileReader();
        thumbnailDataUrl = await new Promise<string>((resolve, reject) => {
          reader.onloadend = () => resolve(reader.result as string);
          reader.onerror = reject;
          reader.readAsDataURL(blob);
        });
      } catch (err) {
        console.warn('[ClipEditor] Failed to capture thumbnail:', err);
        // Continue saving without thumbnail
      }
    }

    // Pass thumbnail directly to save() to avoid React state timing issues
    // NOTE (026-clip-editor-redesign): replayId is now passed to the hook, not save()
    const clip = await editor.save(thumbnailDataUrl);
    if (clip) {
      setShowSaveDialog(false);
      onClipSaved?.();
    }
  }, [editor, onClipSaved, onCaptureFrame]);

  const handleCancelSave = useCallback(() => {
    setShowSaveDialog(false);
  }, []);

  // Handle close with confirmation if unsaved changes (T014)
  const handleCloseRequest = useCallback(() => {
    if (editor.hasUnsavedChanges) {
      // Force save draft immediately before showing dialog (bypass debounce)
      editor.saveDraftNow();
      setShowConfirmClose(true);
    } else {
      // No unsaved changes, close directly
      if (isRecording) {
        onStopRecording();
      }
      if (isPreviewing) {
        onStopPreview();
        editor.stopPreview();
      }
      editor.close();
      onClose();
    }
  }, [editor, isRecording, isPreviewing, onStopRecording, onStopPreview, onClose]);

  // Force close (discard changes)
  const handleForceClose = useCallback(() => {
    if (isRecording) {
      onStopRecording();
    }
    if (isPreviewing) {
      onStopPreview();
      editor.stopPreview();
    }
    editor.discardDraft();
    editor.close();
    onClose();
    setShowConfirmClose(false);
  }, [isRecording, isPreviewing, onStopRecording, onStopPreview, editor, onClose]);

  // Cancel close confirmation
  const handleCancelClose = useCallback(() => {
    setShowConfirmClose(false);
  }, []);

  // Legacy handleClose for backwards compatibility
  const handleClose = handleCloseRequest;

  // Beforeunload listener to warn user about unsaved changes (T015)
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (editor.hasUnsavedChanges) {
        e.preventDefault();
        // Modern browsers require returnValue to be set
        e.returnValue = 'You have unsaved changes. Are you sure you want to leave?';
        return e.returnValue;
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, [editor.hasUnsavedChanges]);

  // Update segment in editor
  const handleSegmentChange = useCallback((start: number, end: number) => {
    editor.setSegment(start, end);
  }, [editor]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (showSaveDialog) {
          setShowSaveDialog(false);
        } else {
          handleClose();
        }
      }
      // R to record (when not in input field and clip is long enough)
      if (e.key === 'r' && !showSaveDialog && canRecord && !isClipTooShort && !(e.target as HTMLElement)?.closest('input, textarea')) {
        handleRecordClick();
      }
      // Enter to stop recording (Space is used for freecam elevation)
      if (e.key === 'Enter' && isRecording && !(e.target as HTMLElement)?.closest('input, textarea')) {
        e.preventDefault();
        handleRecordClick();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showSaveDialog, canRecord, isRecording, isClipTooShort, handleRecordClick, handleClose]);

  return (
    <>
      {/* Bottom Panel - 026-clip-editor-redesign: Off-canvas panel below 3D canvas */}
      <div className="fixed left-0 right-0 bottom-0 h-[55vh] md:h-[280px] z-50 bg-zinc-900/98 backdrop-blur-md border-t border-zinc-700 flex flex-col overflow-hidden">
        {/* Main Content - Vertical on mobile, Horizontal on desktop */}
        <div className="flex-1 flex flex-col md:flex-row min-h-0 overflow-y-auto md:overflow-y-hidden">
          {/* Left Section: Mode & Keyframes */}
          <div className="w-full md:w-[280px] lg:w-[300px] flex-shrink-0 border-b md:border-b-0 md:border-r border-zinc-700 p-3 flex flex-col gap-2 overflow-hidden min-h-0">
            {/* Header with Close */}
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <Film className="w-4 h-4 text-orange-500" />
                Create Clip
              </h3>
              <button
                onClick={handleClose}
                className="p-1 hover:bg-zinc-700 rounded transition-colors"
                title="Close (Esc)"
              >
                <X className="w-4 h-4 text-zinc-400" />
              </button>
            </div>

            {/* Camera Mode Selector */}
            <div className="flex items-center justify-between bg-zinc-800/50 rounded-lg px-3 py-2">
              <span className="text-xs text-zinc-400">Mode</span>
              <ClipModeSelector
                mode={editor.cameraMode}
                onChange={editor.setCameraMode}
                disabled={isRecording || isPreviewing}
              />
            </div>

            {/* Cinematic Mode - Keyframe List (scrollable) */}
            {isCinematicMode && (
              <div className="flex-1 flex flex-col gap-2 min-h-0">
                {/* Keyframe Controls - Full row for buttons */}
                <div className="flex items-center justify-between bg-zinc-800/30 rounded-lg px-2 py-1.5">
                  <span className="text-xs text-zinc-400">
                    Keyframes ({editor.keyframes.length})
                  </span>
                  <div className="flex items-center gap-1.5">
                    {editor.keyframes.length > 0 && (
                      <button
                        onClick={handleToggleMarkers}
                        disabled={isPreviewing || editor.isSaving}
                        title={markersVisible ? "Hide preview" : "Show preview"}
                        className={cn(
                          "p-1.5 rounded transition-all",
                          markersVisible
                            ? "bg-orange-600 hover:bg-orange-500 text-white"
                            : "bg-zinc-700 hover:bg-zinc-600 text-zinc-400",
                          (isPreviewing || editor.isSaving) && "opacity-50 cursor-not-allowed"
                        )}
                      >
                        {markersVisible ? <Eye className="w-3.5 h-3.5" /> : <EyeOff className="w-3.5 h-3.5" />}
                      </button>
                    )}
                    {editor.keyframes.length >= 2 && (
                      <button
                        onClick={() => editor.distributeKeyframesEvenly()}
                        disabled={isPreviewing || editor.isSaving}
                        title="Distribute evenly"
                        className={cn(
                          "p-1.5 rounded bg-zinc-700 hover:bg-zinc-600 text-zinc-300 transition-all",
                          (isPreviewing || editor.isSaving) && "opacity-50 cursor-not-allowed"
                        )}
                      >
                        <Timer className="w-3.5 h-3.5" />
                      </button>
                    )}
                    {editor.keyframes.length > 0 && (
                      <button
                        onClick={handleClearAllKeyframes}
                        disabled={isPreviewing || editor.isSaving}
                        title="Clear all keyframes"
                        className={cn(
                          "p-1.5 rounded bg-zinc-700 hover:bg-red-600 text-zinc-300 hover:text-white transition-all",
                          (isPreviewing || editor.isSaving) && "opacity-50 cursor-not-allowed"
                        )}
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    )}
                    <button
                      onClick={handleAddKeyframe}
                      disabled={isPreviewing || editor.isSaving}
                      className={cn(
                        "flex items-center gap-1 px-2 py-1.5 rounded text-xs font-medium transition-all",
                        "bg-purple-600 hover:bg-purple-500 text-white",
                        (isPreviewing || editor.isSaving) && "opacity-50 cursor-not-allowed"
                      )}
                    >
                      <Plus className="w-3.5 h-3.5" />
                      <span>Add</span>
                    </button>
                  </div>
                </div>

                {/* Keyframe List - Scrollable */}
                <div ref={keyframeListRef} className="flex-1 overflow-y-auto space-y-1 min-h-0 scrollbar-thin">
                  {editor.keyframes.length > 0 ? (
                    editor.keyframes.map((kf, index) => (
                      <div
                        key={kf.id}
                        data-keyframe-id={kf.id}
                        onClick={() => handleKeyframeClick(kf)}
                        className={cn(
                          "flex items-center justify-between px-2 py-1 rounded cursor-pointer transition-colors",
                          editor.selectedKeyframeId === kf.id
                            ? "bg-purple-500/30 border border-purple-500/50"
                            : "bg-zinc-700/50 hover:bg-zinc-600/50"
                        )}
                      >
                        <div className="flex items-center gap-1.5">
                          <Diamond className="w-3 h-3 text-orange-500 fill-orange-500" />
                          <span className="text-xs text-white">#{index + 1}</span>
                          <span className="text-xs text-zinc-400">{formatTime(kf.t / 1000)}</span>
                        </div>
                        <div className="flex items-center gap-0.5">
                          {editor.selectedKeyframeId === kf.id && onUpdateKeyframe && (
                            <button
                              onClick={(e) => { e.stopPropagation(); handleUpdateKeyframe(); }}
                              disabled={isPreviewing || editor.isSaving}
                              className="p-0.5 hover:bg-purple-500/20 rounded transition-colors"
                              title="Update"
                            >
                              <RefreshCw className="w-3 h-3 text-purple-400" />
                            </button>
                          )}
                          <button
                            onClick={(e) => { e.stopPropagation(); handleDeleteKeyframe(kf.id); }}
                            disabled={isPreviewing || editor.isSaving}
                            className="p-0.5 hover:bg-red-500/20 rounded transition-colors"
                            title="Delete"
                          >
                            <Trash2 className="w-3 h-3 text-red-400" />
                          </button>
                        </div>
                      </div>
                    ))
                  ) : (
                    <div className="py-3 text-center text-zinc-500 text-xs bg-zinc-800/30 rounded">
                      Position camera & Add
                    </div>
                  )}
                </div>

                {/* Hints */}
                {editor.keyframes.length > 0 && editor.keyframes.length < 2 && (
                  <div className="text-amber-400 text-xs text-center">
                    +{2 - editor.keyframes.length} more to preview
                  </div>
                )}
              </div>
            )}

            {/* Capture Mode - Recording Status */}
            {!isCinematicMode && (
              <div className="flex-1 flex flex-col justify-center">
                {isRecording && (
                  <div className="flex items-center gap-2 p-2 bg-red-500/20 rounded border border-red-500/50">
                    <Circle className="w-3 h-3 fill-red-500 text-red-500 animate-pulse" />
                    <span className="text-red-400 text-xs font-medium">Recording...</span>
                  </div>
                )}
                {hasRecording && !isRecording && !isPreviewing && (
                  <div className="flex items-center gap-2 p-2 bg-green-500/20 rounded border border-green-500/50">
                    <Circle className="w-3 h-3 fill-green-500 text-green-500" />
                    <span className="text-green-400 text-xs">Ready</span>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Center Section: Timeline (takes remaining space) */}
          <div className="flex-1 flex flex-col p-3 min-w-0">
            {/* Status bar */}
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                {isPreviewing && (
                  <div className="flex items-center gap-1.5 px-2 py-0.5 bg-blue-500/20 rounded border border-blue-500/50">
                    <Play className="w-3 h-3 fill-blue-500 text-blue-500" />
                    <span className="text-blue-400 text-xs">Preview</span>
                  </div>
                )}
                {isClipTooShort && (
                  <div className="flex items-center gap-1 px-2 py-0.5 bg-red-500/20 rounded border border-red-500/50">
                    <AlertCircle className="w-3 h-3 text-red-400" />
                    <span className="text-red-400 text-xs">Min 1s</span>
                  </div>
                )}
                {isClipLong && !isRecording && (
                  <div className="flex items-center gap-1 px-2 py-0.5 bg-amber-500/20 rounded border border-amber-500/50">
                    <AlertCircle className="w-3 h-3 text-amber-400" />
                    <span className="text-amber-400 text-xs">{Math.floor(clipDuration / 60)}:{Math.floor(clipDuration % 60).toString().padStart(2, '0')}</span>
                  </div>
                )}
                {editor.draftSaveError && (
                  <div className="flex items-center gap-1 px-2 py-0.5 bg-orange-500/20 rounded border border-orange-500/50">
                    <AlertCircle className="w-3 h-3 text-orange-400" />
                    <span className="text-orange-400 text-xs truncate max-w-[150px]">{editor.draftSaveError}</span>
                  </div>
                )}
              </div>
              {isCinematicMode && hasEnoughKeyframes && !isPreviewing && (
                <span className="text-green-400 text-xs">Ready!</span>
              )}
            </div>

            {/* Timeline - Full width for precision */}
            <div className="flex-1 bg-zinc-800/50 rounded-lg p-3">
              <ClipTimeline
                currentTime={currentTime}
                maxTime={maxTime}
                startTime={editor.startTime}
                endTime={editor.endTime}
                onSegmentChange={handleSegmentChange}
                onSeek={onSeek}
                disabled={isRecording || isPreviewing}
                keyframes={isCinematicMode ? editor.keyframes : undefined}
                selectedKeyframeId={isCinematicMode ? editor.selectedKeyframeId : undefined}
                onKeyframeClick={isCinematicMode ? (kf) => {
                  onSeek(kf.t / 1000);
                  editor.selectKeyframe(kf.id);
                  onSetActiveKeyframe?.(kf.id);
                  onViewKeyframe?.(kf);
                } : undefined}
                onKeyframeTimeChange={isCinematicMode ? onUpdateKeyframeTime : undefined}
                showKeyframeTrack={isCinematicMode}
              />
            </div>
          </div>

          {/* Right Section: Playback & Actions */}
          <div className="w-full md:w-[180px] lg:w-[200px] flex-shrink-0 border-t md:border-t-0 md:border-l border-zinc-700 p-3 flex flex-col gap-3">
            {/* Playback Controls */}
            <div className="flex items-center justify-center gap-2">
              <button
                onClick={() => onSeek(editor.startTime)}
                disabled={isRecording || editor.isSaving}
                className={cn(
                  "p-1.5 rounded bg-zinc-700 hover:bg-zinc-600 text-white transition-all",
                  (isRecording || editor.isSaving) && "opacity-50 cursor-not-allowed"
                )}
                title="Go to start"
              >
                <SkipBack className="w-4 h-4" />
              </button>
              <button
                onClick={onPlayPause}
                disabled={isRecording || editor.isSaving}
                className={cn(
                  "p-1.5 rounded bg-zinc-700 hover:bg-zinc-600 text-white transition-all",
                  (isRecording || editor.isSaving) && "opacity-50 cursor-not-allowed"
                )}
                title={isPlaying ? "Pause" : "Play"}
              >
                {isPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4 fill-white" />}
              </button>
              <span className="text-xs text-zinc-400 font-mono">{formatTime(currentTime)}</span>
            </div>

            {/* Action Buttons - Row on mobile, stacked on desktop */}
            <div className="flex-1 flex flex-row md:flex-col gap-2">
              {editor.cameraMode === 'capture' && (
                <button
                  onClick={handleRecordClick}
                  disabled={isPreviewing || editor.isSaving || (isClipTooShort && !isRecording)}
                  className={cn(
                    "flex-1 md:flex-none flex items-center justify-center gap-1.5 md:gap-2 py-2 px-3 md:px-0 rounded font-medium text-sm transition-all min-h-[44px] md:min-h-0",
                    isRecording
                      ? "bg-red-600 hover:bg-red-500 text-white"
                      : "bg-zinc-700 hover:bg-zinc-600 text-white",
                    (isPreviewing || editor.isSaving || (isClipTooShort && !isRecording)) && "opacity-50 cursor-not-allowed"
                  )}
                >
                  {isRecording ? (
                    <><Square className="w-4 h-4 fill-white" /><span className="hidden sm:inline">Stop</span></>
                  ) : (
                    <><Circle className="w-4 h-4 fill-red-500 text-red-500" /><span className="hidden sm:inline">Record</span></>
                  )}
                </button>
              )}

              <button
                onClick={handlePreviewClick}
                disabled={!canSave || isRecording || editor.isSaving}
                className={cn(
                  "flex-1 md:flex-none flex items-center justify-center gap-1.5 md:gap-2 py-2 px-3 md:px-0 rounded font-medium text-sm transition-all min-h-[44px] md:min-h-0",
                  isPreviewing
                    ? "bg-blue-600 hover:bg-blue-500 text-white"
                    : "bg-zinc-700 hover:bg-zinc-600 text-white",
                  (!canSave || isRecording || editor.isSaving) && "opacity-50 cursor-not-allowed"
                )}
              >
                {isPreviewing ? (
                  <><Square className="w-4 h-4 fill-white" /><span className="hidden sm:inline">Stop</span></>
                ) : (
                  <><Play className="w-4 h-4 fill-white" /><span className="hidden sm:inline">Preview</span></>
                )}
              </button>

              <button
                onClick={handleSaveClick}
                disabled={!canSave || isRecording || isPreviewing || editor.isSaving}
                className={cn(
                  "flex-1 md:flex-none flex items-center justify-center gap-1.5 md:gap-2 py-2 px-3 md:px-0 rounded font-medium text-sm transition-all min-h-[44px] md:min-h-0",
                  "bg-green-600 hover:bg-green-500 text-white",
                  (!canSave || isRecording || isPreviewing || editor.isSaving) && "opacity-50 cursor-not-allowed"
                )}
              >
                <Save className="w-4 h-4" />
                <span className="hidden sm:inline">Save</span>
              </button>
            </div>

            {/* Keyboard hints - hidden on mobile */}
            <div className="hidden md:flex items-center justify-center gap-2 text-xs text-zinc-500">
              <span><kbd className="px-1 py-0.5 bg-zinc-800 rounded text-[10px]">R</kbd></span>
              <span><kbd className="px-1 py-0.5 bg-zinc-800 rounded text-[10px]">Esc</kbd></span>
            </div>
          </div>
        </div>
      </div>

      {/* Save Dialog */}
      {showSaveDialog && (
        <SaveClipDialog
          title={editor.title}
          description={editor.description}
          onTitleChange={editor.setTitle}
          onDescriptionChange={editor.setDescription}
          onSave={handleSaveConfirm}
          onCancel={handleCancelSave}
          isSaving={editor.isSaving}
          error={editor.saveError}
        />
      )}

      {/* Confirm Close Dialog */}
      {showConfirmClose && (
        <ConfirmCloseDialog
          keyframeCount={editor.keyframes.length}
          onDiscard={handleForceClose}
          onCancel={handleCancelClose}
        />
      )}

      {/* Context Menu for Keyframes */}
      {contextMenu && (
        <div
          data-context-menu
          className="fixed z-[100] bg-zinc-800 border border-zinc-600 rounded-lg shadow-xl py-1 min-w-[120px]"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={(e) => e.stopPropagation()}
          onContextMenu={(e) => e.preventDefault()}
        >
          <button
            onClick={() => handleDeleteKeyframe(contextMenu.keyframeId)}
            className="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-red-400 hover:bg-zinc-700 transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            <span>Delete</span>
          </button>
        </div>
      )}
    </>
  );
}

// Save Dialog Component
interface SaveClipDialogProps {
  title: string;
  description: string;
  onTitleChange: (title: string) => void;
  onDescriptionChange: (desc: string) => void;
  onSave: () => void;
  onCancel: () => void;
  isSaving: boolean;
  error: string | null;
}

function SaveClipDialog({
  title,
  description,
  onTitleChange,
  onDescriptionChange,
  onSave,
  onCancel,
  isSaving,
  error,
}: SaveClipDialogProps) {
  // Handle Enter to save
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && e.ctrlKey && title.trim()) {
      onSave();
    }
  }, [title, onSave]);

  return (
    <div className="fixed inset-0 z-60 flex items-center justify-center p-4 pointer-events-auto">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60"
        onClick={onCancel}
      />

      {/* Dialog */}
      <div className="relative bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl w-full max-w-md">
        <div className="p-6 space-y-4">
          <h4 className="text-lg font-semibold text-white">Save Clip</h4>

          {/* Error */}
          {error && (
            <div className="flex items-center gap-2 p-3 bg-red-500/20 border border-red-500/50 rounded-lg">
              <AlertCircle className="w-4 h-4 text-red-400" />
              <span className="text-red-400 text-sm">{error}</span>
            </div>
          )}

          {/* Title Input */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-zinc-300">
              Title <span className="text-red-400">*</span>
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => onTitleChange(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Enter clip title..."
              className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
              disabled={isSaving}
              maxLength={255}
            />
          </div>

          {/* Description Input */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-zinc-300">
              Description <span className="text-zinc-500">(optional)</span>
            </label>
            <textarea
              value={description}
              onChange={(e) => onDescriptionChange(e.target.value)}
              placeholder="Add a description..."
              rows={3}
              className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
              disabled={isSaving}
            />
          </div>

          {/* Buttons */}
          <div className="flex items-center justify-end gap-3 pt-2">
            <button
              onClick={onCancel}
              disabled={isSaving}
              className="px-4 py-2 text-zinc-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={onSave}
              disabled={isSaving || !title.trim()}
              className={cn(
                "flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium transition-colors",
                (isSaving || !title.trim()) && "opacity-50 cursor-not-allowed"
              )}
            >
              {isSaving ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span>Saving...</span>
                </>
              ) : (
                <>
                  <Save className="w-4 h-4" />
                  <span>Save Clip</span>
                </>
              )}
            </button>
          </div>

          {/* Hint */}
          <p className="text-xs text-zinc-500 text-center">
            Press <kbd className="px-1 py-0.5 bg-zinc-800 rounded">Ctrl+Enter</kbd> to save
          </p>
        </div>
      </div>
    </div>
  );
}

// Confirm Close Dialog Component (026-clip-editor-redesign T013)
interface ConfirmCloseDialogProps {
  keyframeCount: number;
  onDiscard: () => void;
  onCancel: () => void;
}

function ConfirmCloseDialog({
  keyframeCount,
  onDiscard,
  onCancel,
}: ConfirmCloseDialogProps) {
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
          <div className="w-10 h-10 rounded-full bg-amber-500/20 flex items-center justify-center">
            <AlertCircle className="w-5 h-5 text-amber-400" />
          </div>
          <h4 className="text-lg font-semibold text-white">Unsaved Changes</h4>
        </div>

        <p className="text-zinc-300">
          You have unsaved changes{keyframeCount > 0 && ` (${keyframeCount} keyframe${keyframeCount !== 1 ? 's' : ''})`}.
          Your work will be saved as a draft and can be restored when you return.
        </p>

        <p className="text-sm text-zinc-400">
          Do you want to discard your changes or continue editing?
        </p>

        <div className="flex items-center justify-end gap-3 pt-2">
          <button
            onClick={onCancel}
            className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg font-medium transition-colors"
          >
            Continue Editing
          </button>
          <button
            onClick={onDiscard}
            className="px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium transition-colors"
          >
            Discard & Close
          </button>
        </div>
      </div>
    </div>
  );
}
