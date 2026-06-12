import { api } from './api';
import type {
  Environment,
  EnvironmentListResponse,
  CreateEnvironmentRequest,
  UpdateEnvironmentRequest,
} from '../types/environment';

/**
 * Environment API client
 */
export const environmentApi = {
  /**
   * List all environments
   */
  async list(page = 1, limit = 50): Promise<EnvironmentListResponse> {
    return api.get<EnvironmentListResponse>(`/environments?page=${page}&limit=${limit}`);
  },

  /**
   * Get environment by ID
   */
  async get(id: string): Promise<Environment> {
    return api.get<Environment>(`/environments/${id}`);
  },

  /**
   * Get the default environment
   */
  async getDefault(): Promise<Environment | null> {
    try {
      return await api.get<Environment>('/environments/default');
    } catch (error) {
      // Return null if no default environment is set
      if ((error as { statusCode?: number }).statusCode === 404) {
        return null;
      }
      throw error;
    }
  },

  /**
   * Create a new environment
   */
  async create(data: CreateEnvironmentRequest): Promise<Environment> {
    return api.post<Environment>('/environments', data);
  },

  /**
   * Update an environment
   */
  async update(id: string, data: UpdateEnvironmentRequest): Promise<Environment> {
    return api.put<Environment>(`/environments/${id}`, data);
  },

  /**
   * Delete an environment
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/environments/${id}`);
  },

  /**
   * Duplicate an environment
   */
  async duplicate(id: string, newName: string): Promise<Environment> {
    return api.post<Environment>(`/environments/${id}/duplicate`, { name: newName });
  },

  /**
   * Set an environment as the default
   */
  async setDefault(id: string): Promise<void> {
    return api.post(`/environments/${id}/set-default`);
  },
};
