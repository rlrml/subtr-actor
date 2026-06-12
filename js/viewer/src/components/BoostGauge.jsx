import { useState, useEffect, useRef, useCallback } from 'react';

/**
 * Rocket League style boost gauge - circular design with segments
 * @param {number} boost - Boost amount (0-100)
 * @param {number} size - Size of the gauge in pixels
 * @param {number} team - Team number (0 = blue, 1 = red)
 * @param {number} playbackSpeed - Current playback speed (default 1.0)
 */
export function BoostGauge({ boost = 0, size = 180, team = 0, playbackSpeed = 1.0 }) {
    const percentage = Math.round(Math.max(0, Math.min(100, boost)));
    const prevBoostRef = useRef(percentage);
    const [isFlashing, setIsFlashing] = useState(false);
    const flashTimerRef = useRef(null);

    // Animated boost value - smoothly transitions to target
    const [displayedBoost, setDisplayedBoost] = useState(percentage);
    const animationRef = useRef(null);

    // Animation duration scales inversely with playback speed (faster playback = faster animation)
    const baseAnimationDuration = 200; // ms
    const animationDuration = Math.max(50, baseAnimationDuration / Math.max(0.5, playbackSpeed));

    // Animate displayedBoost toward percentage
    useEffect(() => {
        // Cancel any existing animation
        if (animationRef.current) {
            cancelAnimationFrame(animationRef.current);
        }

        const startValue = displayedBoost;
        const endValue = percentage;
        const startTime = performance.now();

        // Skip animation if change is very small or instantaneous (e.g., seeking)
        const diff = Math.abs(endValue - startValue);
        if (diff < 1) {
            setDisplayedBoost(endValue);
            return;
        }

        const animate = (currentTime) => {
            const elapsed = currentTime - startTime;
            const progress = Math.min(1, elapsed / animationDuration);

            // Ease-out cubic for smooth deceleration
            const easeOut = 1 - Math.pow(1 - progress, 3);
            const currentValue = startValue + (endValue - startValue) * easeOut;

            setDisplayedBoost(Math.round(currentValue));

            if (progress < 1) {
                animationRef.current = requestAnimationFrame(animate);
            } else {
                setDisplayedBoost(endValue);
            }
        };

        animationRef.current = requestAnimationFrame(animate);

        return () => {
            if (animationRef.current) {
                cancelAnimationFrame(animationRef.current);
            }
        };
    }, [percentage, animationDuration]);

    // Detect boost increase and trigger flash
    useEffect(() => {
        if (percentage > prevBoostRef.current) {
            // Clear any existing timer
            if (flashTimerRef.current) {
                clearTimeout(flashTimerRef.current);
            }
            setIsFlashing(true);
            flashTimerRef.current = setTimeout(() => {
                setIsFlashing(false);
                flashTimerRef.current = null;
            }, 150);
        }
        prevBoostRef.current = percentage;
    }, [percentage]);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            if (flashTimerRef.current) {
                clearTimeout(flashTimerRef.current);
            }
            if (animationRef.current) {
                cancelAnimationFrame(animationRef.current);
            }
        };
    }, []);

    // Font color based on team and boost level
    const teamColors = {
        0: '#6689F3', // Blue team
        1: '#FEEAAE'  // Orange team
    };
    const fontColor = percentage > 80 ? '#FFFFFF' : (teamColors[team] || teamColors[0]);

    // Center glow color based on team - intensity scales with boost percentage
    // Blue team gets more intense/saturated above 80%
    const teamGlowColors = {
        0: percentage > 80
            ? { r: 50, g: 120, b: 255 }   // Intense blue above 80%
            : { r: 102, g: 137, b: 243 }, // Normal blue #6689F3
        1: { r: 255, g: 30, b: 30 }       // Red team (pure red glow)
    };
    const glowColor = teamGlowColors[team] || teamGlowColors[0];
    // Scale opacity based on boost (0% boost = 0 opacity, 100% boost = 0.9 opacity)
    // When flashing, boost the glow intensity
    const baseGlowOpacity = (percentage / 100) * 0.9;
    const glowOpacity = isFlashing ? Math.min(1, baseGlowOpacity + 0.4) : baseGlowOpacity;
    const glowSpread = isFlashing ? 1.3 : 1; // Extend glow when flashing

    // SVG dimensions
    const center = size / 2;
    const innerRadius = size / 2 - 42; // Inner circle radius (larger margin to fit all bars)

    // Gauge configuration
    // Start at bottom (90° in SVG coords) and go counter-clockwise to top-right
    const startAngle = 90;
    const totalArc = 225;

    // 5 groups of 9 bars = 45 total
    const barsPerGroup = [9, 9, 9, 9, 9];
    const numSegments = barsPerGroup.reduce((a, b) => a + b, 0); // 45

    // Base lengths for each group (with steps between groups)
    // Groups 1-2: linear progression (8→15), then +25% step up
    // Groups 3-4: linear progression (19→28), then +25% step up
    // Group 5: uniform size (35)
    const groupBaseLengths = [8, 11, 19, 23, 35];
    const groupLengthGrowth = [3, 4, 4, 5, 0]; // Linear growth within paired groups, 0 for last group

    // Colors: groups 1-4 gradient from #CD883F to #F1CD7A, group 5 depends on team
    const startColor = { r: 0xCD, g: 0x88, b: 0x3F }; // #CD883F
    const midColor = { r: 0xF1, g: 0xCD, b: 0x7A };   // #F1CD7A
    const endColorOrange = { r: 0xF7, g: 0xD1, b: 0xBB }; // #F7D1BB - Orange team
    const endColorBlue = { r: 0xE7, g: 0xF7, b: 0xFC };   // #E7F7FC - Blue team
    const endColor = team === 0 ? endColorBlue : endColorOrange;

    // Calculate how many segments should be filled based on animated displayedBoost
    const filledSegments = Math.round((displayedBoost / 100) * numSegments);

    // Generate segments
    const segments = [];
    let segmentIndex = 0;

    for (let group = 0; group < 5; group++) {
        const barsInGroup = barsPerGroup[group];
        const baseLength = groupBaseLengths[group];
        const lengthGrowth = groupLengthGrowth[group];

        for (let i = 0; i < barsInGroup; i++) {
            const globalProgress = segmentIndex / (numSegments - 1);
            const localProgress = i / (barsInGroup - 1);

            // Angle for this segment
            const angle = startAngle + (globalProgress * totalArc);
            const angleRad = (angle * Math.PI) / 180;

            const isFilled = segmentIndex < filledSegments;

            // Length: base + linear growth within group
            const segmentLength = baseLength + (localProgress * lengthGrowth);

            // Color calculation
            let color;
            if (group < 4) {
                // Groups 0, 1, 2, 3: gradient from startColor to midColor
                const colorProgress = (group * 9 + i) / (9 * 4 - 1);
                color = {
                    r: Math.round(startColor.r + (midColor.r - startColor.r) * colorProgress),
                    g: Math.round(startColor.g + (midColor.g - startColor.g) * colorProgress),
                    b: Math.round(startColor.b + (midColor.b - startColor.b) * colorProgress)
                };
            } else {
                // Group 4 (5th group): all same color
                color = endColor;
            }

            // Position segment - trapezoid shape (narrower at base, wider at top)
            const innerEdge = innerRadius + 2;
            const outerEdge = innerEdge + segmentLength;

            // Width varies: narrower at inner edge, wider at outer edge
            // Effect is proportional to bar length (short bars = almost rectangle, long bars = trapezoid)
            const baseWidth = 4;
            const outerWidth = baseWidth + segmentLength * 0.08;
            const innerWidth = baseWidth;

            // Calculate perpendicular angle for width offset
            const perpAngle = angleRad + Math.PI / 2;

            // Calculate 4 corners of trapezoid
            const innerLeft = {
                x: center + Math.cos(angleRad) * innerEdge - Math.cos(perpAngle) * (innerWidth / 2),
                y: center + Math.sin(angleRad) * innerEdge - Math.sin(perpAngle) * (innerWidth / 2)
            };
            const innerRight = {
                x: center + Math.cos(angleRad) * innerEdge + Math.cos(perpAngle) * (innerWidth / 2),
                y: center + Math.sin(angleRad) * innerEdge + Math.sin(perpAngle) * (innerWidth / 2)
            };
            const outerLeft = {
                x: center + Math.cos(angleRad) * outerEdge - Math.cos(perpAngle) * (outerWidth / 2),
                y: center + Math.sin(angleRad) * outerEdge - Math.sin(perpAngle) * (outerWidth / 2)
            };
            const outerRight = {
                x: center + Math.cos(angleRad) * outerEdge + Math.cos(perpAngle) * (outerWidth / 2),
                y: center + Math.sin(angleRad) * outerEdge + Math.sin(perpAngle) * (outerWidth / 2)
            };

            // Create polygon points string
            const points = `${innerLeft.x},${innerLeft.y} ${innerRight.x},${innerRight.y} ${outerRight.x},${outerRight.y} ${outerLeft.x},${outerLeft.y}`;

            segments.push({
                points,
                isFilled,
                index: segmentIndex,
                color: `rgb(${color.r}, ${color.g}, ${color.b})`
            });

            segmentIndex++;
        }
    }

    return (
        <div
            className="relative"
            style={{
                width: size,
                height: size,
            }}
        >
            <svg
                width={size}
                height={size}
                viewBox={`0 0 ${size} ${size}`}
            >
                <defs>
                    {/* Team color glow at center - intensity based on boost */}
                    <radialGradient id="teamGlowGradient" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" stopColor={`rgba(${glowColor.r}, ${glowColor.g}, ${glowColor.b}, ${glowOpacity})`} />
                        <stop offset={`${30 * glowSpread}%`} stopColor={`rgba(${glowColor.r}, ${glowColor.g}, ${glowColor.b}, ${glowOpacity * 0.7})`} />
                        <stop offset={`${60 * glowSpread}%`} stopColor={`rgba(${glowColor.r}, ${glowColor.g}, ${glowColor.b}, ${glowOpacity * 0.3})`} />
                        <stop offset="100%" stopColor={`rgba(${glowColor.r}, ${glowColor.g}, ${glowColor.b}, 0)`} />
                    </radialGradient>
                    {/* Dark background shadow */}
                    <radialGradient id="centerGradient" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" stopColor="rgba(0, 0, 0, 0.95)" />
                        <stop offset="50%" stopColor="rgba(0, 0, 0, 0.8)" />
                        <stop offset="80%" stopColor="rgba(0, 0, 0, 0.4)" />
                        <stop offset="100%" stopColor="rgba(0, 0, 0, 0)" />
                    </radialGradient>
                </defs>

                {/* Inner circle background - diffuse dark shadow */}
                <circle
                    cx={center}
                    cy={center}
                    r={size / 2 - 5}
                    fill="url(#centerGradient)"
                />
                {/* Team color glow overlay - intensity based on boost */}
                <circle
                    cx={center}
                    cy={center}
                    r={size / 2 - 5}
                    fill="url(#teamGlowGradient)"
                    style={{ transition: 'opacity 0.15s ease-out' }}
                />

                {/* Segments - only render filled ones */}
                {segments.filter(seg => seg.isFilled).map((seg, i) => {
                    // Flash to white/bright when boost increases
                    const fillColor = isFlashing ? '#FFFFFF' : seg.color;

                    return (
                        <polygon
                            key={i}
                            points={seg.points}
                            fill={fillColor}
                            style={{ transition: 'fill 0.15s ease-out' }}
                        />
                    );
                })}
            </svg>

            {/* Center content */}
            <div
                className="absolute inset-0 flex items-center justify-center"
            >
                {/* Boost number with ghost duplicate */}
                <div className="relative">
                    {/* Ghost duplicate - offset top-right */}
                    {displayedBoost > 0 && (
                        <span
                            className="absolute leading-none"
                            style={{
                                fontSize: size * 0.18,
                                color: fontColor,
                                opacity: 0.3,
                                fontFamily: '"Aspire", sans-serif',
                                fontWeight: 300,
                                letterSpacing: '0.02em',
                                transform: 'translate(5px, -5px)',
                            }}
                        >
                            {displayedBoost}
                        </span>
                    )}
                    {/* Main number */}
                    <span
                        className="relative leading-none"
                        style={{
                            fontSize: size * 0.18,
                            color: displayedBoost > 0 ? fontColor : '#666666',
                            textShadow: displayedBoost > 0
                                ? `0 0 15px ${fontColor}80, 0 0 30px ${fontColor}40`
                                : 'none',
                            fontFamily: '"Aspire", sans-serif',
                            fontWeight: 300,
                            letterSpacing: '0.02em',
                        }}
                    >
                        {displayedBoost}
                    </span>

                    {/* BOOST label - absolutely positioned, only visible above 80% */}
                    <span
                        className="absolute left-1/2 uppercase"
                        style={{
                            fontSize: size * 0.04,
                            color: fontColor,
                            textShadow: `0 0 8px ${fontColor}80`,
                            fontFamily: '"Aspire", sans-serif',
                            fontWeight: 300,
                            letterSpacing: '0.2em',
                            transform: 'translateX(-50%)',
                            top: '100%',
                            marginTop: 0,
                            opacity: percentage > 80 ? 1 : 0,
                            transition: 'opacity 0.2s ease-in-out',
                        }}
                    >
                        BOOST
                    </span>
                </div>
            </div>
        </div>
    );
}
