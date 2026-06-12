/**
 * CheatDetectionBadge - Compact badge for replay cards
 * Shows cheater status indicator
 */

import { Shield, AlertTriangle, Loader2 } from 'lucide-react';
import { cn } from '../../lib/utils';
import type { CheatAnalysisStatus } from '../../types/cheat';

interface CheatDetectionBadgeProps {
  hasCheater: boolean;
  status: CheatAnalysisStatus;
  cheaterCount?: number;
  size?: 'sm' | 'md';
  showText?: boolean;
  showClean?: boolean;
  className?: string;
}

export function CheatDetectionBadge({
  hasCheater,
  status,
  cheaterCount = 0,
  size = 'sm',
  showText = false,
  showClean = false,
  className,
}: CheatDetectionBadgeProps) {
  // Don't show anything for pending status
  if (status === 'pending') {
    return null;
  }

  // Don't show clean badge unless explicitly requested
  if (!hasCheater && status === 'completed' && !showClean) {
    return null;
  }

  // Show loading state for analyzing
  if (status === 'analyzing') {
    return (
      <div
        className={cn(
          'inline-flex items-center gap-1 rounded-full bg-blue-500/20 px-2 py-0.5',
          size === 'md' && 'px-3 py-1',
          className
        )}
      >
        <Loader2
          className={cn(
            'animate-spin text-blue-500',
            size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'
          )}
        />
        {showText && (
          <span className={cn('text-blue-500', size === 'sm' ? 'text-xs' : 'text-sm')}>
            Analyzing
          </span>
        )}
      </div>
    );
  }

  // Show error state
  if (status === 'error' || status === 'unable_to_analyze') {
    return null; // Don't show badge for errors
  }

  // Cheater detected
  if (hasCheater) {
    return (
      <div
        className={cn(
          'inline-flex items-center gap-1 rounded-full bg-red-500/20 px-2 py-0.5',
          size === 'md' && 'px-3 py-1',
          className
        )}
        title={`${cheaterCount} cheater${cheaterCount !== 1 ? 's' : ''} detected`}
      >
        <AlertTriangle
          className={cn(
            'text-red-500',
            size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'
          )}
        />
        {showText && (
          <span className={cn('font-medium text-red-500', size === 'sm' ? 'text-xs' : 'text-sm')}>
            {cheaterCount > 1 ? `${cheaterCount} Cheaters` : 'Cheater'}
          </span>
        )}
      </div>
    );
  }

  // Clean replay
  return (
    <div
      className={cn(
        'inline-flex items-center gap-1 rounded-full bg-green-500/20 px-2 py-0.5',
        size === 'md' && 'px-3 py-1',
        className
      )}
      title="No cheaters detected"
    >
      <Shield
        className={cn(
          'text-green-500',
          size === 'sm' ? 'h-3 w-3' : 'h-4 w-4'
        )}
      />
      {showText && (
        <span className={cn('text-green-500', size === 'sm' ? 'text-xs' : 'text-sm')}>
          Clean
        </span>
      )}
    </div>
  );
}
