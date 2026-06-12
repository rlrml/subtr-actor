import { Globe, EyeOff } from 'lucide-react';
import { cn } from '@/lib/utils';

interface VisibilityBadgeProps {
  visibility: 'public' | 'unlisted';
  size?: 'sm' | 'md';
  className?: string;
}

export function VisibilityBadge({ visibility, size = 'sm', className }: VisibilityBadgeProps) {
  const isPublic = visibility === 'public';

  const sizeClasses = {
    sm: 'px-2 py-0.5 text-xs gap-1',
    md: 'px-2.5 py-1 text-sm gap-1.5',
  };

  const iconSize = size === 'sm' ? 'w-3 h-3' : 'w-4 h-4';

  return (
    <span
      className={cn(
        'inline-flex items-center rounded-full font-medium',
        sizeClasses[size],
        isPublic
          ? 'bg-green-500/10 text-green-400 border border-green-500/20'
          : 'bg-gray-500/10 text-gray-400 border border-gray-500/20',
        className
      )}
    >
      {isPublic ? (
        <>
          <Globe className={iconSize} />
          Public
        </>
      ) : (
        <>
          <EyeOff className={iconSize} />
          Unlisted
        </>
      )}
    </span>
  );
}
