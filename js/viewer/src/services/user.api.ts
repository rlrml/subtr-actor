import { api } from './api';

export interface UserPreferences {
  preferredEnvironmentId: string | null;
  effectiveEnvironmentId: string | null; // The one to actually use (preference or default)
}

/**
 * User API client for preferences
 */
export const userApi = {
  /**
   * Get user preferences (including environment preference)
   */
  async getPreferences(): Promise<UserPreferences> {
    return api.get<UserPreferences>('/users/me/preferences');
  },

  /**
   * Update user's preferred environment
   */
  async updatePreferredEnvironment(environmentId: string | null): Promise<void> {
    await api.patch('/users/me/preferences', { preferredEnvironmentId: environmentId });
  },
};
