/**
 * StatsPanel - Main panel displaying all replay stats
 * (018-stats-compiler)
 */

import { BarChart3, AlertCircle, Loader2 } from 'lucide-react';
import { PossessionBar } from './PossessionBar';
import { TeamStats } from './TeamStats';
import { PlayerStats, type PlayerStatsData } from './PlayerStats';

interface TeamStatsData {
  team: number;
  possession: number;
  avgSpeed: number;
  boostPickups: number;
}

interface MatchStats {
  ballAvgSpeed: number;
  ballMaxSpeed: number;
}

export interface ReplayStatsData {
  available: boolean;
  message?: string;
  stats?: {
    teams: {
      team0: TeamStatsData;
      team1: TeamStatsData;
    };
    players: PlayerStatsData[];
    match: MatchStats;
  };
}

interface StatsPanelProps {
  data: ReplayStatsData | null;
  loading?: boolean;
  error?: string | null;
}

export function StatsPanel({ data, loading = false, error = null }: StatsPanelProps) {
  // Loading state
  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="w-6 h-6 animate-spin text-violet-400" />
        <span className="ml-2 text-gray-400">Loading stats...</span>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="flex items-center justify-center py-8 text-red-400">
        <AlertCircle className="w-5 h-5 mr-2" />
        <span>{error}</span>
      </div>
    );
  }

  // Stats not available (old replay)
  if (!data || !data.available || !data.stats) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-gray-500">
        <BarChart3 className="w-8 h-8 mb-2 opacity-50" />
        <p className="text-sm">Advanced stats are not available for this replay.</p>
        <p className="text-xs mt-1">Stats are only computed for newly uploaded replays.</p>
      </div>
    );
  }

  const { teams, players, match } = data.stats;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-lg bg-violet-500/20 flex items-center justify-center">
          <BarChart3 className="w-5 h-5 text-violet-400" />
        </div>
        <div>
          <h3 className="font-semibold text-white">Advanced Stats</h3>
          <p className="text-sm text-gray-500">Detailed performance metrics</p>
        </div>
      </div>

      {/* Ball Possession */}
      <PossessionBar
        team0Possession={teams.team0.possession}
        team1Possession={teams.team1.possession}
      />

      {/* Team Stats Comparison */}
      <TeamStats team0={teams.team0} team1={teams.team1} />

      {/* Ball Stats */}
      <div className="flex justify-center gap-8 py-3 px-4 rounded-lg bg-gray-800/50 border border-gray-700/50">
        <div className="text-center">
          <div className="text-lg font-mono text-gray-300">{match.ballAvgSpeed.toFixed(1)} km/h</div>
          <div className="text-xs text-gray-500 uppercase tracking-wider">Avg Ball Speed</div>
        </div>
        <div className="w-px bg-gray-700" />
        <div className="text-center">
          <div className="text-lg font-mono text-gray-300">{match.ballMaxSpeed.toFixed(1)} km/h</div>
          <div className="text-xs text-gray-500 uppercase tracking-wider">Max Ball Speed</div>
        </div>
      </div>

      {/* Player Stats */}
      <PlayerStats players={players} />
    </div>
  );
}
