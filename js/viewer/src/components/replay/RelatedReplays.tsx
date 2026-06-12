import { Link } from 'react-router-dom';
import { GitBranch, Star, Calendar, ExternalLink } from 'lucide-react';
import { cn } from '@/lib/utils';

/**
 * Related replay information (033-replay-duplicate-detection / T033)
 */
export interface RelatedReplay {
  id: string;
  title: string | null;
  qualityScore: number | null;
  createdAt: string | Date;
}

interface RelatedReplaysProps {
  relatedReplays: RelatedReplay[];
  className?: string;
}

/**
 * Displays other versions of the same match (same matchGuid)
 */
export function RelatedReplays({ relatedReplays, className }: RelatedReplaysProps) {
  if (relatedReplays.length === 0) {
    return null;
  }

  return (
    <div
      className={cn(
        'rounded-xl border border-violet-500/20 bg-violet-500/5 overflow-hidden',
        className
      )}
    >
      <div className="px-4 py-3 bg-violet-500/10 border-b border-violet-500/20">
        <h3 className="font-semibold text-violet-400 flex items-center gap-2">
          <GitBranch className="w-4 h-4" />
          Other Versions of This Match
          <span className="text-xs px-2 py-0.5 rounded-full bg-violet-500/20 text-violet-300">
            {relatedReplays.length}
          </span>
        </h3>
        <p className="text-xs text-gray-500 mt-1">
          Other players uploaded their replay of the same match
        </p>
      </div>

      <div className="divide-y divide-gray-800/50">
        {relatedReplays.map((replay) => {
          const createdDate = new Date(replay.createdAt);
          const formattedDate = createdDate.toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric',
          });

          return (
            <Link
              key={replay.id}
              to={`/replays/${replay.id}`}
              className="flex items-center justify-between p-3 hover:bg-violet-500/10 transition-colors group"
            >
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium text-gray-200 truncate group-hover:text-white">
                  {replay.title || 'Untitled Replay'}
                </p>
                <div className="flex items-center gap-3 mt-1 text-xs text-gray-500">
                  <span className="flex items-center gap-1">
                    <Calendar className="w-3 h-3" />
                    {formattedDate}
                  </span>
                  {replay.qualityScore !== null && (
                    <span className="flex items-center gap-1">
                      <Star className="w-3 h-3" />
                      Quality: {replay.qualityScore}%
                    </span>
                  )}
                </div>
              </div>
              <ExternalLink className="w-4 h-4 text-gray-500 group-hover:text-violet-400 transition-colors flex-shrink-0 ml-3" />
            </Link>
          );
        })}
      </div>
    </div>
  );
}
