// T022: Refactored to use centralized attemptTokenRefresh from client.ts
// This eliminates duplicate refresh logic across the codebase
import { attemptTokenRefresh, API_URL } from '@/api/client';

const API_BASE = API_URL;

export interface ApiError {
  statusCode: number;
  error: string;
  message: string;
}

export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
    isRetry = false
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;

    // Only add Content-Type header if there's a body
    const headers: HeadersInit = options.body
      ? { 'Content-Type': 'application/json', ...options.headers }
      : { ...options.headers };

    const response = await fetch(url, {
      ...options,
      credentials: 'include',
      headers,
    });

    // Handle 401 Unauthorized - use centralized attemptTokenRefresh
    if (response.status === 401 && !isRetry && !endpoint.includes('/auth/')) {
      const refreshed = await attemptTokenRefresh();
      if (refreshed) {
        // Retry the original request
        return this.request<T>(endpoint, options, true);
      }
    }

    if (!response.ok) {
      const error: ApiError = await response.json().catch(() => ({
        statusCode: response.status,
        error: 'Error',
        message: response.statusText,
      }));
      throw error;
    }

    // Handle empty responses
    const text = await response.text();
    return text ? JSON.parse(text) : (null as T);
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint);
  }

  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async patch<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PATCH',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async postForm<T>(endpoint: string, formData: FormData, isRetry = false): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;

    const response = await fetch(url, {
      method: 'POST',
      credentials: 'include',
      body: formData,
    });

    // Handle 401 Unauthorized - use centralized attemptTokenRefresh
    if (response.status === 401 && !isRetry) {
      const refreshed = await attemptTokenRefresh();
      if (refreshed) {
        return this.postForm<T>(endpoint, formData, true);
      }
    }

    if (!response.ok) {
      const error: ApiError = await response.json().catch(() => ({
        statusCode: response.status,
        error: 'Error',
        message: response.statusText,
      }));
      throw error;
    }

    return response.json();
  }

  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }

  getBinaryUrl(replayId: string): string {
    return `${this.baseUrl}/replays/${replayId}/binary`;
  }
}

export const api = new ApiClient();
