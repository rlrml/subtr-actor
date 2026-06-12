
import { ChevronLeft, ChevronRight } from 'lucide-react';

// Team colors matching Rocket League
const TEAM_COLORS = {
  blue: '#3399ff',
  orange: '#ff6600',
};

interface PlayerIndicatorProps {
  currentPlayer: string;
  playerTeam: number; // 0 = blue, 1 = orange
  players: string[];
  playerTeams: Record<string, number>;
  onNavigate: (player: string) => void;
}

export function PlayerIndicator({
  currentPlayer,
  playerTeam,
  players,
  playerTeams,
  onNavigate,
}: PlayerIndicatorProps) {
  // Sort players by team (blue first, then orange) for consistent navigation
  const sortedPlayers = [...players].sort((a, b) => {
    const teamA = playerTeams[a] ?? 0;
    const teamB = playerTeams[b] ?? 0;
    return teamA - teamB;
  });

  const currentIndex = sortedPlayers.indexOf(currentPlayer);
  const teamColor = playerTeam === 0 ? TEAM_COLORS.blue : TEAM_COLORS.orange;

  const handlePrev = () => {
    if (sortedPlayers.length === 0) return;
    const prevIndex = currentIndex <= 0 ? sortedPlayers.length - 1 : currentIndex - 1;
    onNavigate(sortedPlayers[prevIndex]);
  };

  const handleNext = () => {
    if (sortedPlayers.length === 0) return;
    const nextIndex = currentIndex >= sortedPlayers.length - 1 ? 0 : currentIndex + 1;
    onNavigate(sortedPlayers[nextIndex]);
  };

  return (
    <div className="absolute bottom-28 left-1/2 -translate-x-1/2 z-30 pointer-events-auto">
      <div className="flex items-center gap-2 px-3 py-2 bg-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-700 shadow-lg">
        {/* Previous player button */}
        <button
          onClick={handlePrev}
          className="p-1 rounded hover:bg-white/10 transition-colors text-gray-300 hover:text-white"
          title="Previous player"
        >
          <ChevronLeft className="w-5 h-5" />
        </button>

        {/* Player name with team color */}
        <span
          className="font-semibold text-lg min-w-[120px] text-center"
          style={{ color: teamColor }}
        >
          {currentPlayer}
        </span>

        {/* Next player button */}
        <button
          onClick={handleNext}
          className="p-1 rounded hover:bg-white/10 transition-colors text-gray-300 hover:text-white"
          title="Next player"
        >
          <ChevronRight className="w-5 h-5" />
        </button>
      </div>
    </div>
  );
}
