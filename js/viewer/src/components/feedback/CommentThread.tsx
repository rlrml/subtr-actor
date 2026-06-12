import { useState } from 'react';
import { MessageSquare, MoreHorizontal, Pencil, Trash2 } from 'lucide-react';
import { SafeMarkdown } from './SafeMarkdown';
import { RichEditor } from './RichEditor';
import type { FeedbackComment } from '@/api/feedback.api';

function UserAvatar({ avatarUrl, username, size = 'sm' }: { avatarUrl: string | null; username: string; size?: 'sm' | 'md' }) {
  const sizeClass = size === 'sm' ? 'w-6 h-6' : 'w-8 h-8';
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

interface CommentThreadProps {
  comments: FeedbackComment[];
  currentUserId?: string;
  isAdmin?: boolean;
  onEditComment?: (commentId: string, newContent: string) => Promise<void>;
  onDeleteComment?: (commentId: string) => Promise<void>;
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
    year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
  });
}

interface CommentItemProps {
  comment: FeedbackComment;
  currentUserId?: string;
  isAdmin?: boolean;
  onEdit?: (newContent: string) => Promise<void>;
  onDelete?: () => Promise<void>;
}

function CommentItem({ comment, currentUserId, isAdmin, onEdit, onDelete }: CommentItemProps) {
  const [showMenu, setShowMenu] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(comment.content);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const isAuthor = currentUserId && comment.author?.id === currentUserId;
  const canEdit = isAuthor && !comment.isDeleted;
  const canDelete = (isAuthor || isAdmin) && !comment.isDeleted;

  const handleSaveEdit = async () => {
    if (!editContent.trim() || editContent === comment.content) {
      setIsEditing(false);
      return;
    }
    setIsSubmitting(true);
    try {
      await onEdit?.(editContent.trim());
      setIsEditing(false);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async () => {
    if (!window.confirm('Are you sure you want to delete this comment?')) return;
    setIsSubmitting(true);
    try {
      await onDelete?.();
    } finally {
      setIsSubmitting(false);
    }
  };

  if (comment.isDeleted) {
    return (
      <div className="p-4 rounded-xl bg-gray-900/30 border border-gray-800/50">
        <p className="text-gray-500 italic">[This comment has been deleted]</p>
      </div>
    );
  }

  const author = comment.author;

  return (
    <div className={`p-4 rounded-xl border transition-colors ${author?.isAdmin ? 'bg-violet-500/5 border-violet-500/30' : 'bg-gray-900/50 border-gray-800 hover:border-gray-700'}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <UserAvatar
            avatarUrl={author?.avatarUrl ?? null}
            username={author?.username ?? 'Unknown'}
            size="sm"
          />
          <span className={`font-medium ${author?.isAdmin ? 'text-violet-400' : 'text-gray-200'}`}>
            {author?.username ?? 'Unknown'}
          </span>
          {author?.isAdmin && (
            <span className="px-1.5 py-0.5 text-xs rounded bg-violet-500/20 text-violet-300 font-medium">
              Dev
            </span>
          )}
          <span className="text-gray-500 text-sm">
            {formatRelativeTime(comment.createdAt)}
          </span>
          {comment.isEdited && (
            <span className="text-gray-500 text-xs">(edited)</span>
          )}
        </div>

        {/* Actions menu */}
        {(canEdit || canDelete) && (
          <div className="relative">
            <button
              onClick={() => setShowMenu(!showMenu)}
              className="p-1 rounded hover:bg-gray-800 text-gray-500 hover:text-gray-300 transition-colors"
            >
              <MoreHorizontal className="w-4 h-4" />
            </button>

            {showMenu && (
              <>
                <div
                  className="fixed inset-0 z-10"
                  onClick={() => setShowMenu(false)}
                />
                <div className="absolute right-0 top-full mt-1 w-32 py-1 rounded-lg bg-gray-800 border border-gray-700 shadow-lg z-20">
                  {canEdit && (
                    <button
                      onClick={() => {
                        setShowMenu(false);
                        setIsEditing(true);
                      }}
                      className="flex items-center gap-2 w-full px-3 py-2 text-sm text-gray-300 hover:bg-gray-700"
                    >
                      <Pencil className="w-4 h-4" />
                      Edit
                    </button>
                  )}
                  {canDelete && (
                    <button
                      onClick={() => {
                        setShowMenu(false);
                        handleDelete();
                      }}
                      className="flex items-center gap-2 w-full px-3 py-2 text-sm text-red-400 hover:bg-gray-700"
                    >
                      <Trash2 className="w-4 h-4" />
                      Delete
                    </button>
                  )}
                </div>
              </>
            )}
          </div>
        )}
      </div>

      {/* Content */}
      {isEditing ? (
        <div className="space-y-3">
          <RichEditor
            value={editContent}
            onChange={setEditContent}
            placeholder="Write your comment..."
            minHeight="150px"
          />
          <div className="flex items-center gap-2 justify-end">
            <button
              onClick={() => {
                setIsEditing(false);
                setEditContent(comment.content);
              }}
              className="px-3 py-1.5 text-sm text-gray-400 hover:text-white"
              disabled={isSubmitting}
            >
              Cancel
            </button>
            <button
              onClick={handleSaveEdit}
              disabled={isSubmitting || !editContent.trim()}
              className="px-3 py-1.5 text-sm rounded bg-violet-600 text-white hover:bg-violet-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSubmitting ? 'Saving...' : 'Save'}
            </button>
          </div>
        </div>
      ) : (
        <SafeMarkdown content={comment.content} />
      )}
    </div>
  );
}

export function CommentThread({
  comments,
  currentUserId,
  isAdmin,
  onEditComment,
  onDeleteComment,
}: CommentThreadProps) {
  if (comments.length === 0) {
    return (
      <div className="text-center py-12 px-4 rounded-xl bg-gray-900/30 border border-gray-800/50">
        <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-gray-800/50 flex items-center justify-center">
          <MessageSquare className="w-6 h-6 text-gray-600" />
        </div>
        <p className="text-gray-400">No comments yet</p>
        <p className="text-gray-500 text-sm mt-1">Be the first to share your thoughts!</p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {comments.map((comment) => (
        <CommentItem
          key={comment.id}
          comment={comment}
          currentUserId={currentUserId}
          isAdmin={isAdmin}
          onEdit={onEditComment ? (content) => onEditComment(comment.id, content) : undefined}
          onDelete={onDeleteComment ? () => onDeleteComment(comment.id) : undefined}
        />
      ))}
    </div>
  );
}
