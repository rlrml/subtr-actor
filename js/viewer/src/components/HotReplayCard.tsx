import { Link } from 'react-router-dom';
import { Eye, Play } from 'lucide-react';
import { getDisplayTitle } from '@/utils/replay';
import { CheatDetectionBadge } from './cheat/CheatDetectionBadge';

interface Player {
  id: string;
  name: string;
  team: number;
}

interface Replay {
  id: string;
  originalFilename?: string;
  title?: string | null;
  mapName?: string | null;
  teamSize?: number | null;
  team0Score?: number | null;
  team1Score?: number | null;
  durationSeconds?: number | null;
  players: Player[];
  viewCount?: number;
  // Cheat detection (032-cheat-detection)
  hasCheater?: boolean;
  cheatAnalysisStatus?: string;
}

function getMapShortName(mapName?: string | null): string {
  if (!mapName) return '';
  const cleaned = mapName.replace(/_P$/, '').replace(/_Standard$/, '').replace(/_/g, ' ');
  const shortNames: Record<string, string> = {
    'DFH Stadium': 'DFH Stadium',
    'Mannfield': 'Mannfield',
    'Champions Field': 'Champions Field',
    'Urban Central': 'Urban Central',
    'Beckwith Park': 'Beckwith Park',
    'Utopia Coliseum': 'Utopia Coliseum',
    'Aquadome': 'Aquadome',
    'Starbase Arc': 'Starbase Arc',
    'Farmstead': 'Farmstead',
    'Salty Shores': 'Salty Shores',
    'Forbidden Temple': 'Forbidden Temple',
    'Neon Fields': 'Neon Fields',
  };
  return shortNames[cleaned] || cleaned;
}

function getTeamSize(teamSize?: number | null, players?: Player[]): string {
  if (teamSize) return `${teamSize}v${teamSize}`;
  if (players && players.length > 0) {
    const team0Count = players.filter(p => p.team === 0).length;
    const team1Count = players.filter(p => p.team === 1).length;
    const size = Math.max(team0Count, team1Count);
    if (size > 0) return `${size}v${size}`;
  }
  return '';
}

interface HotReplayCardProps {
  replay: Replay;
  rank?: number;
}

export function HotReplayCard({ replay, rank }: HotReplayCardProps) {
  const {
    id,
    originalFilename,
    title,
    mapName,
    teamSize,
    team0Score,
    team1Score,
    players,
    viewCount = 0,
    hasCheater = false,
    cheatAnalysisStatus,
  } = replay;

  const displayTitle = getDisplayTitle(title, originalFilename);
  const blueWins = (team0Score ?? 0) > (team1Score ?? 0);
  const orangeWins = (team1Score ?? 0) > (team0Score ?? 0);
  const teamSizeStr = getTeamSize(teamSize, players);
  const mapStr = getMapShortName(mapName);

  return (
    <Link to={`/replays/${id}`} className="block group">
      <div className="flex items-center gap-4 p-4 rounded-xl bg-gray-900/50 border border-gray-800 hover:border-orange-500/30 hover:bg-gray-900/80 transition-all">
        {/* Rank */}
        {rank && (
          <div className={`
            w-10 h-10 rounded-xl flex items-center justify-center font-black text-lg flex-shrink-0
            ${rank === 1
              ? 'bg-gradient-to-br from-yellow-400 to-amber-500 text-yellow-900'
              : rank === 2
                ? 'bg-gradient-to-br from-gray-300 to-gray-400 text-gray-700'
                : 'bg-gradient-to-br from-amber-600 to-amber-700 text-amber-100'
            }
          `}>
            {rank}
          </div>
        )}

        {/* Score */}
        <div className="flex items-center gap-1.5 flex-shrink-0 w-[72px] justify-center">
          <span className={`text-xl font-black tabular-nums ${blueWins ? 'text-blue-400' : 'text-blue-400/50'}`}>
            {team0Score ?? '-'}
          </span>
          <span className="text-gray-600">-</span>
          <span className={`text-xl font-black tabular-nums ${orangeWins ? 'text-orange-400' : 'text-orange-400/50'}`}>
            {team1Score ?? '-'}
          </span>
        </div>

        {/* Title & Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-white text-sm truncate group-hover:text-orange-300 transition-colors">
              {displayTitle}
            </h3>
            {/* Cheat Detection Badge (032-cheat-detection) */}
            {cheatAnalysisStatus === 'completed' && hasCheater && (
              <CheatDetectionBadge hasCheater={hasCheater} status="completed" size="sm" />
            )}
          </div>
          <div className="flex items-center gap-2 text-xs text-gray-500">
            {teamSizeStr && (
              <span className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-400 font-medium">
                {teamSizeStr}
              </span>
            )}
            {mapStr && <span className="truncate">{mapStr}</span>}
          </div>
        </div>

        {/* Views */}
        <div className="flex items-center gap-1.5 text-xs text-gray-500 flex-shrink-0">
          <Eye className="w-3.5 h-3.5" />
          <span>{viewCount >= 1000 ? `${(viewCount / 1000).toFixed(1)}k` : viewCount}</span>
        </div>

        {/* Watch button */}
        <div className="w-8 h-8 rounded-lg bg-orange-500/10 flex items-center justify-center text-orange-400 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
          <Play className="w-4 h-4 fill-current" />
        </div>
      </div>
    </Link>
  );
}
