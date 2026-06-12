import { useState, useCallback } from 'react';
import { Key, Smartphone, Ban } from 'lucide-react';
import { DataTable, Column } from '@/components/admin/DataTable';
import { useTokens, useDevices, useRevokeToken, useRevokeDevice, AdminTokenView, AdminDeviceView } from '@/hooks/useAdminApi';
import { formatDistanceToNow } from 'date-fns';
import { toast } from 'sonner';

type Tab = 'tokens' | 'devices';

export default function AdminTokens() {
  const [activeTab, setActiveTab] = useState<Tab>('tokens');
  const [page, setPage] = useState(1);
  const [statusFilter, setStatusFilter] = useState<string>('');

  // Tokens hooks
  const { data: tokens, pagination: tokensPagination, isLoading: tokensLoading, refetch: refetchTokens } = useTokens({
    page,
    limit: 20,
    status: statusFilter as 'active' | 'revoked' | 'expired' | undefined,
  });
  const { revokeToken, isLoading: revokingToken } = useRevokeToken();

  // Devices hooks
  const { data: devices, pagination: devicesPagination, isLoading: devicesLoading, refetch: refetchDevices } = useDevices({
    page,
    limit: 20,
    status: statusFilter as 'active' | 'revoked' | undefined,
  });
  const { revokeDevice, isLoading: revokingDevice } = useRevokeDevice();

  const handleRevokeToken = useCallback(async (token: AdminTokenView) => {
    if (!confirm(`Are you sure you want to revoke token "${token.name}"?`)) {
      return;
    }
    const success = await revokeToken(token.id);
    if (success) {
      toast.success('Token revoked successfully');
      refetchTokens();
    } else {
      toast.error('Failed to revoke token');
    }
  }, [revokeToken, refetchTokens]);

  const handleRevokeDevice = useCallback(async (device: AdminDeviceView) => {
    if (!confirm(`Are you sure you want to revoke device "${device.deviceName || device.clientId}"?`)) {
      return;
    }
    const success = await revokeDevice(device.id);
    if (success) {
      toast.success('Device revoked successfully');
      refetchDevices();
    } else {
      toast.error('Failed to revoke device');
    }
  }, [revokeDevice, refetchDevices]);

  const tokenColumns: Column<AdminTokenView>[] = [
    {
      key: 'name',
      header: 'Token',
      render: (token) => (
        <div>
          <p className="font-medium text-white">{token.name}</p>
          <p className="text-xs text-gray-500">
            {token.owner.username} • Scope: {token.scope}
          </p>
        </div>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (token) => (
        <span
          className={`text-xs px-2 py-1 rounded-full ${
            token.status === 'active'
              ? 'bg-green-500/20 text-green-400'
              : token.status === 'expired'
              ? 'bg-yellow-500/20 text-yellow-400'
              : 'bg-red-500/20 text-red-400'
          }`}
        >
          {token.status}
        </span>
      ),
    },
    {
      key: 'lastUsedAt',
      header: 'Last Used',
      render: (token) => (
        <div>
          <p className="text-gray-300">
            {token.lastUsedAt
              ? formatDistanceToNow(new Date(token.lastUsedAt), { addSuffix: true })
              : 'Never'}
          </p>
          {token.lastUsedIp && (
            <p className="text-xs text-gray-500">{token.lastUsedIp}</p>
          )}
        </div>
      ),
    },
    {
      key: 'createdAt',
      header: 'Created',
      render: (token) => (
        <span className="text-gray-400">
          {formatDistanceToNow(new Date(token.createdAt), { addSuffix: true })}
        </span>
      ),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (token) =>
        token.status === 'active' ? (
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleRevokeToken(token);
            }}
            disabled={revokingToken}
            className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium text-red-400 bg-red-500/10 hover:bg-red-500/20 rounded-lg transition-colors"
          >
            <Ban className="w-3.5 h-3.5" />
            Revoke
          </button>
        ) : null,
    },
  ];

  const deviceColumns: Column<AdminDeviceView>[] = [
    {
      key: 'deviceName',
      header: 'Device',
      render: (device) => (
        <div>
          <p className="font-medium text-white">{device.deviceName || 'Unnamed Device'}</p>
          <p className="text-xs text-gray-500">
            {device.owner.username} • Client: {device.clientId}
          </p>
        </div>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (device) => (
        <span
          className={`text-xs px-2 py-1 rounded-full ${
            device.status === 'active'
              ? 'bg-green-500/20 text-green-400'
              : 'bg-red-500/20 text-red-400'
          }`}
        >
          {device.status}
        </span>
      ),
    },
    {
      key: 'lastUsedAt',
      header: 'Last Used',
      render: (device) => (
        <div>
          <p className="text-gray-300">
            {device.lastUsedAt
              ? formatDistanceToNow(new Date(device.lastUsedAt), { addSuffix: true })
              : 'Never'}
          </p>
          {device.lastUsedIp && (
            <p className="text-xs text-gray-500">{device.lastUsedIp}</p>
          )}
        </div>
      ),
    },
    {
      key: 'createdAt',
      header: 'Created',
      render: (device) => (
        <span className="text-gray-400">
          {formatDistanceToNow(new Date(device.createdAt), { addSuffix: true })}
        </span>
      ),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (device) =>
        device.status === 'active' ? (
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleRevokeDevice(device);
            }}
            disabled={revokingDevice}
            className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium text-red-400 bg-red-500/10 hover:bg-red-500/20 rounded-lg transition-colors"
          >
            <Ban className="w-3.5 h-3.5" />
            Revoke
          </button>
        ) : null,
    },
  ];

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Tokens & Devices</h1>
        <p className="text-gray-400 mt-1">Manage API tokens and authorized devices</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 border-b border-gray-800 pb-4">
        <button
          onClick={() => {
            setActiveTab('tokens');
            setPage(1);
            setStatusFilter('');
          }}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            activeTab === 'tokens'
              ? 'bg-violet-600 text-white'
              : 'text-gray-400 hover:text-white hover:bg-gray-800'
          }`}
        >
          <Key className="w-4 h-4" />
          Personal Access Tokens
        </button>
        <button
          onClick={() => {
            setActiveTab('devices');
            setPage(1);
            setStatusFilter('');
          }}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            activeTab === 'devices'
              ? 'bg-violet-600 text-white'
              : 'text-gray-400 hover:text-white hover:bg-gray-800'
          }`}
        >
          <Smartphone className="w-4 h-4" />
          Authorized Devices
        </button>
      </div>

      {/* Filters */}
      <div className="flex gap-4">
        <select
          value={statusFilter}
          onChange={(e) => {
            setStatusFilter(e.target.value);
            setPage(1);
          }}
          className="px-4 py-2 bg-gray-900/50 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-violet-500"
        >
          <option value="">All statuses</option>
          <option value="active">Active</option>
          <option value="revoked">Revoked</option>
          {activeTab === 'tokens' && <option value="expired">Expired</option>}
        </select>
      </div>

      {/* Table */}
      {activeTab === 'tokens' ? (
        <DataTable
          data={tokens}
          columns={tokenColumns}
          keyExtractor={(token) => token.id}
          pagination={tokensPagination}
          onPageChange={setPage}
          loading={tokensLoading}
          emptyMessage="No tokens found"
        />
      ) : (
        <DataTable
          data={devices}
          columns={deviceColumns}
          keyExtractor={(device) => device.id}
          pagination={devicesPagination}
          onPageChange={setPage}
          loading={devicesLoading}
          emptyMessage="No devices found"
        />
      )}
    </div>
  );
}
