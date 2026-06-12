/**
 * RealtimeStatsPanel - Real-time stats display during playback
 * (018-stats-compiler)
 */

import { useMemo } from 'react';
import { Gauge, Fuel, Plane } from 'lucide-react';

interface AdvancedStatsTimelineEntry {
  time: number;
  frame: number;
  currentSpeed: number;
  currentBoostAmount: number;
  isBoosting: boolean;
  isAirborne: boolean;
  avgSpeedSoFar: number;
  boostConsumedSoFar: number;
  boostPickupsSoFar: number;
  airTimeSecondsSoFar: number;
  offensiveTimeSoFar: number;
  defensiveTimeSoFar: number;
}

interface TeamStatsTimelineEntry {
  time: number;
  possessionPercentage: number;
  avgTeamSpeed: number;
  totalBoostPickups: number;
}

interface MatchStatsTimelineEntry {
  time: number;
  ballSpeed: number;
  avgBallSpeedSoFar: number;
}

export interface AdvancedStatsData {
  playerTimelines: Record<string, AdvancedStatsTimelineEntry[]>;
  teamTimelines: Record<number, TeamStatsTimelineEntry[]>;
  matchTimeline: MatchStatsTimelineEntry[];
}

interface RealtimeStatsPanelProps {
  advancedStats: AdvancedStatsData | null;
  currentTime: number;
  playerName?: string | null;
  playerTeam?: number;
  compact?: boolean;
}

// Binary search to find entry at or before given time
function findAtTime<T extends { time: number }>(timeline: T[], targetTime: number): T | null {
  if (!timeline || timeline.length === 0) return null;
  if (targetTime <= timeline[0].time) return timeline[0];
  if (targetTime >= timeline[timeline.length - 1].time) return timeline[timeline.length - 1];

  let low = 0;
  let high = timeline.length - 1;

  while (low < high) {
    const mid = Math.floor((low + high + 1) / 2);
    if (timeline[mid].time <= targetTime) {
      low = mid;
    } else {
      high = mid - 1;
    }
  }

  return timeline[low];
}

interface StatRowProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  color: string;
}

function StatRow({ icon, label, value, color }: StatRowProps) {
  return (
    <div className="flex items-center gap-2 text-xs">
      <span className={color}>{icon}</span>
      <span className="text-gray-500">{label}:</span>
      <span className="text-gray-300 font-mono">{value}</span>
    </div>
  );
}

export function RealtimeStatsPanel({
  advancedStats,
  currentTime,
  playerName,
  playerTeam = 0,
  compact = false,
}: RealtimeStatsPanelProps) {
  const stats = useMemo(() => {
    if (!advancedStats) return null;

    // Get match stats
    const matchEntry = findAtTime(advancedStats.matchTimeline, currentTime);

    // Get team stats
    const teamEntry = findAtTime(advancedStats.teamTimelines[playerTeam] || [], currentTime);

    // Get player stats if a player is selected
    let playerEntry: AdvancedStatsTimelineEntry | null = null;
    if (playerName && advancedStats.playerTimelines[playerName]) {
      playerEntry = findAtTime(advancedStats.playerTimelines[playerName], currentTime);
    }

    return {
      match: matchEntry,
      team: teamEntry,
      player: playerEntry,
    };
  }, [advancedStats, currentTime, playerName, playerTeam]);

  if (!advancedStats || !stats) {
    return null;
  }

  // Compact mode - just show current player stats in a small bar
  if (compact && stats.player) {
    return (
      <div className="flex items-center gap-3 px-3 py-1.5 rounded bg-black/60 text-xs">
        <div className="flex items-center gap-1">
          <Gauge className="w-3 h-3 text-blue-400" />
          <span className="text-gray-300 font-mono">{stats.player.currentSpeed.toFixed(0)}</span>
          <span className="text-gray-500">km/h</span>
        </div>
        {stats.player.isAirborne && (
          <div className="flex items-center gap-1 text-cyan-400">
            <Plane className="w-3 h-3" />
            <span>AIR</span>
          </div>
        )}
        <div className="flex items-center gap-1">
          <Fuel className="w-3 h-3 text-yellow-400" />
          <span className="text-gray-300 font-mono">{stats.player.boostConsumedSoFar.toFixed(0)}</span>
        </div>
      </div>
    );
  }

  // Full panel
  return (
    <div className="bg-black/60 backdrop-blur-sm rounded-lg p-3 space-y-2">
      {/* Match stats */}
      {stats.match && (
        <div className="flex items-center justify-center gap-4 text-xs text-center pb-2 border-b border-gray-700/50">
          <div>
            <div className="text-gray-300 font-mono">{stats.match.ballSpeed.toFixed(1)}</div>
            <div className="text-gray-500 text-[10px] uppercase">Ball Speed</div>
          </div>
          <div className="w-px h-6 bg-gray-700" />
          <div>
            <div className="text-gray-300 font-mono">{stats.match.avgBallSpeedSoFar.toFixed(1)}</div>
            <div className="text-gray-500 text-[10px] uppercase">Avg Ball</div>
          </div>
        </div>
      )}

      {/* Player stats */}
      {stats.player && playerName && (
        <div className="space-y-1">
          <div className="text-xs text-gray-400 font-medium truncate">{playerName}</div>
          <StatRow
            icon={<Gauge className="w-3 h-3" />}
            label="Speed"
            value={`${stats.player.currentSpeed.toFixed(0)} km/h`}
            color="text-blue-400"
          />
          <StatRow
            icon={<Gauge className="w-3 h-3" />}
            label="Avg"
            value={`${stats.player.avgSpeedSoFar.toFixed(1)} km/h`}
            color="text-blue-400"
          />
          <StatRow
            icon={<Fuel className="w-3 h-3" />}
            label="Boost used"
            value={stats.player.boostConsumedSoFar.toFixed(0)}
            color="text-yellow-400"
          />
          <StatRow
            icon={<Plane className="w-3 h-3" />}
            label="Air time"
            value={`${stats.player.airTimeSecondsSoFar.toFixed(1)}s`}
            color="text-cyan-400"
          />
          {stats.player.isAirborne && (
            <div className="text-cyan-400 text-xs font-bold animate-pulse">IN AIR</div>
          )}
        </div>
      )}
    </div>
  );
}
