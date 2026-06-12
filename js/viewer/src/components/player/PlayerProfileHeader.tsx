/**
 * PlayerProfileHeader - Display player info and platform badge
 * (018-stats-compiler)
 */

import { User, Gamepad2, AlertTriangle } from 'lucide-react';
import { FaSteam, FaPlaystation, FaXbox } from 'react-icons/fa';
import { SiEpicgames, SiNintendoswitch } from 'react-icons/si';

interface PlayerProfileHeaderProps {
  player: {
    id: string;
    displayName: string;
    platform: string;
    platformId: string;
    avatarUrl?: string | null;
    stats: {
      totalMatches: number;
      totalGoals: number;
      totalAssists: number;
      totalSaves: number;
      totalShots: number;
      totalDemos: number;
    };
    firstSeenAt?: string;
    lastSeenAt?: string;
  };
  /** Whether this player has been flagged as a cheater (032-cheat-detection) */
  isFlaggedCheater?: boolean;
}

function getPlatformIcon(platform: string) {
  switch (platform) {
    case 'Steam':
      return <FaSteam className="w-4 h-4" />;
    case 'Epic':
      return <SiEpicgames className="w-4 h-4" />;
    case 'PS4':
    case 'PS5':
      return <FaPlaystation className="w-4 h-4" />;
    case 'Xbox':
      return <FaXbox className="w-4 h-4" />;
    case 'Switch':
      return <SiNintendoswitch className="w-4 h-4" />;
    default:
      return <Gamepad2 className="w-4 h-4" />;
  }
}

function getPlatformLabel(platform: string): string {
  switch (platform) {
    case 'Steam':
      return 'Steam';
    case 'Epic':
      return 'Epic Games';
    case 'PS4':
      return 'PlayStation 4';
    case 'PS5':
      return 'PlayStation 5';
    case 'Xbox':
      return 'Xbox';
    case 'Switch':
      return 'Nintendo Switch';
    default:
      return platform;
  }
}

function formatDate(dateStr?: string): string {
  if (!dateStr) return 'Unknown';
  return new Date(dateStr).toLocaleDateString('en-US', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  });
}

export function PlayerProfileHeader({ player, isFlaggedCheater }: PlayerProfileHeaderProps) {
  const platformLabel = getPlatformLabel(player.platform);
  const platformIcon = getPlatformIcon(player.platform);

  return (
    <div className={`relative overflow-hidden rounded-2xl bg-gradient-to-br from-gray-900 via-gray-900 to-gray-900 border ${
      isFlaggedCheater ? 'border-red-500/50' : 'border-gray-700/50'
    }`}>
      {/* Gradient overlay */}
      <div className={`absolute inset-0 ${
        isFlaggedCheater
          ? 'bg-gradient-to-r from-red-600/10 via-red-600/5 to-transparent'
          : 'bg-gradient-to-r from-violet-600/10 via-blue-600/5 to-transparent'
      }`} />

      <div className="relative p-6 sm:p-8">
        <div className="flex flex-col sm:flex-row items-center sm:items-start gap-6">
          {/* Avatar */}
          <div className={`w-24 h-24 sm:w-32 sm:h-32 rounded-2xl flex items-center justify-center overflow-hidden ring-4 shrink-0 ${
            isFlaggedCheater
              ? 'bg-gradient-to-br from-red-500 to-orange-500 ring-red-900/50'
              : 'bg-gradient-to-br from-violet-500 to-blue-500 ring-gray-800'
          }`}>
            {player.avatarUrl ? (
              <img
                src={player.avatarUrl}
                alt={player.displayName}
                className="w-full h-full object-cover"
              />
            ) : (
              <User className="w-12 h-12 sm:w-16 sm:h-16 text-white/80" />
            )}
          </div>

          {/* Info */}
          <div className="flex-1 text-center sm:text-left">
            <div className="flex flex-col sm:flex-row items-center sm:items-start gap-3 mb-2">
              <h1 className="text-2xl sm:text-3xl font-bold text-white">
                {player.displayName}
              </h1>
              {/* Cheater Badge (032-cheat-detection) */}
              {isFlaggedCheater && (
                <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-red-500/20 border border-red-500/30 text-red-400 text-sm font-semibold animate-pulse">
                  <AlertTriangle className="w-4 h-4" />
                  Cheater Detected
                </span>
              )}
            </div>

            {/* Platform badge */}
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gray-800/60 border border-gray-700/50 mb-4">
              <span className="text-violet-400">{platformIcon}</span>
              <span className="text-sm text-gray-300">{platformLabel}</span>
            </div>

            {/* Quick stats */}
            <div className="grid grid-cols-3 sm:grid-cols-6 gap-3 mt-4">
              <QuickStat label="Matches" value={player.stats.totalMatches} />
              <QuickStat label="Goals" value={player.stats.totalGoals} />
              <QuickStat label="Assists" value={player.stats.totalAssists} />
              <QuickStat label="Saves" value={player.stats.totalSaves} />
              <QuickStat label="Shots" value={player.stats.totalShots} />
              <QuickStat label="Demos" value={player.stats.totalDemos} />
            </div>

            {/* Dates */}
            <div className="flex flex-col sm:flex-row gap-2 sm:gap-6 mt-4 text-xs text-gray-500">
              <span>First seen: {formatDate(player.firstSeenAt)}</span>
              <span>Last seen: {formatDate(player.lastSeenAt)}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function QuickStat({ label, value }: { label: string; value: number }) {
  return (
    <div className="text-center p-2 rounded-lg bg-gray-800/30">
      <div className="text-lg sm:text-xl font-bold text-white">
        {value.toLocaleString()}
      </div>
      <div className="text-xs text-gray-500">{label}</div>
    </div>
  );
}
