// Like API client

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

export interface LikeStatusResponse {
  liked: boolean;
  likeCount: number;
}

export interface ToggleLikeResponse {
  liked: boolean;
  likeCount: number;
}

export const likeApi = {
  // Get like status for a replay
  async getLikeStatus(replayId: string): Promise<LikeStatusResponse> {
    return apiRequest<LikeStatusResponse>(`/replays/${replayId}/like`);
  },

  // Toggle like on a replay
  async toggleLike(replayId: string): Promise<ToggleLikeResponse> {
    return apiRequest<ToggleLikeResponse>(`/replays/${replayId}/like`, {
      method: 'POST',
      body: JSON.stringify({}),
    });
  },
};
