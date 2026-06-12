/**
 * CheatDetectionPanel - Detailed sidebar panel with per-player cheat scores
 * Shows analysis results for each player in the replay
 */

import { useState } from 'react';
import { Shield, AlertTriangle, ExternalLink, Loader2, Keyboard, ToggleLeft, Search } from 'lucide-react';
import { cn } from '../../lib/utils';
import { GradientButton } from '../ui/GradientButton';
import type { CheatAnalysisResult, CheatAnalysisPlayer } from '../../types/cheat';
import { CHEAT_ATTRIBUTION, formatPlatformName, getConfidenceLevel, useRequestCheatAnalysis } from '../../api/cheat';

interface CheatDetectionPanelProps {
  analysis: CheatAnalysisResult;
  replayId?: string;
  onAnalysisRequested?: () => void;
  className?: string;
}

function PlayerCheatCard({ player }: { player: CheatAnalysisPlayer }) {
  const { colorClass } = getConfidenceLevel(player.confidencePercent);
  const isCheater = player.isCheater;

  return (
    <div
      className={cn(
        'rounded-lg border p-3',
        isCheater
          ? 'border-red-500/30 bg-red-500/5'
          : 'border-gray-700/50 bg-gray-800/30'
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="truncate font-medium text-white">{player.name}</span>
            <span className="flex-shrink-0 text-xs text-gray-500">
              {formatPlatformName(player.platform)}
            </span>
          </div>
          <div className="mt-1 flex items-center gap-2 text-xs text-gray-400">
            <span className={cn('font-semibold', colorClass)}>
              {player.confidencePercent.toFixed(1)}%
            </span>
            <span>confidence</span>
          </div>
        </div>
        <div className="flex flex-shrink-0 items-center gap-1">
          {isCheater ? (
            <AlertTriangle className="h-4 w-4 text-red-400" />
          ) : (
            <Shield className="h-4 w-4 text-green-400" />
          )}
        </div>
      </div>

      {/* Additional indicators */}
      {(player.isToggling || player.isKbm) && (
        <div className="mt-2 flex flex-wrap gap-2">
          {player.isToggling && (
            <span className="inline-flex items-center gap-1 rounded bg-amber-500/20 px-1.5 py-0.5 text-xs text-amber-400">
              <ToggleLeft className="h-3 w-3" />
              Toggling
            </span>
          )}
          {player.isKbm && (
            <span className="inline-flex items-center gap-1 rounded bg-violet-500/20 px-1.5 py-0.5 text-xs text-violet-400">
              <Keyboard className="h-3 w-3" />
              KBM
            </span>
          )}
        </div>
      )}
    </div>
  );
}

export function CheatDetectionPanel({ analysis, replayId, onAnalysisRequested, className }: CheatDetectionPanelProps) {
  const { status, players, hasCheater, cheaterCount } = analysis;
  const { mutate: requestAnalysis, loading: requestLoading, error: requestError } = useRequestCheatAnalysis();
  const [analysisRequested, setAnalysisRequested] = useState(false);

  const handleRequestAnalysis = async () => {
    if (!replayId) return;
    try {
      await requestAnalysis(replayId);
      setAnalysisRequested(true);
      onAnalysisRequested?.();
    } catch {
      // Error is handled by the hook
    }
  };

  // Pending state
  if (status === 'pending') {
    return (
      <div className={cn('rounded-lg border border-gray-700 bg-gray-800/50 p-4', className)}>
        <h3 className="mb-3 flex items-center gap-2 font-semibold text-white">
          <Shield className="h-5 w-5 text-gray-400" />
          Cheat Analysis
        </h3>
        {analysisRequested ? (
          <div className="space-y-2">
            <p className="text-sm text-violet-400">
              Analysis queued. This may take a few minutes...
            </p>
            <p className="text-xs text-gray-500">
              The page will update automatically when results are ready.
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            <p className="text-sm text-gray-400">
              This replay hasn't been analyzed for cheats yet.
            </p>
            {replayId && (
              <GradientButton
                onClick={handleRequestAnalysis}
                disabled={requestLoading}
                loading={requestLoading}
                className="w-full"
              >
                <Search className="h-4 w-4" />
                {requestLoading ? 'Requesting...' : 'Analyze for Cheats'}
              </GradientButton>
            )}
            {requestError && (
              <p className="text-xs text-red-400">{requestError.message}</p>
            )}
            <p className="text-xs text-gray-500">
              Powered by{' '}
              <a
                href={CHEAT_ATTRIBUTION.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-gray-400 hover:text-white"
              >
                {CHEAT_ATTRIBUTION.service}
              </a>
            </p>
          </div>
        )}
      </div>
    );
  }

  // Analyzing state
  if (status === 'analyzing') {
    return (
      <div className={cn('rounded-lg border border-violet-500/30 bg-gray-800/50 p-4', className)}>
        <h3 className="mb-3 flex items-center gap-2 font-semibold text-white">
          <Loader2 className="h-5 w-5 animate-spin text-violet-400" />
          Analyzing...
        </h3>
        <p className="text-sm text-gray-400">
          Checking players for unauthorized modifications...
        </p>
      </div>
    );
  }

  // Error state
  if (status === 'error' || status === 'unable_to_analyze') {
    return (
      <div className={cn('rounded-lg border border-gray-700 bg-gray-800/50 p-4', className)}>
        <h3 className="mb-3 flex items-center gap-2 font-semibold text-white">
          <Shield className="h-5 w-5 text-gray-500" />
          Cheat Analysis
        </h3>
        <p className="text-sm text-gray-400">
          {status === 'error'
            ? 'Analysis failed. It will be retried automatically.'
            : 'This replay could not be analyzed.'}
        </p>
      </div>
    );
  }

  // Completed state
  const sortedPlayers = [...players].sort(
    (a, b) => b.confidencePercent - a.confidencePercent
  );

  // Group by team
  const team0Players = sortedPlayers.filter((p) => p.team === 0);
  const team1Players = sortedPlayers.filter((p) => p.team === 1);

  return (
    <div className={cn('rounded-lg border border-gray-700 bg-gray-800/50 p-4', className)}>
      {/* Header */}
      <div className="mb-4 flex items-center justify-between">
        <h3 className="flex items-center gap-2 font-semibold text-white">
          {hasCheater ? (
            <>
              <AlertTriangle className="h-5 w-5 text-red-400" />
              <span>{cheaterCount} Cheater{cheaterCount !== 1 ? 's' : ''}</span>
            </>
          ) : (
            <>
              <Shield className="h-5 w-5 text-green-400" />
              <span>Clean Match</span>
            </>
          )}
        </h3>
      </div>

      {/* Players by team */}
      {players.length > 0 && (
        <div className="space-y-4">
          {/* Team Blue */}
          {team0Players.length > 0 && (
            <div>
              <h4 className="mb-2 text-xs font-medium uppercase text-blue-400">
                Blue Team
              </h4>
              <div className="space-y-2">
                {team0Players.map((player) => (
                  <PlayerCheatCard key={player.id} player={player} />
                ))}
              </div>
            </div>
          )}

          {/* Team Orange */}
          {team1Players.length > 0 && (
            <div>
              <h4 className="mb-2 text-xs font-medium uppercase text-orange-400">
                Orange Team
              </h4>
              <div className="space-y-2">
                {team1Players.map((player) => (
                  <PlayerCheatCard key={player.id} player={player} />
                ))}
              </div>
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
        {analysis.analyzedAt && (
          <p className="mt-1 text-xs text-gray-600">
            Analyzed {new Date(analysis.analyzedAt).toLocaleDateString()}
          </p>
        )}
      </div>
    </div>
  );
}
