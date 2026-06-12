/**
 * ClipTimeline
 *
 * Timeline component with draggable handles for segment selection.
 * Shows a visual representation of the selected clip range.
 * Supports dragging both the playhead and the segment handles.
 *
 * Feature: 024-clip-system
 */

import React, { useRef, useState, useCallback, useEffect } from 'react';
import { ArrowLeftToLine, ArrowRightToLine } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { CameraKeyframe } from '@/api/clips';

interface ClipTimelineProps {
  currentTime: number;
  maxTime: number;
  startTime: number;
  endTime: number;
  onSegmentChange: (start: number, end: number) => void;
  onSeek?: (time: number) => void;
  disabled?: boolean;
  showCurrentTime?: boolean;
  // Cinematic mode keyframes (026-clip-editor-redesign T024)
  keyframes?: CameraKeyframe[];
  selectedKeyframeId?: string | null;
  onKeyframeClick?: (keyframe: CameraKeyframe) => void;
  onKeyframeTimeChange?: (id: string, newTime: number) => void;
  // Two-track mode: separate segment selection track and keyframe track (026-clip-editor-redesign)
  showKeyframeTrack?: boolean;
}

export function ClipTimeline({
  currentTime,
  maxTime,
  startTime,
  endTime,
  onSegmentChange,
  onSeek,
  disabled = false,
  showCurrentTime = true,
  // Cinematic mode keyframes (026-clip-editor-redesign T024)
  keyframes = [],
  selectedKeyframeId = null,
  onKeyframeClick,
  onKeyframeTimeChange,
  // Two-track mode: separate tracks for segment selection and keyframes
  showKeyframeTrack = false,
}: ClipTimelineProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const keyframeTrackRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState<'start' | 'end' | 'region' | 'playhead' | 'keyframe' | 'keyframePlayhead' | null>(null);
  const [dragStartX, setDragStartX] = useState(0);
  const [dragStartValues, setDragStartValues] = useState({ start: 0, end: 0, current: 0 });
  const [justFinishedDrag, setJustFinishedDrag] = useState(false);
  // Keyframe drag state (T026)
  const [draggingKeyframeId, setDraggingKeyframeId] = useState<string | null>(null);
  const [dragKeyframeStartTime, setDragKeyframeStartTime] = useState(0);
  // Editable time state (T035)
  const [editingTime, setEditingTime] = useState<'start' | 'end' | null>(null);
  const [editInputValue, setEditInputValue] = useState('');

  // Track last emitted segment values to adjust playhead on region drag end
  const lastSegmentRef = useRef({ start: startTime, end: endTime });

  // Keep ref in sync with props when not dragging
  useEffect(() => {
    if (!dragging) {
      lastSegmentRef.current = { start: startTime, end: endTime };
    }
  }, [startTime, endTime, dragging]);

  const formatTime = (time: number) => {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60).toString().padStart(2, '0');
    return `${minutes}:${seconds}`;
  };

  // Format timecode with decimal precision (T038)
  const formatTimecode = (time: number): string => {
    const minutes = Math.floor(time / 60);
    const seconds = (time % 60).toFixed(1);
    return `${minutes}:${parseFloat(seconds) < 10 ? '0' : ''}${seconds}`;
  };

  // Parse timecode string to seconds (T037)
  // Accepts formats: "1:30", "90", "1:30.5", "90.5"
  const parseTimecode = (input: string): number | null => {
    const trimmed = input.trim();
    if (!trimmed) return null;

    // Format: "MM:SS" or "MM:SS.D"
    const colonMatch = trimmed.match(/^(\d+):(\d+(?:\.\d+)?)$/);
    if (colonMatch) {
      const minutes = parseInt(colonMatch[1], 10);
      const seconds = parseFloat(colonMatch[2]);
      if (!isNaN(minutes) && !isNaN(seconds)) {
        return minutes * 60 + seconds;
      }
    }

    // Format: just seconds "90" or "90.5"
    const secondsOnly = parseFloat(trimmed);
    if (!isNaN(secondsOnly)) {
      return secondsOnly;
    }

    return null;
  };

  // Get time from X position on segment track (full replay duration)
  const getTimeFromX = useCallback((clientX: number): number => {
    if (!containerRef.current) return 0;
    const rect = containerRef.current.getBoundingClientRect();
    const x = Math.max(0, Math.min(clientX - rect.left, rect.width));
    const percent = x / rect.width;
    return percent * maxTime;
  }, [maxTime]);

  const handleMouseDown = useCallback((e: React.MouseEvent, type: 'start' | 'end' | 'region' | 'playhead') => {
    if (disabled) return;
    e.preventDefault();
    e.stopPropagation();
    setDragging(type);
    setDragStartX(e.clientX);
    setDragStartValues({ start: startTime, end: endTime, current: currentTime });
  }, [disabled, startTime, endTime, currentTime]);

  // Keyframe drag start handler (T026)
  const handleKeyframeMouseDown = useCallback((e: React.MouseEvent, keyframe: CameraKeyframe) => {
    if (disabled) return;
    e.preventDefault();
    e.stopPropagation();
    setDragging('keyframe');
    setDraggingKeyframeId(keyframe.id);
    setDragStartX(e.clientX);
    setDragKeyframeStartTime(keyframe.t);
  }, [disabled]);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!dragging) return;

    const deltaX = e.clientX - dragStartX;
    const segmentDuration = endTime - startTime;

    // Handle keyframe drag - use keyframe track ref for two-track mode
    if (dragging === 'keyframe' && draggingKeyframeId && onKeyframeTimeChange) {
      // In two-track mode, keyframe track represents segment duration
      const trackRef = showKeyframeTrack ? keyframeTrackRef.current : containerRef.current;
      if (!trackRef) return;
      const rect = trackRef.getBoundingClientRect();
      // Keyframe track is always relative to segment duration
      const deltaTime = (deltaX / rect.width) * segmentDuration;
      // Clamp to absolute time bounds (startTime and endTime in ms)
      const startTimeMs = startTime * 1000;
      const endTimeMs = endTime * 1000;
      const newTimeMs = Math.max(startTimeMs, Math.min(dragKeyframeStartTime + deltaTime * 1000, endTimeMs));
      onKeyframeTimeChange(draggingKeyframeId, newTimeMs);
      return;
    }

    // Handle keyframe track playhead drag - converts relative position to absolute time
    if (dragging === 'keyframePlayhead' && keyframeTrackRef.current) {
      const rect = keyframeTrackRef.current.getBoundingClientRect();
      const deltaTime = (deltaX / rect.width) * segmentDuration;
      const newTime = Math.max(startTime, Math.min(dragStartValues.current + deltaTime, endTime));
      onSeek?.(newTime);
      return;
    }

    // For other drags, use container ref (segment track)
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();

    // Handle playhead drag - constrain to segment bounds
    if (dragging === 'playhead') {
      const deltaTime = (deltaX / rect.width) * maxTime;
      const newTime = Math.max(startTime, Math.min(dragStartValues.current + deltaTime, endTime));
      onSeek?.(newTime);
      return;
    }

    // Handle and region dragging - use maxTime range
    const deltaTime = (deltaX / rect.width) * maxTime;

    let newStart = dragStartValues.start;
    let newEnd = dragStartValues.end;

    if (dragging === 'start') {
      newStart = Math.max(0, Math.min(dragStartValues.start + deltaTime, newEnd - 1));
    } else if (dragging === 'end') {
      newEnd = Math.max(newStart + 1, Math.min(dragStartValues.end + deltaTime, maxTime));
    } else if (dragging === 'region') {
      const duration = dragStartValues.end - dragStartValues.start;
      newStart = Math.max(0, Math.min(dragStartValues.start + deltaTime, maxTime - duration));
      newEnd = newStart + duration;
    }

    // Track the emitted values for use in handleMouseUp
    lastSegmentRef.current = { start: newStart, end: newEnd };
    onSegmentChange(newStart, newEnd);
  }, [dragging, dragStartX, dragStartValues, maxTime, onSegmentChange, onSeek, startTime, endTime, draggingKeyframeId, dragKeyframeStartTime, onKeyframeTimeChange, showKeyframeTrack]);

  const handleMouseUp = useCallback(() => {
    // Mark that we just finished dragging to prevent click event
    if (dragging && dragging !== 'playhead' && dragging !== 'keyframe' && dragging !== 'keyframePlayhead') {
      setJustFinishedDrag(true);
      // Reset the flag after a short delay
      setTimeout(() => setJustFinishedDrag(false), 100);

      // If playhead is outside the new segment bounds, push it to the nearest edge
      if (onSeek) {
        const { start: newStart, end: newEnd } = lastSegmentRef.current;
        if (currentTime < newStart) {
          onSeek(newStart);
        } else if (currentTime > newEnd) {
          onSeek(newEnd);
        }
      }
    }
    // Clear keyframe drag state (T026)
    setDraggingKeyframeId(null);
    setDragging(null);
  }, [dragging, currentTime, onSeek]);

  useEffect(() => {
    if (dragging) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
      return () => {
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [dragging, handleMouseMove, handleMouseUp]);

  // Click on timeline to seek (only on the track itself, not handles/region/keyframes)
  // Constrain seek to segment bounds
  const handleTimelineClick = useCallback((e: React.MouseEvent) => {
    if (disabled || dragging || justFinishedDrag || !onSeek) return;
    // Only seek if clicking directly on the track (not on handles, region, or keyframes)
    const target = e.target as HTMLElement;
    if (target.closest('[data-handle]') || target.closest('[data-region]') || target.closest('[data-keyframe]')) return;
    const rawTime = getTimeFromX(e.clientX);
    // Constrain to segment bounds
    const time = Math.max(startTime, Math.min(rawTime, endTime));
    onSeek(time);
  }, [disabled, dragging, justFinishedDrag, onSeek, getTimeFromX, startTime, endTime]);

  // Calculate percentages for segment track (full replay duration)
  const segmentDuration = endTime - startTime;
  const startPercent = (startTime / maxTime) * 100;
  const endPercent = (endTime / maxTime) * 100;
  const currentPercent = (currentTime / maxTime) * 100;
  const duration = segmentDuration;

  // Calculate playhead position for keyframe track (relative to segment)
  const keyframeTrackPlayheadPercent = segmentDuration > 0
    ? ((Math.max(startTime, Math.min(currentTime, endTime)) - startTime) / segmentDuration) * 100
    : 0;

  // Set start time to current playhead position
  const handleSetStart = useCallback(() => {
    if (disabled) return;
    // Ensure at least 1 second duration
    const newStart = Math.min(currentTime, endTime - 1);
    onSegmentChange(newStart, endTime);
  }, [disabled, currentTime, endTime, onSegmentChange]);

  // Set end time to current playhead position
  const handleSetEnd = useCallback(() => {
    if (disabled) return;
    // Ensure at least 1 second duration
    const newEnd = Math.max(currentTime, startTime + 1);
    onSegmentChange(startTime, newEnd);
  }, [disabled, currentTime, startTime, onSegmentChange]);

  // Start editing time on double-click (T036)
  const handleStartEditTime = useCallback((which: 'start' | 'end') => {
    if (disabled) return;
    setEditingTime(which);
    const timeValue = which === 'start' ? startTime : endTime;
    setEditInputValue(formatTimecode(timeValue));
  }, [disabled, startTime, endTime, formatTimecode]);

  // Confirm time edit
  const handleConfirmTimeEdit = useCallback(() => {
    if (!editingTime) return;

    const parsed = parseTimecode(editInputValue);
    if (parsed === null) {
      // Invalid input, cancel
      setEditingTime(null);
      return;
    }

    // Clamp to valid range
    const clamped = Math.max(0, Math.min(parsed, maxTime));

    if (editingTime === 'start') {
      // Ensure at least 1 second duration
      const newStart = Math.min(clamped, endTime - 1);
      onSegmentChange(newStart, endTime);
    } else {
      // Ensure at least 1 second duration
      const newEnd = Math.max(clamped, startTime + 1);
      onSegmentChange(startTime, newEnd);
    }

    setEditingTime(null);
  }, [editingTime, editInputValue, maxTime, startTime, endTime, onSegmentChange]);

  // Cancel time edit
  const handleCancelTimeEdit = useCallback(() => {
    setEditingTime(null);
  }, []);

  // Handle key events for time edit input
  const handleTimeEditKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleConfirmTimeEdit();
    } else if (e.key === 'Escape') {
      handleCancelTimeEdit();
    }
  }, [handleConfirmTimeEdit, handleCancelTimeEdit]);

  return (
    <div className="space-y-2">
      {/* Time labels with trim buttons and editable times (T036) */}
      <div className="flex justify-between items-center text-xs text-zinc-400">
        <div className="flex items-center gap-1">
          <button
            onClick={handleSetStart}
            disabled={disabled || currentTime >= endTime - 1}
            className={cn(
              "flex items-center gap-1 px-1.5 py-0.5 rounded text-zinc-400 hover:text-white hover:bg-zinc-700 transition-colors",
              (disabled || currentTime >= endTime - 1) && "opacity-50 cursor-not-allowed hover:bg-transparent hover:text-zinc-400"
            )}
            title="Set start to current position"
          >
            <ArrowLeftToLine className="w-3 h-3" />
          </button>
          {/* Editable start time */}
          {editingTime === 'start' ? (
            <input
              type="text"
              value={editInputValue}
              onChange={(e) => setEditInputValue(e.target.value)}
              onKeyDown={handleTimeEditKeyDown}
              onBlur={handleConfirmTimeEdit}
              autoFocus
              className="w-16 px-1 py-0.5 bg-zinc-700 border border-blue-500 rounded text-white text-center text-xs focus:outline-none"
              placeholder="0:00"
            />
          ) : (
            <span
              onDoubleClick={() => handleStartEditTime('start')}
              className={cn(
                "cursor-pointer hover:text-white transition-colors px-1 py-0.5 rounded hover:bg-zinc-700",
                !disabled && "select-none"
              )}
              title="Double-click to edit"
            >
              {formatTime(startTime)}
            </span>
          )}
        </div>
        {/* Duration display with real-time update during drag (T039) */}
        <span className={cn(
          "text-blue-400 transition-colors",
          dragging && "text-blue-300 font-medium"
        )}>
          {formatTime(duration)} duration
        </span>
        <div className="flex items-center gap-1">
          {/* Editable end time */}
          {editingTime === 'end' ? (
            <input
              type="text"
              value={editInputValue}
              onChange={(e) => setEditInputValue(e.target.value)}
              onKeyDown={handleTimeEditKeyDown}
              onBlur={handleConfirmTimeEdit}
              autoFocus
              className="w-16 px-1 py-0.5 bg-zinc-700 border border-blue-500 rounded text-white text-center text-xs focus:outline-none"
              placeholder="0:00"
            />
          ) : (
            <span
              onDoubleClick={() => handleStartEditTime('end')}
              className={cn(
                "cursor-pointer hover:text-white transition-colors px-1 py-0.5 rounded hover:bg-zinc-700",
                !disabled && "select-none"
              )}
              title="Double-click to edit"
            >
              {formatTime(endTime)}
            </span>
          )}
          <button
            onClick={handleSetEnd}
            disabled={disabled || currentTime <= startTime + 1}
            className={cn(
              "flex items-center gap-1 px-1.5 py-0.5 rounded text-zinc-400 hover:text-white hover:bg-zinc-700 transition-colors",
              (disabled || currentTime <= startTime + 1) && "opacity-50 cursor-not-allowed hover:bg-transparent hover:text-zinc-400"
            )}
            title="Set end to current position"
          >
            <ArrowRightToLine className="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* Segment selection track (full replay duration) */}
      <div
        ref={containerRef}
        className={cn(
          "relative h-8 bg-zinc-800 rounded-lg overflow-hidden cursor-pointer",
          disabled && "opacity-50 cursor-not-allowed"
        )}
        onClick={handleTimelineClick}
      >
        {/* Full track background */}
        <div className="absolute inset-0 bg-zinc-700" />

        {/* Selected region */}
        <div
          data-region
          className={cn(
            "absolute top-0 bottom-0 bg-blue-500/30 border-y-2 border-blue-500",
            !disabled && "cursor-grab active:cursor-grabbing"
          )}
          style={{
            left: `${startPercent}%`,
            width: `${endPercent - startPercent}%`,
          }}
          onMouseDown={(e) => handleMouseDown(e, 'region')}
        />

        {/* Start handle (T033-T034: larger touch target, better contrast) */}
        <div
          data-handle="start"
          className={cn(
            "absolute top-0 bottom-0 w-5 bg-blue-500 hover:bg-blue-400 transition-colors shadow-lg shadow-blue-500/30",
            "rounded-l-md",
            !disabled && "cursor-ew-resize",
            dragging === 'start' && "bg-blue-400 scale-y-110"
          )}
          style={{ left: `calc(${startPercent}% - 10px)` }}
          onMouseDown={(e) => handleMouseDown(e, 'start')}
        >
          {/* Grip lines */}
          <div className="absolute inset-y-2 left-1/2 -translate-x-1/2 flex gap-0.5">
            <div className="w-0.5 h-full bg-blue-200/60 rounded-full" />
            <div className="w-0.5 h-full bg-blue-200/60 rounded-full" />
          </div>
        </div>

        {/* End handle (T033-T034: larger touch target, better contrast) */}
        <div
          data-handle="end"
          className={cn(
            "absolute top-0 bottom-0 w-5 bg-blue-500 hover:bg-blue-400 transition-colors shadow-lg shadow-blue-500/30",
            "rounded-r-md",
            !disabled && "cursor-ew-resize",
            dragging === 'end' && "bg-blue-400 scale-y-110"
          )}
          style={{ left: `calc(${endPercent}% - 10px)` }}
          onMouseDown={(e) => handleMouseDown(e, 'end')}
        >
          {/* Grip lines */}
          <div className="absolute inset-y-2 left-1/2 -translate-x-1/2 flex gap-0.5">
            <div className="w-0.5 h-full bg-blue-200/60 rounded-full" />
            <div className="w-0.5 h-full bg-blue-200/60 rounded-full" />
          </div>
        </div>

        {/* Current time indicator (playhead) - draggable */}
        {showCurrentTime && (
          <div
            className={cn(
              "absolute top-0 bottom-0 w-4 z-20 -translate-x-1/2",
              !disabled && "cursor-ew-resize"
            )}
            style={{ left: `${currentPercent}%` }}
            onMouseDown={(e) => handleMouseDown(e, 'playhead')}
          >
            {/* Playhead line */}
            <div className="absolute left-1/2 top-0 bottom-0 w-0.5 bg-white -translate-x-1/2" />
            {/* Playhead head */}
            <div className={cn(
              "absolute -top-1 left-1/2 -translate-x-1/2 w-3 h-3 bg-white rounded-full shadow-md",
              dragging === 'playhead' && "scale-125"
            )} />
            {/* Playhead bottom */}
            <div className={cn(
              "absolute -bottom-1 left-1/2 -translate-x-1/2 w-3 h-3 bg-white rounded-full shadow-md",
              dragging === 'playhead' && "scale-125"
            )} />
          </div>
        )}
      </div>

      {/* Keyframe track (zoomed to segment - segment fills 100%) */}
      {showKeyframeTrack && (
        <>
          {/* Label for keyframe track */}
          <div className="flex items-center justify-between text-xs text-zinc-400 mt-2">
            <span className="text-purple-400 font-medium">Keyframes</span>
            <span>{formatTime(startTime)} → {formatTime(endTime)} ({formatTime(duration)})</span>
          </div>

          {/* Keyframe track */}
          <div
            ref={keyframeTrackRef}
            className={cn(
              "relative h-6 bg-zinc-800 rounded-lg overflow-hidden cursor-pointer",
              disabled && "opacity-50 cursor-not-allowed"
            )}
            onClick={(e) => {
              // Click to seek on keyframe track
              if (disabled || dragging || justFinishedDrag || !onSeek) return;
              const target = e.target as HTMLElement;
              if (target.closest('[data-keyframe]')) return; // Don't seek when clicking keyframes
              if (!keyframeTrackRef.current) return;
              const rect = keyframeTrackRef.current.getBoundingClientRect();
              const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
              const percent = x / rect.width;
              // Convert from keyframe track (0-100% of segment) to absolute time
              const time = startTime + percent * segmentDuration;
              onSeek(time);
            }}
          >
            {/* Track background */}
            <div className="absolute inset-0 bg-zinc-700" />

            {/* Keyframe markers - positioned relative to segment (0-100%) */}
            {keyframes.length > 0 && keyframes.map((kf) => {
              const clipDurationMs = segmentDuration * 1000;
              const startTimeMs = startTime * 1000;
              // Calculate position relative to clip start (kf.t is absolute time in ms)
              const rawPercent = clipDurationMs > 0 ? ((kf.t - startTimeMs) / clipDurationMs) * 100 : 0;
              const kfPercent = Math.max(0, Math.min(100, rawPercent));

              const isSelected = kf.id === selectedKeyframeId;
              const isDragging = kf.id === draggingKeyframeId;

              return (
                <div
                  key={kf.id}
                  data-keyframe={kf.id}
                  className={cn(
                    "absolute top-1/2 -translate-y-1/2 -translate-x-1/2 z-10",
                    !disabled && "cursor-ew-resize"
                  )}
                  style={{ left: `${kfPercent}%` }}
                  onMouseDown={(e) => handleKeyframeMouseDown(e, kf)}
                  onClick={(e) => {
                    e.stopPropagation();
                    onKeyframeClick?.(kf);
                  }}
                >
                  {/* Diamond shape */}
                  <div
                    className={cn(
                      "w-3 h-3 rotate-45 transition-all",
                      isSelected || isDragging
                        ? "bg-amber-400 scale-125"
                        : "bg-amber-500 hover:bg-amber-400 hover:scale-110"
                    )}
                  />
                </div>
              );
            })}

            {/* Playhead on keyframe track (relative to segment) - draggable */}
            {showCurrentTime && currentTime >= startTime && currentTime <= endTime && (
              <div
                className={cn(
                  "absolute top-0 bottom-0 w-4 z-20 -translate-x-1/2",
                  !disabled && "cursor-ew-resize"
                )}
                style={{ left: `${keyframeTrackPlayheadPercent}%` }}
                onMouseDown={(e) => {
                  if (disabled) return;
                  e.preventDefault();
                  e.stopPropagation();
                  setDragging('keyframePlayhead');
                  setDragStartX(e.clientX);
                  setDragStartValues({ start: startTime, end: endTime, current: currentTime });
                }}
              >
                {/* Playhead line */}
                <div className="absolute left-1/2 top-0 bottom-0 w-0.5 bg-white -translate-x-1/2" />
                {/* Playhead top handle */}
                <div className={cn(
                  "absolute -top-1 left-1/2 -translate-x-1/2 w-2.5 h-2.5 bg-white rounded-full shadow-md",
                  dragging === 'keyframePlayhead' && "scale-125"
                )} />
                {/* Playhead bottom handle */}
                <div className={cn(
                  "absolute -bottom-1 left-1/2 -translate-x-1/2 w-2.5 h-2.5 bg-white rounded-full shadow-md",
                  dragging === 'keyframePlayhead' && "scale-125"
                )} />
              </div>
            )}
          </div>
        </>
      )}

      {/* Instruction text */}
      <p className="text-xs text-zinc-500 text-center">
        {showKeyframeTrack
          ? 'Top: select clip zone. Bottom: drag playhead to seek, drag keyframes to adjust timing.'
          : 'Drag the white playhead to seek within clip, drag blue handles to adjust range'}
      </p>
    </div>
  );
}
