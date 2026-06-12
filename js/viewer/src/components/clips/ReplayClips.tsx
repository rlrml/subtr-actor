/**
 * ReplayClips Component
 *
 * Displays clips created from a specific replay.
 * Shows a grid of clip cards with pagination.
 *
 * Feature: 024-clip-system (T074)
 */

import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Scissors, Plus, Loader2, ChevronLeft, ChevronRight, Film } from 'lucide-react';
import { ClipCard } from './ClipCard';
import * as clipsApi from '@/api/clips';
import type { ClipListItem } from '@/api/clips';
import { cn } from '@/lib/utils';

interface ReplayClipsProps {
  replayId: string;
  isReady?: boolean; // Can only create clips if replay is ready
}

export function ReplayClips({ replayId, isReady = false }: ReplayClipsProps) {
  const [clips, setClips] = useState<ClipListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);

  const ITEMS_PER_PAGE = 4;

  useEffect(() => {
    async function fetchClips() {
      setLoading(true);
      setError(null);

      try {
        const response = await clipsApi.listClipsByReplay(replayId, {
          page,
          limit: ITEMS_PER_PAGE,
          sortBy: 'createdAt',
          sortOrder: 'desc',
        });

        setClips(response.clips);
        setTotalPages(response.totalPages);
        setTotal(response.total);
      } catch (err) {
        console.error('[ReplayClips] Failed to fetch clips:', err);
        setError('Failed to load clips');
      } finally {
        setLoading(false);
      }
    }

    fetchClips();
  }, [replayId, page]);

  const handlePageChange = (newPage: number) => {
    setPage(newPage);
  };

  // Loading state
  if (loading) {
    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Scissors className="w-5 h-5 text-violet-400" />
            <h3 className="text-lg font-semibold text-white">Clips</h3>
          </div>
        </div>
        <div className="flex items-center justify-center py-8">
          <Loader2 className="w-6 h-6 animate-spin text-violet-400" />
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Scissors className="w-5 h-5 text-violet-400" />
            <h3 className="text-lg font-semibold text-white">Clips</h3>
          </div>
        </div>
        <div className="text-center py-8">
          <p className="text-red-400 text-sm">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Scissors className="w-5 h-5 text-violet-400" />
          <h3 className="text-lg font-semibold text-white">Clips</h3>
          {total > 0 && (
            <span className="text-sm text-zinc-500">({total})</span>
          )}
        </div>

        {isReady && (
          <Link
            to={`/viewer/${replayId}`}
            className="flex items-center gap-2 px-3 py-1.5 text-sm text-white rounded-lg font-medium transition-all bg-gradient-to-r from-violet-600 to-blue-600 hover:from-violet-500 hover:to-blue-500 shadow-lg shadow-violet-500/25 hover:shadow-violet-500/40"
          >
            <Plus className="w-4 h-4" />
            <span>Create Clip</span>
          </Link>
        )}
      </div>

      {/* Empty state */}
      {clips.length === 0 ? (
        <div className="text-center py-8 bg-zinc-800/30 rounded-xl border border-zinc-700/50">
          <Film className="w-10 h-10 text-zinc-600 mx-auto mb-3" />
          <p className="text-zinc-400 text-sm mb-4">
            No clips created from this replay yet.
          </p>
          {isReady && (
            <Link
              to={`/viewer/${replayId}`}
              className="inline-flex items-center gap-2 px-4 py-2 text-sm text-white rounded-lg font-medium transition-all bg-gradient-to-r from-violet-600 to-blue-600 hover:from-violet-500 hover:to-blue-500 shadow-lg shadow-violet-500/25 hover:shadow-violet-500/40"
            >
              <Scissors className="w-4 h-4" />
              <span>Create the first clip</span>
            </Link>
          )}
        </div>
      ) : (
        <>
          {/* Clips Grid */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {clips.map((clip) => (
              <ClipCard key={clip.id} clip={clip} />
            ))}
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-center gap-2 mt-4">
              <button
                onClick={() => handlePageChange(page - 1)}
                disabled={page <= 1}
                className={cn(
                  "p-1.5 rounded-lg transition-colors",
                  page <= 1
                    ? "text-zinc-600 cursor-not-allowed"
                    : "text-zinc-400 hover:text-white hover:bg-zinc-700"
                )}
              >
                <ChevronLeft className="w-4 h-4" />
              </button>

              <span className="text-sm text-zinc-400">
                {page} / {totalPages}
              </span>

              <button
                onClick={() => handlePageChange(page + 1)}
                disabled={page >= totalPages}
                className={cn(
                  "p-1.5 rounded-lg transition-colors",
                  page >= totalPages
                    ? "text-zinc-600 cursor-not-allowed"
                    : "text-zinc-400 hover:text-white hover:bg-zinc-700"
                )}
              >
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
