// Comments API client

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

// Helper function for API requests
async function apiRequest<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const response = await fetch(`${API_URL}${endpoint}`, {
    ...options,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  const data = await response.json();

  if (!response.ok) {
    throw new Error(data.message || 'Request failed');
  }

  return data as T;
}

export interface CommentAuthor {
  id: string;
  username: string;
  isAdmin: boolean;
  avatarUrl: string | null;
}

export interface Comment {
  id: string;
  entityType: string;
  entityId: string;
  authorId: string;
  content: string;
  isDeleted: boolean;
  isEdited?: boolean;
  createdAt: string;
  updatedAt: string;
  author: CommentAuthor;
}

// Alias for Comment with author (they're the same in this API)
export type CommentWithAuthor = Comment;

export interface GetCommentsResponse {
  comments: Comment[];
  total: number;
  hasMore: boolean;
}

export interface CreateCommentRequest {
  entityType: string;
  entityId: string;
  content: string;
}

export interface CreateCommentResponse {
  comment: Comment;
}

export interface UpdateCommentResponse {
  comment: Comment;
}

// Direct function exports for convenience
export async function createComment(data: CreateCommentRequest): Promise<CreateCommentResponse> {
  return apiRequest<CreateCommentResponse>('/comments', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export const commentApi = {
  // Get comments for an entity
  async getComments(
    entityType: string,
    entityId: string,
    limit: number = 50,
    offset: number = 0
  ): Promise<GetCommentsResponse> {
    const params = new URLSearchParams({
      entityType,
      entityId,
      limit: limit.toString(),
      offset: offset.toString(),
    });
    return apiRequest<GetCommentsResponse>(`/comments?${params}`);
  },

  // Create a comment
  async createComment(data: CreateCommentRequest): Promise<CreateCommentResponse> {
    return createComment(data);
  },

  // Update a comment
  async updateComment(id: string, content: string): Promise<UpdateCommentResponse> {
    return apiRequest<UpdateCommentResponse>(`/comments/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({ content }),
    });
  },

  // Delete a comment
  async deleteComment(id: string): Promise<{ message: string }> {
    return apiRequest<{ message: string }>(`/comments/${id}`, {
      method: 'DELETE',
    });
  },
};
