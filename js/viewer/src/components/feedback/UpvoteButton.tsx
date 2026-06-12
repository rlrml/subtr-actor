import { useState, useRef } from 'react';
import { ChevronUp } from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { feedbackApi } from '@/api/feedback.api';

interface UpvoteButtonProps {
  postId: string;
  initialCount: number;
  initialVoted?: boolean;
  onVoteChange?: (voted: boolean, newCount: number) => void;
  size?: 'sm' | 'md' | 'lg';
  orientation?: 'vertical' | 'horizontal';
}

export function UpvoteButton({
  postId,
  initialCount,
  initialVoted = false,
  onVoteChange,
  size = 'md',
  orientation = 'vertical',
}: UpvoteButtonProps) {
  const { isAuthenticated } = useAuth();
  const [voted, setVoted] = useState(initialVoted);
  const [count, setCount] = useState(initialCount);
  const [isLoading, setIsLoading] = useState(false);

  // Use ref for synchronous lock to prevent race conditions
  const isProcessingRef = useRef(false);

  const handleClick = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();

    if (!isAuthenticated) {
      window.location.href = `/login?redirect=${window.location.pathname}`;
      return;
    }

    // Synchronous check to prevent multiple clicks
    if (isProcessingRef.current) return;
    isProcessingRef.current = true;
    setIsLoading(true);

    // Optimistic update
    const wasVoted = voted;
    const oldCount = count;
    const newVoted = !wasVoted;
    const newCount = oldCount + (newVoted ? 1 : -1);

    setVoted(newVoted);
    setCount(newCount);

    try {
      const response = await feedbackApi.toggleVote(postId);
      setVoted(response.voted);
      setCount(response.upvoteCount);
      onVoteChange?.(response.voted, response.upvoteCount);
    } catch (err) {
      // Rollback on error
      console.error('Failed to toggle vote:', err);
      setVoted(wasVoted);
      setCount(oldCount);
    } finally {
      isProcessingRef.current = false;
      setIsLoading(false);
    }
  };

  // Size variants
  const sizeClasses = {
    sm: {
      button: 'p-1.5',
      icon: 'w-4 h-4',
      count: 'text-sm',
      gap: orientation === 'vertical' ? 'gap-0.5' : 'gap-1',
    },
    md: {
      button: 'p-2',
      icon: 'w-5 h-5',
      count: 'text-lg',
      gap: orientation === 'vertical' ? 'gap-1' : 'gap-2',
    },
    lg: {
      button: 'p-3',
      icon: 'w-6 h-6',
      count: 'text-2xl',
      gap: orientation === 'vertical' ? 'gap-1' : 'gap-2',
    },
  };

  const classes = sizeClasses[size];

  return (
    <div
      className={`flex items-center ${classes.gap} ${orientation === 'vertical' ? 'flex-col' : 'flex-row'}`}
    >
      <button
        onClick={handleClick}
        disabled={isLoading}
        className={`
          ${classes.button} rounded-lg transition-all duration-200
          ${voted
            ? 'bg-violet-600/30 text-violet-300 border border-violet-500/50'
            : 'bg-gray-800/50 text-gray-400 border border-gray-700/50 hover:bg-violet-600/20 hover:text-violet-300 hover:border-violet-500/30'
          }
          ${isLoading ? 'opacity-50 cursor-not-allowed' : ''}
        `}
      >
        <ChevronUp className={`${classes.icon} ${voted ? 'stroke-[3]' : ''}`} />
      </button>
      <span
        className={`${classes.count} font-bold ${voted ? 'text-violet-300' : 'text-gray-300'}`}
      >
        {count}
      </span>
    </div>
  );
}
