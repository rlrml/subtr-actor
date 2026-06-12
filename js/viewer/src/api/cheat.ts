/**
 * Cheat Detection API Client
 * Frontend API functions for cheat detection feature (032-cheat-detection)
 */

import { useState, useEffect, useCallback } from 'react';
import { authenticatedFetch } from './client';
import type {
  CheatAnalysisResult,
  PlayerCheatHistory,
  CheatStats,
  ReanalyzeResponse,
  CheatersQueryParams,
  CheatAttribution,
  CheaterSummary,
} from '../types/cheat';

const API_BASE = import.meta.env.VITE_API_URL || '';

// Default attribution for whosbotting.com
export const CHEAT_ATTRIBUTION: CheatAttribution = {
  service: 'whosbotting.com',
  url: 'https://whosbotting.com',
};

/**
 * Get cheat analysis results for a replay
 */
export async function getReplayCheatAnalysis(
  replayId: string
): Promise<CheatAnalysisResult> {
  const response = await fetch(`${API_BASE}/replays/${replayId}/cheat-analysis`);

  if (!response.ok) {
    if (response.status === 404) {
      throw new Error('Replay not found');
    }
    throw new Error('Failed to fetch cheat analysis');
  }

  return response.json();
}

/**
 * Get player cheat history for profile page
 */
export async function getPlayerCheatHistory(
  playerId: string
): Promise<PlayerCheatHistory> {
  const response = await fetch(`${API_BASE}/players/${playerId}/cheat-history`);

  if (!response.ok) {
    if (response.status === 404) {
      throw new Error('Player not found');
    }
    throw new Error('Failed to fetch player cheat history');
  }

  return response.json();
}

/**
 * Response type for cheaters list API
 */
export interface CheatersListResponse {
  cheaters: CheaterSummary[];
  total: number;
  page: number;
  totalPages: number;
  attribution: CheatAttribution;
}

/**
 * Get list of all detected cheaters with pagination
 */
export async function getCheatersList(
  params: CheatersQueryParams = {}
): Promise<CheatersListResponse> {
  const searchParams = new URLSearchParams();

  if (params.page) searchParams.set('page', params.page.toString());
  if (params.limit) searchParams.set('limit', params.limit.toString());
  if (params.search) searchParams.set('search', params.search);
  if (params.sortBy) searchParams.set('sortBy', params.sortBy);
  if (params.sortOrder) searchParams.set('sortOrder', params.sortOrder);

  const queryString = searchParams.toString();
  const url = `${API_BASE}/cheaters${queryString ? `?${queryString}` : ''}`;

  const response = await fetch(url);

  if (!response.ok) {
    throw new Error('Failed to fetch cheaters list');
  }

  return response.json();
}

/**
 * Get aggregate cheat statistics
 */
export async function getCheatStats(): Promise<CheatStats> {
  const response = await fetch(`${API_BASE}/stats/cheaters`);

  if (!response.ok) {
    throw new Error('Failed to fetch cheat statistics');
  }

  return response.json();
}

/**
 * Admin: Re-run cheat analysis on a replay
 */
export async function reanalyzeReplay(
  replayId: string
): Promise<ReanalyzeResponse> {
  return authenticatedFetch<ReanalyzeResponse>(
    `/admin/replays/${replayId}/reanalyze-cheat`,
    {
      method: 'POST',
    }
  );
}

/**
 * Request cheat analysis for a pending replay (public endpoint)
 */
export async function requestCheatAnalysis(
  replayId: string
): Promise<{ message: string; replayId: string; jobId: string }> {
  const response = await fetch(`${API_BASE}/replays/${replayId}/request-cheat-analysis`, {
    method: 'POST',
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new Error(error.message || 'Failed to request cheat analysis');
  }

  return response.json();
}

// ===================
// React Hooks
// ===================

/**
 * Hook to fetch cheat analysis for a replay
 * Polls every 5 seconds if analysis is pending or analyzing
 */
export function useReplayCheatAnalysis(replayId: string | undefined) {
  const [data, setData] = useState<CheatAnalysisResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = useCallback(async () => {
    if (!replayId) return;

    try {
      const result = await getReplayCheatAnalysis(replayId);
      setData(result);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    }
  }, [replayId]);

  useEffect(() => {
    if (!replayId) {
      setData(null);
      return;
    }

    setLoading(true);
    fetchData().finally(() => setLoading(false));
  }, [replayId, fetchData]);

  // Poll if analysis is pending or analyzing
  useEffect(() => {
    if (!data || (data.status !== 'pending' && data.status !== 'analyzing')) {
      return;
    }

    const interval = setInterval(() => {
      fetchData();
    }, 5000);

    return () => clearInterval(interval);
  }, [data, fetchData]);

  return { data, loading, error, refetch: fetchData };
}

/**
 * Hook to fetch player cheat history
 */
export function usePlayerCheatHistory(playerId: string | undefined) {
  const [data, setData] = useState<PlayerCheatHistory | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!playerId) {
      setData(null);
      return;
    }

    setLoading(true);
    getPlayerCheatHistory(playerId)
      .then((result) => {
        setData(result);
        setError(null);
      })
      .catch((err) => {
        setError(err instanceof Error ? err : new Error('Unknown error'));
      })
      .finally(() => {
        setLoading(false);
      });
  }, [playerId]);

  return { data, loading, error };
}

/**
 * Hook to fetch cheaters list
 */
export function useCheatersList(params: CheatersQueryParams = {}) {
  const [data, setData] = useState<CheatersListResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setLoading(true);
    getCheatersList(params)
      .then((result) => {
        setData(result);
        setError(null);
      })
      .catch((err) => {
        setError(err instanceof Error ? err : new Error('Unknown error'));
      })
      .finally(() => {
        setLoading(false);
      });
  }, [params.page, params.limit, params.search, params.sortBy, params.sortOrder]);

  return { data, loading, error };
}

/**
 * Hook to fetch cheat statistics
 */
export function useCheatStats() {
  const [data, setData] = useState<CheatStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setLoading(true);
    getCheatStats()
      .then((result) => {
        setData(result);
        setError(null);
      })
      .catch((err) => {
        setError(err instanceof Error ? err : new Error('Unknown error'));
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  return { data, loading, error };
}

/**
 * Hook for admin reanalyze functionality
 */
export function useReanalyzeReplay() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (replayId: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await reanalyzeReplay(replayId);
      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  return { mutate, loading, error };
}

/**
 * Hook for requesting cheat analysis on a pending replay
 */
export function useRequestCheatAnalysis() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (replayId: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await requestCheatAnalysis(replayId);
      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  return { mutate, loading, error };
}

// ===================
// Helper Functions
// ===================

/**
 * Get confidence level label and color class
 */
export function getConfidenceLevel(confidencePercent: number): {
  label: string;
  colorClass: string;
  badgeVariant: 'destructive' | 'warning' | 'secondary';
} {
  if (confidencePercent > 50) {
    return {
      label: 'Cheater Detected',
      colorClass: 'text-red-500',
      badgeVariant: 'destructive',
    };
  }
  if (confidencePercent > 30) {
    return {
      label: 'Suspicious',
      colorClass: 'text-amber-500',
      badgeVariant: 'warning',
    };
  }
  return {
    label: 'Clean',
    colorClass: 'text-green-500',
    badgeVariant: 'secondary',
  };
}

/**
 * Get analysis status label and color
 */
export function getAnalysisStatusInfo(status: string): {
  label: string;
  colorClass: string;
  isLoading: boolean;
} {
  switch (status) {
    case 'pending':
      return { label: 'Pending Analysis', colorClass: 'text-gray-500', isLoading: false };
    case 'analyzing':
      return { label: 'Analyzing...', colorClass: 'text-blue-500', isLoading: true };
    case 'completed':
      return { label: 'Analysis Complete', colorClass: 'text-green-500', isLoading: false };
    case 'error':
      return { label: 'Analysis Failed', colorClass: 'text-red-500', isLoading: false };
    case 'unable_to_analyze':
      return { label: 'Unable to Analyze', colorClass: 'text-amber-500', isLoading: false };
    default:
      return { label: 'Unknown', colorClass: 'text-gray-500', isLoading: false };
  }
}

/**
 * Format platform name for display
 */
export function formatPlatformName(platform: string): string {
  const platformMap: Record<string, string> = {
    'Epic Games': 'Epic',
    'Steam': 'Steam',
    'PlayStation Network': 'PSN',
    'Xbox Live': 'Xbox',
    'Nintendo Switch Online': 'Switch',
  };
  return platformMap[platform] || platform;
}
