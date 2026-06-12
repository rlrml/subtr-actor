import { useState } from 'react';
import { Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Mail, ArrowLeft, Loader2, CheckCircle2 } from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { forgotPassword } from '@/api/auth';

const forgotPasswordSchema = z.object({
  email: z.string().email('Invalid email address'),
});

type ForgotPasswordForm = z.infer<typeof forgotPasswordSchema>;

export default function ForgotPassword() {
  const [isLoading, setIsLoading] = useState(false);
  const [isSuccess, setIsSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<ForgotPasswordForm>({
    resolver: zodResolver(forgotPasswordSchema),
  });

  const onSubmit = async (data: ForgotPasswordForm) => {
    setIsLoading(true);
    setError(null);

    try {
      await forgotPassword(data.email);
      setIsSuccess(true);
    } catch (err) {
      // Always show success to prevent email enumeration
      setIsSuccess(true);
    } finally {
      setIsLoading(false);
    }
  };

  if (isSuccess) {
    return (
      <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <div className="inline-flex items-center justify-center w-14 h-14 rounded-full bg-green-500/20 mb-4">
              <CheckCircle2 className="w-7 h-7 text-green-400" />
            </div>
            <h1 className="text-2xl font-bold text-white mb-2">Check Your Email</h1>
            <p className="text-gray-400">
              If an account exists with that email address, we've sent a password reset link.
            </p>
          </div>

          <AuthCard>
            <p className="text-gray-400 text-sm mb-6 text-center">
              Check your inbox and spam folder. The link will expire in 1 hour.
            </p>
            <Link to="/login">
              <GradientButton className="w-full py-3">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back to Login
              </GradientButton>
            </Link>
          </AuthCard>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
      <SEOHead
        title="Forgot Password"
        description="Reset your BallCam account password."
        noIndex
      />
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-14 h-14 rounded-full bg-violet-500/20 mb-4">
            <Mail className="w-7 h-7 text-violet-400" />
          </div>
          <h1 className="text-2xl font-bold text-white mb-2">Forgot Password?</h1>
          <p className="text-gray-400">
            Enter your email and we'll send you a reset link
          </p>
        </div>

        <AuthCard>
          {error && (
            <div className="mb-6 p-4 rounded-lg bg-red-500/10 border border-red-500/20">
              <p className="text-sm text-red-400">{error}</p>
            </div>
          )}

          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-300 mb-2">
                Email Address
              </label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
                <input
                  {...register('email')}
                  type="email"
                  id="email"
                  placeholder="you@example.com"
                  className="w-full pl-10 pr-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
                />
              </div>
              {errors.email && (
                <p className="mt-1 text-sm text-red-400">{errors.email.message}</p>
              )}
            </div>

            <GradientButton
              type="submit"
              className="w-full py-3"
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-5 h-5 animate-spin mr-2" />
                  Sending...
                </>
              ) : (
                'Send Reset Link'
              )}
            </GradientButton>
          </form>
        </AuthCard>

        <p className="mt-8 text-center text-sm text-gray-500">
          Remember your password?{' '}
          <Link to="/login" className="text-violet-400 hover:text-violet-300 transition-colors font-medium">
            Sign in
          </Link>
        </p>
      </div>
    </div>
  );
}
