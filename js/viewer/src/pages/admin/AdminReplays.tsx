import { useState, useCallback } from 'react';
import { Search, Trash2, Eye, EyeOff, RefreshCw, Download, FileJson, MessageCircle, Star, User, CheckCircle, AlertTriangle } from 'lucide-react';
import { DataTable, Column } from '@/components/admin/DataTable';
import { useReplays, useDeleteReplay, useRetryReplay, useChangeVisibility, useForceRecompile, useMetrics, AdminReplayView } from '@/hooks/useAdminApi';
import { formatDistanceToNow } from 'date-fns';
import { toast } from 'sonner';
import { Link, useNavigate } from 'react-router-dom';

const API_URL = import.meta.env.VITE_API_URL || '/api';

// Quality score badge component
function QualityBadge({ score }: { score: number | null }) {
  if (score === null) return <span className="text-gray-500 text-xs">-</span>;

  let colorClass = 'bg-gray-500/20 text-gray-400';

  if (score >= 80) {
    colorClass = 'bg-green-500/20 text-green-400';
  } else if (score >= 60) {
    colorClass = 'bg-blue-500/20 text-blue-400';
  } else if (score >= 40) {
    colorClass = 'bg-yellow-500/20 text-yellow-400';
  } else if (score >= 20) {
    colorClass = 'bg-orange-500/20 text-orange-400';
  } else {
    colorClass = 'bg-red-500/20 text-red-400';
  }

  return (
    <span className={`text-xs px-2 py-0.5 rounded-full ${colorClass}`} title={`Quality: ${score}/100`}>
      {score}
    </span>
  );
}

// Version badge component - green if up to date, red if outdated
function VersionBadge({ version, currentVersion }: { version: string | null; currentVersion: string | null }) {
  if (!version) return <span className="text-gray-500 text-xs">-</span>;

  const isUpToDate = version === currentVersion;

  return (
    <span
      className={`inline-flex items-center gap-1 text-xs font-mono px-2 py-0.5 rounded ${
        isUpToDate
          ? 'bg-green-500/20 text-green-400'
          : 'bg-red-500/20 text-red-400'
      }`}
      title={isUpToDate ? 'Up to date' : `Outdated (current: ${currentVersion})`}
    >
      {isUpToDate ? (
        <CheckCircle className="w-3 h-3" />
      ) : (
        <AlertTriangle className="w-3 h-3" />
      )}
      {version}
    </span>
  );
}

export default function AdminReplays() {
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const [searchInput, setSearchInput] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('');
  const [sortBy, setSortBy] = useState<'createdAt' | 'viewCount' | 'likeCount'>('createdAt');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

  const { data: metrics } = useMetrics();
  const currentFrameworkVersion = metrics?.currentFrameworkVersion ?? null;

  const { data: replays, pagination, isLoading, refetch } = useReplays({
    page,
    limit: 20,
    search: search || undefined,
    status: statusFilter || undefined,
    sortBy,
    sortOrder,
  });

  const { deleteReplay, isLoading: deleting } = useDeleteReplay();
  const { retryReplay, isLoading: retrying } = useRetryReplay();
  const { changeVisibility, isLoading: changingVisibility } = useChangeVisibility();
  const { forceRecompile, isLoading: recompiling } = useForceRecompile();

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

  const handleDelete = useCallback(async (replay: AdminReplayView) => {
    if (!confirm(`Are you sure you want to delete "${replay.title || replay.originalFilename}"? This action cannot be undone.`)) {
      return;
    }
    const success = await deleteReplay(replay.id);
    if (success) {
      toast.success('Replay deleted successfully');
      refetch();
    } else {
      toast.error('Failed to delete replay');
    }
  }, [deleteReplay, refetch]);

  const handleRetry = useCallback(async (replay: AdminReplayView) => {
    const success = await retryReplay(replay.id);
    if (success) {
      toast.success('Replay queued for reprocessing');
      refetch();
    } else {
      toast.error('Failed to retry replay');
    }
  }, [retryReplay, refetch]);

  const handleForceRecompile = useCallback(async (replay: AdminReplayView) => {
    if (!confirm(`Force recompile "${replay.title || replay.originalFilename}"? This will reprocess the replay from scratch.`)) {
      return;
    }
    const success = await forceRecompile(replay.id);
    if (success) {
      toast.success('Replay queued for recompilation');
      refetch();
    } else {
      toast.error('Failed to force recompile');
    }
  }, [forceRecompile, refetch]);

  const handleVisibility = useCallback(async (replay: AdminReplayView) => {
    const newVisibility = replay.visibility === 'public' ? 'unlisted' : 'public';
    const result = await changeVisibility(replay.id, newVisibility);
    if (result) {
      toast.success(`Replay is now ${newVisibility}`);
      refetch();
    } else {
      toast.error('Failed to change visibility');
    }
  }, [changeVisibility, refetch]);

  const handleDownloadReplay = useCallback(async (replay: AdminReplayView) => {
    try {
      const response = await fetch(`${API_URL}/admin/replays/${replay.id}/download-replay`, {
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Download failed');

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = replay.originalFilename;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch {
      toast.error('Failed to download replay file');
    }
  }, []);

  const handleDownloadJson = useCallback(async (replay: AdminReplayView) => {
    toast.info('Generating JSON... This may take a moment.');
    try {
      const response = await fetch(`${API_URL}/admin/replays/${replay.id}/download-json`, {
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Download failed');

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = replay.originalFilename.replace(/\.replay$/i, '.json');
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
      toast.success('JSON downloaded');
    } catch {
      toast.error('Failed to generate JSON');
    }
  }, []);

  const handleCommentsClick = useCallback((_replay: AdminReplayView) => {
    // Navigate to comments page with replay filter
    navigate(`/admin/comments?entityType=replay`);
  }, [navigate]);

  const columns: Column<AdminReplayView>[] = [
    {
      key: 'title',
      header: 'Replay',
      render: (replay) => (
        <div className="min-w-[200px]">
          <Link
            to={`/replays/${replay.id}`}
            className="font-medium text-white hover:text-violet-400 transition-colors"
          >
            {replay.title || replay.originalFilename}
          </Link>
          <p className="text-xs text-gray-500">{replay.mapName || 'Unknown map'}</p>
        </div>
      ),
    },
    {
      key: 'owner',
      header: 'Owner',
      render: (replay) => (
        <div className="flex items-center gap-2">
          <User className="w-3.5 h-3.5 text-gray-500" />
          <span className="text-gray-300 text-sm">
            {replay.owner?.username ?? <span className="text-gray-500 italic">Anonymous</span>}
          </span>
        </div>
      ),
    },
    {
      key: 'quality',
      header: 'Quality',
      render: (replay) => <QualityBadge score={replay.qualityScore} />,
    },
    {
      key: 'status',
      header: 'Status',
      render: (replay) => (
        <div className="flex items-center gap-2">
          <span
            className={`text-xs px-2 py-1 rounded-full ${
              replay.status === 'completed'
                ? 'bg-green-500/20 text-green-400'
                : replay.status === 'error'
                ? 'bg-red-500/20 text-red-400'
                : replay.status === 'processing'
                ? 'bg-blue-500/20 text-blue-400'
                : 'bg-yellow-500/20 text-yellow-400'
            }`}
          >
            {replay.status}
          </span>
          {replay.status === 'error' && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleRetry(replay);
              }}
              disabled={retrying}
              className="p-1 text-gray-400 hover:text-white transition-colors"
              title="Retry processing"
            >
              <RefreshCw className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      ),
    },
    {
      key: 'comments',
      header: 'Comments',
      render: (replay) => (
        <button
          onClick={(e) => {
            e.stopPropagation();
            handleCommentsClick(replay);
          }}
          className="flex items-center gap-1 text-gray-300 hover:text-violet-400 transition-colors"
          title="View comments"
        >
          <MessageCircle className="w-3.5 h-3.5" />
          <span>{replay.commentCount}</span>
        </button>
      ),
    },
    {
      key: 'version',
      header: 'Version',
      render: (replay) => (
        <VersionBadge version={replay.frameworkVersion} currentVersion={currentFrameworkVersion} />
      ),
    },
    {
      key: 'viewCount',
      header: 'Views',
      sortable: true,
      render: (replay) => <span className="text-gray-300">{replay.viewCount.toLocaleString()}</span>,
    },
    {
      key: 'likeCount',
      header: 'Likes',
      sortable: true,
      render: (replay) => (
        <div className="flex items-center gap-1 text-gray-300">
          <Star className="w-3.5 h-3.5" />
          <span>{replay.likeCount.toLocaleString()}</span>
        </div>
      ),
    },
    {
      key: 'createdAt',
      header: 'Uploaded',
      sortable: true,
      render: (replay) => (
        <span className="text-gray-400 text-sm">
          {formatDistanceToNow(new Date(replay.createdAt), { addSuffix: true })}
        </span>
      ),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (replay) => (
        <div className="flex items-center gap-1">
          {/* Download .replay */}
          {replay.replayFileKey && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleDownloadReplay(replay);
              }}
              className="p-1.5 text-gray-400 hover:text-blue-400 hover:bg-gray-700 rounded transition-colors"
              title="Download .replay"
            >
              <Download className="w-4 h-4" />
            </button>
          )}
          {/* Download JSON */}
          {replay.replayFileKey && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleDownloadJson(replay);
              }}
              className="p-1.5 text-gray-400 hover:text-green-400 hover:bg-gray-700 rounded transition-colors"
              title="Download JSON"
            >
              <FileJson className="w-4 h-4" />
            </button>
          )}
          {/* Force recompile */}
          {replay.replayFileKey && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleForceRecompile(replay);
              }}
              disabled={recompiling}
              className="p-1.5 text-gray-400 hover:text-orange-400 hover:bg-gray-700 rounded transition-colors"
              title="Force recompile"
            >
              <RefreshCw className="w-4 h-4" />
            </button>
          )}
          {/* Visibility toggle */}
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleVisibility(replay);
            }}
            disabled={changingVisibility}
            className="p-1.5 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors"
            title={replay.visibility === 'public' ? 'Make unlisted' : 'Make public'}
          >
            {replay.visibility === 'public' ? (
              <Eye className="w-4 h-4" />
            ) : (
              <EyeOff className="w-4 h-4" />
            )}
          </button>
          {/* Delete */}
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleDelete(replay);
            }}
            disabled={deleting}
            className="p-1.5 text-gray-400 hover:text-red-400 hover:bg-gray-700 rounded transition-colors"
            title="Delete replay"
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
        <h1 className="text-2xl font-bold text-white">Replay Management</h1>
        <p className="text-gray-400 mt-1">View and manage uploaded replays</p>
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
              placeholder="Search by title, filename, or map..."
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
        <select
          value={statusFilter}
          onChange={(e) => {
            setStatusFilter(e.target.value);
            setPage(1);
          }}
          className="px-4 py-2 bg-gray-900/50 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-violet-500"
        >
          <option value="">All statuses</option>
          <option value="pending">Pending</option>
          <option value="processing">Processing</option>
          <option value="completed">Completed</option>
          <option value="error">Error</option>
        </select>
      </div>

      {/* Replays Table */}
      <DataTable
        data={replays}
        columns={columns}
        keyExtractor={(replay) => replay.id}
        pagination={pagination}
        onPageChange={setPage}
        sortBy={sortBy}
        sortOrder={sortOrder}
        onSort={handleSort}
        loading={isLoading}
        emptyMessage="No replays found"
      />
    </div>
  );
}
