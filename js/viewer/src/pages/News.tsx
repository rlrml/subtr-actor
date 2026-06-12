import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { Newspaper, Eye, MessageCircle, Clock, Loader2, RefreshCw } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { SEOHead } from '@/components/SEO';
import {
  listAnnouncements,
  type Announcement,
} from '@/api/announcements.api';

const PAGE_SIZE = 10;

function AnnouncementCard({ announcement }: { announcement: Announcement }) {
  const date = announcement.publishedAt ?? announcement.createdAt;
  return (
    <Link
      to={`/news/${announcement.slug}`}
      className="group relative block rounded-xl border border-gray-800 bg-gray-900/50 backdrop-blur-sm p-5 sm:p-6 transition-all hover:border-violet-500/50 hover:bg-gray-900/70"
    >
      {/* Subtle gradient overlay on hover */}
      <div className="absolute inset-0 rounded-xl bg-gradient-to-br from-violet-600/5 via-transparent to-blue-600/5 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />

      <div className="relative space-y-3">
        <div className="flex items-center gap-2 text-xs text-gray-500">
          <Clock className="w-3.5 h-3.5" />
          <time dateTime={date}>
            {formatDistanceToNow(new Date(date), { addSuffix: true })}
          </time>
          {announcement.author && (
            <>
              <span className="text-gray-700">•</span>
              <span className="truncate">by {announcement.author.username}</span>
            </>
          )}
        </div>

        <h2 className="text-xl sm:text-2xl font-bold text-white group-hover:text-violet-300 transition-colors line-clamp-2">
          {announcement.title}
        </h2>

        {announcement.excerpt && (
          <p className="text-gray-400 text-sm sm:text-base line-clamp-3 leading-relaxed">
            {announcement.excerpt}
          </p>
        )}

        <div className="flex items-center gap-4 text-xs text-gray-500 pt-2">
          <span className="flex items-center gap-1.5">
            <Eye className="w-3.5 h-3.5" />
            {announcement.viewCount.toLocaleString()}
          </span>
          <span className="flex items-center gap-1.5">
            <MessageCircle className="w-3.5 h-3.5" />
            {announcement.commentCount.toLocaleString()}
          </span>
          <span className="ml-auto text-violet-400 group-hover:text-violet-300 transition-colors">
            Read more →
          </span>
        </div>
      </div>
    </Link>
  );
}

function CardSkeleton() {
  return (
    <div className="rounded-xl border border-gray-800 bg-gray-900/50 p-5 sm:p-6 animate-pulse space-y-3">
      <div className="h-3 bg-gray-700/50 rounded w-32" />
      <div className="h-6 bg-gray-700/50 rounded w-3/4" />
      <div className="h-4 bg-gray-700/50 rounded w-full" />
      <div className="h-4 bg-gray-700/50 rounded w-5/6" />
    </div>
  );
}

export default function News() {
  const [items, setItems] = useState<Announcement[]>([]);
  const [total, setTotal] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPage = useCallback(async (offset: number, append: boolean) => {
    try {
      if (append) setLoadingMore(true);
      else setLoading(true);
      setError(null);

      const response = await listAnnouncements({ limit: PAGE_SIZE, offset });

      setItems((prev) => (append ? [...prev, ...response.items] : response.items));
      setTotal(response.total);
      setHasMore(response.hasMore);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load news');
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  }, []);

  useEffect(() => {
    fetchPage(0, false);
  }, [fetchPage]);

  const handleLoadMore = () => {
    if (!loadingMore && hasMore) {
      fetchPage(items.length, true);
    }
  };

  const handleRefresh = () => {
    fetchPage(0, false);
  };

  return (
    <div className="space-y-6 sm:space-y-8">
      <SEOHead
        title="News"
        description={`Latest BallCam announcements and updates. ${total} stories.`}
      />

      {/* Hero Header */}
      <div className="relative overflow-hidden rounded-xl sm:rounded-2xl bg-gradient-to-br from-violet-900/20 to-blue-900/20 border border-violet-500/20 p-4 sm:p-6 md:p-8">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-violet-600/10 via-transparent to-transparent" />

        <div className="relative flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 sm:gap-6">
          <div className="flex items-center gap-3 sm:gap-4 min-w-0">
            <div className="w-11 h-11 sm:w-14 sm:h-14 rounded-xl bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center flex-shrink-0">
              <Newspaper className="w-5 h-5 sm:w-7 sm:h-7 text-white" />
            </div>
            <div className="min-w-0">
              <h1 className="text-2xl sm:text-3xl font-bold bg-gradient-to-r from-violet-400 via-blue-400 to-cyan-400 bg-clip-text text-transparent">
                News
              </h1>
              <p className="text-gray-400 text-sm sm:text-base mt-0.5 sm:mt-1">
                {total} {total === 1 ? 'story' : 'stories'}
              </p>
            </div>
          </div>

          <button
            onClick={handleRefresh}
            disabled={loading}
            className="self-start sm:self-auto p-2.5 sm:p-3 rounded-lg sm:rounded-xl bg-gray-800/80 text-gray-400 hover:bg-gray-700 hover:text-white disabled:opacity-50 transition-all border border-gray-700 min-h-[44px] min-w-[44px] flex items-center justify-center"
            title="Refresh"
          >
            <RefreshCw className={`w-5 h-5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Content */}
      {loading && items.length === 0 ? (
        <div className="grid gap-4 sm:gap-5">
          <CardSkeleton />
          <CardSkeleton />
          <CardSkeleton />
        </div>
      ) : error ? (
        <div className="flex flex-col items-center justify-center py-20">
          <p className="text-red-400 mb-4">{error}</p>
          <button
            onClick={handleRefresh}
            className="text-violet-400 hover:text-violet-300 underline"
          >
            Retry
          </button>
        </div>
      ) : items.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20">
          <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-violet-600/20 to-blue-600/20 flex items-center justify-center mb-6">
            <Newspaper className="w-10 h-10 text-violet-400" />
          </div>
          <h3 className="text-xl font-semibold text-white mb-2">No news yet</h3>
          <p className="text-gray-400 text-center max-w-md">
            Stay tuned — announcements and updates will appear here.
          </p>
        </div>
      ) : (
        <>
          <div className="grid gap-4 sm:gap-5">
            {items.map((announcement) => (
              <AnnouncementCard key={announcement.id} announcement={announcement} />
            ))}
          </div>

          {hasMore && (
            <div className="flex justify-center pt-2">
              <button
                onClick={handleLoadMore}
                disabled={loadingMore}
                className="flex items-center gap-2 px-5 py-2.5 text-sm font-medium text-gray-300 bg-gray-800/50 hover:bg-gray-800 border border-gray-700 hover:border-violet-500/40 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loadingMore ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin" />
                    Loading...
                  </>
                ) : (
                  <>Load more ({total - items.length} remaining)</>
                )}
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
