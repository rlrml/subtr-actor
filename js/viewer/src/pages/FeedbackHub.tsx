import { useState, useEffect, useCallback } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { Plus, Filter, ChevronDown, ChevronLeft, ChevronRight } from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { feedbackApi } from '@/api/feedback.api';
import { FeedbackList } from '@/components/feedback/FeedbackList';
import type { FeedbackPost, FeedbackCategory, FeedbackStatus } from '@/api/feedback.api';

type SortBy = 'upvotes' | 'newest' | 'oldest';

const POSTS_PER_PAGE = 10;

export default function FeedbackHub() {
  const { isAuthenticated } = useAuth();
  const [searchParams, setSearchParams] = useSearchParams();

  // Data state
  const [posts, setPosts] = useState<FeedbackPost[]>([]);
  const [categories, setCategories] = useState<FeedbackCategory[]>([]);
  const [statuses, setStatuses] = useState<FeedbackStatus[]>([]);
  const [votedPosts, setVotedPosts] = useState<Record<string, boolean>>({});
  const [pagination, setPagination] = useState({
    page: 1,
    limit: POSTS_PER_PAGE,
    total: 0,
    totalPages: 0,
  });

  // UI state
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Filter state from URL
  const selectedCategory = searchParams.get('category') || '';
  const selectedStatus = searchParams.get('status') || '';
  const sortBy = (searchParams.get('sortBy') as SortBy) || 'upvotes';
  const page = parseInt(searchParams.get('page') || '1', 10);

  // Update URL params
  const updateFilters = useCallback((updates: Record<string, string | null>) => {
    const newParams = new URLSearchParams(searchParams);
    Object.entries(updates).forEach(([key, value]) => {
      if (value) {
        newParams.set(key, value);
      } else {
        newParams.delete(key);
      }
    });
    // Reset to page 1 when filters change (except when changing page)
    if (!('page' in updates)) {
      newParams.set('page', '1');
    }
    setSearchParams(newParams);
  }, [searchParams, setSearchParams]);

  // Load categories and statuses on mount
  useEffect(() => {
    const loadMetadata = async () => {
      try {
        const [categoriesRes, statusesRes] = await Promise.all([
          feedbackApi.getCategories(),
          feedbackApi.getStatuses(),
        ]);
        setCategories(categoriesRes.categories);
        setStatuses(statusesRes.statuses);
      } catch (err) {
        console.error('Failed to load metadata:', err);
      }
    };
    loadMetadata();
  }, []);

  // Load posts when filters change
  useEffect(() => {
    const loadPosts = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const response = await feedbackApi.listPosts({
          page,
          limit: POSTS_PER_PAGE,
          category: selectedCategory || undefined,
          status: selectedStatus || undefined,
          sortBy,
        });
        setPosts(response.posts);
        setPagination(response.pagination);
      } catch (err) {
        console.error('Failed to load posts:', err);
        setError('Failed to load feedback posts. Please try again.');
      } finally {
        setIsLoading(false);
      }
    };

    loadPosts();
  }, [page, selectedCategory, selectedStatus, sortBy]);

  // Load vote status for authenticated users
  useEffect(() => {
    const loadVoteStatus = async () => {
      if (!isAuthenticated || posts.length === 0) {
        setVotedPosts({});
        return;
      }

      try {
        const postIds = posts.map(p => p.id);
        const response = await feedbackApi.batchCheckVotes(postIds);
        setVotedPosts(response.votes);
      } catch (err) {
        console.error('Failed to load vote status:', err);
      }
    };

    loadVoteStatus();
  }, [isAuthenticated, posts]);

  // Handle vote toggle
  const handleVote = async (postId: string) => {
    if (!isAuthenticated) {
      // Redirect to login - could also show a modal
      window.location.href = '/login?redirect=/feedback';
      return;
    }

    // Optimistic update
    const wasVoted = votedPosts[postId];
    setVotedPosts(prev => ({ ...prev, [postId]: !wasVoted }));
    setPosts(prev => prev.map(p => {
      if (p.id === postId) {
        return {
          ...p,
          upvoteCount: p.upvoteCount + (wasVoted ? -1 : 1),
        };
      }
      return p;
    }));

    try {
      await feedbackApi.toggleVote(postId);
    } catch (err) {
      // Rollback on error
      console.error('Failed to toggle vote:', err);
      setVotedPosts(prev => ({ ...prev, [postId]: wasVoted }));
      setPosts(prev => prev.map(p => {
        if (p.id === postId) {
          return {
            ...p,
            upvoteCount: p.upvoteCount + (wasVoted ? 1 : -1),
          };
        }
        return p;
      }));
    }
  };

  return (
    <div className="max-w-6xl mx-auto space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold bg-gradient-to-r from-violet-400 to-blue-400 bg-clip-text text-transparent">
            Feedback Hub
          </h1>
          <p className="text-gray-400 mt-1">
            Share your ideas, report bugs, and vote on features
          </p>
        </div>
        <Link
          to={isAuthenticated ? "/feedback/new" : "/login?redirect=/feedback/new"}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-violet-600 to-blue-600 text-white font-medium hover:from-violet-500 hover:to-blue-500 transition-all duration-200 shadow-lg shadow-violet-500/25"
        >
          <Plus className="w-5 h-5" />
          New Post
        </Link>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-4 p-4 rounded-xl bg-gray-900/50 border border-gray-800">
        <div className="flex items-center gap-2 text-gray-400">
          <Filter className="w-4 h-4" />
          <span className="text-sm font-medium">Filters:</span>
        </div>

        {/* Category Filter */}
        <div className="relative">
          <select
            value={selectedCategory}
            onChange={(e) => updateFilters({ category: e.target.value || null })}
            className="appearance-none bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 pr-8 text-sm text-gray-200 focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500"
          >
            <option value="">All Categories</option>
            {categories.map((cat) => (
              <option key={cat.id} value={cat.id}>
                {cat.name}
              </option>
            ))}
          </select>
          <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" />
        </div>

        {/* Status Filter */}
        <div className="relative">
          <select
            value={selectedStatus}
            onChange={(e) => updateFilters({ status: e.target.value || null })}
            className="appearance-none bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 pr-8 text-sm text-gray-200 focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500"
          >
            <option value="">All Statuses</option>
            {statuses.map((status) => (
              <option key={status.id} value={status.id}>
                {status.name}
              </option>
            ))}
          </select>
          <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" />
        </div>

        {/* Sort */}
        <div className="relative ml-auto">
          <select
            value={sortBy}
            onChange={(e) => updateFilters({ sortBy: e.target.value })}
            className="appearance-none bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 pr-8 text-sm text-gray-200 focus:outline-none focus:ring-2 focus:ring-violet-500/50 focus:border-violet-500"
          >
            <option value="upvotes">Most Voted</option>
            <option value="newest">Newest</option>
            <option value="oldest">Oldest</option>
          </select>
          <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" />
        </div>
      </div>

      {/* Error State */}
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400">
          {error}
        </div>
      )}

      {/* Posts List */}
      <FeedbackList
        posts={posts}
        votedPosts={votedPosts}
        onVote={handleVote}
        isLoading={isLoading}
      />

      {/* Pagination */}
      {pagination.totalPages > 1 && (
        <div className="flex items-center justify-center gap-2">
          <button
            onClick={() => updateFilters({ page: String(page - 1) })}
            disabled={page <= 1}
            className="flex items-center gap-1 px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <ChevronLeft className="w-4 h-4" />
            Previous
          </button>

          <div className="flex items-center gap-1">
            {Array.from({ length: Math.min(5, pagination.totalPages) }, (_, i) => {
              let pageNum: number;
              if (pagination.totalPages <= 5) {
                pageNum = i + 1;
              } else if (page <= 3) {
                pageNum = i + 1;
              } else if (page >= pagination.totalPages - 2) {
                pageNum = pagination.totalPages - 4 + i;
              } else {
                pageNum = page - 2 + i;
              }

              return (
                <button
                  key={pageNum}
                  onClick={() => updateFilters({ page: String(pageNum) })}
                  className={`
                    w-10 h-10 rounded-lg font-medium transition-colors
                    ${page === pageNum
                      ? 'bg-violet-600 text-white'
                      : 'bg-gray-800 border border-gray-700 text-gray-300 hover:bg-gray-700'
                    }
                  `}
                >
                  {pageNum}
                </button>
              );
            })}
          </div>

          <button
            onClick={() => updateFilters({ page: String(page + 1) })}
            disabled={page >= pagination.totalPages}
            className="flex items-center gap-1 px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Next
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      )}

      {/* Results info */}
      {!isLoading && posts.length > 0 && (
        <p className="text-center text-sm text-gray-500">
          Showing {(page - 1) * POSTS_PER_PAGE + 1}-{Math.min(page * POSTS_PER_PAGE, pagination.total)} of {pagination.total} posts
        </p>
      )}
    </div>
  );
}
