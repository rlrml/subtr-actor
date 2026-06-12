import { useState } from 'react';
import { Link, useNavigate, useLocation, useSearchParams } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { LogIn, Mail, Lock, AlertCircle, Loader2 } from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { useAuth } from '@/hooks/useAuth';
import { getGoogleAuthUrl, getDiscordAuthUrl } from '@/api/auth';
import { FaGoogle, FaDiscord } from 'react-icons/fa';

const loginSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string().min(1, 'Password is required'),
});

type LoginForm = z.infer<typeof loginSchema>;

export default function Login() {
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const { login, error: authError, clearError } = useAuth();
  const [isLoading, setIsLoading] = useState(false);

  // Support both location.state and query param for return URL
  const returnUrl = searchParams.get('returnUrl');
  const from = returnUrl || (location.state as { from?: { pathname: string } })?.from?.pathname || '/';

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginForm>({
    resolver: zodResolver(loginSchema),
  });

  const onSubmit = async (data: LoginForm) => {
    setIsLoading(true);
    clearError();

    try {
      await login(data);
      navigate(from, { replace: true });
    } catch {
      // Error is handled by AuthContext
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
      <SEOHead
        title="Sign In"
        description="Sign in to your BallCam account to upload replays, save favorites, and collaborate with friends."
        noIndex
      />
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-14 h-14 rounded-full bg-violet-500/20 mb-4">
            <LogIn className="w-7 h-7 text-violet-400" />
          </div>
          <h1 className="text-2xl font-bold text-white mb-2">Welcome Back</h1>
          <p className="text-gray-400">Sign in to your RLView account</p>
        </div>

        <AuthCard>
          {(authError || errors.root) && (
            <div className="mb-6 p-4 rounded-lg bg-red-500/10 border border-red-500/20 flex items-start gap-3">
              <AlertCircle className="w-5 h-5 text-red-400 shrink-0 mt-0.5" />
              <p className="text-sm text-red-400">{authError || errors.root?.message}</p>
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

            <div>
              <label htmlFor="password" className="block text-sm font-medium text-gray-300 mb-2">
                Password
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
                <input
                  {...register('password')}
                  type="password"
                  id="password"
                  placeholder="••••••••"
                  className="w-full pl-10 pr-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
                />
              </div>
              {errors.password && (
                <p className="mt-1 text-sm text-red-400">{errors.password.message}</p>
              )}
            </div>

            <div className="flex justify-end">
              <Link
                to="/forgot-password"
                className="text-sm text-violet-400 hover:text-violet-300 transition-colors"
              >
                Forgot password?
              </Link>
            </div>

            <GradientButton
              type="submit"
              className="w-full py-3"
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-5 h-5 animate-spin mr-2" />
                  Signing in...
                </>
              ) : (
                <>
                  <LogIn className="w-5 h-5 mr-2" />
                  Sign In
                </>
              )}
            </GradientButton>
          </form>

          <div className="relative my-6">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-gray-700/50"></div>
            </div>
            <div className="relative flex justify-center text-sm">
              <span className="px-4 bg-gray-900 text-gray-500">Or continue with</span>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <a
              href={getGoogleAuthUrl()}
              className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-gray-800/50 border border-gray-700/50 text-gray-300 hover:bg-gray-700/50 hover:text-white transition-all"
            >
              <FaGoogle className="w-4 h-4" />
              <span className="text-sm">Google</span>
            </a>
            <a
              href={getDiscordAuthUrl()}
              className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-gray-800/50 border border-gray-700/50 text-gray-300 hover:bg-gray-700/50 hover:text-white transition-all"
            >
              <FaDiscord className="w-4 h-4" />
              <span className="text-sm">Discord</span>
            </a>
          </div>
        </AuthCard>

        <p className="mt-8 text-center text-sm text-gray-500">
          Don't have an account?{' '}
          <Link to="/register" className="text-violet-400 hover:text-violet-300 transition-colors font-medium">
            Sign up
          </Link>
        </p>
      </div>
    </div>
  );
}
