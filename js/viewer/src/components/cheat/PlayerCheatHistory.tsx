/**
 * PlayerCheatHistory - Displays a player's cheat detection history
 * Shows a list of replays where this player was flagged with confidence scores
 */

import { Link } from 'react-router-dom';
import { AlertTriangle, Shield, ExternalLink, Play, Calendar } from 'lucide-react';
import { cn } from '../../lib/utils';
import type { PlayerCheatHistory as PlayerCheatHistoryType } from '../../types/cheat';
import { CHEAT_ATTRIBUTION, getConfidenceLevel } from '../../api/cheat';

interface PlayerCheatHistoryProps {
  history: PlayerCheatHistoryType | null;
  loading?: boolean;
  className?: string;
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString('en-US', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  });
}

export function PlayerCheatHistory({ history, loading, className }: PlayerCheatHistoryProps) {
  if (loading) {
    return (
      <div className={cn('rounded-lg border border-gray-700 bg-gray-800/50 p-4', className)}>
        <h3 className="mb-3 flex items-center gap-2 font-semibold text-white">
          <Shield className="h-5 w-5 text-gray-400" />
          Cheat Analysis
        </h3>
        <div className="animate-pulse space-y-3">
          <div className="h-4 w-3/4 rounded bg-gray-700" />
          <div className="h-20 rounded bg-gray-700" />
        </div>
      </div>
    );
  }

  if (!history) {
    return null;
  }

  // Player has never been flagged
  if (!history.isFlaggedCheater && history.flaggedReplays.length === 0) {
    return (
      <div className={cn('rounded-lg border border-gray-700 bg-gray-800/50 p-4', className)}>
        <h3 className="mb-3 flex items-center gap-2 font-semibold text-white">
          <Shield className="h-5 w-5 text-green-400" />
          Clean Record
        </h3>
        <p className="text-sm text-gray-400">
          This player has no cheat detections in analyzed replays.
        </p>
        <div className="mt-3 border-t border-gray-700 pt-3">
          <p className="text-xs text-gray-500">
            Powered by{' '}
            <a
              href={CHEAT_ATTRIBUTION.url}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-gray-400 hover:text-white"
            >
              {CHEAT_ATTRIBUTION.service}
              <ExternalLink className="h-3 w-3" />
            </a>
          </p>
        </div>
      </div>
    );
  }

  // Player has been flagged
  return (
    <div className={cn('rounded-lg border border-red-500/30 bg-red-500/5 p-4', className)}>
      {/* Header */}
      <div className="mb-4 flex items-center justify-between">
        <h3 className="flex items-center gap-2 font-semibold text-white">
          <AlertTriangle className="h-5 w-5 text-red-400" />
          Cheat History
        </h3>
        <span className="rounded-full bg-red-500/20 px-2 py-0.5 text-xs font-medium text-red-400">
          {history.flaggedReplayCount} {history.flaggedReplayCount === 1 ? 'detection' : 'detections'}
        </span>
      </div>

      {/* Summary */}
      <div className="mb-4 grid grid-cols-2 gap-3">
        <div className="rounded-lg bg-gray-800/50 p-3 text-center">
          <div className="text-lg font-bold text-red-400">{history.flaggedReplayCount}</div>
          <div className="text-xs text-gray-500">Flagged Matches</div>
        </div>
        <div className="rounded-lg bg-gray-800/50 p-3 text-center">
          <div className="text-lg font-bold text-amber-400">
            {history.flaggedReplays.length > 0
              ? (history.flaggedReplays.reduce((sum, r) => sum + r.confidencePercent, 0) / history.flaggedReplays.length).toFixed(1)
              : '--'}%
          </div>
          <div className="text-xs text-gray-500">Avg Confidence</div>
        </div>
      </div>

      {/* Flagged Replays List */}
      {history.flaggedReplays.length > 0 && (
        <div className="space-y-2">
          <h4 className="text-xs font-medium uppercase text-gray-400">Flagged Replays</h4>
          <div className="max-h-48 space-y-2 overflow-y-auto">
            {history.flaggedReplays.map((replay) => {
              const { colorClass } = getConfidenceLevel(replay.confidencePercent);
              return (
                <Link
                  key={replay.replayId}
                  to={`/replays/${replay.replayId}`}
                  className="flex items-center justify-between rounded-lg bg-gray-800/50 p-2 transition-colors hover:bg-gray-700/50"
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <Play className="h-4 w-4 flex-shrink-0 text-gray-500" />
                    <span className="truncate text-sm text-white">
                      {replay.replayTitle || 'Untitled Replay'}
                    </span>
                  </div>
                  <div className="flex items-center gap-3 flex-shrink-0">
                    {replay.playedAt && (
                      <span className="flex items-center gap-1 text-xs text-gray-500">
                        <Calendar className="h-3 w-3" />
                        {formatDate(replay.playedAt)}
                      </span>
                    )}
                    <span className={cn('text-xs font-semibold', colorClass)}>
                      {replay.confidencePercent.toFixed(0)}%
                    </span>
                  </div>
                </Link>
              );
            })}
          </div>
        </div>
      )}

      {/* Timeline */}
      {history.firstFlaggedAt && (
        <div className="mt-4 border-t border-gray-700 pt-3 text-xs text-gray-500">
          <div className="flex justify-between">
            <span>First detected:</span>
            <span>{formatDate(history.firstFlaggedAt)}</span>
          </div>
          {history.lastFlaggedAt && history.lastFlaggedAt !== history.firstFlaggedAt && (
            <div className="mt-1 flex justify-between">
              <span>Last detected:</span>
              <span>{formatDate(history.lastFlaggedAt)}</span>
            </div>
          )}
        </div>
      )}

      {/* Attribution footer */}
      <div className="mt-4 border-t border-gray-700 pt-3">
        <p className="text-xs text-gray-500">
          Powered by{' '}
          <a
            href={CHEAT_ATTRIBUTION.url}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-gray-400 hover:text-white"
          >
            {CHEAT_ATTRIBUTION.service}
            <ExternalLink className="h-3 w-3" />
          </a>
        </p>
      </div>
    </div>
  );
}
