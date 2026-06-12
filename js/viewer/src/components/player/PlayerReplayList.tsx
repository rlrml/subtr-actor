/**
 * PlayerReplayList - Display list of replays for a player
 * (018-stats-compiler)
 */

import { Link } from 'react-router-dom';
import { Play, Trophy, Zap, Shield, Target, Clock, MapPin, Gauge, AlertTriangle } from 'lucide-react';

interface PlayerReplay {
  replayId: string;
  playedAt?: string;
  mapName?: string;
  team: number;
  goals: number;
  assists: number;
  saves: number;
  shots: number;
  score: number;
  avgSpeed?: number | null;
}

interface PlayerReplayListProps {
  replays: PlayerReplay[];
  loading?: boolean;
  /** List of replay IDs where this player was flagged as cheating (032-cheat-detection) */
  flaggedReplayIds?: string[];
}

function formatDate(dateStr?: string): string {
  if (!dateStr) return 'Unknown date';
  return new Date(dateStr).toLocaleDateString('en-US', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  });
}

function getMapDisplayName(mapName?: string): string {
  if (!mapName) return 'Unknown map';
  return mapName.replace(/_P$/, '').replace(/_Standard$/, '').replace(/_/g, ' ');
}

function ReplayRow({ replay, isFlagged }: { replay: PlayerReplay; isFlagged?: boolean }) {
  const teamColor = replay.team === 0 ? 'blue' : 'orange';
  // Use red styling if flagged as cheater (032-cheat-detection)
  const bgColor = isFlagged
    ? 'bg-red-500/5 border-red-500/30 hover:bg-red-500/10'
    : teamColor === 'blue'
      ? 'bg-blue-500/5 border-blue-500/20 hover:bg-blue-500/10'
      : 'bg-orange-500/5 border-orange-500/20 hover:bg-orange-500/10';

  return (
    <Link
      to={`/replays/${replay.replayId}`}
      className={`block p-4 rounded-xl border ${bgColor} transition-colors`}
    >
      <div className="flex items-center justify-between gap-4">
        {/* Left: Match info */}
        <div className="flex items-center gap-4 min-w-0 flex-1">
          {/* Play icon or warning icon for flagged replays */}
          <div className={`w-10 h-10 rounded-lg flex items-center justify-center shrink-0 ${
            isFlagged
              ? 'bg-red-500/20'
              : teamColor === 'blue' ? 'bg-blue-500/20' : 'bg-orange-500/20'
          }`}>
            {isFlagged ? (
              <AlertTriangle className="w-5 h-5 text-red-400" />
            ) : (
              <Play className={`w-5 h-5 ${teamColor === 'blue' ? 'text-blue-400' : 'text-orange-400'}`} />
            )}
          </div>

          {/* Map and date */}
          <div className="min-w-0">
            <div className="flex items-center gap-2 text-sm text-gray-300">
              <MapPin className="w-4 h-4 text-gray-500 shrink-0" />
              <span className="truncate">{getMapDisplayName(replay.mapName)}</span>
              {/* CHEATED badge (032-cheat-detection) */}
              {isFlagged && (
                <span className="ml-2 px-2 py-0.5 rounded text-[10px] font-bold uppercase bg-red-500/30 text-red-400 border border-red-500/30">
                  Cheated
                </span>
              )}
            </div>
            <div className="flex items-center gap-2 text-xs text-gray-500 mt-1">
              <Clock className="w-3 h-3" />
              <span>{formatDate(replay.playedAt)}</span>
            </div>
          </div>
        </div>

        {/* Right: Stats */}
        <div className="flex items-center gap-4 shrink-0">
          {/* Basic stats */}
          <div className="hidden sm:flex items-center gap-3 text-sm">
            <span className="flex items-center gap-1 text-yellow-400" title="Goals">
              <Trophy className="w-4 h-4" />
              {replay.goals}
            </span>
            <span className="flex items-center gap-1 text-cyan-400" title="Assists">
              <Zap className="w-4 h-4" />
              {replay.assists}
            </span>
            <span className="flex items-center gap-1 text-green-400" title="Saves">
              <Shield className="w-4 h-4" />
              {replay.saves}
            </span>
            <span className="flex items-center gap-1 text-violet-400" title="Shots">
              <Target className="w-4 h-4" />
              {replay.shots}
            </span>
          </div>

          {/* Speed (if available) */}
          {replay.avgSpeed != null && (
            <div className="hidden md:flex items-center gap-1 text-sm text-blue-400" title="Avg Speed">
              <Gauge className="w-4 h-4" />
              {replay.avgSpeed.toFixed(0)}
            </div>
          )}

          {/* Score */}
          <div className={`px-3 py-1.5 rounded-lg font-bold text-sm ${
            teamColor === 'blue' ? 'bg-blue-500/20 text-blue-300' : 'bg-orange-500/20 text-orange-300'
          }`}>
            {replay.score} pts
          </div>
        </div>
      </div>

      {/* Mobile stats row */}
      <div className="flex sm:hidden items-center gap-4 mt-3 text-xs text-gray-400">
        <span className="flex items-center gap-1">
          <Trophy className="w-3 h-3 text-yellow-400" />
          {replay.goals}
        </span>
        <span className="flex items-center gap-1">
          <Zap className="w-3 h-3 text-cyan-400" />
          {replay.assists}
        </span>
        <span className="flex items-center gap-1">
          <Shield className="w-3 h-3 text-green-400" />
          {replay.saves}
        </span>
        <span className="flex items-center gap-1">
          <Target className="w-3 h-3 text-violet-400" />
          {replay.shots}
        </span>
      </div>
    </Link>
  );
}

export function PlayerReplayList({ replays, loading, flaggedReplayIds = [] }: PlayerReplayListProps) {
  // Convert to Set for O(1) lookup (032-cheat-detection)
  const flaggedSet = new Set(flaggedReplayIds);

  if (loading) {
    return (
      <div className="space-y-3">
        {[1, 2, 3].map((i) => (
          <div
            key={i}
            className="h-20 rounded-xl bg-gray-800/50 border border-gray-700/50 animate-pulse"
          />
        ))}
      </div>
    );
  }

  if (replays.length === 0) {
    return (
      <div className="text-center py-12 text-gray-500">
        <Play className="w-12 h-12 mx-auto mb-3 opacity-30" />
        <p>No replays found for this player</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-white">Recent Matches</h3>
        <span className="text-sm text-gray-500">{replays.length} replays</span>
      </div>

      <div className="space-y-2">
        {replays.map((replay) => (
          <ReplayRow
            key={replay.replayId}
            replay={replay}
            isFlagged={flaggedSet.has(replay.replayId)}
          />
        ))}
      </div>
    </div>
  );
}
