import { useState, useEffect, useCallback } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { Loader2, Upload, RefreshCw, Play, FolderOpen, RotateCcw } from 'lucide-react';
import { api } from '@/services/api';
import { ReplayCard } from '@/components/ReplayCard';
import { SearchFilters, FilterState, DEFAULT_FILTERS } from '@/components/SearchFilters';
import { Pagination } from '@/components/Pagination';
import { GradientButton } from '@/components/ui/GradientButton';
import { useAuth } from '@/hooks/useAuth';
import { QualityInfoBanner } from '@/components/QualityInfoBanner';

interface Player {
  id: string;
  name: string;
  team: number;
  goals?: number;
  assists?: number;
  saves?: number;
  score?: number;
}

interface ReplayOwner {
  id: string;
  username: string;
  avatarUrl: string | null;
}

interface Replay {
  id: string;
  originalFilename: string;
  title?: string | null;
  visibility?: 'public' | 'unlisted';
  mapName?: string;
  gameMode?: string;
  team0Score?: number;
  team1Score?: number;
  durationSeconds?: number;
  playedAt?: string;
  status: string;
  players: Player[];
  owner?: ReplayOwner | null;
}

interface ReplaysResponse {
  replays: Replay[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export default function Replays() {
  const { user } = useAuth();
  const [searchParams, setSearchParams] = useSearchParams();
  const [replays, setReplays] = useState<Replay[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [page, setPage] = useState(() => parseInt(searchParams.get('page') || '1', 10));
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);

  // Initialize filters from URL params
  const [filters, setFilters] = useState<FilterState>(() => ({
    search: searchParams.get('search') || '',
    map: searchParams.get('map') || '',
    sortBy: (searchParams.get('sortBy') as FilterState['sortBy']) || 'createdAt',
    sortOrder: (searchParams.get('sortOrder') as 'asc' | 'desc') || 'desc',
  }));

  // Sync filters to URL params
  useEffect(() => {
    const params = new URLSearchParams();
    if (filters.search) params.set('search', filters.search);
    if (filters.map) params.set('map', filters.map);
    if (filters.sortBy !== 'createdAt') params.set('sortBy', filters.sortBy);
    if (page > 1) params.set('page', page.toString());
    setSearchParams(params, { replace: true });
  }, [filters, page, setSearchParams]);

  const fetchReplays = useCallback(async () => {
    try {
      setLoading(true);
      setError(undefined);

      const params = new URLSearchParams({
        page: page.toString(),
        limit: '12',
        sortBy: filters.sortBy,
        sortOrder: filters.sortOrder,
      });

      if (filters.search) params.set('search', filters.search);
      if (filters.map) params.set('map', filters.map);

      const data = await api.get<ReplaysResponse>(`/replays?${params}`);
      setReplays(data.replays);
      setTotalPages(data.totalPages);
      setTotal(data.total);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load replays');
    } finally {
      setLoading(false);
    }
  }, [page, filters]);

  useEffect(() => {
    fetchReplays();
  }, [fetchReplays]);

  // Reset to page 1 when filters change (debounced for search)
  useEffect(() => {
    const timer = setTimeout(() => {
      setPage(1);
    }, 300);
    return () => clearTimeout(timer);
  }, [filters.search, filters.map, filters.sortBy]);

  const handleFiltersChange = (newFilters: FilterState) => {
    setFilters(newFilters);
  };

  const handleResetFilters = () => {
    setFilters(DEFAULT_FILTERS);
    setPage(1);
  };

  const hasActiveFilters = filters.search || filters.map || filters.sortBy !== 'createdAt';

  const handleRefresh = () => {
    fetchReplays();
  };

  // Build dynamic description based on filters
  const seoDescription = filters.search
    ? `Browse Rocket League replays matching "${filters.search}". ${total} replays found.`
    : filters.map
      ? `Browse Rocket League replays played on ${filters.map}. ${total} replays available.`
      : `Browse ${total} Rocket League replays. Watch and analyze matches in 3D.`;

  return (
    <div className="space-y-4 sm:space-y-6 md:space-y-8">
      <SEOHead
        title={filters.map ? `Replays on ${filters.map}` : 'Browse Replays'}
        description={seoDescription}
      />
      {/* Hero Header */}
      <div className="relative overflow-hidden rounded-xl sm:rounded-2xl bg-gradient-to-br from-violet-900/20 to-blue-900/20 border border-violet-500/20 p-4 sm:p-6 md:p-8">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-violet-600/10 via-transparent to-transparent" />

        <div className="relative flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 sm:gap-6">
          <div className="flex items-center gap-3 sm:gap-4 min-w-0">
            <div className="w-11 h-11 sm:w-14 sm:h-14 rounded-xl bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center flex-shrink-0">
              <Play className="w-5 h-5 sm:w-7 sm:h-7 text-white" />
            </div>
            <div className="min-w-0">
              <h1 className="text-2xl sm:text-3xl font-bold bg-gradient-to-r from-violet-400 via-blue-400 to-cyan-400 bg-clip-text text-transparent">
                Replays
              </h1>
              <p className="text-gray-400 text-sm sm:text-base mt-0.5 sm:mt-1">
                {total} replay{total !== 1 ? 's' : ''} available
              </p>
            </div>
          </div>

          <div className="flex gap-2 sm:gap-3">
            <button
              onClick={handleRefresh}
              disabled={loading}
              className="p-2.5 sm:p-3 rounded-lg sm:rounded-xl bg-gray-800/80 text-gray-400 hover:bg-gray-700 hover:text-white disabled:opacity-50 transition-all border border-gray-700 min-h-[44px] min-w-[44px] flex items-center justify-center"
              title="Refresh"
            >
              <RefreshCw className={`w-5 h-5 ${loading ? 'animate-spin' : ''}`} />
            </button>
            <Link to="/upload" className="flex-1 sm:flex-none">
              <GradientButton size="lg" className="w-full sm:w-auto min-h-[44px]">
                <Upload className="w-5 h-5" />
                <span className="hidden xs:inline">Upload</span>
              </GradientButton>
            </Link>
          </div>
        </div>
      </div>

      {/* Search and Filters */}
      <SearchFilters
        filters={filters}
        onFiltersChange={handleFiltersChange}
        onReset={handleResetFilters}
      />

      {/* Info Banners */}
      <div className="space-y-3 sm:space-y-4">
        <QualityInfoBanner />
      </div>

      {/* Content */}
      {loading && replays.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20">
          <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-violet-600/20 to-blue-600/20 flex items-center justify-center mb-4">
            <Loader2 className="w-8 h-8 animate-spin text-violet-500" />
          </div>
          <p className="text-gray-400">Loading replays...</p>
        </div>
      ) : error ? (
        <div className="flex flex-col items-center justify-center py-20">
          <div className="w-16 h-16 rounded-2xl bg-red-500/10 flex items-center justify-center mb-4">
            <FolderOpen className="w-8 h-8 text-red-400" />
          </div>
          <p className="text-red-400 mb-4">{error}</p>
          <button
            onClick={handleRefresh}
            className="text-violet-400 hover:text-violet-300 underline"
          >
            Retry
          </button>
        </div>
      ) : replays.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20">
          <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-violet-600/20 to-blue-600/20 flex items-center justify-center mb-6">
            <FolderOpen className="w-10 h-10 text-violet-400" />
          </div>
          <h3 className="text-xl font-semibold text-white mb-2">
            {hasActiveFilters ? 'No results found' : 'No replays yet'}
          </h3>
          <p className="text-gray-400 mb-6 text-center max-w-md">
            {hasActiveFilters
              ? 'Try adjusting your search or filters to find what you\'re looking for.'
              : 'Upload your first replay to start analyzing your games.'}
          </p>
          {hasActiveFilters ? (
            <button
              onClick={handleResetFilters}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-violet-500/20 text-violet-300 hover:bg-violet-500/30 transition-all border border-violet-500/30"
            >
              <RotateCcw className="w-4 h-4" />
              Reset filters
            </button>
          ) : (
            <Link to="/upload">
              <GradientButton size="lg">
                <Upload className="w-5 h-5" />
                Upload a replay
              </GradientButton>
            </Link>
          )}
        </div>
      ) : (
        <>
          {/* Grid */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 sm:gap-5 min-w-0">
            {replays.map((replay) => (
              <ReplayCard
                key={replay.id}
                replay={replay}
                currentUserId={user?.id}
              />
            ))}
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex justify-center pt-4">
              <Pagination
                page={page}
                totalPages={totalPages}
                onPageChange={setPage}
              />
            </div>
          )}
        </>
      )}
    </div>
  );
}
