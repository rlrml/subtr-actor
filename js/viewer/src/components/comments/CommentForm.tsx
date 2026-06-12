import { useState, useRef } from 'react';
import { Send } from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { EmojiPicker } from './EmojiPicker';
import { createComment, type CommentWithAuthor } from '@/api/comment.api';

interface CommentFormProps {
  entityType: 'replay' | 'announcement';
  entityId: string;
  onCommentCreated: (comment: CommentWithAuthor) => void;
}

const MAX_LENGTH = 2000;

export function CommentForm({ entityType, entityId, onCommentCreated }: CommentFormProps) {
  const [content, setContent] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const trimmedContent = content.trim();
  const charCount = trimmedContent.length;
  const isValid = charCount > 0 && charCount <= MAX_LENGTH;

  const handleEmojiSelect = (emoji: string) => {
    const textarea = textareaRef.current;
    if (!textarea) {
      setContent((prev) => prev + emoji);
      return;
    }

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const newContent = content.slice(0, start) + emoji + content.slice(end);
    setContent(newContent);

    // Restore cursor position after emoji
    setTimeout(() => {
      textarea.selectionStart = textarea.selectionEnd = start + emoji.length;
      textarea.focus();
    }, 0);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValid || isSubmitting) return;

    setIsSubmitting(true);
    setError(null);

    try {
      const response = await createComment({
        entityType,
        entityId,
        content: trimmedContent,
      });
      onCommentCreated(response.comment);
      setContent('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to post comment');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Submit on Ctrl+Enter or Cmd+Enter
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      if (isValid && !isSubmitting) {
        handleSubmit(e);
      }
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <div className="relative">
        <textarea
          ref={textareaRef}
          value={content}
          onChange={(e) => setContent(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Write a comment..."
          rows={3}
          maxLength={MAX_LENGTH + 100} // Allow some buffer for validation
          disabled={isSubmitting}
          className="w-full px-4 py-3 bg-gray-800/50 border border-gray-700 rounded-lg
                     text-white placeholder-gray-500 resize-none
                     focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500
                     disabled:opacity-50 disabled:cursor-not-allowed"
        />
      </div>

      {error && (
        <div className="text-red-400 text-sm">
          {error}
        </div>
      )}

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <EmojiPicker onEmojiSelect={handleEmojiSelect} />
          <span className={`text-xs ${charCount > MAX_LENGTH ? 'text-red-400' : 'text-gray-500'}`}>
            {charCount}/{MAX_LENGTH}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-500 hidden sm:inline">
            Ctrl+Enter to send
          </span>
          <GradientButton
            type="submit"
            disabled={!isValid || isSubmitting}
            loading={isSubmitting}
          >
            <Send className="h-4 w-4" />
            {isSubmitting ? 'Posting...' : 'Post'}
          </GradientButton>
        </div>
      </div>
    </form>
  );
}
