import { Link } from 'react-router-dom';
import { LogIn, UserPlus, Lock } from 'lucide-react';
import { AuthCard } from '@/components/ui/GradientCard';
import { GradientButton } from '@/components/ui/GradientButton';

interface AuthRequiredMessageProps {
  title?: string;
  message?: string;
  returnTo?: string;
}

export function AuthRequiredMessage({
  title = 'Authentication Required',
  message = 'You need to be signed in to access this feature.',
  returnTo,
}: AuthRequiredMessageProps) {
  const loginUrl = returnTo ? `/login?returnTo=${encodeURIComponent(returnTo)}` : '/login';
  const registerUrl = returnTo ? `/register?returnTo=${encodeURIComponent(returnTo)}` : '/register';

  return (
    <div className="flex items-center justify-center py-12 px-4">
      <AuthCard className="w-full max-w-md text-center">
        <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-r from-violet-500/20 to-fuchsia-500/20 mb-4">
          <Lock className="w-8 h-8 text-violet-400" />
        </div>

        <h2 className="text-xl font-bold text-white mb-2">{title}</h2>
        <p className="text-gray-400 mb-6">{message}</p>

        <div className="flex flex-col sm:flex-row gap-3 justify-center">
          <Link to={loginUrl}>
            <GradientButton className="w-full sm:w-auto px-6 py-3">
              <LogIn className="w-4 h-4 mr-2" />
              Sign In
            </GradientButton>
          </Link>
          <Link to={registerUrl}>
            <button className="w-full sm:w-auto px-6 py-3 rounded-lg border border-gray-700 text-gray-300 hover:bg-gray-800/50 hover:text-white transition-all flex items-center justify-center gap-2">
              <UserPlus className="w-4 h-4" />
              Create Account
            </button>
          </Link>
        </div>
      </AuthCard>
    </div>
  );
}
