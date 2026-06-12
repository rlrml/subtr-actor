import { useState, useCallback } from 'react';
import { Mail, Eye, CheckCircle, Trash2, X, Clock, User, AtSign } from 'lucide-react';
import { DataTable, Column } from '@/components/admin/DataTable';
import {
  useContactMessages,
  useUpdateContactMessageStatus,
  useDeleteContactMessage,
  ContactMessageView,
} from '@/hooks/useAdminApi';
import { formatDistanceToNow, format } from 'date-fns';

type StatusFilter = 'all' | 'unread' | 'read' | 'processed';

const statusColors: Record<string, { bg: string; text: string }> = {
  unread: { bg: 'bg-blue-500/20', text: 'text-blue-400' },
  read: { bg: 'bg-yellow-500/20', text: 'text-yellow-400' },
  processed: { bg: 'bg-green-500/20', text: 'text-green-400' },
};

export default function AdminContactMessages() {
  const [page, setPage] = useState(1);
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all');
  const [selectedMessage, setSelectedMessage] = useState<ContactMessageView | null>(null);

  const { data: messages, pagination, unreadCount, isLoading, refetch } = useContactMessages({
    page,
    limit: 20,
    status: statusFilter,
    sortOrder: 'desc',
  });

  const { updateStatus, isLoading: isUpdating } = useUpdateContactMessageStatus();
  const { deleteMessage, isLoading: isDeleting } = useDeleteContactMessage();

  const handleStatusChange = useCallback(async (messageId: string, newStatus: 'read' | 'processed') => {
    await updateStatus(messageId, newStatus);
    refetch();
    if (selectedMessage?.id === messageId) {
      setSelectedMessage((prev) => prev ? { ...prev, status: newStatus } : null);
    }
  }, [updateStatus, refetch, selectedMessage]);

  const handleDelete = useCallback(async (messageId: string) => {
    if (!confirm('Are you sure you want to delete this message? This action cannot be undone.')) {
      return;
    }
    const success = await deleteMessage(messageId);
    if (success) {
      refetch();
      if (selectedMessage?.id === messageId) {
        setSelectedMessage(null);
      }
    }
  }, [deleteMessage, refetch, selectedMessage]);

  const handleViewMessage = useCallback((message: ContactMessageView) => {
    setSelectedMessage(message);
    // Auto-mark as read when viewing
    if (message.status === 'unread') {
      handleStatusChange(message.id, 'read');
    }
  }, [handleStatusChange]);

  const columns: Column<ContactMessageView>[] = [
    {
      key: 'sender',
      header: 'Sender',
      render: (message) => (
        <div>
          <div className="flex items-center gap-2">
            <span className="font-medium text-white">{message.name}</span>
            {message.user && (
              <span className="text-xs px-1.5 py-0.5 rounded bg-violet-500/20 text-violet-400">
                User
              </span>
            )}
          </div>
          <p className="text-xs text-gray-500">{message.email}</p>
        </div>
      ),
    },
    {
      key: 'subject',
      header: 'Subject',
      render: (message) => (
        <div className="max-w-md">
          <p className={`font-medium truncate ${message.status === 'unread' ? 'text-white' : 'text-gray-300'}`}>
            {message.subject}
          </p>
          {message.summary && (
            <p className="text-xs text-gray-500 truncate mt-0.5">{message.summary}</p>
          )}
        </div>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (message) => {
        const colors = statusColors[message.status] || statusColors.unread;
        return (
          <span className={`text-xs px-2 py-1 rounded-full ${colors.bg} ${colors.text} capitalize`}>
            {message.status}
          </span>
        );
      },
    },
    {
      key: 'createdAt',
      header: 'Received',
      render: (message) => (
        <span className="text-gray-400 text-sm">
          {formatDistanceToNow(new Date(message.createdAt), { addSuffix: true })}
        </span>
      ),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (message) => (
        <div className="flex items-center gap-2">
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleViewMessage(message);
            }}
            className="p-1.5 rounded hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
            title="View message"
          >
            <Eye className="w-4 h-4" />
          </button>
          {message.status !== 'processed' && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleStatusChange(message.id, 'processed');
              }}
              disabled={isUpdating}
              className="p-1.5 rounded hover:bg-green-500/20 text-gray-400 hover:text-green-400 transition-colors"
              title="Mark as processed"
            >
              <CheckCircle className="w-4 h-4" />
            </button>
          )}
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleDelete(message.id);
            }}
            disabled={isDeleting}
            className="p-1.5 rounded hover:bg-red-500/20 text-gray-400 hover:text-red-400 transition-colors"
            title="Delete message"
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
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white flex items-center gap-3">
            <Mail className="w-7 h-7 text-violet-400" />
            Contact Messages
            {unreadCount > 0 && (
              <span className="text-sm px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400">
                {unreadCount} unread
              </span>
            )}
          </h1>
          <p className="text-gray-400 mt-1">Manage messages from the contact form</p>
        </div>

        {/* Status Filter */}
        <div className="flex items-center gap-2">
          <span className="text-sm text-gray-400">Filter:</span>
          <select
            value={statusFilter}
            onChange={(e) => {
              setStatusFilter(e.target.value as StatusFilter);
              setPage(1);
            }}
            className="px-3 py-1.5 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:outline-none focus:ring-2 focus:ring-violet-500"
          >
            <option value="all">All Messages</option>
            <option value="unread">Unread</option>
            <option value="read">Read</option>
            <option value="processed">Processed</option>
          </select>
        </div>
      </div>

      {/* Messages Table */}
      <DataTable
        data={messages}
        columns={columns}
        keyExtractor={(message) => message.id}
        pagination={pagination}
        onPageChange={setPage}
        loading={isLoading}
        emptyMessage="No contact messages found"
        onRowClick={handleViewMessage}
      />

      {/* Message Detail Modal */}
      {selectedMessage && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-gray-900 rounded-xl border border-gray-800 max-w-2xl w-full max-h-[80vh] overflow-hidden flex flex-col">
            {/* Modal Header */}
            <div className="p-4 border-b border-gray-800 flex items-start justify-between">
              <div>
                <h2 className="text-lg font-semibold text-white">{selectedMessage.subject}</h2>
                <div className="flex items-center gap-3 mt-1 text-sm text-gray-400">
                  <span className="flex items-center gap-1">
                    <User className="w-3.5 h-3.5" />
                    {selectedMessage.name}
                  </span>
                  <span className="flex items-center gap-1">
                    <AtSign className="w-3.5 h-3.5" />
                    {selectedMessage.email}
                  </span>
                </div>
              </div>
              <button
                onClick={() => setSelectedMessage(null)}
                className="p-1.5 rounded hover:bg-gray-800 text-gray-400 hover:text-white transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* Modal Body */}
            <div className="p-4 flex-1 overflow-y-auto">
              {/* AI Summary */}
              {selectedMessage.summary && (
                <div className="mb-4 p-3 rounded-lg bg-violet-500/10 border border-violet-500/20">
                  <p className="text-xs font-medium text-violet-400 mb-1">AI Summary</p>
                  <p className="text-sm text-gray-300">{selectedMessage.summary}</p>
                </div>
              )}

              {/* Message Content */}
              <div className="prose prose-invert prose-sm max-w-none">
                <p className="whitespace-pre-wrap text-gray-300">{selectedMessage.content}</p>
              </div>

              {/* Metadata */}
              <div className="mt-6 pt-4 border-t border-gray-800 grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-gray-500">Received</p>
                  <p className="text-gray-300 flex items-center gap-1.5">
                    <Clock className="w-3.5 h-3.5" />
                    {format(new Date(selectedMessage.createdAt), 'PPpp')}
                  </p>
                </div>
                <div>
                  <p className="text-gray-500">Status</p>
                  <span className={`text-xs px-2 py-1 rounded-full ${statusColors[selectedMessage.status].bg} ${statusColors[selectedMessage.status].text} capitalize`}>
                    {selectedMessage.status}
                  </span>
                </div>
                <div>
                  <p className="text-gray-500">IP Address</p>
                  <p className="text-gray-300 font-mono text-xs">{selectedMessage.senderIp}</p>
                </div>
                {selectedMessage.user && (
                  <div>
                    <p className="text-gray-500">Registered User</p>
                    <p className="text-gray-300">{selectedMessage.user.username}</p>
                  </div>
                )}
              </div>
            </div>

            {/* Modal Footer */}
            <div className="p-4 border-t border-gray-800 flex items-center justify-between">
              <div className="flex items-center gap-2">
                {selectedMessage.status !== 'processed' && (
                  <button
                    onClick={() => handleStatusChange(selectedMessage.id, 'processed')}
                    disabled={isUpdating}
                    className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-green-500/20 text-green-400 hover:bg-green-500/30 transition-colors text-sm"
                  >
                    <CheckCircle className="w-4 h-4" />
                    Mark as Processed
                  </button>
                )}
              </div>
              <button
                onClick={() => handleDelete(selectedMessage.id)}
                disabled={isDeleting}
                className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors text-sm"
              >
                <Trash2 className="w-4 h-4" />
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
