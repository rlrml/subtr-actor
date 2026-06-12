import { FeedbackCard } from './FeedbackCard';
import type { FeedbackPost } from '@/api/feedback.api';

interface FeedbackListProps {
  posts: FeedbackPost[];
  votedPosts?: Record<string, boolean>;
  onVote?: (postId: string) => Promise<void>;
  isLoading?: boolean;
}

function FeedbackCardSkeleton() {
  return (
    <div className="rounded-xl overflow-hidden bg-gradient-to-br from-gray-900 to-gray-950 border border-gray-800 animate-pulse">
      <div className="p-5">
        <div className="flex gap-4">
          {/* Vote skeleton */}
          <div className="flex flex-col items-center shrink-0">
            <div className="w-10 h-10 rounded-lg bg-gray-800" />
            <div className="w-6 h-5 mt-2 rounded bg-gray-800" />
          </div>

          {/* Content skeleton */}
          <div className="flex-1 space-y-3">
            <div className="flex items-center gap-2">
              <div className="w-16 h-5 rounded-full bg-gray-800" />
              <div className="w-20 h-5 rounded-full bg-gray-800" />
            </div>
            <div className="w-3/4 h-6 rounded bg-gray-800" />
            <div className="space-y-2">
              <div className="w-full h-4 rounded bg-gray-800" />
              <div className="w-2/3 h-4 rounded bg-gray-800" />
            </div>
            <div className="flex items-center justify-between pt-2">
              <div className="w-24 h-4 rounded bg-gray-800" />
              <div className="w-12 h-4 rounded bg-gray-800" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function FeedbackList({ posts, votedPosts = {}, onVote, isLoading }: FeedbackListProps) {
  if (isLoading) {
    return (
      <div className="space-y-4">
        {[...Array(5)].map((_, i) => (
          <FeedbackCardSkeleton key={i} />
        ))}
      </div>
    );
  }

  if (posts.length === 0) {
    return (
      <div className="text-center py-16">
        <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-800/50 flex items-center justify-center">
          <svg className="w-8 h-8 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
          </svg>
        </div>
        <h3 className="text-lg font-medium text-gray-300 mb-2">No posts found</h3>
        <p className="text-gray-500 text-sm">
          Be the first to share your feedback!
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {posts.map((post) => (
        <FeedbackCard
          key={post.id}
          post={post}
          userVoted={votedPosts[post.id] ?? false}
          onVote={onVote}
        />
      ))}
    </div>
  );
}
