/**
 * TeamStats - Display team-level advanced stats
 * (018-stats-compiler)
 */

import { Gauge, Fuel } from 'lucide-react';

interface TeamStatsData {
  team: number;
  possession: number;
  avgSpeed: number;
  boostPickups: number;
}

interface TeamStatsProps {
  team0: TeamStatsData;
  team1: TeamStatsData;
}

function StatCompare({
  label,
  icon,
  value0,
  value1,
  unit = '',
  format = (v: number) => v.toFixed(0),
}: {
  label: string;
  icon: React.ReactNode;
  value0: number;
  value1: number;
  unit?: string;
  format?: (v: number) => string;
}) {
  const blueHigher = value0 > value1;
  const orangeHigher = value1 > value0;

  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex items-center gap-2 min-w-[80px] text-blue-400">
        <span className={`font-mono text-sm ${blueHigher ? 'font-bold' : ''}`}>{format(value0)}{unit}</span>
        {blueHigher && <span className="text-xs text-blue-300">▲</span>}
      </div>

      <div className="flex items-center gap-2 text-gray-500">
        <span className="w-5 h-5">{icon}</span>
        <span className="text-xs uppercase tracking-wider whitespace-nowrap">{label}</span>
      </div>

      <div className="flex items-center gap-2 min-w-[80px] justify-end text-orange-400">
        {orangeHigher && <span className="text-xs text-orange-300">▲</span>}
        <span className={`font-mono text-sm ${orangeHigher ? 'font-bold' : ''}`}>{format(value1)}{unit}</span>
      </div>
    </div>
  );
}

export function TeamStats({ team0, team1 }: TeamStatsProps) {
  return (
    <div className="space-y-1">
      <div className="flex justify-between text-xs text-gray-500 uppercase tracking-wider pb-2 border-b border-gray-800">
        <span className="text-blue-500">Blue</span>
        <span>Team Stats</span>
        <span className="text-orange-500">Orange</span>
      </div>

      <StatCompare
        label="Avg Speed"
        icon={<Gauge className="w-4 h-4" />}
        value0={team0.avgSpeed}
        value1={team1.avgSpeed}
        unit=" km/h"
        format={(v) => v.toFixed(1)}
      />

      <StatCompare
        label="Boost Pickups"
        icon={<Fuel className="w-4 h-4" />}
        value0={team0.boostPickups}
        value1={team1.boostPickups}
      />
    </div>
  );
}
