/**
 * PlayerTooltip - Shows player info on hover
 *
 * Features:
 * - Display car name (e.g., "Fennec", "Octane ZSR")
 * - Display hitbox type preset (e.g., "Octane", "Dominus")
 * - Display platform with logo (Steam, Epic, PlayStation, Xbox)
 * - Display match stats (goals, assists, saves, shots, demos) - real-time if timeline available
 * - Display current score - real-time if timeline available
 * - Display ping if timeline available
 * - Appears on hover, follows mouse position
 */

import React, { useMemo } from 'react';
import { Trophy, Target, Shield, Zap, Bot, Wifi, Skull } from 'lucide-react';
import { FaSteam, FaPlaystation, FaXbox } from 'react-icons/fa';
import { SiEpicgames } from 'react-icons/si';
import { getPlayerStatsAtTime } from '@/utils/timelineUtils';

/** Stats timeline entry type */
interface PlayerStatsTimelineEntry {
    time: number;
    frame: number;
    ping: number;
    goals: number;
    assists: number;
    saves: number;
    shots: number;
    score: number;
    demos: number;
    timePlayed?: number;
}

interface PlayerTooltipProps {
    /** Player name */
    playerName: string;
    /** Team (0 = blue, 1 = orange) */
    team: number;
    /** Car display name */
    carName: string;
    /** Hitbox type preset */
    hitboxType: string;
    /** Platform (OnlinePlatform_Steam, OnlinePlatform_Epic, etc.) */
    platform?: string | null;
    /** Goals scored (static end-game value, fallback) */
    goals?: number;
    /** Assists (static end-game value, fallback) */
    assists?: number;
    /** Saves (static end-game value, fallback) */
    saves?: number;
    /** Shots (static end-game value, fallback) */
    shots?: number;
    /** Match score (static end-game points, fallback) */
    matchScore?: number;
    /** Current score during playback (legacy, replaced by timeline) */
    currentScore?: number;
    /** Is this a bot? */
    isBot?: boolean;
    /** Position for the tooltip */
    position: { x: number; y: number };
    /** Player stats timelines for real-time display (optional) */
    playerStatsTimelines?: Record<string, PlayerStatsTimelineEntry[]>;
    /** Current playback time for timeline lookup (optional) */
    currentTime?: number;
}

// Hitbox type colors (matching HitboxManager)
const HITBOX_COLORS: Record<string, string> = {
    Octane: '#00ffff',   // Cyan
    Dominus: '#ff8800',  // Orange
    Plank: '#88ff00',    // Lime green
    Breakout: '#ff0088', // Pink
    Hybrid: '#8800ff',   // Purple
    Merc: '#ffff00',     // Yellow
};

// Platform display info
const PLATFORM_INFO: Record<string, { name: string; icon: React.ReactNode; color: string }> = {
    'OnlinePlatform_Steam': { name: 'Steam', icon: <FaSteam />, color: '#1b2838' },
    'OnlinePlatform_Epic': { name: 'Epic Games', icon: <SiEpicgames />, color: '#2f2f2f' },
    'OnlinePlatform_PS4': { name: 'PlayStation', icon: <FaPlaystation />, color: '#003791' },
    'OnlinePlatform_PS5': { name: 'PlayStation', icon: <FaPlaystation />, color: '#003791' },
    'OnlinePlatform_Dingo': { name: 'Xbox', icon: <FaXbox />, color: '#107c10' },
    'OnlinePlatform_Switch': { name: 'Switch', icon: null, color: '#e60012' },
};

export const PlayerTooltip: React.FC<PlayerTooltipProps> = ({
    playerName,
    team,
    carName,
    hitboxType,
    platform,
    goals = 0,
    assists = 0,
    saves = 0,
    shots = 0,
    matchScore = 0,
    currentScore,
    isBot = false,
    position,
    playerStatsTimelines,
    currentTime,
}) => {
    const hitboxColor = HITBOX_COLORS[hitboxType] || '#ffffff';
    const teamColor = team === 0 ? '#3399ff' : '#ff6600';
    const platformInfo = platform ? PLATFORM_INFO[platform] : null;

    // Get real-time stats from timeline if available
    const realtimeStats = useMemo(() => {
        if (!playerStatsTimelines || currentTime === undefined) return null;
        return getPlayerStatsAtTime(playerStatsTimelines, playerName, currentTime);
    }, [playerStatsTimelines, playerName, currentTime]);

    // Use real-time stats if available, otherwise fall back to static values
    const displayGoals = realtimeStats?.goals ?? goals;
    const displayAssists = realtimeStats?.assists ?? assists;
    const displaySaves = realtimeStats?.saves ?? saves;
    const displayShots = realtimeStats?.shots ?? shots;
    const displayScore = realtimeStats?.score ?? currentScore ?? matchScore;
    const displayDemos = realtimeStats?.demos ?? 0;
    const displayPing = realtimeStats?.ping;
    const hasRealtimeData = realtimeStats !== null;

    return (
        <div
            className="fixed z-[100] bg-gray-900/95 border border-gray-700 rounded-lg shadow-xl backdrop-blur-sm pointer-events-none"
            style={{
                left: position.x + 15,
                top: position.y - 10,
                minWidth: '180px',
            }}
        >
            {/* Header */}
            <div
                className="px-3 py-1.5 border-b border-gray-700 rounded-t-lg flex items-center justify-between gap-2"
                style={{ backgroundColor: `${teamColor}20` }}
            >
                <div className="flex items-center gap-2">
                    <span
                        className="font-semibold text-sm"
                        style={{ color: teamColor }}
                    >
                        {playerName}
                    </span>
                    {isBot && (
                        <Bot size={14} className="text-gray-400" />
                    )}
                </div>
                {platformInfo && (
                    <div
                        className="flex items-center gap-1.5 px-1.5 py-0.5 rounded text-xs"
                        style={{ backgroundColor: platformInfo.color }}
                        title={platformInfo.name}
                    >
                        {platformInfo.icon && (
                            <span className="text-white text-sm">{platformInfo.icon}</span>
                        )}
                    </div>
                )}
            </div>

            {/* Content */}
            <div className="px-3 py-2 space-y-2">
                {/* Car Info Section */}
                <div className="space-y-1">
                    {/* Car Name */}
                    <div className="flex items-center gap-2">
                        <span className="text-xs text-gray-400 w-12">Car</span>
                        <span className="text-white text-sm font-medium">{carName}</span>
                    </div>

                    {/* Hitbox Type */}
                    <div className="flex items-center gap-2">
                        <span className="text-xs text-gray-400 w-12">Hitbox</span>
                        <span
                            className="text-sm font-medium"
                            style={{ color: hitboxColor }}
                        >
                            {hitboxType}
                        </span>
                    </div>
                </div>

                {/* Stats Section */}
                <div className="border-t border-gray-700 pt-2">
                    {/* Current Score & Ping Row */}
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                            <span className="text-xs text-gray-400">Score</span>
                            <span className="text-white font-bold text-lg">{displayScore}</span>
                        </div>
                        {displayPing !== undefined && displayPing > 0 && (
                            <div className="flex items-center gap-1 text-xs" title={`Ping: ${displayPing}ms`}>
                                <Wifi size={10} className={displayPing > 100 ? 'text-red-400' : displayPing > 50 ? 'text-yellow-400' : 'text-green-400'} />
                                <span className={displayPing > 100 ? 'text-red-400' : displayPing > 50 ? 'text-yellow-400' : 'text-green-400'}>
                                    {displayPing}ms
                                </span>
                            </div>
                        )}
                    </div>

                    {/* Match Stats Grid - 5 columns with demos */}
                    <div className="grid grid-cols-5 gap-1">
                        <div className="flex flex-col items-center p-1 bg-gray-800/50 rounded" title="Goals">
                            <Trophy size={12} className="text-yellow-400 mb-0.5" />
                            <span className="text-white text-xs font-bold">{displayGoals}</span>
                        </div>
                        <div className="flex flex-col items-center p-1 bg-gray-800/50 rounded" title="Assists">
                            <Target size={12} className="text-green-400 mb-0.5" />
                            <span className="text-white text-xs font-bold">{displayAssists}</span>
                        </div>
                        <div className="flex flex-col items-center p-1 bg-gray-800/50 rounded" title="Saves">
                            <Shield size={12} className="text-blue-400 mb-0.5" />
                            <span className="text-white text-xs font-bold">{displaySaves}</span>
                        </div>
                        <div className="flex flex-col items-center p-1 bg-gray-800/50 rounded" title="Shots">
                            <Zap size={12} className="text-purple-400 mb-0.5" />
                            <span className="text-white text-xs font-bold">{displayShots}</span>
                        </div>
                        <div className="flex flex-col items-center p-1 bg-gray-800/50 rounded" title="Demolitions">
                            <Skull size={12} className="text-red-400 mb-0.5" />
                            <span className="text-white text-xs font-bold">{displayDemos}</span>
                        </div>
                    </div>

                    {/* Real-time indicator */}
                    {hasRealtimeData && (
                        <div className="flex items-center justify-center mt-2 text-[10px] text-gray-500">
                            <span className="w-1.5 h-1.5 rounded-full bg-green-500 mr-1 animate-pulse"></span>
                            Real-time stats
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default PlayerTooltip;
