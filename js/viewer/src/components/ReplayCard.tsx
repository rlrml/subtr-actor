import { Link } from 'react-router-dom';
import { Clock, Calendar, Eye, Trophy, User, Heart, Activity } from 'lucide-react';
import { DeleteReplayButton } from './DeleteReplayButton';
import { VisibilityToggle } from './VisibilityToggle';
import { VisibilityBadge } from './VisibilityBadge';
import { TeamSizeBadge } from './TeamSizeBadge';
import { CheatDetectionBadge } from './cheat/CheatDetectionBadge';
import { getDisplayTitle } from '@/utils/replay';
import { getQualityCategory } from '@/types/quality';

interface Player {
  id: string;
  name: string;
  team: number;
  goals?: number | null;
  assists?: number | null;
  saves?: number | null;
  shots?: number | null;
  score?: number | null;
}

interface ReplayOwner {
  id: string;
  username: string;
  avatarUrl: string | null;
}

interface Replay {
  id: string;
  originalFilename?: string;
  title?: string | null;
  visibility?: 'public' | 'unlisted';
  mapName?: string | null;
  gameMode?: string | null;
  teamSize?: number | null;
  team0Score?: number | null;
  team1Score?: number | null;
  durationSeconds?: number | null;
  playedAt?: string | null;
  status: string;
  ownerId?: string | null;
  owner?: ReplayOwner | null;
  players: Player[];
  likeCount?: number;
  viewCount?: number;
  qualityScore?: number | null;
  // Cheat detection (032-cheat-detection)
  hasCheater?: boolean;
  cheatAnalysisStatus?: string;
}

interface ReplayCardProps {
  // Support both old and new interfaces
  replay?: Replay;
  id?: string;
  mapName?: string | null;
  gameMode?: string | null;
  team0Score?: number | null;
  team1Score?: number | null;
  durationSeconds?: number | null;
  playedAt?: string | null;
  players?: Player[];
  status?: string;
  // New props for delete functionality
  showDeleteButton?: boolean;
  currentUserId?: string;
  onDelete?: (id: string) => void;
  // Visibility management (for My Replays page)
  showVisibilityToggle?: boolean;
  onVisibilityChange?: (id: string, newVisibility: 'public' | 'unlisted') => Promise<void>;
}

function formatDuration(seconds?: number | null): string {
  if (!seconds) return '--:--';
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function formatDate(dateStr?: string | null): string {
  if (!dateStr) return 'Unknown';
  const date = new Date(dateStr);
  return date.toLocaleDateString('en-US', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  });
}

function getMapDisplayName(mapName?: string | null): string {
  if (!mapName) return 'Unknown map';
  const cleaned = mapName.replace(/_P$/, '').replace(/_Standard$/, '').replace(/_/g, ' ');
  return cleaned;
}

export function ReplayCard(props: ReplayCardProps) {
  // Support both direct props and replay object
  const replay = props.replay;
  const id = replay?.id ?? props.id ?? '';
  const mapName = replay?.mapName ?? props.mapName;
  const teamSize = replay?.teamSize;
  const team0Score = replay?.team0Score ?? props.team0Score;
  const team1Score = replay?.team1Score ?? props.team1Score;
  const durationSeconds = replay?.durationSeconds ?? props.durationSeconds;
  const playedAt = replay?.playedAt ?? props.playedAt;
  const players = replay?.players ?? props.players ?? [];
  const status = replay?.status ?? props.status ?? 'unknown';
  const ownerId = replay?.ownerId;
  const originalFilename = replay?.originalFilename;
  const title = replay?.title;
  const likeCount = replay?.likeCount ?? 0;
  const viewCount = replay?.viewCount ?? 0;
  const qualityScore = replay?.qualityScore;
  // Cheat detection (032-cheat-detection)
  const hasCheater = replay?.hasCheater ?? false;
  const cheatAnalysisStatus = replay?.cheatAnalysisStatus;

  // Display title with fallback to filename
  const displayTitle = getDisplayTitle(title, originalFilename);

  const { showDeleteButton, currentUserId, onDelete, showVisibilityToggle, onVisibilityChange } = props;
  const owner = replay?.owner;
  const visibility = replay?.visibility ?? 'public';

  const team0Players = players.filter(p => p.team === 0);
  const team1Players = players.filter(p => p.team === 1);
  const isReady = status === 'ready';
  const isProcessing = status === 'processing';

  // Show delete button only if user is owner
  const canDelete = showDeleteButton && currentUserId && ownerId === currentUserId;

  // Handler for visibility toggle
  const handleVisibilityToggle = async (newVisibility: 'public' | 'unlisted') => {
    if (onVisibilityChange) {
      await onVisibilityChange(id, newVisibility);
    }
  };

  // Determine if current user is the owner (for display)
  const isCurrentUserOwner = currentUserId && owner && owner.id === currentUserId;

  // Determine winner for accent color
  const blueWins = (team0Score ?? 0) > (team1Score ?? 0);
  const orangeWins = (team1Score ?? 0) > (team0Score ?? 0);

  return (
    <Link to={`/replays/${id}`} className="block group h-full min-w-0">
      <div className="relative h-full flex flex-col rounded-xl overflow-hidden bg-gradient-to-br from-gray-900 to-gray-950 border border-gray-800 hover:border-violet-500/50 transition-all duration-300 hover:shadow-lg hover:shadow-violet-500/10 min-w-0">
        {/* Winner accent bar - red if cheater detected (032-cheat-detection) */}
        <div className={`absolute top-0 left-0 right-0 h-1 ${
          hasCheater && cheatAnalysisStatus === 'completed' ? 'bg-gradient-to-r from-red-500 to-red-400' :
          blueWins ? 'bg-gradient-to-r from-blue-500 to-blue-400' :
          orangeWins ? 'bg-gradient-to-r from-orange-500 to-orange-400' :
          'bg-gradient-to-r from-violet-500 to-blue-500'
        }`} />

        {/* Subtle gradient overlay */}
        <div className="absolute inset-0 bg-gradient-to-br from-violet-600/5 via-transparent to-blue-600/5 pointer-events-none" />

        {/* Processing overlay */}
        {isProcessing && (
          <div className="absolute inset-0 bg-gray-900/80 backdrop-blur-sm flex items-center justify-center z-10">
            <div className="flex flex-col items-center gap-2">
              <div className="w-8 h-8 border-2 border-yellow-400 border-t-transparent rounded-full animate-spin" />
              <span className="text-sm text-yellow-400 font-medium">Processing...</span>
            </div>
          </div>
        )}

        <div className="relative p-3 xs:p-4 sm:p-5 flex flex-col flex-grow overflow-hidden">
          {/* Header with title, map and meta */}
          <div className="space-y-2 mb-3 xs:mb-4 min-w-0">
            {/* Title row with actions */}
            <div className="flex items-start justify-between gap-2 min-w-0">
              <div className="flex items-center gap-2 min-w-0 flex-1 overflow-hidden">
                <h3 className="font-bold text-gray-200 truncate group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:from-violet-400 group-hover:to-blue-400 group-hover:bg-clip-text transition-all duration-300 min-w-0">
                  {displayTitle}
                </h3>
                {/* Cheat Detection Badge (032-cheat-detection) */}
                {cheatAnalysisStatus === 'completed' && hasCheater && (
                  <CheatDetectionBadge hasCheater={hasCheater} status="completed" size="sm" />
                )}
              </div>
              <div className="flex items-center gap-1.5 xs:gap-2 flex-shrink-0">
                {/* Visibility toggle or badge */}
                {showVisibilityToggle && onVisibilityChange ? (
                  <VisibilityToggle
                    visibility={visibility}
                    onToggle={handleVisibilityToggle}
                  />
                ) : visibility === 'unlisted' ? (
                  <VisibilityBadge visibility={visibility} />
                ) : null}
                {canDelete && (
                  <DeleteReplayButton
                    replayId={id}
                    replayName={displayTitle}
                    onDelete={onDelete}
                  />
                )}
                {!isReady && !isProcessing && (
                  <span className="px-2 py-1 text-xs rounded-full bg-red-500/20 text-red-400 border border-red-500/30 animate-pulse">
                    Error
                  </span>
                )}
              </div>
            </div>

            {/* Map info row */}
            <div className="flex items-center gap-1.5 text-xs xs:text-sm text-gray-500 min-w-0">
              <TeamSizeBadge teamSize={teamSize} playerCount={players.length} variant="compact" />
              <span className="truncate">{getMapDisplayName(mapName)}</span>
            </div>
          </div>

          {/* Score section with glow */}
          <div className="relative mb-3 xs:mb-4">
            <div className="absolute inset-0 rounded-xl bg-gradient-to-r from-blue-500/10 via-transparent to-orange-500/10 blur-xl" />
            <div className="relative flex items-center justify-center gap-2 py-2.5 xs:py-3 sm:py-4 rounded-xl bg-gray-800/80 border border-gray-700/50">
              <div className="text-center flex-1 min-w-0">
                <div
                  className="text-2xl xs:text-3xl sm:text-4xl font-black text-blue-400"
                  style={{ textShadow: blueWins ? '0 0 20px rgba(96,165,250,0.6)' : 'none' }}
                >
                  {team0Score ?? '-'}
                </div>
                <div className="text-[10px] xs:text-xs text-blue-400/70 mt-0.5 xs:mt-1 font-medium flex items-center justify-center gap-1">
                  {blueWins && <Trophy className="w-3 h-3" />}
                  Blue
                </div>
              </div>

              <div className="flex flex-col items-center flex-shrink-0">
                <div className="text-gray-600 font-bold text-sm xs:text-base sm:text-lg">VS</div>
              </div>

              <div className="text-center flex-1 min-w-0">
                <div
                  className="text-2xl xs:text-3xl sm:text-4xl font-black text-orange-400"
                  style={{ textShadow: orangeWins ? '0 0 20px rgba(251,146,60,0.6)' : 'none' }}
                >
                  {team1Score ?? '-'}
                </div>
                <div className="text-[10px] xs:text-xs text-orange-400/70 mt-0.5 xs:mt-1 font-medium flex items-center justify-center gap-1">
                  {orangeWins && <Trophy className="w-3 h-3" />}
                  Orange
                </div>
              </div>
            </div>
          </div>

          {/* Teams */}
          <div className="grid grid-cols-2 gap-2 xs:gap-3 sm:gap-4 mb-3 xs:mb-4 overflow-hidden">
            <div className="space-y-1 xs:space-y-1.5 min-w-0 overflow-hidden">
              {team0Players.slice(0, 3).map(p => (
                <div key={p.id} className="text-blue-300 truncate text-[11px] xs:text-xs font-medium px-1.5 xs:px-2 py-1 rounded bg-blue-500/10 border border-blue-500/20">
                  {p.name}
                </div>
              ))}
              {team0Players.length > 3 && (
                <div className="text-blue-400/50 text-[11px] xs:text-xs pl-1.5 xs:pl-2">+{team0Players.length - 3} more</div>
              )}
              {team0Players.length === 0 && (
                <div className="text-gray-600 text-[11px] xs:text-xs px-1.5 xs:px-2 py-1">No players</div>
              )}
            </div>
            <div className="space-y-1 xs:space-y-1.5 min-w-0 overflow-hidden">
              {team1Players.slice(0, 3).map(p => (
                <div key={p.id} className="text-orange-300 truncate text-[11px] xs:text-xs font-medium px-1.5 xs:px-2 py-1 rounded bg-orange-500/10 border border-orange-500/20 text-right">
                  {p.name}
                </div>
              ))}
              {team1Players.length > 3 && (
                <div className="text-orange-400/50 text-[11px] xs:text-xs pr-1.5 xs:pr-2 text-right">+{team1Players.length - 3} more</div>
              )}
              {team1Players.length === 0 && (
                <div className="text-gray-600 text-[11px] xs:text-xs px-1.5 xs:px-2 py-1 text-right">No players</div>
              )}
            </div>
          </div>

          {/* Spacer to push footer and button to bottom */}
          <div className="flex-grow" />

          {/* Footer with meta - 2 rows for better readability */}
          <div className="space-y-2 pt-3 border-t border-gray-800/50">
            {/* Row 1: Duration, Likes, Quality */}
            <div className="flex flex-wrap items-center gap-1.5 xs:gap-2 text-xs text-gray-400">
              <div className="flex items-center gap-1 xs:gap-1.5 bg-gray-800/50 px-1.5 xs:px-2 py-1 rounded">
                <Clock className="w-3 xs:w-3.5 h-3 xs:h-3.5 text-violet-400" />
                <span className="text-[11px] xs:text-xs">{formatDuration(durationSeconds)}</span>
              </div>
              <div className="flex items-center gap-1 xs:gap-1.5 bg-gray-800/50 px-1.5 xs:px-2 py-1 rounded">
                <Heart className={`w-3 xs:w-3.5 h-3 xs:h-3.5 ${likeCount > 0 ? 'text-red-400' : 'text-gray-500'}`} />
                <span className="text-[11px] xs:text-xs">{likeCount}</span>
              </div>
              <div className="flex items-center gap-1 xs:gap-1.5 bg-gray-800/50 px-1.5 xs:px-2 py-1 rounded" title={`${viewCount.toLocaleString()} vues`}>
                <Eye className={`w-3 xs:w-3.5 h-3 xs:h-3.5 ${viewCount > 0 ? 'text-blue-400' : 'text-gray-500'}`} />
                <span className="text-[11px] xs:text-xs">{viewCount >= 1000 ? `${(viewCount / 1000).toFixed(1)}k` : viewCount}</span>
              </div>
              {qualityScore !== null && qualityScore !== undefined && (
                <div
                  className={`flex items-center gap-1 xs:gap-1.5 px-1.5 xs:px-2 py-1 rounded text-[11px] xs:text-xs ${
                    getQualityCategory(qualityScore) === 'good'
                      ? 'bg-green-500/10 text-green-400'
                      : getQualityCategory(qualityScore) === 'medium'
                        ? 'bg-amber-500/10 text-amber-400'
                        : 'bg-red-500/10 text-red-400'
                  }`}
                  title={`Data quality: ${qualityScore}%`}
                >
                  <Activity className="w-3 xs:w-3.5 h-3 xs:h-3.5" />
                  {qualityScore}%
                </div>
              )}
            </div>

            {/* Row 2: Owner, Date */}
            <div className="flex items-center justify-between gap-2 text-[11px] xs:text-xs text-gray-400">
              <div className="flex items-center gap-1 xs:gap-1.5 min-w-0">
                <User className="w-3 xs:w-3.5 h-3 xs:h-3.5 text-violet-400 shrink-0" />
                {isCurrentUserOwner ? (
                  <span className="text-violet-300 font-medium">You</span>
                ) : owner ? (
                  <span className="truncate max-w-[80px] xs:max-w-[100px]">{owner.username}</span>
                ) : (
                  <span className="text-gray-500">Anon</span>
                )}
              </div>
              <div className="flex items-center gap-1 xs:gap-1.5 shrink-0">
                <Calendar className="w-3 xs:w-3.5 h-3 xs:h-3.5 text-violet-400" />
                {formatDate(playedAt)}
              </div>
            </div>
          </div>

          {/* Play button hint */}
          {isReady && (
            <div className="relative overflow-hidden mt-4">
              <div className="absolute inset-0 bg-gradient-to-r from-violet-600/20 to-blue-600/20 blur-xl group-hover:from-violet-600/40 group-hover:to-blue-600/40 transition-all duration-300" />
              <div className="relative flex items-center justify-center gap-2 py-2.5 rounded-lg bg-gradient-to-r from-violet-600/20 to-blue-600/20 border border-violet-500/30 text-violet-300 text-sm font-medium group-hover:from-violet-600/30 group-hover:to-blue-600/30 group-hover:border-violet-500/50 group-hover:text-white transition-all duration-300">
                <Eye className="w-4 h-4" />
                View replay
              </div>
            </div>
          )}
        </div>
      </div>
    </Link>
  );
}
