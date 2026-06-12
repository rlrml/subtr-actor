import { useState, useEffect, useCallback } from 'react';
import { Link2, Loader2, Trash2, CheckCircle2, AlertCircle } from 'lucide-react';
import { FaGoogle, FaDiscord } from 'react-icons/fa';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { getLinkedAccounts, unlinkAccount, getGoogleAuthUrl, getDiscordAuthUrl } from '@/api/auth';
import type { OAuthLink } from '@/api/auth';

interface LinkedAccountsSectionProps {
  hasPassword: boolean;
}

export function LinkedAccountsSection({ hasPassword }: LinkedAccountsSectionProps) {
  const [links, setLinks] = useState<OAuthLink[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [unlinkingProvider, setUnlinkingProvider] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const fetchLinks = useCallback(async () => {
    try {
      const data = await getLinkedAccounts();
      setLinks(data.links);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load linked accounts');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchLinks();
  }, [fetchLinks]);

  const handleUnlink = async (provider: 'google' | 'discord') => {
    // Check if user can unlink
    const otherLinks = links.filter((l) => l.provider !== provider);
    if (!hasPassword && otherLinks.length === 0) {
      setError('You must have at least one authentication method. Set a password first or link another account.');
      return;
    }

    setUnlinkingProvider(provider);
    setError(null);
    setSuccessMessage(null);

    try {
      await unlinkAccount(provider);
      setLinks((prev) => prev.filter((l) => l.provider !== provider));
      setSuccessMessage(`${provider.charAt(0).toUpperCase() + provider.slice(1)} account unlinked successfully.`);
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to unlink account');
    } finally {
      setUnlinkingProvider(null);
    }
  };

  const googleLink = links.find((l) => l.provider === 'google');
  const discordLink = links.find((l) => l.provider === 'discord');

  return (
    <AuthCard>
      <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <Link2 className="w-5 h-5 text-violet-400" />
        Linked Accounts
      </h3>

      <p className="text-gray-400 text-sm mb-4">
        Connect your social accounts for easy sign-in.
      </p>

      {error && (
        <div className="mb-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm flex items-center gap-2">
          <AlertCircle className="w-4 h-4 flex-shrink-0" />
          {error}
        </div>
      )}

      {successMessage && (
        <div className="mb-4 p-3 rounded-lg bg-green-500/10 border border-green-500/20 text-green-400 text-sm flex items-center gap-2">
          <CheckCircle2 className="w-4 h-4" />
          {successMessage}
        </div>
      )}

      {isLoading ? (
        <div className="flex justify-center py-8">
          <Loader2 className="w-6 h-6 animate-spin text-violet-500" />
        </div>
      ) : (
        <div className="space-y-3">
          {/* Google */}
          <div className="flex items-center justify-between p-4 rounded-lg bg-gray-800/50 border border-gray-700">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-lg bg-white/10 flex items-center justify-center">
                <FaGoogle className="w-5 h-5 text-white" />
              </div>
              <div>
                <p className="font-medium text-white">Google</p>
                {googleLink ? (
                  <p className="text-sm text-gray-400">
                    {googleLink.providerEmail || googleLink.providerUsername || 'Connected'}
                  </p>
                ) : (
                  <p className="text-sm text-gray-500">Not connected</p>
                )}
              </div>
            </div>
            {googleLink ? (
              <button
                onClick={() => handleUnlink('google')}
                disabled={unlinkingProvider === 'google'}
                className="flex items-center gap-2 px-3 py-2 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors disabled:opacity-50"
              >
                {unlinkingProvider === 'google' ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Trash2 className="w-4 h-4" />
                )}
                Unlink
              </button>
            ) : (
              <a href={getGoogleAuthUrl(true)}>
                <GradientButton size="sm">
                  Connect
                </GradientButton>
              </a>
            )}
          </div>

          {/* Discord */}
          <div className="flex items-center justify-between p-4 rounded-lg bg-gray-800/50 border border-gray-700">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-lg bg-[#5865F2]/20 flex items-center justify-center">
                <FaDiscord className="w-5 h-5 text-[#5865F2]" />
              </div>
              <div>
                <p className="font-medium text-white">Discord</p>
                {discordLink ? (
                  <p className="text-sm text-gray-400">
                    {discordLink.providerUsername || discordLink.providerEmail || 'Connected'}
                  </p>
                ) : (
                  <p className="text-sm text-gray-500">Not connected</p>
                )}
              </div>
            </div>
            {discordLink ? (
              <button
                onClick={() => handleUnlink('discord')}
                disabled={unlinkingProvider === 'discord'}
                className="flex items-center gap-2 px-3 py-2 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors disabled:opacity-50"
              >
                {unlinkingProvider === 'discord' ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Trash2 className="w-4 h-4" />
                )}
                Unlink
              </button>
            ) : (
              <a href={getDiscordAuthUrl(true)}>
                <GradientButton size="sm">
                  Connect
                </GradientButton>
              </a>
            )}
          </div>
        </div>
      )}

      <p className="text-xs text-gray-500 mt-4">
        Note: You must always have at least one way to sign in. Set a password before unlinking all social accounts.
      </p>
    </AuthCard>
  );
}
