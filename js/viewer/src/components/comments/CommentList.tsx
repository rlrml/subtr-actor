import { useState, useEffect, useCallback, useContext } from 'react';
import { MessageCircle, Loader2, LogIn, ChevronDown } from 'lucide-react';
import { Link } from 'react-router-dom';
import { commentApi, type Comment, type CommentWithAuthor } from '@/api/comment.api';
import { CommentItem } from './CommentItem';
import { CommentForm } from './CommentForm';
import { AuthContext } from '@/contexts/AuthContext';

const COMMENTS_PER_PAGE = 10;

interface CommentListProps {
  entityType: 'replay' | 'announcement';
  entityId: string;
  onCommentCountChange?: (count: number) => void;
}

export function CommentList({
  entityType,
  entityId,
  onCommentCountChange,
}: CommentListProps) {
  const authContext = useContext(AuthContext);
  const user = authContext?.user;
  const isAuthenticated = authContext?.isAuthenticated ?? false;

  const [comments, setComments] = useState<(Comment & { isEdited?: boolean })[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [total, setTotal] = useState(0);
  const [hasMore, setHasMore] = useState(false);

  const fetchComments = useCallback(async (offset = 0, append = false) => {
    try {
      if (append) {
        setLoadingMore(true);
      } else {
        setLoading(true);
      }
      setError(null);

      const response = await commentApi.getComments(entityType, entityId, COMMENTS_PER_PAGE, offset);

      if (append) {
        setComments((prev) => [...prev, ...response.comments]);
      } else {
        setComments(response.comments);
      }

      setTotal(response.total);
      setHasMore(response.hasMore);
      onCommentCountChange?.(response.total);
    } catch (err) {
      setError('Failed to load comments');
      console.error('Error fetching comments:', err);
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  }, [entityType, entityId, onCommentCountChange]);

  useEffect(() => {
    fetchComments(0, false);
  }, [fetchComments]);

  const handleLoadMore = useCallback(() => {
    if (!loadingMore && hasMore) {
      fetchComments(comments.length, true);
    }
  }, [loadingMore, hasMore, comments.length, fetchComments]);

  // Handle comment updated
  const handleCommentUpdated = useCallback((updatedComment: Comment) => {
    setComments((prev) =>
      prev.map((c) => (c.id === updatedComment.id ? { ...updatedComment, isEdited: true } : c))
    );
  }, []);

  // Handle comment deleted
  const handleCommentDeleted = useCallback((commentId: string) => {
    setComments((prev) => prev.filter((c) => c.id !== commentId));
    setTotal((prev) => {
      const newTotal = Math.max(0, prev - 1);
      onCommentCountChange?.(newTotal);
      return newTotal;
    });
  }, [onCommentCountChange]);

  // Add new comment to list (called by CommentForm) - add at top since list is sorted by newest first
  const handleCommentCreated = useCallback((newComment: CommentWithAuthor) => {
    setComments((prev) => [newComment, ...prev]);
    setTotal((prev) => {
      const newTotal = prev + 1;
      onCommentCountChange?.(newTotal);
      return newTotal;
    });
  }, [onCommentCountChange]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-8">
        <p className="text-red-400">{error}</p>
        <button
          onClick={() => fetchComments()}
          className="mt-2 text-sm text-violet-400 hover:text-violet-300"
        >
          Try again
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-2 text-gray-400">
        <MessageCircle className="w-5 h-5" />
        <span className="font-medium">
          {total} {total === 1 ? 'Comment' : 'Comments'}
        </span>
      </div>

      {/* Comment Form or Login Prompt */}
      {isAuthenticated ? (
        <CommentForm
          entityType={entityType}
          entityId={entityId}
          onCommentCreated={handleCommentCreated}
        />
      ) : (
        <div className="flex items-center gap-3 p-4 bg-gray-800/50 border border-gray-700 rounded-lg">
          <LogIn className="w-5 h-5 text-gray-400" />
          <span className="text-gray-400">
            <Link to="/login" className="text-violet-400 hover:text-violet-300 font-medium">
              Log in
            </Link>
            {' '}to leave a comment
          </span>
        </div>
      )}

      {/* Comments List */}
      {comments.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          No comments yet. Be the first to comment!
        </div>
      ) : (
        <div className="space-y-3">
          {comments.map((comment) => (
            <CommentItem
              key={comment.id}
              comment={comment}
              currentUserId={user?.id}
              isAdmin={user?.isAdmin}
              onCommentUpdated={handleCommentUpdated}
              onCommentDeleted={handleCommentDeleted}
            />
          ))}

          {/* Load More Button */}
          {hasMore && (
            <div className="flex justify-center pt-4">
              <button
                onClick={handleLoadMore}
                disabled={loadingMore}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-400
                           bg-gray-800/50 hover:bg-gray-800 border border-gray-700 rounded-lg
                           transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loadingMore ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin" />
                    Loading...
                  </>
                ) : (
                  <>
                    <ChevronDown className="w-4 h-4" />
                    Load older comments ({total - comments.length} remaining)
                  </>
                )}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
