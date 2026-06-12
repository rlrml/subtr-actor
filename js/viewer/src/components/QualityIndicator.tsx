import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Activity, ChevronDown, ChevronUp, Zap, BarChart3, HelpCircle } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { QualityMetrics, QualityCategory } from '@/types/quality';
import { getQualityCategory } from '@/types/quality';

interface QualityIndicatorProps {
  score: number | null | undefined;
  metrics?: QualityMetrics | null;
  showDetails?: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const categoryConfig: Record<QualityCategory, { label: string; color: string; bgColor: string; borderColor: string }> = {
  good: {
    label: 'Good Quality',
    color: 'text-green-400',
    bgColor: 'bg-green-500/10',
    borderColor: 'border-green-500/30',
  },
  medium: {
    label: 'Acceptable Quality',
    color: 'text-amber-400',
    bgColor: 'bg-amber-500/10',
    borderColor: 'border-amber-500/30',
  },
  bad: {
    label: 'Low Quality',
    color: 'text-red-400',
    bgColor: 'bg-red-500/10',
    borderColor: 'border-red-500/30',
  },
};

export function QualityIndicator({
  score,
  metrics,
  showDetails = false,
  size = 'md',
  className,
}: QualityIndicatorProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Handle null/undefined score
  if (score === null || score === undefined) {
    return (
      <div
        className={cn(
          'inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gray-500/10 border border-gray-500/30',
          size === 'sm' && 'px-2 py-1 text-xs',
          size === 'lg' && 'px-4 py-2',
          className
        )}
      >
        <HelpCircle className={cn('w-4 h-4 text-gray-400', size === 'sm' && 'w-3 h-3')} />
        <span className="text-gray-400">Not analyzed</span>
      </div>
    );
  }

  const category = getQualityCategory(score);
  const config = categoryConfig[category];

  const sizeClasses = {
    sm: 'px-2 py-1 text-xs gap-1.5',
    md: 'px-3 py-1.5 text-sm gap-2',
    lg: 'px-4 py-2 gap-2',
  };

  const iconSizes = {
    sm: 'w-3 h-3',
    md: 'w-4 h-4',
    lg: 'w-5 h-5',
  };

  // Simple badge without expansion
  if (!showDetails || !metrics) {
    return (
      <div
        className={cn(
          'inline-flex items-center rounded-lg border',
          config.bgColor,
          config.borderColor,
          sizeClasses[size],
          className
        )}
        title={`${config.label}: ${score}%`}
      >
        <Activity className={cn(iconSizes[size], config.color)} />
        <span className={cn('font-medium', config.color)}>{score}%</span>
      </div>
    );
  }

  // Expandable badge with details
  return (
    <div className={cn('rounded-xl border overflow-hidden', config.bgColor, config.borderColor, className)}>
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className={cn(
          'w-full flex items-center justify-between gap-3 px-4 py-3 transition-colors',
          'hover:bg-white/5'
        )}
      >
        <div className="flex items-center gap-3">
          <div className={cn('w-10 h-10 rounded-lg flex items-center justify-center', config.bgColor)}>
            <Activity className={cn('w-5 h-5', config.color)} />
          </div>
          <div className="text-left">
            <div className={cn('font-semibold', config.color)}>{config.label}</div>
            <div className="text-sm text-gray-400">Score: {score}%</div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <span className={cn('text-2xl font-bold', config.color)}>{score}%</span>
          {isExpanded ? (
            <ChevronUp className="w-4 h-4 text-gray-400" />
          ) : (
            <ChevronDown className="w-4 h-4 text-gray-400" />
          )}
        </div>
      </button>

      {isExpanded && (
        <div className="px-4 pb-4 pt-2 border-t border-gray-700/50 space-y-4">
          {/* Metrics grid */}
          <div className="grid grid-cols-3 gap-3">
            {/* Bad Frames */}
            <div className="p-3 rounded-lg bg-gray-800/50 border border-gray-700/50">
              <div className="flex items-center gap-2 mb-1">
                <Activity className="w-4 h-4 text-orange-400" />
                <span className="text-xs text-gray-400">Bad Frames</span>
              </div>
              <div className="text-lg font-semibold text-white">{metrics.badFrameCount}</div>
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
              <div className="text-xs text-gray-500">{metrics.gapFrameCount} frames missing</div>
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

          {/* Explanation */}
          <p className="text-xs text-gray-500">
            {category === 'good'
              ? 'This replay has excellent data quality and should play back smoothly.'
              : category === 'medium'
                ? 'This replay has some data quality issues. You may notice occasional stuttering.'
                : 'This replay has significant data quality issues. Objects may appear to teleport or move erratically.'}
            {' '}
            <Link
              to="/faq/replay-quality"
              className="text-violet-400 hover:underline inline-flex items-center gap-1"
            >
              Learn more <HelpCircle className="w-3 h-3" />
            </Link>
          </p>

          {/* Frame info */}
          <div className="text-xs text-gray-500 flex items-center justify-between">
            <span>Analyzed {metrics.analyzedFrames.toLocaleString()} of {metrics.totalFrames.toLocaleString()} frames</span>
            <span>Framework v{metrics.frameworkVersion}</span>
          </div>
        </div>
      )}
    </div>
  );
}
