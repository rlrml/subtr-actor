import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { Newspaper, ArrowRight, MessageCircle, Eye } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { getLatestAnnouncements, type Announcement } from '@/api/announcements.api';

/**
 * Homepage block showing the 3 most recent published announcements.
 * Renders nothing if the list is empty (no awkward empty UI).
 */
export function LatestNews() {
  const [items, setItems] = useState<Announcement[] | null>(null);

  useEffect(() => {
    let cancelled = false;
    getLatestAnnouncements(3)
      .then((data) => {
        if (!cancelled) setItems(data);
      })
      .catch(() => {
        // Silent fail on home — just don't show the section.
        if (!cancelled) setItems([]);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Hide the section entirely while loading or if empty.
  if (!items || items.length === 0) return null;

  return (
    <section className="relative space-y-6">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-3">
          <div className="p-3 bg-violet-500/20 rounded-xl">
            <Newspaper className="w-7 h-7 text-violet-400" />
          </div>
          <div>
            <h2 className="text-3xl lg:text-4xl font-bold text-white">Latest News</h2>
            <p className="text-gray-400 text-sm mt-1">
              Updates, announcements, and stories from the BallCam team
            </p>
          </div>
        </div>
        <div className="flex-1 h-px bg-gradient-to-r from-violet-500/30 to-transparent" />
        <Link
          to="/news"
          className="hidden sm:inline-flex items-center gap-1.5 text-sm text-violet-400 hover:text-violet-300 transition-colors whitespace-nowrap"
        >
          View all
          <ArrowRight className="w-4 h-4" />
        </Link>
      </div>

      <div className="grid gap-4 sm:gap-5 md:grid-cols-3">
        {items.map((announcement) => {
          const date = announcement.publishedAt ?? announcement.createdAt;
          return (
            <Link
              key={announcement.id}
              to={`/news/${announcement.slug}`}
              className="group relative flex flex-col rounded-xl border border-gray-800 bg-gray-900/50 backdrop-blur-sm p-5 transition-all hover:border-violet-500/50 hover:bg-gray-900/70"
            >
              <div className="absolute inset-0 rounded-xl bg-gradient-to-br from-violet-600/5 via-transparent to-blue-600/5 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />

              <div className="relative flex flex-col flex-1 space-y-3">
                <time
                  dateTime={date}
                  className="text-xs text-gray-500"
                >
                  {formatDistanceToNow(new Date(date), { addSuffix: true })}
                </time>

                <h3 className="text-lg font-bold text-white group-hover:text-violet-300 transition-colors line-clamp-2">
                  {announcement.title}
                </h3>

                {announcement.excerpt && (
                  <p className="text-sm text-gray-400 line-clamp-3 flex-1">
                    {announcement.excerpt}
                  </p>
                )}

                <div className="flex items-center gap-3 text-xs text-gray-500 pt-2 mt-auto">
                  <span className="flex items-center gap-1">
                    <Eye className="w-3 h-3" />
                    {announcement.viewCount.toLocaleString()}
                  </span>
                  <span className="flex items-center gap-1">
                    <MessageCircle className="w-3 h-3" />
                    {announcement.commentCount.toLocaleString()}
                  </span>
                </div>
              </div>
            </Link>
          );
        })}
      </div>

      {/* Mobile "View all" CTA */}
      <div className="sm:hidden flex justify-center">
        <Link
          to="/news"
          className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg border border-violet-500/30 text-violet-400 hover:text-violet-300 hover:border-violet-500/60 hover:bg-violet-500/5 transition-all text-sm"
        >
          View all news
          <ArrowRight className="w-4 h-4" />
        </Link>
      </div>
    </section>
  );
}

export default LatestNews;
