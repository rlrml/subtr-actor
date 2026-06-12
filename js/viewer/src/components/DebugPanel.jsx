import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import {
    Bug,
    ChevronDown,
    ChevronRight,
    Activity,
    Circle,
    X,
    Settings,
    BarChart3,
    Users,
    Zap,
    ArrowUp,
    Fuel,
    GripVertical,
    Wand2,
    Upload
} from 'lucide-react';

// ============================================================================
// PANEL STORAGE & DIMENSIONS
// ============================================================================

const PANEL_STORAGE_KEY = 'debug-panel-state';

const DEFAULT_PANEL_STATE = {
    x: 16,
    y: typeof window !== 'undefined' ? window.innerHeight - 500 : 200,
    width: 360,
    height: 450,
};

const MIN_WIDTH = 320;
const MAX_WIDTH = 600;
const MIN_HEIGHT = 300;
const MAX_HEIGHT = typeof window !== 'undefined' ? window.innerHeight - 100 : 700;

function loadPanelState() {
    try {
        const saved = localStorage.getItem(PANEL_STORAGE_KEY);
        if (saved) {
            const parsed = JSON.parse(saved);
            return {
                x: Math.max(0, Math.min(parsed.x ?? DEFAULT_PANEL_STATE.x, window.innerWidth - 100)),
                y: Math.max(0, Math.min(parsed.y ?? DEFAULT_PANEL_STATE.y, window.innerHeight - 100)),
                width: Math.max(MIN_WIDTH, Math.min(parsed.width ?? DEFAULT_PANEL_STATE.width, MAX_WIDTH)),
                height: Math.max(MIN_HEIGHT, Math.min(parsed.height ?? DEFAULT_PANEL_STATE.height, MAX_HEIGHT)),
            };
        }
    } catch {
        // Ignore parse errors
    }
    return { ...DEFAULT_PANEL_STATE };
}

function savePanelState(state) {
    try {
        localStorage.setItem(PANEL_STORAGE_KEY, JSON.stringify(state));
    } catch {
        // Ignore storage errors
    }
}

// ============================================================================
// VELOCITY HISTORY TRACKING
// ============================================================================

const HISTORY_SIZE = 300; // ~5 seconds at 60fps
const SUPERSONIC_THRESHOLD = 2200; // uu/s

function useVelocityHistory(position, prevPositionRef, deltaTime) {
    const historyRef = useRef([]);

    useEffect(() => {
        if (!position || !prevPositionRef.current) {
            prevPositionRef.current = position ? { ...position } : null;
            return;
        }

        const dx = position.x - prevPositionRef.current.x;
        const dy = position.y - prevPositionRef.current.y;
        const dz = position.z - prevPositionRef.current.z;
        const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
        const velocity = deltaTime > 0 ? distance / deltaTime : 0;

        historyRef.current.push({
            time: Date.now(),
            velocity: velocity,
            position: { ...position }
        });

        // Keep only last N samples
        if (historyRef.current.length > HISTORY_SIZE) {
            historyRef.current = historyRef.current.slice(-HISTORY_SIZE);
        }

        prevPositionRef.current = { ...position };
    }, [position, deltaTime]);

    return historyRef.current;
}

// ============================================================================
// GENERIC PHYSICS GRAPH COMPONENT
// ============================================================================

const TIME_WINDOW_MS = 5000; // Show last 5 seconds

// Generic graph component for any metric
function PhysicsGraph({
    data,
    color,
    label,
    height = 80,
    unit = '',
    thresholdValue = null,
    thresholdLabel = '',
    minValue = null,
    maxValueDefault = 100,
    valueFormatter = (v) => v.toFixed(0)
}) {
    const width = 280;
    const padding = { top: 5, right: 5, bottom: 15, left: 35 };
    const graphWidth = width - padding.left - padding.right;
    const graphHeight = height - padding.top - padding.bottom;

    // Generate unique ID for gradient
    const gradientId = useMemo(() => `physics-grad-${label.replace(/[^a-zA-Z0-9]/g, '')}-${Math.random().toString(36).substr(2, 9)}`, [label]);

    // Filter data to time window and calculate path
    const { filteredData, maxValue, minVal, pathData, areaPath } = useMemo(() => {
        const now = Date.now();
        const minTime = now - TIME_WINDOW_MS;
        const filtered = data.filter(d => d.time >= minTime);

        if (filtered.length === 0) {
            return { filteredData: [], maxValue: maxValueDefault, minVal: minValue || 0, pathData: '', areaPath: '' };
        }

        const values = filtered.map(d => d.value);
        const dataMax = Math.max(...values);
        const dataMin = Math.min(...values);
        const maxVal = Math.max(maxValueDefault, dataMax * 1.1, thresholdValue ? thresholdValue * 1.1 : 0);
        const minVal = minValue !== null ? minValue : Math.min(0, dataMin);
        const range = maxVal - minVal;

        const points = filtered.map(point => {
            const x = padding.left + ((point.time - minTime) / TIME_WINDOW_MS) * graphWidth;
            const y = padding.top + graphHeight - ((point.value - minVal) / range) * graphHeight;
            return { x, y };
        });

        const linePath = points.map((p, i) =>
            `${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`
        ).join(' ');

        const area = points.length > 1
            ? `${linePath} L ${points[points.length - 1].x.toFixed(1)} ${padding.top + graphHeight} L ${points[0].x.toFixed(1)} ${padding.top + graphHeight} Z`
            : '';

        return { filteredData: filtered, maxValue: maxVal, minVal, pathData: linePath, areaPath: area };
    }, [data, graphWidth, graphHeight, maxValueDefault, thresholdValue, minValue]);

    const currentValue = filteredData.length > 0 ? filteredData[filteredData.length - 1].value : 0;
    const isAboveThreshold = thresholdValue && currentValue >= thresholdValue;
    const range = maxValue - minVal;

    // Y-axis labels (dynamic based on range)
    const yLabels = useMemo(() => {
        const labels = [];
        const step = range / 4;
        for (let i = 0; i <= 4; i++) {
            labels.push(minVal + step * i);
        }
        return labels;
    }, [range, minVal]);

    return (
        <div className="bg-gray-900/50 rounded-lg border border-gray-700/50 p-2 overflow-hidden">
            <div className="flex items-center justify-between mb-1">
                <span className="text-[10px] text-gray-400">{label}</span>
                <span className={`text-xs font-mono font-bold ${isAboveThreshold ? 'text-red-400' : 'text-white'}`}>
                    {valueFormatter(currentValue)}{unit}
                </span>
            </div>
            <svg width="100%" viewBox={`0 0 ${width} ${height}`} className="overflow-hidden">
                {/* Grid lines */}
                {yLabels.map((v, i) => {
                    const y = padding.top + graphHeight - ((v - minVal) / range) * graphHeight;
                    const isThreshold = thresholdValue && Math.abs(v - thresholdValue) < range * 0.05;
                    return (
                        <g key={i}>
                            <line
                                x1={padding.left}
                                y1={y}
                                x2={width - padding.right}
                                y2={y}
                                stroke={isThreshold ? '#ef4444' : '#374151'}
                                strokeWidth={isThreshold ? 1 : 0.5}
                                strokeDasharray={isThreshold ? '3,3' : undefined}
                                opacity={0.5}
                            />
                            <text
                                x={padding.left - 3}
                                y={y}
                                textAnchor="end"
                                dominantBaseline="middle"
                                className="text-[8px] fill-gray-500"
                            >
                                {v.toFixed(0)}
                            </text>
                        </g>
                    );
                })}

                {/* Threshold line if specified */}
                {thresholdValue && (
                    <line
                        x1={padding.left}
                        y1={padding.top + graphHeight - ((thresholdValue - minVal) / range) * graphHeight}
                        x2={width - padding.right}
                        y2={padding.top + graphHeight - ((thresholdValue - minVal) / range) * graphHeight}
                        stroke="#ef4444"
                        strokeWidth={1}
                        strokeDasharray="3,3"
                        opacity={0.7}
                    />
                )}

                {/* Gradient fill */}
                <defs>
                    <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={color} stopOpacity="0.4" />
                        <stop offset="100%" stopColor={color} stopOpacity="0.05" />
                    </linearGradient>
                </defs>

                {/* Area fill */}
                {areaPath && (
                    <path d={areaPath} fill={`url(#${gradientId})`} />
                )}

                {/* Line */}
                {pathData && (
                    <path
                        d={pathData}
                        fill="none"
                        stroke={color}
                        strokeWidth={1.5}
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    />
                )}

                {/* Current value dot */}
                {filteredData.length > 0 && (
                    <circle
                        cx={padding.left + graphWidth}
                        cy={padding.top + graphHeight - ((currentValue - minVal) / range) * graphHeight}
                        r={3}
                        fill={isAboveThreshold ? '#ef4444' : color}
                    />
                )}

                {/* Threshold label */}
                {isAboveThreshold && thresholdLabel && (
                    <text
                        x={width - padding.right}
                        y={padding.top + graphHeight - ((thresholdValue - minVal) / range) * graphHeight - 8}
                        textAnchor="end"
                        className="text-[7px] fill-red-400 font-bold"
                    >
                        {thresholdLabel}
                    </text>
                )}

                {/* Time axis labels */}
                <text x={padding.left} y={height - 2} textAnchor="start" className="text-[7px] fill-gray-600">-5s</text>
                <text x={padding.left + graphWidth / 2} y={height - 2} textAnchor="middle" className="text-[7px] fill-gray-600">-2.5s</text>
                <text x={padding.left + graphWidth} y={height - 2} textAnchor="end" className="text-[7px] fill-gray-600">now</text>
            </svg>
        </div>
    );
}

// ============================================================================
// VELOCITY GRAPH COMPONENT (backward compatible wrapper)
// ============================================================================

function VelocityGraph({ data, color, label, height = 80 }) {
    const width = 280;
    const padding = { top: 5, right: 5, bottom: 15, left: 35 };
    const graphWidth = width - padding.left - padding.right;
    const graphHeight = height - padding.top - padding.bottom;

    // Generate unique ID for gradient (avoid special chars in SVG IDs)
    const gradientId = useMemo(() => `vel-grad-${label.replace(/[^a-zA-Z0-9]/g, '')}-${Math.random().toString(36).substr(2, 9)}`, [label]);

    // Filter data to time window and calculate path
    const { filteredData, maxVelocity, pathData, areaPath } = useMemo(() => {
        const now = Date.now();
        const minTime = now - TIME_WINDOW_MS;

        // Filter to last 5 seconds
        const filtered = data.filter(d => d.time >= minTime);

        if (filtered.length === 0) {
            return { filteredData: [], maxVelocity: SUPERSONIC_THRESHOLD, pathData: '', areaPath: '' };
        }

        // Calculate max velocity for scale
        const max = Math.max(...filtered.map(d => d.velocity));
        const maxVel = Math.max(SUPERSONIC_THRESHOLD * 1.1, max * 1.1);

        // Create path - X is based on time position in window
        const points = filtered.map(point => {
            const x = padding.left + ((point.time - minTime) / TIME_WINDOW_MS) * graphWidth;
            const y = padding.top + graphHeight - (point.velocity / maxVel) * graphHeight;
            return { x, y };
        });

        const linePath = points.map((p, i) =>
            `${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`
        ).join(' ');

        // Area path for fill
        const area = points.length > 1
            ? `${linePath} L ${points[points.length - 1].x.toFixed(1)} ${padding.top + graphHeight} L ${points[0].x.toFixed(1)} ${padding.top + graphHeight} Z`
            : '';

        return { filteredData: filtered, maxVelocity: maxVel, pathData: linePath, areaPath: area };
    }, [data, graphWidth, graphHeight]);

    // Current velocity
    const currentVelocity = filteredData.length > 0 ? filteredData[filteredData.length - 1].velocity : 0;
    const isSupersonic = currentVelocity >= SUPERSONIC_THRESHOLD;

    // Y-axis labels
    const yLabels = [0, 1000, 2000, SUPERSONIC_THRESHOLD];

    return (
        <div className="bg-gray-900/50 rounded-lg border border-gray-700/50 p-2 overflow-hidden">
            <div className="flex items-center justify-between mb-1">
                <span className="text-[10px] text-gray-400">{label}</span>
                <span className={`text-xs font-mono font-bold ${isSupersonic ? 'text-red-400' : 'text-white'}`}>
                    {(currentVelocity / 27.78).toFixed(0)} km/h
                </span>
            </div>
            <svg width="100%" viewBox={`0 0 ${width} ${height}`} className="overflow-hidden">
                {/* Grid lines */}
                {yLabels.map((v) => {
                    const y = padding.top + graphHeight - (v / maxVelocity) * graphHeight;
                    return (
                        <g key={v}>
                            <line
                                x1={padding.left}
                                y1={y}
                                x2={width - padding.right}
                                y2={y}
                                stroke={v === SUPERSONIC_THRESHOLD ? '#ef4444' : '#374151'}
                                strokeWidth={v === SUPERSONIC_THRESHOLD ? 1 : 0.5}
                                strokeDasharray={v === SUPERSONIC_THRESHOLD ? '3,3' : undefined}
                                opacity={0.5}
                            />
                            <text
                                x={padding.left - 3}
                                y={y}
                                textAnchor="end"
                                dominantBaseline="middle"
                                className="text-[8px] fill-gray-500"
                            >
                                {v}
                            </text>
                        </g>
                    );
                })}

                {/* Gradient fill */}
                <defs>
                    <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={color} stopOpacity="0.4" />
                        <stop offset="100%" stopColor={color} stopOpacity="0.05" />
                    </linearGradient>
                </defs>

                {/* Area fill */}
                {areaPath && (
                    <path
                        d={areaPath}
                        fill={`url(#${gradientId})`}
                    />
                )}

                {/* Line */}
                {pathData && (
                    <path
                        d={pathData}
                        fill="none"
                        stroke={color}
                        strokeWidth={1.5}
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    />
                )}

                {/* Current value dot - at the end of the line */}
                {filteredData.length > 0 && (
                    <circle
                        cx={padding.left + graphWidth}
                        cy={padding.top + graphHeight - (currentVelocity / maxVelocity) * graphHeight}
                        r={3}
                        fill={isSupersonic ? '#ef4444' : color}
                    />
                )}

                {/* Supersonic label */}
                {isSupersonic && (
                    <text
                        x={width - padding.right}
                        y={padding.top + graphHeight - (SUPERSONIC_THRESHOLD / maxVelocity) * graphHeight - 8}
                        textAnchor="end"
                        className="text-[7px] fill-red-400 font-bold"
                    >
                        SUPERSONIC
                    </text>
                )}

                {/* Time axis labels */}
                <text
                    x={padding.left}
                    y={height - 2}
                    textAnchor="start"
                    className="text-[7px] fill-gray-600"
                >
                    -5s
                </text>
                <text
                    x={padding.left + graphWidth / 2}
                    y={height - 2}
                    textAnchor="middle"
                    className="text-[7px] fill-gray-600"
                >
                    -2.5s
                </text>
                <text
                    x={padding.left + graphWidth}
                    y={height - 2}
                    textAnchor="end"
                    className="text-[7px] fill-gray-600"
                >
                    now
                </text>
            </svg>
        </div>
    );
}

// ============================================================================
// POSITION DELTA GRAPH - Shows actual movement per frame (detects stalls)
// ============================================================================

function PositionDeltaGraph({ positionData, color, label, height = 80 }) {
    const width = 280;
    const padding = { top: 5, right: 5, bottom: 15, left: 35 };
    const graphWidth = width - padding.left - padding.right;
    const graphHeight = height - padding.top - padding.bottom;

    // Calculate position delta (actual movement between frames)
    const { deltaData, maxDelta, pathData, stats } = useMemo(() => {
        const now = Date.now();
        const minTime = now - TIME_WINDOW_MS;

        // Filter to last 5 seconds
        const filtered = positionData.filter(d => d.time >= minTime);

        if (filtered.length < 2) {
            return { deltaData: [], maxDelta: 100, pathData: '', stats: { avgDelta: 0, stallCount: 0, stallPercent: 0 } };
        }

        // Calculate position delta between consecutive frames
        const deltaPoints = [];
        for (let i = 1; i < filtered.length; i++) {
            const dt = (filtered[i].time - filtered[i - 1].time) / 1000; // seconds
            if (dt > 0 && dt < 0.1) { // Ignore gaps > 100ms
                const dx = filtered[i].x - filtered[i - 1].x;
                const dy = filtered[i].y - filtered[i - 1].y;
                const dz = filtered[i].z - filtered[i - 1].z;
                const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);

                // Calculate expected speed (distance / time)
                const speed = distance / dt; // uu/s

                deltaPoints.push({
                    time: filtered[i].time,
                    delta: distance,
                    speed: speed,
                    dt: dt
                });
            }
        }

        if (deltaPoints.length === 0) {
            return { deltaData: [], maxDelta: 100, pathData: '', stats: { avgDelta: 0, stallCount: 0, stallPercent: 0 } };
        }

        // Calculate stats
        const avgSpeed = deltaPoints.reduce((a, b) => a + b.speed, 0) / deltaPoints.length;

        // Stall detection: speed drops below 20% of average suddenly
        let stallCount = 0;
        for (let i = 1; i < deltaPoints.length; i++) {
            const prevSpeed = deltaPoints[i - 1].speed;
            const currSpeed = deltaPoints[i].speed;
            // Detect sudden drops (prev was > 500, now < 100)
            if (prevSpeed > 500 && currSpeed < 100) {
                stallCount++;
            }
        }
        const stallPercent = (stallCount / deltaPoints.length) * 100;

        // Max for scale
        const speeds = deltaPoints.map(d => d.speed);
        const maxSpeed = Math.max(2000, Math.max(...speeds) * 1.1);

        // Create path
        const points = deltaPoints.map(point => {
            const x = padding.left + ((point.time - minTime) / TIME_WINDOW_MS) * graphWidth;
            const y = padding.top + graphHeight - (point.speed / maxSpeed) * graphHeight;
            return { x, y };
        });

        const linePath = points.map((p, i) =>
            `${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`
        ).join(' ');

        return {
            deltaData: deltaPoints,
            maxDelta: maxSpeed,
            pathData: linePath,
            stats: { avgDelta: avgSpeed, stallCount, stallPercent }
        };
    }, [positionData, graphWidth, graphHeight]);

    const currentSpeed = deltaData.length > 0 ? deltaData[deltaData.length - 1].speed : 0;

    return (
        <div className="bg-gray-900/50 rounded-lg border border-gray-700/50 p-2 overflow-hidden">
            <div className="flex items-center justify-between mb-1">
                <span className="text-[10px] text-gray-400">{label}</span>
                <div className="flex items-center gap-2">
                    <span className={`text-[9px] px-1.5 py-0.5 rounded ${stats.stallPercent > 5 ? 'bg-red-500/30 text-red-300' : stats.stallPercent > 1 ? 'bg-yellow-500/30 text-yellow-300' : 'bg-green-500/30 text-green-300'}`}>
                        Stalls: {stats.stallCount}
                    </span>
                    <span className={`text-xs font-mono font-bold ${currentSpeed < 100 ? 'text-red-400' : 'text-white'}`}>
                        {(currentSpeed / 27.78).toFixed(0)} km/h
                    </span>
                </div>
            </div>
            <svg width="100%" viewBox={`0 0 ${width} ${height}`} className="overflow-hidden">
                {/* Grid lines */}
                {[0, 500, 1000, 2000].map((v) => {
                    const y = padding.top + graphHeight - (v / maxDelta) * graphHeight;
                    return (
                        <g key={v}>
                            <line
                                x1={padding.left}
                                y1={y}
                                x2={width - padding.right}
                                y2={y}
                                stroke={v === 0 ? '#ef4444' : '#374151'}
                                strokeWidth={v === 0 ? 1 : 0.5}
                                opacity={0.5}
                            />
                            <text
                                x={padding.left - 3}
                                y={y}
                                textAnchor="end"
                                dominantBaseline="middle"
                                className="text-[7px] fill-gray-500"
                            >
                                {v}
                            </text>
                        </g>
                    );
                })}

                {/* Stall threshold line (100 uu/s = ~3.6 km/h) */}
                <line
                    x1={padding.left}
                    y1={padding.top + graphHeight - (100 / maxDelta) * graphHeight}
                    x2={width - padding.right}
                    y2={padding.top + graphHeight - (100 / maxDelta) * graphHeight}
                    stroke="#ef4444"
                    strokeWidth={1}
                    strokeDasharray="3,3"
                    opacity={0.7}
                />

                {/* Line */}
                {pathData && (
                    <path
                        d={pathData}
                        fill="none"
                        stroke={color}
                        strokeWidth={1.5}
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    />
                )}

                {/* Current value dot */}
                {deltaData.length > 0 && (
                    <circle
                        cx={padding.left + graphWidth}
                        cy={padding.top + graphHeight - (currentSpeed / maxDelta) * graphHeight}
                        r={3}
                        fill={currentSpeed < 100 ? '#ef4444' : color}
                    />
                )}

                {/* Time axis */}
                <text x={padding.left} y={height - 2} textAnchor="start" className="text-[7px] fill-gray-600">-5s</text>
                <text x={padding.left + graphWidth} y={height - 2} textAnchor="end" className="text-[7px] fill-gray-600">now</text>
            </svg>
            <div className="text-[8px] text-gray-500 mt-1">
                Red dashed = stall threshold (3.6 km/h). Drops to 0 = object stopped moving.
            </div>
        </div>
    );
}

// ============================================================================
// ACCELERATION GRAPH COMPONENT - Shows velocity changes to detect jitter
// ============================================================================

function AccelerationGraph({ data, color, label, height = 80 }) {
    const width = 280;
    const padding = { top: 5, right: 5, bottom: 15, left: 35 };
    const graphWidth = width - padding.left - padding.right;
    const graphHeight = height - padding.top - padding.bottom;

    const gradientId = useMemo(() => `accel-grad-${label.replace(/[^a-zA-Z0-9]/g, '')}-${Math.random().toString(36).substr(2, 9)}`, [label]);

    // Calculate acceleration (velocity change) from velocity data
    const { accelerationData, maxAccel, pathData, zeroLineY, stats } = useMemo(() => {
        const now = Date.now();
        const minTime = now - TIME_WINDOW_MS;

        // Filter to last 5 seconds
        const filtered = data.filter(d => d.time >= minTime);

        if (filtered.length < 2) {
            return { accelerationData: [], maxAccel: 1000, pathData: '', zeroLineY: height / 2, stats: { avg: 0, max: 0, jitter: 0 } };
        }

        // Calculate acceleration (velocity change per frame)
        const accelPoints = [];
        for (let i = 1; i < filtered.length; i++) {
            const dt = (filtered[i].time - filtered[i - 1].time) / 1000; // seconds
            if (dt > 0) {
                const dv = filtered[i].velocity - filtered[i - 1].velocity;
                const acceleration = dv / dt; // uu/s²
                accelPoints.push({
                    time: filtered[i].time,
                    acceleration: acceleration
                });
            }
        }

        if (accelPoints.length === 0) {
            return { accelerationData: [], maxAccel: 1000, pathData: '', zeroLineY: height / 2, stats: { avg: 0, max: 0, jitter: 0 } };
        }

        // Calculate stats
        const absAccels = accelPoints.map(p => Math.abs(p.acceleration));
        const avgAccel = absAccels.reduce((a, b) => a + b, 0) / absAccels.length;
        const maxAccelValue = Math.max(...absAccels);

        // Jitter score: count of sudden changes (acceleration > 5000 uu/s²)
        const jitterCount = absAccels.filter(a => a > 5000).length;
        const jitterPercent = (jitterCount / absAccels.length) * 100;

        // Calculate max for scale (symmetric around 0)
        const max = Math.max(5000, maxAccelValue * 1.1);

        // Create path - Y centered at 0
        const centerY = padding.top + graphHeight / 2;
        const points = accelPoints.map(point => {
            const x = padding.left + ((point.time - minTime) / TIME_WINDOW_MS) * graphWidth;
            const y = centerY - (point.acceleration / max) * (graphHeight / 2);
            return { x, y, accel: point.acceleration };
        });

        const linePath = points.map((p, i) =>
            `${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`
        ).join(' ');

        return {
            accelerationData: accelPoints,
            maxAccel: max,
            pathData: linePath,
            zeroLineY: centerY,
            stats: { avg: avgAccel, max: maxAccelValue, jitter: jitterPercent }
        };
    }, [data, graphWidth, graphHeight, height]);

    // Current acceleration
    const currentAccel = accelerationData.length > 0 ? accelerationData[accelerationData.length - 1].acceleration : 0;

    return (
        <div className="bg-gray-900/50 rounded-lg border border-gray-700/50 p-2 overflow-hidden">
            <div className="flex items-center justify-between mb-1">
                <span className="text-[10px] text-gray-400">{label}</span>
                <div className="flex items-center gap-2">
                    <span className={`text-[9px] px-1.5 py-0.5 rounded ${stats.jitter > 20 ? 'bg-red-500/30 text-red-300' : stats.jitter > 5 ? 'bg-yellow-500/30 text-yellow-300' : 'bg-green-500/30 text-green-300'}`}>
                        Jitter: {stats.jitter.toFixed(0)}%
                    </span>
                    <span className={`text-xs font-mono font-bold ${Math.abs(currentAccel) > 5000 ? 'text-red-400' : 'text-white'}`}>
                        {currentAccel > 0 ? '+' : ''}{(currentAccel / 1000).toFixed(1)}k
                    </span>
                </div>
            </div>
            <svg width="100%" viewBox={`0 0 ${width} ${height}`} className="overflow-hidden">
                {/* Zero line */}
                <line
                    x1={padding.left}
                    y1={zeroLineY}
                    x2={width - padding.right}
                    y2={zeroLineY}
                    stroke="#6b7280"
                    strokeWidth={1}
                    opacity={0.8}
                />

                {/* Threshold lines */}
                {[5000, -5000].map((v) => {
                    const y = zeroLineY - (v / maxAccel) * (graphHeight / 2);
                    return (
                        <line
                            key={v}
                            x1={padding.left}
                            y1={y}
                            x2={width - padding.right}
                            y2={y}
                            stroke="#ef4444"
                            strokeWidth={0.5}
                            strokeDasharray="2,2"
                            opacity={0.5}
                        />
                    );
                })}

                {/* Y-axis labels */}
                <text x={padding.left - 3} y={zeroLineY} textAnchor="end" dominantBaseline="middle" className="text-[7px] fill-gray-500">0</text>
                <text x={padding.left - 3} y={padding.top + 5} textAnchor="end" dominantBaseline="middle" className="text-[7px] fill-gray-500">+{(maxAccel/1000).toFixed(0)}k</text>
                <text x={padding.left - 3} y={padding.top + graphHeight - 5} textAnchor="end" dominantBaseline="middle" className="text-[7px] fill-gray-500">-{(maxAccel/1000).toFixed(0)}k</text>

                {/* Positive/Negative gradient fills */}
                <defs>
                    <linearGradient id={`${gradientId}-pos`} x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="#22c55e" stopOpacity="0.3" />
                        <stop offset="100%" stopColor="#22c55e" stopOpacity="0" />
                    </linearGradient>
                    <linearGradient id={`${gradientId}-neg`} x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="#ef4444" stopOpacity="0" />
                        <stop offset="100%" stopColor="#ef4444" stopOpacity="0.3" />
                    </linearGradient>
                </defs>

                {/* Line */}
                {pathData && (
                    <path
                        d={pathData}
                        fill="none"
                        stroke={color}
                        strokeWidth={1.5}
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    />
                )}

                {/* Current value dot */}
                {accelerationData.length > 0 && (
                    <circle
                        cx={padding.left + graphWidth}
                        cy={zeroLineY - (currentAccel / maxAccel) * (graphHeight / 2)}
                        r={3}
                        fill={Math.abs(currentAccel) > 5000 ? '#ef4444' : color}
                    />
                )}

                {/* Time axis labels */}
                <text x={padding.left} y={height - 2} textAnchor="start" className="text-[7px] fill-gray-600">-5s</text>
                <text x={padding.left + graphWidth} y={height - 2} textAnchor="end" className="text-[7px] fill-gray-600">now</text>
            </svg>
        </div>
    );
}

// ============================================================================
// KEYFRAME TIMELINE VISUALIZATION
// ============================================================================

function KeyframeTimeline({
    ballTimeline = [],
    playerTimelines = {},
    currentTime = 0,
    windowSize = 2, // seconds to show
    onWindowSizeChange = null,
    // Playback controls
    isPlaying = true,
    playbackSpeed = 1.0,
    onPlayPause = null,
    onPlaybackSpeedChange = null,
    onSeek = null
}) {
    const labelWidth = 70; // Fixed width for labels
    const timelineWidth = 240; // Width for the actual timeline
    const width = labelWidth + timelineWidth + 10;
    const height = 120;
    const padding = { top: 20, right: 5, bottom: 25, left: 5 };
    const rowHeight = 14;

    // Selected keyframe state
    const [selectedFrame, setSelectedFrame] = useState(null);
    // Filter state: Set of visible entity names ('ball' + player names)
    const [visibleEntities, setVisibleEntities] = useState(null); // null = all visible
    const [showFilterMenu, setShowFilterMenu] = useState(false);
    // Drag state for scrubbing
    const [isDragging, setIsDragging] = useState(false);
    const dragStartRef = useRef({ x: 0, time: 0 });
    const svgRef = useRef(null);
    // Tracked metrics state: array of { entity: 'ball' | playerName, metric: 'speed' | 'velocity.x' | etc, color: string }
    const [trackedMetrics, setTrackedMetrics] = useState([]);

    // Calculate total duration from ball timeline
    const duration = ballTimeline.length > 0 ? ballTimeline[ballTimeline.length - 1].time : 0;

    // Calculate time window centered on current time
    const halfWindow = windowSize / 2;
    const startTime = currentTime - halfWindow;
    const endTime = currentTime + halfWindow;

    // Handle drag to scrub timeline
    const handleMouseDown = (e) => {
        if (!onSeek) return;
        // Only start drag if clicking on timeline area (not on labels)
        const rect = svgRef.current?.getBoundingClientRect();
        if (!rect) return;
        const x = e.clientX - rect.left;
        if (x < labelWidth) return; // Don't drag from label area

        setIsDragging(true);
        dragStartRef.current = { x: e.clientX, time: currentTime };
        e.preventDefault();
    };

    useEffect(() => {
        if (!isDragging) return;

        const handleMouseMove = (e) => {
            const deltaX = e.clientX - dragStartRef.current.x;
            // Convert pixel delta to time delta (negative because dragging right = going back in time)
            const timeDelta = -(deltaX / timelineWidth) * windowSize;
            let newTime = dragStartRef.current.time + timeDelta;
            // Clamp to valid range
            newTime = Math.max(0, Math.min(duration, newTime));
            onSeek?.(newTime);
        };

        const handleMouseUp = () => {
            setIsDragging(false);
        };

        window.addEventListener('mousemove', handleMouseMove);
        window.addEventListener('mouseup', handleMouseUp);
        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);
        };
    }, [isDragging, windowSize, timelineWidth, duration, onSeek]);

    // Helper to convert time to x position (relative to timeline area)
    const timeToX = (time) => {
        return labelWidth + ((time - startTime) / windowSize) * timelineWidth;
    };

    // Generate grid lines for seconds
    const gridLines = useMemo(() => {
        const lines = [];
        const firstSecond = Math.ceil(startTime);
        for (let t = firstSecond; t <= Math.floor(endTime); t++) {
            if (t >= startTime && t <= endTime) {
                lines.push(t);
            }
        }
        return lines;
    }, [startTime, endTime]);

    // All entity names for filtering
    const playerNames = Object.keys(playerTimelines);
    const allEntities = ['ball', ...playerNames];

    // Filtered entities to display
    const displayedEntities = visibleEntities ? allEntities.filter(e => visibleEntities.has(e)) : allEntities;
    const showBall = !visibleEntities || visibleEntities.has('ball');
    const displayedPlayers = displayedEntities.filter(e => e !== 'ball');

    // Filter keyframes within visible window
    const visibleBallFrames = useMemo(() => {
        if (!showBall) return [];
        return ballTimeline.filter(k => k.time >= startTime && k.time <= endTime);
    }, [ballTimeline, startTime, endTime, showBall]);

    const visiblePlayerFrames = useMemo(() => {
        const result = {};
        displayedPlayers.forEach(name => {
            const timeline = playerTimelines[name] || [];
            result[name] = timeline.filter(k => k.time >= startTime && k.time <= endTime);
        });
        return result;
    }, [playerTimelines, startTime, endTime, displayedPlayers]);

    const totalRows = (showBall ? 1 : 0) + displayedPlayers.length;
    const contentHeight = Math.max(totalRows * rowHeight + 10, 30);

    // Colors for players
    const playerColors = ['#f97316', '#22c55e', '#ec4899', '#06b6d4', '#eab308', '#8b5cf6'];
    const getPlayerColor = (name) => {
        const idx = playerNames.indexOf(name);
        return playerColors[idx % playerColors.length];
    };

    // Handle keyframe click
    const handleFrameClick = (type, frame, name = null, color = '#3b82f6') => {
        setSelectedFrame({ type, name, frame, color });
    };

    // Toggle entity visibility
    const toggleEntity = (entity) => {
        setVisibleEntities(prev => {
            if (prev === null) {
                // Currently showing all, create set with all except clicked
                const newSet = new Set(allEntities);
                newSet.delete(entity);
                return newSet.size === 0 ? null : newSet;
            } else {
                const newSet = new Set(prev);
                if (newSet.has(entity)) {
                    newSet.delete(entity);
                } else {
                    newSet.add(entity);
                }
                // If all are visible again, return null
                if (newSet.size === allEntities.length) return null;
                // If none visible, show all
                if (newSet.size === 0) return null;
                return newSet;
            }
        });
    };

    // Show all entities
    const showAll = () => setVisibleEntities(null);

    // Format value for display
    const formatValue = (val) => {
        if (val === null || val === undefined) return 'null';
        if (typeof val === 'number') return val.toFixed(2);
        if (typeof val === 'boolean') return val ? 'true' : 'false';
        if (typeof val === 'object') return JSON.stringify(val);
        return String(val);
    };

    // Metric colors for tracked graphs
    const metricColors = ['#f97316', '#22c55e', '#3b82f6', '#ec4899', '#eab308', '#06b6d4', '#8b5cf6', '#ef4444'];

    // Get value from a keyframe for a given metric path
    // For 'movement' metric, we need the timeline and frame index
    const getMetricValue = (frame, metricPath, timeline = null, frameIndex = -1) => {
        if (metricPath === 'speed') {
            if (!frame.velocity) return null;
            return Math.sqrt(frame.velocity.x ** 2 + frame.velocity.y ** 2 + frame.velocity.z ** 2);
        }
        if (metricPath === 'movement') {
            // Calculate position delta from previous frame
            if (!timeline || frameIndex <= 0 || !frame.position) return null;
            const prevFrame = timeline[frameIndex - 1];
            if (!prevFrame?.position) return null;
            const dx = frame.position.x - prevFrame.position.x;
            const dy = frame.position.y - prevFrame.position.y;
            const dz = frame.position.z - prevFrame.position.z;
            return Math.sqrt(dx * dx + dy * dy + dz * dz);
        }
        if (metricPath === 'movement.x') {
            if (!timeline || frameIndex <= 0 || !frame.position) return null;
            const prevFrame = timeline[frameIndex - 1];
            if (!prevFrame?.position) return null;
            return Math.abs(frame.position.x - prevFrame.position.x);
        }
        if (metricPath === 'movement.y') {
            if (!timeline || frameIndex <= 0 || !frame.position) return null;
            const prevFrame = timeline[frameIndex - 1];
            if (!prevFrame?.position) return null;
            return Math.abs(frame.position.y - prevFrame.position.y);
        }
        if (metricPath === 'movement.z') {
            if (!timeline || frameIndex <= 0 || !frame.position) return null;
            const prevFrame = timeline[frameIndex - 1];
            if (!prevFrame?.position) return null;
            return Math.abs(frame.position.z - prevFrame.position.z);
        }
        const parts = metricPath.split('.');
        let val = frame;
        for (const part of parts) {
            if (val === null || val === undefined) return null;
            val = val[part];
        }
        return typeof val === 'number' ? val : null;
    };

    // Add a metric to track
    const addTrackedMetric = (entity, metric) => {
        // Check if already tracking this exact metric
        const exists = trackedMetrics.some(m => m.entity === entity && m.metric === metric);
        if (exists) return;

        const color = metricColors[trackedMetrics.length % metricColors.length];
        setTrackedMetrics(prev => [...prev, {
            entity,
            metric,
            color,
            id: `${entity}-${metric}-${Date.now()}`,
            scaleMode: 'fixed' // 'fixed' = full timeline, 'auto' = visible window only
        }]);
    };

    // Remove a tracked metric
    const removeTrackedMetric = (id) => {
        setTrackedMetrics(prev => prev.filter(m => m.id !== id));
    };

    // Toggle scale mode for a tracked metric
    const toggleScaleMode = (id) => {
        setTrackedMetrics(prev => prev.map(m =>
            m.id === id
                ? { ...m, scaleMode: m.scaleMode === 'fixed' ? 'auto' : 'fixed' }
                : m
        ));
    };

    // Check if a metric is being tracked
    const isMetricTracked = (entity, metric) => {
        return trackedMetrics.some(m => m.entity === entity && m.metric === metric);
    };

    // Get timeline for entity
    const getEntityTimeline = (entity) => {
        if (entity === 'ball') return ballTimeline;
        return playerTimelines[entity] || [];
    };

    // Render add button for a metric
    const renderAddButton = (metric, label = null) => {
        const entity = selectedFrame.type === 'ball' ? 'ball' : selectedFrame.name;
        const tracked = isMetricTracked(entity, metric);
        return (
            <button
                onClick={() => tracked ? null : addTrackedMetric(entity, metric)}
                className={`ml-1 w-4 h-4 text-[8px] rounded flex items-center justify-center transition-all ${
                    tracked
                        ? 'bg-green-600/50 text-green-300 cursor-default'
                        : 'bg-gray-700 hover:bg-purple-600 text-gray-400 hover:text-white'
                }`}
                title={tracked ? 'Already tracking' : `Track ${label || metric}`}
            >
                {tracked ? '✓' : '+'}
            </button>
        );
    };

    return (
        <div className="bg-gray-900/50 rounded-lg border border-gray-700/50 p-2">
            {/* Header */}
            <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                    <span className="text-[10px] text-gray-400 font-medium">Keyframe Timeline</span>
                    {/* Filter button */}
                    <div className="relative">
                        <button
                            onClick={() => setShowFilterMenu(!showFilterMenu)}
                            className={`text-[9px] px-1.5 py-0.5 rounded ${
                                visibleEntities ? 'bg-purple-600 text-white' : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                            }`}
                        >
                            Filter {visibleEntities ? `(${visibleEntities.size})` : ''}
                        </button>
                        {/* Filter dropdown */}
                        {showFilterMenu && (
                            <div className="absolute top-full left-0 mt-1 bg-gray-800 border border-gray-600 rounded shadow-lg z-50 min-w-[150px]">
                                <div className="p-1 border-b border-gray-700">
                                    <button
                                        onClick={showAll}
                                        className="text-[9px] text-cyan-400 hover:text-cyan-300 w-full text-left px-2 py-1"
                                    >
                                        Show All
                                    </button>
                                </div>
                                <div className="max-h-[150px] overflow-y-auto">
                                    {/* Ball */}
                                    <label className="flex items-center gap-2 px-2 py-1 hover:bg-gray-700 cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={!visibleEntities || visibleEntities.has('ball')}
                                            onChange={() => toggleEntity('ball')}
                                            className="w-3 h-3 rounded"
                                        />
                                        <span className="w-2 h-2 bg-blue-500 rounded-sm"></span>
                                        <span className="text-[9px] text-gray-300">Ball</span>
                                    </label>
                                    {/* Players */}
                                    {playerNames.map((name, idx) => (
                                        <label key={name} className="flex items-center gap-2 px-2 py-1 hover:bg-gray-700 cursor-pointer">
                                            <input
                                                type="checkbox"
                                                checked={!visibleEntities || visibleEntities.has(name)}
                                                onChange={() => toggleEntity(name)}
                                                className="w-3 h-3 rounded"
                                            />
                                            <span
                                                className="w-2 h-2 rounded-sm"
                                                style={{ backgroundColor: playerColors[idx % playerColors.length] }}
                                            ></span>
                                            <span className="text-[9px] text-gray-300 truncate max-w-[100px]">{name}</span>
                                        </label>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    <span className="text-[9px] text-gray-500">Window:</span>
                    <input
                        type="range"
                        min="0.25"
                        max="5"
                        step="0.25"
                        value={windowSize}
                        onChange={(e) => onWindowSizeChange?.(parseFloat(e.target.value))}
                        className="w-24 h-1 accent-purple-500"
                    />
                    <span className="text-[9px] text-gray-400 w-8">{windowSize}s</span>
                </div>
            </div>

            {/* Close filter menu when clicking outside */}
            {showFilterMenu && (
                <div
                    className="fixed inset-0 z-40"
                    onClick={() => setShowFilterMenu(false)}
                />
            )}

            <svg
                ref={svgRef}
                width="100%"
                viewBox={`0 0 ${width} ${Math.max(height, contentHeight + padding.top + padding.bottom)}`}
                onMouseDown={handleMouseDown}
                className={isDragging ? 'cursor-grabbing' : 'cursor-grab'}
                style={{ userSelect: 'none' }}
            >
                {/* Timeline background */}
                <rect
                    x={labelWidth}
                    y={padding.top}
                    width={timelineWidth}
                    height={contentHeight}
                    fill={isDragging ? '#1f2937' : '#111827'}
                    rx={2}
                />

                {/* Grid lines for seconds */}
                {gridLines.map((t) => {
                    const x = timeToX(t);
                    return (
                        <g key={t}>
                            <line
                                x1={x}
                                y1={padding.top}
                                x2={x}
                                y2={padding.top + contentHeight}
                                stroke="#374151"
                                strokeWidth={0.5}
                            />
                            <text
                                x={x}
                                y={padding.top + contentHeight + 12}
                                textAnchor="middle"
                                className="text-[8px] fill-gray-500"
                            >
                                {t.toFixed(1)}s
                            </text>
                        </g>
                    );
                })}

                {/* Current time indicator (center line) */}
                <line
                    x1={timeToX(currentTime)}
                    y1={padding.top - 5}
                    x2={timeToX(currentTime)}
                    y2={padding.top + contentHeight + 5}
                    stroke="#a855f7"
                    strokeWidth={2}
                />
                <polygon
                    points={`${timeToX(currentTime)},${padding.top - 8} ${timeToX(currentTime) - 4},${padding.top - 2} ${timeToX(currentTime) + 4},${padding.top - 2}`}
                    fill="#a855f7"
                />

                {/* Ball keyframes row */}
                {showBall && (
                    <g transform={`translate(0, ${padding.top + 5})`}>
                        <text x={padding.left} y={rowHeight / 2 + 3} className="text-[8px] fill-gray-400">Ball</text>
                        {visibleBallFrames.map((frame, i) => {
                            const isSelected = selectedFrame?.type === 'ball' && selectedFrame?.frame?.time === frame.time;
                            const x = timeToX(frame.time);
                            // Only render if within timeline area
                            if (x < labelWidth || x > labelWidth + timelineWidth) return null;
                            return (
                                <g key={i}>
                                    <rect
                                        x={x - 4}
                                        y={-2}
                                        width={8}
                                        height={rowHeight + 2}
                                        fill="transparent"
                                        className="cursor-pointer"
                                        onClick={() => handleFrameClick('ball', frame, null, '#3b82f6')}
                                    />
                                    <line
                                        x1={x}
                                        y1={0}
                                        x2={x}
                                        y2={rowHeight - 2}
                                        stroke={isSelected ? '#ffffff' : '#3b82f6'}
                                        strokeWidth={isSelected ? 3 : 1.5}
                                        opacity={isSelected ? 1 : 0.8}
                                    className="pointer-events-none"
                                />
                            </g>
                        );
                    })}
                </g>
                )}

                {/* Player keyframes rows */}
                {displayedPlayers.map((name, rowIdx) => {
                    const frames = visiblePlayerFrames[name] || [];
                    const yOffset = padding.top + 5 + (showBall ? 1 : 0) * rowHeight + rowIdx * rowHeight;
                    const color = getPlayerColor(name);

                    return (
                        <g key={name} transform={`translate(0, ${yOffset})`}>
                            <text x={padding.left} y={rowHeight / 2 + 2} className="text-[7px] fill-gray-400">
                                {name.length > 8 ? name.substring(0, 8) + '…' : name}
                            </text>
                            {frames.map((frame, i) => {
                                const isSelected = selectedFrame?.type === 'player' &&
                                                   selectedFrame?.name === name &&
                                                   selectedFrame?.frame?.time === frame.time;
                                const x = timeToX(frame.time);
                                // Only render if within timeline area
                                if (x < labelWidth || x > labelWidth + timelineWidth) return null;
                                return (
                                    <g key={i}>
                                        {/* Clickable area */}
                                        <rect
                                            x={x - 4}
                                            y={-2}
                                            width={8}
                                            height={rowHeight + 2}
                                            fill="transparent"
                                            className="cursor-pointer"
                                            onClick={() => handleFrameClick('player', frame, name, color)}
                                        />
                                        <line
                                            x1={x}
                                            y1={0}
                                            x2={x}
                                            y2={rowHeight - 2}
                                            stroke={isSelected ? '#ffffff' : color}
                                            strokeWidth={isSelected ? 3 : 1.5}
                                            opacity={isSelected ? 1 : 0.8}
                                            className="pointer-events-none"
                                        />
                                    </g>
                                );
                            })}
                        </g>
                    );
                })}

                {/* Stats */}
                <text x={width - 5} y={12} textAnchor="end" className="text-[8px] fill-gray-500">
                    Ball: {visibleBallFrames.length} frames
                </text>
            </svg>

            {/* Tracked Metrics Graphs - directly under timeline for correlation */}
            {trackedMetrics.length > 0 && (
                <div className="space-y-0.5">
                    {trackedMetrics.map((tracked) => {
                        const timeline = getEntityTimeline(tracked.entity);
                        // Keep track of original indices for movement calculation
                        const visibleFramesWithIndex = timeline
                            .map((k, idx) => ({ frame: k, idx }))
                            .filter(({ frame }) => frame.time >= startTime && frame.time <= endTime);

                        // Calculate min/max for scaling based on scaleMode
                        let minVal, maxVal;
                        const visibleValues = visibleFramesWithIndex
                            .map(({ frame, idx }) => getMetricValue(frame, tracked.metric, timeline, idx))
                            .filter(v => v !== null);
                        const allValues = timeline
                            .map((f, idx) => getMetricValue(f, tracked.metric, timeline, idx))
                            .filter(v => v !== null);

                        if (tracked.scaleMode === 'fixed') {
                            // Hybrid mode: visible window min/max + margin based on global variance
                            const visibleMin = visibleValues.length > 0 ? Math.min(...visibleValues) : 0;
                            const visibleMax = visibleValues.length > 0 ? Math.max(...visibleValues) : 1;
                            const globalMin = allValues.length > 0 ? Math.min(...allValues) : 0;
                            const globalMax = allValues.length > 0 ? Math.max(...allValues) : 1;
                            const globalRange = globalMax - globalMin || 1;

                            // Add 15% margin based on global range to prevent constant rescaling
                            const margin = globalRange * 0.15;
                            minVal = visibleMin - margin;
                            maxVal = visibleMax + margin;

                            // But don't exceed global bounds
                            minVal = Math.max(minVal, globalMin - margin * 0.5);
                            maxVal = Math.min(maxVal, globalMax + margin * 0.5);
                        } else {
                            // Auto mode: pure visible window, rescales constantly
                            minVal = visibleValues.length > 0 ? Math.min(...visibleValues) : 0;
                            maxVal = visibleValues.length > 0 ? Math.max(...visibleValues) : 1;
                        }
                        const range = maxVal - minVal || 1;

                        // Graph dimensions - compact
                        const graphHeight = 40;
                        const graphPadding = 4;

                        // Build path for the curve
                        const pathPoints = visibleFramesWithIndex
                            .map(({ frame, idx }) => {
                                const val = getMetricValue(frame, tracked.metric, timeline, idx);
                                if (val === null) return null;
                                const x = timeToX(frame.time);
                                const y = graphPadding + (1 - (val - minVal) / range) * (graphHeight - 2 * graphPadding);
                                return { x, y, time: frame.time };
                            })
                            .filter(p => p !== null && p.x >= labelWidth && p.x <= labelWidth + timelineWidth);

                        const pathD = pathPoints.length > 1
                            ? `M ${pathPoints.map(p => `${p.x},${p.y}`).join(' L ')}`
                            : '';

                        // Current value at playhead
                        const currentIdx = timeline.findIndex(f => f.time >= currentTime);
                        const actualIdx = currentIdx > 0 ? currentIdx - 1 : 0;
                        const currentFrame = timeline[actualIdx];
                        const currentValue = currentFrame ? getMetricValue(currentFrame, tracked.metric, timeline, actualIdx) : null;

                        return (
                            <div key={tracked.id} className="relative">
                                {/* Graph SVG - aligned with timeline above */}
                                <svg
                                    width="100%"
                                    viewBox={`0 0 ${width} ${graphHeight}`}
                                    className="overflow-visible"
                                >
                                    {/* Background */}
                                    <rect
                                        x={labelWidth}
                                        y={0}
                                        width={timelineWidth}
                                        height={graphHeight}
                                        fill="#0f172a"
                                    />

                                    {/* Grid lines - aligned with timeline */}
                                    {gridLines.map(t => {
                                        const x = timeToX(t);
                                        return (
                                            <line
                                                key={t}
                                                x1={x}
                                                y1={0}
                                                x2={x}
                                                y2={graphHeight}
                                                stroke="#1e293b"
                                                strokeWidth={0.5}
                                            />
                                        );
                                    })}

                                    {/* Keyframe markers - full height vertical bars */}
                                    {visibleFramesWithIndex.map(({ frame }, i) => {
                                        const x = timeToX(frame.time);
                                        if (x < labelWidth || x > labelWidth + timelineWidth) return null;
                                        return (
                                            <line
                                                key={i}
                                                x1={x}
                                                y1={0}
                                                x2={x}
                                                y2={graphHeight}
                                                stroke={tracked.color}
                                                strokeWidth={1}
                                                opacity={0.3}
                                            />
                                        );
                                    })}

                                    {/* Value curve */}
                                    {pathD && (
                                        <path
                                            d={pathD}
                                            fill="none"
                                            stroke={tracked.color}
                                            strokeWidth={1.5}
                                            strokeLinecap="round"
                                            strokeLinejoin="round"
                                        />
                                    )}

                                    {/* Current time indicator - aligned with timeline */}
                                    <line
                                        x1={timeToX(currentTime)}
                                        y1={0}
                                        x2={timeToX(currentTime)}
                                        y2={graphHeight}
                                        stroke="#a855f7"
                                        strokeWidth={2}
                                    />

                                    {/* Label on the left */}
                                    <text x={labelWidth - 3} y={graphHeight / 2 + 3} textAnchor="end" className="text-[7px] fill-gray-400">
                                        {tracked.entity === 'ball' ? 'Ball' : tracked.entity.substring(0, 6)}
                                    </text>
                                    <text x={labelWidth - 3} y={graphHeight / 2 + 11} textAnchor="end" className="text-[6px] fill-gray-500">
                                        {tracked.metric}
                                    </text>

                                    {/* Current value */}
                                    {currentValue !== null && (
                                        <text x={labelWidth + timelineWidth + 3} y={graphHeight / 2 + 3} className="text-[8px] font-mono" fill={tracked.color}>
                                            {currentValue.toFixed(0)}
                                        </text>
                                    )}

                                    {/* Scale mode toggle button */}
                                    <g
                                        className="cursor-pointer"
                                        onClick={() => toggleScaleMode(tracked.id)}
                                    >
                                        <rect
                                            x={width - 38}
                                            y={2}
                                            width={22}
                                            height={12}
                                            rx={2}
                                            fill={tracked.scaleMode === 'fixed' ? '#065f46' : '#1e40af'}
                                            className="hover:opacity-80"
                                        />
                                        <text
                                            x={width - 27}
                                            y={11}
                                            textAnchor="middle"
                                            className="text-[7px] fill-white pointer-events-none font-medium"
                                        >
                                            {tracked.scaleMode === 'fixed' ? 'FIX' : 'AUTO'}
                                        </text>
                                    </g>

                                    {/* Remove button */}
                                    <g
                                        className="cursor-pointer"
                                        onClick={() => removeTrackedMetric(tracked.id)}
                                    >
                                        <circle cx={width - 8} cy={8} r={6} fill="#374151" className="hover:fill-red-900" />
                                        <text x={width - 8} y={11} textAnchor="middle" className="text-[8px] fill-gray-400 hover:fill-red-400 pointer-events-none">✕</text>
                                    </g>
                                </svg>
                            </div>
                        );
                    })}
                    {/* Clear all button */}
                    <div className="flex justify-end">
                        <button
                            onClick={() => setTrackedMetrics([])}
                            className="text-[8px] text-gray-500 hover:text-red-400 transition-colors px-1"
                        >
                            Clear all graphs
                        </button>
                    </div>
                </div>
            )}

            {/* Selected Frame Details */}
            {selectedFrame && (
                <div className="mt-2 p-2 rounded bg-gray-800/80 border border-gray-600/50">
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                            <span
                                className="w-3 h-3 rounded-sm"
                                style={{ backgroundColor: selectedFrame.color }}
                            />
                            <span className="text-[10px] font-medium text-white">
                                {selectedFrame.type === 'ball' ? 'Ball' : selectedFrame.name}
                            </span>
                            <span className="text-[9px] text-gray-400">
                                @ {selectedFrame.frame.time?.toFixed(3)}s
                            </span>
                        </div>
                        <button
                            onClick={() => setSelectedFrame(null)}
                            className="text-gray-500 hover:text-white text-xs"
                        >
                            ✕
                        </button>
                    </div>

                    <div className="space-y-1 text-[9px] font-mono">
                        {/* Position with trackable axes */}
                        {selectedFrame.frame.position && (
                            <div className="flex items-center justify-between">
                                <span className="text-gray-400">position:</span>
                                <div className="flex items-center">
                                    <span className="text-green-400">
                                        ({formatValue(selectedFrame.frame.position.x)}, {formatValue(selectedFrame.frame.position.y)}, {formatValue(selectedFrame.frame.position.z)})
                                    </span>
                                    <div className="flex ml-1">
                                        {renderAddButton('position.x', 'pos.x')}
                                        {renderAddButton('position.y', 'pos.y')}
                                        {renderAddButton('position.z', 'pos.z')}
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Velocity with trackable axes */}
                        {selectedFrame.frame.velocity && (
                            <div className="flex items-center justify-between">
                                <span className="text-gray-400">velocity:</span>
                                <div className="flex items-center">
                                    <span className="text-cyan-400">
                                        ({formatValue(selectedFrame.frame.velocity.x)}, {formatValue(selectedFrame.frame.velocity.y)}, {formatValue(selectedFrame.frame.velocity.z)})
                                    </span>
                                    <div className="flex ml-1">
                                        {renderAddButton('velocity.x', 'vel.x')}
                                        {renderAddButton('velocity.y', 'vel.y')}
                                        {renderAddButton('velocity.z', 'vel.z')}
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Speed (calculated from velocity) */}
                        {selectedFrame.frame.velocity && (
                            <div className="flex items-center justify-between">
                                <span className="text-gray-400">speed:</span>
                                <div className="flex items-center">
                                    <span className="text-yellow-400">
                                        {Math.sqrt(
                                            selectedFrame.frame.velocity.x ** 2 +
                                            selectedFrame.frame.velocity.y ** 2 +
                                            selectedFrame.frame.velocity.z ** 2
                                        ).toFixed(1)} uu/s
                                    </span>
                                    {renderAddButton('speed', 'speed')}
                                </div>
                            </div>
                        )}

                        {/* Movement (position delta between frames) */}
                        {selectedFrame.frame.position && (
                            <div className="flex items-center justify-between">
                                <span className="text-gray-400">movement:</span>
                                <div className="flex items-center">
                                    <span className="text-pink-400 text-[8px]">
                                        (position delta)
                                    </span>
                                    {renderAddButton('movement', 'move')}
                                </div>
                            </div>
                        )}

                        {/* Rotation */}
                        {selectedFrame.frame.rotation && (
                            <div className="flex items-center justify-between">
                                <span className="text-gray-400">rotation:</span>
                                <span className="text-purple-400">
                                    ({formatValue(selectedFrame.frame.rotation.x)}, {formatValue(selectedFrame.frame.rotation.y)}, {formatValue(selectedFrame.frame.rotation.z)}, {formatValue(selectedFrame.frame.rotation.w)})
                                </span>
                            </div>
                        )}

                        {/* Angular Velocity */}
                        {selectedFrame.frame.angularVelocity && (
                            <div className="flex items-center justify-between">
                                <span className="text-gray-400">angularVel:</span>
                                <div className="flex items-center">
                                    <span className="text-orange-400">
                                        ({formatValue(selectedFrame.frame.angularVelocity.x)}, {formatValue(selectedFrame.frame.angularVelocity.y)}, {formatValue(selectedFrame.frame.angularVelocity.z)})
                                    </span>
                                    <div className="flex ml-1">
                                        {renderAddButton('angularVelocity.x', 'ang.x')}
                                        {renderAddButton('angularVelocity.y', 'ang.y')}
                                        {renderAddButton('angularVelocity.z', 'ang.z')}
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Boost (for players) */}
                        {selectedFrame.frame.boost !== undefined && (
                            <div className="flex items-center justify-between">
                                <span className="text-gray-400">boost:</span>
                                <div className="flex items-center">
                                    <span className="text-amber-400">{formatValue(selectedFrame.frame.boost)}</span>
                                    {renderAddButton('boost', 'boost')}
                                </div>
                            </div>
                        )}

                        {/* Sleeping */}
                        {selectedFrame.frame.sleeping !== undefined && (
                            <div className="flex justify-between">
                                <span className="text-gray-400">sleeping:</span>
                                <span className={selectedFrame.frame.sleeping ? 'text-red-400' : 'text-green-400'}>
                                    {selectedFrame.frame.sleeping ? 'true' : 'false'}
                                </span>
                            </div>
                        )}

                        {/* Show all other properties */}
                        {Object.entries(selectedFrame.frame)
                            .filter(([key]) => !['time', 'position', 'velocity', 'rotation', 'angularVelocity', 'boost', 'sleeping'].includes(key))
                            .map(([key, value]) => (
                                <div key={key} className="flex justify-between">
                                    <span className="text-gray-400">{key}:</span>
                                    <span className="text-gray-300 truncate max-w-[180px]">{formatValue(value)}</span>
                                </div>
                            ))
                        }
                    </div>
                </div>
            )}

            {/* Legend with frame rate info */}
            <div className="flex flex-wrap gap-2 mt-2 text-[8px] text-gray-500">
                <span className="flex items-center gap-1">
                    <span className="w-2 h-2 bg-blue-500 rounded-sm"></span>
                    Ball (~{ballTimeline.length > 1 ? (1000 / ((ballTimeline[ballTimeline.length - 1]?.time - ballTimeline[0]?.time) / ballTimeline.length * 1000)).toFixed(0) : '?'} Hz)
                </span>
                {playerNames.slice(0, 2).map((name, idx) => {
                    const timeline = playerTimelines[name] || [];
                    const hz = timeline.length > 1 ? (1000 / ((timeline[timeline.length - 1]?.time - timeline[0]?.time) / timeline.length * 1000)).toFixed(0) : '?';
                    return (
                        <span key={name} className="flex items-center gap-1">
                            <span className="w-2 h-2 rounded-sm" style={{ backgroundColor: playerColors[idx] }}></span>
                            {name.length > 6 ? name.substring(0, 6) + '…' : name} (~{hz} Hz)
                        </span>
                    );
                })}
            </div>

            {!selectedFrame && !isDragging && (
                <div className="text-[8px] text-gray-600 mt-1 text-center">
                    Drag timeline to scrub • Click keyframe for details
                </div>
            )}

            {/* Playback Controls */}
            <div className="mt-3 pt-3 border-t border-gray-700/50">
                <div className="flex items-center gap-3">
                    {/* Play/Pause button */}
                    <button
                        onClick={() => onPlayPause?.()}
                        className={`flex items-center justify-center w-8 h-8 rounded-lg transition-all ${
                            isPlaying
                                ? 'bg-amber-600 hover:bg-amber-500 text-white'
                                : 'bg-green-600 hover:bg-green-500 text-white'
                        }`}
                        title={isPlaying ? 'Pause' : 'Play'}
                    >
                        {isPlaying ? (
                            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                                <rect x="6" y="4" width="4" height="16" />
                                <rect x="14" y="4" width="4" height="16" />
                            </svg>
                        ) : (
                            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M8 5v14l11-7z" />
                            </svg>
                        )}
                    </button>

                    {/* Speed control */}
                    <div className="flex-1 flex items-center gap-2">
                        <span className="text-[9px] text-gray-500 w-10">Speed:</span>
                        <input
                            type="range"
                            min="0.1"
                            max="2"
                            step="0.1"
                            value={playbackSpeed}
                            onChange={(e) => onPlaybackSpeedChange?.(parseFloat(e.target.value))}
                            className="flex-1 h-1.5 accent-purple-500 cursor-pointer"
                        />
                        <span className={`text-[10px] font-mono w-8 text-right ${
                            playbackSpeed < 0.5 ? 'text-cyan-400' :
                            playbackSpeed > 1 ? 'text-orange-400' : 'text-gray-300'
                        }`}>
                            {playbackSpeed.toFixed(1)}x
                        </span>
                    </div>
                </div>

                {/* Speed presets */}
                <div className="flex gap-1 mt-2">
                    {[0.1, 0.25, 0.5, 1, 1.5, 2].map(speed => (
                        <button
                            key={speed}
                            onClick={() => onPlaybackSpeedChange?.(speed)}
                            className={`flex-1 px-1 py-0.5 text-[9px] rounded transition-all ${
                                Math.abs(playbackSpeed - speed) < 0.05
                                    ? 'bg-purple-600 text-white'
                                    : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white'
                            }`}
                        >
                            {speed}x
                        </button>
                    ))}
                </div>

                {/* Current time display */}
                <div className="flex justify-between items-center mt-2 text-[10px]">
                    <span className="text-gray-500">
                        <span className="font-mono text-purple-400">{currentTime.toFixed(2)}s</span>
                        <span className="text-gray-600"> / </span>
                        <span className="font-mono text-gray-400">{duration.toFixed(2)}s</span>
                    </span>
                    <span className="text-gray-600">
                        {duration > 0 ? ((currentTime / duration) * 100).toFixed(1) : 0}%
                    </span>
                </div>
            </div>
        </div>
    );
}

// ============================================================================
// MAIN DEBUG PANEL COMPONENT
// ============================================================================

export function DebugPanel({
    actors = {},
    playerTeams = {},
    ballActorId,
    currentTime = 0,
    frameInfo = null,
    interpolationEnabled = true,
    onInterpolationToggle = null,
    playerBoosts = {},
    // New interpolation settings props
    interpolationMethod = 'hermite',
    onInterpolationMethodChange = null,
    smoothingWindowSize = 5,
    onSmoothingWindowSizeChange = null,
    // Playback state - pause graphs when not playing
    isPlaying = true,
    // Timeline data for keyframe visualization
    ballTimeline = [],
    playerTimelines = {},
    // Playback controls
    playbackSpeed = 1.0,
    onPlaybackSpeedChange = null,
    onSeek = null,
    onPlayPause = null,
    // Debug: Load local replay file
    onLoadReplayFile = null
}) {
    const [isOpen, setIsOpen] = useState(false);
    const [activeSection, setActiveSection] = useState('info'); // 'info' | 'physics' | 'interpolation' | 'timeline' | 'playback'
    const [physicsSubTab, setPhysicsSubTab] = useState('ball'); // 'ball' | car.id
    const [expandedPlayers, setExpandedPlayers] = useState(new Set());
    const [timelineWindowSize, setTimelineWindowSize] = useState(2); // seconds
    const [isLoadingFile, setIsLoadingFile] = useState(false);
    const fileInputRef = useRef(null);

    // Handle file load for local .rlrf files
    const handleFileSelect = useCallback(async (e) => {
        const file = e.target.files?.[0];
        if (!file || !onLoadReplayFile) return;

        try {
            setIsLoadingFile(true);
            const arrayBuffer = await file.arrayBuffer();
            await onLoadReplayFile(arrayBuffer, file.name);
        } catch (err) {
            console.error('[DebugPanel] Failed to load replay file:', err);
            alert('Failed to load replay file: ' + err.message);
        } finally {
            setIsLoadingFile(false);
            // Reset input so same file can be selected again
            if (fileInputRef.current) {
                fileInputRef.current.value = '';
            }
        }
    }, [onLoadReplayFile]);

    // Panel drag & resize state
    const [panelState, setPanelState] = useState(loadPanelState);
    const [isDragging, setIsDragging] = useState(false);
    const [isResizing, setIsResizing] = useState(false);
    const dragStartRef = useRef(null);
    const resizeStartRef = useRef(null);
    const panelRef = useRef(null);

    // Store actors in refs to access latest values in setInterval
    // (React props don't change reference when Three.js objects are mutated)
    const actorsRef = useRef(actors);
    const ballActorIdRef = useRef(ballActorId);
    const playerBoostsRef = useRef(playerBoosts);
    const isPlayingRef = useRef(isPlaying);

    // Keep refs updated
    useEffect(() => {
        actorsRef.current = actors;
        ballActorIdRef.current = ballActorId;
        playerBoostsRef.current = playerBoosts;
        isPlayingRef.current = isPlaying;
    }, [actors, ballActorId, playerBoosts, isPlaying]);

    // Physics tracking state
    const [physicsData, setPhysicsData] = useState({
        ball: {
            velocity: [],
            altitude: [],
            angularVelocity: [],
            position: [] // {time, x, y, z} for position delta graph
        },
        players: {} // Each player: { velocity: [], altitude: [], angularVelocity: [], boost: [], position: [] }
    });
    const lastUpdateRef = useRef(Date.now());

    // ============================================================================
    // DRAG & RESIZE HANDLERS
    // ============================================================================

    const handleDragStart = useCallback((e) => {
        e.preventDefault();
        setIsDragging(true);
        dragStartRef.current = {
            x: e.clientX,
            y: e.clientY,
            panelX: panelState.x,
            panelY: panelState.y,
        };
    }, [panelState.x, panelState.y]);

    const handleResizeStart = useCallback((e) => {
        e.preventDefault();
        e.stopPropagation();
        setIsResizing(true);
        resizeStartRef.current = {
            x: e.clientX,
            y: e.clientY,
            width: panelState.width,
            height: panelState.height,
        };
    }, [panelState.width, panelState.height]);

    useEffect(() => {
        if (!isDragging && !isResizing) return;

        const handleMouseMove = (e) => {
            if (isDragging && dragStartRef.current) {
                const dx = e.clientX - dragStartRef.current.x;
                const dy = e.clientY - dragStartRef.current.y;
                const newX = Math.max(0, Math.min(dragStartRef.current.panelX + dx, window.innerWidth - 100));
                const newY = Math.max(0, Math.min(dragStartRef.current.panelY + dy, window.innerHeight - 100));
                setPanelState(prev => ({ ...prev, x: newX, y: newY }));
            } else if (isResizing && resizeStartRef.current) {
                const dx = e.clientX - resizeStartRef.current.x;
                const dy = e.clientY - resizeStartRef.current.y;
                const newWidth = Math.max(MIN_WIDTH, Math.min(resizeStartRef.current.width + dx, MAX_WIDTH));
                const newHeight = Math.max(MIN_HEIGHT, Math.min(resizeStartRef.current.height + dy, window.innerHeight - panelState.y - 20));
                setPanelState(prev => ({ ...prev, width: newWidth, height: newHeight }));
            }
        };

        const handleMouseUp = () => {
            if (isDragging || isResizing) {
                setIsDragging(false);
                setIsResizing(false);
                dragStartRef.current = null;
                resizeStartRef.current = null;
                // Save to localStorage
                setPanelState(prev => {
                    savePanelState(prev);
                    return prev;
                });
            }
        };

        document.addEventListener('mousemove', handleMouseMove);
        document.addEventListener('mouseup', handleMouseUp);

        return () => {
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', handleMouseUp);
        };
    }, [isDragging, isResizing, panelState.y]);

    // F3 keyboard shortcut
    useEffect(() => {
        const handleKeyDown = (e) => {
            if (e.key === 'F3') {
                e.preventDefault();
                setIsOpen(prev => !prev);
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, []);

    // Get ball data
    const ball = actors[ballActorId];
    const ballPosition = ball?.position;

    // Get cars data
    const cars = useMemo(() => {
        return Object.entries(actors)
            .filter(([_, actor]) => actor.userData?.isCar && actor.userData?.playerId)
            .map(([actorId, actor]) => ({
                id: actorId,
                name: actor.userData.playerId,
                position: actor.position,
                rawPosition: actor.userData?.location,
                rotation: actor.rotation,
                team: playerTeams[actor.userData.playerId],
                sleeping: actor.userData?.sleeping || false,
                boost: actor.userData?.boost || 0
            }));
    }, [actors, playerTeams]);

    const blueCars = cars.filter(car => car.team === 0);
    const orangeCars = cars.filter(car => car.team === 1);

    // Update physics data with interval (Three.js objects are mutated, not replaced)
    useEffect(() => {
        if (!isOpen || activeSection !== 'physics') return;

        const updatePhysics = () => {
            // Don't update if playback is paused
            if (!isPlayingRef.current) return;

            const now = Date.now();
            const deltaTime = (now - lastUpdateRef.current) / 1000;
            lastUpdateRef.current = now;

            if (deltaTime <= 0 || deltaTime > 0.5) return;

            // Use refs to get latest values (props don't update when Three.js objects are mutated)
            const currentActors = actorsRef.current;
            const currentBallActorId = ballActorIdRef.current;
            const currentPlayerBoosts = playerBoostsRef.current;

            // Get current ball
            const currentBall = currentActors[currentBallActorId];
            const currentBallPos = currentBall?.position;
            const ballAngVel = currentBall?.userData?.angularVelocity;

            // Get current cars with all physics data
            const currentCars = Object.entries(currentActors)
                .filter(([_, actor]) => actor.userData?.isCar && actor.userData?.playerId)
                .map(([actorId, actor]) => ({
                    id: actorId,
                    playerName: actor.userData?.playerId,
                    position: actor.position ? { x: actor.position.x, y: actor.position.y, z: actor.position.z } : null,
                    angularVelocity: actor.userData?.angularVelocity
                }));

            setPhysicsData(prev => {
                const newData = {
                    ball: {
                        velocity: [...prev.ball.velocity],
                        altitude: [...prev.ball.altitude],
                        angularVelocity: [...prev.ball.angularVelocity],
                        position: [...prev.ball.position]
                    },
                    players: { ...prev.players }
                };

                // Ball physics
                if (currentBallPos) {
                    // Velocity - use actual velocity from replay data (userData.velocity)
                    const ballVelocity = currentBall.userData?.velocity;
                    if (ballVelocity) {
                        const velocity = Math.sqrt(
                            ballVelocity.x * ballVelocity.x +
                            ballVelocity.y * ballVelocity.y +
                            ballVelocity.z * ballVelocity.z
                        );
                        newData.ball.velocity = [...prev.ball.velocity, { time: now, velocity }].slice(-HISTORY_SIZE);
                    }

                    // Altitude (Y position in Three.js = Z in Rocket League)
                    // userData.location is already converted: RL(x,y,z) -> Three.js(x,z,y)
                    // So userData.location.y contains the height (was Z in RL)
                    const altitude = currentBall.userData?.location?.y ?? currentBallPos.y;
                    newData.ball.altitude = [...prev.ball.altitude, { time: now, value: altitude }].slice(-HISTORY_SIZE);

                    // Angular velocity (rotation speed)
                    if (ballAngVel) {
                        const angVelMagnitude = Math.sqrt(
                            ballAngVel.x * ballAngVel.x +
                            ballAngVel.y * ballAngVel.y +
                            ballAngVel.z * ballAngVel.z
                        );
                        newData.ball.angularVelocity = [...prev.ball.angularVelocity, { time: now, value: angVelMagnitude }].slice(-HISTORY_SIZE);
                    }

                    // Position (for delta graph - actual rendered position)
                    newData.ball.position = [...prev.ball.position, {
                        time: now,
                        x: currentBallPos.x,
                        y: currentBallPos.y,
                        z: currentBallPos.z
                    }].slice(-HISTORY_SIZE);
                }

                // Player physics
                currentCars.forEach(car => {
                    if (!car.position) return;

                    const prevPlayer = prev.players[car.id] || {
                        velocity: [],
                        altitude: [],
                        angularVelocity: [],
                        boost: [],
                        position: []
                    };

                    const newPlayerData = {
                        velocity: [...prevPlayer.velocity],
                        altitude: [...prevPlayer.altitude],
                        angularVelocity: [...prevPlayer.angularVelocity],
                        boost: [...prevPlayer.boost],
                        position: [...prevPlayer.position]
                    };

                    // Velocity - use actual velocity from replay data (userData.velocity)
                    const actor = currentActors[car.id];
                    const carVelocity = actor?.userData?.velocity;
                    if (carVelocity) {
                        const velocity = Math.sqrt(
                            carVelocity.x * carVelocity.x +
                            carVelocity.y * carVelocity.y +
                            carVelocity.z * carVelocity.z
                        );
                        newPlayerData.velocity = [...prevPlayer.velocity, { time: now, velocity }].slice(-HISTORY_SIZE);
                    }

                    // Altitude (userData.location.y = height, was Z in Rocket League)
                    const altitude = actor?.userData?.location?.y ?? car.position.y;
                    newPlayerData.altitude = [...prevPlayer.altitude, { time: now, value: altitude }].slice(-HISTORY_SIZE);

                    // Angular velocity (from Three.js userData if available)
                    if (car.angularVelocity) {
                        const angVelMagnitude = Math.sqrt(
                            car.angularVelocity.x * car.angularVelocity.x +
                            car.angularVelocity.y * car.angularVelocity.y +
                            car.angularVelocity.z * car.angularVelocity.z
                        );
                        newPlayerData.angularVelocity = [...prevPlayer.angularVelocity, { time: now, value: angVelMagnitude }].slice(-HISTORY_SIZE);
                    }

                    // Boost (from playerBoosts ref, keyed by playerName)
                    const boostValue = currentPlayerBoosts[car.playerName] ?? 0;
                    newPlayerData.boost = [...prevPlayer.boost, { time: now, value: boostValue }].slice(-HISTORY_SIZE);

                    // Position (for delta graph - actual rendered position)
                    newPlayerData.position = [...prevPlayer.position, {
                        time: now,
                        x: car.position.x,
                        y: car.position.y,
                        z: car.position.z
                    }].slice(-HISTORY_SIZE);

                    newData.players[car.id] = newPlayerData;
                });

                return newData;
            });
        };

        // Update at 30fps for smooth graphs
        const interval = setInterval(updatePhysics, 33);
        return () => clearInterval(interval);
    }, [isOpen, activeSection]); // Only depend on UI state, not on actors/playerBoosts (we use refs)

    const togglePlayer = (playerId) => {
        setExpandedPlayers(prev => {
            const next = new Set(prev);
            if (next.has(playerId)) {
                next.delete(playerId);
            } else {
                next.add(playerId);
            }
            return next;
        });
    };

    // Format position helper
    const formatPos = (pos) => pos ? `(${pos.x.toFixed(1)}, ${pos.y.toFixed(1)}, ${pos.z.toFixed(1)})` : 'N/A';
    const formatRot = (rot) => rot ? `(${(rot.x * 180 / Math.PI).toFixed(0)}°, ${(rot.y * 180 / Math.PI).toFixed(0)}°, ${(rot.z * 180 / Math.PI).toFixed(0)}°)` : 'N/A';

    return (
        <>
            {/* Toggle Button - New gradient style */}
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="pointer-events-auto relative p-[1px] rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 transition-all shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40"
                title="Toggle Debug Panel (F3)"
            >
                <div className="bg-gray-900 rounded-[7px] p-2">
                    <Bug size={20} className="text-cyan-400" />
                </div>
            </button>

            {/* Debug Panel */}
            {isOpen && (
                <div className="fixed inset-0 z-[9998] pointer-events-none">
                    {/* Panel with gradient border */}
                    <div
                        ref={panelRef}
                        className="absolute p-[1px] rounded-xl bg-gradient-to-br from-cyan-500/50 via-blue-500/30 to-cyan-500/50 shadow-2xl shadow-cyan-500/10 pointer-events-auto"
                        style={{
                            left: panelState.x,
                            top: panelState.y,
                            width: panelState.width,
                            height: panelState.height,
                        }}
                    >
                        <div className="bg-gray-900/95 backdrop-blur-sm rounded-[11px] overflow-hidden flex flex-col h-full">
                            {/* Header - Draggable */}
                            <div
                                className={`flex items-center justify-between p-3 bg-gradient-to-r from-cyan-900/30 to-blue-900/30 border-b border-cyan-500/20 ${isDragging ? 'cursor-grabbing' : 'cursor-grab'}`}
                                onMouseDown={handleDragStart}
                            >
                                <div className="flex items-center gap-2">
                                    <GripVertical size={14} className="text-gray-500" />
                                    <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-cyan-500 to-blue-500 flex items-center justify-center">
                                        <Bug size={14} className="text-white" />
                                    </div>
                                    <span className="font-bold text-sm text-white select-none">Debug Panel</span>
                                </div>
                                <button
                                    onClick={(e) => { e.stopPropagation(); setIsOpen(false); }}
                                    className="p-1 rounded-lg bg-gray-800/50 hover:bg-red-500/80 text-gray-400 hover:text-white transition-colors"
                                >
                                    <X size={14} />
                                </button>
                            </div>

                            {/* Tab Navigation */}
                            <div className="flex gap-1 p-2 border-b border-gray-800 bg-gray-900/50">
                                <button
                                    onClick={() => setActiveSection('info')}
                                    className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-all ${
                                        activeSection === 'info'
                                            ? 'bg-gradient-to-r from-cyan-600 to-blue-600 text-white shadow-lg shadow-cyan-500/25'
                                            : 'text-gray-400 hover:text-white hover:bg-gray-800/50'
                                    }`}
                                >
                                    <Settings size={12} />
                                    Info
                                </button>
                                <button
                                    onClick={() => setActiveSection('physics')}
                                    className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-all ${
                                        activeSection === 'physics'
                                            ? 'bg-gradient-to-r from-cyan-600 to-blue-600 text-white shadow-lg shadow-cyan-500/25'
                                            : 'text-gray-400 hover:text-white hover:bg-gray-800/50'
                                    }`}
                                >
                                    <BarChart3 size={12} />
                                    Physics
                                </button>
                                <button
                                    onClick={() => setActiveSection('interpolation')}
                                    className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-all ${
                                        activeSection === 'interpolation'
                                            ? 'bg-gradient-to-r from-cyan-600 to-blue-600 text-white shadow-lg shadow-cyan-500/25'
                                            : 'text-gray-400 hover:text-white hover:bg-gray-800/50'
                                    }`}
                                >
                                    <Wand2 size={12} />
                                    Interpolation
                                </button>
                                <button
                                    onClick={() => setActiveSection('timeline')}
                                    className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-all ${
                                        activeSection === 'timeline'
                                            ? 'bg-gradient-to-r from-cyan-600 to-blue-600 text-white shadow-lg shadow-cyan-500/25'
                                            : 'text-gray-400 hover:text-white hover:bg-gray-800/50'
                                    }`}
                                >
                                    <Activity size={12} />
                                    Timeline
                                </button>
                            </div>

                            {/* Content */}
                            <div className="p-3 overflow-y-auto flex-1 space-y-3">
                                {activeSection === 'info' && (
                                    <>
                                        {/* Controls Section */}
                                        <div className="p-2 rounded-lg bg-gray-800/30 border border-gray-700/50">
                                            <div className="flex items-center gap-2 mb-2">
                                                <Settings size={12} className="text-cyan-400" />
                                                <span className="text-xs font-medium text-white">Controls</span>
                                            </div>
                                            <label className="flex items-center gap-2 cursor-pointer">
                                                <input
                                                    type="checkbox"
                                                    checked={interpolationEnabled}
                                                    onChange={(e) => onInterpolationToggle?.(e.target.checked)}
                                                    className="w-4 h-4 rounded border-gray-600 bg-gray-800 text-cyan-500 focus:ring-cyan-500 focus:ring-offset-0"
                                                />
                                                <span className={`text-xs ${interpolationEnabled ? 'text-cyan-400' : 'text-red-400'}`}>
                                                    Interpolation {interpolationEnabled ? 'ON' : 'OFF'}
                                                </span>
                                            </label>
                                            <p className="text-[10px] text-gray-500 mt-1 ml-6">
                                                {interpolationEnabled ? 'Smooth position updates' : 'Raw frame data only'}
                                            </p>
                                        </div>

                                        {/* Load Local Replay */}
                                        {onLoadReplayFile && (
                                            <div className="p-2 rounded-lg bg-gray-800/30 border border-gray-700/50">
                                                <div className="flex items-center gap-2 mb-2">
                                                    <Upload size={12} className="text-orange-400" />
                                                    <span className="text-xs font-medium text-white">Load Local Replay</span>
                                                </div>
                                                <input
                                                    ref={fileInputRef}
                                                    type="file"
                                                    accept=".rlrf"
                                                    onChange={handleFileSelect}
                                                    className="hidden"
                                                />
                                                <button
                                                    onClick={() => fileInputRef.current?.click()}
                                                    disabled={isLoadingFile}
                                                    className={`w-full px-3 py-2 text-xs font-medium rounded-lg transition-all flex items-center justify-center gap-2 ${
                                                        isLoadingFile
                                                            ? 'bg-gray-700 text-gray-400 cursor-wait'
                                                            : 'bg-gradient-to-r from-orange-600 to-amber-600 text-white hover:from-orange-500 hover:to-amber-500'
                                                    }`}
                                                >
                                                    <Upload size={14} />
                                                    {isLoadingFile ? 'Loading...' : 'Select .rlrf file'}
                                                </button>
                                                <p className="text-[10px] text-gray-500 mt-1">
                                                    Load a local binary replay for testing
                                                </p>
                                            </div>
                                        )}

                                        {/* Frame Info */}
                                        <div className="p-2 rounded-lg bg-gray-800/30 border border-gray-700/50">
                                            <div className="flex items-center gap-2 mb-2">
                                                <Activity size={12} className="text-violet-400" />
                                                <span className="text-xs font-medium text-white">Frame Info</span>
                                            </div>
                                            <div className="grid grid-cols-2 gap-2 text-[11px]">
                                                <div>
                                                    <span className="text-gray-500">Time:</span>
                                                    <span className="text-yellow-300 ml-1 font-mono">{currentTime.toFixed(2)}s</span>
                                                </div>
                                                {frameInfo && (
                                                    <div>
                                                        <span className="text-gray-500">Frame:</span>
                                                        <span className="text-yellow-300 ml-1 font-mono">{frameInfo.currentFrame}/{frameInfo.totalFrames}</span>
                                                    </div>
                                                )}
                                            </div>
                                        </div>

                                        {/* Ball Section */}
                                        <div className="p-2 rounded-lg bg-gradient-to-br from-orange-900/20 to-orange-800/10 border border-orange-500/20">
                                            <div className="flex items-center gap-2 mb-2">
                                                <Circle size={12} className="text-orange-400" />
                                                <span className="text-xs font-medium text-white">Ball</span>
                                                <span className={`ml-auto text-[10px] font-bold ${ball?.userData?.sleeping ? 'text-red-400' : 'text-green-400'}`}>
                                                    {ball?.userData?.sleeping ? 'SLEEPING' : 'AWAKE'}
                                                </span>
                                            </div>
                                            {ballPosition ? (
                                                <div className="space-y-1 text-[10px] font-mono">
                                                    <div className="flex justify-between">
                                                        <span className="text-gray-400">Pos:</span>
                                                        <span className="text-orange-300">{formatPos(ballPosition)}</span>
                                                    </div>
                                                    {ball?.rotation && (
                                                        <div className="flex justify-between">
                                                            <span className="text-gray-400">Rot:</span>
                                                            <span className="text-gray-300">{formatRot(ball.rotation)}</span>
                                                        </div>
                                                    )}
                                                </div>
                                            ) : (
                                                <span className="text-xs text-gray-500">Not spawned</span>
                                            )}
                                        </div>

                                        {/* Blue Team */}
                                        {blueCars.length > 0 && (
                                            <div className="p-2 rounded-lg bg-gradient-to-br from-blue-900/20 to-blue-800/10 border border-blue-500/20">
                                                <div className="flex items-center gap-2 mb-2">
                                                    <Users size={12} className="text-blue-400" />
                                                    <span className="text-xs font-medium text-white">Blue Team</span>
                                                </div>
                                                <div className="space-y-2">
                                                    {blueCars.map(car => (
                                                        <div key={car.id} className="bg-blue-900/20 rounded p-2 border-l-2 border-blue-500">
                                                            <button
                                                                onClick={() => togglePlayer(car.id)}
                                                                className="w-full flex items-center gap-2"
                                                            >
                                                                {expandedPlayers.has(car.id) ?
                                                                    <ChevronDown size={12} className="text-gray-400" /> :
                                                                    <ChevronRight size={12} className="text-gray-400" />
                                                                }
                                                                <span className="text-xs text-blue-300 font-medium">{car.name}</span>
                                                                <span className={`ml-auto text-[9px] font-bold ${car.sleeping ? 'text-red-400' : 'text-green-400'}`}>
                                                                    {car.sleeping ? 'SLEEP' : 'AWAKE'}
                                                                </span>
                                                            </button>
                                                            {expandedPlayers.has(car.id) && (
                                                                <div className="mt-2 pt-2 border-t border-blue-500/20 space-y-1 text-[10px] font-mono">
                                                                    <div className="text-gray-500">ID: {car.id}</div>
                                                                    <div className="flex justify-between">
                                                                        <span className="text-gray-400">Pos:</span>
                                                                        <span className="text-blue-300">{formatPos(car.position)}</span>
                                                                    </div>
                                                                    <div className="flex justify-between">
                                                                        <span className="text-gray-400">Rot:</span>
                                                                        <span className="text-gray-300">{formatRot(car.rotation)}</span>
                                                                    </div>
                                                                </div>
                                                            )}
                                                        </div>
                                                    ))}
                                                </div>
                                            </div>
                                        )}

                                        {/* Orange Team */}
                                        {orangeCars.length > 0 && (
                                            <div className="p-2 rounded-lg bg-gradient-to-br from-orange-900/20 to-orange-800/10 border border-orange-500/20">
                                                <div className="flex items-center gap-2 mb-2">
                                                    <Users size={12} className="text-orange-400" />
                                                    <span className="text-xs font-medium text-white">Orange Team</span>
                                                </div>
                                                <div className="space-y-2">
                                                    {orangeCars.map(car => (
                                                        <div key={car.id} className="bg-orange-900/20 rounded p-2 border-l-2 border-orange-500">
                                                            <button
                                                                onClick={() => togglePlayer(car.id)}
                                                                className="w-full flex items-center gap-2"
                                                            >
                                                                {expandedPlayers.has(car.id) ?
                                                                    <ChevronDown size={12} className="text-gray-400" /> :
                                                                    <ChevronRight size={12} className="text-gray-400" />
                                                                }
                                                                <span className="text-xs text-orange-300 font-medium">{car.name}</span>
                                                                <span className={`ml-auto text-[9px] font-bold ${car.sleeping ? 'text-red-400' : 'text-green-400'}`}>
                                                                    {car.sleeping ? 'SLEEP' : 'AWAKE'}
                                                                </span>
                                                            </button>
                                                            {expandedPlayers.has(car.id) && (
                                                                <div className="mt-2 pt-2 border-t border-orange-500/20 space-y-1 text-[10px] font-mono">
                                                                    <div className="text-gray-500">ID: {car.id}</div>
                                                                    <div className="flex justify-between">
                                                                        <span className="text-gray-400">Pos:</span>
                                                                        <span className="text-orange-300">{formatPos(car.position)}</span>
                                                                    </div>
                                                                    <div className="flex justify-between">
                                                                        <span className="text-gray-400">Rot:</span>
                                                                        <span className="text-gray-300">{formatRot(car.rotation)}</span>
                                                                    </div>
                                                                </div>
                                                            )}
                                                        </div>
                                                    ))}
                                                </div>
                                            </div>
                                        )}
                                    </>
                                )}

                                {activeSection === 'physics' && (
                                    <>
                                        {/* Physics Sub-tabs: Ball + Players */}
                                        <div className="flex flex-wrap gap-1 mb-3 p-1 bg-gray-800/30 rounded-lg">
                                            <button
                                                onClick={() => setPhysicsSubTab('ball')}
                                                className={`flex items-center gap-1 px-2 py-1 text-[10px] font-medium rounded transition-all ${
                                                    physicsSubTab === 'ball'
                                                        ? 'bg-orange-500/30 text-orange-300 border border-orange-500/50'
                                                        : 'text-gray-400 hover:text-white hover:bg-gray-700/30'
                                                }`}
                                            >
                                                <Circle size={8} />
                                                Ball
                                            </button>
                                            {blueCars.map(car => (
                                                <button
                                                    key={car.id}
                                                    onClick={() => setPhysicsSubTab(car.id)}
                                                    className={`flex items-center gap-1 px-2 py-1 text-[10px] font-medium rounded transition-all ${
                                                        physicsSubTab === car.id
                                                            ? 'bg-blue-500/30 text-blue-300 border border-blue-500/50'
                                                            : 'text-gray-400 hover:text-white hover:bg-gray-700/30'
                                                    }`}
                                                >
                                                    <div className="w-2 h-2 rounded-full bg-blue-500" />
                                                    {car.name.length > 10 ? car.name.slice(0, 10) + '...' : car.name}
                                                </button>
                                            ))}
                                            {orangeCars.map(car => (
                                                <button
                                                    key={car.id}
                                                    onClick={() => setPhysicsSubTab(car.id)}
                                                    className={`flex items-center gap-1 px-2 py-1 text-[10px] font-medium rounded transition-all ${
                                                        physicsSubTab === car.id
                                                            ? 'bg-orange-500/30 text-orange-300 border border-orange-500/50'
                                                            : 'text-gray-400 hover:text-white hover:bg-gray-700/30'
                                                    }`}
                                                >
                                                    <div className="w-2 h-2 rounded-full bg-orange-500" />
                                                    {car.name.length > 10 ? car.name.slice(0, 10) + '...' : car.name}
                                                </button>
                                            ))}
                                        </div>

                                        {/* Ball Physics */}
                                        {physicsSubTab === 'ball' && (
                                            <div className="space-y-2">
                                                <VelocityGraph
                                                    data={physicsData.ball.velocity}
                                                    color="#f97316"
                                                    label="Velocity (from replay data)"
                                                />
                                                <PositionDeltaGraph
                                                    positionData={physicsData.ball.position}
                                                    color="#22c55e"
                                                    label="Actual Movement (rendered position delta)"
                                                />
                                                <AccelerationGraph
                                                    data={physicsData.ball.velocity}
                                                    color="#fbbf24"
                                                    label="Acceleration (velocity change)"
                                                />
                                                <PhysicsGraph
                                                    data={physicsData.ball.altitude}
                                                    color="#fb923c"
                                                    label="Altitude"
                                                    unit=" uu"
                                                    minValue={0}
                                                    maxValueDefault={2000}
                                                    thresholdValue={642}
                                                    thresholdLabel="GOAL HEIGHT"
                                                    valueFormatter={(v) => v.toFixed(0)}
                                                />
                                            </div>
                                        )}

                                        {/* Player Physics */}
                                        {cars.map(car => physicsSubTab === car.id && (
                                            <div key={car.id} className="space-y-2">
                                                <div className={`text-xs font-medium ${car.team === 0 ? 'text-blue-300' : 'text-orange-300'}`}>
                                                    {car.name}
                                                </div>
                                                <VelocityGraph
                                                    data={physicsData.players[car.id]?.velocity || []}
                                                    color={car.team === 0 ? '#3b82f6' : '#f97316'}
                                                    label="Velocity (from replay data)"
                                                />
                                                <PositionDeltaGraph
                                                    positionData={physicsData.players[car.id]?.position || []}
                                                    color={car.team === 0 ? '#22c55e' : '#84cc16'}
                                                    label="Actual Movement (rendered position delta)"
                                                />
                                                <AccelerationGraph
                                                    data={physicsData.players[car.id]?.velocity || []}
                                                    color={car.team === 0 ? '#60a5fa' : '#fbbf24'}
                                                    label="Acceleration (velocity change)"
                                                />
                                                <PhysicsGraph
                                                    data={physicsData.players[car.id]?.boost || []}
                                                    color={car.team === 0 ? '#22d3ee' : '#fbbf24'}
                                                    label="Boost"
                                                    unit="%"
                                                    minValue={0}
                                                    maxValueDefault={100}
                                                    thresholdValue={100}
                                                    thresholdLabel="FULL"
                                                    valueFormatter={(v) => v.toFixed(0)}
                                                />
                                            </div>
                                        ))}

                                        {/* Physics Legend */}
                                        <div className="p-2 rounded-lg bg-gray-800/30 border border-gray-700/50 text-[10px] text-gray-400 space-y-1">
                                            <div className="flex items-center gap-2">
                                                <Zap size={10} className="text-red-400" />
                                                <span>Supersonic: {SUPERSONIC_THRESHOLD} uu/s (~79 km/h)</span>
                                            </div>
                                            <div className="flex items-center gap-2">
                                                <ArrowUp size={10} className="text-green-400" />
                                                <span>Altitude: Height above ground (uu = Unreal Units)</span>
                                            </div>
                                            <div className="flex items-center gap-2">
                                                <Fuel size={10} className="text-cyan-400" />
                                                <span>Boost: Current boost percentage (0-100%)</span>
                                            </div>
                                            <div className="text-gray-500 mt-1">
                                                Tracking last 5 seconds at 30 FPS
                                            </div>
                                        </div>
                                    </>
                                )}

                                {activeSection === 'interpolation' && (
                                    <>
                                        {/* Interpolation Method Selection */}
                                        <div className="p-3 rounded-lg bg-gradient-to-br from-purple-900/20 to-indigo-800/10 border border-purple-500/20">
                                            <div className="flex items-center gap-2 mb-3">
                                                <Wand2 size={14} className="text-purple-400" />
                                                <span className="text-sm font-medium text-white">Interpolation Method</span>
                                            </div>

                                            {/* Method Radio Buttons */}
                                            <div className="space-y-2">
                                                {[
                                                    { value: 'adaptive-smooth', label: 'Adaptive Smooth ★', desc: 'Recommended: adapts smoothing to speed/direction changes', category: 'adaptive' },
                                                    { value: 'lerp-smooth', label: 'Linear + Moving Avg', desc: 'LERP with moving average filter' },
                                                    { value: 'lerp', label: 'Linear (LERP)', desc: 'Simple linear interpolation between keyframes' },
                                                    { value: 'position-lerp', label: 'Position LERP', desc: 'Ignores velocity - pure position interpolation', category: 'position' },
                                                    { value: 'position-catmull', label: 'Position Catmull-Rom', desc: 'Smooth spline through positions only (no velocity)', category: 'position' },
                                                    { value: 'position-smooth', label: 'Position + Low-pass', desc: 'Position lerp with low-pass filter to reduce jitter', category: 'position' },
                                                    { value: 'one-euro', label: 'One Euro Filter', desc: 'Adaptive: smooth when slow, responsive when fast' },
                                                    { value: 'lerp-ema', label: 'Linear + EMA', desc: 'Exponential Moving Average - recent frames weighted more' },
                                                    { value: 'lerp-dema', label: 'Linear + Double EMA', desc: 'Holt\'s method - tracks trends for smoother motion' },
                                                    { value: 'lerp-wma', label: 'Linear + Weighted MA', desc: 'Weighted average - linear weight distribution' },
                                                    { value: 'lerp-gauss', label: 'Linear + Gaussian', desc: 'Gaussian-weighted smoothing - bell curve distribution' },
                                                    { value: 'catmull-rom', label: 'Catmull-Rom Spline', desc: 'Smooth curve through 4 keyframes' },
                                                    { value: 'hermite', label: 'Hermite Spline', desc: 'C1-smooth curves using positions + velocities as tangents', category: 'physics' },
                                                    { value: 'predict-correct', label: 'Predict + Correct', desc: 'Dead reckoning with velocity - experimental', category: 'physics' },
                                                    { value: 'velocity-smooth', label: 'Velocity Smooth', desc: 'Velocity-based with clamped correction + gravity - best for bad replays', category: 'physics' },
                                                    { value: 'physics-tick', label: 'Physics Tick', desc: 'Constant velocity + linear error distribution - reduced jitter', category: 'physics' },
                                                    { value: 'velocity-only', label: 'Velocity Only', desc: 'EXPERIMENTAL: Pure velocity, ignores positions - smoothest but may drift', category: 'physics' },
                                                    { value: 'smart-hybrid', label: 'Smart Hybrid', desc: 'Auto-detects collisions: velocity-based for normal, lerp for impacts', category: 'physics' },
                                                    { value: 'time-shifted', label: 'Time-Shifted (Filter)', desc: 'Filters bad frames then lerps - may lose precision', category: 'physics' },
                                                    { value: 'physics-sim', label: '⭐ Physics Sim (120Hz)', desc: 'Simulates RL 120Hz physics, corrects 30Hz recording offsets', category: 'physics' }
                                                ].map(method => (
                                                    <label
                                                        key={method.value}
                                                        className={`flex items-start gap-3 p-2 rounded-lg cursor-pointer transition-all ${
                                                            interpolationMethod === method.value
                                                                ? 'bg-purple-500/20 border border-purple-500/50'
                                                                : 'bg-gray-800/30 border border-transparent hover:bg-gray-700/30'
                                                        }`}
                                                    >
                                                        <input
                                                            type="radio"
                                                            name="interpolationMethod"
                                                            value={method.value}
                                                            checked={interpolationMethod === method.value}
                                                            onChange={(e) => onInterpolationMethodChange?.(e.target.value)}
                                                            className="mt-1 w-4 h-4 text-purple-500 bg-gray-800 border-gray-600 focus:ring-purple-500"
                                                        />
                                                        <div>
                                                            <div className={`text-xs font-medium ${interpolationMethod === method.value ? 'text-purple-300' : 'text-gray-300'}`}>
                                                                {method.label}
                                                            </div>
                                                            <div className="text-[10px] text-gray-500 mt-0.5">{method.desc}</div>
                                                        </div>
                                                    </label>
                                                ))}
                                            </div>
                                        </div>

                                        {/* Smoothing Settings (visible for methods that use smoothing window) */}
                                        {['lerp-smooth', 'lerp-ema', 'lerp-dema', 'lerp-wma', 'lerp-gauss', 'one-euro', 'position-smooth', 'adaptive-smooth'].includes(interpolationMethod) && (
                                            <div className="p-3 rounded-lg bg-gray-800/30 border border-gray-700/50">
                                                <div className="flex items-center justify-between mb-3">
                                                    <span className="text-xs font-medium text-white">
                                                        {interpolationMethod === 'one-euro' ? 'Min Cutoff (lower = smoother)' :
                                                         interpolationMethod === 'position-smooth' ? 'Low-pass Alpha (lower = smoother)' : 'Smoothing Window Size'}
                                                    </span>
                                                    <span className="text-sm font-mono font-bold text-purple-400">{smoothingWindowSize}</span>
                                                </div>
                                                <input
                                                    type="range"
                                                    min="2"
                                                    max="20"
                                                    value={smoothingWindowSize}
                                                    onChange={(e) => onSmoothingWindowSizeChange?.(parseInt(e.target.value))}
                                                    className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-purple-500"
                                                />
                                                <div className="flex justify-between text-[10px] text-gray-500 mt-1">
                                                    <span>2 (less smooth)</span>
                                                    <span>20 (more smooth)</span>
                                                </div>
                                                <div className="mt-2 p-2 rounded bg-gray-900/50 text-[10px] text-gray-400">
                                                    <strong className="text-yellow-400">Note:</strong> Higher values = smoother motion but more latency (~{Math.round(smoothingWindowSize * 16.67)}ms delay at 60fps)
                                                </div>
                                            </div>
                                        )}

                                        {/* Enable/Disable Toggle */}
                                        <div className="p-3 rounded-lg bg-gray-800/30 border border-gray-700/50">
                                            <label className="flex items-center justify-between cursor-pointer">
                                                <span className="text-xs font-medium text-white">Interpolation Enabled</span>
                                                <div className="relative">
                                                    <input
                                                        type="checkbox"
                                                        checked={interpolationEnabled}
                                                        onChange={(e) => onInterpolationToggle?.(e.target.checked)}
                                                        className="sr-only"
                                                    />
                                                    <div className={`w-10 h-5 rounded-full transition-colors ${interpolationEnabled ? 'bg-purple-500' : 'bg-gray-600'}`}>
                                                        <div className={`w-4 h-4 rounded-full bg-white shadow transform transition-transform ${interpolationEnabled ? 'translate-x-5' : 'translate-x-0.5'} mt-0.5`} />
                                                    </div>
                                                </div>
                                            </label>
                                            <div className="text-[10px] text-gray-500 mt-1">
                                                {interpolationEnabled ? 'Smooth position updates between keyframes' : 'Raw frame data only (jumpy movement)'}
                                            </div>
                                        </div>

                                        {/* Current Status */}
                                        <div className="p-2 rounded-lg bg-gray-800/30 border border-gray-700/50">
                                            <div className="text-[10px] text-gray-400 space-y-1">
                                                <div className="flex justify-between">
                                                    <span>Current Method:</span>
                                                    <span className="font-mono text-purple-300">{interpolationMethod}</span>
                                                </div>
                                                <div className="flex justify-between">
                                                    <span>Status:</span>
                                                    <span className={interpolationEnabled ? 'text-green-400' : 'text-red-400'}>
                                                        {interpolationEnabled ? 'ENABLED' : 'DISABLED'}
                                                    </span>
                                                </div>
                                                {['lerp-smooth', 'lerp-ema', 'lerp-dema', 'lerp-wma', 'lerp-gauss', 'one-euro', 'position-smooth'].includes(interpolationMethod) && (
                                                    <div className="flex justify-between">
                                                        <span>{interpolationMethod === 'position-smooth' ? 'Alpha:' : 'Window Size:'}</span>
                                                        <span className="font-mono text-purple-300">{smoothingWindowSize} {interpolationMethod === 'position-smooth' ? '' : 'frames'}</span>
                                                    </div>
                                                )}
                                            </div>
                                        </div>

                                        {/* Help Section */}
                                        <div className="p-2 rounded-lg bg-gray-900/50 border border-gray-700/30 text-[10px] text-gray-500 space-y-1.5">
                                            <div className="text-xs font-medium text-gray-400 mb-2">Methods:</div>
                                            <div>• <strong className="text-gray-300">Hermite</strong>: Uses velocity data - physically accurate</div>
                                            <div>• <strong className="text-gray-300">LERP</strong>: Default, no smoothing, raw data</div>
                                            <div className="text-xs font-medium text-green-400 mt-2 mb-1">Position-based (ignores velocity):</div>
                                            <div>• <strong className="text-green-300">Position LERP</strong>: Pure position interpolation</div>
                                            <div>• <strong className="text-green-300">Position Catmull-Rom</strong>: Spline through positions</div>
                                            <div>• <strong className="text-green-300">Position + Low-pass</strong>: Positions + filter</div>
                                            <div className="text-xs font-medium text-gray-400 mt-2 mb-1">With smoothing:</div>
                                            <div>• <strong className="text-gray-300">Moving Avg</strong>: Simple average over N frames</div>
                                            <div>• <strong className="text-gray-300">EMA</strong>: Exponential - recent frames weighted more</div>
                                            <div>• <strong className="text-cyan-300">One Euro</strong>: Adaptive - smooth when slow, responsive when fast</div>
                                            <div className="text-yellow-400/80 mt-2">Use slow-motion (0.1x speed) to evaluate smoothness</div>
                                        </div>
                                    </>
                                )}

                                {/* Timeline Section */}
                                {activeSection === 'timeline' && (
                                    <>
                                        <KeyframeTimeline
                                            ballTimeline={ballTimeline}
                                            playerTimelines={playerTimelines}
                                            currentTime={currentTime}
                                            windowSize={timelineWindowSize}
                                            onWindowSizeChange={setTimelineWindowSize}
                                            isPlaying={isPlaying}
                                            playbackSpeed={playbackSpeed}
                                            onPlayPause={onPlayPause}
                                            onPlaybackSpeedChange={onPlaybackSpeedChange}
                                            onSeek={onSeek}
                                        />

                                        {/* Timeline Stats */}
                                        <div className="p-2 rounded-lg bg-gray-800/30 border border-gray-700/50">
                                            <div className="text-[10px] text-gray-400 space-y-1">
                                                <div className="text-xs font-medium text-white mb-2">Keyframe Statistics</div>
                                                <div className="flex justify-between">
                                                    <span>Ball keyframes:</span>
                                                    <span className="font-mono text-blue-400">{ballTimeline.length}</span>
                                                </div>
                                                {Object.entries(playerTimelines).slice(0, 4).map(([name, timeline]) => (
                                                    <div key={name} className="flex justify-between">
                                                        <span className="truncate max-w-[150px]">{name}:</span>
                                                        <span className="font-mono text-orange-400">{timeline.length}</span>
                                                    </div>
                                                ))}
                                                {Object.keys(playerTimelines).length > 4 && (
                                                    <div className="text-gray-500 text-center">
                                                        +{Object.keys(playerTimelines).length - 4} more players...
                                                    </div>
                                                )}
                                            </div>
                                        </div>

                                        {/* Help */}
                                        <div className="p-2 rounded-lg bg-gray-900/50 border border-gray-700/30 text-[10px] text-gray-500 space-y-1">
                                            <div className="text-xs font-medium text-gray-400 mb-1">Understanding the Timeline</div>
                                            <div>• Each <strong className="text-gray-300">vertical line</strong> = one keyframe from replay data</div>
                                            <div>• <strong className="text-purple-400">Purple line</strong> = current playback position</div>
                                            <div>• Ball typically updates at <strong className="text-blue-400">~28 Hz</strong></div>
                                            <div>• Cars typically update at <strong className="text-orange-400">~13 Hz</strong> with high variance</div>
                                            <div className="text-yellow-400/80 mt-1">Irregular spacing = source of jitter</div>
                                        </div>
                                    </>
                                )}
                            </div>

                            {/* Footer Stats */}
                            <div className="px-3 py-2 border-t border-gray-800 bg-gray-900/50 text-[10px] text-gray-500 flex items-center justify-between">
                                <span>Actors: {Object.keys(actors).length} | Cars: {cars.length}</span>
                                <span className="text-gray-600">Press F3 to toggle</span>
                            </div>
                        </div>

                        {/* Resize handle */}
                        <div
                            className="absolute bottom-0 right-0 w-4 h-4 cursor-se-resize z-10"
                            onMouseDown={handleResizeStart}
                        >
                            <svg
                                className="w-full h-full text-gray-500 hover:text-cyan-400 transition-colors"
                                viewBox="0 0 24 24"
                                fill="currentColor"
                            >
                                <path d="M22 22H20V20H22V22ZM22 18H20V16H22V18ZM18 22H16V20H18V22ZM22 14H20V12H22V14ZM18 18H16V16H18V18ZM14 22H12V20H14V22Z" />
                            </svg>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
}
