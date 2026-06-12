/**
 * PlayerStats - Display individual player advanced stats
 * (018-stats-compiler)
 */

import { Trophy, Zap, Shield, Target, Gauge, Fuel, Plane } from 'lucide-react';
import { PlayerLink } from '@/components/player';

export interface PlayerStatsData {
  playerId: string;
  playerName: string;
  platform: string;
  team: number;
  // Basic stats
  goals: number;
  assists: number;
  saves: number;
  shots: number;
  demos: number;
  score: number;
  // Advanced stats
  avgSpeed: number;
  maxSpeed: number;
  boostConsumed: number;
  boostPickupsSmall: number;
  boostPickupsLarge: number;
  airTimeSeconds: number;
  airTimePercentage: number;
  offensivePercentage: number;
  defensivePercentage: number;
}

interface PlayerStatsProps {
  players: PlayerStatsData[];
}

function StatBadge({ icon, value, label, color }: {
  icon: React.ReactNode;
  value: string | number;
  label: string;
  color: string;
}) {
  return (
    <div className="flex items-center gap-1.5" title={label}>
      <span className={color}>{icon}</span>
      <span className="text-sm text-gray-300">{value}</span>
    </div>
  );
}

function PositioningBar({
  offensive,
  defensive,
  teamColor
}: {
  offensive: number;
  defensive: number;
  teamColor: 'blue' | 'orange';
}) {
  const midfield = 100 - offensive - defensive;
  // Blue team: def (blue) -> mid (gray) -> off (orange, attacking orange goal)
  // Orange team: def (orange) -> mid (gray) -> off (blue, attacking blue goal)
  const colors = teamColor === 'blue'
    ? { def: 'bg-blue-500', mid: 'bg-gray-500', off: 'bg-orange-500' }
    : { def: 'bg-orange-500', mid: 'bg-gray-500', off: 'bg-blue-500' };

  return (
    <div className="flex gap-0.5 h-1.5 rounded overflow-hidden" title={`Def: ${defensive.toFixed(0)}% | Mid: ${midfield.toFixed(0)}% | Off: ${offensive.toFixed(0)}%`}>
      <div className={`${colors.def}`} style={{ width: `${defensive}%` }} />
      <div className={`${colors.mid} opacity-60`} style={{ width: `${midfield}%` }} />
      <div className={`${colors.off}`} style={{ width: `${offensive}%` }} />
    </div>
  );
}

function PlayerRow({ player, teamColor }: { player: PlayerStatsData; teamColor: 'blue' | 'orange' }) {
  const bgColor = teamColor === 'blue' ? 'bg-blue-500/5 border-blue-500/20' : 'bg-orange-500/5 border-orange-500/20';

  return (
    <div className={`p-3 rounded-lg border ${bgColor}`}>
      {/* Header: Name and Score */}
      <div className="flex items-center justify-between mb-2">
        <PlayerLink
          playerId={player.playerId}
          name={player.playerName}
          platform={player.platform}
          team={player.team}
          className="font-medium"
        />
        <span className="text-gray-400 text-sm font-mono">{player.score} pts</span>
      </div>

      {/* Basic Stats Row */}
      <div className="flex items-center gap-3 mb-3 text-xs">
        <StatBadge icon={<Trophy className="w-3 h-3" />} value={player.goals} label="Goals" color="text-yellow-400" />
        <StatBadge icon={<Zap className="w-3 h-3" />} value={player.assists} label="Assists" color="text-cyan-400" />
        <StatBadge icon={<Shield className="w-3 h-3" />} value={player.saves} label="Saves" color="text-green-400" />
        <StatBadge icon={<Target className="w-3 h-3" />} value={player.shots} label="Shots" color="text-violet-400" />
      </div>

      {/* Advanced Stats */}
      <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-gray-400">
        <div className="flex items-center gap-1">
          <Gauge className="w-3 h-3 text-blue-400" />
          <span>Speed: </span>
          <span className="text-gray-300">{player.avgSpeed.toFixed(1)} km/h</span>
        </div>
        <div className="flex items-center gap-1">
          <Fuel className="w-3 h-3 text-yellow-400" />
          <span>Boost: </span>
          <span className="text-gray-300">{player.boostConsumed.toFixed(0)}</span>
        </div>
        <div className="flex items-center gap-1">
          <Plane className="w-3 h-3 text-cyan-400" />
          <span>Air time: </span>
          <span className="text-gray-300">{player.airTimePercentage.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1">
          <span className="text-gray-500">Pickups:</span>
          <span className="text-gray-300">{player.boostPickupsSmall}s / {player.boostPickupsLarge}L</span>
        </div>
      </div>

      {/* Positioning Bar */}
      <div className="mt-2">
        <PositioningBar
          offensive={player.offensivePercentage}
          defensive={player.defensivePercentage}
          teamColor={teamColor}
        />
      </div>
    </div>
  );
}

export function PlayerStats({ players }: PlayerStatsProps) {
  const team0Players = players.filter(p => p.team === 0);
  const team1Players = players.filter(p => p.team === 1);

  return (
    <div className="space-y-4">
      <h4 className="text-sm font-semibold text-gray-400 uppercase tracking-wider">Player Stats</h4>

      <div className="grid md:grid-cols-2 gap-4">
        {/* Blue Team */}
        <div className="space-y-2">
          <div className="text-xs text-blue-500 uppercase tracking-wider mb-2">Blue Team</div>
          {team0Players.map(player => (
            <PlayerRow key={player.playerId} player={player} teamColor="blue" />
          ))}
        </div>

        {/* Orange Team */}
        <div className="space-y-2">
          <div className="text-xs text-orange-500 uppercase tracking-wider mb-2">Orange Team</div>
          {team1Players.map(player => (
            <PlayerRow key={player.playerId} player={player} teamColor="orange" />
          ))}
        </div>
      </div>
    </div>
  );
}
