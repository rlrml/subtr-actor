import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { Video, Search, Upload, Loader2, AlertCircle, Film } from 'lucide-react';
import { ReplayCard } from '@/components/ReplayCard';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { useAuth } from '@/hooks/useAuth';
import { AuthRequiredMessage } from '@/components/AuthRequiredMessage';
import { EmailVerificationRequired } from '@/components/EmailVerificationRequired';
import { api } from '@/services/api';
import { toast } from 'sonner';

interface Replay {
  id: string;
  originalFilename: string;
  title: string | null;
  visibility: 'public' | 'unlisted';
  mapName: string | null;
  gameMode: string | null;
  team0Score: number | null;
  team1Score: number | null;
  durationSeconds: number | null;
  playedAt: string | null;
  createdAt: string;
  status: string;
  ownerId: string | null;
  players: Array<{
    id: string;
    name: string;
    team: number;
    score: number | null;
    goals: number | null;
    assists: number | null;
    saves: number | null;
    shots: number | null;
  }>;
}

interface ReplaysResponse {
  replays: Replay[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export default function MyReplays() {
  const { isAuthenticated, isLoading: authLoading, user } = useAuth();
  const [replays, setReplays] = useState<Replay[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [search, setSearch] = useState('');
  const [searchInput, setSearchInput] = useState('');

  const fetchReplays = useCallback(async () => {
    if (!isAuthenticated) return;

    setIsLoading(true);
    setError(null);

    try {
      const params = new URLSearchParams({
        page: page.toString(),
        limit: '12',
      });

      if (search) {
        params.append('search', search);
      }

      const data = await api.get<ReplaysResponse>(`/users/me/replays?${params.toString()}`);
      setReplays(data.replays);
      setTotalPages(data.totalPages);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load replays');
    } finally {
      setIsLoading(false);
    }
  }, [isAuthenticated, page, search]);

  useEffect(() => {
    fetchReplays();
  }, [fetchReplays]);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setSearch(searchInput);
    setPage(1);
  };

  const handleDelete = (deletedId: string) => {
    setReplays((prev) => prev.filter((r) => r.id !== deletedId));
  };

  const handleVisibilityChange = useCallback(async (id: string, newVisibility: 'public' | 'unlisted') => {
    // Optimistic update
    setReplays((prev) =>
      prev.map((r) => (r.id === id ? { ...r, visibility: newVisibility } : r))
    );

    try {
      await api.patch(`/replays/${id}`, { visibility: newVisibility });
      toast.success(`Replay is now ${newVisibility}`);
    } catch (err) {
      // Revert on error
      const oldVisibility = newVisibility === 'public' ? 'unlisted' : 'public';
      setReplays((prev) =>
        prev.map((r) => (r.id === id ? { ...r, visibility: oldVisibility } : r))
      );
      toast.error('Failed to update visibility');
    }
  }, []);

  if (authLoading) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-violet-500" />
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <AuthRequiredMessage
        title="Sign In to View Your Replays"
        message="You need to be signed in to see your uploaded replays."
        returnTo="/my-replays"
      />
    );
  }

  if (user && !user.emailVerified) {
    return (
      <EmailVerificationRequired
        title="Verify Your Email"
        message="Please verify your email address to access your replays."
      />
    );
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="flex items-center gap-4">
          <div className="w-14 h-14 rounded-xl bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center">
            <Video className="w-7 h-7 text-white" />
          </div>
          <div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-violet-400 via-blue-400 to-cyan-400 bg-clip-text text-transparent">
              My Replays
            </h1>
            <p className="text-gray-400 mt-1">
              Manage your uploaded replays
            </p>
          </div>
        </div>

        <Link to="/upload">
          <GradientButton>
            <Upload className="w-4 h-4" />
            Upload New
          </GradientButton>
        </Link>
      </div>

      {/* Search */}
      <form onSubmit={handleSearch} className="flex gap-3">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            placeholder="Search your replays..."
            className="w-full pl-10 pr-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
          />
        </div>
        <button
          type="submit"
          className="px-6 py-3 rounded-lg bg-gray-800 text-white hover:bg-gray-700 transition-colors"
        >
          Search
        </button>
      </form>

      {/* Content */}
      {isLoading ? (
        <div className="flex items-center justify-center py-20">
          <Loader2 className="w-8 h-8 animate-spin text-violet-500" />
        </div>
      ) : error ? (
        <AuthCard>
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-red-400 mx-auto mb-4" />
            <p className="text-red-400">{error}</p>
            <button
              onClick={fetchReplays}
              className="mt-4 text-violet-400 hover:text-violet-300 underline"
            >
              Try again
            </button>
          </div>
        </AuthCard>
      ) : replays.length === 0 ? (
        <AuthCard>
          <div className="text-center">
            <Film className="w-16 h-16 text-gray-600 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-white mb-2">
              {search ? 'No replays found' : 'No replays yet'}
            </h3>
            <p className="text-gray-400 mb-6">
              {search
                ? 'Try a different search term'
                : 'Upload your first replay to get started'}
            </p>
            {!search && (
              <Link to="/upload">
                <GradientButton>
                  <Upload className="w-4 h-4" />
                  Upload Replay
                </GradientButton>
              </Link>
            )}
          </div>
        </AuthCard>
      ) : (
        <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {replays.map((replay) => (
              <ReplayCard
                key={replay.id}
                replay={replay}
                showDeleteButton
                showVisibilityToggle
                currentUserId={user?.id}
                onDelete={handleDelete}
                onVisibilityChange={handleVisibilityChange}
              />
            ))}
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-center gap-2">
              <button
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={page === 1}
                className="px-4 py-2 rounded-lg bg-gray-800 text-white disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-700 transition-colors"
              >
                Previous
              </button>
              <span className="px-4 py-2 text-gray-400">
                Page {page} of {totalPages}
              </span>
              <button
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                disabled={page === totalPages}
                className="px-4 py-2 rounded-lg bg-gray-800 text-white disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-700 transition-colors"
              >
                Next
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
