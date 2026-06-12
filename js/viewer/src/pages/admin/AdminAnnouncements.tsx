import { useCallback, useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import {
  Plus,
  Edit,
  Trash2,
  Eye,
  EyeOff,
  MessageCircle,
  Eye as ViewsIcon,
  Loader2,
} from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { toast } from 'sonner';
import { DataTable, Column } from '@/components/admin/DataTable';
import { GradientButton } from '@/components/ui/GradientButton';
import {
  listAllAnnouncementsAdmin,
  publishAnnouncement,
  unpublishAnnouncement,
  deleteAnnouncement,
  type Announcement,
  type AnnouncementStatusFilter,
} from '@/api/announcements.api';

const PAGE_SIZE = 20;

export default function AdminAnnouncements() {
  const navigate = useNavigate();
  const [items, setItems] = useState<Announcement[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [statusFilter, setStatusFilter] = useState<AnnouncementStatusFilter>('all');
  const [loading, setLoading] = useState(true);
  const [busyId, setBusyId] = useState<string | null>(null);

  const fetch = useCallback(async () => {
    setLoading(true);
    try {
      const offset = (page - 1) * PAGE_SIZE;
      const response = await listAllAnnouncementsAdmin({
        status: statusFilter,
        limit: PAGE_SIZE,
        offset,
      });
      setItems(response.items);
      setTotal(response.total);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load announcements';
      toast.error(message);
    } finally {
      setLoading(false);
    }
  }, [page, statusFilter]);

  useEffect(() => {
    fetch();
  }, [fetch]);

  // Reset to page 1 when filter changes
  useEffect(() => {
    setPage(1);
  }, [statusFilter]);

  const handleTogglePublish = async (announcement: Announcement) => {
    setBusyId(announcement.id);
    try {
      if (announcement.isPublished) {
        await unpublishAnnouncement(announcement.id);
        toast.success('Announcement unpublished');
      } else {
        await publishAnnouncement(announcement.id);
        toast.success('Announcement published');
      }
      await fetch();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Action failed';
      toast.error(message);
    } finally {
      setBusyId(null);
    }
  };

  const handleDelete = async (announcement: Announcement) => {
    const confirmed = window.confirm(
      `Delete "${announcement.title}"? This will also remove all comments. This cannot be undone.`,
    );
    if (!confirmed) return;

    setBusyId(announcement.id);
    try {
      await deleteAnnouncement(announcement.id);
      toast.success('Announcement deleted');
      await fetch();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Delete failed';
      toast.error(message);
    } finally {
      setBusyId(null);
    }
  };

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  const columns: Column<Announcement>[] = [
    {
      key: 'title',
      header: 'Title',
      render: (a) => (
        <div className="min-w-0">
          <Link
            to={`/admin/announcements/${a.id}`}
            className="font-medium text-white hover:text-violet-400 transition-colors line-clamp-1"
          >
            {a.title}
          </Link>
          <p className="text-xs text-gray-500 truncate">/{a.slug}</p>
        </div>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (a) =>
        a.isPublished ? (
          <span className="inline-flex items-center gap-1 text-xs px-2 py-1 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
            Published
          </span>
        ) : (
          <span className="inline-flex items-center gap-1 text-xs px-2 py-1 rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/30">
            Draft
          </span>
        ),
    },
    {
      key: 'publishedAt',
      header: 'Published',
      render: (a) =>
        a.publishedAt ? (
          <span className="text-gray-400 text-xs">
            {formatDistanceToNow(new Date(a.publishedAt), { addSuffix: true })}
          </span>
        ) : (
          <span className="text-gray-600 text-xs">—</span>
        ),
    },
    {
      key: 'viewCount',
      header: 'Views',
      render: (a) => (
        <span className="inline-flex items-center gap-1 text-gray-300 text-xs">
          <ViewsIcon className="w-3 h-3" />
          {a.viewCount.toLocaleString()}
        </span>
      ),
    },
    {
      key: 'commentCount',
      header: 'Comments',
      render: (a) => (
        <span className="inline-flex items-center gap-1 text-gray-300 text-xs">
          <MessageCircle className="w-3 h-3" />
          {a.commentCount.toLocaleString()}
        </span>
      ),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (a) => {
        const isBusy = busyId === a.id;
        return (
          <div className="flex items-center gap-1">
            <Link
              to={`/admin/announcements/${a.id}`}
              className="p-1.5 rounded-lg text-gray-400 hover:text-violet-300 hover:bg-violet-500/10 transition-colors"
              title="Edit"
            >
              <Edit className="w-4 h-4" />
            </Link>
            <button
              onClick={() => handleTogglePublish(a)}
              disabled={isBusy}
              className="p-1.5 rounded-lg text-gray-400 hover:text-emerald-300 hover:bg-emerald-500/10 transition-colors disabled:opacity-40"
              title={a.isPublished ? 'Unpublish' : 'Publish'}
            >
              {isBusy ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : a.isPublished ? (
                <EyeOff className="w-4 h-4" />
              ) : (
                <Eye className="w-4 h-4" />
              )}
            </button>
            <button
              onClick={() => handleDelete(a)}
              disabled={isBusy}
              className="p-1.5 rounded-lg text-gray-400 hover:text-red-300 hover:bg-red-500/10 transition-colors disabled:opacity-40"
              title="Delete"
            >
              <Trash2 className="w-4 h-4" />
            </button>
          </div>
        );
      },
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-white">Announcements</h1>
          <p className="text-gray-400 mt-1 text-sm">
            Create and manage news posts shown on the public site
          </p>
        </div>
        <GradientButton
          onClick={() => navigate('/admin/announcements/new')}
          size="md"
        >
          <Plus className="w-4 h-4" />
          New announcement
        </GradientButton>
      </div>

      {/* Status filter */}
      <div className="flex items-center gap-2 flex-wrap">
        {(['all', 'published', 'draft'] as const).map((status) => {
          const isActive = statusFilter === status;
          const label = status === 'all' ? 'All' : status === 'published' ? 'Published' : 'Drafts';
          return (
            <button
              key={status}
              onClick={() => setStatusFilter(status)}
              className={
                'px-3 py-1.5 rounded-lg text-sm font-medium transition-all border ' +
                (isActive
                  ? 'bg-gradient-to-r from-violet-600/30 to-blue-600/30 text-white border-violet-500/40'
                  : 'bg-gray-800/50 text-gray-400 hover:text-white hover:bg-gray-800 border-gray-700')
              }
            >
              {label}
            </button>
          );
        })}
      </div>

      {/* Table */}
      <DataTable
        data={items}
        columns={columns}
        keyExtractor={(item) => item.id}
        loading={loading}
        emptyMessage="No announcements yet. Create one to get started."
        pagination={{
          page,
          totalPages,
          total,
          limit: PAGE_SIZE,
        }}
        onPageChange={setPage}
      />
    </div>
  );
}
