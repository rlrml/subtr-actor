import { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import {
  ArrowLeft,
  Calendar,
  MessageSquare,
  Pencil,
  Trash2,
  ChevronDown,
} from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { feedbackApi } from '@/api/feedback.api';
import { CategoryBadge } from '@/components/feedback/CategoryBadge';
import { StatusBadge } from '@/components/feedback/StatusBadge';
import { UpvoteButton } from '@/components/feedback/UpvoteButton';
import { SafeMarkdown } from '@/components/feedback/SafeMarkdown';
import { CommentThread } from '@/components/feedback/CommentThread';
import { CommentForm } from '@/components/feedback/CommentForm';
import type { FeedbackPost as FeedbackPostType, FeedbackComment, FeedbackStatus } from '@/api/feedback.api';

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function UserAvatar({ avatarUrl, username, size = 'md' }: { avatarUrl: string | null; username: string; size?: 'sm' | 'md' }) {
  const sizeClass = size === 'sm' ? 'w-6 h-6' : 'w-8 h-8';
  const textSize = size === 'sm' ? 'text-[10px]' : 'text-sm';

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

export default function FeedbackPost() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { user } = useAuth();

  // Data state
  const [post, setPost] = useState<FeedbackPostType | null>(null);
  const [comments, setComments] = useState<FeedbackComment[]>([]);
  const [userVoted, setUserVoted] = useState(false);
  const [statuses, setStatuses] = useState<FeedbackStatus[]>([]);

  // UI state
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showStatusMenu, setShowStatusMenu] = useState(false);
  const [isUpdatingStatus, setIsUpdatingStatus] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const isAuthor = user?.id === post?.author.id;
  const isAdmin = user?.isAdmin ?? false;

  // Load post data
  const loadPost = useCallback(async () => {
    if (!id) return;

    try {
      const response = await feedbackApi.getPost(id);
      setPost(response.post);
      setComments(response.comments);
      setUserVoted(response.userVoted);
    } catch (err) {
      console.error('Failed to load post:', err);
      setError('Post not found or has been deleted.');
    } finally {
      setIsLoading(false);
    }
  }, [id]);

  useEffect(() => {
    loadPost();
  }, [loadPost]);

  // Load statuses for admin
  useEffect(() => {
    if (!isAdmin) return;
    const loadStatuses = async () => {
      try {
        const response = await feedbackApi.getStatuses();
        setStatuses(response.statuses);
      } catch (err) {
        console.error('Failed to load statuses:', err);
      }
    };
    loadStatuses();
  }, [isAdmin]);

  // Handle status change (admin only)
  const handleStatusChange = async (statusId: string) => {
    if (!post || !isAdmin) return;
    setIsUpdatingStatus(true);
    try {
      const response = await feedbackApi.updatePostStatus(post.id, statusId);
      setPost(response.post);
      setShowStatusMenu(false);
    } catch (err) {
      console.error('Failed to update status:', err);
    } finally {
      setIsUpdatingStatus(false);
    }
  };

  // Handle delete post
  const handleDelete = async () => {
    if (!post || !window.confirm('Are you sure you want to delete this post? This action cannot be undone.')) {
      return;
    }
    setIsDeleting(true);
    try {
      await feedbackApi.deletePost(post.id);
      navigate('/feedback');
    } catch (err) {
      console.error('Failed to delete post:', err);
      alert('Failed to delete post. Please try again.');
    } finally {
      setIsDeleting(false);
    }
  };

  // Handle new comment
  const handleAddComment = async (content: string) => {
    if (!post) return;
    const response = await feedbackApi.createComment(post.id, content);
    setComments((prev) => [...prev, response.comment]);
    // Update comment count
    setPost((prev) => prev ? { ...prev, commentCount: prev.commentCount + 1 } : null);
  };

  // Handle edit comment
  const handleEditComment = async (commentId: string, newContent: string) => {
    if (!post) return;
    const response = await feedbackApi.updateComment(post.id, commentId, newContent);
    setComments((prev) =>
      prev.map((c) => (c.id === commentId ? { ...c, ...response.comment, author: c.author, isEdited: true } : c))
    );
  };

  // Handle delete comment
  const handleDeleteComment = async (commentId: string) => {
    if (!post) return;
    await feedbackApi.deleteComment(post.id, commentId);
    setComments((prev) =>
      prev.map((c) => (c.id === commentId ? { ...c, isDeleted: true } : c))
    );
    // Update comment count
    setPost((prev) => prev ? { ...prev, commentCount: prev.commentCount - 1 } : null);
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-violet-500" />
      </div>
    );
  }

  // Error state
  if (error || !post) {
    return (
      <div className="max-w-5xl mx-auto">
        <Link
          to="/feedback"
          className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to Feedback Hub
        </Link>
        <div className="text-center py-12">
          <h2 className="text-xl font-bold text-white mb-2">Post Not Found</h2>
          <p className="text-gray-400">{error || 'This post may have been deleted.'}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-5xl mx-auto">
      {/* Back link */}
      <Link
        to="/feedback"
        className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
      >
        <ArrowLeft className="w-4 h-4" />
        Back to Feedback Hub
      </Link>

      {/* Main Post Card */}
      <div className="rounded-xl border border-gray-800 bg-gradient-to-br from-gray-900 to-gray-950 overflow-hidden">
        <div className="p-6">
          <div className="flex gap-4">
            {/* Upvote */}
            <div className="shrink-0">
              <UpvoteButton
                postId={post.id}
                initialCount={post.upvoteCount}
                initialVoted={userVoted}
                size="lg"
              />
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0 space-y-4">
              {/* Header with badges */}
              <div className="flex items-start justify-between gap-4">
                <div className="flex flex-wrap items-center gap-2">
                  <CategoryBadge
                    name={post.category.name}
                    color={post.category.color}
                  />

                  {/* Status with admin dropdown */}
                  {isAdmin ? (
                    <div className="relative">
                      <button
                        onClick={() => setShowStatusMenu(!showStatusMenu)}
                        disabled={isUpdatingStatus}
                        className="flex items-center gap-1"
                      >
                        <StatusBadge name={post.status.name} color={post.status.color} />
                        <ChevronDown className="w-3 h-3 text-gray-400" />
                      </button>

                      {showStatusMenu && (
                        <>
                          <div
                            className="fixed inset-0 z-10"
                            onClick={() => setShowStatusMenu(false)}
                          />
                          <div className="absolute left-0 top-full mt-1 w-40 py-1 rounded-lg bg-gray-800 border border-gray-700 shadow-lg z-20">
                            {statuses.map((status) => (
                              <button
                                key={status.id}
                                onClick={() => handleStatusChange(status.id)}
                                className={`
                                  flex items-center gap-2 w-full px-3 py-2 text-sm text-left transition-colors
                                  ${status.id === post.status.id
                                    ? 'bg-violet-500/20 text-violet-300'
                                    : 'text-gray-300 hover:bg-gray-700'
                                  }
                                `}
                              >
                                <span
                                  className="w-2 h-2 rounded-full"
                                  style={{ backgroundColor: status.color }}
                                />
                                {status.name}
                              </button>
                            ))}
                          </div>
                        </>
                      )}
                    </div>
                  ) : (
                    <StatusBadge name={post.status.name} color={post.status.color} />
                  )}
                </div>

                {/* Actions */}
                {(isAuthor || isAdmin) && (
                  <div className="flex items-center gap-2">
                    {isAuthor && (
                      <Link
                        to={`/feedback/${post.id}/edit`}
                        className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                        title="Edit post"
                      >
                        <Pencil className="w-4 h-4" />
                      </Link>
                    )}
                    <button
                      onClick={handleDelete}
                      disabled={isDeleting}
                      className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors disabled:opacity-50"
                      title="Delete post"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                )}
              </div>

              {/* Title */}
              <h1 className="text-2xl font-bold text-white">
                {post.title}
              </h1>

              {/* Author and date */}
              <div className="flex flex-wrap items-center gap-3 text-sm text-gray-400">
                <div className="flex items-center gap-2">
                  <UserAvatar avatarUrl={post.author.avatarUrl} username={post.author.username} size="sm" />
                  <span>
                    <span className={`font-medium ${post.author.isAdmin ? 'text-violet-400' : 'text-gray-200'}`}>
                      {post.author.username}
                    </span>
                    {post.author.isAdmin && (
                      <span className="ml-1 px-1.5 py-0.5 text-xs rounded bg-violet-500/20 text-violet-300 font-medium">
                        Dev
                      </span>
                    )}
                  </span>
                </div>
                <span className="flex items-center gap-1">
                  <Calendar className="w-4 h-4" />
                  {formatDate(post.createdAt)}
                </span>
                {post.isEdited && (
                  <span className="text-gray-500">(edited)</span>
                )}
              </div>

              {/* Content */}
              <div className="pt-4 border-t border-gray-800">
                <SafeMarkdown content={post.content} />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Comments Section */}
      <div className="mt-8">
        <h2 className="flex items-center gap-2 text-lg font-bold text-white mb-6">
          <MessageSquare className="w-5 h-5" />
          Comments ({post.commentCount})
        </h2>

        {/* Comments List */}
        <CommentThread
          comments={comments}
          currentUserId={user?.id}
          isAdmin={isAdmin}
          onEditComment={handleEditComment}
          onDeleteComment={handleDeleteComment}
        />

        {/* Comment Form at the end */}
        <div className="mt-6">
          <CommentForm onSubmit={handleAddComment} />
        </div>
      </div>
    </div>
  );
}
