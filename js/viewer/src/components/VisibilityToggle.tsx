import { useState } from 'react';
import { Globe, EyeOff, Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';

interface VisibilityToggleProps {
  visibility: 'public' | 'unlisted';
  onToggle: (newVisibility: 'public' | 'unlisted') => Promise<void>;
  disabled?: boolean;
  className?: string;
}

export function VisibilityToggle({ visibility, onToggle, disabled, className }: VisibilityToggleProps) {
  const [loading, setLoading] = useState(false);
  const isPublic = visibility === 'public';

  const handleClick = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();

    if (disabled || loading) return;

    setLoading(true);
    try {
      const newVisibility = isPublic ? 'unlisted' : 'public';
      await onToggle(newVisibility);
    } finally {
      setLoading(false);
    }
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      disabled={disabled || loading}
      className={cn(
        'inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium transition-all',
        isPublic
          ? 'bg-green-500/10 text-green-400 border border-green-500/20 hover:bg-green-500/20'
          : 'bg-gray-500/10 text-gray-400 border border-gray-500/20 hover:bg-gray-500/20',
        disabled && 'opacity-50 cursor-not-allowed',
        className
      )}
      title={isPublic ? 'Click to make unlisted' : 'Click to make public'}
    >
      {loading ? (
        <Loader2 className="w-3 h-3 animate-spin" />
      ) : isPublic ? (
        <Globe className="w-3 h-3" />
      ) : (
        <EyeOff className="w-3 h-3" />
      )}
      {isPublic ? 'Public' : 'Unlisted'}
    </button>
  );
}
