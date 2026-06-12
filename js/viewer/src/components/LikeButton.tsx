import { useState, useEffect, useContext, useRef, useCallback } from 'react';
import { Heart } from 'lucide-react';
import { likeApi } from '@/api/like.api';
import { AuthContext } from '@/contexts/AuthContext';
import { Link } from 'react-router-dom';

// Debounce delay in milliseconds
const DEBOUNCE_DELAY = 300;

interface LikeButtonProps {
  replayId: string;
  initialLiked?: boolean;
  initialCount?: number;
  size?: 'sm' | 'md' | 'lg';
  showCount?: boolean;
  onLikeChange?: (liked: boolean, count: number) => void;
}

export function LikeButton({
  replayId,
  initialLiked = false,
  initialCount = 0,
  size = 'md',
  showCount = true,
  onLikeChange,
}: LikeButtonProps) {
  const authContext = useContext(AuthContext);
  const isAuthenticated = authContext?.isAuthenticated ?? false;

  const [liked, setLiked] = useState(initialLiked);
  const [count, setCount] = useState(initialCount);
  const [isLoading, setIsLoading] = useState(false);
  const [showLoginPrompt, setShowLoginPrompt] = useState(false);

  // Debounce ref to track pending API call
  const debounceRef = useRef<NodeJS.Timeout | null>(null);
  const pendingStateRef = useRef<{ liked: boolean; count: number } | null>(null);

  // Fetch like status on mount
  useEffect(() => {
    const fetchLikeStatus = async () => {
      try {
        const status = await likeApi.getLikeStatus(replayId);
        setLiked(status.liked);
        setCount(status.likeCount);
      } catch (err) {
        console.error('Failed to fetch like status:', err);
      }
    };

    fetchLikeStatus();
  }, [replayId]);

  // Cleanup debounce on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  const executeToggle = useCallback(async (originalLiked: boolean, originalCount: number) => {
    setIsLoading(true);
    try {
      const result = await likeApi.toggleLike(replayId);
      setLiked(result.liked);
      setCount(result.likeCount);
      onLikeChange?.(result.liked, result.likeCount);
    } catch (err) {
      // Revert on error
      setLiked(originalLiked);
      setCount(originalCount);
      console.error('Failed to toggle like:', err);
    } finally {
      setIsLoading(false);
      pendingStateRef.current = null;
    }
  }, [replayId, onLikeChange]);

  const handleClick = () => {
    if (!isAuthenticated) {
      setShowLoginPrompt(true);
      setTimeout(() => setShowLoginPrompt(false), 3000);
      return;
    }

    // Cancel any pending debounce
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    // Store original state for potential revert (only on first click of sequence)
    if (!pendingStateRef.current) {
      pendingStateRef.current = { liked, count };
    }

    // Optimistic update
    const newLiked = !liked;
    const newCount = newLiked ? count + 1 : count - 1;
    setLiked(newLiked);
    setCount(newCount);

    // Debounce the API call
    debounceRef.current = setTimeout(() => {
      const originalState = pendingStateRef.current!;
      executeToggle(originalState.liked, originalState.count);
    }, DEBOUNCE_DELAY);
  };

  const sizeConfig = {
    sm: {
      container: 'gap-1.5',
      button: 'px-2.5 py-1.5',
      icon: 'w-4 h-4',
      text: 'text-xs',
    },
    md: {
      container: 'gap-2',
      button: 'px-3 py-2',
      icon: 'w-5 h-5',
      text: 'text-sm',
    },
    lg: {
      container: 'gap-2.5',
      button: 'px-4 py-2.5',
      icon: 'w-6 h-6',
      text: 'text-base',
    },
  };

  const config = sizeConfig[size];

  return (
    <div className={`relative inline-flex items-center ${config.container}`}>
      <button
        onClick={handleClick}
        disabled={isLoading}
        className={`
          group relative flex items-center gap-2 ${config.button}
          rounded-xl font-medium transition-all duration-300
          ${liked
            ? 'bg-gradient-to-r from-rose-500/20 to-pink-500/20 text-rose-400 border border-rose-500/30 shadow-[0_0_20px_rgba(244,63,94,0.2)] hover:shadow-[0_0_30px_rgba(244,63,94,0.3)] hover:border-rose-500/50'
            : 'bg-gray-800/60 text-gray-400 border border-gray-700/50 hover:bg-gray-800 hover:text-gray-200 hover:border-gray-600'
          }
          disabled:opacity-50 disabled:cursor-not-allowed
          focus:outline-none focus:ring-2 focus:ring-rose-500/40 focus:ring-offset-2 focus:ring-offset-gray-900
          active:scale-95
        `}
        title={liked ? 'Unlike' : 'Like'}
      >
        <Heart
          className={`
            ${config.icon} transition-all duration-300
            ${liked
              ? 'fill-rose-400 text-rose-400 scale-110 drop-shadow-[0_0_8px_rgba(244,63,94,0.5)]'
              : 'group-hover:scale-110'
            }
          `}
        />
        {showCount && (
          <span className={`${config.text} font-semibold tabular-nums`}>
            {count}
          </span>
        )}
      </button>

      {/* Login prompt popup */}
      {showLoginPrompt && (
        <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-3 px-4 py-2.5 bg-gray-900/95 backdrop-blur-sm border border-gray-700 rounded-xl shadow-2xl whitespace-nowrap z-50 animate-in fade-in slide-in-from-bottom-2 duration-200">
          <Link to="/login" className="text-violet-400 hover:text-violet-300 font-medium">
            Log in
          </Link>
          <span className="text-gray-400"> to like replays</span>
          <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-[1px] border-8 border-transparent border-t-gray-700" />
        </div>
      )}
    </div>
  );
}
