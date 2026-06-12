import { useState, useCallback } from 'react';
import { ThumbsUp, MessageSquare } from 'lucide-react';
import { DataTable, Column } from '@/components/admin/DataTable';
import { useFeedbacks, AdminFeedbackView } from '@/hooks/useAdminApi';
import { formatDistanceToNow } from 'date-fns';
import { Link } from 'react-router-dom';

export default function AdminFeedbacks() {
  const [page, setPage] = useState(1);
  const [sortBy, setSortBy] = useState<'createdAt' | 'upvoteCount'>('createdAt');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

  const { data: feedbacks, pagination, isLoading } = useFeedbacks({
    page,
    limit: 20,
    sortBy,
    sortOrder,
  });

  const handleSort = useCallback((key: string) => {
    if (key === sortBy) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortBy(key as typeof sortBy);
      setSortOrder('desc');
    }
  }, [sortBy, sortOrder]);

  const columns: Column<AdminFeedbackView>[] = [
    {
      key: 'title',
      header: 'Feedback',
      render: (feedback) => (
        <div>
          <Link
            to={`/feedback/${feedback.id}`}
            className="font-medium text-white hover:text-violet-400 transition-colors"
          >
            {feedback.title}
          </Link>
          <p className="text-xs text-gray-500">
            by {feedback.author.username} • {formatDistanceToNow(new Date(feedback.createdAt), { addSuffix: true })}
          </p>
        </div>
      ),
    },
    {
      key: 'category',
      header: 'Category',
      render: (feedback) => (
        <span
          className="text-xs px-2 py-1 rounded-full"
          style={{
            backgroundColor: `${feedback.category.color}20`,
            color: feedback.category.color,
          }}
        >
          {feedback.category.name}
        </span>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (feedback) => (
        <span
          className="text-xs px-2 py-1 rounded-full"
          style={{
            backgroundColor: `${feedback.status.color}20`,
            color: feedback.status.color,
          }}
        >
          {feedback.status.name}
        </span>
      ),
    },
    {
      key: 'upvoteCount',
      header: 'Votes',
      sortable: true,
      render: (feedback) => (
        <div className="flex items-center gap-1 text-gray-300">
          <ThumbsUp className="w-3.5 h-3.5" />
          <span>{feedback.upvoteCount}</span>
        </div>
      ),
    },
    {
      key: 'commentCount',
      header: 'Comments',
      render: (feedback) => (
        <div className="flex items-center gap-1 text-gray-300">
          <MessageSquare className="w-3.5 h-3.5" />
          <span>{feedback.commentCount}</span>
        </div>
      ),
    },
    {
      key: 'createdAt',
      header: 'Created',
      sortable: true,
      render: (feedback) => (
        <span className="text-gray-400">
          {formatDistanceToNow(new Date(feedback.createdAt), { addSuffix: true })}
        </span>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Feedback Management</h1>
        <p className="text-gray-400 mt-1">View and manage community feedback</p>
      </div>

      {/* Feedbacks Table */}
      <DataTable
        data={feedbacks}
        columns={columns}
        keyExtractor={(feedback) => feedback.id}
        pagination={pagination}
        onPageChange={setPage}
        sortBy={sortBy}
        sortOrder={sortOrder}
        onSort={handleSort}
        loading={isLoading}
        emptyMessage="No feedback found"
      />
    </div>
  );
}
