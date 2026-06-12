import React, { useRef, useState, useCallback, useEffect } from 'react';
import { Play, Pause, SkipBack, SkipForward } from 'lucide-react';
import { IoMdFootball } from "react-icons/io";
import { GiMineExplosion, GiShieldImpact } from "react-icons/gi";
import { cn } from '@/lib/utils';

interface PlaybackEvent {
    type: string;
    time: number;
    color: string;
    description?: string;
    team?: number;
}

interface PlaybackControlsProps {
    isPlaying: boolean;
    currentTime: number;
    maxTime: number;
    onPlayPause: () => void;
    onSeek: (time: number) => void;
    onSeekCommit?: (time: number) => void;
    onEventClick?: (event: PlaybackEvent) => void;
    playbackSpeed: number;
    onPlaybackSpeedChange: (speed: number) => void;
    events?: PlaybackEvent[];
    controlsDisabled?: boolean;
}

const PLAYBACK_SPEEDS = [0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 1, 1.25, 1.5, 2];

export function PlaybackControls({
    isPlaying,
    currentTime,
    maxTime,
    onPlayPause,
    onSeek,
    onSeekCommit,
    onEventClick,
    playbackSpeed = 1.0,
    onPlaybackSpeedChange,
    events = [],
    controlsDisabled = false,
}: PlaybackControlsProps) {
    const progressBarRef = useRef<HTMLDivElement>(null);
    const [isDragging, setIsDragging] = useState(false);
    const [hoverTime, setHoverTime] = useState<number | null>(null);
    const [hoverPosition, setHoverPosition] = useState<number>(0);
    const [dragProgress, setDragProgress] = useState<number | null>(null);
    const dragTimeRef = useRef<number>(0);

    const formatTime = (time: number) => {
        const minutes = Math.floor(time / 60);
        const seconds = Math.floor(time % 60).toString().padStart(2, '0');
        return `${minutes}:${seconds}`;
    };

    const formatTimeDetailed = (time: number) => {
        const minutes = Math.floor(time / 60);
        const seconds = Math.floor(time % 60).toString().padStart(2, '0');
        const ms = Math.floor((time % 1) * 100).toString().padStart(2, '0');
        return `${minutes}:${seconds}.${ms}`;
    };

    const getTimeFromPosition = useCallback((clientX: number) => {
        if (!progressBarRef.current) return 0;
        const rect = progressBarRef.current.getBoundingClientRect();
        const x = Math.max(0, Math.min(clientX - rect.left, rect.width));
        return (x / rect.width) * maxTime;
    }, [maxTime]);

    const getProgressFromPosition = useCallback((clientX: number) => {
        if (!progressBarRef.current) return 0;
        const rect = progressBarRef.current.getBoundingClientRect();
        const x = Math.max(0, Math.min(clientX - rect.left, rect.width));
        return (x / rect.width) * 100;
    }, []);

    const handleMouseDown = useCallback((e: React.MouseEvent) => {
        if (controlsDisabled) return;
        e.preventDefault();
        setIsDragging(true);
        const time = getTimeFromPosition(e.clientX);
        const progress = getProgressFromPosition(e.clientX);
        dragTimeRef.current = time;
        setDragProgress(progress);
        onSeek(time);
    }, [controlsDisabled, getTimeFromPosition, getProgressFromPosition, onSeek]);

    const handleLocalMouseMove = useCallback((e: React.MouseEvent) => {
        if (!progressBarRef.current) return;
        const rect = progressBarRef.current.getBoundingClientRect();
        const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
        setHoverPosition(x);
        setHoverTime(getTimeFromPosition(e.clientX));
    }, [getTimeFromPosition]);

    const handleMouseLeave = useCallback(() => {
        if (!isDragging) {
            setHoverTime(null);
        }
    }, [isDragging]);

    // Global mouse move and up handlers for drag
    useEffect(() => {
        if (!isDragging) return;

        const handleGlobalMouseMove = (e: MouseEvent) => {
            if (!progressBarRef.current || controlsDisabled) return;
            const time = getTimeFromPosition(e.clientX);
            const progress = getProgressFromPosition(e.clientX);
            dragTimeRef.current = time;
            setDragProgress(progress);
            onSeek(time);

            // Update hover position during drag
            const rect = progressBarRef.current.getBoundingClientRect();
            const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
            setHoverPosition(x);
            setHoverTime(time);
        };

        const handleGlobalMouseUp = () => {
            setIsDragging(false);
            setDragProgress(null);
            onSeekCommit?.(dragTimeRef.current);
            setHoverTime(null);
        };

        window.addEventListener('mousemove', handleGlobalMouseMove);
        window.addEventListener('mouseup', handleGlobalMouseUp);

        return () => {
            window.removeEventListener('mousemove', handleGlobalMouseMove);
            window.removeEventListener('mouseup', handleGlobalMouseUp);
        };
    }, [isDragging, controlsDisabled, getTimeFromPosition, getProgressFromPosition, onSeek, onSeekCommit]);

    const skipBack = () => {
        if (controlsDisabled) return;
        const newTime = Math.max(0, currentTime - 5);
        onSeek(newTime);
        onSeekCommit?.(newTime);
    };

    const skipForward = () => {
        if (controlsDisabled) return;
        const newTime = Math.min(maxTime, currentTime + 5);
        onSeek(newTime);
        onSeekCommit?.(newTime);
    };

    // Use dragProgress during drag for instant feedback, otherwise use calculated percent
    const displayPercent = isDragging && dragProgress !== null
        ? dragProgress
        : (maxTime > 0 ? (currentTime / maxTime) * 100 : 0);

    const getEventIcon = (type: string) => {
        switch (type) {
            case 'save': return GiShieldImpact;
            case 'demo': return GiMineExplosion;
            default: return IoMdFootball;
        }
    };

    return (
        <div className="playback-controls-container">
            {/* Main controls bar */}
            <div className="playback-controls">
                {/* Left section: Play controls */}
                <div className="playback-controls-left">
                    <button
                        onClick={skipBack}
                        disabled={controlsDisabled}
                        className="playback-btn playback-btn-secondary"
                        title="Reculer de 5 secondes"
                    >
                        <SkipBack size={18} />
                    </button>

                    <button
                        onClick={onPlayPause}
                        disabled={controlsDisabled}
                        className="playback-btn playback-btn-primary"
                    >
                        {isPlaying ? <Pause size={22} /> : <Play size={22} className="ml-0.5" />}
                    </button>

                    <button
                        onClick={skipForward}
                        disabled={controlsDisabled}
                        className="playback-btn playback-btn-secondary"
                        title="Avancer de 5 secondes"
                    >
                        <SkipForward size={18} />
                    </button>
                </div>

                {/* Center section: Time and Progress */}
                <div className="playback-controls-center">
                    {/* Current time */}
                    <span className="playback-time playback-time-current">
                        {formatTimeDetailed(currentTime)}
                    </span>

                    {/* Progress bar container */}
                    <div className="playback-progress-wrapper">
                        {/* Event markers above the progress bar */}
                        <div className="playback-events">
                            {events.map((event, idx) => {
                                const Icon = getEventIcon(event.type);
                                const leftPercent = (event.time / maxTime) * 100;

                                return (
                                    <div
                                        key={idx}
                                        className="playback-event-marker"
                                        style={{ left: `${leftPercent}%` }}
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            if (!controlsDisabled && onEventClick) {
                                                onEventClick(event);
                                            }
                                        }}
                                    >
                                        <div
                                            className="playback-event-icon"
                                            style={{ backgroundColor: event.color }}
                                        >
                                            <Icon size={12} color="#fff" />
                                        </div>
                                        {/* Event tooltip */}
                                        <div className="playback-event-tooltip">
                                            <div className="playback-event-tooltip-type" style={{ color: event.color }}>
                                                <Icon size={12} />
                                                <span>{event.type.toUpperCase()}</span>
                                            </div>
                                            {event.description && (
                                                <div className="playback-event-tooltip-desc">{event.description}</div>
                                            )}
                                            <div className="playback-event-tooltip-time">{formatTime(event.time)}</div>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>

                        {/* Progress bar */}
                        <div
                            ref={progressBarRef}
                            className={cn(
                                "playback-progress",
                                controlsDisabled && "playback-progress-disabled",
                                isDragging && "playback-progress-dragging"
                            )}
                            onMouseDown={handleMouseDown}
                            onMouseMove={handleLocalMouseMove}
                            onMouseLeave={handleMouseLeave}
                        >
                            {/* Background track */}
                            <div className="playback-progress-track" />

                            {/* Buffered/loaded indicator (optional visual) */}
                            <div className="playback-progress-buffer" style={{ width: '100%' }} />

                            {/* Progress fill - uses displayPercent for instant drag feedback */}
                            <div
                                className="playback-progress-fill"
                                style={{ width: `${displayPercent}%` }}
                            />

                            {/* Hover preview - only when not dragging */}
                            {hoverTime !== null && !isDragging && (
                                <div
                                    className="playback-progress-hover"
                                    style={{ width: `${(hoverTime / maxTime) * 100}%` }}
                                />
                            )}

                            {/* Hover time tooltip */}
                            {hoverTime !== null && (
                                <div
                                    className="playback-hover-tooltip"
                                    style={{ left: `${hoverPosition}px` }}
                                >
                                    {formatTimeDetailed(hoverTime)}
                                </div>
                            )}
                        </div>
                    </div>

                    {/* Total time */}
                    <span className="playback-time playback-time-total">
                        {formatTime(maxTime)}
                    </span>
                </div>

                {/* Right section: Speed control */}
                <div className="playback-controls-right">
                    <select
                        value={playbackSpeed}
                        onChange={(e) => onPlaybackSpeedChange(Number(e.target.value))}
                        disabled={controlsDisabled}
                        className="playback-speed-select"
                    >
                        {PLAYBACK_SPEEDS.map(speed => (
                            <option key={speed} value={speed}>
                                {speed}x
                            </option>
                        ))}
                    </select>
                </div>
            </div>
        </div>
    );
}
