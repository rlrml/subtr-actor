import { useState, useRef, useEffect } from 'react';
import { formatDistanceToNow } from 'date-fns';
import { X, Check, Loader2 } from 'lucide-react';
import { Comment, commentApi } from '@/api/comment.api';

interface CommentItemProps {
  comment: Comment & { isEdited?: boolean };
  currentUserId?: string;
  isAdmin?: boolean;
  onEdit?: (comment: Comment) => void;
  onDelete?: (commentId: string) => void;
  onCommentUpdated?: (comment: Comment) => void;
  onCommentDeleted?: (commentId: string) => void;
}

export function CommentItem({
  comment,
  currentUserId,
  isAdmin,
  onCommentUpdated,
  onCommentDeleted,
}: CommentItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(comment.content);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const isAuthor = currentUserId && comment.author?.id === currentUserId;
  const canEdit = isAuthor;
  const canDelete = isAuthor || isAdmin;
  const author = comment.author;

  // Focus textarea when entering edit mode
  useEffect(() => {
    if (isEditing && textareaRef.current) {
      textareaRef.current.focus();
      textareaRef.current.selectionStart = textareaRef.current.value.length;
    }
  }, [isEditing]);

  const handleStartEdit = () => {
    setEditContent(comment.content);
    setIsEditing(true);
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setEditContent(comment.content);
  };

  const handleSaveEdit = async () => {
    const trimmedContent = editContent.trim();
    if (!trimmedContent || trimmedContent === comment.content) {
      handleCancelEdit();
      return;
    }

    setIsSubmitting(true);
    setError(null);
    try {
      const response = await commentApi.updateComment(comment.id, trimmedContent);
      onCommentUpdated?.(response.comment);
      setIsEditing(false);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to update comment';
      setError(message);
      console.error('Failed to update comment:', err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async () => {
    setIsSubmitting(true);
    setError(null);
    try {
      await commentApi.deleteComment(comment.id);
      onCommentDeleted?.(comment.id);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to delete comment';
      setError(message);
      console.error('Failed to delete comment:', err);
      setShowDeleteConfirm(false);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Escape') {
      handleCancelEdit();
    } else if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      handleSaveEdit();
    }
  };

  // Format content with line breaks
  const formattedContent = comment.content.split('\n').map((line, index, arr) => (
    <span key={index}>
      {line}
      {index < arr.length - 1 && <br />}
    </span>
  ));

  return (
    <div className="flex gap-3 p-4 bg-gray-800/50 rounded-lg border border-gray-700/50">
      {/* Avatar */}
      <div className="flex-shrink-0">
        {author?.avatarUrl ? (
          <img
            src={author.avatarUrl}
            alt={author.username ?? 'User'}
            className="w-10 h-10 rounded-full object-cover"
          />
        ) : (
          <div className="w-10 h-10 rounded-full bg-gradient-to-br from-violet-500 to-blue-500 flex items-center justify-center text-white font-medium">
            {(author?.username ?? '?').charAt(0).toUpperCase()}
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        {/* Header */}
        <div className="flex items-center gap-2 flex-wrap">
          <span className="font-medium text-white">
            {author?.username ?? 'Unknown'}
          </span>

          {/* Dev badge for admins */}
          {author?.isAdmin && (
            <span className="px-1.5 py-0.5 text-xs font-medium bg-violet-500/20 text-violet-400 rounded border border-violet-500/30">
              Dev
            </span>
          )}

          {/* Timestamp */}
          <span className="text-sm text-gray-500">
            {formatDistanceToNow(new Date(comment.createdAt), { addSuffix: true })}
          </span>

          {/* Edited indicator */}
          {comment.isEdited && (
            <span className="text-xs text-gray-500 italic">(edited)</span>
          )}
        </div>

        {/* Comment content or edit form */}
        {isEditing ? (
          <div className="mt-3 space-y-3">
            <textarea
              ref={textareaRef}
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={isSubmitting}
              rows={3}
              maxLength={2000}
              className="w-full px-3 py-3 bg-gray-700 border border-gray-600 rounded-lg
                         text-white text-base placeholder-gray-500 resize-none
                         focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500
                         disabled:opacity-50"
            />
            <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2">
              <span className="text-xs text-gray-500 text-center sm:text-left hidden sm:block">
                Ctrl+Enter to save, Esc to cancel
              </span>
              <div className="flex gap-2 justify-end">
                <button
                  onClick={handleCancelEdit}
                  disabled={isSubmitting}
                  className="flex-1 sm:flex-none flex items-center justify-center gap-2 px-4 py-2.5 sm:p-1.5 text-gray-400 hover:text-white hover:bg-gray-700 rounded-lg sm:rounded transition-colors disabled:opacity-50 min-h-[44px] sm:min-h-0"
                  title="Cancel"
                >
                  <X className="w-4 h-4" />
                  <span className="sm:hidden">Cancel</span>
                </button>
                <button
                  onClick={handleSaveEdit}
                  disabled={isSubmitting || !editContent.trim()}
                  className="flex-1 sm:flex-none flex items-center justify-center gap-2 px-4 py-2.5 sm:p-1.5 text-green-400 hover:text-green-300 hover:bg-green-500/20 rounded-lg sm:rounded transition-colors disabled:opacity-50 min-h-[44px] sm:min-h-0"
                  title="Save"
                >
                  {isSubmitting ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <>
                      <Check className="w-4 h-4" />
                      <span className="sm:hidden">Save</span>
                    </>
                  )}
                </button>
              </div>
            </div>
          </div>
        ) : (
          <div className="mt-1 text-gray-300 break-words whitespace-pre-wrap">
            {formattedContent}
          </div>
        )}

        {/* Error message */}
        {error && (
          <div className="mt-2 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* Actions (only show when not editing) */}
        {!isEditing && (canEdit || canDelete) && (
          <div className="mt-2 flex gap-2">
            {canEdit && (
              <button
                onClick={handleStartEdit}
                className="text-xs text-gray-500 hover:text-gray-300 transition-colors"
              >
                Edit
              </button>
            )}
            {canDelete && (
              <>
                {showDeleteConfirm ? (
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-red-400">Delete?</span>
                    <button
                      onClick={handleDelete}
                      disabled={isSubmitting}
                      className="text-xs text-red-400 hover:text-red-300 font-medium transition-colors disabled:opacity-50"
                    >
                      {isSubmitting ? 'Deleting...' : 'Yes'}
                    </button>
                    <button
                      onClick={() => setShowDeleteConfirm(false)}
                      disabled={isSubmitting}
                      className="text-xs text-gray-500 hover:text-gray-300 transition-colors disabled:opacity-50"
                    >
                      No
                    </button>
                  </div>
                ) : (
                  <button
                    onClick={() => setShowDeleteConfirm(true)}
                    className="text-xs text-gray-500 hover:text-red-400 transition-colors"
                  >
                    Delete
                  </button>
                )}
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
