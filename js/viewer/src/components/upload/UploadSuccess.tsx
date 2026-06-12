import { Link } from 'react-router-dom';
import { CheckCircle, Play, FileText, Upload, ExternalLink, GitBranch } from 'lucide-react';
import { cn } from '@/lib/utils';

/**
 * Related replay info from upload response (033-replay-duplicate-detection)
 */
export interface RelatedReplayInfo {
  id: string;
  title: string | null;
  qualityScore: number | null;
  createdAt: string | Date;
}

interface UploadSuccessProps {
  /** The ID of the successfully uploaded replay */
  replayId: string;
  /** Optional replay title to display */
  title?: string | null;
  /** Related replays from the same match (T036) */
  relatedReplays?: RelatedReplayInfo[];
  /** Callback when user wants to upload another replay */
  onUploadAnother?: () => void;
  className?: string;
}

/**
 * Success state shown after a replay is successfully uploaded (033-replay-duplicate-detection / US2)
 * Keeps user on page with clear navigation options instead of auto-redirecting
 */
export function UploadSuccess({
  replayId,
  title,
  relatedReplays = [],
  onUploadAnother,
  className,
}: UploadSuccessProps) {
  return (
    <div
      className={cn(
        'relative overflow-hidden rounded-xl border',
        'bg-gradient-to-r from-green-500/10 via-emerald-500/10 to-green-500/10 border-green-500/30',
        className
      )}
    >
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_left,_var(--tw-gradient-stops))] from-green-500/10 via-transparent to-transparent" />

      <div className="relative p-6">
        <div className="flex items-start gap-4">
          <div className="w-12 h-12 rounded-xl bg-green-500/20 flex items-center justify-center flex-shrink-0">
            <CheckCircle className="w-6 h-6 text-green-400" />
          </div>

          <div className="flex-1 min-w-0">
            <h3 className="font-semibold text-green-400 text-lg flex items-center gap-2">
              Upload Successful
            </h3>

            <p className="text-sm text-gray-400 mt-1">
              {title ? (
                <>Your replay <span className="text-white font-medium">"{title}"</span> has been processed and is ready to view.</>
              ) : (
                <>Your replay has been processed and is ready to view.</>
              )}
            </p>

            {/* Navigation buttons */}
            <div className="flex flex-wrap items-center gap-3 mt-5">
              {/* Primary action: Watch replay */}
              <Link
                to={`/viewer/${replayId}`}
                className="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-violet-600 to-blue-600 text-white hover:from-violet-500 hover:to-blue-500 transition-all text-sm font-medium shadow-lg shadow-violet-500/20"
              >
                <Play className="w-4 h-4" />
                Regarder le replay
              </Link>

              {/* Secondary action: View details */}
              <Link
                to={`/replays/${replayId}`}
                className="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gray-700/50 text-gray-200 hover:bg-gray-700 hover:text-white transition-all text-sm font-medium border border-gray-600/50"
              >
                <FileText className="w-4 h-4" />
                Voir les détails
              </Link>

              {/* Tertiary action: Upload another */}
              {onUploadAnother && (
                <button
                  onClick={onUploadAnother}
                  className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl text-gray-400 hover:text-gray-200 hover:bg-gray-800/50 transition-all text-sm"
                >
                  <Upload className="w-4 h-4" />
                  Upload another
                </button>
              )}
            </div>

            {/* Related replays from same match (T036) */}
            {relatedReplays.length > 0 && (
              <div className="mt-4 pt-4 border-t border-gray-700/50">
                <div className="flex items-center gap-2 text-sm text-violet-400 mb-2">
                  <GitBranch className="w-4 h-4" />
                  <span className="font-medium">
                    {relatedReplays.length} other {relatedReplays.length === 1 ? 'version' : 'versions'} of this match
                  </span>
                </div>
                <div className="flex flex-wrap gap-2">
                  {relatedReplays.slice(0, 3).map((related) => (
                    <Link
                      key={related.id}
                      to={`/replays/${related.id}`}
                      className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg bg-violet-500/10 text-violet-300 hover:bg-violet-500/20 transition-colors border border-violet-500/20"
                    >
                      {related.title || 'Untitled'}
                      {related.qualityScore !== null && (
                        <span className="text-violet-400/60">({related.qualityScore}%)</span>
                      )}
                    </Link>
                  ))}
                  {relatedReplays.length > 3 && (
                    <span className="text-xs text-gray-500 flex items-center">
                      +{relatedReplays.length - 3} more
                    </span>
                  )}
                </div>
              </div>
            )}

            {/* Quick link hint */}
            <div className="mt-4 pt-4 border-t border-gray-700/50">
              <p className="text-xs text-gray-500 flex items-center gap-2">
                <ExternalLink className="w-3.5 h-3.5" />
                Direct link:{' '}
                <code className="text-violet-400 bg-violet-500/10 px-2 py-0.5 rounded">
                  {window.location.origin}/replays/{replayId}
                </code>
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
