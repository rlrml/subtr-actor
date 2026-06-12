import { useState } from 'react';
import { XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, AreaChart, Area } from 'recharts';
import { useAnalytics } from '@/hooks/useAdminApi';
import { format, parseISO } from 'date-fns';

type Period = '7d' | '30d' | '90d' | 'all';

const periodLabels: Record<Period, string> = {
  '7d': 'Last 7 days',
  '30d': 'Last 30 days',
  '90d': 'Last 90 days',
  'all': 'All time',
};

export default function AdminAnalytics() {
  const [period, setPeriod] = useState<Period>('7d');
  const { data: analytics, isLoading, error } = useAnalytics(period);

  const formatDate = (dateStr: string) => {
    try {
      return format(parseISO(dateStr), 'MMM d');
    } catch {
      return dateStr;
    }
  };

  const renderChart = (
    title: string,
    data: { date: string; value: number }[],
    color: string,
    total: number
  ) => (
    <div className="bg-gray-900/50 border border-gray-800 rounded-xl p-5">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-white">{title}</h3>
        <span className="text-2xl font-bold text-white">{total.toLocaleString()}</span>
      </div>
      <div className="h-[200px]">
        {data.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            No data available
          </div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={data}>
              <defs>
                <linearGradient id={`gradient-${title}`} x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor={color} stopOpacity={0.3} />
                  <stop offset="95%" stopColor={color} stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis
                dataKey="date"
                tickFormatter={formatDate}
                stroke="#6b7280"
                fontSize={12}
              />
              <YAxis stroke="#6b7280" fontSize={12} />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#1f2937',
                  border: '1px solid #374151',
                  borderRadius: '0.5rem',
                }}
                labelFormatter={formatDate}
              />
              <Area
                type="monotone"
                dataKey="value"
                stroke={color}
                fill={`url(#gradient-${title})`}
                strokeWidth={2}
              />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Analytics</h1>
          <p className="text-gray-400 mt-1">Platform analytics and trends</p>
        </div>
        <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-6 text-center">
          <p className="text-red-400">Failed to load analytics data</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Analytics</h1>
          <p className="text-gray-400 mt-1">Platform analytics and trends</p>
        </div>
        <div className="flex gap-2">
          {(Object.keys(periodLabels) as Period[]).map((p) => (
            <button
              key={p}
              onClick={() => setPeriod(p)}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                period === p
                  ? 'bg-violet-600 text-white'
                  : 'bg-gray-800 text-gray-400 hover:text-white'
              }`}
            >
              {periodLabels[p]}
            </button>
          ))}
        </div>
      </div>

      {/* Charts Grid */}
      {isLoading ? (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="bg-gray-900/50 border border-gray-800 rounded-xl p-5">
              <div className="h-[250px] flex items-center justify-center">
                <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-violet-500" />
              </div>
            </div>
          ))}
        </div>
      ) : analytics ? (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {renderChart(
            'Uploads',
            analytics.series.uploads.data,
            '#8b5cf6',
            analytics.series.uploads.total
          )}
          {renderChart(
            'Registrations',
            analytics.series.registrations.data,
            '#06b6d4',
            analytics.series.registrations.total
          )}
          {renderChart(
            'Views',
            analytics.series.views.data,
            '#10b981',
            analytics.series.views.total
          )}
          {renderChart(
            'Active Users',
            analytics.series.activeUsers.data,
            '#f59e0b',
            analytics.series.activeUsers.total
          )}
        </div>
      ) : null}
    </div>
  );
}
