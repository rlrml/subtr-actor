import { useState, useEffect } from 'react';
import { Link, useParams } from 'react-router-dom';
import {
  Newspaper,
  Eye,
  ArrowLeft,
  Loader2,
  AlertTriangle,
  Calendar,
} from 'lucide-react';
import { format, formatDistanceToNow } from 'date-fns';
import { SEOHead } from '@/components/SEO';
import { CommentList } from '@/components/comments/CommentList';
import { MarkdownContent } from '@/components/MarkdownContent';
import {
  getAnnouncementBySlug,
  type Announcement,
} from '@/api/announcements.api';

export default function NewsDetail() {
  const { slug } = useParams<{ slug: string }>();
  const [announcement, setAnnouncement] = useState<Announcement | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notFound, setNotFound] = useState(false);

  useEffect(() => {
    if (!slug) return;

    let cancelled = false;
    setLoading(true);
    setError(null);
    setNotFound(false);

    getAnnouncementBySlug(slug)
      .then((data) => {
        if (!cancelled) {
          setAnnouncement(data);
          setLoading(false);
        }
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        const message = err instanceof Error ? err.message : 'Failed to load article';
        // Backend returns 404 with the standard error envelope
        const isNotFound =
          (err as { error?: string })?.error === 'Not Found' ||
          /not found/i.test(message);
        if (isNotFound) {
          setNotFound(true);
        } else {
          setError(message);
        }
        setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [slug]);

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <Loader2 className="w-10 h-10 text-violet-500 animate-spin mb-4" />
        <p className="text-gray-400">Loading article...</p>
      </div>
    );
  }

  if (notFound) {
    return (
      <div className="max-w-2xl mx-auto py-12 sm:py-20 text-center">
        <SEOHead title="Article not found" description="The article you're looking for does not exist." noIndex />
        <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-violet-600/20 to-blue-600/20 flex items-center justify-center mx-auto mb-6">
          <Newspaper className="w-10 h-10 text-violet-400" />
        </div>
        <h1 className="text-2xl font-bold text-white mb-2">Article not found</h1>
        <p className="text-gray-400 mb-8">
          The article you're looking for doesn't exist or has been unpublished.
        </p>
        <Link
          to="/news"
          className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg bg-gradient-to-r from-violet-600 to-blue-600 text-white font-medium hover:from-violet-500 hover:to-blue-500 transition-all"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to News
        </Link>
      </div>
    );
  }

  if (error || !announcement) {
    return (
      <div className="max-w-2xl mx-auto py-12 sm:py-20 text-center">
        <div className="w-20 h-20 rounded-2xl bg-red-500/10 flex items-center justify-center mx-auto mb-6">
          <AlertTriangle className="w-10 h-10 text-red-400" />
        </div>
        <h1 className="text-2xl font-bold text-white mb-2">Failed to load</h1>
        <p className="text-red-400 mb-8">{error ?? 'Unknown error'}</p>
        <Link
          to="/news"
          className="text-violet-400 hover:text-violet-300 underline"
        >
          Back to News
        </Link>
      </div>
    );
  }

  const date = announcement.publishedAt ?? announcement.createdAt;
  const seoDescription =
    announcement.excerpt ??
    announcement.contentMd.slice(0, 200).replace(/\s+/g, ' ');

  return (
    <article className="max-w-3xl mx-auto space-y-6 sm:space-y-8">
      <SEOHead
        title={announcement.title}
        description={seoDescription}
        type="article"
      />

      {/* Back link */}
      <div>
        <Link
          to="/news"
          className="inline-flex items-center gap-2 text-sm text-gray-400 hover:text-violet-300 transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to News
        </Link>
      </div>

      {/* Header */}
      <header className="space-y-4 pb-6 border-b border-gray-800">
        {!announcement.isPublished && (
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-amber-500/10 border border-amber-500/30 text-amber-300 text-xs font-medium">
            <AlertTriangle className="w-3 h-3" />
            Draft preview (admin only)
          </div>
        )}

        <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold text-white leading-tight">
          {announcement.title}
        </h1>

        <div className="flex flex-wrap items-center gap-x-5 gap-y-2 text-sm text-gray-400">
          {announcement.author && (
            <div className="flex items-center gap-2">
              {announcement.author.avatarUrl ? (
                <img
                  src={announcement.author.avatarUrl}
                  alt={announcement.author.username}
                  className="w-7 h-7 rounded-full border border-gray-800"
                />
              ) : (
                <div className="w-7 h-7 rounded-full bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center text-xs font-bold text-white">
                  {announcement.author.username.charAt(0).toUpperCase()}
                </div>
              )}
              <span className="text-gray-300">{announcement.author.username}</span>
            </div>
          )}

          <span className="flex items-center gap-1.5">
            <Calendar className="w-3.5 h-3.5" />
            <time dateTime={date} title={format(new Date(date), 'PPpp')}>
              {formatDistanceToNow(new Date(date), { addSuffix: true })}
            </time>
          </span>

          <span className="flex items-center gap-1.5">
            <Eye className="w-3.5 h-3.5" />
            {announcement.viewCount.toLocaleString()} views
          </span>
        </div>
      </header>

      {/* Body */}
      <div className="rounded-xl border border-gray-800 bg-gray-900/40 backdrop-blur-sm p-5 sm:p-8">
        <MarkdownContent content={announcement.contentMd} />
      </div>

      {/* Comments */}
      <section className="rounded-xl border border-gray-800 bg-gray-900/40 backdrop-blur-sm p-5 sm:p-8">
        <CommentList entityType="announcement" entityId={announcement.id} />
      </section>
    </article>
  );
}
