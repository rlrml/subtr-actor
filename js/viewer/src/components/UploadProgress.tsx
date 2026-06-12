import { CheckCircle, XCircle, Loader2, Sparkles, Cog } from 'lucide-react';
import { cn } from '@/lib/utils';

export type UploadStatus = 'idle' | 'uploading' | 'processing' | 'success' | 'error';

interface UploadProgressProps {
  status: UploadStatus;
  progress?: number;
  error?: string;
}

export function UploadProgress({ status, progress = 0, error }: UploadProgressProps) {
  if (status === 'idle') return null;

  const statusConfig = {
    uploading: {
      icon: Loader2,
      iconClass: 'animate-spin text-violet-400',
      bgClass: 'from-violet-500/20 to-blue-500/20',
      borderClass: 'border-violet-500/30',
      label: 'Uploading your replay...',
      sublabel: 'Please wait while we upload your file',
      showProgress: true,
    },
    processing: {
      icon: Cog,
      iconClass: 'animate-spin text-blue-400',
      bgClass: 'from-blue-500/20 to-cyan-500/20',
      borderClass: 'border-blue-500/30',
      label: 'Processing replay...',
      sublabel: 'Compiling replay data for 3D playback',
      showProgress: false,
    },
    success: {
      icon: CheckCircle,
      iconClass: 'text-green-400',
      bgClass: 'from-green-500/20 to-emerald-500/20',
      borderClass: 'border-green-500/30',
      label: 'Upload complete!',
      sublabel: 'Redirecting to viewer...',
      showProgress: false,
    },
    error: {
      icon: XCircle,
      iconClass: 'text-red-400',
      bgClass: 'from-red-500/20 to-orange-500/20',
      borderClass: 'border-red-500/30',
      label: 'Upload failed',
      sublabel: error || 'An error occurred during upload',
      showProgress: false,
    },
  };

  const config = statusConfig[status as keyof typeof statusConfig];
  if (!config) return null;

  const Icon = config.icon;

  return (
    <div className={cn(
      'relative overflow-hidden rounded-xl border',
      config.borderClass
    )}>
      {/* Background gradient */}
      <div className={cn(
        'absolute inset-0 bg-gradient-to-r opacity-50',
        config.bgClass
      )} />

      <div className="relative p-5">
        <div className="flex items-center gap-4">
          {/* Icon with glow */}
          <div className="relative">
            <div className={cn(
              'absolute inset-0 rounded-xl blur-lg opacity-50',
              status === 'uploading' && 'bg-violet-500',
              status === 'processing' && 'bg-blue-500',
              status === 'success' && 'bg-green-500',
              status === 'error' && 'bg-red-500'
            )} />
            <div className={cn(
              'relative w-12 h-12 rounded-xl flex items-center justify-center',
              status === 'uploading' && 'bg-violet-500/20',
              status === 'processing' && 'bg-blue-500/20',
              status === 'success' && 'bg-green-500/20',
              status === 'error' && 'bg-red-500/20'
            )}>
              <Icon className={cn('w-6 h-6', config.iconClass)} />
            </div>
          </div>

          <div className="flex-1">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-white">{config.label}</span>
              {status === 'success' && (
                <Sparkles className="w-4 h-4 text-yellow-400" />
              )}
            </div>
            <p className="text-sm text-gray-400">{config.sublabel}</p>
          </div>

          {config.showProgress && (
            <span className="text-lg font-bold text-violet-400">{progress}%</span>
          )}
        </div>

        {/* Progress bar */}
        {config.showProgress && (
          <div className="mt-4">
            <div className="h-2 bg-gray-800 rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-violet-500 to-blue-500 transition-all duration-300 rounded-full"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}

        {/* Processing animation dots */}
        {status === 'processing' && (
          <div className="mt-4 flex items-center justify-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-blue-400 animate-bounce" style={{ animationDelay: '0ms' }} />
            <div className="w-2 h-2 rounded-full bg-blue-400 animate-bounce" style={{ animationDelay: '150ms' }} />
            <div className="w-2 h-2 rounded-full bg-blue-400 animate-bounce" style={{ animationDelay: '300ms' }} />
          </div>
        )}
      </div>
    </div>
  );
}
