import { LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils';

interface MetricCardProps {
  title: string;
  value: number | string;
  subtitle?: string;
  icon?: LucideIcon;
  trend?: {
    value: number;
    label: string;
    positive?: boolean;
  };
  className?: string;
  loading?: boolean;
}

export function MetricCard({
  title,
  value,
  subtitle,
  icon: Icon,
  trend,
  className,
  loading = false,
}: MetricCardProps) {
  if (loading) {
    return (
      <div
        className={cn(
          'bg-gray-900/50 border border-gray-800 rounded-xl p-5',
          className
        )}
      >
        <div className="animate-pulse">
          <div className="h-4 bg-gray-700 rounded w-24 mb-3" />
          <div className="h-8 bg-gray-700 rounded w-16 mb-2" />
          <div className="h-3 bg-gray-700 rounded w-20" />
        </div>
      </div>
    );
  }

  return (
    <div
      className={cn(
        'bg-gray-900/50 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors',
        className
      )}
    >
      <div className="flex items-start justify-between">
        <div>
          <p className="text-sm font-medium text-gray-400">{title}</p>
          <p className="text-3xl font-bold text-white mt-1">
            {typeof value === 'number' ? value.toLocaleString() : value}
          </p>
          {subtitle && (
            <p className="text-sm text-gray-500 mt-1">{subtitle}</p>
          )}
          {trend && (
            <div className="flex items-center gap-1 mt-2">
              <span
                className={cn(
                  'text-sm font-medium',
                  trend.positive ? 'text-green-400' : 'text-red-400'
                )}
              >
                {trend.positive ? '+' : ''}{trend.value}
              </span>
              <span className="text-xs text-gray-500">{trend.label}</span>
            </div>
          )}
        </div>
        {Icon && (
          <div className="p-2.5 bg-gradient-to-br from-violet-600/20 to-blue-600/20 rounded-lg">
            <Icon className="w-5 h-5 text-violet-400" />
          </div>
        )}
      </div>
    </div>
  );
}

export default MetricCard;
