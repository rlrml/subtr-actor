/**
 * PlayerStatsOverview - Display detailed player statistics
 * (018-stats-compiler)
 */

import { Trophy, Zap, Shield, Target, Gauge, Fuel, Plane, Crosshair } from 'lucide-react';

interface PlayerStatsOverviewProps {
  stats: {
    totalMatches: number;
    totalGoals: number;
    totalAssists: number;
    totalSaves: number;
    totalShots: number;
    totalDemos: number;
    avgSpeed?: number | null;
    avgBoostConsumption?: number | null;
    avgAirTimePercentage?: number | null;
    avgOffensivePercentage?: number | null;
  };
}

function StatCard({
  icon,
  label,
  value,
  unit,
  color,
  size = 'normal',
}: {
  icon: React.ReactNode;
  label: string;
  value: number | string;
  unit?: string;
  color: string;
  size?: 'normal' | 'large';
}) {
  return (
    <div className={`p-4 rounded-xl bg-gray-800/50 border border-gray-700/50 ${size === 'large' ? 'col-span-2' : ''}`}>
      <div className="flex items-center gap-2 mb-2">
        <span className={color}>{icon}</span>
        <span className="text-sm text-gray-400">{label}</span>
      </div>
      <div className="flex items-baseline gap-1">
        <span className="text-2xl font-bold text-white">
          {typeof value === 'number' ? value.toLocaleString() : value}
        </span>
        {unit && <span className="text-sm text-gray-500">{unit}</span>}
      </div>
    </div>
  );
}

function PercentageBar({
  label,
  value,
  color,
  icon,
}: {
  label: string;
  value: number;
  color: string;
  icon: React.ReactNode;
}) {
  return (
    <div className="p-4 rounded-xl bg-gray-800/50 border border-gray-700/50">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <span className={color}>{icon}</span>
          <span className="text-sm text-gray-400">{label}</span>
        </div>
        <span className="text-lg font-bold text-white">{value.toFixed(1)}%</span>
      </div>
      <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
        <div
          className={`h-full ${color.replace('text-', 'bg-')} opacity-70 transition-all duration-500`}
          style={{ width: `${Math.min(value, 100)}%` }}
        />
      </div>
    </div>
  );
}

export function PlayerStatsOverview({ stats }: PlayerStatsOverviewProps) {
  // Calculate per-match averages
  const matches = stats.totalMatches || 1;
  const goalsPerMatch = stats.totalGoals / matches;
  const assistsPerMatch = stats.totalAssists / matches;
  const savesPerMatch = stats.totalSaves / matches;
  const shotsPerMatch = stats.totalShots / matches;
  const shotAccuracy = stats.totalShots > 0 ? (stats.totalGoals / stats.totalShots) * 100 : 0;

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-white">Career Statistics</h3>

      {/* Basic Stats Grid */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        <StatCard
          icon={<Trophy className="w-5 h-5" />}
          label="Goals"
          value={stats.totalGoals}
          color="text-yellow-400"
        />
        <StatCard
          icon={<Zap className="w-5 h-5" />}
          label="Assists"
          value={stats.totalAssists}
          color="text-cyan-400"
        />
        <StatCard
          icon={<Shield className="w-5 h-5" />}
          label="Saves"
          value={stats.totalSaves}
          color="text-green-400"
        />
        <StatCard
          icon={<Target className="w-5 h-5" />}
          label="Shots"
          value={stats.totalShots}
          color="text-violet-400"
        />
        <StatCard
          icon={<Crosshair className="w-5 h-5" />}
          label="Demos"
          value={stats.totalDemos}
          color="text-red-400"
        />
        <StatCard
          icon={<Target className="w-5 h-5" />}
          label="Shot Accuracy"
          value={shotAccuracy.toFixed(1)}
          unit="%"
          color="text-blue-400"
        />
      </div>

      {/* Per Match Averages */}
      <div>
        <h4 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
          Per Match Averages
        </h4>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <div className="p-3 rounded-lg bg-gray-800/30 text-center">
            <div className="text-xl font-bold text-yellow-400">{goalsPerMatch.toFixed(2)}</div>
            <div className="text-xs text-gray-500">Goals/Match</div>
          </div>
          <div className="p-3 rounded-lg bg-gray-800/30 text-center">
            <div className="text-xl font-bold text-cyan-400">{assistsPerMatch.toFixed(2)}</div>
            <div className="text-xs text-gray-500">Assists/Match</div>
          </div>
          <div className="p-3 rounded-lg bg-gray-800/30 text-center">
            <div className="text-xl font-bold text-green-400">{savesPerMatch.toFixed(2)}</div>
            <div className="text-xs text-gray-500">Saves/Match</div>
          </div>
          <div className="p-3 rounded-lg bg-gray-800/30 text-center">
            <div className="text-xl font-bold text-violet-400">{shotsPerMatch.toFixed(2)}</div>
            <div className="text-xs text-gray-500">Shots/Match</div>
          </div>
        </div>
      </div>

      {/* Advanced Stats (if available) */}
      {(stats.avgSpeed || stats.avgBoostConsumption || stats.avgAirTimePercentage || stats.avgOffensivePercentage) && (
        <div>
          <h4 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
            Advanced Metrics
          </h4>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {stats.avgSpeed != null && (
              <StatCard
                icon={<Gauge className="w-5 h-5" />}
                label="Avg Speed"
                value={stats.avgSpeed.toFixed(1)}
                unit="km/h"
                color="text-blue-400"
              />
            )}
            {stats.avgBoostConsumption != null && (
              <StatCard
                icon={<Fuel className="w-5 h-5" />}
                label="Avg Boost Used"
                value={stats.avgBoostConsumption.toFixed(0)}
                unit="/match"
                color="text-yellow-400"
              />
            )}
            {stats.avgAirTimePercentage != null && (
              <PercentageBar
                icon={<Plane className="w-5 h-5" />}
                label="Air Time"
                value={stats.avgAirTimePercentage}
                color="text-cyan-400"
              />
            )}
            {stats.avgOffensivePercentage != null && (
              <PercentageBar
                icon={<Target className="w-5 h-5" />}
                label="Offensive Time"
                value={stats.avgOffensivePercentage}
                color="text-red-400"
              />
            )}
          </div>
        </div>
      )}
    </div>
  );
}
