import { useState, useCallback } from 'react';
import { Search, Trash2, RotateCcw, ExternalLink, AlertTriangle } from 'lucide-react';
import { DataTable, Column } from '@/components/admin/DataTable';
import { useAdminComments, useRestoreComment, useHardDeleteComment, AdminCommentView } from '@/hooks/useAdminApi';
import { formatDistanceToNow } from 'date-fns';
import { Link } from 'react-router-dom';
import { toast } from 'sonner';

export default function AdminComments() {
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const [searchInput, setSearchInput] = useState('');
  const [entityTypeFilter, setEntityTypeFilter] = useState<string>('');
  const [deletedFilter, setDeletedFilter] = useState<string>('');
  const [sortBy, setSortBy] = useState<'createdAt' | 'updatedAt'>('createdAt');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

  const { data: comments, pagination, isLoading, refetch } = useAdminComments({
    page,
    limit: 20,
    search: search || undefined,
    entityType: entityTypeFilter || undefined,
    isDeleted: deletedFilter === '' ? undefined : deletedFilter === 'true',
    sortBy,
    sortOrder,
  });

  const { restoreComment, isLoading: restoring } = useRestoreComment();
  const { hardDeleteComment, isLoading: deleting } = useHardDeleteComment();

  const handleSearch = useCallback((e: React.FormEvent) => {
    e.preventDefault();
    setSearch(searchInput);
    setPage(1);
  }, [searchInput]);

  const handleSort = useCallback((key: string) => {
    if (key === sortBy) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortBy(key as typeof sortBy);
      setSortOrder('desc');
    }
  }, [sortBy, sortOrder]);

  const handleRestore = useCallback(async (comment: AdminCommentView) => {
    if (!confirm('Are you sure you want to restore this comment?')) {
      return;
    }

    const success = await restoreComment(comment.id);
    if (success) {
      toast.success('Comment restored successfully');
      refetch();
    } else {
      toast.error('Failed to restore comment');
    }
  }, [restoreComment, refetch]);

  const handleHardDelete = useCallback(async (comment: AdminCommentView) => {
    if (!confirm('Are you sure you want to permanently delete this comment? This action cannot be undone.')) {
      return;
    }

    const success = await hardDeleteComment(comment.id);
    if (success) {
      toast.success('Comment permanently deleted');
      refetch();
    } else {
      toast.error('Failed to delete comment');
    }
  }, [hardDeleteComment, refetch]);

  const columns: Column<AdminCommentView>[] = [
    {
      key: 'content',
      header: 'Comment',
      render: (comment) => (
        <div className="max-w-md">
          <p className={`text-sm ${comment.isDeleted ? 'text-gray-500 italic' : 'text-gray-200'} line-clamp-2`}>
            {comment.isDeleted ? '[deleted]' : comment.content}
          </p>
          {comment.replay && (
            <Link
              to={`/replays/${comment.replay.id}`}
              className="flex items-center gap-1 text-xs text-violet-400 hover:text-violet-300 mt-1"
              onClick={(e) => e.stopPropagation()}
            >
              <ExternalLink className="w-3 h-3" />
              {comment.replay.title || 'Unnamed replay'}
            </Link>
          )}
        </div>
      ),
    },
    {
      key: 'author',
      header: 'Author',
      render: (comment) => (
        <div className="flex items-center gap-2">
          {comment.author.avatarUrl ? (
            <img src={comment.author.avatarUrl} alt={comment.author.username} className="w-6 h-6 rounded-full" />
          ) : (
            <div className="w-6 h-6 rounded-full bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center text-white text-xs font-medium">
              {comment.author.username[0].toUpperCase()}
            </div>
          )}
          <span className="text-gray-300 text-sm">{comment.author.username}</span>
        </div>
      ),
    },
    {
      key: 'entityType',
      header: 'Type',
      render: (comment) => (
        <span className="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-300 capitalize">
          {comment.entityType}
        </span>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (comment) => (
        comment.isDeleted ? (
          <span className="flex items-center gap-1 text-xs px-2 py-0.5 rounded bg-red-500/20 text-red-400">
            <AlertTriangle className="w-3 h-3" />
            Deleted
          </span>
        ) : (
          <span className="text-xs px-2 py-0.5 rounded bg-green-500/20 text-green-400">
            Active
          </span>
        )
      ),
    },
    {
      key: 'createdAt',
      header: 'Created',
      sortable: true,
      render: (comment) => (
        <span className="text-gray-400 text-sm">
          {formatDistanceToNow(new Date(comment.createdAt), { addSuffix: true })}
        </span>
      ),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (comment) => (
        <div className="flex items-center gap-2">
          {comment.isDeleted && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleRestore(comment);
              }}
              disabled={restoring}
              className="p-1.5 rounded-lg text-green-400 hover:bg-green-500/20 transition-colors"
              title="Restore comment"
            >
              <RotateCcw className="w-4 h-4" />
            </button>
          )}
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleHardDelete(comment);
            }}
            disabled={deleting}
            className="p-1.5 rounded-lg text-red-400 hover:bg-red-500/20 transition-colors"
            title="Permanently delete"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Comment Management</h1>
        <p className="text-gray-400 mt-1">View and moderate user comments</p>
      </div>

      {/* Search and Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <form onSubmit={handleSearch} className="flex-1 flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="Search in comments..."
              className="w-full pl-10 pr-4 py-2 bg-gray-900/50 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-violet-500"
            />
          </div>
          <button
            type="submit"
            className="px-4 py-2 bg-violet-600 text-white rounded-lg hover:bg-violet-700 transition-colors"
          >
            Search
          </button>
        </form>

        {/* Filters */}
        <div className="flex gap-2">
          <select
            value={entityTypeFilter}
            onChange={(e) => {
              setEntityTypeFilter(e.target.value);
              setPage(1);
            }}
            className="px-3 py-2 bg-gray-900/50 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-violet-500"
          >
            <option value="">All types</option>
            <option value="replay">Replay</option>
            <option value="feedback">Feedback</option>
          </select>

          <select
            value={deletedFilter}
            onChange={(e) => {
              setDeletedFilter(e.target.value);
              setPage(1);
            }}
            className="px-3 py-2 bg-gray-900/50 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-violet-500"
          >
            <option value="">All status</option>
            <option value="false">Active</option>
            <option value="true">Deleted</option>
          </select>
        </div>
      </div>

      {/* Comments Table */}
      <DataTable
        data={comments}
        columns={columns}
        keyExtractor={(comment) => comment.id}
        pagination={pagination}
        onPageChange={setPage}
        sortBy={sortBy}
        sortOrder={sortOrder}
        onSort={handleSort}
        loading={isLoading}
        emptyMessage="No comments found"
      />
    </div>
  );
}
