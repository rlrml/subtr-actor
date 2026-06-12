import { useState, useCallback } from 'react';
import { Search, Shield, ShieldOff } from 'lucide-react';
import { DataTable, Column } from '@/components/admin/DataTable';
import { useUsers, useToggleAdmin, AdminUserView } from '@/hooks/useAdminApi';
import { formatDistanceToNow } from 'date-fns';
import { toast } from 'sonner';

export default function AdminUsers() {
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const [searchInput, setSearchInput] = useState('');
  const [sortBy, setSortBy] = useState<'createdAt' | 'lastLoginAt' | 'replayCount'>('createdAt');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

  const { data: users, pagination, isLoading, refetch } = useUsers({
    page,
    limit: 20,
    search: search || undefined,
    sortBy,
    sortOrder,
  });

  const { toggleAdmin, isLoading: togglingAdmin } = useToggleAdmin();

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

  const handleToggleAdmin = useCallback(async (user: AdminUserView) => {
    const newStatus = !user.isAdmin;
    const action = newStatus ? 'promote' : 'demote';

    if (!confirm(`Are you sure you want to ${action} ${user.username} ${newStatus ? 'to' : 'from'} admin?`)) {
      return;
    }

    const result = await toggleAdmin(user.id, newStatus);
    if (result) {
      toast.success(`${user.username} has been ${newStatus ? 'promoted to' : 'demoted from'} admin`);
      refetch();
    } else {
      toast.error('Failed to update admin status');
    }
  }, [toggleAdmin, refetch]);

  const columns: Column<AdminUserView>[] = [
    {
      key: 'username',
      header: 'User',
      render: (user) => (
        <div className="flex items-center gap-3">
          {user.avatarUrl ? (
            <img src={user.avatarUrl} alt={user.username} className="w-8 h-8 rounded-full" />
          ) : (
            <div className="w-8 h-8 rounded-full bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center text-white text-sm font-medium">
              {user.username[0].toUpperCase()}
            </div>
          )}
          <div>
            <p className="font-medium text-white">{user.username}</p>
            <p className="text-xs text-gray-500">{user.email || 'No email'}</p>
          </div>
        </div>
      ),
    },
    {
      key: 'replayCount',
      header: 'Replays',
      sortable: true,
      render: (user) => <span className="text-gray-300">{user.replayCount}</span>,
    },
    {
      key: 'oauthProviders',
      header: 'Providers',
      render: (user) => (
        <div className="flex gap-1">
          {user.oauthProviders.map((provider) => (
            <span
              key={provider}
              className="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-300 capitalize"
            >
              {provider}
            </span>
          ))}
          {user.oauthProviders.length === 0 && (
            <span className="text-xs text-gray-500">Email only</span>
          )}
        </div>
      ),
    },
    {
      key: 'createdAt',
      header: 'Joined',
      sortable: true,
      render: (user) => (
        <span className="text-gray-400">
          {formatDistanceToNow(new Date(user.createdAt), { addSuffix: true })}
        </span>
      ),
    },
    {
      key: 'lastLoginAt',
      header: 'Last Login',
      sortable: true,
      render: (user) => (
        <span className="text-gray-400">
          {user.lastLoginAt
            ? formatDistanceToNow(new Date(user.lastLoginAt), { addSuffix: true })
            : 'Never'}
        </span>
      ),
    },
    {
      key: 'isAdmin',
      header: 'Role',
      render: (user) => (
        <button
          onClick={(e) => {
            e.stopPropagation();
            handleToggleAdmin(user);
          }}
          disabled={togglingAdmin}
          className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-medium transition-colors ${
            user.isAdmin
              ? 'bg-violet-500/20 text-violet-400 hover:bg-violet-500/30'
              : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
          }`}
        >
          {user.isAdmin ? <Shield className="w-3.5 h-3.5" /> : <ShieldOff className="w-3.5 h-3.5" />}
          {user.isAdmin ? 'Admin' : 'User'}
        </button>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">User Management</h1>
        <p className="text-gray-400 mt-1">View and manage platform users</p>
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
              placeholder="Search by username or email..."
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
      </div>

      {/* Users Table */}
      <DataTable
        data={users}
        columns={columns}
        keyExtractor={(user) => user.id}
        pagination={pagination}
        onPageChange={setPage}
        sortBy={sortBy}
        sortOrder={sortOrder}
        onSort={handleSort}
        loading={isLoading}
        emptyMessage="No users found"
      />
    </div>
  );
}
