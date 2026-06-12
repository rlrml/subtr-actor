import { Users, Play, Eye, Heart, MessageSquare, AlertCircle, Clock, TrendingUp } from 'lucide-react';
import { MetricCard } from '@/components/admin/MetricCard';
import { useMetrics, useRecentReplays, useRecentUsers } from '@/hooks/useAdminApi';
import { formatDistanceToNow } from 'date-fns';
import { Link } from 'react-router-dom';

export default function AdminDashboard() {
  const { data: metrics, isLoading: metricsLoading } = useMetrics(30000); // Refresh every 30s
  const { data: recentReplays, isLoading: replaysLoading } = useRecentReplays(5);
  const { data: recentUsers, isLoading: usersLoading } = useRecentUsers(5);

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Dashboard Overview</h1>
        <p className="text-gray-400 mt-1">Platform metrics and recent activity</p>
      </div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Total Users"
          value={metrics?.totalUsers ?? 0}
          icon={Users}
          trend={metrics ? { value: metrics.usersToday, label: 'today', positive: true } : undefined}
          loading={metricsLoading}
        />
        <MetricCard
          title="Total Replays"
          value={metrics?.totalReplays ?? 0}
          icon={Play}
          trend={metrics ? { value: metrics.replaysToday, label: 'today', positive: true } : undefined}
          loading={metricsLoading}
        />
        <MetricCard
          title="Total Views"
          value={metrics?.totalViews ?? 0}
          icon={Eye}
          loading={metricsLoading}
        />
        <MetricCard
          title="Total Likes"
          value={metrics?.totalLikes ?? 0}
          icon={Heart}
          loading={metricsLoading}
        />
      </div>

      {/* Secondary Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Active Today"
          value={metrics?.activeUsersToday ?? 0}
          icon={TrendingUp}
          trend={metrics ? { value: metrics.activeUsers7d, label: 'this week', positive: true } : undefined}
          loading={metricsLoading}
        />
        <MetricCard
          title="Pending Replays"
          value={metrics?.pendingReplays ?? 0}
          icon={Clock}
          loading={metricsLoading}
        />
        <MetricCard
          title="Failed Replays"
          value={metrics?.failedReplays ?? 0}
          icon={AlertCircle}
          loading={metricsLoading}
        />
        <MetricCard
          title="Total Comments"
          value={metrics?.totalComments ?? 0}
          icon={MessageSquare}
          loading={metricsLoading}
        />
      </div>

      {/* Recent Activity Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Replays */}
        <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-5">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Recent Replays</h2>
            <Link
              to="/admin/replays"
              className="text-sm text-violet-400 hover:text-violet-300 transition-colors"
            >
              View all
            </Link>
          </div>
          {replaysLoading ? (
            <div className="space-y-3">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="h-12 bg-gray-700/30 rounded animate-pulse" />
              ))}
            </div>
          ) : recentReplays.length === 0 ? (
            <p className="text-gray-500 text-center py-8">No replays yet</p>
          ) : (
            <div className="space-y-2">
              {recentReplays.map((replay) => (
                <div
                  key={replay.id}
                  className="flex items-center justify-between p-3 rounded-lg hover:bg-gray-800/50 transition-colors"
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-200 truncate">
                      {replay.title || replay.originalFilename}
                    </p>
                    <p className="text-xs text-gray-500">
                      {replay.owner?.username ?? 'Anonymous'} • {formatDistanceToNow(new Date(replay.createdAt), { addSuffix: true })}
                    </p>
                  </div>
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
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Recent Users */}
        <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-5">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Recent Users</h2>
            <Link
              to="/admin/users"
              className="text-sm text-violet-400 hover:text-violet-300 transition-colors"
            >
              View all
            </Link>
          </div>
          {usersLoading ? (
            <div className="space-y-3">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="h-12 bg-gray-700/30 rounded animate-pulse" />
              ))}
            </div>
          ) : recentUsers.length === 0 ? (
            <p className="text-gray-500 text-center py-8">No users yet</p>
          ) : (
            <div className="space-y-2">
              {recentUsers.map((user) => (
                <div
                  key={user.id}
                  className="flex items-center justify-between p-3 rounded-lg hover:bg-gray-800/50 transition-colors"
                >
                  <div className="flex items-center gap-3">
                    {user.avatarUrl ? (
                      <img
                        src={user.avatarUrl}
                        alt={user.username}
                        className="w-8 h-8 rounded-full"
                      />
                    ) : (
                      <div className="w-8 h-8 rounded-full bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center text-white text-sm font-medium">
                        {user.username[0].toUpperCase()}
                      </div>
                    )}
                    <div>
                      <p className="text-sm font-medium text-gray-200">{user.username}</p>
                      <p className="text-xs text-gray-500">
                        {user.replayCount} replays • {formatDistanceToNow(new Date(user.createdAt), { addSuffix: true })}
                      </p>
                    </div>
                  </div>
                  {user.isAdmin && (
                    <span className="text-xs px-2 py-1 rounded-full bg-violet-500/20 text-violet-400">
                      Admin
                    </span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
