import { useState, useRef } from 'react';
import { Link } from 'react-router-dom';
import { ChevronUp, MessageSquare, Clock } from 'lucide-react';
import { CategoryBadge } from './CategoryBadge';
import { StatusBadge } from './StatusBadge';
import type { FeedbackPost } from '@/api/feedback.api';

interface FeedbackCardProps {
  post: FeedbackPost;
  userVoted?: boolean;
  onVote?: (postId: string) => Promise<void>;
  isVoting?: boolean;
}

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  return date.toLocaleDateString('en-US', {
    day: 'numeric',
    month: 'short',
  });
}

function UserAvatar({ avatarUrl, username, size = 'sm' }: { avatarUrl: string | null; username: string; size?: 'sm' | 'md' }) {
  const sizeClass = size === 'sm' ? 'w-5 h-5' : 'w-8 h-8';
  const textSize = size === 'sm' ? 'text-[10px]' : 'text-xs';

  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={username}
        className={`${sizeClass} rounded-full object-cover`}
      />
    );
  }

  return (
    <div className={`${sizeClass} rounded-full bg-gray-700 flex items-center justify-center`}>
      <span className={`${textSize} font-medium text-gray-400`}>
        {username.charAt(0).toUpperCase()}
      </span>
    </div>
  );
}

export function FeedbackCard({ post, userVoted = false, onVote, isVoting = false }: FeedbackCardProps) {
  const [localVoting, setLocalVoting] = useState(false);
  const isProcessingRef = useRef(false);

  const handleVoteClick = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();

    // Synchronous check to prevent race conditions
    if (isProcessingRef.current || isVoting) return;
    isProcessingRef.current = true;
    setLocalVoting(true);

    try {
      await onVote?.(post.id);
    } finally {
      isProcessingRef.current = false;
      setLocalVoting(false);
    }
  };

  const voting = localVoting || isVoting;

  return (
    <Link to={`/feedback/${post.id}`} className="block group">
      <div className="relative rounded-xl overflow-hidden bg-gradient-to-br from-gray-900 to-gray-950 border border-gray-800 hover:border-violet-500/50 transition-all duration-300 hover:shadow-lg hover:shadow-violet-500/10">
        {/* Subtle gradient overlay */}
        <div className="absolute inset-0 bg-gradient-to-br from-violet-600/5 via-transparent to-blue-600/5 pointer-events-none" />

        <div className="relative p-5">
          <div className="flex gap-4">
            {/* Vote Section */}
            <div className="flex flex-col items-center shrink-0">
              <button
                onClick={handleVoteClick}
                disabled={voting}
                className={`
                  p-2 rounded-lg transition-all duration-200
                  ${userVoted
                    ? 'bg-violet-600/30 text-violet-300 border border-violet-500/50'
                    : 'bg-gray-800/50 text-gray-400 border border-gray-700/50 hover:bg-violet-600/20 hover:text-violet-300 hover:border-violet-500/30'
                  }
                  ${voting ? 'opacity-50 cursor-not-allowed' : ''}
                `}
              >
                <ChevronUp className={`w-5 h-5 ${userVoted ? 'stroke-[3]' : ''} ${voting ? 'animate-pulse' : ''}`} />
              </button>
              <span className={`text-lg font-bold mt-1 ${userVoted ? 'text-violet-300' : 'text-gray-300'}`}>
                {post.upvoteCount}
              </span>
            </div>

            {/* Content Section */}
            <div className="flex-1 min-w-0 space-y-3">
              {/* Header with badges */}
              <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-2 flex-wrap">
                  <CategoryBadge
                    name={post.category.name}
                    color={post.category.color}
                    size="sm"
                  />
                  <StatusBadge
                    name={post.status.name}
                    color={post.status.color}
                    size="sm"
                  />
                </div>
                <div className="flex items-center gap-1.5 text-xs text-gray-500 shrink-0">
                  <Clock className="w-3.5 h-3.5" />
                  {formatRelativeTime(post.createdAt)}
                </div>
              </div>

              {/* Title */}
              <h3 className="font-bold text-white text-lg leading-tight group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:from-violet-400 group-hover:to-blue-400 group-hover:bg-clip-text transition-all duration-300">
                {post.title}
              </h3>

              {/* Summary (AI-generated only, no raw markdown) */}
              {post.summary && (
                <p className="text-sm text-gray-400 line-clamp-2">
                  {post.summary}
                </p>
              )}

              {/* Footer */}
              <div className="flex items-center justify-between pt-2">
                <div className="flex items-center gap-2 text-sm text-gray-500">
                  <UserAvatar
                    avatarUrl={post.author.avatarUrl}
                    username={post.author.username}
                    size="sm"
                  />
                  <span className={`font-medium ${post.author.isAdmin ? 'text-violet-400' : 'text-gray-300'}`}>
                    {post.author.username}
                  </span>
                  {post.author.isAdmin && (
                    <span className="px-1.5 py-0.5 text-xs rounded bg-violet-500/20 text-violet-300 font-medium">
                      Dev
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-1.5 text-gray-500">
                  <MessageSquare className="w-4 h-4" />
                  <span className="text-sm">{post.commentCount}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Link>
  );
}
