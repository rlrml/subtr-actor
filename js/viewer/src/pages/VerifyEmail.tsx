import { useEffect, useState } from 'react';
import { useSearchParams, Link } from 'react-router-dom';
import { CheckCircle2, XCircle, Loader2, Mail } from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { verifyEmail } from '@/api/auth';
import { useAuth } from '@/hooks/useAuth';

type VerificationState = 'loading' | 'success' | 'error' | 'no-token';

export default function VerifyEmail() {
  const [searchParams] = useSearchParams();
  const [state, setState] = useState<VerificationState>('loading');
  const [error, setError] = useState<string>('');
  const { refreshUser } = useAuth();

  const token = searchParams.get('token');

  useEffect(() => {
    if (!token) {
      setState('no-token');
      return;
    }

    const verify = async () => {
      try {
        await verifyEmail(token);
        // Refresh user data to update emailVerified status
        await refreshUser();
        setState('success');
      } catch (err) {
        setState('error');
        setError(
          err instanceof Error
            ? err.message
            : 'Failed to verify email. The link may have expired.'
        );
      }
    };

    verify();
  }, [token, refreshUser]);

  if (state === 'loading') {
    return (
      <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
        <AuthCard className="w-full max-w-md">
          <div className="text-center">
            <Loader2 className="w-12 h-12 text-violet-500 animate-spin mx-auto mb-4" />
            <h1 className="text-2xl font-bold text-white mb-2">Verifying Email</h1>
            <p className="text-gray-400">Please wait while we verify your email address...</p>
          </div>
        </AuthCard>
      </div>
    );
  }

  if (state === 'success') {
    return (
      <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
        <AuthCard className="w-full max-w-md">
          <div className="text-center">
            <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-r from-green-500 to-emerald-500 mb-4">
              <CheckCircle2 className="w-8 h-8 text-white" />
            </div>
            <h1 className="text-2xl font-bold text-white mb-2">Email Verified!</h1>
            <p className="text-gray-400 mb-6">
              Your email has been successfully verified. You can now access all features of your account.
            </p>
            <Link to="/">
              <GradientButton className="px-6 py-3">
                Go to Home
              </GradientButton>
            </Link>
          </div>
        </AuthCard>
      </div>
    );
  }

  if (state === 'no-token') {
    return (
      <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <div className="inline-flex items-center justify-center w-14 h-14 rounded-full bg-yellow-500/20 mb-4">
              <Mail className="w-7 h-7 text-yellow-400" />
            </div>
            <h1 className="text-2xl font-bold text-white mb-2">Verification Link Missing</h1>
            <p className="text-gray-400">
              No verification token found.
            </p>
          </div>

          <AuthCard>
            <div className="text-center">
              <p className="text-gray-400 text-sm mb-6">
                Please use the link sent to your email address.
              </p>
              <Link to="/">
                <GradientButton className="px-6 py-3">
                  Go to Home
                </GradientButton>
              </Link>
            </div>
          </AuthCard>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-14 h-14 rounded-full bg-red-500/20 mb-4">
            <XCircle className="w-7 h-7 text-red-400" />
          </div>
          <h1 className="text-2xl font-bold text-white mb-2">Verification Failed</h1>
          <p className="text-gray-400">
            {error || 'The verification link is invalid or has expired.'}
          </p>
        </div>

        <AuthCard>
          <div className="text-center">
            <Link to="/login">
              <GradientButton className="w-full py-3">
                Go to Login
              </GradientButton>
            </Link>
            <p className="text-sm text-gray-500 mt-4">
              Need a new verification email? Log in to request a new one.
            </p>
          </div>
        </AuthCard>
      </div>
    </div>
  );
}
