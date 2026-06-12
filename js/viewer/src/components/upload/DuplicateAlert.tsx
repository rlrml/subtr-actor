import { Link } from 'react-router-dom';
import { Copy, ExternalLink, AlertCircle } from 'lucide-react';
import { cn } from '@/lib/utils';

/**
 * Duplicate replay information from the API (033-replay-duplicate-detection)
 */
export interface DuplicateInfo {
  /** The existing replay's ID (only if public) */
  replayId?: string;
  /** Message explaining the duplicate */
  message: string;
  /** Whether a link can be provided (false if original is unlisted) */
  hasLink: boolean;
}

interface DuplicateAlertProps {
  duplicate: DuplicateInfo;
  className?: string;
  onRetry?: () => void;
}

/**
 * Alert shown when user attempts to upload a replay that already exists
 */
export function DuplicateAlert({ duplicate, className, onRetry }: DuplicateAlertProps) {
  return (
    <div
      className={cn(
        'relative overflow-hidden rounded-xl border',
        'bg-gradient-to-r from-blue-500/10 via-violet-500/10 to-blue-500/10 border-blue-500/30',
        className
      )}
    >
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_left,_var(--tw-gradient-stops))] from-blue-500/10 via-transparent to-transparent" />

      <div className="relative p-4">
        <div className="flex items-start gap-4">
          <div className="w-10 h-10 rounded-lg bg-blue-500/20 flex items-center justify-center flex-shrink-0">
            <Copy className="w-5 h-5 text-blue-400" />
          </div>

          <div className="flex-1 min-w-0">
            <h3 className="font-semibold text-blue-400 flex items-center gap-2">
              Replay Already Exists
            </h3>

            <p className="text-sm text-gray-400 mt-1">
              {duplicate.message}
            </p>

            <div className="flex items-center gap-3 mt-4">
              {/* Show link to existing replay if available */}
              {duplicate.hasLink && duplicate.replayId && (
                <Link
                  to={`/replays/${duplicate.replayId}`}
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-blue-500/20 text-blue-400 hover:bg-blue-500/30 transition-colors text-sm font-medium"
                >
                  <ExternalLink className="w-4 h-4" />
                  View Existing Replay
                </Link>
              )}

              {/* Info about unlisted replay */}
              {!duplicate.hasLink && (
                <div className="flex items-center gap-2 text-xs text-gray-500">
                  <AlertCircle className="w-4 h-4" />
                  <span>The existing replay is unlisted</span>
                </div>
              )}

              {/* Try another file button */}
              {onRetry && (
                <button
                  onClick={onRetry}
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-700/50 text-gray-300 hover:bg-gray-700 transition-colors text-sm font-medium"
                >
                  Upload Different File
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
