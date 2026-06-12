/**
 * CheatDetectionAlert - Prominent banner for replay detail page
 * Shows when cheaters are detected in the replay
 */

import { AlertTriangle, ExternalLink, Loader2, Shield } from 'lucide-react';
import { Link } from 'react-router-dom';
import type { CheatAnalysisStatus } from '../../types/cheat';
import { CHEAT_ATTRIBUTION } from '../../api/cheat';

interface CheatDetectionAlertProps {
  status: CheatAnalysisStatus;
  hasCheater: boolean;
  cheaterCount: number;
  error?: string;
}

export function CheatDetectionAlert({
  status,
  hasCheater,
  cheaterCount,
  error,
}: CheatDetectionAlertProps) {
  // Pending - don't show anything
  if (status === 'pending') {
    return null;
  }

  // Analyzing - show progress indicator
  if (status === 'analyzing') {
    return (
      <div className="mb-4 flex items-center gap-3 rounded-lg border border-blue-500/30 bg-blue-500/10 px-4 py-3">
        <Loader2 className="h-5 w-5 animate-spin text-blue-400" />
        <div>
          <p className="font-medium text-blue-300">Analyzing for cheats...</p>
          <p className="text-sm text-blue-400/70">
            This replay is being analyzed by{' '}
            <a
              href={CHEAT_ATTRIBUTION.url}
              target="_blank"
              rel="noopener noreferrer"
              className="underline hover:text-blue-300"
            >
              {CHEAT_ATTRIBUTION.service}
            </a>
          </p>
        </div>
      </div>
    );
  }

  // Error - show warning but don't block
  if (status === 'error') {
    return (
      <div className="mb-4 flex items-center gap-3 rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3">
        <AlertTriangle className="h-5 w-5 text-amber-400" />
        <div>
          <p className="font-medium text-amber-300">Cheat analysis failed</p>
          <p className="text-sm text-amber-400/70">
            {error || 'The analysis will be retried automatically.'}
          </p>
        </div>
      </div>
    );
  }

  // Unable to analyze - show info
  if (status === 'unable_to_analyze') {
    return (
      <div className="mb-4 flex items-center gap-3 rounded-lg border border-gray-500/30 bg-gray-500/10 px-4 py-3">
        <AlertTriangle className="h-5 w-5 text-gray-400" />
        <div>
          <p className="font-medium text-gray-300">Unable to analyze</p>
          <p className="text-sm text-gray-400">
            This replay could not be analyzed for cheats.
          </p>
        </div>
      </div>
    );
  }

  // Completed - no cheaters (show clean banner)
  if (!hasCheater) {
    return (
      <div className="mb-4 flex items-center gap-3 rounded-lg border border-green-500/30 bg-green-500/10 px-4 py-3">
        <Shield className="h-5 w-5 text-green-400" />
        <div>
          <p className="font-medium text-green-300">No cheaters detected</p>
          <p className="text-sm text-green-400/70">
            This replay has been analyzed by{' '}
            <a
              href={CHEAT_ATTRIBUTION.url}
              target="_blank"
              rel="noopener noreferrer"
              className="underline hover:text-green-300"
            >
              {CHEAT_ATTRIBUTION.service}
            </a>
            {' '}and no unauthorized modifications were found.
          </p>
        </div>
      </div>
    );
  }

  // Cheaters detected - show prominent warning
  return (
    <div className="mb-4 rounded-lg border border-red-500/50 bg-red-500/10">
      <div className="flex items-start gap-4 px-4 py-4">
        <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-red-500/20">
          <AlertTriangle className="h-5 w-5 text-red-400" />
        </div>
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-red-300">
            {cheaterCount} Cheater{cheaterCount !== 1 ? 's' : ''} Detected
          </h3>
          <p className="mt-1 text-sm text-red-400/80">
            Our anti-cheat system has detected {cheaterCount === 1 ? 'a player' : 'players'} using
            unauthorized modifications in this match.{' '}
            <Link to="/cheaters" className="underline hover:text-red-300">
              View all detected cheaters
            </Link>
          </p>
          <p className="mt-2 text-xs text-gray-500">
            Powered by{' '}
            <a
              href={CHEAT_ATTRIBUTION.url}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-gray-400 hover:text-gray-300"
            >
              {CHEAT_ATTRIBUTION.service}
              <ExternalLink className="h-3 w-3" />
            </a>
          </p>
        </div>
      </div>
    </div>
  );
}
