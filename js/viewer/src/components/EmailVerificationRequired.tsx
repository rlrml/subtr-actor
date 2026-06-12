import { useState, useEffect } from 'react';
import { Mail, RefreshCw, CheckCircle, AlertCircle } from 'lucide-react';
import { AuthCard } from '@/components/ui/GradientCard';
import { GradientButton } from '@/components/ui/GradientButton';
import * as authApi from '@/api/auth';
import { useAuth } from '@/hooks/useAuth';

interface EmailVerificationRequiredProps {
  title?: string;
  message?: string;
}

export function EmailVerificationRequired({
  title = 'Email Verification Required',
  message = 'Please verify your email address to access this feature.',
}: EmailVerificationRequiredProps) {
  const { user } = useAuth();
  const [isResending, setIsResending] = useState(false);
  const [cooldown, setCooldown] = useState(0);
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  // Handle cooldown timer
  useEffect(() => {
    if (cooldown <= 0) return;

    const timer = setInterval(() => {
      setCooldown((prev) => Math.max(0, prev - 1));
    }, 1000);

    return () => clearInterval(timer);
  }, [cooldown]);

  const handleResend = async () => {
    if (isResending || cooldown > 0) return;

    setIsResending(true);
    setFeedback(null);

    try {
      const response = await authApi.resendVerificationEmail();
      setFeedback({ type: 'success', message: response.message });
      setCooldown(60); // 60 second cooldown after successful send
    } catch (err) {
      const error = err as { message?: string };
      const errorMessage = error.message || 'Failed to send verification email';

      // Extract remaining time from rate limit message if present
      const match = errorMessage.match(/wait (\d+) seconds/);
      if (match) {
        setCooldown(parseInt(match[1], 10));
      }

      setFeedback({ type: 'error', message: errorMessage });
    } finally {
      setIsResending(false);
    }
  };

  return (
    <div className="flex items-center justify-center py-12 px-4">
      <AuthCard className="w-full max-w-md text-center">
        <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-r from-amber-500/20 to-orange-500/20 mb-4">
          <Mail className="w-8 h-8 text-amber-400" />
        </div>

        <h2 className="text-xl font-bold text-white mb-2">{title}</h2>
        <p className="text-gray-400 mb-2">{message}</p>

        {user?.email && (
          <p className="text-sm text-gray-500 mb-6">
            A verification email was sent to <span className="text-gray-300">{user.email}</span>
          </p>
        )}

        {/* Feedback message */}
        {feedback && (
          <div
            className={`flex items-center gap-2 p-3 rounded-lg mb-4 ${
              feedback.type === 'success'
                ? 'bg-green-500/10 text-green-400'
                : 'bg-red-500/10 text-red-400'
            }`}
          >
            {feedback.type === 'success' ? (
              <CheckCircle className="w-5 h-5 flex-shrink-0" />
            ) : (
              <AlertCircle className="w-5 h-5 flex-shrink-0" />
            )}
            <span className="text-sm">{feedback.message}</span>
          </div>
        )}

        <GradientButton
          onClick={handleResend}
          disabled={isResending || cooldown > 0}
          className="w-full px-6 py-3"
        >
          {isResending ? (
            <>
              <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              Sending...
            </>
          ) : cooldown > 0 ? (
            <>
              <RefreshCw className="w-4 h-4 mr-2" />
              Resend in {cooldown}s
            </>
          ) : (
            <>
              <Mail className="w-4 h-4 mr-2" />
              Resend Verification Email
            </>
          )}
        </GradientButton>

        <p className="text-xs text-gray-500 mt-4">
          Check your spam folder if you don't see the email.
        </p>
      </AuthCard>
    </div>
  );
}
