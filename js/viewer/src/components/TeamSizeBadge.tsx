import { Users } from 'lucide-react';

interface TeamSizeBadgeProps {
  teamSize?: number | null;
  /** Number of players (used as fallback when teamSize is not available) */
  playerCount?: number;
  /** Visual style variant */
  variant?: 'default' | 'compact';
  className?: string;
}

/**
 * Displays the match format (1v1, 2v2, 3v3, 4v4) as a badge.
 * Falls back to computing from player count if teamSize is not available.
 */
export function TeamSizeBadge({
  teamSize,
  playerCount,
  variant = 'default',
  className = ''
}: TeamSizeBadgeProps) {
  // Determine team size: use teamSize if available, otherwise compute from player count
  let size: number | null = null;

  if (teamSize !== undefined && teamSize !== null && teamSize >= 1 && teamSize <= 4) {
    size = teamSize;
  } else if (playerCount !== undefined && playerCount > 0) {
    // Compute from player count (assumes equal teams)
    size = Math.ceil(playerCount / 2);
    if (size > 4) size = 4; // Cap at 4v4
  }

  // Don't render if we can't determine the size
  if (size === null) {
    return null;
  }

  const formatLabel = `${size}v${size}`;

  // Color coding by team size
  const colorClasses = {
    1: 'bg-violet-500/20 text-violet-300 border-violet-500/30',
    2: 'bg-blue-500/20 text-blue-300 border-blue-500/30',
    3: 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30',
    4: 'bg-amber-500/20 text-amber-300 border-amber-500/30',
  }[size] || 'bg-gray-500/20 text-gray-300 border-gray-500/30';

  if (variant === 'compact') {
    return (
      <span
        className={`inline-flex items-center px-1.5 py-0.5 text-[10px] font-bold rounded border ${colorClasses} ${className}`}
        title={`${size} vs ${size} match`}
      >
        {formatLabel}
      </span>
    );
  }

  return (
    <div
      className={`inline-flex items-center gap-1 px-2 py-1 text-xs font-medium rounded-md border ${colorClasses} ${className}`}
      title={`${size} vs ${size} match`}
    >
      <Users className="w-3 h-3" />
      <span>{formatLabel}</span>
    </div>
  );
}
