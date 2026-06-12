import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import {
  Settings,
  User,
  Mail,
  Lock,
  Loader2,
  CheckCircle2,
  AlertCircle,
  Eye,
  EyeOff,
} from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { AuthRequiredMessage } from '@/components/AuthRequiredMessage';
import { EmailVerificationRequired } from '@/components/EmailVerificationRequired';
import { LinkedAccountsSection } from '@/components/profile/LinkedAccountsSection';
import { useAuth } from '@/hooks/useAuth';
import { updateProfile, changePassword, changeEmail } from '@/api/auth';

const usernameSchema = z.object({
  username: z
    .string()
    .min(3, 'Username must be at least 3 characters')
    .max(50, 'Username must be at most 50 characters')
    .regex(/^[a-zA-Z0-9_-]+$/, 'Username can only contain letters, numbers, underscores, and hyphens'),
});

const passwordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z
    .string()
    .min(8, 'Password must be at least 8 characters')
    .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
    .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
    .regex(/[0-9]/, 'Password must contain at least one number'),
  confirmPassword: z.string(),
}).refine((data) => data.newPassword === data.confirmPassword, {
  message: "Passwords don't match",
  path: ['confirmPassword'],
});

const emailSchema = z.object({
  newEmail: z.string().email('Invalid email address'),
});

type UsernameForm = z.infer<typeof usernameSchema>;
type PasswordForm = z.infer<typeof passwordSchema>;
type EmailForm = z.infer<typeof emailSchema>;

export default function Profile() {
  const navigate = useNavigate();
  const { user, isLoading: authLoading, isAuthenticated, refreshUser } = useAuth();

  const [usernameLoading, setUsernameLoading] = useState(false);
  const [usernameSuccess, setUsernameSuccess] = useState(false);
  const [usernameError, setUsernameError] = useState<string | null>(null);

  const [passwordLoading, setPasswordLoading] = useState(false);
  const [passwordSuccess, setPasswordSuccess] = useState(false);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [showPasswords, setShowPasswords] = useState(false);

  const [emailLoading, setEmailLoading] = useState(false);
  const [emailSuccess, setEmailSuccess] = useState(false);
  const [emailError, setEmailError] = useState<string | null>(null);

  const usernameForm = useForm<UsernameForm>({
    resolver: zodResolver(usernameSchema),
    defaultValues: {
      username: user?.username || '',
    },
  });

  const passwordForm = useForm<PasswordForm>({
    resolver: zodResolver(passwordSchema),
  });

  const emailForm = useForm<EmailForm>({
    resolver: zodResolver(emailSchema),
  });

  if (authLoading) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-violet-500" />
      </div>
    );
  }

  if (!isAuthenticated || !user) {
    return (
      <AuthRequiredMessage
        title="Sign In to View Profile"
        message="You need to be signed in to access your profile settings."
        returnTo="/profile"
      />
    );
  }

  if (!user.emailVerified) {
    return (
      <EmailVerificationRequired
        title="Verify Your Email"
        message="Please verify your email address to access your profile settings."
      />
    );
  }

  const onUsernameSubmit = async (data: UsernameForm) => {
    setUsernameLoading(true);
    setUsernameError(null);
    setUsernameSuccess(false);

    try {
      await updateProfile({ username: data.username });
      await refreshUser();
      setUsernameSuccess(true);
      setTimeout(() => setUsernameSuccess(false), 3000);
    } catch (err) {
      setUsernameError(err instanceof Error ? err.message : 'Failed to update username');
    } finally {
      setUsernameLoading(false);
    }
  };

  const onPasswordSubmit = async (data: PasswordForm) => {
    setPasswordLoading(true);
    setPasswordError(null);
    setPasswordSuccess(false);

    try {
      await changePassword(data.currentPassword, data.newPassword);
      setPasswordSuccess(true);
      passwordForm.reset();
      // User will be logged out, redirect to login
      setTimeout(() => {
        navigate('/login');
      }, 2000);
    } catch (err) {
      setPasswordError(err instanceof Error ? err.message : 'Failed to change password');
    } finally {
      setPasswordLoading(false);
    }
  };

  const onEmailSubmit = async (data: EmailForm) => {
    setEmailLoading(true);
    setEmailError(null);
    setEmailSuccess(false);

    try {
      await changeEmail(data.newEmail);
      setEmailSuccess(true);
      emailForm.reset();
    } catch (err) {
      setEmailError(err instanceof Error ? err.message : 'Failed to request email change');
    } finally {
      setEmailLoading(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto space-y-8">
      <SEOHead
        title="Profile Settings"
        description="Manage your BallCam account settings, username, email, and password."
        noIndex
      />

      {/* Header */}
      <div className="flex items-center gap-4">
        <div className="w-14 h-14 rounded-xl bg-violet-500/20 flex items-center justify-center">
          <Settings className="w-7 h-7 text-violet-400" />
        </div>
        <div>
          <h1 className="text-3xl font-bold text-white">
            Profile Settings
          </h1>
          <p className="text-gray-400 mt-1">
            Manage your account settings
          </p>
        </div>
      </div>

      {/* Profile Info */}
      <AuthCard>
        <div className="flex items-center gap-4">
          {user.avatarUrl ? (
            <img
              src={user.avatarUrl}
              alt={user.username}
              className="w-16 h-16 rounded-full object-cover"
            />
          ) : (
            <div className="w-16 h-16 rounded-full bg-gradient-to-r from-violet-600 to-blue-600 flex items-center justify-center">
              <span className="text-2xl font-bold text-white">
                {user.username.charAt(0).toUpperCase()}
              </span>
            </div>
          )}
          <div>
            <h2 className="text-xl font-semibold text-white">{user.username}</h2>
            <p className="text-gray-400">{user.email}</p>
            {!user.emailVerified && (
              <span className="inline-flex items-center gap-1 mt-1 text-xs text-amber-400">
                <AlertCircle className="w-3 h-3" />
                Email not verified
              </span>
            )}
          </div>
        </div>
      </AuthCard>

      {/* Username Section */}
      <AuthCard>
        <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <User className="w-5 h-5 text-violet-400" />
          Change Username
        </h3>

        {usernameError && (
          <div className="mb-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
            {usernameError}
          </div>
        )}

        {usernameSuccess && (
          <div className="mb-4 p-3 rounded-lg bg-green-500/10 border border-green-500/20 text-green-400 text-sm flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4" />
            Username updated successfully!
          </div>
        )}

        <form onSubmit={usernameForm.handleSubmit(onUsernameSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Username
            </label>
            <input
              {...usernameForm.register('username')}
              type="text"
              className="w-full px-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
            />
            {usernameForm.formState.errors.username && (
              <p className="mt-1 text-sm text-red-400">
                {usernameForm.formState.errors.username.message}
              </p>
            )}
          </div>

          <GradientButton type="submit" disabled={usernameLoading}>
            {usernameLoading ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin mr-2" />
                Saving...
              </>
            ) : (
              'Update Username'
            )}
          </GradientButton>
        </form>
      </AuthCard>

      {/* Email Section */}
      <AuthCard>
        <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <Mail className="w-5 h-5 text-violet-400" />
          Change Email
        </h3>

        <p className="text-gray-400 text-sm mb-4">
          Current email: <span className="text-white">{user.email}</span>
        </p>

        {emailError && (
          <div className="mb-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
            {emailError}
          </div>
        )}

        {emailSuccess && (
          <div className="mb-4 p-3 rounded-lg bg-green-500/10 border border-green-500/20 text-green-400 text-sm flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4" />
            Verification email sent! Check your inbox.
          </div>
        )}

        <form onSubmit={emailForm.handleSubmit(onEmailSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              New Email Address
            </label>
            <input
              {...emailForm.register('newEmail')}
              type="email"
              placeholder="newemail@example.com"
              className="w-full px-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
            />
            {emailForm.formState.errors.newEmail && (
              <p className="mt-1 text-sm text-red-400">
                {emailForm.formState.errors.newEmail.message}
              </p>
            )}
          </div>

          <GradientButton type="submit" disabled={emailLoading}>
            {emailLoading ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin mr-2" />
                Sending...
              </>
            ) : (
              'Send Verification Email'
            )}
          </GradientButton>
        </form>
      </AuthCard>

      {/* Linked Accounts Section */}
      <LinkedAccountsSection hasPassword={user.hasPassword} />

      {/* Password Section */}
      {user.hasPassword && (
        <AuthCard>
          <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
            <Lock className="w-5 h-5 text-violet-400" />
            Change Password
          </h3>

          {passwordError && (
            <div className="mb-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
              {passwordError}
            </div>
          )}

          {passwordSuccess && (
            <div className="mb-4 p-3 rounded-lg bg-green-500/10 border border-green-500/20 text-green-400 text-sm flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4" />
              Password changed! Redirecting to login...
            </div>
          )}

          <form onSubmit={passwordForm.handleSubmit(onPasswordSubmit)} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Current Password
              </label>
              <div className="relative">
                <input
                  {...passwordForm.register('currentPassword')}
                  type={showPasswords ? 'text' : 'password'}
                  className="w-full px-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
                />
                <button
                  type="button"
                  onClick={() => setShowPasswords(!showPasswords)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300"
                >
                  {showPasswords ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
                </button>
              </div>
              {passwordForm.formState.errors.currentPassword && (
                <p className="mt-1 text-sm text-red-400">
                  {passwordForm.formState.errors.currentPassword.message}
                </p>
              )}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                New Password
              </label>
              <input
                {...passwordForm.register('newPassword')}
                type={showPasswords ? 'text' : 'password'}
                className="w-full px-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
              />
              {passwordForm.formState.errors.newPassword && (
                <p className="mt-1 text-sm text-red-400">
                  {passwordForm.formState.errors.newPassword.message}
                </p>
              )}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Confirm New Password
              </label>
              <input
                {...passwordForm.register('confirmPassword')}
                type={showPasswords ? 'text' : 'password'}
                className="w-full px-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
              />
              {passwordForm.formState.errors.confirmPassword && (
                <p className="mt-1 text-sm text-red-400">
                  {passwordForm.formState.errors.confirmPassword.message}
                </p>
              )}
            </div>

            <GradientButton type="submit" disabled={passwordLoading}>
              {passwordLoading ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin mr-2" />
                  Changing...
                </>
              ) : (
                'Change Password'
              )}
            </GradientButton>
          </form>
        </AuthCard>
      )}
    </div>
  );
}
