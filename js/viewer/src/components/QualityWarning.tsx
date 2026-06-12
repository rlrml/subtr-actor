import { useState } from 'react';
import { Link } from 'react-router-dom';
import { AlertTriangle, ChevronDown, ChevronUp, Activity, Zap, BarChart3, HelpCircle } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { QualityMetrics } from '@/types/quality';
import { QUALITY_THRESHOLDS } from '@/types/quality';

interface QualityWarningProps {
  score: number;
  metrics?: QualityMetrics | null;
  className?: string;
}

export function QualityWarning({ score, metrics, className }: QualityWarningProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Only show warning if score is below threshold
  if (score >= QUALITY_THRESHOLDS.WARNING) {
    return null;
  }

  const isBad = score < QUALITY_THRESHOLDS.MEDIUM;

  return (
    <div
      className={cn(
        'relative overflow-hidden rounded-xl border',
        isBad
          ? 'bg-gradient-to-r from-red-500/10 via-orange-500/10 to-red-500/10 border-red-500/30'
          : 'bg-gradient-to-r from-amber-500/10 via-orange-500/10 to-amber-500/10 border-amber-500/30',
        className
      )}
    >
      <div
        className={cn(
          'absolute inset-0 bg-[radial-gradient(ellipse_at_left,_var(--tw-gradient-stops))]',
          isBad ? 'from-red-500/10' : 'from-amber-500/10',
          'via-transparent to-transparent'
        )}
      />

      <div className="relative p-4">
        <div className="flex items-start gap-4">
          <div
            className={cn(
              'w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0',
              isBad ? 'bg-red-500/20' : 'bg-amber-500/20'
            )}
          >
            <AlertTriangle className={cn('w-5 h-5', isBad ? 'text-red-400' : 'text-amber-400')} />
          </div>

          <div className="flex-1 min-w-0">
            <div className="flex items-center justify-between gap-2">
              <h3
                className={cn(
                  'font-semibold flex items-center gap-2',
                  isBad ? 'text-red-400' : 'text-amber-400'
                )}
              >
                {isBad ? 'Low Quality Replay' : 'Quality Warning'}
                <span
                  className={cn(
                    'text-xs px-2 py-0.5 rounded-full font-normal',
                    isBad
                      ? 'bg-red-500/20 text-red-300'
                      : 'bg-amber-500/20 text-amber-300'
                  )}
                >
                  {score}%
                </span>
              </h3>

              {metrics && (
                <button
                  onClick={() => setIsExpanded(!isExpanded)}
                  className={cn(
                    'flex items-center gap-1 text-xs px-2 py-1 rounded-lg transition-colors',
                    isBad
                      ? 'text-red-400 hover:bg-red-500/20'
                      : 'text-amber-400 hover:bg-amber-500/20'
                  )}
                >
                  {isExpanded ? 'Less' : 'Details'}
                  {isExpanded ? (
                    <ChevronUp className="w-3 h-3" />
                  ) : (
                    <ChevronDown className="w-3 h-3" />
                  )}
                </button>
              )}
            </div>

            <p className="text-sm text-gray-400 mt-1">
              {isBad
                ? 'This replay has significant data quality issues. The visualization may appear choppy with teleporting objects.'
                : 'This replay has some data quality issues. The visualization may have occasional stutters.'}
              {' '}
              <Link
                to="/faq/replay-quality"
                className="text-violet-400 hover:underline"
              >
                Why?
              </Link>
            </p>

            {/* Expanded details */}
            {isExpanded && metrics && (
              <div className="mt-4 pt-4 border-t border-gray-700/50 space-y-3">
                <div className="grid grid-cols-3 gap-3">
                  {/* Bad Frames */}
                  <div className="p-3 rounded-lg bg-gray-800/50 border border-gray-700/50">
                    <div className="flex items-center gap-2 mb-1">
                      <Activity className="w-4 h-4 text-orange-400" />
                      <span className="text-xs text-gray-400">Bad Frames</span>
                    </div>
                    <div className="text-lg font-semibold text-white">
                      {metrics.badFrameCount}
                    </div>
                    <div className="text-xs text-gray-500">
                      {(metrics.badFrameRate * 100).toFixed(1)}% of frames
                    </div>
                  </div>

                  {/* Gaps */}
                  <div className="p-3 rounded-lg bg-gray-800/50 border border-gray-700/50">
                    <div className="flex items-center gap-2 mb-1">
                      <Zap className="w-4 h-4 text-yellow-400" />
                      <span className="text-xs text-gray-400">Data Gaps</span>
                    </div>
                    <div className="text-lg font-semibold text-white">{metrics.gapCount}</div>
                    <div className="text-xs text-gray-500">
                      {metrics.gapFrameCount} frames missing
                    </div>
                  </div>

                  {/* Velocity Error */}
                  <div className="p-3 rounded-lg bg-gray-800/50 border border-gray-700/50">
                    <div className="flex items-center gap-2 mb-1">
                      <BarChart3 className="w-4 h-4 text-blue-400" />
                      <span className="text-xs text-gray-400">Velocity Error</span>
                    </div>
                    <div className="text-lg font-semibold text-white">
                      {metrics.avgVelocityError.toFixed(1)}%
                    </div>
                    <div className="text-xs text-gray-500">average deviation</div>
                  </div>
                </div>

                <p className="text-xs text-gray-500">
                  Quality issues are caused by how Rocket League records replay data.
                  We apply frame filtering to improve quality automatically.{' '}
                  <Link
                    to="/faq/replay-quality"
                    className="text-violet-400 hover:underline inline-flex items-center gap-1"
                  >
                    Learn more <HelpCircle className="w-3 h-3" />
                  </Link>
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
