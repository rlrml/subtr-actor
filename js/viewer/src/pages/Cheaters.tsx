/**
 * Cheaters Page - List of detected cheaters with search and pagination
 * (032-cheat-detection)
 */

import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import {
  Shield,
  Search,
  AlertTriangle,
  ChevronLeft,
  ChevronRight,
  Loader2,
  ExternalLink,
  Calendar,
} from 'lucide-react';
import { GradientCard } from '@/components/ui/GradientCard';
import { useCheatersList, useCheatStats, CHEAT_ATTRIBUTION, formatPlatformName, getConfidenceLevel } from '@/api/cheat';
import type { CheaterSummary } from '@/types/cheat';
import { cn } from '@/lib/utils';

export default function Cheaters() {
  const [search, setSearch] = useState('');
  const [debouncedSearch, setDebouncedSearch] = useState('');
  const [page, setPage] = useState(1);
  const limit = 20;

  // Debounce search input
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearch(search);
      setPage(1); // Reset to first page on new search
    }, 300);

    return () => clearTimeout(timer);
  }, [search]);

  // Fetch cheaters list with pagination and search
  const { data, loading, error } = useCheatersList({
    page,
    limit,
    search: debouncedSearch || undefined,
  });

  // Fetch global stats for header
  const { data: stats } = useCheatStats();

  const handlePreviousPage = useCallback(() => {
    setPage((p) => Math.max(1, p - 1));
  }, []);

  const handleNextPage = useCallback(() => {
    if (data && page < data.totalPages) {
      setPage((p) => p + 1);
    }
  }, [data, page]);

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    });
  };

  return (
    <div className="space-y-6">
      <SEOHead
        title="Cheaters Detected - BallCam"
        description="Browse the list of players detected using unauthorized modifications in Rocket League replays. Powered by whosbotting.com."
        noIndex
      />

      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-white flex items-center gap-3">
            <Shield className="w-8 h-8 text-red-400" />
            Cheaters Detected
          </h1>
          <p className="text-gray-400 mt-1">
            Players flagged for using unauthorized modifications
          </p>
        </div>

        {/* Stats summary */}
        {stats && (
          <div className="flex gap-4">
            <div className="px-4 py-2 rounded-lg bg-red-500/10 border border-red-500/20">
              <div className="text-xl font-bold text-red-400">{stats.totalCheatersDetected}</div>
              <div className="text-xs text-gray-500">Total Cheaters</div>
            </div>
            <div className="px-4 py-2 rounded-lg bg-amber-500/10 border border-amber-500/20">
              <div className="text-xl font-bold text-amber-400">{stats.totalReplaysAnalyzed}</div>
              <div className="text-xs text-gray-500">Replays Analyzed</div>
            </div>
            <div className="px-4 py-2 rounded-lg bg-gray-800 border border-gray-700">
              <div className="text-xl font-bold text-gray-300">
                {stats.cheaterPercentage.toFixed(1)}%
              </div>
              <div className="text-xs text-gray-500">Detection Rate</div>
            </div>
          </div>
        )}
      </div>

      {/* Search bar */}
      <div className="relative">
        <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search by player name..."
          className="w-full pl-12 pr-4 py-3 rounded-xl bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-red-500/50 focus:border-red-500/50 transition-all"
        />
      </div>

      {/* Loading state */}
      {loading && (
        <div className="flex flex-col items-center justify-center py-16">
          <Loader2 className="w-8 h-8 text-red-400 animate-spin mb-4" />
          <p className="text-gray-400">Loading cheaters list...</p>
        </div>
      )}

      {/* Error state */}
      {error && (
        <div className="flex flex-col items-center justify-center py-16">
          <AlertTriangle className="w-8 h-8 text-red-400 mb-4" />
          <p className="text-red-400">{error.message || 'Failed to load cheaters'}</p>
        </div>
      )}

      {/* Results */}
      {!loading && !error && data && (
        <>
          {/* Results count */}
          <div className="flex items-center justify-between text-sm text-gray-400">
            <span>
              Showing {data.cheaters.length} of {data.total} cheaters
              {debouncedSearch && ` matching "${debouncedSearch}"`}
            </span>
            {data.total > limit && (
              <span>
                Page {page} of {data.totalPages}
              </span>
            )}
          </div>

          {/* Empty state */}
          {data.cheaters.length === 0 && (
            <div className="text-center py-16">
              <Shield className="w-16 h-16 text-gray-700 mx-auto mb-4" />
              <h2 className="text-xl font-semibold text-gray-400 mb-2">
                {debouncedSearch ? 'No cheaters found' : 'No cheaters detected yet'}
              </h2>
              <p className="text-gray-500">
                {debouncedSearch
                  ? 'Try adjusting your search terms'
                  : 'Upload replays to start detecting cheaters'}
              </p>
            </div>
          )}

          {/* Cheaters list */}
          {data.cheaters.length > 0 && (
            <div className="space-y-3">
              {data.cheaters.map((cheater: CheaterSummary) => {
                const { colorClass } = getConfidenceLevel(cheater.highestConfidence);
                return (
                  <Link
                    key={cheater.id}
                    to={`/players/${cheater.id}`}
                    className="block group"
                  >
                    <GradientCard className="border-red-500/20 hover:border-red-500/40 transition-colors">
                      <div className="flex items-center justify-between gap-4">
                        {/* Player info */}
                        <div className="flex items-center gap-4 min-w-0">
                          <div className="w-12 h-12 rounded-xl bg-red-500/20 flex items-center justify-center shrink-0">
                            <AlertTriangle className="w-6 h-6 text-red-400" />
                          </div>
                          <div className="min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="font-semibold text-white truncate group-hover:text-red-300 transition-colors">
                                {cheater.name}
                              </span>
                              <span className="text-xs text-gray-500 bg-gray-800 px-2 py-0.5 rounded shrink-0">
                                {formatPlatformName(cheater.platform)}
                              </span>
                            </div>
                            <div className="flex items-center gap-3 mt-1 text-xs text-gray-500">
                              <span className="flex items-center gap-1">
                                <Calendar className="w-3 h-3" />
                                First: {formatDate(cheater.firstFlaggedAt)}
                              </span>
                              {cheater.lastFlaggedAt !== cheater.firstFlaggedAt && (
                                <span>Last: {formatDate(cheater.lastFlaggedAt)}</span>
                              )}
                            </div>
                          </div>
                        </div>

                        {/* Stats */}
                        <div className="flex items-center gap-4 shrink-0">
                          <div className="text-center">
                            <div className="text-lg font-bold text-red-400">
                              {cheater.flaggedReplayCount}
                            </div>
                            <div className="text-xs text-gray-500">Detections</div>
                          </div>
                          <div className="text-center">
                            <div className={cn('text-lg font-bold', colorClass)}>
                              {cheater.highestConfidence.toFixed(0)}%
                            </div>
                            <div className="text-xs text-gray-500">Confidence</div>
                          </div>
                        </div>
                      </div>
                    </GradientCard>
                  </Link>
                );
              })}
            </div>
          )}

          {/* Pagination */}
          {data.totalPages > 1 && (
            <div className="flex items-center justify-center gap-4 pt-4">
              <button
                onClick={handlePreviousPage}
                disabled={page === 1}
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                <ChevronLeft className="w-4 h-4" />
                Previous
              </button>

              <div className="flex items-center gap-2">
                {/* Page indicators */}
                {Array.from({ length: Math.min(5, data.totalPages) }, (_, i) => {
                  let pageNum;
                  if (data.totalPages <= 5) {
                    pageNum = i + 1;
                  } else if (page <= 3) {
                    pageNum = i + 1;
                  } else if (page >= data.totalPages - 2) {
                    pageNum = data.totalPages - 4 + i;
                  } else {
                    pageNum = page - 2 + i;
                  }
                  return (
                    <button
                      key={pageNum}
                      onClick={() => setPage(pageNum)}
                      className={cn(
                        'w-10 h-10 rounded-lg text-sm font-medium transition-colors',
                        pageNum === page
                          ? 'bg-red-500/20 border border-red-500/30 text-red-400'
                          : 'bg-gray-800 border border-gray-700 text-gray-400 hover:bg-gray-700'
                      )}
                    >
                      {pageNum}
                    </button>
                  );
                })}
              </div>

              <button
                onClick={handleNextPage}
                disabled={page >= data.totalPages}
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                Next
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          )}
        </>
      )}

      {/* Attribution footer */}
      <div className="text-center pt-8 border-t border-gray-800">
        <p className="text-sm text-gray-500">
          Cheat detection powered by{' '}
          <a
            href={CHEAT_ATTRIBUTION.url}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-gray-400 hover:text-white transition-colors"
          >
            {CHEAT_ATTRIBUTION.service}
            <ExternalLink className="w-3 h-3" />
          </a>
        </p>
        <p className="text-xs text-gray-600 mt-2">
          Detection uses a {'>'}50% confidence threshold as recommended by whosbotting.com
        </p>
      </div>
    </div>
  );
}
