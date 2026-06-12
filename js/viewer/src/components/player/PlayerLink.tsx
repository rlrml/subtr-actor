/**
 * PlayerLink - Link to player profile with hover preview
 * (018-stats-compiler)
 */

import { useState, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { ExternalLink } from 'lucide-react';
import { FaSteam, FaPlaystation, FaXbox } from 'react-icons/fa';
import { SiEpicgames, SiNintendoswitch } from 'react-icons/si';
import { api } from '@/services/api';

interface PlayerLinkProps {
  /** Player ID from the players table (preferred) */
  playerId?: string;
  /** Platform identifier (Steam, Epic, etc.) */
  platform?: string;
  /** Platform-specific player ID */
  platformId?: string;
  /** Display name to show */
  name: string;
  /** Team number for styling (0 = blue, 1 = orange) */
  team?: number;
  /** Additional CSS classes */
  className?: string;
  /** Show external link icon */
  showIcon?: boolean;
  /** Show platform badge/icon */
  showPlatform?: boolean;
  /** Compact mode - reduces spacing */
  compact?: boolean;
}

/** Get platform icon component */
function PlatformIcon({ platform, size }: { platform: string; size: number }) {
  const iconClass = "opacity-70";

  switch (platform) {
    case 'Steam':
      return <FaSteam className={iconClass} size={size} />;
    case 'Epic':
      return <SiEpicgames className={iconClass} size={size} />;
    case 'PS4':
    case 'PS5':
      return <FaPlaystation className={iconClass} size={size} />;
    case 'Xbox':
      return <FaXbox className={iconClass} size={size} />;
    case 'Switch':
      return <SiNintendoswitch className={iconClass} size={size} />;
    default:
      return <span className="opacity-60 text-[10px]">{platform.charAt(0)}</span>;
  }
}

interface PlayerLookupResult {
  player: {
    id: string;
    displayName: string;
    platform: string;
    stats: {
      totalMatches: number;
      totalGoals: number;
    };
  };
}

/**
 * Renders a player name as a link to their profile.
 * Can resolve player ID from platform + platformId if needed.
 */
export function PlayerLink({
  playerId,
  platform,
  platformId,
  name,
  team,
  className = '',
  showIcon = false,
  showPlatform = false,
  compact = false,
}: PlayerLinkProps) {
  // Render platform badge if requested
  const platformBadge = showPlatform && platform ? (
    <span title={platform}>
      <PlatformIcon platform={platform} size={compact ? 10 : 12} />
    </span>
  ) : null;
  const [resolvedId, setResolvedId] = useState<string | null>(playerId || null);
  const [isLoading, setIsLoading] = useState(false);
  const [notFound, setNotFound] = useState(false);

  // Determine styling based on team
  const teamColorClass = team === 0
    ? 'text-blue-400 hover:text-blue-300'
    : team === 1
      ? 'text-orange-400 hover:text-orange-300'
      : 'text-violet-400 hover:text-violet-300';

  // Lazy load player ID on click if we don't have it
  const handleClick = useCallback(async (e: React.MouseEvent) => {
    // If we already have an ID, let the link work normally
    if (resolvedId) return;

    // If we can't look up, show name only (no link)
    if (!platform || !platformId) return;

    // Prevent navigation while we look up the player
    e.preventDefault();
    setIsLoading(true);

    try {
      const result = await api.get<PlayerLookupResult>(
        `/players/lookup?platform=${encodeURIComponent(platform)}&platformId=${encodeURIComponent(platformId)}`
      );
      setResolvedId(result.player.id);
      // Navigate after resolution
      window.location.href = `/players/${result.player.id}`;
    } catch {
      // Player not found in stats system
      setNotFound(true);
    } finally {
      setIsLoading(false);
    }
  }, [resolvedId, platform, platformId]);

  // If player not in stats system, show as plain text
  if (notFound) {
    return (
      <span className={`inline-flex items-center gap-1 ${className} text-gray-400`} title="Player stats not available">
        {platformBadge}
        {name}
      </span>
    );
  }

  // If we don't have enough info to link, show plain text
  if (!playerId && (!platform || !platformId)) {
    return (
      <span className={`inline-flex items-center gap-1 ${className} ${teamColorClass}`}>
        {platformBadge}
        {name}
      </span>
    );
  }

  // If we have a resolved ID, show direct link
  if (resolvedId) {
    return (
      <Link
        to={`/players/${resolvedId}`}
        className={`inline-flex items-center gap-1 ${teamColorClass} hover:underline transition-colors ${className}`}
        title="View player profile"
      >
        {platformBadge}
        {name}
        {showIcon && <ExternalLink className="w-3 h-3 opacity-60" />}
      </Link>
    );
  }

  // Show link that resolves on click
  return (
    <button
      onClick={handleClick}
      disabled={isLoading}
      className={`inline-flex items-center gap-1 ${teamColorClass} hover:underline transition-colors ${className} ${isLoading ? 'opacity-50 cursor-wait' : 'cursor-pointer'}`}
      title="View player profile"
    >
      {isLoading ? (
        <span className="animate-pulse">Loading...</span>
      ) : (
        <>
          {platformBadge}
          {name}
          {showIcon && <ExternalLink className="w-3 h-3 opacity-60" />}
        </>
      )}
    </button>
  );
}

/**
 * Compact version for use in tables/lists
 */
export function PlayerLinkCompact({
  playerId,
  platform,
  platformId,
  name,
  team,
}: Omit<PlayerLinkProps, 'className' | 'showIcon'>) {
  return (
    <PlayerLink
      playerId={playerId}
      platform={platform}
      platformId={platformId}
      name={name}
      team={team}
      className="text-sm font-medium"
    />
  );
}
