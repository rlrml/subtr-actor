/**
 * Clips Page
 *
 * List user's clips with pagination, sorting, and filtering.
 *
 * Feature: 024-clip-system (US3 - Management)
 */

import { useState, useEffect, useCallback } from 'react';
import { Link, useSearchParams, useNavigate } from 'react-router-dom';
import { Scissors, Plus, Loader2, SortAsc, ChevronLeft, ChevronRight } from 'lucide-react';
import { ClipCard } from '@/components/clips/ClipCard';
import * as clipsApi from '@/api/clips';
import type { ClipListItem, ListClipsParams } from '@/api/clips';
import { useAuth } from '@/hooks/useAuth';
import { cn } from '@/lib/utils';

type SortBy = 'createdAt' | 'viewCount' | 'likeCount';
type SortOrder = 'asc' | 'desc';

export default function Clips() {
  const { user, isLoading: authLoading } = useAuth();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  // State
  const [clips, setClips] = useState<ClipListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(() => parseInt(searchParams.get('page') || '1'));
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const [sortBy, setSortBy] = useState<SortBy>((searchParams.get('sortBy') as SortBy) || 'createdAt');
  const [sortOrder, setSortOrder] = useState<SortOrder>((searchParams.get('sortOrder') as SortOrder) || 'desc');

  const ITEMS_PER_PAGE = 12;

  // Redirect if not logged in
  useEffect(() => {
    if (!authLoading && !user) {
      navigate('/login', { state: { from: '/my-clips' } });
    }
  }, [user, authLoading, navigate]);

  // Fetch clips
  const fetchClips = useCallback(async () => {
    if (!user) return;

    setLoading(true);
    setError(null);

    try {
      const params: ListClipsParams = {
        page,
        limit: ITEMS_PER_PAGE,
        sortBy,
        sortOrder,
      };

      const response = await clipsApi.listMyClips(params);
      setClips(response.clips);
      setTotalPages(response.totalPages);
      setTotal(response.total);
    } catch (err) {
      console.error('[Clips] Failed to fetch clips:', err);
      setError('Failed to load clips. Please try again.');
    } finally {
      setLoading(false);
    }
  }, [user, page, sortBy, sortOrder]);

  useEffect(() => {
    fetchClips();
  }, [fetchClips]);

  // Update URL params when state changes
  useEffect(() => {
    const params = new URLSearchParams();
    if (page > 1) params.set('page', page.toString());
    if (sortBy !== 'createdAt') params.set('sortBy', sortBy);
    if (sortOrder !== 'desc') params.set('sortOrder', sortOrder);
    setSearchParams(params, { replace: true });
  }, [page, sortBy, sortOrder, setSearchParams]);

  // Handlers
  const handlePageChange = (newPage: number) => {
    setPage(newPage);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  const handleSortChange = (newSortBy: SortBy) => {
    if (newSortBy === sortBy) {
      setSortOrder(sortOrder === 'desc' ? 'asc' : 'desc');
    } else {
      setSortBy(newSortBy);
      setSortOrder('desc');
    }
    setPage(1);
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this clip?')) return;

    try {
      await clipsApi.deleteClip(id);
      // Refresh list
      fetchClips();
    } catch (err) {
      console.error('[Clips] Failed to delete clip:', err);
      alert('Failed to delete clip. Please try again.');
    }
  };

  const handleEdit = (id: string) => {
    navigate(`/clips/${id}/edit`);
  };

  if (authLoading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
      </div>
    );
  }

  if (!user) {
    return null; // Will redirect
  }

  return (
    <div className="max-w-7xl mx-auto px-4 py-8">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-8">
          <div className="flex items-center gap-3">
            <Scissors className="w-8 h-8 text-blue-500" />
            <div>
              <h1 className="text-2xl font-bold text-white">My Clips</h1>
              <p className="text-sm text-zinc-400">{total} clip{total !== 1 ? 's' : ''}</p>
            </div>
          </div>

          <Link
            to="/replays"
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors"
          >
            <Plus className="w-4 h-4" />
            <span>Create New Clip</span>
          </Link>
        </div>

        {/* Sort Controls */}
        <div className="flex items-center gap-4 mb-6">
          <span className="text-sm text-zinc-400">Sort by:</span>
          <div className="flex items-center gap-2">
            {[
              { key: 'createdAt' as SortBy, label: 'Date' },
              { key: 'viewCount' as SortBy, label: 'Views' },
              { key: 'likeCount' as SortBy, label: 'Likes' },
            ].map(({ key, label }) => (
              <button
                key={key}
                onClick={() => handleSortChange(key)}
                className={cn(
                  "flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors",
                  sortBy === key
                    ? "bg-blue-500/20 text-blue-400 border border-blue-500/50"
                    : "text-zinc-400 hover:text-white hover:bg-zinc-700"
                )}
              >
                {label}
                {sortBy === key && (
                  <SortAsc className={cn(
                    "w-3 h-3 transition-transform",
                    sortOrder === 'desc' && "rotate-180"
                  )} />
                )}
              </button>
            ))}
          </div>
        </div>

        {/* Content */}
        {loading ? (
          <div className="flex items-center justify-center py-20">
            <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
          </div>
        ) : error ? (
          <div className="text-center py-20">
            <p className="text-red-400 mb-4">{error}</p>
            <button
              onClick={fetchClips}
              className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-white rounded-lg transition-colors"
            >
              Try Again
            </button>
          </div>
        ) : clips.length === 0 ? (
          <div className="text-center py-20">
            <Scissors className="w-16 h-16 text-zinc-600 mx-auto mb-4" />
            <h2 className="text-xl font-semibold text-white mb-2">No clips yet</h2>
            <p className="text-zinc-400 mb-6">
              Create your first clip from any replay!
            </p>
            <Link
              to="/replays"
              className="inline-flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors"
            >
              <Plus className="w-5 h-5" />
              <span>Browse Replays</span>
            </Link>
          </div>
        ) : (
          <>
            {/* Clips Grid */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
              {clips.map((clip) => (
                <ClipCard
                  key={clip.id}
                  clip={clip}
                  showActions
                  onDelete={handleDelete}
                  onEdit={handleEdit}
                />
              ))}
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
              <div className="flex items-center justify-center gap-2 mt-8">
                <button
                  onClick={() => handlePageChange(page - 1)}
                  disabled={page <= 1}
                  className={cn(
                    "p-2 rounded-lg transition-colors",
                    page <= 1
                      ? "text-zinc-600 cursor-not-allowed"
                      : "text-zinc-400 hover:text-white hover:bg-zinc-700"
                  )}
                >
                  <ChevronLeft className="w-5 h-5" />
                </button>

                <div className="flex items-center gap-1">
                  {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                    let pageNum: number;
                    if (totalPages <= 5) {
                      pageNum = i + 1;
                    } else if (page <= 3) {
                      pageNum = i + 1;
                    } else if (page >= totalPages - 2) {
                      pageNum = totalPages - 4 + i;
                    } else {
                      pageNum = page - 2 + i;
                    }

                    return (
                      <button
                        key={pageNum}
                        onClick={() => handlePageChange(pageNum)}
                        className={cn(
                          "w-10 h-10 rounded-lg font-medium transition-colors",
                          page === pageNum
                            ? "bg-blue-600 text-white"
                            : "text-zinc-400 hover:text-white hover:bg-zinc-700"
                        )}
                      >
                        {pageNum}
                      </button>
                    );
                  })}
                </div>

                <button
                  onClick={() => handlePageChange(page + 1)}
                  disabled={page >= totalPages}
                  className={cn(
                    "p-2 rounded-lg transition-colors",
                    page >= totalPages
                      ? "text-zinc-600 cursor-not-allowed"
                      : "text-zinc-400 hover:text-white hover:bg-zinc-700"
                  )}
                >
                  <ChevronRight className="w-5 h-5" />
                </button>
              </div>
            )}
          </>
        )}
    </div>
  );
}
