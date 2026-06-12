import { useState, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { ArrowLeft, Send, AlertCircle } from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { feedbackApi } from '@/api/feedback.api';
import { RichEditor } from '@/components/feedback/RichEditor';
import type { FeedbackCategory } from '@/api/feedback.api';

const TITLE_MIN_LENGTH = 5;
const TITLE_MAX_LENGTH = 200;
const CONTENT_MIN_LENGTH = 10;
const CONTENT_MAX_LENGTH = 10000;

export default function NewFeedback() {
  const navigate = useNavigate();
  const { isAuthenticated, isLoading: authLoading, user } = useAuth();

  // Form state
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [categoryId, setCategoryId] = useState('');
  const [categories, setCategories] = useState<FeedbackCategory[]>([]);

  // UI state
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [touched, setTouched] = useState({ title: false, content: false, category: false });

  // Load categories on mount
  useEffect(() => {
    const loadCategories = async () => {
      try {
        const response = await feedbackApi.getCategories();
        setCategories(response.categories);
        // Pre-select first category
        if (response.categories.length > 0) {
          setCategoryId(response.categories[0].id);
        }
      } catch (err) {
        console.error('Failed to load categories:', err);
      }
    };
    loadCategories();
  }, []);

  // Redirect to login if not authenticated
  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      navigate('/login?redirect=/feedback/new');
    }
  }, [authLoading, isAuthenticated, navigate]);

  // Check if email is verified
  const emailVerified = user?.emailVerified ?? false;

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

  const categoryError = touched.category && !categoryId ? 'Please select a category' : null;

  const isValid =
    title.length >= TITLE_MIN_LENGTH &&
    title.length <= TITLE_MAX_LENGTH &&
    content.length >= CONTENT_MIN_LENGTH &&
    content.length <= CONTENT_MAX_LENGTH &&
    categoryId !== '';

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setTouched({ title: true, content: true, category: true });

    if (!isValid) return;

    setIsSubmitting(true);
    setError(null);

    try {
      const response = await feedbackApi.createPost({
        title: title.trim(),
        content: content.trim(),
        categoryId,
      });
      navigate(`/feedback/${response.post.id}`);
    } catch (err: unknown) {
      console.error('Failed to create post:', err);
      const apiError = err as { message?: string };
      setError(apiError.message || 'Failed to create post. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Loading state
  if (authLoading) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-violet-500" />
      </div>
    );
  }

  // Email verification required
  if (!emailVerified) {
    return (
      <div className="max-w-4xl mx-auto">
        <Link
          to="/feedback"
          className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to Feedback Hub
        </Link>

        <div className="rounded-xl border border-yellow-500/30 bg-yellow-500/10 p-8 text-center">
          <AlertCircle className="w-12 h-12 text-yellow-400 mx-auto mb-4" />
          <h2 className="text-xl font-bold text-white mb-2">Email Verification Required</h2>
          <p className="text-gray-400 mb-6">
            You need to verify your email address before you can create feedback posts.
          </p>
          <Link
            to="/profile"
            className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-yellow-600 text-white font-medium hover:bg-yellow-500 transition-colors"
          >
            Go to Profile
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto">
      {/* Back link */}
      <Link
        to="/feedback"
        className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
      >
        <ArrowLeft className="w-4 h-4" />
        Back to Feedback Hub
      </Link>

      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-violet-400 to-blue-400 bg-clip-text text-transparent">
          New Feedback Post
        </h1>
        <p className="text-gray-400 mt-2">
          Share your ideas, report bugs, or suggest improvements
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
        {/* Category */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Category <span className="text-red-400">*</span>
          </label>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
            {categories.map((category) => (
              <button
                key={category.id}
                type="button"
                onClick={() => {
                  setCategoryId(category.id);
                  setTouched((t) => ({ ...t, category: true }));
                }}
                className={`
                  p-3 rounded-lg border transition-all text-left
                  ${categoryId === category.id
                    ? 'border-violet-500 bg-violet-500/20'
                    : 'border-gray-700 bg-gray-800/50 hover:border-gray-600'
                  }
                `}
              >
                <div
                  className="text-sm font-medium mb-1"
                  style={{ color: categoryId === category.id ? category.color : undefined }}
                >
                  {category.name}
                </div>
                {category.description && (
                  <div className="text-xs text-gray-500 line-clamp-2">
                    {category.description}
                  </div>
                )}
              </button>
            ))}
          </div>
          {categoryError && (
            <p className="mt-2 text-sm text-red-400">{categoryError}</p>
          )}
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
            placeholder="Describe your feedback in detail. You can use markdown formatting..."
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
            to="/feedback"
            className="px-4 py-2 rounded-lg text-gray-400 hover:text-white transition-colors"
          >
            Cancel
          </Link>
          <button
            type="submit"
            disabled={!isValid || isSubmitting}
            className="flex items-center gap-2 px-6 py-2 rounded-lg bg-gradient-to-r from-violet-600 to-blue-600 text-white font-medium hover:from-violet-500 hover:to-blue-500 transition-all duration-200 shadow-lg shadow-violet-500/25 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? (
              <>
                <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Submitting...
              </>
            ) : (
              <>
                <Send className="w-4 h-4" />
                Submit Feedback
              </>
            )}
          </button>
        </div>
      </form>
    </div>
  );
}
