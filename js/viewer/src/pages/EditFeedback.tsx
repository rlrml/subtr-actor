import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { ArrowLeft, Save, AlertCircle } from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { feedbackApi } from '@/api/feedback.api';
import { RichEditor } from '@/components/feedback/RichEditor';
import type { FeedbackPost } from '@/api/feedback.api';

const TITLE_MIN_LENGTH = 5;
const TITLE_MAX_LENGTH = 200;
const CONTENT_MIN_LENGTH = 10;
const CONTENT_MAX_LENGTH = 10000;

export default function EditFeedback() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { user, isLoading: authLoading, isAuthenticated } = useAuth();

  // Data state
  const [post, setPost] = useState<FeedbackPost | null>(null);

  // Form state
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');

  // UI state
  const [isLoading, setIsLoading] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [touched, setTouched] = useState({ title: false, content: false });

  // Load post data
  useEffect(() => {
    const loadPost = async () => {
      if (!id) return;

      try {
        const response = await feedbackApi.getPost(id);
        setPost(response.post);
        setTitle(response.post.title);
        setContent(response.post.content);
      } catch (err) {
        console.error('Failed to load post:', err);
        setError('Post not found or has been deleted.');
      } finally {
        setIsLoading(false);
      }
    };

    loadPost();
  }, [id]);

  // Redirect if not authenticated or not author
  useEffect(() => {
    if (authLoading || isLoading) return;

    if (!isAuthenticated) {
      navigate(`/login?redirect=/feedback/${id}/edit`);
      return;
    }

    if (post && user && post.author.id !== user.id) {
      navigate(`/feedback/${id}`);
    }
  }, [authLoading, isAuthenticated, isLoading, post, user, id, navigate]);

  // Validation
  const titleError = touched.title
    ? title.length < TITLE_MIN_LENGTH
      ? `Title must be at least ${TITLE_MIN_LENGTH} characters`
      : title.length > TITLE_MAX_LENGTH
        ? `Title must be less than ${TITLE_MAX_LENGTH} characters`
        : null
    : null;

  const contentError = touched.content
    ? content.length < CONTENT_MIN_LENGTH
      ? `Content must be at least ${CONTENT_MIN_LENGTH} characters`
      : content.length > CONTENT_MAX_LENGTH
        ? `Content must be less than ${CONTENT_MAX_LENGTH} characters`
        : null
    : null;

  const isValid =
    title.length >= TITLE_MIN_LENGTH &&
    title.length <= TITLE_MAX_LENGTH &&
    content.length >= CONTENT_MIN_LENGTH &&
    content.length <= CONTENT_MAX_LENGTH;

  const hasChanges = post && (title !== post.title || content !== post.content);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setTouched({ title: true, content: true });

    if (!isValid || !post) return;

    setIsSubmitting(true);
    setError(null);

    try {
      await feedbackApi.updatePost(post.id, {
        title: title.trim(),
        content: content.trim(),
      });
      navigate(`/feedback/${post.id}`);
    } catch (err: unknown) {
      console.error('Failed to update post:', err);
      const apiError = err as { message?: string };
      setError(apiError.message || 'Failed to update post. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Loading state
  if (isLoading || authLoading) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-violet-500" />
      </div>
    );
  }

  // Error state (post not found)
  if (error && !post) {
    return (
      <div className="max-w-2xl mx-auto">
        <Link
          to="/feedback"
          className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to Feedback Hub
        </Link>
        <div className="text-center py-12">
          <h2 className="text-xl font-bold text-white mb-2">Post Not Found</h2>
          <p className="text-gray-400">{error}</p>
        </div>
      </div>
    );
  }

  if (!post) return null;

  return (
    <div className="max-w-2xl mx-auto">
      {/* Back link */}
      <Link
        to={`/feedback/${post.id}`}
        className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
      >
        <ArrowLeft className="w-4 h-4" />
        Back to Post
      </Link>

      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-violet-400 to-blue-400 bg-clip-text text-transparent">
          Edit Feedback Post
        </h1>
        <p className="text-gray-400 mt-2">
          Update your feedback post
        </p>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-6 p-4 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 flex items-center gap-3">
          <AlertCircle className="w-5 h-5 shrink-0" />
          {error}
        </div>
      )}

      {/* Form */}
      <form onSubmit={handleSubmit} className="space-y-6">
        {/* Category (read-only) */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Category
          </label>
          <div
            className="inline-flex items-center px-3 py-2 rounded-lg border border-gray-700 bg-gray-800/50 text-sm"
            style={{ color: post.category.color }}
          >
            {post.category.name}
          </div>
          <p className="mt-1 text-xs text-gray-500">Category cannot be changed after creation</p>
        </div>

        {/* Title */}
        <div>
          <label htmlFor="title" className="block text-sm font-medium text-gray-300 mb-2">
            Title <span className="text-red-400">*</span>
          </label>
          <input
            type="text"
            id="title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            onBlur={() => setTouched((t) => ({ ...t, title: true }))}
            placeholder="A clear, concise title for your feedback"
            maxLength={TITLE_MAX_LENGTH}
            className={`
              w-full px-4 py-3 rounded-lg bg-gray-800/50 border text-white placeholder-gray-500
              focus:outline-none focus:ring-2 focus:ring-violet-500/50 transition-colors
              ${titleError ? 'border-red-500' : 'border-gray-700 focus:border-violet-500'}
            `}
          />
          <div className="flex items-center justify-between mt-2">
            {titleError ? (
              <p className="text-sm text-red-400">{titleError}</p>
            ) : (
              <span />
            )}
            <span className={`text-xs ${title.length > TITLE_MAX_LENGTH * 0.9 ? 'text-yellow-400' : 'text-gray-500'}`}>
              {title.length}/{TITLE_MAX_LENGTH}
            </span>
          </div>
        </div>

        {/* Content */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Description <span className="text-red-400">*</span>
          </label>
          <RichEditor
            value={content}
            onChange={(value) => {
              setContent(value);
              setTouched((t) => ({ ...t, content: true }));
            }}
            placeholder="Describe your feedback in detail..."
            minHeight="200px"
          />
          <div className="flex items-center justify-between mt-2">
            {contentError ? (
              <p className="text-sm text-red-400">{contentError}</p>
            ) : (
              <span className="text-xs text-gray-500">
                Supports markdown formatting
              </span>
            )}
            <span className={`text-xs ${content.length > CONTENT_MAX_LENGTH * 0.9 ? 'text-yellow-400' : 'text-gray-500'}`}>
              {content.length}/{CONTENT_MAX_LENGTH}
            </span>
          </div>
        </div>

        {/* Submit */}
        <div className="flex items-center justify-end gap-4 pt-4">
          <Link
            to={`/feedback/${post.id}`}
            className="px-4 py-2 rounded-lg text-gray-400 hover:text-white transition-colors"
          >
            Cancel
          </Link>
          <button
            type="submit"
            disabled={!isValid || isSubmitting || !hasChanges}
            className="flex items-center gap-2 px-6 py-2 rounded-lg bg-gradient-to-r from-violet-600 to-blue-600 text-white font-medium hover:from-violet-500 hover:to-blue-500 transition-all duration-200 shadow-lg shadow-violet-500/25 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? (
              <>
                <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="w-4 h-4" />
                Save Changes
              </>
            )}
          </button>
        </div>
      </form>
    </div>
  );
}
