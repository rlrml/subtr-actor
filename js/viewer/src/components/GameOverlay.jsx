import React, { useState, useRef, useEffect } from 'react';
import { Camera, Eye, Crown, Video, Circle, User, MoreVertical, UserMinus, Ban, ChevronRight, Users, Radio } from 'lucide-react';
import { BoostGauge } from './BoostGauge';
import { RealtimePossession, RealtimeStatsPanel } from './stats';
import { PlayerTooltip } from './PlayerTooltip';
import { PlaybackControls } from './PlaybackControls';
import { QuickToolbar } from './QuickToolbar';


export function GameOverlay({
    // Mode: 'replay' (default) or 'live' (027-live-viewer)
    mode = 'replay',
    isPlaying = false,
    currentTime = 0,
    maxTime = 0,
    gameTimeMap = [],
    countdownEvents = [],
    onPlayPause = null, // Optional in live mode
    onSeek = null, // Optional in live mode
    onSeekCommit = null, // Called when user releases the slider (for collab sync)
    onEventClick = null, // Optional in live mode
    cameraMode,
    onCameraModeChange,
    players = [],
    playerTeams = {},
    selectedPlayer = null,
    onPlayerSelect = null,
    cameraSettings = null,
    onCameraSettingsChange = null,
    events = [],
    playerBoosts = {},
    playerScores = {},
    playbackSpeed = 1.0,
    onPlaybackSpeedChange = null,
    textOverlays = [],
    playerCarInfo = {}, // { playerName: { carName, hitboxType } }
    controlsDisabled = false, // Disable playback controls (for collab viewers)
    isInSession = false, // Whether user is in a collab session
    // Collab participants props
    participants = {}, // Record<string, Participant>
    selfId = null,
    hostId = null,
    isHost = false, // Whether current user is the host
    followingId = null,
    onFollowViewer = null, // (participantId: string | null) => void
    onKickParticipant = null, // (participantId: string) => void
    onBanParticipant = null, // (participantId: string) => void
    onTransferHost = null, // (participantId: string) => void
    onStartSession = null, // () => void - Start a new collab session
    // Environment props
    currentEnvironmentId = null,
    onEnvironmentChange = null,
    isLoadingEnvironment = false,
    // Real-time stats timeline
    playerStatsTimelines = null, // { [playerName]: PlayerStatsTimelineEntry[] }
    // Game event timeline (for overtime detection)
    gameEventTimeline = [], // GameEventTimelineEntry[]
    // Advanced stats timelines (018-stats-compiler)
    advancedStats = null, // { playerTimelines, teamTimelines, matchTimeline }
    // Quality indicator (016-replay-quality-indicator)
    qualityScore = null,
    qualityMetrics = null,
    // Clip system (024-clip-system)
    onCreateClip = null,
    isClipEditorOpen = false,
    // Live mode specific props (027-live-viewer)
    viewerCount = 0,
    broadcasterName = '',
    liveTitle = '', // Stream title
    liveGameInfo = null, // { timeRemaining, scoreBlue, scoreOrange, isOvertime, isRoundActive }
    // v9 protocol: Replay and podium states
    isInReplay = false,
    isOnPodium = false,
}) {
    // State for player tooltip on hover
    const [hoveredPlayer, setHoveredPlayer] = useState(null); // { playerName, team, x, y }
    // State for participant context menu
    const [openMenuId, setOpenMenuId] = useState(null); // participantId or null
    const menuRef = useRef(null);

    // Close menu when clicking outside
    useEffect(() => {
        const handleClickOutside = (e) => {
            if (menuRef.current && !menuRef.current.contains(e.target)) {
                setOpenMenuId(null);
            }
        };
        if (openMenuId) {
            document.addEventListener('mousedown', handleClickOutside);
            return () => document.removeEventListener('mousedown', handleClickOutside);
        }
    }, [openMenuId]);
    const formatTime = (time) => {
        const minutes = Math.floor(time / 60);
        const seconds = Math.floor(time % 60).toString().padStart(2, '0');
        return `${minutes}:${seconds}`;
    };

    // Get current countdown status
    const getCurrentCountdown = () => {
        for (const countdown of countdownEvents) {
            if (currentTime >= countdown.startTime && currentTime <= countdown.goTime) {
                const timeLeft = countdown.goTime - currentTime;
                if (timeLeft <= 0.05) return 'GO!';
                const countNumber = Math.ceil(timeLeft);
                return countNumber.toString();
            }
        }

        return null;
    };

    const countdownText = getCurrentCountdown();

    // Get current overtime status from game event timeline (binary search)
    const getCurrentOvertimeStatus = () => {
        if (!gameEventTimeline || gameEventTimeline.length === 0) return false;

        // Binary search for the most recent entry at or before currentTime
        let low = 0;
        let high = gameEventTimeline.length - 1;

        while (low < high) {
            const mid = Math.floor((low + high + 1) / 2);
            if (gameEventTimeline[mid].time <= currentTime) {
                low = mid;
            } else {
                high = mid - 1;
            }
        }

        return gameEventTimeline[low]?.isOvertime ?? false;
    };

    const isOvertime = getCurrentOvertimeStatus();

    // Calculate current score based on goals that happened before currentTime
    const calculateScore = () => {
        let blueScore = 0;
        let orangeScore = 0;

        events.forEach(event => {
            if (event.type === 'goal' && event.time <= currentTime) {
                if (event.team === 0) {
                    blueScore++;
                } else if (event.team === 1) {
                    orangeScore++;
                }
            }
        });

        return { blueScore, orangeScore };
    };

    const { blueScore, orangeScore } = calculateScore();

    // Separate players by team
    const bluePlayers = players.filter(p => playerTeams[p] === 0);
    const orangePlayers = players.filter(p => playerTeams[p] === 1);

    return (
        <div className="absolute left-0 right-0 top-16 bottom-0 pointer-events-none flex flex-col justify-between p-4">
            {/* Top Bar: Debug/Info or Header */}
            <div className="grid grid-cols-[1fr_auto_1fr] items-start gap-4">
                {/* Player Stats Panel - Top Left - RLCS Broadcast Style */}
                <div className="pointer-events-auto flex flex-col gap-1 justify-self-start">
                    {/* Section Title: Players */}
                    <div className="text-[10px] font-semibold text-gray-400 uppercase tracking-wider mb-1 px-1">
                        Players
                    </div>

                    {/* Blue Team */}
                    <div className="flex flex-col">
                        {bluePlayers.map((p, idx) => {
                            const boost = Math.round(playerBoosts[p] || 0);
                            const score = playerScores[p] || 0;
                            const isSelected = cameraMode === 'player' && selectedPlayer === p;
                            return (
                                <button
                                    key={p}
                                    onClick={() => {
                                        onCameraModeChange('player');
                                        onPlayerSelect(p);
                                    }}
                                    className={`flex items-center gap-0 transition-all duration-150 ${isSelected ? 'scale-105 z-10' : 'hover:scale-[1.02] hover:brightness-110'}`}
                                    style={{ marginBottom: idx < bluePlayers.length - 1 ? '2px' : '0' }}
                                    onMouseEnter={(e) => {
                                        setHoveredPlayer({
                                            playerName: p,
                                            team: 0,
                                            x: e.clientX,
                                            y: e.clientY,
                                        });
                                    }}
                                    onMouseMove={(e) => {
                                        if (hoveredPlayer?.playerName === p) {
                                            setHoveredPlayer({
                                                playerName: p,
                                                team: 0,
                                                x: e.clientX,
                                                y: e.clientY,
                                            });
                                        }
                                    }}
                                    onMouseLeave={() => setHoveredPlayer(null)}
                                >
                                    {/* Player info card with boost as background fill */}
                                    <div
                                        className={`relative flex items-center h-8 overflow-hidden border-l-2 shadow-lg shadow-blue-500/20 transition-all ${isSelected ? 'border-white ring-2 ring-white/50' : 'border-blue-400'}`}
                                    >
                                        {/* Dark background */}
                                        <div className="absolute inset-0 bg-gradient-to-r from-blue-950 to-blue-900" />

                                        {/* Boost fill */}
                                        <div
                                            className="absolute inset-y-0 left-0 bg-gradient-to-r from-blue-600 to-blue-500 transition-all duration-150"
                                            style={{ width: `${boost}%` }}
                                        />

                                        {/* Player name */}
                                        <div className="relative px-3 flex items-center min-w-[140px]">
                                            <span className="text-white font-semibold text-sm tracking-wide truncate drop-shadow-md">
                                                {p}
                                            </span>
                                        </div>

                                        {/* Score */}
                                        <div className="relative px-3 h-full flex items-center justify-center bg-black/30 min-w-[50px]">
                                            <span className="text-white font-mono font-bold text-sm">
                                                {score}
                                            </span>
                                        </div>
                                    </div>

                                    {/* Camera indicator */}
                                    {isSelected && (
                                        <div className="ml-2 flex items-center justify-center w-6 h-6 bg-white/20 rounded-full">
                                            <Camera size={14} className="text-white" />
                                        </div>
                                    )}
                                </button>
                            );
                        })}
                    </div>

                    {/* Spacer between teams */}
                    <div className="h-3"></div>

                    {/* Orange Team */}
                    <div className="flex flex-col">
                        {orangePlayers.map((p, idx) => {
                            const boost = Math.round(playerBoosts[p] || 0);
                            const score = playerScores[p] || 0;
                            const isSelected = cameraMode === 'player' && selectedPlayer === p;
                            return (
                                <button
                                    key={p}
                                    onClick={() => {
                                        onCameraModeChange('player');
                                        onPlayerSelect(p);
                                    }}
                                    className={`flex items-center gap-0 transition-all duration-150 ${isSelected ? 'scale-105 z-10' : 'hover:scale-[1.02] hover:brightness-110'}`}
                                    style={{ marginBottom: idx < orangePlayers.length - 1 ? '2px' : '0' }}
                                    onMouseEnter={(e) => {
                                        setHoveredPlayer({
                                            playerName: p,
                                            team: 1,
                                            x: e.clientX,
                                            y: e.clientY,
                                        });
                                    }}
                                    onMouseMove={(e) => {
                                        if (hoveredPlayer?.playerName === p) {
                                            setHoveredPlayer({
                                                playerName: p,
                                                team: 1,
                                                x: e.clientX,
                                                y: e.clientY,
                                            });
                                        }
                                    }}
                                    onMouseLeave={() => setHoveredPlayer(null)}
                                >
                                    {/* Player info card with boost as background fill */}
                                    <div
                                        className={`relative flex items-center h-8 overflow-hidden border-l-2 shadow-lg shadow-orange-500/20 transition-all ${isSelected ? 'border-white ring-2 ring-white/50' : 'border-orange-400'}`}
                                    >
                                        {/* Dark background */}
                                        <div className="absolute inset-0 bg-gradient-to-r from-orange-950 to-orange-900" />

                                        {/* Boost fill */}
                                        <div
                                            className="absolute inset-y-0 left-0 bg-gradient-to-r from-orange-600 to-orange-500 transition-all duration-150"
                                            style={{ width: `${boost}%` }}
                                        />

                                        {/* Player name */}
                                        <div className="relative px-3 flex items-center min-w-[140px]">
                                            <span className="text-white font-semibold text-sm tracking-wide truncate drop-shadow-md">
                                                {p}
                                            </span>
                                        </div>

                                        {/* Score */}
                                        <div className="relative px-3 h-full flex items-center justify-center bg-black/30 min-w-[50px]">
                                            <span className="text-white font-mono font-bold text-sm">
                                                {score}
                                            </span>
                                        </div>
                                    </div>

                                    {/* Camera indicator */}
                                    {isSelected && (
                                        <div className="ml-2 flex items-center justify-center w-6 h-6 bg-white/20 rounded-full">
                                            <Camera size={14} className="text-white" />
                                        </div>
                                    )}
                                </button>
                            );
                        })}
                    </div>

                    {/* Participants Section - Only in collab session */}
                    {isInSession && Object.keys(participants).length > 0 && (
                        <>
                            {/* Section Title: Participants */}
                            <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider mt-4 mb-1 px-1 flex items-center gap-1">
                                <Eye size={12} className="text-violet-400" />
                                Participants ({Object.keys(participants).length})
                            </div>

                            {/* Participants List */}
                            <div className="flex flex-col gap-0.5">
                                {Object.values(participants)
                                    .sort((a, b) => {
                                        // Self first
                                        if (a.id === selfId) return -1;
                                        if (b.id === selfId) return 1;
                                        // Then host
                                        if (a.id === hostId) return -1;
                                        if (b.id === hostId) return 1;
                                        // Then by join time
                                        return a.joinedAt - b.joinedAt;
                                    })
                                    .map((participant) => {
                                        const isSelf = participant.id === selfId;
                                        const isParticipantHost = participant.id === hostId;
                                        const isFollowingThisParticipant = followingId === participant.id;
                                        // Check if this participant is following me (to prevent follow loops)
                                        const isFollowingMe = participant.followingId === selfId;
                                        // Check if this participant is following someone else
                                        const participantFollowingId = participant.followingId;
                                        const participantFollowingNickname = participantFollowingId && participants[participantFollowingId]?.nickname;
                                        // Can't follow yourself, can't follow someone who is following you (loop prevention)
                                        const canFollow = !isSelf && !isFollowingMe && onFollowViewer;
                                        const isMenuOpen = openMenuId === participant.id;

                                        // Camera mode icon and label with target player
                                        const getCameraModeInfo = (mode, targetPlayer) => {
                                            switch (mode) {
                                                case 'free':
                                                    return { icon: Video, label: 'Free', detail: null };
                                                case 'ballOrbit':
                                                    return { icon: Circle, label: 'Ball', detail: null };
                                                case 'player':
                                                    return { icon: User, label: 'Player', detail: targetPlayer };
                                                default:
                                                    return { icon: Video, label: '?', detail: null };
                                            }
                                        };

                                        const cameraModeInfo = getCameraModeInfo(participant.camera?.mode, participant.camera?.targetPlayer);
                                        const CameraModeIcon = cameraModeInfo.icon;

                                        return (
                                            <div
                                                key={participant.id}
                                                className={`relative flex items-center h-10 overflow-visible transition-all group ${
                                                    isFollowingThisParticipant
                                                        ? 'ring-2 ring-violet-500/70 scale-105 z-20'
                                                        : 'z-10'
                                                } ${isSelf ? 'opacity-80' : ''} ${isFollowingMe ? 'opacity-50' : ''}`}
                                                style={{ borderLeft: `3px solid ${participant.color}` }}
                                            >
                                                {/* Background */}
                                                <div className={`absolute inset-0 ${isFollowingThisParticipant ? 'bg-gradient-to-r from-violet-950 to-violet-900' : participantFollowingId ? 'bg-gradient-to-r from-gray-800 to-gray-700' : 'bg-gradient-to-r from-gray-900 to-gray-800'}`} />

                                                {/* Main clickable area for follow */}
                                                <button
                                                    onClick={() => {
                                                        if (canFollow) {
                                                            if (isFollowingThisParticipant) {
                                                                onFollowViewer(null);
                                                            } else {
                                                                onFollowViewer(participant.id);
                                                            }
                                                        }
                                                    }}
                                                    disabled={!canFollow}
                                                    className={`relative flex items-center gap-2 px-2 flex-1 min-w-0 h-full ${canFollow ? 'hover:brightness-110 cursor-pointer' : 'cursor-default'}`}
                                                    title={
                                                        isSelf
                                                            ? 'You'
                                                            : isFollowingMe
                                                                ? `${participant.nickname} is following you`
                                                                : isFollowingThisParticipant
                                                                    ? 'Click to stop following'
                                                                    : `Click to follow ${participant.nickname}`
                                                    }
                                                >
                                                    {/* Color dot */}
                                                    <div
                                                        className="w-2.5 h-2.5 rounded-full flex-shrink-0"
                                                        style={{ backgroundColor: participant.color }}
                                                    />

                                                    {/* Nickname + indicators */}
                                                    <div className="flex flex-col min-w-0">
                                                        <div className="flex items-center gap-1">
                                                            <span className="text-white text-sm font-medium truncate max-w-[80px]">
                                                                {participant.nickname}
                                                            </span>
                                                            {isSelf && (
                                                                <span className="text-[10px] text-gray-500">(You)</span>
                                                            )}
                                                            {isParticipantHost && (
                                                                <Crown size={12} className="text-yellow-500 flex-shrink-0" />
                                                            )}
                                                        </div>
                                                        {/* Sub-info: following status */}
                                                        {participantFollowingNickname && !isSelf && (
                                                            <span className="text-[10px] text-violet-400 flex items-center gap-0.5 -mt-0.5">
                                                                <Eye size={9} />
                                                                {participantFollowingId === selfId ? 'you' : participantFollowingNickname}
                                                            </span>
                                                        )}
                                                    </div>
                                                </button>

                                                {/* Camera mode indicator with detail */}
                                                <div className="relative flex items-center gap-1 px-2 h-full bg-black/20" title={cameraModeInfo.detail ? `Watching: ${cameraModeInfo.detail}` : cameraModeInfo.label}>
                                                    <CameraModeIcon size={12} className="text-gray-400" />
                                                    <div className="flex flex-col">
                                                        <span className="text-[11px] text-gray-400 font-medium leading-tight">
                                                            {cameraModeInfo.label}
                                                        </span>
                                                        {cameraModeInfo.detail && (
                                                            <span className="text-[9px] text-gray-500 leading-tight truncate max-w-[50px]">
                                                                {cameraModeInfo.detail}
                                                            </span>
                                                        )}
                                                    </div>
                                                </div>

                                                {/* Follow indicator (when I'm following this participant) */}
                                                {isFollowingThisParticipant && (
                                                    <div className="relative flex items-center justify-center px-2 h-full bg-violet-600">
                                                        <Eye size={14} className="text-white" />
                                                    </div>
                                                )}

                                                {/* Context menu button (not for self) */}
                                                {!isSelf && (isHost || canFollow) && (
                                                    <button
                                                        onClick={(e) => {
                                                            e.stopPropagation();
                                                            setOpenMenuId(isMenuOpen ? null : participant.id);
                                                        }}
                                                        className="relative flex items-center justify-center w-7 h-full bg-black/30 hover:bg-black/50 transition-colors"
                                                    >
                                                        <MoreVertical size={14} className="text-gray-400" />
                                                    </button>
                                                )}

                                                {/* Context menu dropdown */}
                                                {isMenuOpen && (
                                                    <div
                                                        ref={menuRef}
                                                        className="absolute right-0 top-full mt-1 bg-gray-900 border border-gray-700 rounded-lg shadow-xl z-50 min-w-[160px] py-1 overflow-hidden"
                                                    >
                                                        {/* Follow/Unfollow */}
                                                        {canFollow && (
                                                            <button
                                                                onClick={() => {
                                                                    if (isFollowingThisParticipant) {
                                                                        onFollowViewer(null);
                                                                    } else {
                                                                        onFollowViewer(participant.id);
                                                                    }
                                                                    setOpenMenuId(null);
                                                                }}
                                                                className="w-full flex items-center gap-2 px-3 py-2 text-xs text-left hover:bg-gray-800 transition-colors"
                                                            >
                                                                <Eye size={14} className="text-violet-400" />
                                                                <span>{isFollowingThisParticipant ? 'Stop following' : 'Follow camera'}</span>
                                                            </button>
                                                        )}

                                                        {/* Host actions */}
                                                        {isHost && !isParticipantHost && (
                                                            <>
                                                                {canFollow && <div className="border-t border-gray-700 my-1" />}

                                                                {/* Promote to host */}
                                                                {onTransferHost && (
                                                                    <button
                                                                        onClick={() => {
                                                                            onTransferHost(participant.id);
                                                                            setOpenMenuId(null);
                                                                        }}
                                                                        className="w-full flex items-center gap-2 px-3 py-2 text-xs text-left hover:bg-gray-800 transition-colors"
                                                                    >
                                                                        <Crown size={14} className="text-yellow-500" />
                                                                        <span>Promote to host</span>
                                                                    </button>
                                                                )}

                                                                {/* Kick */}
                                                                {onKickParticipant && (
                                                                    <button
                                                                        onClick={() => {
                                                                            onKickParticipant(participant.id);
                                                                            setOpenMenuId(null);
                                                                        }}
                                                                        className="w-full flex items-center gap-2 px-3 py-2 text-xs text-left hover:bg-gray-800 transition-colors text-orange-400"
                                                                    >
                                                                        <UserMinus size={14} />
                                                                        <span>Kick</span>
                                                                    </button>
                                                                )}

                                                                {/* Ban */}
                                                                {onBanParticipant && (
                                                                    <button
                                                                        onClick={() => {
                                                                            onBanParticipant(participant.id);
                                                                            setOpenMenuId(null);
                                                                        }}
                                                                        className="w-full flex items-center gap-2 px-3 py-2 text-xs text-left hover:bg-gray-800 transition-colors text-red-400"
                                                                    >
                                                                        <Ban size={14} />
                                                                        <span>Ban from session</span>
                                                                    </button>
                                                                )}
                                                            </>
                                                        )}
                                                    </div>
                                                )}
                                            </div>
                                        );
                                    })}
                            </div>
                        </>
                    )}
                </div>

                {/* Rocket League Style Score Display + Watch Together CTA (replay) or Live Info (live) */}
                <div className="flex flex-col items-center gap-2 justify-self-center">
                    {/* Live Mode: LIVE Badge + Status Badges + Broadcaster Info */}
                    {mode === 'live' && (
                        <div className="flex items-center gap-3 mb-1">
                            <div className="flex items-center gap-2">
                                <div className="flex items-center gap-2 bg-red-600 text-white px-3 py-1 rounded-full text-sm font-semibold animate-pulse">
                                    <Radio size={14} />
                                    <span>LIVE</span>
                                </div>
                                {/* v9: REPLAY Badge */}
                                {isInReplay && (
                                    <div className="flex items-center gap-2 bg-purple-600 text-white px-3 py-1 rounded-full text-sm font-semibold">
                                        <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                                            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                                            <path d="M3 3v5h5" />
                                            <path d="M12 7v5l4 2" />
                                        </svg>
                                        <span>REPLAY</span>
                                        {liveGameInfo?.timeDilation !== undefined && liveGameInfo.timeDilation < 0.95 && (
                                            <span className="text-xs opacity-75">
                                                {(liveGameInfo.timeDilation * 100).toFixed(0)}%
                                            </span>
                                        )}
                                    </div>
                                )}
                                {/* v9: PODIUM Badge */}
                                {isOnPodium && (
                                    <div className="flex items-center gap-2 bg-yellow-600 text-white px-3 py-1 rounded-full text-sm font-semibold">
                                        <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M5 16L3 5l5.5 5L12 4l3.5 6L21 5l-2 11H5z" />
                                            <path d="M5 16v4h14v-4" />
                                        </svg>
                                        <span>PODIUM</span>
                                    </div>
                                )}
                            </div>
                            {broadcasterName && (
                                <div className="bg-black/50 backdrop-blur-sm rounded-lg px-3 py-1">
                                    {liveTitle && (
                                        <div className="text-white text-sm font-semibold mb-0.5 max-w-[200px] truncate" title={liveTitle}>
                                            {liveTitle}
                                        </div>
                                    )}
                                    <div className="flex items-center gap-2">
                                        <span className="text-gray-300 text-xs font-medium">{broadcasterName}</span>
                                        <span className="text-gray-500">•</span>
                                        <div className="flex items-center gap-1 text-gray-400 text-xs">
                                            <Users size={12} />
                                            <span>{viewerCount}</span>
                                        </div>
                                    </div>
                                </div>
                            )}
                        </div>
                    )}

                    {/* Score Display with integrated possession bar */}
                    <div className="pointer-events-none flex flex-col">
                        <div className="flex items-center">
                            {/* Blue Team Score Box */}
                            <div className={`relative flex items-center justify-center w-16 h-14 bg-gradient-to-b from-blue-500 to-blue-700 border-2 border-blue-300/50 shadow-[0_0_15px_rgba(59,130,246,0.5),inset_0_0_20px_rgba(255,255,255,0.1)] ${(mode === 'replay' && advancedStats?.teamTimelines) ? '' : 'rounded-l-md'}`}>
                                <span className="text-white text-3xl font-bold drop-shadow-[0_2px_2px_rgba(0,0,0,0.8)]">
                                    {mode === 'live' ? (liveGameInfo?.scoreBlue ?? 0) : blueScore}
                                </span>
                            </div>

                            {/* Timer Box */}
                            <div className={`relative flex flex-col items-center justify-center px-4 h-14 border-y-2 shadow-[0_0_15px_rgba(0,0,0,0.5),inset_0_0_20px_rgba(255,255,255,0.05)] ${
                                (mode === 'live' ? liveGameInfo?.isOvertime : isOvertime)
                                    ? 'bg-gradient-to-b from-red-600 to-red-800 border-red-400/50'
                                    : 'bg-gradient-to-b from-gray-800 to-gray-900 border-gray-600/50'
                            }`}>
                                {(mode === 'live' ? liveGameInfo?.isOvertime : isOvertime) && (
                                    <span className="text-red-200 text-[10px] font-bold uppercase tracking-widest drop-shadow-[0_1px_1px_rgba(0,0,0,0.8)] -mt-0.5">
                                        Overtime
                                    </span>
                                )}
                                <span className={`font-bold font-mono tracking-wider drop-shadow-[0_2px_2px_rgba(0,0,0,0.8)] ${
                                    (mode === 'live' ? liveGameInfo?.isOvertime : isOvertime) ? 'text-white text-2xl' : 'text-white text-3xl'
                                }`}>
                                    {(() => {
                                        // Live mode: use liveGameInfo directly
                                        if (mode === 'live') {
                                            if (!liveGameInfo) return formatTime(300);
                                            const gameTime = Math.floor(liveGameInfo.timeRemaining);
                                            if (liveGameInfo.isOvertime) {
                                                return `+${formatTime(Math.max(0, gameTime))}`;
                                            }
                                            return formatTime(Math.max(0, gameTime));
                                        }

                                        // Replay mode: use gameTimeMap
                                        if (gameTimeMap.length === 0) {
                                            return formatTime(300); // Default 5:00
                                        }

                                        // Find the closest entry in gameTimeMap for current replay time
                                        let closestEntry = gameTimeMap[0];
                                        for (let i = 0; i < gameTimeMap.length; i++) {
                                            if (gameTimeMap[i].replayTime <= currentTime) {
                                                closestEntry = gameTimeMap[i];
                                            } else {
                                                break;
                                            }
                                        }

                                        const gameTime = Math.floor(closestEntry.gameTimeRemaining);

                                        // In overtime, SecondsRemaining increments from 0
                                        // So gameTimeRemaining represents the overtime elapsed time directly
                                        if (isOvertime) {
                                            return `+${formatTime(Math.max(0, gameTime))}`;
                                        }

                                        return formatTime(Math.max(0, gameTime));
                                    })()}
                                </span>
                            </div>

                            {/* Orange Team Score Box */}
                            <div className={`relative flex items-center justify-center w-16 h-14 bg-gradient-to-b from-orange-500 to-orange-700 border-2 border-orange-300/50 shadow-[0_0_15px_rgba(249,115,22,0.5),inset_0_0_20px_rgba(255,255,255,0.1)] ${(mode === 'replay' && advancedStats?.teamTimelines) ? '' : 'rounded-r-md'}`}>
                                <span className="text-white text-3xl font-bold drop-shadow-[0_2px_2px_rgba(0,0,0,0.8)]">
                                    {mode === 'live' ? (liveGameInfo?.scoreOrange ?? 0) : orangeScore}
                                </span>
                            </div>
                        </div>

                        {/* Integrated Possession Bar (018-stats-compiler) - Only in replay mode */}
                        {mode === 'replay' && advancedStats?.teamTimelines && (
                            <div className="h-4 bg-gray-900/90 rounded-b-md border-x-2 border-b-2 border-gray-700/50 shadow-[0_4px_10px_rgba(0,0,0,0.4)] flex items-center">
                                <RealtimePossession
                                    teamTimelines={advancedStats.teamTimelines}
                                    currentTime={currentTime}
                                    inline
                                />
                            </div>
                        )}
                    </div>

                    {/* Watch Together Button - Only in replay mode when not in session */}
                    {mode === 'replay' && !isInSession && onStartSession && (
                        <button
                            onClick={onStartSession}
                            className="pointer-events-auto group relative flex items-center justify-center gap-2 w-full h-10 bg-gradient-to-b from-violet-500 to-violet-700 hover:from-violet-400 hover:to-violet-600 text-white text-sm font-semibold rounded-md transition-all shadow-lg shadow-violet-500/40 hover:shadow-violet-500/60 hover:scale-105 border border-violet-400/30 overflow-hidden"
                        >
                            {/* Continuous animated shine effect */}
                            <div
                                className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent"
                                style={{ animation: 'shine-sweep 3s ease-in-out infinite' }}
                            />
                            <Users size={18} className="relative z-10" />
                            <span className="relative z-10">Watch Together</span>
                        </button>
                    )}
                </div>

                {/* Text Overlays - Countdown & Goal Display */}
                {textOverlays.length > 0 && (
                    <div className="absolute inset-0 flex items-center justify-center pointer-events-none z-50">
                        <div className="flex flex-col items-center gap-2">
                            {textOverlays.map((overlay, idx) => (
                                <div key={`${overlay.type}-${idx}`} className="flex flex-col items-center">
                                    {overlay.type === 'countdown' && (
                                        <div
                                            className="text-[6rem] animate-pulse"
                                            style={{
                                                fontFamily: 'Bourgeois, sans-serif',
                                                color: '#ffcb58',
                                                textShadow: '0 6px 25px rgba(0, 0, 0, 0.35), 0 0 4px #ffcb58, 0 0 8px #ffcb58, 0 0 15px #ffcb58',
                                            }}
                                        >
                                            {overlay.text}
                                        </div>
                                    )}
                                    {overlay.type === 'goal' && (
                                        <div className="flex flex-col items-center">
                                            <div
                                                className="text-[6rem]"
                                                style={{
                                                    fontFamily: 'Bourgeois, sans-serif',
                                                    color: '#ffcb58',
                                                    textShadow: '0 6px 25px rgba(0, 0, 0, 0.35), 0 0 4px #ffcb58, 0 0 8px #ffcb58, 0 0 15px #ffcb58',
                                                }}
                                            >
                                                {overlay.text}
                                            </div>
                                        </div>
                                    )}
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {/* Fallback: Old countdown system if no textOverlays */}
                {textOverlays.length === 0 && countdownText && (
                    <div className="absolute inset-0 flex items-center justify-center pointer-events-none z-50">
                        <div
                            className="text-[6rem] animate-pulse"
                            style={{
                                fontFamily: 'Bourgeois, sans-serif',
                                color: '#ffcb58',
                                textShadow: '0 6px 25px rgba(0, 0, 0, 0.35), 0 0 4px #ffcb58, 0 0 8px #ffcb58, 0 0 15px #ffcb58',
                            }}
                        >
                            {countdownText}
                        </div>
                    </div>
                )}

                {/* Quick Toolbar - Top Right: Environment, Camera Mode, Keyboard Shortcuts, Quality */}
                <div className="pointer-events-auto justify-self-end">
                    <QuickToolbar
                        cameraMode={cameraMode}
                        onCameraModeChange={onCameraModeChange}
                        currentEnvironmentId={mode === 'replay' ? currentEnvironmentId : null}
                        onEnvironmentChange={mode === 'replay' ? onEnvironmentChange : null}
                        isLoadingEnvironment={mode === 'replay' ? isLoadingEnvironment : false}
                        hideEnvironment={mode === 'live'}
                        isInSession={mode === 'replay' ? isInSession : false}
                        isHost={mode === 'replay' ? isHost : false}
                        qualityScore={mode === 'replay' ? qualityScore : null}
                        onCreateClip={mode === 'replay' ? onCreateClip : null}
                        isClipEditorOpen={mode === 'replay' ? isClipEditorOpen : false}
                    />
                </div>
            </div>

            {/* Real-time Stats Panel - Bottom Left - Only in PlayerCam mode (018-stats-compiler) - Replay mode only */}
            {mode === 'replay' && cameraMode === 'player' && selectedPlayer && advancedStats && (
                <div className="absolute bottom-28 left-8 pointer-events-none">
                    <RealtimeStatsPanel
                        advancedStats={advancedStats}
                        currentTime={currentTime}
                        playerName={selectedPlayer}
                        playerTeam={playerTeams[selectedPlayer] || 0}
                    />
                </div>
            )}

            {/* Boost Gauge - Bottom Right - Only in PlayerCam mode */}
            {cameraMode === 'player' && selectedPlayer && (
                <div className="absolute bottom-28 right-8 pointer-events-none">
                    <BoostGauge
                        boost={playerBoosts[selectedPlayer] || 0}
                        size={220}
                        team={playerTeams[selectedPlayer] || 0}
                    />
                </div>
            )}

            {/* Bottom Bar: Playback Controls (replay mode only) or ESC hint (live mode) */}
            {mode === 'replay' ? (
                <div className="pointer-events-auto relative z-10">
                    <PlaybackControls
                        isPlaying={isPlaying}
                        currentTime={currentTime}
                        maxTime={maxTime}
                        onPlayPause={onPlayPause}
                        onSeek={onSeek}
                        onSeekCommit={onSeekCommit}
                        onEventClick={onEventClick}
                        playbackSpeed={playbackSpeed}
                        onPlaybackSpeedChange={onPlaybackSpeedChange}
                        events={events}
                        controlsDisabled={controlsDisabled}
                    />
                </div>
            ) : (
                <div className="pointer-events-none flex justify-center">
                    <div className="text-gray-500 text-xs">
                        Press <kbd className="px-1 py-0.5 bg-gray-700 rounded text-gray-300">ESC</kbd> to release mouse
                    </div>
                </div>
            )}

            {/* Player Info Tooltip */}
            {hoveredPlayer && (
                <PlayerTooltip
                    playerName={hoveredPlayer.playerName}
                    team={hoveredPlayer.team}
                    carName={playerCarInfo[hoveredPlayer.playerName]?.carName || 'Unknown'}
                    hitboxType={playerCarInfo[hoveredPlayer.playerName]?.hitboxType || 'Octane'}
                    platform={playerCarInfo[hoveredPlayer.playerName]?.platform}
                    goals={playerCarInfo[hoveredPlayer.playerName]?.goals}
                    assists={playerCarInfo[hoveredPlayer.playerName]?.assists}
                    saves={playerCarInfo[hoveredPlayer.playerName]?.saves}
                    shots={playerCarInfo[hoveredPlayer.playerName]?.shots}
                    matchScore={playerCarInfo[hoveredPlayer.playerName]?.matchScore}
                    currentScore={playerScores[hoveredPlayer.playerName]}
                    isBot={playerCarInfo[hoveredPlayer.playerName]?.isBot}
                    position={{ x: hoveredPlayer.x, y: hoveredPlayer.y }}
                    playerStatsTimelines={playerStatsTimelines}
                    currentTime={currentTime}
                />
            )}
        </div>
    );
}
