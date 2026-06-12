import { useState } from 'react';
import { Send, LogIn } from 'lucide-react';
import { Link } from 'react-router-dom';
import { RichEditor } from './RichEditor';
import { useAuth } from '@/hooks/useAuth';

interface CommentFormProps {
  onSubmit: (content: string) => Promise<void>;
  placeholder?: string;
}

const MIN_LENGTH = 3;
const MAX_LENGTH = 5000;

export function CommentForm({ onSubmit, placeholder = 'Share your thoughts...' }: CommentFormProps) {
  const { isAuthenticated, user } = useAuth();
  const [content, setContent] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const emailVerified = user?.emailVerified ?? false;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (content.length < MIN_LENGTH) {
      setError(`Comment must be at least ${MIN_LENGTH} characters`);
      return;
    }
    if (content.length > MAX_LENGTH) {
      setError(`Comment must be less than ${MAX_LENGTH} characters`);
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      await onSubmit(content.trim());
      setContent('');
    } catch (err: unknown) {
      const apiError = err as { message?: string };
      setError(apiError.message || 'Failed to post comment. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Not authenticated
  if (!isAuthenticated) {
    return (
      <div className="rounded-xl border border-gray-700 bg-gray-900/50 p-6 text-center">
        <LogIn className="w-8 h-8 text-gray-500 mx-auto mb-3" />
        <p className="text-gray-400 mb-4">Sign in to join the conversation</p>
        <Link
          to={`/login?redirect=${window.location.pathname}`}
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-violet-600 text-white font-medium hover:bg-violet-500 transition-colors"
        >
          <LogIn className="w-4 h-4" />
          Sign In
        </Link>
      </div>
    );
  }

  // Email not verified
  if (!emailVerified) {
    return (
      <div className="rounded-xl border border-yellow-500/30 bg-yellow-500/10 p-6 text-center">
        <p className="text-yellow-400 mb-2">Verify your email to comment</p>
        <Link
          to="/profile"
          className="text-sm text-yellow-300 hover:underline"
        >
          Go to Profile →
        </Link>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <RichEditor
        value={content}
        onChange={setContent}
        placeholder={placeholder}
        minHeight="120px"
      />

      {error && (
        <p className="text-sm text-red-400">{error}</p>
      )}

      <div className="flex items-center justify-between">
        <span className={`text-xs ${content.length > MAX_LENGTH * 0.9 ? 'text-yellow-400' : 'text-gray-500'}`}>
          {content.length}/{MAX_LENGTH}
        </span>
        <button
          type="submit"
          disabled={isSubmitting || content.length < MIN_LENGTH || content.length > MAX_LENGTH}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-violet-600 to-blue-600 text-white font-medium hover:from-violet-500 hover:to-blue-500 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isSubmitting ? (
            <>
              <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              Posting...
            </>
          ) : (
            <>
              <Send className="w-4 h-4" />
              Post Comment
            </>
          )}
        </button>
      </div>
    </form>
  );
}
