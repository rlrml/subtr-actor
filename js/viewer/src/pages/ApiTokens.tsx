import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import {
  Key,
  Loader2,
  Plus,
  Trash2,
  Copy,
  Check,
  AlertCircle,
  Clock,
  Calendar,
  Book,
} from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { AuthRequiredMessage } from '@/components/AuthRequiredMessage';
import { EmailVerificationRequired } from '@/components/EmailVerificationRequired';
import { useAuth } from '@/hooks/useAuth';
import { tokenApi, type ApiToken } from '@/services/token.api';
import { toast } from 'sonner';

const createTokenSchema = z.object({
  name: z
    .string()
    .min(1, 'Name is required')
    .max(100, 'Name must be 100 characters or less'),
  expiresIn: z.enum(['never', '30d', '90d', '1y']),
});

type CreateTokenForm = z.infer<typeof createTokenSchema>;

function formatDate(dateString: string | null): string {
  if (!dateString) return 'Never';
  const date = new Date(dateString);
  return date.toLocaleDateString('fr-FR', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function formatRelativeTime(dateString: string | null): string {
  if (!dateString) return 'Never used';
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return 'Today';
  if (diffDays === 1) return 'Yesterday';
  if (diffDays < 7) return `${diffDays} days ago`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
  return formatDate(dateString);
}

function getExpirationDate(expiresIn: string): string | null {
  if (expiresIn === 'never') return null;

  const now = new Date();
  switch (expiresIn) {
    case '30d':
      now.setDate(now.getDate() + 30);
      break;
    case '90d':
      now.setDate(now.getDate() + 90);
      break;
    case '1y':
      now.setFullYear(now.getFullYear() + 1);
      break;
  }
  return now.toISOString();
}

export default function ApiTokens() {
  const { user, isLoading: authLoading, isAuthenticated } = useAuth();

  const [tokens, setTokens] = useState<ApiToken[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newToken, setNewToken] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const form = useForm<CreateTokenForm>({
    resolver: zodResolver(createTokenSchema),
    defaultValues: {
      name: '',
      expiresIn: 'never',
    },
  });

  const loadTokens = useCallback(async () => {
    try {
      const data = await tokenApi.list();
      setTokens(data);
    } catch (error) {
      toast.error('Failed to load tokens');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (isAuthenticated && user?.emailVerified) {
      loadTokens();
    }
  }, [isAuthenticated, user?.emailVerified, loadTokens]);

  const onSubmit = async (data: CreateTokenForm) => {
    setCreating(true);
    try {
      const expiresAt = getExpirationDate(data.expiresIn);
      const response = await tokenApi.create({
        name: data.name,
        expires_at: expiresAt,
      });

      setNewToken(response.token);
      setTokens((prev) => [
        {
          id: response.tokenInfo.id,
          name: response.tokenInfo.name,
          scope: response.tokenInfo.scope,
          expiresAt: response.tokenInfo.expiresAt,
          lastUsedAt: null,
          createdAt: response.tokenInfo.createdAt,
        },
        ...prev,
      ]);

      form.reset();
      toast.success('Token created successfully');
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : 'Failed to create token'
      );
    } finally {
      setCreating(false);
    }
  };

  const handleCopy = async () => {
    if (!newToken) return;
    try {
      await navigator.clipboard.writeText(newToken);
      setCopied(true);
      toast.success('Token copied to clipboard');
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error('Failed to copy token');
    }
  };

  const handleDelete = async (tokenId: string) => {
    if (!confirm('Are you sure you want to revoke this token? This cannot be undone.')) {
      return;
    }

    setDeletingId(tokenId);
    try {
      await tokenApi.revoke(tokenId);
      setTokens((prev) => prev.filter((t) => t.id !== tokenId));
      toast.success('Token revoked');
    } catch (error) {
      toast.error('Failed to revoke token');
    } finally {
      setDeletingId(null);
    }
  };

  const handleDismissNewToken = () => {
    setNewToken(null);
    setShowCreateForm(false);
  };

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
        title="Sign In to Manage API Tokens"
        message="You need to be signed in to create and manage API tokens."
        returnTo="/settings/tokens"
      />
    );
  }

  if (!user.emailVerified) {
    return <EmailVerificationRequired />;
  }

  return (
    <div className="max-w-4xl mx-auto px-4 py-8">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-3">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-violet-500 to-fuchsia-500 flex items-center justify-center">
            <Key className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white">API Tokens</h1>
            <p className="text-gray-400 text-sm">
              Manage Personal Access Tokens for API access
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <Link
            to="/docs/api"
            className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800/50 transition-colors"
          >
            <Book className="w-4 h-4" />
            <span className="hidden sm:inline">API Docs</span>
          </Link>
          {!showCreateForm && !newToken && (
            <GradientButton onClick={() => setShowCreateForm(true)}>
              <Plus className="w-4 h-4 mr-2" />
              New Token
            </GradientButton>
          )}
        </div>
      </div>

      {/* New Token Display (shown only once after creation) */}
      {newToken && (
        <AuthCard className="mb-6 border-green-500/30 bg-green-500/5">
          <div className="flex items-start gap-3">
            <div className="w-10 h-10 rounded-lg bg-green-500/20 flex items-center justify-center flex-shrink-0">
              <Key className="w-5 h-5 text-green-400" />
            </div>
            <div className="flex-1 min-w-0">
              <h3 className="text-lg font-semibold text-white mb-1">
                Token Created Successfully
              </h3>
              <p className="text-amber-400 text-sm mb-4">
                Copy this token now. You won't be able to see it again!
              </p>

              <div className="flex items-center gap-2 mb-4">
                <code className="flex-1 bg-gray-900/50 border border-gray-700 rounded-lg px-4 py-3 text-sm text-gray-300 font-mono overflow-x-auto">
                  {newToken}
                </code>
                <button
                  onClick={handleCopy}
                  className="p-3 rounded-lg bg-violet-500/20 hover:bg-violet-500/30 text-violet-400 transition-colors"
                  title="Copy to clipboard"
                >
                  {copied ? (
                    <Check className="w-5 h-5" />
                  ) : (
                    <Copy className="w-5 h-5" />
                  )}
                </button>
              </div>

              <GradientButton
                variant="secondary"
                onClick={handleDismissNewToken}
              >
                I've saved the token
              </GradientButton>
            </div>
          </div>
        </AuthCard>
      )}

      {/* Create Form */}
      {showCreateForm && !newToken && (
        <AuthCard className="mb-6">
          <h3 className="text-lg font-semibold text-white mb-4">
            Create New Token
          </h3>

          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Token Name
              </label>
              <input
                {...form.register('name')}
                placeholder="e.g., Tournament Site, Discord Bot"
                className="w-full bg-gray-900/50 border border-gray-700 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-violet-500 transition-colors"
              />
              {form.formState.errors.name && (
                <p className="mt-1 text-sm text-red-400">
                  {form.formState.errors.name.message}
                </p>
              )}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Expiration
              </label>
              <select
                {...form.register('expiresIn')}
                className="w-full bg-gray-900/50 border border-gray-700 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-violet-500 transition-colors"
              >
                <option value="never">Never expires</option>
                <option value="30d">30 days</option>
                <option value="90d">90 days</option>
                <option value="1y">1 year</option>
              </select>
            </div>

            <div className="flex items-center gap-3 pt-2">
              <GradientButton type="submit" disabled={creating}>
                {creating ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Creating...
                  </>
                ) : (
                  'Create Token'
                )}
              </GradientButton>
              <GradientButton
                type="button"
                variant="secondary"
                onClick={() => {
                  setShowCreateForm(false);
                  form.reset();
                }}
              >
                Cancel
              </GradientButton>
            </div>
          </form>
        </AuthCard>
      )}

      {/* Token List */}
      <AuthCard>
        <h3 className="text-lg font-semibold text-white mb-4">Active Tokens</h3>

        {loading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-8 h-8 text-violet-400 animate-spin" />
          </div>
        ) : tokens.length === 0 ? (
          <div className="text-center py-8">
            <Key className="w-12 h-12 text-gray-600 mx-auto mb-3" />
            <p className="text-gray-400">No API tokens yet</p>
            <p className="text-gray-500 text-sm mt-1">
              Create a token to start using the API
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {tokens.map((token) => (
              <div
                key={token.id}
                className="flex items-center justify-between p-4 rounded-lg bg-gray-900/30 border border-gray-800 hover:border-gray-700 transition-colors"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-medium text-white">{token.name}</span>
                    <span className="px-2 py-0.5 rounded text-xs bg-violet-500/20 text-violet-400">
                      {token.scope}
                    </span>
                  </div>
                  <div className="flex items-center gap-4 text-sm text-gray-500">
                    <span className="flex items-center gap-1">
                      <Calendar className="w-3.5 h-3.5" />
                      Created {formatDate(token.createdAt)}
                    </span>
                    <span className="flex items-center gap-1">
                      <Clock className="w-3.5 h-3.5" />
                      {formatRelativeTime(token.lastUsedAt)}
                    </span>
                    {token.expiresAt && (
                      <span className="flex items-center gap-1 text-amber-500">
                        <AlertCircle className="w-3.5 h-3.5" />
                        Expires {formatDate(token.expiresAt)}
                      </span>
                    )}
                  </div>
                </div>

                <button
                  onClick={() => handleDelete(token.id)}
                  disabled={deletingId === token.id}
                  className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-red-500/10 transition-colors disabled:opacity-50"
                  title="Revoke token"
                >
                  {deletingId === token.id ? (
                    <Loader2 className="w-5 h-5 animate-spin" />
                  ) : (
                    <Trash2 className="w-5 h-5" />
                  )}
                </button>
              </div>
            ))}
          </div>
        )}
      </AuthCard>

      {/* Usage Info */}
      <div className="mt-6 p-4 rounded-lg bg-gray-900/30 border border-gray-800">
        <div className="flex items-center justify-between">
          <div>
            <h4 className="text-sm font-medium text-gray-300 mb-1">
              Quick Start
            </h4>
            <p className="text-gray-500 text-sm">
              Use your token in the Authorization header: <code className="text-gray-400">Bearer pat_xxx</code>
            </p>
          </div>
          <Link
            to="/docs/api"
            className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-violet-500/20 text-violet-400 hover:bg-violet-500/30 transition-colors text-sm"
          >
            <Book className="w-4 h-4" />
            View Full Documentation
          </Link>
        </div>
      </div>
    </div>
  );
}
