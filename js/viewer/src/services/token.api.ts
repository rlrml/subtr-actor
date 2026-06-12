import { api } from './api';

// Types
export interface ApiToken {
  id: string;
  name: string;
  scope: string;
  expiresAt: string | null;
  lastUsedAt: string | null;
  createdAt: string;
}

export interface CreateTokenRequest {
  name: string;
  expires_at?: string | null;
  scope?: 'replay:upload';
}

export interface CreateTokenResponse {
  message: string;
  token: string; // Plain token - shown only once
  tokenInfo: {
    id: string;
    name: string;
    scope: string;
    expiresAt: string | null;
    createdAt: string;
  };
}

export interface TokenListResponse {
  tokens: ApiToken[];
}

// API Service
export const tokenApi = {
  /**
   * List all active tokens for the current user
   */
  async list(): Promise<ApiToken[]> {
    const response = await api.get<TokenListResponse>('/tokens');
    return response.tokens;
  },

  /**
   * Create a new Personal Access Token
   * The token string is only returned once - save it immediately!
   */
  async create(data: CreateTokenRequest): Promise<CreateTokenResponse> {
    return api.post<CreateTokenResponse>('/tokens', data);
  },

  /**
   * Revoke (delete) a token
   */
  async revoke(tokenId: string): Promise<void> {
    await api.delete(`/tokens/${tokenId}`);
  },
};
