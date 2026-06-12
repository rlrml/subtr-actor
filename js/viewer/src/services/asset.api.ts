import { api } from './api';
import type { Asset, AssetType, AssetListResponse } from '../types/environment';

export interface ListAssetsParams {
  type?: AssetType;
  page?: number;
  limit?: number;
  search?: string;
}

/**
 * Asset API client
 */
export const assetApi = {
  /**
   * List assets with optional filtering
   */
  async list(params: ListAssetsParams = {}): Promise<AssetListResponse> {
    const searchParams = new URLSearchParams();
    if (params.type) searchParams.set('type', params.type);
    if (params.page) searchParams.set('page', String(params.page));
    if (params.limit) searchParams.set('limit', String(params.limit));
    if (params.search) searchParams.set('search', params.search);

    const query = searchParams.toString();
    return api.get<AssetListResponse>(`/assets${query ? `?${query}` : ''}`);
  },

  /**
   * Get asset by ID
   */
  async get(id: string): Promise<Asset> {
    return api.get<Asset>(`/assets/${id}`);
  },

  /**
   * Upload a new asset
   */
  async upload(file: File, type: AssetType, name?: string): Promise<Asset> {
    const formData = new FormData();
    // IMPORTANT: Text fields must come BEFORE the file for @fastify/multipart
    formData.append('type', type);
    formData.append('name', name || file.name.replace(/\.[^.]+$/, ''));
    formData.append('file', file);

    return api.postForm<Asset>('/assets', formData);
  },

  /**
   * Delete an asset
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/assets/${id}`);
  },

  /**
   * Rename an asset
   */
  async rename(id: string, name: string): Promise<Asset> {
    return api.patch<Asset>(`/assets/${id}`, { name });
  },

  /**
   * Get asset download URL
   */
  getDownloadUrl(id: string): string {
    const baseUrl = import.meta.env.VITE_API_URL || '/api';
    return `${baseUrl}/assets/${id}/download`;
  },
};
