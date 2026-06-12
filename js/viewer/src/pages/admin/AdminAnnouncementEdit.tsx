import { useEffect, useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import {
  ArrowLeft,
  Save,
  Send,
  Trash2,
  Loader2,
  AlertTriangle,
  ExternalLink,
  Eye,
  EyeOff,
} from 'lucide-react';
import { toast } from 'sonner';
import { GradientButton } from '@/components/ui/GradientButton';
import { MarkdownSplitPane } from '@/components/feedback/MarkdownSplitPane';
import {
  getAnnouncementByIdAdmin,
  createAnnouncement,
  updateAnnouncement,
  publishAnnouncement,
  unpublishAnnouncement,
  deleteAnnouncement,
  type Announcement,
} from '@/api/announcements.api';

const TITLE_MAX = 200;
const EXCERPT_MAX = 500;
const CONTENT_MAX = 50_000;

export default function AdminAnnouncementEdit() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const isEditMode = Boolean(id);

  const [loading, setLoading] = useState(isEditMode);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [togglingPublish, setTogglingPublish] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [announcement, setAnnouncement] = useState<Announcement | null>(null);
  const [title, setTitle] = useState('');
  const [excerpt, setExcerpt] = useState('');
  const [contentMd, setContentMd] = useState('');

  // Load existing announcement in edit mode
  useEffect(() => {
    if (!isEditMode || !id) return;
    let cancelled = false;
    setLoading(true);
    setError(null);
    getAnnouncementByIdAdmin(id)
      .then((data) => {
        if (cancelled) return;
        setAnnouncement(data);
        setTitle(data.title);
        setExcerpt(data.excerpt ?? '');
        setContentMd(data.contentMd);
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : 'Failed to load announcement');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [id, isEditMode]);

  const isPublished = announcement?.isPublished ?? false;

  // Form validation
  const validation = useMemo(() => {
    const trimmedTitle = title.trim();
    const trimmedContent = contentMd.trim();
    const trimmedExcerpt = excerpt.trim();
    if (!trimmedTitle) return 'Title is required';
    if (trimmedTitle.length > TITLE_MAX) return `Title cannot exceed ${TITLE_MAX} characters`;
    if (!trimmedContent) return 'Content cannot be empty';
    if (trimmedContent.length > CONTENT_MAX) return `Content cannot exceed ${CONTENT_MAX} characters`;
    if (trimmedExcerpt.length > EXCERPT_MAX) return `Excerpt cannot exceed ${EXCERPT_MAX} characters`;
    return null;
  }, [title, contentMd, excerpt]);

  const isValid = !validation;

  const handleSave = async (publish: boolean) => {
    if (!isValid) {
      toast.error(validation ?? 'Form invalid');
      return;
    }

    setSaving(true);
    try {
      const trimmedExcerpt = excerpt.trim();
      const payload = {
        title: title.trim(),
        contentMd: contentMd.trim(),
        excerpt: trimmedExcerpt ? trimmedExcerpt : undefined,
      };

      if (isEditMode && announcement) {
        // Update
        const updated = await updateAnnouncement(announcement.id, {
          ...payload,
          excerpt: trimmedExcerpt ? trimmedExcerpt : null,
        });
        // If user clicked "Save & publish" and it was a draft, also publish.
        if (publish && !updated.isPublished) {
          const published = await publishAnnouncement(updated.id);
          setAnnouncement(published);
          toast.success('Announcement published');
        } else {
          setAnnouncement(updated);
          toast.success('Announcement updated');
        }
      } else {
        // Create
        const created = await createAnnouncement({
          ...payload,
          isPublished: publish,
        });
        toast.success(publish ? 'Announcement published' : 'Draft saved');
        // Navigate to edit mode for the newly created item
        navigate(`/admin/announcements/${created.id}`, { replace: true });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Save failed';
      toast.error(message);
      setError(message);
    } finally {
      setSaving(false);
    }
  };

  const handleTogglePublish = async () => {
    if (!announcement) return;
    setTogglingPublish(true);
    try {
      const result = announcement.isPublished
        ? await unpublishAnnouncement(announcement.id)
        : await publishAnnouncement(announcement.id);
      setAnnouncement(result);
      toast.success(result.isPublished ? 'Announcement published' : 'Announcement unpublished');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Action failed');
    } finally {
      setTogglingPublish(false);
    }
  };

  const handleDelete = async () => {
    if (!announcement) return;
    const confirmed = window.confirm(
      `Delete "${announcement.title}"? This will also remove all comments. This cannot be undone.`,
    );
    if (!confirmed) return;
    setDeleting(true);
    try {
      await deleteAnnouncement(announcement.id);
      toast.success('Announcement deleted');
      navigate('/admin/announcements');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Delete failed');
      setDeleting(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="w-8 h-8 text-violet-500 animate-spin" />
      </div>
    );
  }

  if (isEditMode && error && !announcement) {
    return (
      <div className="max-w-xl mx-auto py-12 text-center space-y-4">
        <AlertTriangle className="w-12 h-12 text-red-400 mx-auto" />
        <h1 className="text-xl font-bold text-white">Failed to load announcement</h1>
        <p className="text-red-400">{error}</p>
        <Link to="/admin/announcements" className="text-violet-400 hover:text-violet-300 underline">
          Back to list
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col gap-4">
        <Link
          to="/admin/announcements"
          className="inline-flex items-center gap-2 text-sm text-gray-400 hover:text-violet-300 transition-colors w-fit"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to announcements
        </Link>

        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-white">
              {isEditMode ? 'Edit announcement' : 'New announcement'}
            </h1>
            {isEditMode && announcement && (
              <div className="flex items-center gap-3 text-sm text-gray-400 mt-1">
                <span className="text-gray-500">slug:</span>
                <code className="px-2 py-0.5 rounded bg-gray-800 text-violet-300 text-xs">
                  {announcement.slug}
                </code>
                {isPublished ? (
                  <span className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                    Published
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/30">
                    Draft
                  </span>
                )}
                {isPublished && (
                  <Link
                    to={`/news/${announcement.slug}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1 text-violet-400 hover:text-violet-300 text-xs"
                  >
                    View live
                    <ExternalLink className="w-3 h-3" />
                  </Link>
                )}
              </div>
            )}
          </div>

          {isEditMode && announcement && (
            <div className="flex items-center gap-2">
              <button
                onClick={handleTogglePublish}
                disabled={togglingPublish}
                className="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg text-sm font-medium border border-gray-700 bg-gray-800/50 text-gray-300 hover:text-white hover:bg-gray-800 transition-all disabled:opacity-50"
              >
                {togglingPublish ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : isPublished ? (
                  <EyeOff className="w-4 h-4" />
                ) : (
                  <Eye className="w-4 h-4" />
                )}
                {isPublished ? 'Unpublish' : 'Publish'}
              </button>
              <button
                onClick={handleDelete}
                disabled={deleting}
                className="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg text-sm font-medium border border-red-500/30 bg-red-500/10 text-red-300 hover:bg-red-500/20 transition-all disabled:opacity-50"
              >
                {deleting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Trash2 className="w-4 h-4" />}
                Delete
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Validation banner */}
      {validation && (
        <div className="px-4 py-2 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-300 text-sm flex items-center gap-2">
          <AlertTriangle className="w-4 h-4 flex-shrink-0" />
          {validation}
        </div>
      )}

      {/* Form */}
      <div className="space-y-5 rounded-xl border border-gray-800 bg-gray-900/40 backdrop-blur-sm p-5 sm:p-6">
        {/* Title */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Title <span className="text-red-400">*</span>
          </label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            maxLength={TITLE_MAX + 50}
            placeholder="A clear, concise headline"
            className="w-full px-4 py-2.5 bg-gray-800/50 border border-gray-700 rounded-lg
                       text-white placeholder-gray-500
                       focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500"
          />
          <div className="flex justify-end mt-1">
            <span className={`text-xs ${title.length > TITLE_MAX ? 'text-red-400' : 'text-gray-500'}`}>
              {title.length}/{TITLE_MAX}
            </span>
          </div>
        </div>

        {/* Excerpt */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Excerpt <span className="text-gray-500 text-xs font-normal">(optional, shown in list & cards)</span>
          </label>
          <textarea
            value={excerpt}
            onChange={(e) => setExcerpt(e.target.value)}
            maxLength={EXCERPT_MAX + 50}
            rows={3}
            placeholder="Short teaser for cards and previews. Plain text only."
            className="w-full px-4 py-2.5 bg-gray-800/50 border border-gray-700 rounded-lg
                       text-white placeholder-gray-500 resize-none
                       focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500"
          />
          <div className="flex justify-end mt-1">
            <span className={`text-xs ${excerpt.length > EXCERPT_MAX ? 'text-red-400' : 'text-gray-500'}`}>
              {excerpt.length}/{EXCERPT_MAX}
            </span>
          </div>
        </div>

        {/* Content (markdown split pane) */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Content <span className="text-red-400">*</span>{' '}
            <span className="text-gray-500 text-xs font-normal">(Markdown — live preview on the right)</span>
          </label>
          <div className="rounded-lg border border-gray-700 overflow-hidden bg-gray-900/30">
            <MarkdownSplitPane
              value={contentMd}
              onChange={setContentMd}
              placeholder="# My announcement&#10;&#10;Write your news content in **markdown**..."
              minHeight="400px"
            />
          </div>
          <div className="flex justify-end mt-1">
            <span className={`text-xs ${contentMd.length > CONTENT_MAX ? 'text-red-400' : 'text-gray-500'}`}>
              {contentMd.length.toLocaleString()}/{CONTENT_MAX.toLocaleString()}
            </span>
          </div>
        </div>
      </div>

      {/* Sticky actions */}
      <div className="sticky bottom-4 z-10 flex flex-wrap gap-3 justify-end p-4 rounded-xl border border-gray-800 bg-gray-900/80 backdrop-blur-md">
        <Link
          to="/admin/announcements"
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium border border-gray-700 bg-gray-800/50 text-gray-300 hover:text-white hover:bg-gray-800 transition-all"
        >
          Cancel
        </Link>

        <GradientButton
          variant="secondary"
          onClick={() => handleSave(false)}
          disabled={!isValid || saving}
          loading={saving}
        >
          <Save className="w-4 h-4" />
          {isEditMode ? 'Save changes' : 'Save draft'}
        </GradientButton>

        {(!isEditMode || !isPublished) && (
          <GradientButton
            onClick={() => handleSave(true)}
            disabled={!isValid || saving}
            loading={saving}
          >
            <Send className="w-4 h-4" />
            {isEditMode ? 'Save & publish' : 'Publish now'}
          </GradientButton>
        )}
      </div>
    </div>
  );
}
