/**
 * PossessionBar - Visual ball possession bar
 * (018-stats-compiler)
 */

interface PossessionBarProps {
  team0Possession: number;
  team1Possession: number;
  className?: string;
}

export function PossessionBar({ team0Possession, team1Possession, className = '' }: PossessionBarProps) {
  // Normalize to ensure they sum to 100
  const total = team0Possession + team1Possession;
  const blue = total > 0 ? (team0Possession / total) * 100 : 50;
  const orange = total > 0 ? (team1Possession / total) * 100 : 50;

  return (
    <div className={`space-y-2 ${className}`}>
      <div className="flex justify-between text-sm">
        <span className="text-blue-400 font-medium">{blue.toFixed(0)}%</span>
        <span className="text-gray-400 text-xs uppercase tracking-wider">Ball Possession</span>
        <span className="text-orange-400 font-medium">{orange.toFixed(0)}%</span>
      </div>

      <div className="relative h-3 rounded-full bg-gray-800 overflow-hidden">
        {/* Blue team side (left) */}
        <div
          className="absolute inset-y-0 left-0 bg-gradient-to-r from-blue-500 to-blue-400 transition-all duration-500"
          style={{ width: `${blue}%` }}
        />

        {/* Orange team side (right) */}
        <div
          className="absolute inset-y-0 right-0 bg-gradient-to-l from-orange-500 to-orange-400 transition-all duration-500"
          style={{ width: `${orange}%` }}
        />

        {/* Center divider */}
        <div
          className="absolute top-0 bottom-0 w-0.5 bg-gray-700 z-10"
          style={{ left: `${blue}%`, transform: 'translateX(-50%)' }}
        />
      </div>
    </div>
  );
}
