import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { UserPlus, Mail, Lock, User, AlertCircle, Loader2, CheckCircle2 } from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { useAuth } from '@/hooks/useAuth';
import { getGoogleAuthUrl, getDiscordAuthUrl } from '@/api/auth';
import { FaGoogle, FaDiscord } from 'react-icons/fa';

const registerSchema = z.object({
  username: z
    .string()
    .min(3, 'Username must be at least 3 characters')
    .max(50, 'Username must be at most 50 characters')
    .regex(/^[a-zA-Z0-9_]+$/, 'Username can only contain letters, numbers, and underscores'),
  email: z.string().email('Invalid email address'),
  password: z
    .string()
    .min(8, 'Password must be at least 8 characters')
    .regex(
      /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/,
      'Password must contain at least one uppercase letter, one lowercase letter, and one number'
    ),
});

type RegisterForm = z.infer<typeof registerSchema>;

export default function Register() {
  const navigate = useNavigate();
  const { register: registerUser, error: authError, clearError } = useAuth();
  const [isLoading, setIsLoading] = useState(false);
  const [success, setSuccess] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
    watch,
  } = useForm<RegisterForm>({
    resolver: zodResolver(registerSchema),
  });

  const password = watch('password', '');

  const passwordChecks = [
    { label: 'At least 8 characters', valid: password.length >= 8 },
    { label: 'One uppercase letter', valid: /[A-Z]/.test(password) },
    { label: 'One lowercase letter', valid: /[a-z]/.test(password) },
    { label: 'One number', valid: /\d/.test(password) },
  ];

  const onSubmit = async (data: RegisterForm) => {
    setIsLoading(true);
    clearError();

    try {
      await registerUser(data);
      setSuccess(true);
      setTimeout(() => {
        navigate('/');
      }, 2000);
    } catch {
      // Error is handled by AuthContext
    } finally {
      setIsLoading(false);
    }
  };

  if (success) {
    return (
      <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
        <div className="w-full max-w-md p-8 text-center rounded-2xl bg-gray-900/80 backdrop-blur-sm border border-gray-800 shadow-xl">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-r from-green-500 to-emerald-500 mb-4">
            <CheckCircle2 className="w-8 h-8 text-white" />
          </div>
          <h1 className="text-2xl font-bold text-white mb-2">Account Created!</h1>
          <p className="text-gray-400 mb-4">
            Please check your email to verify your account.
          </p>
          <p className="text-sm text-gray-500">Redirecting to home page...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
      <SEOHead
        title="Create Account"
        description="Create your free BallCam account to upload Rocket League replays, save favorites, and collaborate with friends."
        noIndex
      />
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-14 h-14 rounded-full bg-violet-500/20 mb-4">
            <UserPlus className="w-7 h-7 text-violet-400" />
          </div>
          <h1 className="text-2xl font-bold text-white mb-2">Create Account</h1>
          <p className="text-gray-400">Join RLView to upload and manage your replays</p>
        </div>

        <AuthCard>
          {authError && (
            <div className="mb-6 p-4 rounded-lg bg-red-500/10 border border-red-500/20 flex items-start gap-3">
              <AlertCircle className="w-5 h-5 text-red-400 shrink-0 mt-0.5" />
              <p className="text-sm text-red-400">{authError}</p>
            </div>
          )}

          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            <div>
              <label htmlFor="username" className="block text-sm font-medium text-gray-300 mb-2">
                Username
              </label>
              <div className="relative">
                <User className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
                <input
                  {...register('username')}
                  type="text"
                  id="username"
                  placeholder="coolplayer123"
                  className="w-full pl-10 pr-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
                />
              </div>
              {errors.username && (
                <p className="mt-1 text-sm text-red-400">{errors.username.message}</p>
              )}
            </div>

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

              {password && (
                <div className="mt-3 space-y-1">
                  {passwordChecks.map((check, index) => (
                    <div key={index} className="flex items-center gap-2 text-sm">
                      <CheckCircle2
                        className={`w-4 h-4 ${
                          check.valid ? 'text-green-400' : 'text-gray-600'
                        }`}
                      />
                      <span className={check.valid ? 'text-green-400' : 'text-gray-500'}>
                        {check.label}
                      </span>
                    </div>
                  ))}
                </div>
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
                  Creating account...
                </>
              ) : (
                <>
                  <UserPlus className="w-5 h-5 mr-2" />
                  Create Account
                </>
              )}
            </GradientButton>
          </form>

          <div className="relative my-6">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-gray-700/50"></div>
            </div>
            <div className="relative flex justify-center text-sm">
              <span className="px-4 bg-gray-900 text-gray-500">Or sign up with</span>
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
          Already have an account?{' '}
          <Link to="/login" className="text-violet-400 hover:text-violet-300 transition-colors font-medium">
            Sign in
          </Link>
        </p>
      </div>
    </div>
  );
}
