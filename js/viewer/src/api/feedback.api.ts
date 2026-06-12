import { api } from '../services/api';

// Types
export interface FeedbackCategory {
  id: string;
  name: string;
  description: string | null;
  color: string;
  icon: string | null;
  displayOrder: number;
}

export interface FeedbackStatus {
  id: string;
  name: string;
  color: string;
  displayOrder: number;
}

export interface FeedbackAuthor {
  id: string | null;
  username: string;
  isAdmin: boolean;
  avatarUrl: string | null;
}

export interface FeedbackPost {
  id: string;
  title: string;
  content: string;
  summary: string | null; // AI-generated summary
  author: FeedbackAuthor;
  category: {
    id: string;
    name: string;
    color: string;
    icon: string | null;
  };
  status: {
    id: string;
    name: string;
    color: string;
  };
  upvoteCount: number;
  commentCount: number;
  createdAt: string;
  updatedAt: string;
  isEdited?: boolean;
}

export interface FeedbackComment {
  id: string;
  content: string;
  author: FeedbackAuthor;
  isDeleted: boolean;
  createdAt: string;
  updatedAt: string;
  isEdited?: boolean;
}

export interface PostListResponse {
  posts: FeedbackPost[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}

export interface PostDetailResponse {
  post: FeedbackPost;
  comments: FeedbackComment[];
  userVoted: boolean;
}

export interface VoteResponse {
  voted: boolean;
  upvoteCount: number;
}

export interface BatchVoteStatusResponse {
  votes: Record<string, boolean>;
}

// API functions
export const feedbackApi = {
  // Categories & Statuses
  async getCategories(): Promise<{ categories: FeedbackCategory[] }> {
    return api.get('/feedback/categories');
  },

  async getStatuses(): Promise<{ statuses: FeedbackStatus[] }> {
    return api.get('/feedback/statuses');
  },

  // Posts
  async listPosts(params: {
    page?: number;
    limit?: number;
    category?: string;
    status?: string;
    sortBy?: 'upvotes' | 'newest' | 'oldest';
  } = {}): Promise<PostListResponse> {
    const searchParams = new URLSearchParams();
    if (params.page) searchParams.set('page', String(params.page));
    if (params.limit) searchParams.set('limit', String(params.limit));
    if (params.category) searchParams.set('category', params.category);
    if (params.status) searchParams.set('status', params.status);
    if (params.sortBy) searchParams.set('sortBy', params.sortBy);

    const query = searchParams.toString();
    return api.get(`/feedback/posts${query ? `?${query}` : ''}`);
  },

  async getPost(id: string): Promise<PostDetailResponse> {
    return api.get(`/feedback/posts/${id}`);
  },

  async createPost(data: {
    title: string;
    content: string;
    categoryId: string;
  }): Promise<{ post: FeedbackPost }> {
    return api.post('/feedback/posts', data);
  },

  async updatePost(
    id: string,
    data: { title?: string; content?: string }
  ): Promise<{ post: FeedbackPost }> {
    return api.put(`/feedback/posts/${id}`, data);
  },

  async deletePost(id: string): Promise<void> {
    return api.delete(`/feedback/posts/${id}`);
  },

  async updatePostStatus(
    id: string,
    statusId: string
  ): Promise<{ post: FeedbackPost }> {
    return api.patch(`/feedback/posts/${id}/status`, { statusId });
  },

  // Comments
  async createComment(
    postId: string,
    content: string
  ): Promise<{ comment: FeedbackComment }> {
    return api.post(`/feedback/posts/${postId}/comments`, { content });
  },

  async updateComment(
    postId: string,
    commentId: string,
    content: string
  ): Promise<{ comment: FeedbackComment }> {
    return api.put(`/feedback/posts/${postId}/comments/${commentId}`, { content });
  },

  async deleteComment(postId: string, commentId: string): Promise<void> {
    return api.delete(`/feedback/posts/${postId}/comments/${commentId}`);
  },

  // Votes
  async toggleVote(postId: string): Promise<VoteResponse> {
    return api.post(`/feedback/posts/${postId}/vote`);
  },

  async checkVoteStatus(postId: string): Promise<VoteResponse> {
    return api.get(`/feedback/posts/${postId}/vote`);
  },

  async batchCheckVotes(postIds: string[]): Promise<BatchVoteStatusResponse> {
    return api.post('/feedback/votes/status', { postIds });
  },
};
