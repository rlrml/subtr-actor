import { useState, useEffect, useContext } from 'react';
import { X, ThumbsUp, ThumbsDown, Send } from 'lucide-react';
import { likeApi } from '@/api/like.api';
import { createComment } from '@/api/comment.api';
import { AuthContext } from '@/contexts/AuthContext';

interface FeedbackPopupProps {
  replayId: string;
  isVisible: boolean;
  onClose: () => void;
}

const SESSION_KEY_PREFIX = 'replay_feedback_shown_';

export function FeedbackPopup({ replayId, isVisible, onClose }: FeedbackPopupProps) {
  const authContext = useContext(AuthContext);
  const isAuthenticated = authContext?.isAuthenticated ?? false;

  const [step, setStep] = useState<'question' | 'comment' | 'done'>('question');
  const [liked, setLiked] = useState(false);
  const [comment, setComment] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [shouldShow, setShouldShow] = useState(false);

  // Check if popup was already shown in this session (only on mount/replayId change)
  useEffect(() => {
    const sessionKey = SESSION_KEY_PREFIX + replayId;
    const wasShown = sessionStorage.getItem(sessionKey) === 'true';
    if (!wasShown) {
      setShouldShow(true);
    }
  }, [replayId]);

  // Mark as shown when popup becomes visible
  useEffect(() => {
    if (isVisible && shouldShow) {
      const sessionKey = SESSION_KEY_PREFIX + replayId;
      sessionStorage.setItem(sessionKey, 'true');
    }
  }, [isVisible, shouldShow, replayId]);

  // Don't show for unauthenticated users or if already shown before
  if (!isAuthenticated || !isVisible || !shouldShow) {
    return null;
  }

  const handleYes = async () => {
    setIsSubmitting(true);
    try {
      await likeApi.toggleLike(replayId);
      setLiked(true);
      setStep('comment');
    } catch (err) {
      console.error('Failed to like:', err);
      setStep('comment');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleNo = () => {
    setLiked(false);
    setStep('comment');
  };

  const handleSubmitComment = async () => {
    if (!comment.trim()) {
      onClose();
      return;
    }

    setIsSubmitting(true);
    try {
      await createComment({
        entityType: 'replay',
        entityId: replayId,
        content: comment.trim(),
      });
      setStep('done');
    } catch (err) {
      console.error('Failed to post comment:', err);
      onClose();
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleSkipComment = () => {
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="relative w-full max-w-md mx-4 bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl overflow-hidden">
        {/* Close button */}
        <button
          onClick={onClose}
          className="absolute top-4 right-4 p-1.5 text-gray-400 hover:text-white rounded-lg hover:bg-gray-800 transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        <div className="p-6">
          {step === 'question' && (
            <div className="text-center space-y-6">
              <h3 className="text-xl font-semibold text-white">
                Did you enjoy this replay?
              </h3>
              <p className="text-gray-400">
                Your feedback helps others discover great content
              </p>
              <div className="flex justify-center gap-4">
                <button
                  onClick={handleYes}
                  disabled={isSubmitting}
                  className="flex items-center gap-2 px-6 py-3 bg-green-600 hover:bg-green-700 text-white rounded-xl disabled:opacity-50 transition-colors"
                >
                  <ThumbsUp className="w-5 h-5" />
                  Yes!
                </button>
                <button
                  onClick={handleNo}
                  disabled={isSubmitting}
                  className="flex items-center gap-2 px-6 py-3 border border-gray-600 text-gray-300 hover:bg-gray-800 rounded-xl disabled:opacity-50 transition-colors"
                >
                  <ThumbsDown className="w-5 h-5" />
                  Not really
                </button>
              </div>
            </div>
          )}

          {step === 'comment' && (
            <div className="space-y-4">
              <div className="text-center">
                {liked ? (
                  <p className="text-green-400 font-medium">Thanks for liking! 💚</p>
                ) : (
                  <p className="text-gray-400">No worries!</p>
                )}
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-2">
                  Want to leave a comment? (optional)
                </label>
                <textarea
                  value={comment}
                  onChange={(e) => setComment(e.target.value)}
                  placeholder="Share your thoughts..."
                  rows={3}
                  maxLength={2000}
                  className="w-full px-4 py-3 bg-gray-800 border border-gray-700 rounded-lg
                             text-white placeholder-gray-500 resize-none
                             focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500"
                />
              </div>
              <div className="flex justify-end gap-3">
                <button
                  onClick={handleSkipComment}
                  className="px-4 py-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors"
                >
                  Skip
                </button>
                <button
                  onClick={handleSubmitComment}
                  disabled={isSubmitting || !comment.trim()}
                  className="flex items-center gap-2 px-4 py-2 bg-violet-600 hover:bg-violet-700 text-white rounded-lg disabled:opacity-50 transition-colors"
                >
                  <Send className="w-4 h-4" />
                  Post Comment
                </button>
              </div>
            </div>
          )}

          {step === 'done' && (
            <div className="text-center py-4 space-y-4">
              <p className="text-green-400 font-medium text-lg">
                Thanks for your feedback! 🎉
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
